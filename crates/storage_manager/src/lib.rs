use changeradar_database::{Database, DatabaseError};
use changeradar_types::StorageStats;
use changeradar_version_store::{GarbageCollectionReport, VersionStore, VersionStoreError};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum StorageManagerError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Version store error: {0}")]
    VersionStore(#[from] VersionStoreError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_versions_per_file: Option<usize>,
    pub max_age_days: Option<i64>,
    pub max_storage_bytes: Option<u64>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_versions_per_file: Some(50),
            max_age_days: Some(90),
            max_storage_bytes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPreview {
    pub candidate_versions_count: usize,
    pub estimated_reclaimed_bytes: u64,
    pub affected_files_count: usize,
    pub candidate_version_ids: Vec<String>,
}

#[derive(Clone)]
pub struct StorageManager {
    db: Database,
    store: VersionStore,
}

impl StorageManager {
    pub fn new(db: Database, store: VersionStore) -> Self {
        Self { db, store }
    }

    /// Retrieve full storage overview metrics.
    pub fn get_stats(&self) -> Result<StorageStats, StorageManagerError> {
        let stats = self.db.get_storage_stats()?;
        Ok(stats)
    }

    /// Preview which versions would be removed by a retention policy, WITHOUT deleting anything.
    /// Pinned versions are ALWAYS protected and excluded from candidates.
    pub fn preview_cleanup(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<CleanupPreview, StorageManagerError> {
        let files = self.db.list_files(None, 100_000)?;
        let mut candidate_ids = Vec::new();
        let mut estimated_bytes: u64 = 0;
        let mut affected_files = 0;

        let cutoff_time = policy
            .max_age_days
            .map(|days| Utc::now() - Duration::days(days));

        for file in files {
            let mut versions = self.db.list_versions_for_file(&file.id)?;
            // Sort ascending by version number (oldest first)
            versions.sort_by_key(|v| v.version_number);

            let mut file_affected = false;
            let total_vers = versions.len();

            for (idx, ver) in versions.iter().enumerate() {
                // NEVER delete pinned versions
                if ver.is_pinned {
                    continue;
                }

                // Never delete the latest version of an active file
                if idx == total_vers - 1 && !file.is_deleted {
                    continue;
                }

                let mut should_delete = false;

                // 1. Check max versions per file rule
                if let Some(max_vers) = policy.max_versions_per_file {
                    if total_vers > max_vers && idx < (total_vers - max_vers) {
                        should_delete = true;
                    }
                }

                // 2. Check age rule
                if let Some(cutoff) = cutoff_time {
                    if ver.created_at < cutoff {
                        should_delete = true;
                    }
                }

                if should_delete {
                    candidate_ids.push(ver.id.clone());
                    estimated_bytes += ver.file_size;
                    file_affected = true;
                }
            }

            if file_affected {
                affected_files += 1;
            }
        }

        Ok(CleanupPreview {
            candidate_versions_count: candidate_ids.len(),
            estimated_reclaimed_bytes: estimated_bytes,
            affected_files_count: affected_files,
            candidate_version_ids: candidate_ids,
        })
    }

    /// Execute the retention cleanup based on the preview.
    /// Safely deletes the unpinned version records and executes garbage collection on unreferenced CAS objects.
    pub fn execute_cleanup(
        &self,
        preview: &CleanupPreview,
    ) -> Result<GarbageCollectionReport, StorageManagerError> {
        info!(
            "Executing cleanup of {} candidate versions",
            preview.candidate_versions_count
        );

        for id in &preview.candidate_version_ids {
            self.store.delete_version(id)?;
        }

        // Reclaim disk space from orphaned CAS objects
        let gc_report = self.store.garbage_collect()?;
        info!(
            "Garbage collection reclaimed {} bytes across {} objects",
            gc_report.reclaimed_bytes, gc_report.deleted_objects_count
        );

        self.db.record_audit_event(
            "STORAGE_CLEANUP",
            &format!(
                "Pruned {} versions, reclaimed {} bytes",
                preview.candidate_versions_count, gc_report.reclaimed_bytes
            ),
        )?;

        Ok(gc_report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_stats_and_retention_preview() {
        let tmp = tempdir().unwrap();
        let db = Database::in_memory().unwrap();
        let store = VersionStore::new(tmp.path().join("objects"), db.clone()).unwrap();
        let manager = StorageManager::new(db, store);

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.total_objects, 0);

        let policy = RetentionPolicy::default();
        let preview = manager.preview_cleanup(&policy).unwrap();
        assert_eq!(preview.candidate_versions_count, 0);
    }
}
