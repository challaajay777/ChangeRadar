use changeradar_database::{Database, DatabaseError};
use changeradar_hashing::{hash_file, HashingError};
use changeradar_types::{ChangeKind, Confidence, FileEvent, RecoveryPreflight, RecoveryResult};
use changeradar_version_store::{VersionStore, VersionStoreError};
use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Version store error: {0}")]
    VersionStore(#[from] VersionStoreError),
    #[error("Hashing error: {0}")]
    Hashing(#[from] HashingError),
    #[error("I/O error during recovery: {0}")]
    Io(#[from] std::io::Error),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("File record not found: {0}")]
    FileNotFound(String),
    #[error("Folder record not found: {0}")]
    FolderNotFound(String),
    #[error("CAS object corrupted or missing: {0}")]
    CorruptCasObject(String),
    #[error("Restored file hash mismatch: expected {expected}, actual {actual}")]
    PostRestoreMismatch { expected: String, actual: String },
}

#[derive(Clone)]
pub struct RecoveryCenter {
    db: Database,
    store: VersionStore,
}

impl RecoveryCenter {
    pub fn new(db: Database, store: VersionStore) -> Self {
        Self { db, store }
    }

    /// Perform pre-flight safety inspection before any recovery action.
    /// Checks CAS integrity, destination conflicts, and determines if a pre-restore backup is needed.
    pub fn preflight_check(
        &self,
        version_id: &str,
        custom_destination: Option<&Path>,
    ) -> Result<RecoveryPreflight, RecoveryError> {
        let version = self
            .db
            .get_version(version_id)?
            .ok_or_else(|| RecoveryError::VersionNotFound(version_id.to_string()))?;

        let file = self
            .db
            .get_file(&version.file_id)?
            .ok_or_else(|| RecoveryError::FileNotFound(version.file_id.clone()))?;

        let folder = self
            .db
            .get_folder(&file.folder_id)?
            .ok_or_else(|| RecoveryError::FolderNotFound(file.folder_id.clone()))?;

        let destination_path = match custom_destination {
            Some(d) => d.to_path_buf(),
            None => PathBuf::from(&folder.path).join(&file.relative_path),
        };

        let destination_exists = destination_path.exists();
        let destination_size = if destination_exists {
            fs::metadata(&destination_path).ok().map(|m| m.len())
        } else {
            None
        };

        // Verify CAS content object can be read and decoded
        let cas_verified = self.store.get_version_bytes(version_id).is_ok();

        let mut warning = None;
        if destination_exists {
            warning = Some(format!(
                "Destination file already exists ({} bytes). It will be backed up into history before replacement.",
                destination_size.unwrap_or(0)
            ));
        }

        Ok(RecoveryPreflight {
            version_id: version.id,
            content_hash: version.content_hash,
            file_size: version.file_size,
            target_destination: destination_path.to_string_lossy().to_string(),
            destination_exists,
            destination_size,
            cas_object_verified: cas_verified,
            can_proceed: cas_verified,
            warning,
        })
    }

    /// Safely restore a historical file version.
    /// Follows the 8-step safety procedure:
    /// 1. Verify CAS object integrity
    /// 2. Validate destination
    /// 3. Back up existing file to CAS if present
    /// 4. Atomic write to temp file
    /// 5. Atomic rename to destination
    /// 6. Post-restore hash verification
    /// 7. Update database file record & audit event
    /// 8. Return structured recovery result
    pub fn restore_version(
        &self,
        version_id: &str,
        custom_destination: Option<&Path>,
    ) -> Result<RecoveryResult, RecoveryError> {
        let job_id = Uuid::new_v4().to_string();

        let version = self
            .db
            .get_version(version_id)?
            .ok_or_else(|| RecoveryError::VersionNotFound(version_id.to_string()))?;

        let file = self
            .db
            .get_file(&version.file_id)?
            .ok_or_else(|| RecoveryError::FileNotFound(version.file_id.clone()))?;

        let folder = self
            .db
            .get_folder(&file.folder_id)?
            .ok_or_else(|| RecoveryError::FolderNotFound(file.folder_id.clone()))?;

        let dest_path = match custom_destination {
            Some(d) => d.to_path_buf(),
            None => PathBuf::from(&folder.path).join(&file.relative_path),
        };

        // 1. Verify CAS object
        let content_bytes = self
            .store
            .get_version_bytes(version_id)
            .map_err(|_| RecoveryError::CorruptCasObject(version.content_hash.clone()))?;

        // 2. Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 3. If file currently exists at destination, safely back it up into CAS first
        let mut backup_version_id = None;
        if dest_path.exists() {
            if let Ok(existing_bytes) = fs::read(&dest_path) {
                let backup_ver = self.store.store_bytes(
                    &existing_bytes,
                    &file.id,
                    ChangeKind::Modified,
                    Some("Automatic safety backup created prior to restore".to_string()),
                )?;
                backup_version_id = Some(backup_ver.id);
            }
        }

        // 4. Atomic write: write to unique temp file in the same directory first
        let temp_file_name = format!(".cr_restore_{}_{}.tmp", &job_id[0..8], Uuid::new_v4());
        let temp_path = match dest_path.parent() {
            Some(parent) => parent.join(temp_file_name),
            None => PathBuf::from(temp_file_name),
        };
        {
            let mut tmp_file = match File::create(&temp_path) {
                Ok(f) => f,
                Err(e) => return Err(RecoveryError::Io(e)),
            };
            if let Err(e) = tmp_file.write_all(&content_bytes) {
                let _ = fs::remove_file(&temp_path);
                return Err(RecoveryError::Io(e));
            }
            if let Err(e) = tmp_file.flush() {
                let _ = fs::remove_file(&temp_path);
                return Err(RecoveryError::Io(e));
            }
        }

        // 5. Atomic replacement
        #[cfg(windows)]
        {
            if dest_path.exists() {
                let _ = fs::remove_file(&dest_path);
            }
            if let Err(e) = fs::rename(&temp_path, &dest_path) {
                let _ = fs::remove_file(&temp_path);
                return Err(RecoveryError::Io(e));
            }
        }
        #[cfg(not(windows))]
        {
            if let Err(e) = fs::rename(&temp_path, &dest_path) {
                let _ = fs::remove_file(&temp_path);
                return Err(RecoveryError::Io(e));
            }
        }

        // 6. Post-restore hash verification
        let restored_hash = match hash_file(&dest_path) {
            Ok(h) => h,
            Err(e) => {
                return Err(RecoveryError::Hashing(e));
            }
        };
        if restored_hash != version.content_hash {
            return Err(RecoveryError::PostRestoreMismatch {
                expected: version.content_hash,
                actual: restored_hash,
            });
        }

        // 7. Update database state
        let mut updated_file = file.clone();
        updated_file.current_hash = restored_hash.clone();
        updated_file.file_size = content_bytes.len() as u64;
        updated_file.is_deleted = false;
        updated_file.updated_at = Utc::now();
        self.db.upsert_file(&updated_file)?;

        // Record restore event
        let event = FileEvent {
            id: Uuid::new_v4().to_string(),
            folder_id: folder.id,
            file_id: Some(file.id),
            event_type: ChangeKind::Restored,
            old_path: None,
            new_path: file.relative_path.clone(),
            old_hash: None,
            new_hash: Some(restored_hash.clone()),
            file_size_diff: content_bytes.len() as i64,
            summary: format!(
                "Restored version #{} of '{}'",
                version.version_number, file.file_name
            ),
            confidence: Confidence::Exact,
            created_at: Utc::now(),
        };
        self.db.record_event(&event)?;

        self.db.record_audit_event(
            "RESTORE_VERSION",
            &format!(
                "Successfully restored version {} to {:?}",
                version_id, dest_path
            ),
        )?;

        info!(
            "Successfully restored version {} to {:?}",
            version_id, dest_path
        );

        Ok(RecoveryResult {
            job_id,
            destination_path: dest_path.to_string_lossy().to_string(),
            restored_hash,
            bytes_written: content_bytes.len() as u64,
            backup_version_id,
            success: true,
            message: format!(
                "Restored version #{} ({} bytes) successfully.",
                version.version_number,
                content_bytes.len()
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_safe_restore_pipeline() {
        let tmp = tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        let store = VersionStore::new(tmp.path().join("objects"), db.clone()).unwrap();

        let monitored_dir = tmp.path().join("monitored");
        fs::create_dir_all(&monitored_dir).unwrap();

        let folder = changeradar_types::MonitoredFolder::new(
            monitored_dir.to_string_lossy().to_string(),
            "Docs".to_string(),
        );
        db.add_folder(&folder).unwrap();

        let target_file_path = monitored_dir.join("report.txt");
        fs::write(&target_file_path, b"Version 1 original content").unwrap();

        let file_rec = changeradar_types::FileRecord {
            id: "f-1".to_string(),
            folder_id: folder.id.clone(),
            relative_path: "report.txt".to_string(),
            file_name: "report.txt".to_string(),
            extension: "txt".to_string(),
            file_size: 26,
            current_hash: "hash1".to_string(),
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.upsert_file(&file_rec).unwrap();

        let ver1 = store
            .store_bytes(
                b"Version 1 original content",
                "f-1",
                ChangeKind::Created,
                None,
            )
            .unwrap();

        // Overwrite file with new content
        fs::write(&target_file_path, b"Version 2 corrupted or overwritten").unwrap();

        let recovery = RecoveryCenter::new(db, store);
        let preflight = recovery.preflight_check(&ver1.id, None).unwrap();
        assert!(preflight.can_proceed);
        assert!(preflight.destination_exists);

        let result = recovery.restore_version(&ver1.id, None).unwrap();
        assert!(result.success);
        assert_eq!(result.restored_hash, ver1.content_hash);

        // Verify disk content was restored to version 1
        let disk_content = fs::read_to_string(&target_file_path).unwrap();
        assert_eq!(disk_content, "Version 1 original content");
    }
}
