use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: i32 = 1;

pub fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;

        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
        ",
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        apply_v1_migration(conn)?;
    }

    Ok(())
}

fn apply_v1_migration(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Monitored Folders
        CREATE TABLE IF NOT EXISTS monitored_folders (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            scan_interval_secs INTEGER NOT NULL DEFAULT 300,
            ignore_patterns TEXT NOT NULL DEFAULT '[]',
            max_storage_bytes INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Files metadata
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            folder_id TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            extension TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            current_hash TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE CASCADE,
            UNIQUE(folder_id, relative_path)
        );
        CREATE INDEX IF NOT EXISTS idx_files_folder_id ON files(folder_id);
        CREATE INDEX IF NOT EXISTS idx_files_relative_path ON files(relative_path);
        CREATE INDEX IF NOT EXISTS idx_files_current_hash ON files(current_hash);

        -- Content-Addressed Storage (CAS) objects
        CREATE TABLE IF NOT EXISTS content_objects (
            content_hash TEXT PRIMARY KEY,
            original_size INTEGER NOT NULL,
            compressed_size INTEGER NOT NULL,
            compression_algorithm TEXT NOT NULL DEFAULT 'zstd',
            reference_count INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        -- File Versions history
        CREATE TABLE IF NOT EXISTS file_versions (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL,
            version_number INTEGER NOT NULL,
            content_hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            change_kind TEXT NOT NULL,
            notes TEXT,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (content_hash) REFERENCES content_objects(content_hash),
            UNIQUE(file_id, version_number)
        );
        CREATE INDEX IF NOT EXISTS idx_file_versions_file_id ON file_versions(file_id);
        CREATE INDEX IF NOT EXISTS idx_file_versions_hash ON file_versions(content_hash);
        CREATE INDEX IF NOT EXISTS idx_file_versions_created_at ON file_versions(created_at);

        -- File Events (Timeline & Audit)
        CREATE TABLE IF NOT EXISTS file_events (
            id TEXT PRIMARY KEY,
            folder_id TEXT NOT NULL,
            file_id TEXT,
            event_type TEXT NOT NULL,
            old_path TEXT,
            new_path TEXT NOT NULL,
            old_hash TEXT,
            new_hash TEXT,
            file_size_diff INTEGER NOT NULL DEFAULT 0,
            summary TEXT NOT NULL,
            confidence TEXT NOT NULL DEFAULT 'EXACT',
            created_at TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_file_events_folder ON file_events(folder_id);
        CREATE INDEX IF NOT EXISTS idx_file_events_created ON file_events(created_at);
        CREATE INDEX IF NOT EXISTS idx_file_events_type ON file_events(event_type);

        -- Snapshots
        CREATE TABLE IF NOT EXISTS snapshots (
            id TEXT PRIMARY KEY,
            folder_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            file_count INTEGER NOT NULL DEFAULT 0,
            total_bytes INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS snapshot_items (
            snapshot_id TEXT NOT NULL,
            file_id TEXT NOT NULL,
            relative_path TEXT NOT NULL,
            version_id TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            PRIMARY KEY (snapshot_id, file_id),
            FOREIGN KEY (snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE
        );

        -- Comparison Results Cache
        CREATE TABLE IF NOT EXISTS comparison_results (
            id TEXT PRIMARY KEY,
            version_a_id TEXT NOT NULL,
            version_b_id TEXT NOT NULL,
            comparator_type TEXT NOT NULL,
            summary TEXT NOT NULL,
            diff_data_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(version_a_id, version_b_id, comparator_type)
        );

        -- Notification Rules
        CREATE TABLE IF NOT EXISTS notification_rules (
            id TEXT PRIMARY KEY,
            folder_id TEXT,
            name TEXT NOT NULL,
            event_types TEXT NOT NULL,
            path_pattern TEXT,
            min_size_bytes INTEGER,
            priority TEXT NOT NULL DEFAULT 'NORMAL',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE CASCADE
        );

        -- Notifications
        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            rule_id TEXT,
            title TEXT NOT NULL,
            message TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'NORMAL',
            is_read INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (rule_id) REFERENCES notification_rules(id) ON DELETE SET NULL
        );
        CREATE INDEX IF NOT EXISTS idx_notifications_unread ON notifications(is_read, created_at);

        -- File Notes & Pinned Versions
        CREATE TABLE IF NOT EXISTS file_notes (
            id TEXT PRIMARY KEY,
            file_id TEXT NOT NULL,
            version_id TEXT,
            note TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS pinned_versions (
            version_id TEXT PRIMARY KEY,
            reason TEXT,
            pinned_at TEXT NOT NULL,
            FOREIGN KEY (version_id) REFERENCES file_versions(id) ON DELETE CASCADE
        );

        -- Scan Jobs
        CREATE TABLE IF NOT EXISTS scan_jobs (
            id TEXT PRIMARY KEY,
            folder_id TEXT NOT NULL,
            status TEXT NOT NULL,
            files_scanned INTEGER NOT NULL DEFAULT 0,
            changes_detected INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE CASCADE
        );

        -- Recovery Jobs
        CREATE TABLE IF NOT EXISTS recovery_jobs (
            id TEXT PRIMARY KEY,
            target_path TEXT NOT NULL,
            destination_path TEXT NOT NULL,
            version_id TEXT NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            created_at TEXT NOT NULL
        );

        -- Audit Events
        CREATE TABLE IF NOT EXISTS audit_events (
            id TEXT PRIMARY KEY,
            action TEXT NOT NULL,
            details TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at);

        -- Plugins
        CREATE TABLE IF NOT EXISTS plugins (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT,
            capabilities TEXT NOT NULL,
            is_enabled INTEGER NOT NULL DEFAULT 1,
            installed_at TEXT NOT NULL
        );

        -- Settings
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Record migration
        INSERT INTO schema_migrations (version, applied_at)
        VALUES (1, datetime('now'));
        ",
    )?;

    Ok(())
}
