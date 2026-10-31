# ChangeRadar Test Coverage & Verification Matrix

**Audit Date:** October 31, 2026  
**Test Suite Status:** 38 / 38 Tests PASSING (0 Failures, 0 Skipped, 0 Warnings)  
**Rust Version:** 1.99.1 (x86_64-pc-windows-msvc)

---

## 1. Test Suite Summary

```
Total Automated Test Cases: 38
Passed: 38 (100%)
Failed: 0 (0%)
Clippy Warnings: 0 (-D warnings enforced)
Formatting: 100% compliant (cargo fmt --check clean)
```

---

## 2. Test Coverage by Subsystem

### 2.1 Filesystem Monitoring (`crates/watcher`, `crates/event_engine`)
- **Integration Test:** `crates/core/tests/audit_filesystem_monitoring_test.rs`
- **Scenarios Covered (16/16 verified):**
  1. Create a file on disk
  2. Modify a file
  3. Modify a file repeatedly (rapid successive modifications)
  4. Delete a file and verify database marks `is_deleted = true`
  5. Rename a file within monitored folder
  6. Move a file across two different monitored folders
  7. Create nested directory structures
  8. Create and track files inside nested directories
  9. Delete an entire directory tree and reconcile all contained files
  10. Atomic write pattern (editor saving to `.tmp` then atomic rename)
  11. Modify a file while engine is actively monitoring
  12. Restart engine and verify monitoring configuration persists
  13. Bulk directory scan (50 files created simultaneously)
  14. Ignored directories (`.git`, `node_modules`, temp files)
  15. Unicode filenames and special characters (`файл_тест_中文_🚀.txt`)
  16. Offline change detection on engine restart

### 2.2 Recovery Engine (`crates/recovery`)
- **Integration Test:** `crates/core/tests/audit_recovery_engine_test.rs`
- **Unit Test:** `crates/recovery/src/lib.rs` (`test_safe_restore_pipeline`)
- **Scenarios Covered:**
  - Restore historical version to a custom destination path
  - Restore over an existing file (pre-flight conflict warning)
  - Automatic safety backup of destination content prior to overwrite
  - Atomic temporary write and atomic rename replacement
  - Post-restore BLAKE3 verification on restored disk bytes
  - Restore deleted files (re-creating disk file from CAS)
  - Restore Unicode filenames
  - Corrupted CAS object rejection (simulated corrupted CAS bytes detected and rejected)
  - Recovery timeline audit trail recording

### 2.3 CAS & Version Storage (`crates/version_store`, `crates/database`)
- **Integration Test:** `crates/core/tests/audit_cas_and_database_test.rs`
- **Unit Tests:** `crates/version_store/src/lib.rs` (`test_deduplication`, `test_store_and_retrieve_version`)
- **Scenarios Covered:**
  - Deduplication: 2 identical files stored share 1 CAS object
  - Reference counting: Incremented on conflict, decremented on version delete
  - Distinct content with similar names stored in separate CAS objects
  - Multiple version retrieval: Every historical version retrievable and verifiable
  - Retention cleanup preview and execution
  - Garbage collection: Unreferenced CAS objects reclaimed, referenced objects protected
  - CAS integrity verification over entire storage tree

### 2.4 Snapshot Engine (`crates/snapshot_engine`)
- **Integration Test:** `crates/core/tests/audit_snapshot_engine_test.rs`
- **Unit Test:** `crates/snapshot_engine/src/lib.rs` (`test_snapshot_creation_and_listing`)
- **Scenarios Covered:**
  - Snapshot creation with nested files
  - Detecting modifications after snapshot creation
  - Detecting additions after snapshot creation
  - Detecting deletions after snapshot creation
  - Snapshot comparison (added, removed, modified items)
  - Folder rollback: Reverting directory state to snapshot baseline

### 2.5 Universal Comparison Engine (`crates/comparison`)
- **Integration Test:** `crates/core/tests/audit_comparison_engine_test.rs`
- **Unit Tests:** `crates/comparison/src/lib.rs` (`test_text_comparison`, `test_json_comparison`, `test_csv_comparison`)
- **Scenarios Covered:**
  - Text diff: Additions, deletions, empty files, Unicode lines, line endings
  - JSON diff: Nested objects, arrays, added/removed keys, modified values, invalid JSON rejection
  - CSV diff: Row changes, column changes, custom delimiters (TSV)
  - Image diff: Identical images, dimension mismatch, perceptual hashing
  - PDF diff: Heuristic text extraction
  - Binary diff: Byte differences, size diffs, hex sample generation

### 2.6 Local REST API & Security (`crates/local_api`)
- **Integration Tests:** `crates/local_api/tests/api_routes_test.rs` & `crates/core/tests/audit_security_and_api_test.rs`
- **Scenarios Covered:**
  - Router creation without panic (verifying Axum 0.8 `{param}` syntax)
  - Missing Bearer token rejection (`401 Unauthorized`)
  - Invalid Bearer token rejection (`401 Unauthorized`)
  - CORS origin restriction: Untrusted origins denied access-control-allow-origin
  - CORS local origin allowance: `localhost:5173` permitted
  - All 14 documented API endpoints queried and validated
  - Live TCP server startup and health check response
  - Plugin crash isolation: Panicking plugins caught via `catch_unwind` without crashing host

### 2.7 Platform, Hashing & Intelligence
- `crates/hashing`: BLAKE3 hashing, perceptual hashing, Hamming distance similarity
- `crates/platform`: Platform detection, OS-specific path resolution, file reveal in explorer
- `crates/configuration`: Zero telemetry guarantee, config persistence
- `crates/intelligence`: Radar score computation, velocity & volume scoring
- `crates/search`: Unified search across files, events, and folders
