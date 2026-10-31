use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin error: {0}")]
    Execution(String),
    #[error("Plugin not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub is_enabled: bool,
}

pub trait ChangeRadarPlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn on_file_changed(&self, path: &str, old_bytes: &[u8], new_bytes: &[u8]) -> Option<String>;
}

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn ChangeRadarPlugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut reg = Self {
            plugins: HashMap::new(),
        };
        reg.register(Arc::new(LogAnalyzerPlugin));
        reg.register(Arc::new(WordCounterPlugin));
        reg
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: Arc<dyn ChangeRadarPlugin>) {
        let m = plugin.manifest();
        info!("Registered plugin '{}' v{}", m.name, m.version);
        self.plugins.insert(m.id, plugin);
    }

    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        self.plugins.values().map(|p| p.manifest()).collect()
    }

    pub fn run_plugins_on_change(
        &self,
        path: &str,
        old_bytes: &[u8],
        new_bytes: &[u8],
    ) -> Vec<(String, String)> {
        let mut results = Vec::new();
        for (id, plugin) in &self.plugins {
            if plugin.manifest().is_enabled {
                if let Some(msg) = plugin.on_file_changed(path, old_bytes, new_bytes) {
                    results.push((id.clone(), msg));
                }
            }
        }
        results
    }
}

// ==========================================
// EXAMPLE PLUGIN 1: LOG ANALYZER
// ==========================================

pub struct LogAnalyzerPlugin;

impl ChangeRadarPlugin for LogAnalyzerPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "plugin-log-analyzer".to_string(),
            name: "Log Shift Analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects new ERROR or WARN entries in log files".to_string(),
            author: "ChangeRadar Team".to_string(),
            capabilities: vec!["analyzer".to_string(), "logs".to_string()],
            is_enabled: true,
        }
    }

    fn on_file_changed(&self, path: &str, old_bytes: &[u8], new_bytes: &[u8]) -> Option<String> {
        if !path.ends_with(".log") && !path.ends_with(".txt") {
            return None;
        }

        let old_text = String::from_utf8_lossy(old_bytes);
        let new_text = String::from_utf8_lossy(new_bytes);

        let old_errors = old_text.matches("ERROR").count();
        let new_errors = new_text.matches("ERROR").count();

        if new_errors > old_errors {
            Some(format!(
                "Log Analyzer detected {} new ERROR entries",
                new_errors - old_errors
            ))
        } else {
            None
        }
    }
}

// ==========================================
// EXAMPLE PLUGIN 2: WORD COUNTER
// ==========================================

pub struct WordCounterPlugin;

impl ChangeRadarPlugin for WordCounterPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "plugin-word-counter".to_string(),
            name: "Document Word Counter".to_string(),
            version: "1.0.0".to_string(),
            description: "Tracks net word changes in Markdown and text documents".to_string(),
            author: "ChangeRadar Team".to_string(),
            capabilities: vec!["analyzer".to_string(), "documents".to_string()],
            is_enabled: true,
        }
    }

    fn on_file_changed(&self, path: &str, old_bytes: &[u8], new_bytes: &[u8]) -> Option<String> {
        if !path.ends_with(".md") && !path.ends_with(".markdown") && !path.ends_with(".txt") {
            return None;
        }

        let old_text = String::from_utf8_lossy(old_bytes);
        let new_text = String::from_utf8_lossy(new_bytes);

        let old_words = old_text.split_whitespace().count();
        let new_words = new_text.split_whitespace().count();

        let diff = (new_words as i64) - (old_words as i64);
        if diff != 0 {
            Some(format!(
                "Word count: {} words ({} net shift)",
                new_words,
                if diff > 0 {
                    format!("+{}", diff)
                } else {
                    format!("{}", diff)
                }
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_plugins_registration() {
        let registry = PluginRegistry::new();
        let manifests = registry.list_manifests();
        assert_eq!(manifests.len(), 2);
        assert!(manifests.iter().any(|m| m.id == "plugin-log-analyzer"));
        assert!(manifests.iter().any(|m| m.id == "plugin-word-counter"));
    }

    #[test]
    fn test_word_counter_plugin_diff() {
        let plugin = WordCounterPlugin;
        let old_text = b"Hello world";
        let new_text = b"Hello beautiful world from ChangeRadar";
        let result = plugin.on_file_changed("doc.md", old_text, new_text);
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.contains("5 words"));
        assert!(msg.contains("+3 net shift"));
    }
}
