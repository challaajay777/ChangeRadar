use changeradar_comparison::UniversalComparator;
use changeradar_core::{
    ChangeRadarEngine, CleanupPreview, DoctorReport, PluginManifest, RetentionPolicy,
};
use changeradar_types::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;

pub struct DesktopState {
    pub engine: ChangeRadarEngine,
}

#[tauri::command]
fn get_overview(state: State<Arc<DesktopState>>) -> Result<SystemOverview, String> {
    state
        .engine
        .intelligence
        .get_overview(true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_folders(state: State<Arc<DesktopState>>) -> Result<Vec<MonitoredFolder>, String> {
    state.engine.list_folders().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_folder(
    path: String,
    name: Option<String>,
    state: State<Arc<DesktopState>>,
) -> Result<MonitoredFolder, String> {
    state
        .engine
        .add_folder(&path, name.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_folder(id: String, state: State<Arc<DesktopState>>) -> Result<(), String> {
    state.engine.remove_folder(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn pause_folder(id: String, state: State<Arc<DesktopState>>) -> Result<(), String> {
    state.engine.pause_folder(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn resume_folder(id: String, state: State<Arc<DesktopState>>) -> Result<(), String> {
    state.engine.resume_folder(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_events(
    folder_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<FileEvent>, String> {
    state
        .engine
        .db
        .list_events(
            folder_id.as_deref(),
            limit.unwrap_or(100),
            offset.unwrap_or(0),
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_files(
    folder_id: Option<String>,
    limit: Option<usize>,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<FileRecord>, String> {
    state
        .engine
        .db
        .list_files(folder_id.as_deref(), limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_file_versions(
    file_id: String,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<FileVersion>, String> {
    state
        .engine
        .db
        .list_versions_for_file(&file_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn compare_versions(
    version_a_id: String,
    version_b_id: String,
    state: State<Arc<DesktopState>>,
) -> Result<ComparisonOutput, String> {
    let bytes_a = state
        .engine
        .store
        .get_version_bytes(&version_a_id)
        .map_err(|e| e.to_string())?;

    let bytes_b = state
        .engine
        .store
        .get_version_bytes(&version_b_id)
        .map_err(|e| e.to_string())?;

    let ver_a = state.engine.db.get_version(&version_a_id).ok().flatten();
    let file = ver_a
        .as_ref()
        .and_then(|v| state.engine.db.get_file(&v.file_id).ok().flatten());
    let file_name = file
        .map(|f| f.file_name)
        .unwrap_or_else(|| "file.txt".to_string());

    Ok(UniversalComparator::compare(&file_name, &bytes_a, &bytes_b))
}

#[tauri::command]
fn list_snapshots(
    folder_id: Option<String>,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<Snapshot>, String> {
    state
        .engine
        .db
        .list_snapshots(folder_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn create_snapshot(
    name: String,
    description: Option<String>,
    folder_id: Option<String>,
    state: State<Arc<DesktopState>>,
) -> Result<Snapshot, String> {
    state
        .engine
        .snapshots
        .create_snapshot(folder_id.as_deref(), &name, description.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_snapshot(
    snapshot_id: String,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<RecoveryResult>, String> {
    state
        .engine
        .snapshots
        .restore_snapshot(&snapshot_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn recovery_preflight(
    version_id: String,
    destination: Option<String>,
    state: State<Arc<DesktopState>>,
) -> Result<RecoveryPreflight, String> {
    let dest_path = destination.as_ref().map(PathBuf::from);
    state
        .engine
        .recovery
        .preflight_check(&version_id, dest_path.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn recovery_restore(
    version_id: String,
    destination: Option<String>,
    state: State<Arc<DesktopState>>,
) -> Result<RecoveryResult, String> {
    let dest_path = destination.as_ref().map(PathBuf::from);
    state
        .engine
        .recovery
        .restore_version(&version_id, dest_path.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_storage_stats(state: State<Arc<DesktopState>>) -> Result<StorageStats, String> {
    state
        .engine
        .storage_mgr
        .get_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_cleanup(
    max_versions: usize,
    max_age_days: i64,
    state: State<Arc<DesktopState>>,
) -> Result<CleanupPreview, String> {
    let policy = RetentionPolicy {
        max_versions_per_file: Some(max_versions),
        max_age_days: Some(max_age_days),
        max_storage_bytes: None,
    };
    state
        .engine
        .storage_mgr
        .preview_cleanup(&policy)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn search_query(
    query: String,
    limit: Option<usize>,
    state: State<Arc<DesktopState>>,
) -> Result<changeradar_core::SearchResults, String> {
    state
        .engine
        .search
        .search(&query, limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_notifications(
    limit: Option<usize>,
    state: State<Arc<DesktopState>>,
) -> Result<Vec<NotificationItem>, String> {
    state
        .engine
        .db
        .list_notifications(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn mark_notification_read(id: String, state: State<Arc<DesktopState>>) -> Result<(), String> {
    state
        .engine
        .db
        .mark_notification_read(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_plugins(state: State<Arc<DesktopState>>) -> Result<Vec<PluginManifest>, String> {
    Ok(state.engine.plugins.list_manifests())
}

#[tauri::command]
fn run_doctor(state: State<Arc<DesktopState>>) -> Result<DoctorReport, String> {
    state.engine.run_doctor().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_activity_stats(
    state: State<Arc<DesktopState>>,
) -> Result<changeradar_core::ActivityStats, String> {
    state
        .engine
        .intelligence
        .get_activity_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal_in_explorer(path: String) -> Result<(), String> {
    changeradar_platform::reveal_in_explorer(Path::new(&path)).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine =
        ChangeRadarEngine::initialize(None).expect("Failed to initialize ChangeRadar Engine");
    let state = Arc::new(DesktopState { engine });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_overview,
            list_folders,
            add_folder,
            remove_folder,
            pause_folder,
            resume_folder,
            list_events,
            list_files,
            list_file_versions,
            compare_versions,
            list_snapshots,
            create_snapshot,
            restore_snapshot,
            recovery_preflight,
            recovery_restore,
            get_storage_stats,
            preview_cleanup,
            search_query,
            list_notifications,
            mark_notification_read,
            list_plugins,
            run_doctor,
            get_activity_stats,
            reveal_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running ChangeRadar desktop application");
}
