# ChangeRadar Privacy Policy & Architecture Guarantees

**Last Updated:** October 2026  
**Status:** Strictly Local-First, Zero-Telemetry Default

ChangeRadar is engineered from the ground up to respect user privacy and sovereign ownership of personal and business data.

---

## 1. Core Principles

1. **Local-First & Offline-Only:** ChangeRadar operates completely without an internet connection. It never requires a cloud account, subscription, or external registration.
2. **Zero Telemetry by Default:** The application contains zero tracking, analytics, phone-home beacons, or usage reporting. 
3. **No Cloud Uploads:** Your monitored files, historical revisions, and content hashes remain strictly on your local disk at all times.
4. **Secret-Free Logging:** Application diagnostics and audit logs never record file contents, encryption keys, or private document text.

---

## 2. What Data Is Stored and Where

ChangeRadar stores all data inside standard operating system application directories:

| Operating System | Default Data Directory |
| :--- | :--- |
| **Windows 10 / 11** | `%LOCALAPPDATA%\ChangeRadar\` |
| **Linux** | `~/.local/share/changeradar/` |
| **macOS** | `~/Library/Application Support/ChangeRadar/` |

### Internal Layout:

```text
ChangeRadar/
├── database/
│   └── changeradar.db        # SQLite database (metadata, events, snapshots)
├── objects/
│   ├── ab/
│   │   └── cd/
│   │       └── <hash>.zst    # Content-Addressed Storage (Zstandard compressed revisions)
├── logs/                     # Internal engine log files
├── exports/                  # User-requested export archives
└── backups/                  # Pre-restore safety backups
```

---

## 3. Cryptographic Content Hashing

ChangeRadar uses the **BLAKE3** cryptographic hashing algorithm to detect identical content and verify storage integrity:
- Hashes are calculated through streaming buffers (64KB chunks).
- Entire files are never loaded into RAM unnecessarily.
- Hashes are stored in SQLite and used as addresses for content deduplication.

---

## 4. Local Developer API

The ChangeRadar Background Daemon exposes an optional Developer API on port `41820`:
- **Host Binding:** Binds strictly to `127.0.0.1` (localhost).
- **Network Isolation:** Remote network connections are rejected by default.
- **Authentication:** Privileged operations require local token authentication.

---

## 5. Plugin Sandboxing

ChangeRadar third-party plugins execute within bounded permission interfaces:
- Plugins cannot access arbitrary filesystem locations outside monitored folders.
- Plugins cannot make outbound network requests without explicit capability declarations.

---

## 6. Data Deletion & Portability

- **Full Control:** Deleting a monitored folder or historical version removes records from SQLite and purges unreferenced objects from disk via safe garbage collection.
- **Complete Erasure:** Deleting the `ChangeRadar/` folder entirely removes all stored history and metadata without leaving hidden background files.
- **Portability:** Revisions and archives can be exported and imported freely without vendor lock-in.
