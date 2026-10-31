# ChangeRadar Installation Guide

ChangeRadar is distributed as a cross-platform desktop application (Tauri 2), a background daemon (`changeradard`), and a command-line interface (`changeradar`).

---

## 1. System Requirements

### Hardware Requirements
| Component | Minimum | Recommended |
|:---|:---|:---|
| **CPU** | 64-bit dual-core processor | 64-bit quad-core processor (x86_64 or ARM64) |
| **RAM** | 512 MB (Daemon/CLI), 2 GB (Desktop UI) | 4 GB+ |
| **Storage** | 100 MB for binaries; storage depends on monitored files | SSD storage for fast CAS and SQLite WAL performance |

### Operating Systems
- **Windows**: Windows 10 (version 1809+) or Windows 11 (64-bit).
- **Linux**: Ubuntu 20.04+, Debian 11+, Fedora 36+, Arch Linux, or any modern glibc 2.31+ distribution.
- **macOS**: macOS 11.0 (Big Sur) or newer (Intel & Apple Silicon).

---

## 2. Installation Options

### 2.1 Installing Pre-built Binaries (Recommended)

#### Windows
1. Download the latest `ChangeRadar-Setup-<version>.exe` or standalone portable zip from GitHub Releases.
2. Run the installer or extract the zip to `C:\Program Files\ChangeRadar\`.
3. Add `C:\Program Files\ChangeRadar\bin` to your user `PATH`.

#### Linux
Download the package for your distribution:
- **Debian / Ubuntu**:
  ```bash
  sudo dpkg -i changeradar_<version>_amd64.deb
  sudo apt-get install -f
  ```
- **Fedora / RHEL**:
  ```bash
  sudo rpm -i changeradar-<version>.x86_64.rpm
  ```
- **AppImage**:
  ```bash
  chmod +x ChangeRadar-<version>.AppImage
  ./ChangeRadar-<version>.AppImage
  ```

#### macOS
1. Download `ChangeRadar-<version>.dmg`.
2. Open the disk image and drag **ChangeRadar** into your `/Applications` directory.
3. For CLI access, symlink the binaries:
   ```bash
   sudo ln -s /Applications/ChangeRadar.app/Contents/MacOS/changeradar /usr/local/bin/changeradar
   sudo ln -s /Applications/ChangeRadar.app/Contents/MacOS/changeradard /usr/local/bin/changeradard
   ```

---

## 3. Building From Source

### 3.1 Prerequisites
- **Rust Toolchain**: 1.75 or newer (`rustup default stable`)
- **Node.js**: v18.0.0 or newer and `npm`
- **C Compiler**:
  - Windows: MSVC (C++ Build Tools) or MinGW GCC (ucrt64)
  - Linux: `build-essential libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev`
  - macOS: Xcode Command Line Tools (`xcode-select --install`)

### 3.2 Compilation Steps

```bash
# 1. Clone the repository
git clone https://github.com/changeradar/changeradar.git
cd ChangeRadar

# 2. Build the Rust workspace (CLI, Daemon, Core, and libraries)
cargo build --release

# 3. Verify built binaries
./target/release/changeradar --version
./target/release/changeradard --version

# 4. Build Desktop UI Frontend
cd apps/desktop
npm install
npm run build
cd ../..

# 5. Build Desktop Tauri Application
cargo tauri build
```

The resulting packages will be in:
- Windows: `target/release/bundle/msi/` or `target/release/bundle/nsis/`
- Linux: `target/release/bundle/deb/` or `target/release/bundle/appimage/`
- macOS: `target/release/bundle/dmg/`

---

## 4. Running the Background Daemon as a Service

### 4.1 Linux (`systemd`)
Create a user systemd service file at `~/.config/systemd/user/changeradard.service`:

```ini
[Unit]
Description=ChangeRadar File Intelligence Daemon
After=network.target

[Service]
ExecStart=/usr/local/bin/changeradard
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
```

Enable and start the service:
```bash
systemctl --user daemon-reload
systemctl --user enable --now changeradard.service
```

### 4.2 macOS (`launchd`)
Create a launch agent at `~/Library/LaunchAgents/com.changeradar.daemon.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.changeradar.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/changeradard</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load the daemon:
```bash
launchctl load ~/Library/LaunchAgents/com.changeradar.daemon.plist
```

### 4.3 Windows (Scheduled Task / Service)
You can register `changeradard.exe` to run silently at user logon using Windows Task Scheduler:

```powershell
$Action = New-ScheduledTaskAction -Execute "C:\Program Files\ChangeRadar\bin\changeradard.exe"
$Trigger = New-ScheduledTaskTrigger -AtLogOn
$Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit 0
Register-ScheduledTask -TaskName "ChangeRadarDaemon" -Action $Action -Trigger $Trigger -Settings $Settings
```

---

## 5. Verification & Health Check

After installation, verify that the ChangeRadar engine and database initialize properly:

```bash
changeradar doctor
```

Output:
```text
ChangeRadar Health & Diagnostic Check
======================================
Database:               OK (C:\Users\ajays\AppData\Local\ChangeRadar\database\changeradar.db)
Storage CAS:            OK (42 objects, 0 corrupted)
Filesystem Watcher:     OK (Active)
Daemon Status:          OK (Running)
Disk Space Available:   128.4 GB (Quota limit: 50.0 GB)
Status: Healthy
```
