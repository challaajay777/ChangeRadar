# ChangeRadar Engineering Audit: Code Remediation & Fix Log

**Audit Date:** October 31, 2026  
**Auditor:** Principal Rust Systems Engineer & Desktop Application Architect  
**Workspace:** `C:\Users\ajays\.gemini\antigravity\scratch\ChangeRadar`

---

## Fix 1: [HIGH] DEF-002: Plugin Execution Crash Isolation
- **Component:** `crates/plugin_api/src/lib.rs`
- **Issue:** A panicking plugin directly terminated the calling monitoring thread and daemon process.
- **Code Changes:**
  ```rust
  // Wrapped plugin execution in std::panic::catch_unwind with AssertUnwindSafe
  let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
      p.on_file_changed(&path_str, &old_vec, &new_vec)
  }));
  match outcome {
      Ok(Some(msg)) => results.push((plugin_id, msg)),
      Ok(None) => {},
      Err(err) => {
          tracing::error!("Plugin '{}' panicked during execution: {:?}", plugin_id, err);
      }
  }
  ```
- **Verification:** `cargo test --test audit_security_and_api_test` (`test_api_security_and_plugin_crash_isolation`). **PASSED.**

---

## Fix 2: [MEDIUM] DEF-001: Deduplication Ratio & Storage Metrics
- **Component:** `crates/database/src/lib.rs`
- **Issue:** `get_storage_stats` computed single-object compression ratio instead of logical version deduplication ratio.
- **Code Changes:**
  ```rust
  // Added query for total logical raw bytes across all tracked versions
  let total_logical_raw_bytes: i64 = conn
      .query_row(
          "SELECT COALESCE(SUM(file_size), 0) FROM file_versions",
          [],
          |row| row.get(0),
      )
      .unwrap_or(unique_raw_bytes);

  let ratio = if comp_bytes > 0 {
      (total_logical_raw_bytes as f64) / (comp_bytes as f64)
  } else {
      1.0
  };
  ```
- **Verification:** `cargo test --test audit_cas_and_database_test` (`test_cas_and_database_audit`). **PASSED.**

---

## Fix 3: [CRITICAL] DEF-003: Desktop Application Release Packaging
- **Component:** `apps/desktop/package.json` & `scripts/build_release.ps1`
- **Issue:** Release build script omitted `changeradar-desktop.exe` and native installers (only built CLI/Daemon and copied Vite bundle).
- **Code Changes:**
  1. Installed `@tauri-apps/cli@2.12.4` in `apps/desktop/package.json`.
  2. Added `-p changeradar-desktop` to release cargo build command in `scripts/build_release.ps1`.
  3. Added copying of `target/release/changeradar-desktop.exe` to `dist-release/bin`.
  4. Added copying of Tauri bundles (`ChangeRadar_1.0.0_x64_en-US.msi` and `ChangeRadar_1.0.0_x64-setup.exe`) to `dist-release/installer`.
  5. Updated `SHA256SUMS.txt` generation to include all desktop binaries and installers.
- **Verification:** Ran `npm run tauri build` and `build_release.ps1`. Verified 36.1 MB native executable and both installers exist in `dist-release`.

---

## Fix 4: [HIGH] DEF-004: Restrict Local REST API CORS Configuration
- **Component:** `crates/local_api/src/lib.rs`
- **Issue:** Local REST API allowed wildcard `Any` origin, enabling cross-origin requests from arbitrary websites.
- **Code Changes:**
  ```rust
  let cors = CorsLayer::new()
      .allow_origin(AllowOrigin::predicate(|origin, _| {
          if let Ok(s) = origin.to_str() {
              s.starts_with("http://localhost:")
                  || s.starts_with("http://127.0.0.1:")
                  || s.starts_with("tauri://")
                  || s.starts_with("http://tauri.localhost")
                  || s.starts_with("https://tauri.localhost")
          } else {
              false
          }
      }))
      .allow_methods(Any)
      .allow_headers(Any);
  ```
- **Verification:** Added `test_cors_origin_restriction` in `crates/local_api/tests/api_routes_test.rs`. **PASSED.**

---

## Fix 5: [HIGH] DEF-005: Desktop REST API Authorization & Mock Masking
- **Component:** `apps/desktop/src/services/api.ts`
- **Issue:** Frontend REST fallback never transmitted Bearer tokens, resulting in 401 Unauthorized errors and falling back to hardcoded mock data.
- **Code Changes:**
  1. Updated `fetchRest` to read stored Bearer token from `localStorage.getItem('changeradar_api_token')`.
  2. Implemented full REST route dispatch for all commands (`files/:id/versions`, `search?q=`, `recovery/preflight`, `recovery/restore`, `storage/cleanup`).
  3. Added warning logs on 401 Unauthorized.
  4. Added API Bearer token configuration input to `SettingsPage.tsx`.
- **Verification:** `npm run build` in `apps/desktop`. **PASSED.**

---

## Fix 6: [HIGH] DEF-006: Recovery Engine Temporary File Collisions & Atomicity
- **Component:** `crates/recovery/src/lib.rs`
- **Issue:** Temporary restore path used `with_extension` which mangled dotfiles, and failed writes left orphaned `.tmp` files.
- **Code Changes:**
  1. Generated unique temp filename in destination parent directory (`.cr_restore_<job_id>_<uuid>.tmp`).
  2. Added explicit deletion of temp file on write, flush, or rename failure.
  3. Preserved atomic replacement on Windows with pre-restore CAS backup.
- **Verification:** `cargo test --test audit_recovery_engine_test`. **PASSED.**

---

## Fix 7: [MEDIUM] DEF-007: CAS Version Store Temp Path Collision Under Concurrent Writes
- **Component:** `crates/version_store/src/lib.rs`
- **Issue:** Deterministic `.tmp` path collision under concurrent writes.
- **Code Changes:**
  1. Generated unique temp filename with UUID: `target_path.with_extension(format!("tmp.{}", Uuid::new_v4()))`.
  2. Added failure cleanup guards for intermediate encoder and writer states.
  3. Handled concurrent destination creation gracefully.
- **Verification:** `cargo test --test audit_cas_and_database_test`. **PASSED.**

---

## Fix 8: [MEDIUM] DEF-008: Defensive Garbage Collection Unreferenced Object Filter
- **Component:** `crates/database/src/lib.rs`
- **Issue:** Garbage collection query did not defensively check active references in `snapshot_items`.
- **Code Changes:**
  ```sql
  SELECT content_hash FROM content_objects 
  WHERE reference_count = 0 
    AND content_hash NOT IN (SELECT content_hash FROM file_versions)
    AND content_hash NOT IN (SELECT content_hash FROM snapshot_items)
  ```
- **Verification:** `cargo test --test audit_cas_and_database_test`. **PASSED.**

---

## Fix 9: [LOW] DEF-010: Settings Page Preference Persistence
- **Component:** `apps/desktop/src/pages/SettingsPage.tsx`
- **Issue:** Theme and settings did not persist across restarts.
- **Code Changes:**
  1. Initialized state from `localStorage` (`changeradar_theme`, `changeradar_api_token`).
  2. Persisted settings to `localStorage` in `saveSettings`.
- **Verification:** Frontend build clean (`npm run build`).

---

## Fix 10: [LOW] DEF-009: Eliminate Redundant Unwraps in Local API Handlers
- **Component:** `crates/local_api/src/lib.rs`
- **Issue:** Handlers utilized `serde_json::to_value(...).unwrap()`, which was redundant and could theoretically panic if custom serialization failed.
- **Code Changes:**
  Replaced all `to_value(...).unwrap()` calls across 14 API handlers with the zero-panic `serde_json::json!(...)` macro.
- **Verification:** `cargo test -p changeradar-local-api`. **PASSED.**

---

## Fix 11: [LOW] Watcher Queue Overflow Logging & Tauri Identifier Format
- **Components:** `crates/watcher/src/lib.rs`, `apps/desktop/src-tauri/tauri.conf.json`
- **Issue:**
  1. Silent event drop when channel capacity reached 10,000 without logging warning for reconciliation fallback.
  2. Tauri bundle identifier had an upstream schema warning.
- **Code Changes:**
  1. In `FolderWatcherManager::dispatch_event`, matched `TrySendError::Full` to emit `warn!` log notifying that reconciliation scan will recover events.
  2. Updated `apps/desktop/src-tauri/tauri.conf.json` identifier to `io.changeradar.desktop`.
- **Verification:** `cargo clippy --workspace --all-targets --all-features -- -D warnings`. **0 warnings.**

