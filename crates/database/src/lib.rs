pub mod schema;

use changeradar_types::*;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Entity not found: {0}")]
    NotFound(String),
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: String,
}

impl Database {
    /// Open or create database at given filesystem path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let p_str = path.as_ref().to_string_lossy().to_string();
        let conn = Connection::open(&path)?;
        schema::initialize_database(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: p_str,
        })
    }

    /// Open in-memory database for testing.
    pub fn in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        schema::initialize_database(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: ":memory:".to_string(),
        })
    }

    pub fn get_db_path(&self) -> &str {
        &self.db_path
    }

    // ==========================================
    // MONITORED FOLDERS
    // ==========================================

    pub fn add_folder(&self, folder: &MonitoredFolder) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let ignore_json = serde_json::to_string(&folder.ignore_patterns)?;
        conn.execute(
            "INSERT INTO monitored_folders (
                id, path, display_name, is_active, scan_interval_secs, ignore_patterns, max_storage_bytes, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                folder.id,
                folder.path,
                folder.display_name,
                if folder.is_active { 1 } else { 0 },
                folder.scan_interval_secs as i64,
                ignore_json,
                folder.max_storage_bytes.map(|v| v as i64),
                folder.created_at.to_rfc3339(),
                folder.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_folders(&self) -> Result<Vec<MonitoredFolder>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, path, display_name, is_active, scan_interval_secs, ignore_patterns, max_storage_bytes, created_at, updated_at
             FROM monitored_folders ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let ignore_str: String = row.get(5)?;
            let ignore_patterns: Vec<String> =
                serde_json::from_str(&ignore_str).unwrap_or_default();
            let created_str: String = row.get(7)?;
            let updated_str: String = row.get(8)?;
            let max_bytes: Option<i64> = row.get(6)?;

            Ok(MonitoredFolder {
                id: row.get(0)?,
                path: row.get(1)?,
                display_name: row.get(2)?,
                is_active: row.get::<_, i32>(3)? != 0,
                scan_interval_secs: row.get::<_, i64>(4)? as u64,
                ignore_patterns,
                max_storage_bytes: max_bytes.map(|v| v as u64),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut folders = Vec::new();
        for r in rows {
            folders.push(r?);
        }
        Ok(folders)
    }

    pub fn get_folder(&self, id: &str) -> Result<Option<MonitoredFolder>, DatabaseError> {
        let folders = self.list_folders()?;
        Ok(folders.into_iter().find(|f| f.id == id))
    }

    pub fn get_folder_by_path(&self, path: &str) -> Result<Option<MonitoredFolder>, DatabaseError> {
        let folders = self.list_folders()?;
        Ok(folders.into_iter().find(|f| f.path == path))
    }

    pub fn set_folder_active(&self, id: &str, active: bool) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE monitored_folders SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
            params![if active { 1 } else { 0 }, now, id],
        )?;
        Ok(())
    }

    pub fn delete_folder(&self, id: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM monitored_folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==========================================
    // FILES
    // ==========================================

    pub fn upsert_file(&self, file: &FileRecord) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO files (
                id, folder_id, relative_path, file_name, extension, file_size, current_hash, is_deleted, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(folder_id, relative_path) DO UPDATE SET
                file_size = excluded.file_size,
                current_hash = excluded.current_hash,
                is_deleted = excluded.is_deleted,
                updated_at = excluded.updated_at",
            params![
                file.id,
                file.folder_id,
                file.relative_path,
                file.file_name,
                file.extension,
                file.file_size as i64,
                file.current_hash,
                if file.is_deleted { 1 } else { 0 },
                file.created_at.to_rfc3339(),
                file.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_file(&self, id: &str) -> Result<Option<FileRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, relative_path, file_name, extension, file_size, current_hash, is_deleted, created_at, updated_at
             FROM files WHERE id = ?1",
        )?;

        let res = stmt
            .query_row(params![id], |row| {
                let created_str: String = row.get(8)?;
                let updated_str: String = row.get(9)?;
                Ok(FileRecord {
                    id: row.get(0)?,
                    folder_id: row.get(1)?,
                    relative_path: row.get(2)?,
                    file_name: row.get(3)?,
                    extension: row.get(4)?,
                    file_size: row.get::<_, i64>(5)? as u64,
                    current_hash: row.get(6)?,
                    is_deleted: row.get::<_, i32>(7)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(res)
    }

    pub fn get_file_by_path(
        &self,
        folder_id: &str,
        relative_path: &str,
    ) -> Result<Option<FileRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, relative_path, file_name, extension, file_size, current_hash, is_deleted, created_at, updated_at
             FROM files WHERE folder_id = ?1 AND relative_path = ?2",
        )?;

        let res = stmt
            .query_row(params![folder_id, relative_path], |row| {
                let created_str: String = row.get(8)?;
                let updated_str: String = row.get(9)?;
                Ok(FileRecord {
                    id: row.get(0)?,
                    folder_id: row.get(1)?,
                    relative_path: row.get(2)?,
                    file_name: row.get(3)?,
                    extension: row.get(4)?,
                    file_size: row.get::<_, i64>(5)? as u64,
                    current_hash: row.get(6)?,
                    is_deleted: row.get::<_, i32>(7)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(res)
    }

    pub fn list_files(
        &self,
        folder_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FileRecord>, DatabaseError> {
        let conn = self.conn.lock();
        let query = if let Some(fid) = folder_id {
            format!(
                "SELECT id, folder_id, relative_path, file_name, extension, file_size, current_hash, is_deleted, created_at, updated_at
                 FROM files WHERE folder_id = '{}' ORDER BY updated_at DESC LIMIT {}",
                fid, limit
            )
        } else {
            format!(
                "SELECT id, folder_id, relative_path, file_name, extension, file_size, current_hash, is_deleted, created_at, updated_at
                 FROM files ORDER BY updated_at DESC LIMIT {}",
                limit
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(8)?;
            let updated_str: String = row.get(9)?;
            Ok(FileRecord {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                relative_path: row.get(2)?,
                file_name: row.get(3)?,
                extension: row.get(4)?,
                file_size: row.get::<_, i64>(5)? as u64,
                current_hash: row.get(6)?,
                is_deleted: row.get::<_, i32>(7)? != 0,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut files = Vec::new();
        for r in rows {
            files.push(r?);
        }
        Ok(files)
    }

    pub fn count_tracked_files(&self) -> Result<u64, DatabaseError> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM files WHERE is_deleted = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    // ==========================================
    // CONTENT-ADDRESSED STORAGE (CAS)
    // ==========================================

    pub fn insert_content_object(&self, obj: &ContentObject) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO content_objects (
                content_hash, original_size, compressed_size, compression_algorithm, reference_count, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(content_hash) DO UPDATE SET
                reference_count = reference_count + 1",
            params![
                obj.content_hash,
                obj.original_size as i64,
                obj.compressed_size as i64,
                obj.compression_algorithm,
                obj.reference_count as i64,
                obj.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_content_object(&self, hash: &str) -> Result<Option<ContentObject>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT content_hash, original_size, compressed_size, compression_algorithm, reference_count, created_at
             FROM content_objects WHERE content_hash = ?1",
        )?;

        let res = stmt
            .query_row(params![hash], |row| {
                let created_str: String = row.get(5)?;
                Ok(ContentObject {
                    content_hash: row.get(0)?,
                    original_size: row.get::<_, i64>(1)? as u64,
                    compressed_size: row.get::<_, i64>(2)? as u64,
                    compression_algorithm: row.get(3)?,
                    reference_count: row.get::<_, i64>(4)? as u32,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(res)
    }

    pub fn decrement_content_object_ref(&self, hash: &str) -> Result<u32, DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE content_objects SET reference_count = MAX(0, reference_count - 1) WHERE content_hash = ?1",
            params![hash],
        )?;
        let count: i64 = conn
            .query_row(
                "SELECT reference_count FROM content_objects WHERE content_hash = ?1",
                params![hash],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count as u32)
    }

    pub fn list_unreferenced_content_hashes(&self) -> Result<Vec<String>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt =
            conn.prepare("SELECT content_hash FROM content_objects WHERE reference_count = 0")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut res = Vec::new();
        for r in rows {
            res.push(r?);
        }
        Ok(res)
    }

    pub fn delete_content_object(&self, hash: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM content_objects WHERE content_hash = ?1",
            params![hash],
        )?;
        Ok(())
    }

    // ==========================================
    // FILE VERSIONS
    // ==========================================

    pub fn create_version(&self, version: &FileVersion) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO file_versions (
                id, file_id, version_number, content_hash, file_size, change_kind, notes, is_pinned, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                version.id,
                version.file_id,
                version.version_number as i64,
                version.content_hash,
                version.file_size as i64,
                version.change_kind.to_string(),
                version.notes,
                if version.is_pinned { 1 } else { 0 },
                version.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn get_version(&self, id: &str) -> Result<Option<FileVersion>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, version_number, content_hash, file_size, change_kind, notes, is_pinned, created_at
             FROM file_versions WHERE id = ?1",
        )?;

        let res = stmt
            .query_row(params![id], |row| {
                let kind_str: String = row.get(5)?;
                let created_str: String = row.get(8)?;
                Ok(FileVersion {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    version_number: row.get::<_, i64>(2)? as u32,
                    content_hash: row.get(3)?,
                    file_size: row.get::<_, i64>(4)? as u64,
                    change_kind: kind_str.parse().unwrap_or(ChangeKind::Modified),
                    notes: row.get(6)?,
                    is_pinned: row.get::<_, i32>(7)? != 0,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            })
            .optional()?;
        Ok(res)
    }

    pub fn list_versions_for_file(&self, file_id: &str) -> Result<Vec<FileVersion>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, file_id, version_number, content_hash, file_size, change_kind, notes, is_pinned, created_at
             FROM file_versions WHERE file_id = ?1 ORDER BY version_number DESC",
        )?;

        let rows = stmt.query_map(params![file_id], |row| {
            let kind_str: String = row.get(5)?;
            let created_str: String = row.get(8)?;
            Ok(FileVersion {
                id: row.get(0)?,
                file_id: row.get(1)?,
                version_number: row.get::<_, i64>(2)? as u32,
                content_hash: row.get(3)?,
                file_size: row.get::<_, i64>(4)? as u64,
                change_kind: kind_str.parse().unwrap_or(ChangeKind::Modified),
                notes: row.get(6)?,
                is_pinned: row.get::<_, i32>(7)? != 0,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_latest_version(&self, file_id: &str) -> Result<Option<FileVersion>, DatabaseError> {
        let versions = self.list_versions_for_file(file_id)?;
        Ok(versions.into_iter().next())
    }

    pub fn set_version_pinned(&self, id: &str, pinned: bool) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE file_versions SET is_pinned = ?1 WHERE id = ?2",
            params![if pinned { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn update_version_note(&self, id: &str, note: Option<String>) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE file_versions SET notes = ?1 WHERE id = ?2",
            params![note, id],
        )?;
        Ok(())
    }

    pub fn count_total_versions(&self) -> Result<u64, DatabaseError> {
        let conn = self.conn.lock();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM file_versions", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    pub fn delete_version(&self, id: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM file_versions WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==========================================
    // FILE EVENTS & TIMELINE
    // ==========================================

    pub fn record_event(&self, event: &FileEvent) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO file_events (
                id, folder_id, file_id, event_type, old_path, new_path, old_hash, new_hash, file_size_diff, summary, confidence, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                event.id,
                event.folder_id,
                event.file_id,
                event.event_type.to_string(),
                event.old_path,
                event.new_path,
                event.old_hash,
                event.new_hash,
                event.file_size_diff,
                event.summary,
                event.confidence.to_string(),
                event.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_events(
        &self,
        folder_id: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<FileEvent>, DatabaseError> {
        let conn = self.conn.lock();
        let query = if let Some(fid) = folder_id {
            format!(
                "SELECT id, folder_id, file_id, event_type, old_path, new_path, old_hash, new_hash, file_size_diff, summary, confidence, created_at
                 FROM file_events WHERE folder_id = '{}' ORDER BY created_at DESC LIMIT {} OFFSET {}",
                fid, limit, offset
            )
        } else {
            format!(
                "SELECT id, folder_id, file_id, event_type, old_path, new_path, old_hash, new_hash, file_size_diff, summary, confidence, created_at
                 FROM file_events ORDER BY created_at DESC LIMIT {} OFFSET {}",
                limit, offset
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let event_type_str: String = row.get(3)?;
            let confidence_str: String = row.get(10)?;
            let created_str: String = row.get(11)?;
            Ok(FileEvent {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                file_id: row.get(2)?,
                event_type: event_type_str.parse().unwrap_or(ChangeKind::Modified),
                old_path: row.get(4)?,
                new_path: row.get(5)?,
                old_hash: row.get(6)?,
                new_hash: row.get(7)?,
                file_size_diff: row.get(8)?,
                summary: row.get(9)?,
                confidence: confidence_str.parse().unwrap_or(Confidence::Exact),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn count_events_today(&self) -> Result<u64, DatabaseError> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM file_events WHERE date(created_at) = date('now')",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    // ==========================================
    // SNAPSHOTS
    // ==========================================

    pub fn create_snapshot(&self, snapshot: &Snapshot) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO snapshots (id, folder_id, name, description, file_count, total_bytes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snapshot.id,
                snapshot.folder_id,
                snapshot.name,
                snapshot.description,
                snapshot.file_count as i64,
                snapshot.total_bytes as i64,
                snapshot.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn add_snapshot_items(&self, items: &[SnapshotItem]) -> Result<(), DatabaseError> {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO snapshot_items (snapshot_id, file_id, relative_path, version_id, content_hash, file_size)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for item in items {
                stmt.execute(params![
                    item.snapshot_id,
                    item.file_id,
                    item.relative_path,
                    item.version_id,
                    item.content_hash,
                    item.file_size as i64
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_snapshots(&self, folder_id: Option<&str>) -> Result<Vec<Snapshot>, DatabaseError> {
        let conn = self.conn.lock();
        let query = if let Some(fid) = folder_id {
            format!(
                "SELECT id, folder_id, name, description, file_count, total_bytes, created_at
                 FROM snapshots WHERE folder_id = '{}' ORDER BY created_at DESC",
                fid
            )
        } else {
            "SELECT id, folder_id, name, description, file_count, total_bytes, created_at
             FROM snapshots ORDER BY created_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(6)?;
            Ok(Snapshot {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                file_count: row.get::<_, i64>(4)? as u64,
                total_bytes: row.get::<_, i64>(5)? as u64,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_snapshot(&self, id: &str) -> Result<Option<Snapshot>, DatabaseError> {
        let snapshots = self.list_snapshots(None)?;
        Ok(snapshots.into_iter().find(|s| s.id == id))
    }

    pub fn get_snapshot_items(
        &self,
        snapshot_id: &str,
    ) -> Result<Vec<SnapshotItem>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT snapshot_id, file_id, relative_path, version_id, content_hash, file_size
             FROM snapshot_items WHERE snapshot_id = ?1",
        )?;

        let rows = stmt.query_map(params![snapshot_id], |row| {
            Ok(SnapshotItem {
                snapshot_id: row.get(0)?,
                file_id: row.get(1)?,
                relative_path: row.get(2)?,
                version_id: row.get(3)?,
                content_hash: row.get(4)?,
                file_size: row.get::<_, i64>(5)? as u64,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_snapshot(&self, id: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM snapshots WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ==========================================
    // NOTIFICATIONS
    // ==========================================

    pub fn create_notification(&self, item: &NotificationItem) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO notifications (id, rule_id, title, message, priority, is_read, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.id,
                item.rule_id,
                item.title,
                item.message,
                item.priority.to_string(),
                if item.is_read { 1 } else { 0 },
                item.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_notifications(&self, limit: usize) -> Result<Vec<NotificationItem>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, rule_id, title, message, priority, is_read, created_at
             FROM notifications ORDER BY created_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let priority_str: String = row.get(4)?;
            let created_str: String = row.get(6)?;
            Ok(NotificationItem {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                title: row.get(2)?,
                message: row.get(3)?,
                priority: priority_str.parse().unwrap_or(Priority::Normal),
                is_read: row.get::<_, i32>(5)? != 0,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn mark_notification_read(&self, id: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE notifications SET is_read = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn mark_all_notifications_read(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        conn.execute("UPDATE notifications SET is_read = 1 WHERE is_read = 0", [])?;
        Ok(())
    }

    // ==========================================
    // NOTIFICATION RULES
    // ==========================================

    pub fn add_notification_rule(&self, rule: &NotificationRule) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let events_json = serde_json::to_string(&rule.event_types)?;
        conn.execute(
            "INSERT INTO notification_rules (
                id, folder_id, name, event_types, path_pattern, min_size_bytes, priority, is_active, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                rule.id,
                rule.folder_id,
                rule.name,
                events_json,
                rule.path_pattern,
                rule.min_size_bytes.map(|v| v as i64),
                rule.priority.to_string(),
                if rule.is_active { 1 } else { 0 },
                rule.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_notification_rules(&self) -> Result<Vec<NotificationRule>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, name, event_types, path_pattern, min_size_bytes, priority, is_active, created_at
             FROM notification_rules ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let events_str: String = row.get(3)?;
            let event_types: Vec<ChangeKind> =
                serde_json::from_str(&events_str).unwrap_or_default();
            let priority_str: String = row.get(6)?;
            let created_str: String = row.get(8)?;
            let min_bytes: Option<i64> = row.get(5)?;

            Ok(NotificationRule {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                name: row.get(2)?,
                event_types,
                path_pattern: row.get(4)?,
                min_size_bytes: min_bytes.map(|v| v as u64),
                priority: priority_str.parse().unwrap_or(Priority::Normal),
                is_active: row.get::<_, i32>(7)? != 0,
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // ==========================================
    // SETTINGS
    // ==========================================

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let res = stmt.query_row(params![key], |row| row.get(0)).optional()?;
        Ok(res)
    }

    // ==========================================
    // AUDIT EVENTS
    // ==========================================

    pub fn record_audit_event(&self, action: &str, details: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock();
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO audit_events (id, action, details, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, action, details, now],
        )?;
        Ok(())
    }

    // ==========================================
    // STORAGE METRICS & DIAGNOSTICS
    // ==========================================

    pub fn get_storage_stats(&self) -> Result<StorageStats, DatabaseError> {
        let conn = self.conn.lock();
        let (total_objects, comp_bytes, raw_bytes): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(compressed_size), 0), COALESCE(SUM(original_size), 0) FROM content_objects",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        let pinned: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM file_versions WHERE is_pinned = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let db_bytes = std::fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        let ratio = if comp_bytes > 0 {
            (raw_bytes as f64) / (comp_bytes as f64)
        } else {
            1.0
        };

        Ok(StorageStats {
            total_objects: total_objects as u64,
            compressed_bytes: comp_bytes as u64,
            raw_bytes: raw_bytes as u64,
            pinned_versions: pinned as u64,
            deduplication_ratio: (ratio * 100.0).round() / 100.0,
            database_bytes: db_bytes,
        })
    }

    pub fn run_integrity_check(&self) -> Result<String, DatabaseError> {
        let conn = self.conn.lock();
        let res: String = conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_init_and_folders() {
        let db = Database::in_memory().unwrap();
        let folder = MonitoredFolder::new("C:/test/path".into(), "Test Folder".into());
        db.add_folder(&folder).unwrap();

        let list = db.list_folders().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].display_name, "Test Folder");

        let fetched = db.get_folder(&folder.id).unwrap().unwrap();
        assert_eq!(fetched.path, "C:/test/path");
    }

    #[test]
    fn test_file_versioning_and_cas() {
        let db = Database::in_memory().unwrap();
        let folder = MonitoredFolder::new("C:/test/path".into(), "Test".into());
        db.add_folder(&folder).unwrap();

        let hash = "abc1234567890defabc1234567890defabc1234567890defabc1234567890def";
        let cas_obj = ContentObject {
            content_hash: hash.to_string(),
            original_size: 1024,
            compressed_size: 400,
            compression_algorithm: "zstd".to_string(),
            reference_count: 1,
            created_at: Utc::now(),
        };
        db.insert_content_object(&cas_obj).unwrap();

        let file = FileRecord {
            id: "file-1".to_string(),
            folder_id: folder.id.clone(),
            relative_path: "docs/spec.txt".to_string(),
            file_name: "spec.txt".to_string(),
            extension: "txt".to_string(),
            file_size: 1024,
            current_hash: hash.to_string(),
            is_deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.upsert_file(&file).unwrap();

        let version = FileVersion {
            id: "ver-1".to_string(),
            file_id: file.id.clone(),
            version_number: 1,
            content_hash: hash.to_string(),
            file_size: 1024,
            change_kind: ChangeKind::Created,
            notes: Some("Initial revision".to_string()),
            is_pinned: false,
            created_at: Utc::now(),
        };
        db.create_version(&version).unwrap();

        let versions = db.list_versions_for_file(&file.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version_number, 1);
        assert_eq!(versions[0].notes.as_deref(), Some("Initial revision"));
    }
}
