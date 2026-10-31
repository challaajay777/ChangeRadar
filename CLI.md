# ChangeRadar CLI (`changeradar`) — Command Line Reference

ChangeRadar provides a CLI tool that communicates directly with the same underlying engine, SQLite database, and Content-Addressed Storage as the desktop application.

---

## 1. Global Flags

| Flag | Shorthand | Description |
| :--- | :--- | :--- |
| `--json` | `-j` | Format all output as machine-readable JSON |
| `--data-dir <DIR>` | | Custom data directory instead of default OS path |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

---

## 2. Command Reference

### `changeradar status`
Displays the overall health, active watch trees, tracked file count, changes today, and the transparent Change Radar Score.

```bash
changeradar status
```

**JSON Output Example:**
```bash
changeradar status --json
```

---

### `changeradar folder`
Manage monitored directories.

```bash
# Add a folder to watch
changeradar folder add C:/Users/name/Documents --name "Personal Docs"

# List all configured folders
changeradar folder list

# Pause monitoring on a folder
changeradar folder pause <folder-id>

# Resume monitoring
changeradar folder resume <folder-id>

# Remove folder from monitoring
changeradar folder remove <folder-id>
```

---

### `changeradar history`
View chronological timeline of detected filesystem events.

```bash
# View recent 25 events
changeradar history

# Limit to 50 events for a specific folder
changeradar history --limit 50 --folder <folder-id>
```

---

### `changeradar search <query>`
Unified global search across file names, relative paths, events, and folders.

```bash
changeradar search "quarterly_report"
changeradar search "results.csv" --limit 10
```

---

### `changeradar file`
Inspect tracked files and historical revisions.

```bash
# List tracked files
changeradar file list --limit 100

# View complete version history for a file
changeradar file versions <file-id>
```

---

### `changeradar compare <version-a> <version-b>`
Compute universal semantic or Myers text difference between two historical versions.

```bash
changeradar compare ver-123 ver-456
```

---

### `changeradar snapshot`
Create and restore consistent point-in-time directory snapshots.

```bash
# Create named snapshot
changeradar snapshot create --name "Pre-Update Backup" --description "Prior to major refactor"

# List snapshots
changeradar snapshot list

# Revert entire directory to a snapshot
changeradar snapshot restore <snapshot-id>
```

---

### `changeradar restore <version-id>`
Safely and reversibly restore a historical file version.

```bash
# Restore to original location (pre-flight checks & automatic backup)
changeradar restore <version-id>

# Restore to an alternative destination
changeradar restore <version-id> --dest C:/Temp/recovered_file.txt
```

---

### `changeradar storage`
Inspect Content-Addressed Storage metrics, unique CAS objects, raw vs compressed size, and deduplication ratio.

```bash
changeradar storage
```

---

### `changeradar cleanup`
Preview or execute storage retention rules.

```bash
# Preview what will be removed (WITHOUT deleting anything)
changeradar cleanup --preview --max-age-days 90 --max-versions 50

# Apply cleanup policy and garbage collect unreferenced CAS objects
changeradar cleanup --apply
```

---

### `changeradar doctor`
Run full self-diagnostic tests (SQLite integrity check, CAS cryptographic hash verification, watcher health).

```bash
changeradar doctor
```
