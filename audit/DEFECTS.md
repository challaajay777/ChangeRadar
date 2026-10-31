# ChangeRadar Engineering Audit: Confirmed Defects & Resolutions

**Audit Date:** October 31, 2026  
**Auditor:** Independent Systems & QA Engineering Team  
**Scope:** Full repository inspection, test execution, desktop packaging, security validation  
**Repository:** `https://github.com/challaajay777/ChangeRadar`

---

## Defect Summary

| ID | Severity | Component | Summary | Status |
|---|---|---|---|---|
| **DEF-001** | MEDIUM | `crates/database` | `deduplication_ratio` in `get_storage_stats` calculated single-object compression ratio instead of logical version deduplication ratio | **FIXED** |
| **DEF-002** | HIGH | `crates/plugin_api` | Lack of crash isolation in plugin runner: panicking plugins terminate the entire monitoring engine | **FIXED** |
| **DEF-003** | CRITICAL | `apps/desktop` / `scripts` | Desktop release packaging omitted native desktop executable (`changeradar-desktop.exe`) and installer | **FIXED** |
| **DEF-004** | HIGH | `crates/local_api` | Local REST API configured with permissive `allow_origin(Any)` CORS on port 41820 | **FIXED** |
| **DEF-005** | HIGH | `apps/desktop` / `api.ts` | Frontend REST fallback lacks Bearer token authentication and falls back to mock demo data on failure | **FIXED** |
| **DEF-006** | HIGH | `crates/recovery` | Recovery temp files collide on non-extension files and unhandled error leaves orphaned `.tmp` files | **FIXED** |
| **DEF-007** | MEDIUM | `crates/version_store` | CAS object store uses non-unique `.tmp` paths vulnerable to concurrent clashing and leakage | **FIXED** |
| **DEF-008** | MEDIUM | `crates/database` | Garbage collection query did not defensively check active references in `snapshot_items` | **FIXED** |
| **DEF-009** | LOW | `crates/local_api` | Redundant `to_value().unwrap()` in 16 handler return paths causing potential panics | **FIXED** |
| **DEF-010** | LOW | `apps/desktop` | Settings page did not persist theme or API token across application restarts | **FIXED** |

---

## Detailed Defect Reports

### DEF-001: Incorrect Deduplication Ratio Calculation
- **Severity:** Medium
- **Component:** `crates/database/src/lib.rs` (lines 893–925)
- **Reproduction Steps:**
  1. Store 2 identical files of 54 bytes each in a monitored folder.
  2. Query `storage_mgr.get_stats()`.
  3. `deduplication_ratio` returned `1.0` (or `< 1.0` if zstd header overhead on tiny file exceeded raw size).
- **Root Cause:**
  The query used `SELECT SUM(compressed_size), SUM(original_size) FROM content_objects`. Because content objects are already deduplicated, `original_size` only counted the unique 54 bytes, completely ignoring the total logical bytes stored across `file_versions`.
- **Fix:**
  Added `SELECT COALESCE(SUM(file_size), 0) FROM file_versions` to compute the total logical bytes tracked (`total_logical_raw_bytes`), and calculated ratio as `total_logical_raw_bytes / comp_bytes`.
- **Regression Test:** `crates/core/tests/audit_cas_and_database_test.rs` (`test_cas_and_database_audit`).

---

### DEF-002: Plugin Execution Lack of Crash Isolation
- **Severity:** High
- **Component:** `crates/plugin_api/src/lib.rs` (lines 61–76)
- **Reproduction Steps:**
  1. Register a plugin whose `on_file_changed` callback panics (e.g., `panic!("boom")`).
  2. Trigger file change via `run_plugins_on_change`.
  3. The panic propagates unhandled and terminates the host thread / daemon.
- **Root Cause:**
  Plugin trait method invocation was not guarded against panics. In Rust, an unhandled panic on a worker thread terminates the thread and aborts process execution.
- **Fix:**
  Wrapped plugin execution in `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))`. On panic, the engine logs an error with `tracing::error!` and continues executing subsequent plugins cleanly.
- **Regression Test:** `crates/core/tests/audit_security_and_api_test.rs` (`test_api_security_and_plugin_crash_isolation`).

---

### DEF-003: Desktop Release Packaging Omitted Native Desktop Executable
- **Severity:** Critical
- **Component:** `scripts/build_release.ps1` & `apps/desktop/package.json`
- **Reproduction Steps:**
  1. Run `scripts/build_release.ps1`.
  2. Inspect `dist-release/bin`.
  3. Only `changeradar.exe` (CLI) and `changeradard.exe` (Daemon) existed. No `changeradar-desktop.exe` or installer was built or copied.
- **Root Cause:**
  The release script only invoked `cargo build --release -p changeradar-cli -p changeradar-daemon` and built the Vite bundle into `dist-release/ui`. `@tauri-apps/cli` was missing from `apps/desktop/package.json`.
- **Fix:**
  1. Installed `@tauri-apps/cli` into `apps/desktop/package.json`.
  2. Added `-p changeradar-desktop` to release cargo build command.
  3. Copied `changeradar-desktop.exe` to `dist-release/bin/changeradar-desktop.exe`.
  4. Added desktop binary SHA-256 checksum to `dist-release/SHA256SUMS.txt`.
- **Verification:** Native release binary `target/release/changeradar-desktop.exe` (35.8 MB) compiled and packaged into release distribution.

---

### DEF-004: Local REST API Permissive CORS Configuration
- **Severity:** High
- **Component:** `crates/local_api/src/lib.rs` (lines 73–76)
- **Reproduction Steps:**
  1. Start local API daemon on `127.0.0.1:41820`.
  2. Issue an HTTP `OPTIONS` request with `Origin: https://malicious-website.com`.
  3. API returned `access-control-allow-origin: *`.
- **Root Cause:**
  CORS layer was initialized with `CorsLayer::new().allow_origin(Any)`. For a desktop daemon with privileged filesystem operations, this allows arbitrary web pages visited in a browser to issue cross-origin requests.
- **Fix:**
  Replaced `allow_origin(Any)` with an explicit origin predicate allowing only trusted local and desktop origins (`http://localhost:*`, `http://127.0.0.1:*`, `tauri://localhost`, `http://tauri.localhost`, `https://tauri.localhost`).
- **Regression Test:** `crates/local_api/tests/api_routes_test.rs` (`test_cors_origin_restriction`).

---

### DEF-005: Frontend REST Fallback Missing Bearer Authentication & Mock Fallback
- **Severity:** High
- **Component:** `apps/desktop/src/services/api.ts`
- **Reproduction Steps:**
  1. Run frontend in browser mode against running daemon.
  2. All API calls returned 401 Unauthorized because no Authorization header was passed.
  3. Frontend caught 401 and displayed hardcoded mock data (`getMockFallback`) without informing the user.
- **Root Cause:**
  `fetchRest` did not read or transmit the Bearer token from configuration / localStorage, and silently masked HTTP errors with mock data.
- **Fix:**
  Updated `fetchRest` to retrieve stored API token from `localStorage.getItem('changeradar_api_token')`, pass `Authorization: Bearer <token>`, implement full REST route dispatch for all commands, and warn on 401. Added token configuration input to `SettingsPage.tsx`.
- **Verification:** Frontend build clean (`npm run build`).

---

### DEF-006: Recovery Engine Temporary File Collisions & Non-Atomic Cleanup
- **Severity:** High
- **Component:** `crates/recovery/src/lib.rs` (lines 167–181)
- **Reproduction Steps:**
  1. Restore a file without an extension (e.g. `LICENSE` or `.gitignore`).
  2. `with_extension("tmp-restore-...")` mangles the filename.
  3. If write or flush fails, the temporary file was leaked on disk.
- **Root Cause:**
  `Path::with_extension` replaces the extension, which fails on dotfiles or extensions with multiple parts. Missing cleanup guard on I/O failure.
- **Fix:**
  Used parent directory join with dedicated hidden temporary pattern (`.cr_restore_<job_id>_<uuid>.tmp`). Added immediate deletion of the temp file on any subsequent I/O or rename failure.
- **Regression Test:** `crates/core/tests/audit_recovery_engine_test.rs` (`test_recovery_engine_comprehensive_audit`).

---

### DEF-007: CAS Version Store Temp Path Collision Under Concurrent Writes
- **Severity:** Medium
- **Component:** `crates/version_store/src/lib.rs` (lines 135 & 195)
- **Reproduction Steps:**
  1. Two concurrent threads store the same file content or different files producing identical object hash paths.
  2. Both threads wrote to `target_path.with_extension("tmp")`.
  3. Race condition caused one thread to overwrite or fail with lock contention.
- **Root Cause:**
  Deterministic `.tmp` extension path used across all concurrent writers.
- **Fix:**
  Generated unique temporary filenames using `Uuid::new_v4()` (`.tmp.<uuid>`), cleaned up on failure, and safely handled Windows target replacement.
- **Regression Test:** `crates/core/tests/audit_cas_and_database_test.rs` (`test_cas_and_database_audit`).

---

### DEF-008: Defensive Garbage Collection Unreferenced Object Filter
- **Severity:** Medium
- **Component:** `crates/database/src/lib.rs` (lines 367–377)
- **Root Cause:**
  `list_unreferenced_content_hashes` only checked `WHERE reference_count = 0`. While foreign keys protect active versions, `snapshot_items` also references CAS hashes.
- **Fix:**
  Added defensive exclusion clauses:
  `AND content_hash NOT IN (SELECT content_hash FROM file_versions) AND content_hash NOT IN (SELECT content_hash FROM snapshot_items)`.
- **Regression Test:** `crates/core/tests/audit_cas_and_database_test.rs`.

---

### DEF-009: Redundant `to_value().unwrap()` in Local API Handlers
- **Severity:** Low
- **Component:** `crates/local_api/src/lib.rs`
- **Root Cause:**
  16 Axum handlers wrapped serializable response types in `Json(serde_json::to_value(res).unwrap())`. In Axum, `Json(res)` directly serializes types implementing `Serialize`.
- **Fix:**
  Simplified handler returns and eliminated panic paths.

---

### DEF-010: Settings Page Preference Persistence
- **Severity:** Low
- **Component:** `apps/desktop/src/pages/SettingsPage.tsx`
- **Root Cause:**
  Clicking "Save Configuration" only toggled a temporary visual toast in local React state and never persisted theme or API tokens.
- **Fix:**
  Added persistence to `localStorage` for theme (`changeradar_theme`) and API Bearer token (`changeradar_api_token`). Added API Token input field.
