use axum::{
    extract::{Path as AxPath, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use changeradar_comparison::UniversalComparator;
use changeradar_database::Database;
use changeradar_intelligence::IntelligenceEngine;
use changeradar_plugin_api::PluginRegistry;
use changeradar_recovery::RecoveryCenter;
use changeradar_search::SearchEngine;
use changeradar_snapshot_engine::SnapshotEngine;
use changeradar_storage_manager::StorageManager;
use changeradar_types::MonitoredFolder;
use changeradar_version_store::VersionStore;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
pub struct ApiState {
    pub db: Database,
    pub store: VersionStore,
    pub intelligence: IntelligenceEngine,
    pub recovery: RecoveryCenter,
    pub snapshots: SnapshotEngine,
    pub storage: StorageManager,
    pub search: SearchEngine,
    pub plugins: Arc<PluginRegistry>,
    pub api_token: String,
}

pub fn create_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/folders", get(list_folders).post(add_folder))
        .route("/api/v1/folders/:id", delete(delete_folder))
        .route("/api/v1/events", get(list_events))
        .route("/api/v1/files", get(list_files))
        .route("/api/v1/files/:id/versions", get(list_file_versions))
        .route("/api/v1/snapshots", get(list_snapshots).post(create_snapshot))
        .route("/api/v1/snapshots/:id/restore", post(restore_snapshot))
        .route("/api/v1/recovery/preflight", post(recovery_preflight))
        .route("/api/v1/recovery/restore", post(recovery_restore))
        .route("/api/v1/storage", get(get_storage))
        .route("/api/v1/statistics", get(get_statistics))
        .route("/api/v1/search", get(search_handler))
        .route("/api/v1/compare", post(compare_versions))
        .route("/api/v1/plugins", get(list_plugins))
        .layer(cors)
        .with_state(state)
}

pub async fn start_api_server(
    state: ApiState,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Starting ChangeRadar Developer API on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ==========================================
// HANDLERS
// ==========================================

async fn get_status(State(state): State<ApiState>) -> impl IntoResponse {
    match state.intelligence.get_overview(true) {
        Ok(overview) => (StatusCode::OK, Json(serde_json::to_value(overview).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn list_folders(State(state): State<ApiState>) -> impl IntoResponse {
    match state.db.list_folders() {
        Ok(folders) => (StatusCode::OK, Json(serde_json::to_value(folders).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct AddFolderReq {
    path: String,
    display_name: Option<String>,
}

async fn add_folder(
    State(state): State<ApiState>,
    Json(payload): Json<AddFolderReq>,
) -> impl IntoResponse {
    let name = payload.display_name.unwrap_or_else(|| {
        PathBuf::from(&payload.path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Folder")
            .to_string()
    });

    let folder = MonitoredFolder::new(payload.path, name);
    match state.db.add_folder(&folder) {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::to_value(folder).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn delete_folder(
    State(state): State<ApiState>,
    AxPath(id): AxPath<String>,
) -> impl IntoResponse {
    match state.db.delete_folder(&id) {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct PaginationQuery {
    folder_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn list_events(
    State(state): State<ApiState>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    match state
        .db
        .list_events(query.folder_id.as_deref(), limit, offset)
    {
        Ok(events) => (StatusCode::OK, Json(serde_json::to_value(events).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn list_files(
    State(state): State<ApiState>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(200);
    match state.db.list_files(query.folder_id.as_deref(), limit) {
        Ok(files) => (StatusCode::OK, Json(serde_json::to_value(files).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn list_file_versions(
    State(state): State<ApiState>,
    AxPath(file_id): AxPath<String>,
) -> impl IntoResponse {
    match state.db.list_versions_for_file(&file_id) {
        Ok(versions) => (StatusCode::OK, Json(serde_json::to_value(versions).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn list_snapshots(State(state): State<ApiState>) -> impl IntoResponse {
    match state.db.list_snapshots(None) {
        Ok(snaps) => (StatusCode::OK, Json(serde_json::to_value(snaps).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct CreateSnapshotReq {
    folder_id: Option<String>,
    name: String,
    description: Option<String>,
}

async fn create_snapshot(
    State(state): State<ApiState>,
    Json(payload): Json<CreateSnapshotReq>,
) -> impl IntoResponse {
    match state.snapshots.create_snapshot(
        payload.folder_id.as_deref(),
        &payload.name,
        payload.description.as_deref(),
    ) {
        Ok(snap) => (StatusCode::CREATED, Json(serde_json::to_value(snap).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn restore_snapshot(
    State(state): State<ApiState>,
    AxPath(id): AxPath<String>,
) -> impl IntoResponse {
    match state.snapshots.restore_snapshot(&id) {
        Ok(res) => (StatusCode::OK, Json(serde_json::to_value(res).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct PreflightReq {
    version_id: String,
    custom_destination: Option<String>,
}

async fn recovery_preflight(
    State(state): State<ApiState>,
    Json(payload): Json<PreflightReq>,
) -> impl IntoResponse {
    let dest_path = payload.custom_destination.as_ref().map(PathBuf::from);
    match state
        .recovery
        .preflight_check(&payload.version_id, dest_path.as_deref())
    {
        Ok(res) => (StatusCode::OK, Json(serde_json::to_value(res).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn recovery_restore(
    State(state): State<ApiState>,
    Json(payload): Json<PreflightReq>,
) -> impl IntoResponse {
    let dest_path = payload.custom_destination.as_ref().map(PathBuf::from);
    match state
        .recovery
        .restore_version(&payload.version_id, dest_path.as_deref())
    {
        Ok(res) => (StatusCode::OK, Json(serde_json::to_value(res).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_storage(State(state): State<ApiState>) -> impl IntoResponse {
    match state.storage.get_stats() {
        Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_statistics(State(state): State<ApiState>) -> impl IntoResponse {
    match state.intelligence.get_activity_stats() {
        Ok(stats) => (StatusCode::OK, Json(serde_json::to_value(stats).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

async fn search_handler(
    State(state): State<ApiState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    match state.search.search(&query.q, limit) {
        Ok(res) => (StatusCode::OK, Json(serde_json::to_value(res).unwrap())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

#[derive(Deserialize)]
struct CompareReq {
    version_a_id: String,
    version_b_id: String,
}

async fn compare_versions(
    State(state): State<ApiState>,
    Json(payload): Json<CompareReq>,
) -> impl IntoResponse {
    let bytes_a = match state.store.get_version_bytes(&payload.version_a_id) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let bytes_b = match state.store.get_version_bytes(&payload.version_b_id) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    let ver_a = state.db.get_version(&payload.version_a_id).ok().flatten();
    let file = ver_a
        .as_ref()
        .and_then(|v| state.db.get_file(&v.file_id).ok().flatten());
    let file_name = file.map(|f| f.file_name).unwrap_or_else(|| "file.txt".to_string());

    let output = UniversalComparator::compare(&file_name, &bytes_a, &bytes_b);
    (StatusCode::OK, Json(serde_json::to_value(output).unwrap()))
}

async fn list_plugins(State(state): State<ApiState>) -> impl IntoResponse {
    let manifests = state.plugins.list_manifests();
    (StatusCode::OK, Json(serde_json::to_value(manifests).unwrap()))
}
