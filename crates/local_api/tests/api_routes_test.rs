use axum::{
    body::Body,
    http::{header::AUTHORIZATION, Request, StatusCode},
};
use changeradar_database::Database;
use changeradar_intelligence::IntelligenceEngine;
use changeradar_local_api::{create_router, start_api_server, ApiState};
use changeradar_plugin_api::PluginRegistry;
use changeradar_recovery::RecoveryCenter;
use changeradar_search::SearchEngine;
use changeradar_snapshot_engine::SnapshotEngine;
use changeradar_storage_manager::StorageManager;
use changeradar_types::{ChangeKind, FileRecord, MonitoredFolder};
use changeradar_version_store::VersionStore;
use chrono::Utc;
use serde_json::Value;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_API_TOKEN: &str = "cr-test-auth-token-987654";

fn setup_test_state() -> (ApiState, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary test directory");
    let db_path = temp_dir.path().join("changeradar_test.db");
    let db = Database::open(&db_path).expect("Failed to initialize test SQLite DB");

    let objects_dir = temp_dir.path().join("objects");
    let store = VersionStore::new(&objects_dir, db.clone()).expect("Failed to init VersionStore");

    let intelligence = IntelligenceEngine::new(db.clone());
    let recovery = RecoveryCenter::new(db.clone(), store.clone());
    let snapshots = SnapshotEngine::new(db.clone(), recovery.clone());
    let storage = StorageManager::new(db.clone(), store.clone());
    let search = SearchEngine::new(db.clone());
    let plugins = Arc::new(PluginRegistry::new());

    let state = ApiState {
        db,
        store,
        intelligence,
        recovery,
        snapshots,
        storage,
        search,
        plugins,
        api_token: TEST_API_TOKEN.to_string(),
    };

    (state, temp_dir)
}

#[tokio::test]
async fn test_route_registration_does_not_panic() {
    let (state, _temp) = setup_test_state();
    // In Axum 0.8, any invalid route segment (such as ':id') causes create_router to panic immediately.
    // This test asserts that create_router completes successfully.
    let router = create_router(state);
    drop(router);
}

#[tokio::test]
async fn test_authentication_middleware_rejects_missing_token() {
    let (state, _temp) = setup_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/api/v1/folders")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["error"], "Unauthorized");
    assert_eq!(
        json["message"],
        "Missing or invalid Bearer authentication token"
    );
}

#[tokio::test]
async fn test_authentication_middleware_rejects_invalid_token() {
    let (state, _temp) = setup_test_state();
    let app = create_router(state);

    let req = Request::builder()
        .uri("/api/v1/folders")
        .method("GET")
        .header(AUTHORIZATION, "Bearer invalid-wrong-token")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["error"], "Unauthorized");
}

#[tokio::test]
async fn test_all_14_documented_api_endpoints() {
    let (state, temp) = setup_test_state();

    // Seed test data in SQLite
    let test_folder =
        MonitoredFolder::new("C:/TestWatchFolder".to_string(), "Test Folder".to_string());
    let folder_id = test_folder.id.clone();
    state.db.add_folder(&test_folder).unwrap();

    let test_file = FileRecord {
        id: Uuid::new_v4().to_string(),
        folder_id: folder_id.clone(),
        relative_path: "document.txt".to_string(),
        file_name: "document.txt".to_string(),
        extension: "txt".to_string(),
        file_size: 27,
        current_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(),
        is_deleted: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let file_id = test_file.id.clone();
    state.db.upsert_file(&test_file).unwrap();

    let version_1 = state
        .store
        .store_bytes(
            b"Hello ChangeRadar Version 1",
            &file_id,
            ChangeKind::Created,
            None,
        )
        .unwrap();
    let v1_id = version_1.id.clone();

    let version_2 = state
        .store
        .store_bytes(
            b"Hello ChangeRadar Version 2 with updates",
            &file_id,
            ChangeKind::Modified,
            None,
        )
        .unwrap();
    let v2_id = version_2.id.clone();

    let auth_header = format!("Bearer {}", TEST_API_TOKEN);

    // =========================================================================
    // 1. GET /api/v1/health
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/health")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["engine_state"], "running");
    }

    // =========================================================================
    // 2. GET /api/v1/folders
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/folders")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // 3. POST /api/v1/folders
    // =========================================================================
    let new_folder_id = {
        let app = create_router(state.clone());
        let payload = serde_json::json!({
            "path": "C:/NewMonitoredProject",
            "name": "NewProject"
        });
        let req = Request::builder()
            .uri("/api/v1/folders")
            .method("POST")
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["display_name"], "NewProject");
        body["id"].as_str().unwrap().to_string()
    };

    // =========================================================================
    // 4. DELETE /api/v1/folders/{id} (Tests route capture syntax `{id}`)
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri(format!("/api/v1/folders/{}", new_folder_id))
            .method("DELETE")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
    }

    // =========================================================================
    // 5. GET /api/v1/files
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/files")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 1);
    }

    // =========================================================================
    // 6. GET /api/v1/files/{id}/versions (Tests route capture syntax `{id}`)
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri(format!("/api/v1/files/{}/versions", file_id))
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 2);
    }

    // =========================================================================
    // 7. GET /api/v1/events
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/events?limit=10")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body.is_array());
    }

    // =========================================================================
    // 8. GET /api/v1/snapshots
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/snapshots")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body.is_array());
    }

    // =========================================================================
    // 9. POST /api/v1/snapshots
    // =========================================================================
    {
        let app = create_router(state.clone());
        let payload = serde_json::json!({
            "folder_id": folder_id,
            "name": "Pre-Release Snapshot",
            "description": "Checkpoint before refactor"
        });
        let req = Request::builder()
            .uri("/api/v1/snapshots")
            .method("POST")
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["name"], "Pre-Release Snapshot");
    }

    // =========================================================================
    // 10. POST /api/v1/compare
    // =========================================================================
    {
        let app = create_router(state.clone());
        let payload = serde_json::json!({
            "version_a_id": v1_id,
            "version_b_id": v2_id,
            "mode": "unified"
        });
        let req = Request::builder()
            .uri("/api/v1/compare")
            .method("POST")
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["comparator_type"], "TEXT");
        assert!(body["text_diff"].is_object());
    }

    // =========================================================================
    // 11. POST /api/v1/recovery/restore
    // =========================================================================
    {
        let app = create_router(state.clone());
        let restore_target = temp.path().join("restored_doc.txt");
        let payload = serde_json::json!({
            "version_id": v1_id,
            "target_path": restore_target.to_str().unwrap()
        });
        let req = Request::builder()
            .uri("/api/v1/recovery/restore")
            .method("POST")
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
        assert!(restore_target.exists());
    }

    // =========================================================================
    // 12. GET /api/v1/intelligence/score
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/intelligence/score")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body["score"].is_number());
        assert!(body["score_components"].is_object());
    }

    // =========================================================================
    // 13. GET /api/v1/storage
    // =========================================================================
    {
        let app = create_router(state.clone());
        let req = Request::builder()
            .uri("/api/v1/storage")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body["total_objects"].is_number());
    }

    // =========================================================================
    // 14. POST /api/v1/storage/cleanup
    // =========================================================================
    {
        let app = create_router(state.clone());
        let payload = serde_json::json!({
            "dry_run": true
        });
        let req = Request::builder()
            .uri("/api/v1/storage/cleanup")
            .method("POST")
            .header("Content-Type", "application/json")
            .header(AUTHORIZATION, &auth_header)
            .body(Body::from(payload.to_string()))
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        assert!(body["versions_pruned"].is_number());
        assert!(body["objects_deleted"].is_number());
        assert!(body["bytes_freed"].is_number());
    }
}

#[tokio::test]
async fn test_live_server_starts_and_remains_running() {
    let (state, _temp) = setup_test_state();

    // Bind to an ephemeral port (port 0 lets the OS pick an available port)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // release so start_api_server can bind to it

    let state_clone = state.clone();
    let server_task = tokio::spawn(async move {
        let _ = start_api_server(state_clone, port).await;
    });

    // Allow the server task to bind and enter the event loop
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Verify server task is still active and has NOT panicked
    assert!(
        !server_task.is_finished(),
        "start_api_server task must remain running without panicking"
    );

    // Verify TCP connectivity to the API port
    let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port)).await;
    assert!(
        stream.is_ok(),
        "Must be able to establish a TCP stream to the running API server"
    );

    // Send HTTP GET /api/v1/health over the raw TCP connection
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = stream.unwrap();
    let request_str = format!(
        "GET /api/v1/health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAuthorization: Bearer {}\r\nConnection: close\r\n\r\n",
        port, TEST_API_TOKEN
    );
    stream.write_all(request_str.as_bytes()).await.unwrap();

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).await.unwrap();
    let response_str = String::from_utf8_lossy(&response_bytes);

    assert!(
        response_str.starts_with("HTTP/1.1 200 OK"),
        "Live API server should return HTTP/1.1 200 OK on /api/v1/health, got: {}",
        response_str
    );
    assert!(
        response_str.contains("\"status\":\"healthy\""),
        "Live response must contain healthy status payload"
    );

    // Terminate test server cleanly
    server_task.abort();
}
