pub use changeradar_comparison::UniversalComparator;
pub use changeradar_configuration::{AppConfig, AppPaths};
pub use changeradar_database::{Database, DatabaseError};
pub use changeradar_event_engine::{EventProcessor, ReconciliationSummary};
pub use changeradar_hashing::{
    hash_bytes, hash_file, perceptual_hash_image, perceptual_similarity,
};
pub use changeradar_intelligence::{ActivityStats, IntelligenceEngine};
pub use changeradar_local_api::{start_api_server, ApiState};
pub use changeradar_notification_engine::NotificationEngine;
pub use changeradar_plugin_api::{ChangeRadarPlugin, PluginManifest, PluginRegistry};
pub use changeradar_recovery::RecoveryCenter;
pub use changeradar_search::{SearchEngine, SearchResults};
pub use changeradar_snapshot_engine::{SnapshotComparison, SnapshotEngine};
pub use changeradar_storage_manager::{CleanupPreview, RetentionPolicy, StorageManager};
pub use changeradar_types::*;
pub use changeradar_version_store::{GarbageCollectionReport, IntegrityReport, VersionStore};
pub use changeradar_watcher::{FolderWatcherManager, RawFsEvent};

use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Version store error: {0}")]
    VersionStore(#[from] changeradar_version_store::VersionStoreError),
    #[error("Configuration error: {0}")]
    Config(#[from] changeradar_configuration::ConfigError),
    #[error("Watcher error: {0}")]
    Watcher(#[from] changeradar_watcher::WatcherError),
    #[error("Recovery error: {0}")]
    Recovery(#[from] changeradar_recovery::RecoveryError),
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] changeradar_snapshot_engine::SnapshotError),
    #[error("Search error: {0}")]
    Search(#[from] changeradar_search::SearchError),
    #[error("Intelligence error: {0}")]
    Intelligence(#[from] changeradar_intelligence::IntelligenceError),
    #[error("Storage error: {0}")]
    Storage(#[from] changeradar_storage_manager::StorageManagerError),
    #[error("Folder not found: {0}")]
    FolderNotFound(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoctorReport {
    pub database_integrity: String,
    pub database_path: String,
    pub storage_stats: StorageStats,
    pub cas_integrity: IntegrityReport,
    pub monitored_folders_count: usize,
    pub active_folders_count: usize,
    pub total_tracked_files: u64,
    pub platform: String,
    pub all_healthy: bool,
}

#[derive(Clone)]
pub struct ChangeRadarEngine {
    pub paths: AppPaths,
    pub config: Arc<RwLock<AppConfig>>,
    pub db: Database,
    pub store: VersionStore,
    pub storage_mgr: StorageManager,
    pub watcher_mgr: Arc<FolderWatcherManager>,
    pub event_processor: EventProcessor,
    pub recovery: RecoveryCenter,
    pub snapshots: SnapshotEngine,
    pub intelligence: IntelligenceEngine,
    pub notifications: NotificationEngine,
    pub search: SearchEngine,
    pub plugins: Arc<PluginRegistry>,
    #[allow(dead_code)]
    is_monitoring: Arc<RwLock<bool>>,
}

impl ChangeRadarEngine {
    /// Initialize full ChangeRadar engine at default or specified data directory.
    pub fn initialize(custom_root: Option<&Path>) -> Result<Self, EngineError> {
        let paths = match custom_root {
            Some(p) => AppPaths::from_root(p)?,
            None => AppPaths::resolve()?,
        };

        let config = paths.load_config();
        let db = Database::open(&paths.db_path)?;
        let store = VersionStore::new(&paths.objects_dir, db.clone())?;
        let storage_mgr = StorageManager::new(db.clone(), store.clone());

        let (event_tx, mut event_rx) = mpsc::channel::<RawFsEvent>(2048);
        let watcher_mgr = Arc::new(FolderWatcherManager::new(event_tx));
        let event_processor = EventProcessor::new(db.clone(), store.clone());

        let recovery = RecoveryCenter::new(db.clone(), store.clone());
        let snapshots = SnapshotEngine::new(db.clone(), recovery.clone());
        let intelligence = IntelligenceEngine::new(db.clone());
        let notifications = NotificationEngine::new(db.clone());
        let search = SearchEngine::new(db.clone());
        let plugins = Arc::new(PluginRegistry::default());

        let engine = Self {
            paths,
            config: Arc::new(RwLock::new(config)),
            db: db.clone(),
            store: store.clone(),
            storage_mgr,
            watcher_mgr,
            event_processor: event_processor.clone(),
            recovery,
            snapshots,
            intelligence,
            notifications: notifications.clone(),
            search,
            plugins: plugins.clone(),
            is_monitoring: Arc::new(RwLock::new(true)),
        };

        // Spawn background event processing loop
        let notifs_clone = notifications;
        let proc_clone = event_processor;
        let plugins_clone = plugins;
        let store_clone = store;

        tokio::spawn(async move {
            info!("Event processing loop started");
            while let Some(raw) = event_rx.recv().await {
                let path_str = raw.path.to_string_lossy().to_string();
                match proc_clone.process_raw_event(raw) {
                    Ok(Some(ev)) => {
                        let _ = notifs_clone.evaluate_event(&ev);

                        // Trigger plugins if content modified
                        if ev.event_type == ChangeKind::Modified
                            || ev.event_type == ChangeKind::Created
                        {
                            if let (Some(new_h), Some(old_h)) = (ev.new_hash, ev.old_hash) {
                                if let (Ok(new_b), Ok(old_b)) = (
                                    store_clone.get_version_bytes_by_hash(&new_h),
                                    store_clone.get_version_bytes_by_hash(&old_h),
                                ) {
                                    let results = plugins_clone
                                        .run_plugins_on_change(&path_str, &old_b, &new_b);
                                    for (_pid, msg) in results {
                                        let _ = notifs_clone.notify_system(
                                            "Plugin Alert",
                                            &msg,
                                            Priority::Normal,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error processing filesystem event: {}", e);
                    }
                }
            }
        });

        // Auto-start monitoring existing active folders
        let folders = db.list_folders().unwrap_or_default();
        for f in folders {
            if f.is_active {
                let _ = engine.watcher_mgr.watch_folder(f);
            }
        }

        info!("ChangeRadar Engine initialized successfully");
        Ok(engine)
    }

    // ==========================================
    // FOLDER OPERATIONS
    // ==========================================

    pub fn add_folder(
        &self,
        path: &str,
        name: Option<&str>,
    ) -> Result<MonitoredFolder, EngineError> {
        let p = PathBuf::from(path);
        let display_name =
            name.unwrap_or_else(|| p.file_name().and_then(|s| s.to_str()).unwrap_or("Folder"));

        let folder =
            MonitoredFolder::new(p.to_string_lossy().to_string(), display_name.to_string());
        self.db.add_folder(&folder)?;
        self.watcher_mgr.watch_folder(folder.clone())?;

        // Run initial reconciliation
        let _ = self.event_processor.reconcile_folder(&folder.id);

        self.db.record_audit_event(
            "FOLDER_ADDED",
            &format!(
                "Added monitored folder '{}' at {}",
                folder.display_name, folder.path
            ),
        )?;

        Ok(folder)
    }

    pub fn remove_folder(&self, folder_id: &str) -> Result<(), EngineError> {
        let _ = self.watcher_mgr.unwatch_folder(folder_id);
        self.db.delete_folder(folder_id)?;
        self.db.record_audit_event(
            "FOLDER_REMOVED",
            &format!("Removed monitored folder {}", folder_id),
        )?;
        Ok(())
    }

    pub fn list_folders(&self) -> Result<Vec<MonitoredFolder>, EngineError> {
        let folders = self.db.list_folders()?;
        Ok(folders)
    }

    pub fn pause_folder(&self, folder_id: &str) -> Result<(), EngineError> {
        self.watcher_mgr.unwatch_folder(folder_id)?;
        self.db.set_folder_active(folder_id, false)?;
        Ok(())
    }

    pub fn resume_folder(&self, folder_id: &str) -> Result<(), EngineError> {
        let folder = self
            .db
            .get_folder(folder_id)?
            .ok_or_else(|| EngineError::FolderNotFound(folder_id.to_string()))?;

        self.db.set_folder_active(folder_id, true)?;
        self.watcher_mgr.watch_folder(folder.clone())?;
        let _ = self.event_processor.reconcile_folder(folder_id);
        Ok(())
    }

    // ==========================================
    // DIAGNOSTICS & DOCTOR
    // ==========================================

    pub fn run_doctor(&self) -> Result<DoctorReport, EngineError> {
        let db_integ = self.db.run_integrity_check()?;
        let storage_stats = self.db.get_storage_stats()?;
        let cas_integ = self.store.verify_integrity()?;
        let folders = self.db.list_folders()?;
        let active_folders = folders.iter().filter(|f| f.is_active).count();
        let total_files = self.db.count_tracked_files()?;

        let all_healthy = db_integ == "ok" && cas_integ.corrupted_count == 0;

        Ok(DoctorReport {
            database_integrity: db_integ,
            database_path: self.paths.db_path.to_string_lossy().to_string(),
            storage_stats,
            cas_integrity: cas_integ,
            monitored_folders_count: folders.len(),
            active_folders_count: active_folders,
            total_tracked_files: total_files,
            platform: changeradar_platform::get_platform_info(),
            all_healthy,
        })
    }
}
