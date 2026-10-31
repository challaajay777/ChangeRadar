# Changelog

All notable changes to **ChangeRadar** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-10-31

### Initial Production Release

#### Core & Architecture
- **Multi-Crate Workspace**: Fully modular architecture with 18 specialized crates (`types`, `hashing`, `database`, `version_store`, `storage_manager`, `watcher`, `event_engine`, `comparison`, `recovery`, `intelligence`, `notification_engine`, `snapshot_engine`, `search`, `plugin_api`, `configuration`, `platform`, `local_api`, `core`).
- **Local-First & Offline-First**: Zero mandatory cloud services, no mandatory accounts, and zero telemetry by default.
- **SQLite Database Layer**: 17 relational tables with WAL mode, foreign key enforcement, typed repositories, and schema migrations.
- **Content-Addressed Storage (CAS)**:
  - 256-bit BLAKE3 streaming hashing in 64 KB buffers.
  - Two-tier sharded directory structure (`objects/ab/cd/<hash>.zst`).
  - Zstandard compression (level 3) with content-level deduplication across files and versions.
  - Atomic object writes with CRC/checksum verification.
  - Storage retention policies, manual version pinning, and garbage collection.

#### Filesystem Monitoring
- Recursive real-time monitoring via OS event channels (`ReadDirectoryChangesW`, `inotify`, `FSEvents`).
- Event coalescing and debouncing to properly classify atomic writes (save-via-temp-and-rename).
- Dynamic glob-based ignore rules with sensible defaults (`.git/`, `node_modules/`, `target/`, temporary office locks).
- Periodic background reconciliation scanner to recover missed events and handle drive reconnections.

#### Universal Comparison Engine
- **Text & Code**: Myers LCS line-by-line diff, unified patch output, and side-by-side aligned diffs with character-level additions and deletions.
- **Structural JSON**: Deep recursive comparator with RFC 6901 / JSONPath markers (e.g., `$.users[0].name`).
- **Tabular CSV**: Header-aware row and cell comparisons with primary-key tracking.
- **Images**: Dimension comparison, exact byte matching, and 64-bit perceptual difference hashing (`dHash`) with visual similarity scoring (0–100%).
- **PDF Documents**: Content stream text extraction and structural textual difference analysis.
- **Binary Files**: Hex dump comparison with offset, byte delta, and chunk classification.

#### Non-Destructive 8-Step Recovery Pipeline
1. Pre-flight verification of source CAS object integrity and disk space.
2. Conflict detection against current disk state.
3. Automatic pre-restore safety backup into CAS before overwriting existing files.
4. Flexible target strategies (original, new path, isolated quarantine).
5. Atomic decompression to temporary file (`.changeradar-restore-tmp-*.tmp`).
6. Atomic file replacement using OS primitives.
7. Post-restore BLAKE3 hash recalculation and assertion against requested historical version.
8. Permanent database audit trail recording source, destination, user notes, and safety backup ID.

#### Change Radar Score & Intelligence
- Dynamic score from 0 to 100 based on velocity, churn, and volume.
- Color-coded risk status (Calm, Moderate, Elevated, High Alert).
- Anomaly detectors for mass deletion bursts, rapid expansion, and unusual activity velocity.
- 7-day velocity tracking, top active files, extension distributions, and activity heatmaps.

#### Directory Snapshot Engine
- Capture point-in-time snapshot manifests of entire directory trees.
- Snapshot comparison to highlight added, modified, deleted, and unchanged files.
- Full directory hierarchy rollback using the recovery pipeline.

#### Sandboxed Plugin Architecture
- Capability-based permissions (`read_file_content`, `inspect_events`, `compute_metrics`, `emit_notifications`).
- Zero network socket permissions.
- Built-in `LogAnalyzerPlugin` for flagging error spikes in log files.
- Built-in `WordCounterPlugin` for tracking writing productivity.

#### User Interfaces & Developer Tools
- **ChangeRadar Desktop**: Tauri 2 + React 19 + TypeScript workstation UI with 14 navigation sections and quick command palette (`Ctrl+K`).
- **ChangeRadar CLI (`changeradar`)**: Full command suite for status, folders, history, search, compare, restore, snapshots, storage cleanup, and doctor diagnostics with `--json` support.
- **ChangeRadar Daemon (`changeradard`)**: Headless background service with periodic reconciliation and authenticated local Axum REST API on `127.0.0.1:41820`.
