# ChangeRadar Plugin Development Guide

ChangeRadar features an extensible, capability-based plugin system that allows developers to write domain-specific analyzers, format comparators, and event handlers while enforcing strict local execution and isolation boundaries.

---

## 1. Plugin Architecture

ChangeRadar plugins execute within a sandboxed runtime environment defined by `crates/plugin_api`. Plugins interact with the engine via strictly declared capabilities and lifecycle hooks.

```
       ┌────────────────────────────────────────────────────────┐
       │                 ChangeRadar Core Engine                │
       └───────────────────────────┬────────────────────────────┘
                                   │ Event Bus / Hook Dispatch
       ┌───────────────────────────▼────────────────────────────┐
       │                  Plugin Runtime Sandbox                 │
       │  ┌──────────────────────┐    ┌──────────────────────┐  │
       │  │  Log Analyzer Plugin │    │  Word Counter Plugin │  │
       │  │ (Flags error spikes) │    │(Tracks document delta│  │
       │  └──────────────────────┘    └──────────────────────┘  │
       └────────────────────────────────────────────────────────┘
```

### Core Security Guarantees
- **No Network Access**: Plugins have zero network permissions. Sockets, HTTP requests, and DNS lookups are strictly prohibited.
- **Capability Isolation**: Plugins only receive data for file types or paths explicitly declared in their manifest.
- **Read-Only Inspection**: Plugins cannot directly mutate or delete monitored files or CAS objects.

---

## 2. Plugin Manifest (`plugin.json`)

Every plugin must provide a valid `plugin.json` manifest:

```json
{
  "id": "com.example.markdown-linter",
  "name": "Markdown Linter & Word Analytics",
  "version": "1.0.0",
  "author": "Jane Doe",
  "description": "Analyzes markdown changes, counts headers, and validates internal links.",
  "entrypoint": "main.wasm",
  "capabilities": [
    "read_file_content",
    "inspect_events",
    "emit_notifications"
  ],
  "file_patterns": [
    "*.md",
    "*.markdown"
  ],
  "settings": {
    "warn_on_broken_links": true,
    "max_heading_depth": 4
  }
}
```

### Manifest Fields
| Field | Type | Description |
|:---|:---|:---|
| `id` | `string` | Unique reverse-DNS identifier (e.g., `com.company.pluginname`) |
| `name` | `string` | Human-readable plugin name displayed in the UI |
| `version` | `string` | Semantic version string (`x.y.z`) |
| `author` | `string` | Author or organization name |
| `description` | `string` | Summary of plugin capabilities |
| `capabilities`| `string[]` | Granted capabilities (see Section 3) |
| `file_patterns`| `string[]`| Glob patterns of monitored files routed to this plugin |
| `settings` | `object` | Configurable options exposed in the ChangeRadar Settings UI |

---

## 3. Declared Capabilities

Plugins must declare only the permissions they require:

- `inspect_events`: Receives notifications when matching files are created, modified, renamed, or deleted.
- `read_file_content`: Allows the plugin to inspect historical or current file content buffers.
- `compute_metrics`: Allows the plugin to register custom metrics into the Change Intelligence dashboard.
- `emit_notifications`: Grants permission to post priority alerts to the user's notification feed.

---

## 4. Lifecycle Hooks

A plugin can implement the following hooks:

### 4.1 `initialize(config: PluginConfig) -> Result<(), PluginError>`
Invoked when the plugin is loaded during engine startup.

### 4.2 `on_event(event: FileEvent) -> Option<PluginAction>`
Invoked whenever a filesystem event matches the plugin's `file_patterns`.

```rust
fn on_event(&self, event: &FileEvent) -> Option<PluginNotification> {
    if event.kind == ChangeKind::Modified && event.path.ends_with(".log") {
        // Inspect log content for fatal error spikes
    }
    None
}
```

### 4.3 `analyze_content(path: &str, content: &[u8]) -> PluginAnalysisResult`
Invoked during indexing to extract custom metadata, word counts, or structural analysis.

---

## 5. Built-in Plugins Reference

ChangeRadar ships with two pre-installed built-in plugins:

### 5.1 Log Analyzer Plugin (`changeradar.builtin.log-analyzer`)
- **Monitored Patterns**: `*.log`, `*.txt`
- **Functionality**:
  - Scans newly written log lines for severity keywords (`ERROR`, `FATAL`, `PANIC`, `CRITICAL`).
  - Computes an error-velocity ratio.
  - Automatically raises a **HIGH** priority notification if more than 20 errors occur within a 30-second window.

### 5.2 Word Counter Plugin (`changeradar.builtin.word-counter`)
- **Monitored Patterns**: `*.md`, `*.txt`, `*.tex`, `*.rst`
- **Functionality**:
  - Calculates total word count, character count, and estimated reading time.
  - Tracks word additions and deletions per revision.
  - Feeds writing velocity data into the Change Intelligence dashboard.

---

## 6. Packaging and Installing Plugins

1. Organize your plugin directory:
   ```
   my-plugin/
   ├── plugin.json
   └── lib.wasm (or native library)
   ```
2. Copy the directory to the ChangeRadar plugins folder:
   - **Windows**: `%LOCALAPPDATA%\ChangeRadar\plugins\my-plugin\`
   - **Linux**: `~/.local/share/changeradar/plugins/my-plugin/`
   - **macOS**: `~/Library/Application Support/ChangeRadar/plugins/my-plugin/`
3. Restart `changeradard` or reopen the ChangeRadar desktop application.
4. Navigate to the **Plugins** section in the desktop UI to verify activation and configure settings.
