use changeradar_database::{Database, DatabaseError};
use changeradar_recovery::{RecoveryCenter, RecoveryError};
use changeradar_types::{RecoveryResult, Snapshot, SnapshotItem};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Recovery error: {0}")]
    Recovery(#[from] RecoveryError),
    #[error("Snapshot not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub struct SnapshotComparison {
    pub added_files: Vec<SnapshotItem>,
    pub removed_files: Vec<SnapshotItem>,
    pub modified_files: Vec<(SnapshotItem, SnapshotItem)>, // (old, new)
}

#[derive(Clone)]
pub struct SnapshotEngine {
    db: Database,
    recovery: RecoveryCenter,
}

impl SnapshotEngine {
    pub fn new(db: Database, recovery: RecoveryCenter) -> Self {
        Self { db, recovery }
    }

    /// Create a consistent point-in-time snapshot of a folder (or all folders).
    pub fn create_snapshot(
        &self,
        folder_id: Option<&str>,
        name: &str,
        description: Option<&str>,
    ) -> Result<Snapshot, SnapshotError> {
        let files = self.db.list_files(folder_id, 1_000_000)?;
        let mut items = Vec::new();
        let mut total_bytes: u64 = 0;
        let snapshot_id = Uuid::new_v4().to_string();

        for file in files {
            if file.is_deleted {
                continue;
            }

            // Get latest version
            if let Some(latest_ver) = self.db.get_latest_version(&file.id)? {
                total_bytes += latest_ver.file_size;
                items.push(SnapshotItem {
                    snapshot_id: snapshot_id.clone(),
                    file_id: file.id.clone(),
                    relative_path: file.relative_path.clone(),
                    version_id: latest_ver.id,
                    content_hash: latest_ver.content_hash,
                    file_size: latest_ver.file_size,
                });
            }
        }

        let snapshot = Snapshot {
            id: snapshot_id,
            folder_id: folder_id.map(|s| s.to_string()),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            file_count: items.len() as u64,
            total_bytes,
            created_at: Utc::now(),
        };

        self.db.create_snapshot(&snapshot)?;
        self.db.add_snapshot_items(&items)?;

        self.db.record_audit_event(
            "SNAPSHOT_CREATED",
            &format!(
                "Created snapshot '{}' with {} files ({} bytes)",
                name,
                snapshot.file_count,
                snapshot.total_bytes
            ),
        )?;

        info!(
            "Created snapshot '{}' (id: {}) with {} files",
            name, snapshot.id, snapshot.file_count
        );
        Ok(snapshot)
    }

    /// Compare two snapshots to determine file additions, removals, and modifications.
    pub fn compare_snapshots(
        &self,
        snapshot_id_a: &str,
        snapshot_id_b: &str,
    ) -> Result<SnapshotComparison, SnapshotError> {
        let items_a = self.db.get_snapshot_items(snapshot_id_a)?;
        let items_b = self.db.get_snapshot_items(snapshot_id_b)?;

        let map_a: HashMap<String, SnapshotItem> = items_a
            .into_iter()
            .map(|item| (item.relative_path.clone(), item))
            .collect();

        let map_b: HashMap<String, SnapshotItem> = items_b
            .into_iter()
            .map(|item| (item.relative_path.clone(), item))
            .collect();

        let keys_a: HashSet<&String> = map_a.keys().collect();
        let keys_b: HashSet<&String> = map_b.keys().collect();

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        for &k in keys_b.difference(&keys_a) {
            added.push(map_b[k].clone());
        }

        for &k in keys_a.difference(&keys_b) {
            removed.push(map_a[k].clone());
        }

        for &k in keys_a.intersection(&keys_b) {
            let item_a = &map_a[k];
            let item_b = &map_b[k];
            if item_a.content_hash != item_b.content_hash {
                modified.push((item_a.clone(), item_b.clone()));
            }
        }

        Ok(SnapshotComparison {
            added_files: added,
            removed_files: removed,
            modified_files: modified,
        })
    }

    /// Restore all files from a snapshot using the recovery engine.
    pub fn restore_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<Vec<RecoveryResult>, SnapshotError> {
        let _snapshot = self
            .db
            .get_snapshot(snapshot_id)?
            .ok_or_else(|| SnapshotError::NotFound(snapshot_id.to_string()))?;

        let items = self.db.get_snapshot_items(snapshot_id)?;
        let mut results = Vec::new();

        for item in items {
            let res = self.recovery.restore_version(&item.version_id, None)?;
            results.push(res);
        }

        self.db.record_audit_event(
            "SNAPSHOT_RESTORED",
            &format!(
                "Restored {} files from snapshot {}",
                results.len(),
                snapshot_id
            ),
        )?;

        Ok(results)
    }
}
