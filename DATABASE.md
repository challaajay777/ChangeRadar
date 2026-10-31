# ChangeRadar — Database Architecture & Schema Specification

**Database Engine:** SQLite 3 (Bundled C source)  
**Journal Mode:** Write-Ahead Logging (`WAL`)  
**Synchronous Level:** `NORMAL`  
**Foreign Keys:** Strictly Enforced (`PRAGMA foreign_keys = ON;`)  
**Busy Timeout:** 5,000 ms

---

## 1. Storage Philosophy

ChangeRadar strictly follows the separation of metadata and file content:
- **SQLite Database:** Stores folder configurations, file metadata, event timeline, version pointers, rules, notifications, and snapshot manifests.
- **Content-Addressed Storage (CAS):** Actual historical file content is compressed with Zstandard and stored in the filesystem addressed by its **BLAKE3** cryptographic hash.

Enormous file contents are never stored inside SQLite table rows, ensuring the database file remains lightweight (typically a few megabytes) and highly responsive.

---

## 2. Table Schemas

### 2.1 `monitored_folders`
Stores folders configured for recursive watching.
```sql
CREATE TABLE monitored_folders (
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
```

### 2.2 `files`
Tracks logical file entities.
```sql
CREATE TABLE files (
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
CREATE INDEX idx_files_folder_id ON files(folder_id);
CREATE INDEX idx_files_relative_path ON files(relative_path);
CREATE INDEX idx_files_current_hash ON files(current_hash);
```

### 2.3 `content_objects`
Tracks unique compressed objects in CAS.
```sql
CREATE TABLE content_objects (
    content_hash TEXT PRIMARY KEY,
    original_size INTEGER NOT NULL,
    compressed_size INTEGER NOT NULL,
    compression_algorithm TEXT NOT NULL DEFAULT 'zstd',
    reference_count INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
```

### 2.4 `file_versions`
Historical revisions for each tracked file.
```sql
CREATE TABLE file_versions (
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
CREATE INDEX idx_file_versions_file_id ON file_versions(file_id);
CREATE INDEX idx_file_versions_hash ON file_versions(content_hash);
CREATE INDEX idx_file_versions_created_at ON file_versions(created_at);
```

### 2.5 `file_events`
Audit trail and timeline feed.
```sql
CREATE TABLE file_events (
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
CREATE INDEX idx_file_events_folder ON file_events(folder_id);
CREATE INDEX idx_file_events_created ON file_events(created_at);
CREATE INDEX idx_file_events_type ON file_events(event_type);
```

### 2.6 `snapshots` & `snapshot_items`
Point-in-time state captures.
```sql
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    folder_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    file_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (folder_id) REFERENCES monitored_folders(id) ON DELETE SET NULL
);

CREATE TABLE snapshot_items (
    snapshot_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    version_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    PRIMARY KEY (snapshot_id, file_id),
    FOREIGN KEY (snapshot_id) REFERENCES snapshots(id) ON DELETE CASCADE
);
```

### 2.7 `notification_rules` & `notifications`
Custom alert rules and dispatched events.
```sql
CREATE TABLE notification_rules (
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

CREATE TABLE notifications (
    id TEXT PRIMARY KEY,
    rule_id TEXT,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    priority TEXT NOT NULL DEFAULT 'NORMAL',
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES notification_rules(id) ON DELETE SET NULL
);
CREATE INDEX idx_notifications_unread ON notifications(is_read, created_at);
```

---

## 3. Database Maintenance & Verification

- **Integrity Checks:** The database can be tested at any time via `PRAGMA integrity_check;`, exposed via CLI `changeradar doctor` and UI Diagnostics.
- **Transactions:** Multi-row batch operations (snapshot creation, recovery records, retention cleanup) use strict SQLite transactions.
