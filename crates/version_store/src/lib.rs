use changeradar_database::{Database, DatabaseError};
use changeradar_hashing::{hash_bytes, hash_file, HashingError};
use changeradar_types::{ChangeKind, ContentObject, FileVersion};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum VersionStoreError {
    #[error("I/O error in version store: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Hashing error: {0}")]
    Hashing(#[from] HashingError),
    #[error("Decompression failed: {0}")]
    Decompression(String),
    #[error("Compression failed: {0}")]
    Compression(String),
    #[error("Integrity check failed: expected hash {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("Content object not found on disk: {0}")]
    ObjectMissing(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub total_checked: usize,
    pub valid_count: usize,
    pub corrupted_count: usize,
    pub missing_count: usize,
    pub corrupted_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCollectionReport {
    pub deleted_objects_count: usize,
    pub reclaimed_bytes: u64,
}

#[derive(Clone)]
pub struct VersionStore {
    objects_dir: PathBuf,
    db: Database,
}

impl VersionStore {
    pub fn new<P: AsRef<Path>>(objects_dir: P, db: Database) -> Result<Self, VersionStoreError> {
        let dir = objects_dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        Ok(Self {
            objects_dir: dir,
            db,
        })
    }

    /// Return relative 2-tier subfolder path for an object hash: `objects/ab/cd/<hash>.zst`
    fn object_path(&self, hash: &str) -> PathBuf {
        let sub1 = if hash.len() >= 2 { &hash[0..2] } else { "xx" };
        let sub2 = if hash.len() >= 4 { &hash[2..4] } else { "yy" };
        self.objects_dir
            .join(sub1)
            .join(sub2)
            .join(format!("{}.zst", hash))
    }

    /// Stores a file's content into CAS with Zstandard compression and creates a new FileVersion record.
    /// If content already exists in CAS, deduplicates automatically by bumping reference count.
    pub fn store_file<P: AsRef<Path>>(
        &self,
        path: P,
        file_id: &str,
        change_kind: ChangeKind,
        notes: Option<String>,
    ) -> Result<FileVersion, VersionStoreError> {
        let path_ref = path.as_ref();
        let hash = hash_file(path_ref)?;
        let metadata = fs::metadata(path_ref)?;
        let raw_size = metadata.len();

        self.ensure_cas_object_from_file(path_ref, &hash, raw_size)?;
        self.record_new_version(file_id, &hash, raw_size, change_kind, notes)
    }

    /// Store arbitrary byte slice directly into CAS.
    pub fn store_bytes(
        &self,
        bytes: &[u8],
        file_id: &str,
        change_kind: ChangeKind,
        notes: Option<String>,
    ) -> Result<FileVersion, VersionStoreError> {
        let hash = hash_bytes(bytes);
        let raw_size = bytes.len() as u64;

        self.ensure_cas_object_from_bytes(bytes, &hash)?;
        self.record_new_version(file_id, &hash, raw_size, change_kind, notes)
    }

    fn ensure_cas_object_from_file(
        &self,
        path: &Path,
        hash: &str,
        raw_size: u64,
    ) -> Result<(), VersionStoreError> {
        let target_path = self.object_path(hash);

        if target_path.exists() {
            // Already present in CAS: increment ref count in DB
            let obj = ContentObject {
                content_hash: hash.to_string(),
                original_size: raw_size,
                compressed_size: fs::metadata(&target_path)?.len(),
                compression_algorithm: "zstd".to_string(),
                reference_count: 1,
                created_at: Utc::now(),
            };
            self.db.insert_content_object(&obj)?;
            return Ok(());
        }

        // Parent directories
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Stream and compress to temporary file first, then atomic rename
        let temp_path = target_path.with_extension(format!("tmp.{}", Uuid::new_v4()));
        {
            let src = File::open(path)?;
            let mut src_reader = BufReader::new(src);

            let dst = match File::create(&temp_path) {
                Ok(f) => f,
                Err(e) => return Err(VersionStoreError::Io(e)),
            };
            let dst_writer = BufWriter::new(dst);

            // Zstandard level 3 (high compression speed and great ratio)
            let mut encoder = match zstd::Encoder::new(dst_writer, 3) {
                Ok(enc) => enc,
                Err(e) => {
                    let _ = fs::remove_file(&temp_path);
                    return Err(VersionStoreError::Compression(e.to_string()));
                }
            };
            if let Err(e) = std::io::copy(&mut src_reader, &mut encoder) {
                let _ = fs::remove_file(&temp_path);
                return Err(VersionStoreError::Io(e));
            }
            if let Err(e) = encoder.finish() {
                let _ = fs::remove_file(&temp_path);
                return Err(VersionStoreError::Compression(e.to_string()));
            }
        }

        #[cfg(windows)]
        {
            if target_path.exists() {
                let _ = fs::remove_file(&temp_path);
            } else if let Err(e) = fs::rename(&temp_path, &target_path) {
                let _ = fs::remove_file(&temp_path);
                if !target_path.exists() {
                    return Err(VersionStoreError::Io(e));
                }
            }
        }
        #[cfg(not(windows))]
        {
            if let Err(e) = fs::rename(&temp_path, &target_path) {
                let _ = fs::remove_file(&temp_path);
                if !target_path.exists() {
                    return Err(VersionStoreError::Io(e));
                }
            }
        }

        let compressed_size = fs::metadata(&target_path)?.len();

        let obj = ContentObject {
            content_hash: hash.to_string(),
            original_size: raw_size,
            compressed_size,
            compression_algorithm: "zstd".to_string(),
            reference_count: 1,
            created_at: Utc::now(),
        };
        self.db.insert_content_object(&obj)?;
        Ok(())
    }

    fn ensure_cas_object_from_bytes(
        &self,
        bytes: &[u8],
        hash: &str,
    ) -> Result<(), VersionStoreError> {
        let target_path = self.object_path(hash);
        let raw_size = bytes.len() as u64;

        if target_path.exists() {
            let obj = ContentObject {
                content_hash: hash.to_string(),
                original_size: raw_size,
                compressed_size: fs::metadata(&target_path)?.len(),
                compression_algorithm: "zstd".to_string(),
                reference_count: 1,
                created_at: Utc::now(),
            };
            self.db.insert_content_object(&obj)?;
            return Ok(());
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let compressed = zstd::encode_all(bytes, 3)
            .map_err(|e| VersionStoreError::Compression(e.to_string()))?;

        let temp_path = target_path.with_extension(format!("tmp.{}", Uuid::new_v4()));
        if let Err(e) = fs::write(&temp_path, &compressed) {
            let _ = fs::remove_file(&temp_path);
            return Err(VersionStoreError::Io(e));
        }

        #[cfg(windows)]
        {
            if target_path.exists() {
                let _ = fs::remove_file(&temp_path);
            } else if let Err(e) = fs::rename(&temp_path, &target_path) {
                let _ = fs::remove_file(&temp_path);
                if !target_path.exists() {
                    return Err(VersionStoreError::Io(e));
                }
            }
        }
        #[cfg(not(windows))]
        {
            if let Err(e) = fs::rename(&temp_path, &target_path) {
                let _ = fs::remove_file(&temp_path);
                if !target_path.exists() {
                    return Err(VersionStoreError::Io(e));
                }
            }
        }

        let obj = ContentObject {
            content_hash: hash.to_string(),
            original_size: raw_size,
            compressed_size: compressed.len() as u64,
            compression_algorithm: "zstd".to_string(),
            reference_count: 1,
            created_at: Utc::now(),
        };
        self.db.insert_content_object(&obj)?;
        Ok(())
    }

    fn record_new_version(
        &self,
        file_id: &str,
        hash: &str,
        raw_size: u64,
        change_kind: ChangeKind,
        notes: Option<String>,
    ) -> Result<FileVersion, VersionStoreError> {
        let existing = self.db.list_versions_for_file(file_id)?;
        let next_version_number = existing.iter().map(|v| v.version_number).max().unwrap_or(0) + 1;

        let version = FileVersion {
            id: Uuid::new_v4().to_string(),
            file_id: file_id.to_string(),
            version_number: next_version_number,
            content_hash: hash.to_string(),
            file_size: raw_size,
            change_kind,
            notes,
            is_pinned: false,
            created_at: Utc::now(),
        };

        self.db.create_version(&version)?;
        Ok(version)
    }

    /// Retrieve and decompress historical version data into memory.
    pub fn get_version_bytes(&self, version_id: &str) -> Result<Vec<u8>, VersionStoreError> {
        let version = self
            .db
            .get_version(version_id)?
            .ok_or_else(|| VersionStoreError::VersionNotFound(version_id.to_string()))?;

        self.get_version_bytes_by_hash(&version.content_hash)
    }

    /// Retrieve and decompress raw content object by its BLAKE3 hash.
    pub fn get_version_bytes_by_hash(&self, hash: &str) -> Result<Vec<u8>, VersionStoreError> {
        let obj_path = self.object_path(hash);
        if !obj_path.exists() {
            return Err(VersionStoreError::ObjectMissing(hash.to_string()));
        }

        let compressed_data = fs::read(&obj_path)?;
        let decompressed = zstd::decode_all(compressed_data.as_slice())
            .map_err(|e| VersionStoreError::Decompression(e.to_string()))?;

        let actual_hash = hash_bytes(&decompressed);
        if actual_hash != hash {
            return Err(VersionStoreError::HashMismatch {
                expected: hash.to_string(),
                actual: actual_hash,
            });
        }

        Ok(decompressed)
    }

    /// Export a version directly to a target filesystem location.
    pub fn export_version<P: AsRef<Path>>(
        &self,
        version_id: &str,
        destination: P,
    ) -> Result<u64, VersionStoreError> {
        let bytes = self.get_version_bytes(version_id)?;
        let dest_ref = destination.as_ref();
        if let Some(parent) = dest_ref.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest_ref, &bytes)?;
        Ok(bytes.len() as u64)
    }

    /// Delete an individual version record. Decrements the reference count of the underlying CAS object.
    pub fn delete_version(&self, version_id: &str) -> Result<(), VersionStoreError> {
        if let Some(version) = self.db.get_version(version_id)? {
            self.db
                .decrement_content_object_ref(&version.content_hash)?;
            self.db.delete_version(version_id)?;
        }
        Ok(())
    }

    /// Pin or unpin a version.
    pub fn pin_version(&self, version_id: &str, pinned: bool) -> Result<(), VersionStoreError> {
        self.db.set_version_pinned(version_id, pinned)?;
        Ok(())
    }

    /// Run full integrity verification across all CAS content objects.
    pub fn verify_integrity(&self) -> Result<IntegrityReport, VersionStoreError> {
        let _stats = self.db.get_storage_stats()?;
        let mut report = IntegrityReport {
            total_checked: 0,
            valid_count: 0,
            corrupted_count: 0,
            missing_count: 0,
            corrupted_hashes: Vec::new(),
        };

        // Enumerate object files
        if !self.objects_dir.exists() {
            return Ok(report);
        }

        for entry in walkdir::WalkDir::new(&self.objects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("zst")
            {
                report.total_checked += 1;
                let file_name = entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let expected_hash = file_name.to_string();

                match fs::read(entry.path()) {
                    Ok(compressed) => match zstd::decode_all(compressed.as_slice()) {
                        Ok(decompressed) => {
                            let actual = hash_bytes(&decompressed);
                            if actual == expected_hash {
                                report.valid_count += 1;
                            } else {
                                report.corrupted_count += 1;
                                report.corrupted_hashes.push(expected_hash);
                            }
                        }
                        Err(_) => {
                            report.corrupted_count += 1;
                            report.corrupted_hashes.push(expected_hash);
                        }
                    },
                    Err(_) => {
                        report.missing_count += 1;
                    }
                }
            }
        }

        info!(
            "Integrity check complete: {} total, {} valid, {} corrupted",
            report.total_checked, report.valid_count, report.corrupted_count
        );
        Ok(report)
    }

    /// Garbage collection: Deletes unreferenced CAS objects (ref count == 0).
    /// Preserves pinned versions and active snapshots.
    pub fn garbage_collect(&self) -> Result<GarbageCollectionReport, VersionStoreError> {
        let unreferenced = self.db.list_unreferenced_content_hashes()?;
        let mut report = GarbageCollectionReport {
            deleted_objects_count: 0,
            reclaimed_bytes: 0,
        };

        for hash in unreferenced {
            let path = self.object_path(&hash);
            if path.exists() {
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                if let Err(e) = fs::remove_file(&path) {
                    warn!("Failed to delete CAS object {}: {}", hash, e);
                } else {
                    report.deleted_objects_count += 1;
                    report.reclaimed_bytes += size;
                }
            }
            self.db.delete_content_object(&hash)?;
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_file(db: &Database, file_id: &str) {
        let folder = changeradar_types::MonitoredFolder::new(
            format!("C:/test/{}", file_id),
            format!("Test {}", file_id),
        );
        let _ = db.add_folder(&folder);
        let file = changeradar_types::FileRecord {
            id: file_id.to_string(),
            folder_id: folder.id,
            relative_path: format!("{}.txt", file_id),
            file_name: format!("{}.txt", file_id),
            extension: "txt".to_string(),
            file_size: 100,
            current_hash: "initial_hash".to_string(),
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.upsert_file(&file).unwrap();
    }

    #[test]
    fn test_store_and_retrieve_version() {
        let tmp = tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        setup_test_file(&db, "file-123");
        let store = VersionStore::new(tmp.path().join("objects"), db).unwrap();

        let data = b"Hello, ChangeRadar historical version!";
        let ver = store
            .store_bytes(
                data,
                "file-123",
                ChangeKind::Created,
                Some("Test note".into()),
            )
            .unwrap();
        assert_eq!(ver.version_number, 1);

        let retrieved = store.get_version_bytes(&ver.id).unwrap();
        assert_eq!(retrieved, data);

        let report = store.verify_integrity().unwrap();
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.corrupted_count, 0);
    }

    #[test]
    fn test_deduplication() {
        let tmp = tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        setup_test_file(&db, "file-1");
        setup_test_file(&db, "file-2");
        let store = VersionStore::new(tmp.path().join("objects"), db).unwrap();

        let data = b"Identical file content across versions";
        let ver1 = store
            .store_bytes(data, "file-1", ChangeKind::Created, None)
            .unwrap();
        let ver2 = store
            .store_bytes(data, "file-2", ChangeKind::Created, None)
            .unwrap();

        assert_eq!(ver1.content_hash, ver2.content_hash);

        // Check CAS object ref count
        let obj = store
            .db
            .get_content_object(&ver1.content_hash)
            .unwrap()
            .unwrap();
        assert_eq!(obj.reference_count, 2);
    }
}
