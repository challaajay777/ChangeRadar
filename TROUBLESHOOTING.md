# ChangeRadar Troubleshooting Guide

This guide provides diagnostic solutions for common configuration, operating system, and filesystem issues encountered when running ChangeRadar.

---

## 1. Running Self-Diagnostics

The quickest way to identify issues is to run the built-in diagnostic tool via CLI or Desktop:

```bash
changeradar doctor
```

In the Desktop UI, navigate to the **Diagnostics** tab in the sidebar.

The doctor utility checks:
- SQLite database connectivity, schema integrity, and WAL mode status.
- CAS object directory permissions and corruption checks.
- Filesystem watcher capability and event channel health.
- Available disk space versus configured storage quota.
- Background daemon connectivity.

---

## 2. Common Issues & Solutions

### 2.1 Linux: "OS Error: No space left on device" (inotify limit reached)

**Symptoms:**
When monitoring directories with thousands of subfolders on Linux, the daemon fails to register watchers.

**Cause:**
Linux kernel limits the number of directories that can be watched simultaneously via `fs.inotify.max_user_watches`.

**Resolution:**
Increase the inotify watches limit:
```bash
# Check current limit
cat /proc/sys/fs/inotify/max_user_watches

# Temporarily increase limit
sudo sysctl fs.inotify.max_user_watches=524288

# Make the change permanent
echo "fs.inotify.max_user_watches=524288" | sudo tee -a /etc/sysctl.d/99-changeradar.conf
sudo sysctl --system
```

---

### 2.2 Windows: "The system cannot find the path specified" (Long Path Limitations)

**Symptoms:**
Files nested deep in directory hierarchies (>260 characters) fail to index or restore.

**Cause:**
Legacy Win32 API 260-character path limit (`MAX_PATH`).

**Resolution:**
Enable Win32 Long Paths in Windows:
1. Open PowerShell as Administrator.
2. Run:
   ```powershell
   New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
     -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
   ```
3. Restart Windows.

---

### 2.3 Files Inactive or Changes Not Detected

**Symptoms:**
Saving changes in an application does not generate new versions in ChangeRadar.

**Potential Causes:**
1. **Ignore Rules**: Check whether the file matches default ignore patterns (e.g., `.git/`, temporary files, `~*`, `*.tmp`).
2. **Paused Folder**: Ensure monitoring is not paused for that folder in the **Folders** page.
3. **Atomic Writes with Quick Deletion**: Some editors save by creating a `.tmp` file, unlinking the original, and renaming. ChangeRadar handles coalescing, but if the save was aborted, no modification event is emitted.

**Resolution:**
Trigger a manual reconciliation scan:
```bash
changeradar folder list
# Force reconciliation check
changeradar doctor --reconcile
```

---

### 2.4 "Database is locked" (SQLite Busy Timeout)

**Symptoms:**
Log messages showing `sqlite3_step failed: database is locked`.

**Cause:**
Multiple processes attempting concurrent writes without WAL mode, or an unclosed transaction during intensive recovery.

**Resolution:**
ChangeRadar enables WAL mode by default. To reset any lingering locks:
1. Stop the daemon:
   ```bash
   pkill changeradard  # Linux/macOS
   taskkill /F /IM changeradard.exe  # Windows
   ```
2. Check SQLite database integrity:
   ```bash
   sqlite3 "%LOCALAPPDATA%\ChangeRadar\database\changeradar.db" "PRAGMA integrity_check;"
   ```
3. Restart ChangeRadar.

---

### 2.5 Storage Quota Exceeded

**Symptoms:**
Warning notifications appear stating that the storage quota has been reached. Version archiving is paused.

**Resolution:**
1. Inspect current storage consumption:
   ```bash
   changeradar storage
   ```
2. Trigger an immediate cleanup based on retention rules:
   ```bash
   changeradar cleanup --execute
   ```
3. Adjust the quota limit in `%LOCALAPPDATA%\ChangeRadar\config.json`:
   ```json
   {
     "storage": {
       "quota_limit_bytes": 107374182400
     }
   }
   ```
   (Example: 100 GB = 107,374,182,400 bytes).

---

### 2.6 Corrupted CAS Object Detected

**Symptoms:**
A file restoration fails with `ChecksumMismatchError`.

**Cause:**
Underlying disk corruption, bad sectors, or external process tampering with files inside `storage/objects/`.

**Resolution:**
1. Run integrity scan:
   ```bash
   changeradar doctor --verify-cas
   ```
2. The report will list any corrupted objects. ChangeRadar will flag the affected version in SQLite as unrecoverable while preserving unaffected historical versions.
