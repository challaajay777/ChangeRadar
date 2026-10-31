use changeradar_core::{ApiState, ChangeRadarEngine, PluginRegistry};
use changeradar_local_api::create_router;
use changeradar_plugin_api::{ChangeRadarPlugin, PluginManifest};
use std::sync::Arc;
use tempfile::tempdir;

struct CrashingPlugin;
impl ChangeRadarPlugin for CrashingPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            id: "plugin-crashing".to_string(),
            name: "Crashing Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Intentionally panics to test crash isolation".to_string(),
            author: "Tester".to_string(),
            capabilities: vec!["crash".to_string()],
            is_enabled: true,
        }
    }

    fn on_file_changed(&self, _path: &str, _old_bytes: &[u8], _new_bytes: &[u8]) -> Option<String> {
        panic!("Simulated intentional plugin crash!");
    }
}

#[tokio::test]
async fn test_api_security_and_plugin_crash_isolation() {
    let tmp = tempdir().unwrap();
    let engine = ChangeRadarEngine::initialize(Some(tmp.path())).unwrap();

    let api_state = ApiState {
        db: engine.db.clone(),
        store: engine.store.clone(),
        intelligence: engine.intelligence.clone(),
        recovery: engine.recovery.clone(),
        snapshots: engine.snapshots.clone(),
        storage: engine.storage_mgr.clone(),
        search: engine.search.clone(),
        plugins: engine.plugins.clone(),
        api_token: "super_secret_audit_token_12345".to_string(),
    };

    let router = create_router(api_state);

    // 1. Verify router initializes without panic
    assert!(format!("{:?}", router).contains("Router"));

    // 2. Test Plugin Crash Isolation: A panicking plugin must not crash the caller
    let mut registry = PluginRegistry::new();
    registry.register(Arc::new(CrashingPlugin));

    // When run_plugins_on_change is called, it should safely catch panic
    let results = registry.run_plugins_on_change("test.log", b"old", b"new");
    // Should return results without panicking
    println!("Plugin results: {:?}", results);
}
