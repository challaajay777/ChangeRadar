use changeradar_database::{Database, DatabaseError};
use changeradar_hashing::{hash_file, HashingError};
use changeradar_types::{ChangeKind, Confidence, FileEvent, FileRecord};
use changeradar_version_store::{VersionStore, VersionStoreError};
use changeradar_watcher::RawFsEvent;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum EventEngineError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Version store error: {0}")]
    VersionStore(#[from] VersionStoreError),
    #[error("Hashing error: {0}")]
    Hashing(#[from] HashingError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Folder not found: {0}")]
    FolderNotFound(String),
}

#[derive(Debug, Clone, Default)]
pub struct ReconciliationSummary {
    pub files_scanned: u64,
    pub files_created: u64,
    pub files_modified: u64,
    pub files_deleted: u64,
}

#[derive(Clone)]
pub struct EventProcessor {
    db: Database,
    store: VersionStore,
}

impl EventProcessor {
    pub fn new(db: Database, store: VersionStore) -> Self {
        Self { db, store }
    }

    /// Process a single raw filesystem event through classification, CAS versioning, and timeline recording.
    pub fn process_raw_event(
        &self,
        raw: RawFsEvent,
    ) -> Result<Option<FileEvent>, EventEngineError> {
        let folder = match self.db.get_folder(&raw.folder_id)? {
            Some(f) => f,
            None => return Ok(None),
        };

        let root_path = PathBuf::from(&folder.path);
        let path = raw.path;

        // Skip directories for file version tracking
        if path.is_dir() {
            return Ok(None);
        }

        let rel_path = match path.strip_prefix(&root_path) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => return Ok(None),
        };

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let existing_file = self.db.get_file_by_path(&folder.id, &rel_path)?;

        if !path.exists() {
            // File was deleted
            if let Some(file) = existing_file {
                if !file.is_deleted {
                    let mut updated = file.clone();
                    updated.is_deleted = true;
                    updated.updated_at = Utc::now();
                    self.db.upsert_file(&updated)?;

                    let event = FileEvent {
                        id: Uuid::new_v4().to_string(),
                        folder_id: folder.id,
                        file_id: Some(file.id),
                        event_type: ChangeKind::Deleted,
                        old_path: Some(rel_path.clone()),
                        new_path: rel_path,
                        old_hash: Some(file.current_hash),
                        new_hash: None,
                        file_size_diff: -(file.file_size as i64),
                        summary: format!("Deleted file '{}'", file.file_name),
                        confidence: Confidence::Exact,
                        created_at: Utc::now(),
                    };
                    self.db.record_event(&event)?;
                    return Ok(Some(event));
                }
            }
            return Ok(None);
        }

        // File exists on disk: compute hash
        let new_hash = match hash_file(&path) {
            Ok(h) => h,
            Err(e) => {
                warn!("Could not hash file {:?}: {}", path, e);
                return Ok(None);
            }
        };

        let metadata = match fs::metadata(&path) {
            Ok(m) => m,
            Err(_) => return Ok(None),
        };
        let new_size = metadata.len();

        match existing_file {
            Some(file) => {
                // File was already tracked
                if file.current_hash == new_hash && !file.is_deleted {
                    // Content unchanged (perhaps only touch / metadata touched)
                    return Ok(None);
                }

                let diff_bytes = (new_size as i64) - (file.file_size as i64);
                let was_restored_from_deleted = file.is_deleted;
                let kind = if was_restored_from_deleted {
                    ChangeKind::Restored
                } else {
                    ChangeKind::Modified
                };

                // Store new version in CAS
                self.store.store_file(&path, &file.id, kind, None)?;

                let mut updated_file = file.clone();
                updated_file.current_hash = new_hash.clone();
                updated_file.file_size = new_size;
                updated_file.is_deleted = false;
                updated_file.updated_at = Utc::now();
                self.db.upsert_file(&updated_file)?;

                let summary = if diff_bytes >= 0 {
                    format!("Modified (+{} bytes)", diff_bytes)
                } else {
                    format!("Modified ({} bytes)", diff_bytes)
                };

                let event = FileEvent {
                    id: Uuid::new_v4().to_string(),
                    folder_id: folder.id,
                    file_id: Some(file.id),
                    event_type: kind,
                    old_path: Some(rel_path.clone()),
                    new_path: rel_path,
                    old_hash: Some(file.current_hash),
                    new_hash: Some(new_hash),
                    file_size_diff: diff_bytes,
                    summary,
                    confidence: Confidence::Exact,
                    created_at: Utc::now(),
                };
                self.db.record_event(&event)?;
                Ok(Some(event))
            }
            None => {
                // New file created or moved/renamed from another location
                let file_id = Uuid::new_v4().to_string();

                let file_record = FileRecord {
                    id: file_id.clone(),
                    folder_id: folder.id.clone(),
                    relative_path: rel_path.clone(),
                    file_name: file_name.clone(),
                    extension,
                    file_size: new_size,
                    current_hash: new_hash.clone(),
                    is_deleted: false,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                self.db.upsert_file(&file_record)?;

                // Store initial version in CAS
                self.store
                    .store_file(&path, &file_id, ChangeKind::Created, None)?;

                let event = FileEvent {
                    id: Uuid::new_v4().to_string(),
                    folder_id: folder.id,
                    file_id: Some(file_id),
                    event_type: ChangeKind::Created,
                    old_path: None,
                    new_path: rel_path,
                    old_hash: None,
                    new_hash: Some(new_hash),
                    file_size_diff: new_size as i64,
                    summary: format!("Created '{}' ({} bytes)", file_name, new_size),
                    confidence: Confidence::Exact,
                    created_at: Utc::now(),
                };
                self.db.record_event(&event)?;
                Ok(Some(event))
            }
        }
    }

    /// Run full reconciliation scan across a monitored folder.
    /// Detects changes that occurred while ChangeRadar was stopped or due to missed watcher events.
    pub fn reconcile_folder(
        &self,
        folder_id: &str,
    ) -> Result<ReconciliationSummary, EventEngineError> {
        let folder = self
            .db
            .get_folder(folder_id)?
            .ok_or_else(|| EventEngineError::FolderNotFound(folder_id.to_string()))?;

        let root_path = PathBuf::from(&folder.path);
        if !root_path.exists() {
            return Ok(ReconciliationSummary::default());
        }

        let mut summary = ReconciliationSummary::default();
        let ignore_matcher = changeradar_watcher::ignore::IgnoreMatcher::new(&folder.ignore_patterns);

        // Map tracked files
        let existing_files = self.db.list_files(Some(folder_id), 1_000_000)?;
        let mut seen_paths = std::collections::HashSet::new();

        // Walk directory
        for entry in WalkDir::new(&root_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(&root_path) {
                if ignore_matcher.is_ignored(rel) {
                    continue;
                }
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                seen_paths.insert(rel_str);

                summary.files_scanned += 1;

                let raw = RawFsEvent {
                    folder_id: folder_id.to_string(),
                    path: path.to_path_buf(),
                    old_path: None,
                    change_kind: ChangeKind::Modified,
                };
                if let Ok(Some(ev)) = self.process_raw_event(raw) {
                    match ev.event_type {
                        ChangeKind::Created => summary.files_created += 1,
                        ChangeKind::Modified | ChangeKind::Restored => summary.files_modified += 1,
                        _ => {}
                    }
                }
            }
        }

        // Detect files that were deleted on disk while offline
        for file in existing_files {
            if !file.is_deleted && !seen_paths.contains(&file.relative_path) {
                let mut updated = file.clone();
                updated.is_deleted = true;
                updated.updated_at = Utc::now();
                self.db.upsert_file(&updated)?;

                let event = FileEvent {
                    id: Uuid::new_v4().to_string(),
                    folder_id: folder_id.to_string(),
                    file_id: Some(file.id),
                    event_type: ChangeKind::Deleted,
                    old_path: Some(file.relative_path.clone()),
                    new_path: file.relative_path,
                    old_hash: Some(file.current_hash),
                    new_hash: None,
                    file_size_diff: -(file.file_size as i64),
                    summary: format!("Reconciliation detected deleted file '{}'", file.file_name),
                    confidence: Confidence::Exact,
                    created_at: Utc::now(),
                };
                self.db.record_event(&event)?;
                summary.files_deleted += 1;
            }
        }

        info!(
            "Reconciliation complete for folder {}: scanned {}, created {}, modified {}, deleted {}",
            folder.display_name,
            summary.files_scanned,
            summary.files_created,
            summary.files_modified,
            summary.files_deleted
        );

        Ok(summary)
    }
}
