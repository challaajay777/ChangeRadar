# ChangeRadar Development Guide

This document describes how to set up the development environment, execute tests, add features, and run local builds for ChangeRadar.

---

## 1. Prerequisites

- **Rust**: 1.75 or newer with `clippy` and `rustfmt` components:
  ```bash
  rustup component add clippy rustfmt
  ```
- **Node.js**: v18.0.0+ (v20+ recommended) and `npm`.
- **Git**: Installed and configured.
- **C Compiler**:
  - Windows: MSVC C++ Build Tools or MinGW GCC (ucrt64 / msys2).
  - Linux: `gcc`, `make`, `pkg-config`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`.
  - macOS: Xcode Command Line Tools.

---

## 2. Repository Layout

ChangeRadar is organized as a Cargo workspace with 18 crates and 3 applications:

```
ChangeRadar/
├── Cargo.toml               # Workspace manifest
├── crates/
│   ├── types/               # Common data structures and DTOs
│   ├── hashing/             # Streaming BLAKE3 & perceptual dHash
│   ├── database/            # SQLite migrations & typed repositories
│   ├── version_store/       # Content-Addressed Storage (CAS)
│   ├── storage_manager/     # Retention policies, quotas, GC
│   ├── watcher/             # Recursive filesystem watcher & ignore logic
│   ├── event_engine/        # Change classification & event coalescing
│   ├── comparison/          # Text, JSON, CSV, Image, PDF, Binary diffs
│   ├── recovery/            # 8-step safe file recovery pipeline
│   ├── intelligence/        # Change Radar score (0-100) & anomaly detection
│   ├── notification_engine/ # Priority notification dispatch
│   ├── snapshot_engine/     # Directory point-in-time snapshots
│   ├── search/              # Unified search index across events and files
│   ├── plugin_api/          # Plugin sandboxing, capabilities, and hooks
│   ├── configuration/       # Config file persistence (%LOCALAPPDATA%)
│   ├── platform/            # Native shell file manager integration
│   ├── local_api/           # Axum REST server on 127.0.0.1:41820
│   └── core/                # Master ChangeRadarEngine facade
└── apps/
    ├── cli/                 # `changeradar` command line interface
    ├── daemon/              # `changeradard` headless background daemon
    └── desktop/             # Tauri 2 + React 19 + TypeScript workstation UI
        ├── src/             # React components, pages, hooks, state
        └── src-tauri/       # Tauri IPC commands & native bridge
```

---

## 3. Building and Running

### 3.1 Compiling All Backend Crates and Apps
```bash
cargo build
```

To build in release mode with optimizations:
```bash
cargo build --release
```

### 3.2 Running the CLI
```bash
cargo run -p changeradar-cli -- status
cargo run -p changeradar-cli -- folder list
```

### 3.3 Running the Daemon
```bash
cargo run -p changeradar-daemon
```

### 3.4 Running the Desktop UI in Development Mode
```bash
cd apps/desktop
npm install
npm run dev
```

To run the Tauri desktop application with the native shell window:
```bash
cd apps/desktop
npm run tauri dev
```

---

## 4. Running Tests

### 4.1 Running All Rust Tests
```bash
cargo test --workspace
```

### 4.2 Running Specific Tests
```bash
# Test the CAS version store
cargo test -p changeradar-version-store

# Test the recovery engine
cargo test -p changeradar-recovery

# Test the universal comparison algorithms
cargo test -p changeradar-comparison

# Run end-to-end integration tests
cargo test -p changeradar-core --test e2e_workflow
```

### 4.3 Testing the Frontend Build
```bash
cd apps/desktop
npm run build
```

---

## 5. Code Quality & Formatting

We enforce strict formatting and clippy linter standards:

```bash
# Check code formatting
cargo fmt --all -- --check

# Apply formatting automatically
cargo fmt --all

# Run clippy across all crates and targets
cargo clippy --workspace --all-targets -- -D warnings
```

---

## 6. Architecture Guidelines

1. **Local-First & Zero Telemetry**: Never add code that contacts external hosts, cloud servers, or collects telemetry.
2. **CAS Invariants**: Content-Addressed Storage is write-once and read-many. Never mutate an existing object in `objects/ab/cd/<hash>.zst`.
3. **Atomic Operations**: Any file write or recovery action must write to a temporary file first, recalculate/verify its checksum, and atomically replace the destination.
4. **Bounded Memory**: Never read an entire file into memory using `std::fs::read` without checking its size or using streaming buffers (`BufReader` with 64 KB chunks).
