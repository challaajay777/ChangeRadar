pub mod ignore;

use changeradar_types::{ChangeKind, MonitoredFolder};
use ignore::IgnoreMatcher;
use notify::{
    event::{ModifyKind, RenameMode},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Folder already watched: {0}")]
    AlreadyWatched(String),
    #[error("Folder not watched: {0}")]
    NotWatched(String),
}

#[derive(Debug, Clone)]
pub struct RawFsEvent {
    pub folder_id: String,
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub change_kind: ChangeKind,
}

pub struct ActiveWatcher {
    #[allow(dead_code)]
    folder: MonitoredFolder,
    #[allow(dead_code)]
    ignore: IgnoreMatcher,
    _watcher: RecommendedWatcher,
}

pub struct FolderWatcherManager {
    watchers: Arc<RwLock<HashMap<String, ActiveWatcher>>>,
    event_tx: mpsc::Sender<RawFsEvent>,
}

impl FolderWatcherManager {
    pub fn new(event_tx: mpsc::Sender<RawFsEvent>) -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Add and start monitoring a folder recursively.
    pub fn watch_folder(&self, folder: MonitoredFolder) -> Result<(), WatcherError> {
        let mut watchers = self.watchers.write();
        if watchers.contains_key(&folder.id) {
            return Err(WatcherError::AlreadyWatched(folder.id.clone()));
        }

        let folder_id = folder.id.clone();
        let folder_path = PathBuf::from(&folder.path);

        if !folder_path.exists() {
            warn!("Monitored path does not exist on disk: {:?}", folder_path);
        }

        let ignore = IgnoreMatcher::new(&folder.ignore_patterns);
        let ignore_clone = ignore.clone();
        let tx = self.event_tx.clone();
        let fid = folder_id.clone();
        let root_path = folder_path.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    Self::handle_notify_event(&fid, &root_path, &ignore_clone, &tx, event);
                }
                Err(e) => {
                    error!("Filesystem watcher error for folder {}: {}", fid, e);
                }
            },
            Config::default(),
        )?;

        if folder_path.exists() {
            watcher.watch(&folder_path, RecursiveMode::Recursive)?;
            info!("Now watching folder '{}' at {:?}", folder.display_name, folder_path);
        }

        watchers.insert(
            folder_id,
            ActiveWatcher {
                folder,
                ignore,
                _watcher: watcher,
            },
        );

        Ok(())
    }

    /// Stop monitoring a folder and release OS watcher resources.
    pub fn unwatch_folder(&self, folder_id: &str) -> Result<(), WatcherError> {
        let mut watchers = self.watchers.write();
        if watchers.remove(folder_id).is_some() {
            info!("Stopped watching folder {}", folder_id);
            Ok(())
        } else {
            Err(WatcherError::NotWatched(folder_id.to_string()))
        }
    }

    /// List active watched folder IDs.
    pub fn active_folders(&self) -> Vec<String> {
        self.watchers.read().keys().cloned().collect()
    }

    fn handle_notify_event(
        folder_id: &str,
        root_path: &Path,
        ignore: &IgnoreMatcher,
        tx: &mpsc::Sender<RawFsEvent>,
        event: Event,
    ) {
        let paths = event.paths;
        if paths.is_empty() {
            return;
        }

        let target_path = &paths[0];

        // Compute relative path for ignore evaluation
        if let Ok(rel) = target_path.strip_prefix(root_path) {
            if ignore.is_ignored(rel) {
                return;
            }
        } else if ignore.is_ignored(target_path) {
            return;
        }

        let (change_kind, old_path) = match event.kind {
            EventKind::Create(_) => (ChangeKind::Created, None),
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if paths.len() >= 2 => {
                (ChangeKind::Renamed, Some(paths[0].clone()))
            }
            EventKind::Modify(ModifyKind::Name(RenameMode::To)) => (ChangeKind::Renamed, None),
            EventKind::Modify(ModifyKind::Metadata(_)) => (ChangeKind::MetadataChanged, None),
            EventKind::Modify(_) => (ChangeKind::Modified, None),
            EventKind::Remove(_) => (ChangeKind::Deleted, None),
            _ => return,
        };

        let raw = RawFsEvent {
            folder_id: folder_id.to_string(),
            path: if old_path.is_some() && paths.len() >= 2 {
                paths[1].clone()
            } else {
                target_path.clone()
            },
            old_path,
            change_kind,
        };

        let _ = tx.try_send(raw);
    }
}
