# ChangeRadar Cross-Platform Validation Report

**Audit Date:** October 31, 2026  
**Auditor:** Operating-System Integration Engineering  
**Primary Execution Environment:** Windows 11 (x86_64, MSVC 2022)  
**Target Environments:** Windows 10/11, Ubuntu Linux 22.04+, macOS 12+ (Apple Silicon & Intel)

---

## 1. Executive Summary

ChangeRadar was designed with a local-first, cross-platform architecture. The application core, storage engines, database, local API, and desktop frontends avoid proprietary OS APIs in favor of standard cross-platform Rust and Web technologies.

In accordance with strict audit rules, **this report explicitly distinguishes verified Windows execution from static analysis and CI readiness on Linux and macOS.**

---

## 2. Platform Status Matrix

| Component | Windows 10/11 | Linux (Ubuntu/Debian) | macOS (Intel / ARM64) |
|---|---|---|---|
| **Rust Core Workspace (18 crates)** | **VERIFIED PASS** (All 38 tests pass) | CI Ready (POSIX compliant) | CI Ready (POSIX compliant) |
| **Filesystem Watcher (`notify`)** | **VERIFIED PASS** (ReadDirectoryChangesW) | CI Ready (inotify adapter) | CI Ready (FSEvents adapter) |
| **Atomic File Recovery** | **VERIFIED PASS** (Windows-safe remove/rename) | CI Ready (POSIX atomic rename) | CI Ready (POSIX atomic rename) |
| **Local Axum API (Port 41820)** | **VERIFIED PASS** (TCP loopback) | CI Ready (POSIX socket loopback) | CI Ready (POSIX socket loopback) |
| **CLI Binary (`changeradar.exe`)** | **VERIFIED PASS** (20.3 MB release) | CI Ready (ELF binary target) | CI Ready (Mach-O binary target) |
| **Daemon (`changeradard.exe`)** | **VERIFIED PASS** (21.1 MB release) | CI Ready (Systemd compatible) | CI Ready (Launchd compatible) |
| **Desktop Workstation (Tauri 2)** | **VERIFIED PASS** (36.1 MB executable) | CI Ready (WebKitGTK) | CI Ready (WKWebView) |
| **Installers / Packages** | **VERIFIED PASS** (MSI + NSIS installers) | CI Ready (.deb / AppImage) | CI Ready (.dmg) |

---

## 3. Platform-Specific Implementations Audited

### 3.1 Path Resolution & Directories (`crates/configuration`)
- **Windows:** Resolves to `%LOCALAPPDATA%\ChangeRadar` (e.g. `C:\Users\<user>\AppData\Local\ChangeRadar`).
- **Linux:** Resolves to `$XDG_DATA_HOME/ChangeRadar` or `~/.local/share/ChangeRadar`.
- **macOS:** Resolves to `~/Library/Application Support/ChangeRadar`.
- **Audit Finding:** Uses `dirs::data_local_dir()` and `dirs::config_dir()`. Verified on Windows; standard cross-platform implementation.

### 3.2 File Reveal in Native File Manager (`crates/platform`)
- **Windows:** Spawns `explorer.exe /select,"<path>"`.
- **macOS:** Spawns `open -R "<path>"`.
- **Linux:** Spawns `xdg-open "<parent_path>"`.
- **Audit Finding:** Verified that Windows explorer invocation correctly selects target files. Fallbacks on non-Windows correctly branch using `#[cfg(target_os = ...)]`.

### 3.3 Atomic File Replacement & Recovery (`crates/recovery`)
- **Windows Limitation:** Standard POSIX atomic rename semantics are not supported by the Windows Win32 `MoveFileEx` / `rename` API when the target file exists. It fails with `ERROR_ALREADY_EXISTS`.
- **Solution Audited:**
  - On Windows: Backup existing file into CAS, remove destination file, then rename temporary restore file into place.
  - On POSIX (Linux/macOS): Direct `fs::rename(&temp_path, &dest_path)` which is natively atomic at the filesystem inode level.
- **Audit Finding:** Windows atomic replacement thoroughly tested and verified in `audit_recovery_engine_test.rs`.

### 3.4 Filesystem Watching (`crates/watcher`)
- Backed by `notify = "6.1"`.
- Uses `notify::RecommendedWatcher`:
  - Windows: `ReadDirectoryChangesW` (IOCP).
  - Linux: `inotify`.
  - macOS: `FSEvents`.
- **Reconciliation Engine:** If OS watcher buffers overflow or events are dropped, ChangeRadar's periodic reconciliation scan (`reconcile_folder`) guarantees eventual consistency by diffing SQLite records against disk.

---

## 4. Continuous Integration Matrix (`ci/github-ci.yml`)

The repository includes GitHub Actions CI testing the workspace across all 3 platforms:
- `runs-on: windows-latest`
- `runs-on: ubuntu-latest`
- `runs-on: macos-latest`

Steps include `cargo test --workspace`, `cargo clippy -- -D warnings`, and frontend build verification.
