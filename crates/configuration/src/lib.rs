use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error in configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_port: u16,
    pub api_token: String,
    pub theme: String, // "dark", "light", "system"
    pub auto_start_monitoring: bool,
    pub default_scan_interval_secs: u64,
    pub retention_days: i64,
    pub max_versions_per_file: usize,
    pub telemetry_enabled: bool, // ALWAYS false by default
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_port: 41820,
            api_token: Uuid::new_v4().to_string(),
            theme: "dark".to_string(),
            auto_start_monitoring: true,
            default_scan_interval_secs: 300,
            retention_days: 90,
            max_versions_per_file: 50,
            telemetry_enabled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub db_path: PathBuf,
    pub objects_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub exports_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub config_file: PathBuf,
}

impl AppPaths {
    /// Resolve standard OS-appropriate application paths.
    pub fn resolve() -> Result<Self, ConfigError> {
        let base_data_dir = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ChangeRadar");

        Self::from_root(&base_data_dir)
    }

    pub fn from_root<P: AsRef<Path>>(root: P) -> Result<Self, ConfigError> {
        let r = root.as_ref().to_path_buf();
        let db_dir = r.join("database");
        let objects_dir = r.join("objects");
        let logs_dir = r.join("logs");
        let exports_dir = r.join("exports");
        let backups_dir = r.join("backups");

        fs::create_dir_all(&db_dir)?;
        fs::create_dir_all(&objects_dir)?;
        fs::create_dir_all(&logs_dir)?;
        fs::create_dir_all(&exports_dir)?;
        fs::create_dir_all(&backups_dir)?;

        Ok(Self {
            root_dir: r.clone(),
            db_path: db_dir.join("changeradar.db"),
            objects_dir,
            logs_dir,
            exports_dir,
            backups_dir,
            config_file: r.join("config.json"),
        })
    }

    pub fn load_config(&self) -> AppConfig {
        if self.config_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_file) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }

        let default_cfg = AppConfig::default();
        let _ = self.save_config(&default_cfg);
        default_cfg
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_file, json)?;
        Ok(())
    }
}
