use changeradar_core::{
    ChangeRadarEngine, RetentionPolicy, UniversalComparator,
};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_complete_e2e_changeradar_lifecycle() {
    // 1. Setup isolated temporary storage environment
    let tmp = tempdir().unwrap();
    let engine_root = tmp.path().join("changeradar_data");
    let monitored_root = tmp.path().join("my_documents");
    fs::create_dir_all(&monitored_root).unwrap();

    // 2. Initialize Engine
    let engine = ChangeRadarEngine::initialize(Some(&engine_root)).unwrap();

    // 3. Add Monitored Folder
    let folder = engine
        .add_folder(&monitored_root.to_string_lossy(), Some("My Documents"))
        .unwrap();
    assert_eq!(folder.display_name, "My Documents");
    assert!(folder.is_active);

    // 4. Create initial files on disk
    let file1_path = monitored_root.join("report.txt");
    fs::write(&file1_path, b"Version 1: Preliminary Analysis Report\nLine 2 data\n").unwrap();

    let file2_path = monitored_root.join("config.json");
    fs::write(&file2_path, br#"{"app":"ChangeRadar","version":1}"#).unwrap();

    // 5. Run reconciliation to index disk state into SQLite & CAS
    let summary1 = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(summary1.files_created, 2);

    let tracked_files = engine.db.list_files(Some(&folder.id), 10).unwrap();
    assert_eq!(tracked_files.len(), 2);

    let file1_rec = tracked_files
        .iter()
        .find(|f| f.file_name == "report.txt")
        .unwrap();

    let versions_v1 = engine.db.list_versions_for_file(&file1_rec.id).unwrap();
    assert_eq!(versions_v1.len(), 1);
    assert_eq!(versions_v1[0].version_number, 1);
    let v1_id = versions_v1[0].id.clone();
    let v1_hash = versions_v1[0].content_hash.clone();

    // 6. Modify report.txt to produce Version 2
    fs::write(
        &file1_path,
        b"Version 1: Preliminary Analysis Report\nLine 2 data modified\nLine 3 new insights\n",
    )
    .unwrap();

    let summary2 = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(summary2.files_modified, 1);

    let versions_v2 = engine.db.list_versions_for_file(&file1_rec.id).unwrap();
    assert_eq!(versions_v2.len(), 2);
    let v2_id = versions_v2[0].id.clone(); // ordered descending
    assert_eq!(versions_v2[0].version_number, 2);
    assert_ne!(v1_hash, versions_v2[0].content_hash);

    // 7. Test Universal Comparison (Myers text diff)
    let bytes_v1 = engine.store.get_version_bytes(&v1_id).unwrap();
    let bytes_v2 = engine.store.get_version_bytes(&v2_id).unwrap();
    let diff_output = UniversalComparator::compare("report.txt", &bytes_v1, &bytes_v2);
    assert!(diff_output.added_count >= 1);
    assert!(diff_output.text_diff.is_some());

    // 8. Test Snapshot Engine
    let snapshot = engine
        .snapshots
        .create_snapshot(Some(&folder.id), "Pre-Recovery Milestone", Some("Before reverting report"))
        .unwrap();
    assert_eq!(snapshot.file_count, 2);
    assert_eq!(snapshot.name, "Pre-Recovery Milestone");

    // 9. Test Recovery Pre-flight Checks
    let preflight = engine.recovery.preflight_check(&v1_id, None).unwrap();
    assert!(preflight.can_proceed);
    assert!(preflight.cas_object_verified);
    assert!(preflight.destination_exists);

    // 10. Test Safe Reversible Recovery: Restore Version 1
    let recovery_result = engine.recovery.restore_version(&v1_id, None).unwrap();
    assert!(recovery_result.success);
    assert_eq!(recovery_result.restored_hash, v1_hash);
    assert!(recovery_result.backup_version_id.is_some());

    // Verify disk content was restored to Version 1 exactly
    let disk_content = fs::read_to_string(&file1_path).unwrap();
    assert_eq!(
        disk_content,
        "Version 1: Preliminary Analysis Report\nLine 2 data\n"
    );

    // 11. Test Storage Stats and Retention Preview
    let storage_stats = engine.storage_mgr.get_stats().unwrap();
    assert!(storage_stats.total_objects >= 2);
    assert!(storage_stats.compressed_bytes > 0);

    let retention = RetentionPolicy {
        max_versions_per_file: Some(5),
        max_age_days: Some(30),
        max_storage_bytes: None,
    };
    let cleanup_preview = engine.storage_mgr.preview_cleanup(&retention).unwrap();
    assert_eq!(cleanup_preview.candidate_versions_count, 0); // No versions exceed 5 revisions yet

    // 12. Test ChangeRadar Doctor & Diagnostics
    let doctor_report = engine.run_doctor().unwrap();
    assert_eq!(doctor_report.database_integrity, "ok");
    assert_eq!(doctor_report.cas_integrity.corrupted_count, 0);
    assert!(doctor_report.all_healthy);
    assert_eq!(doctor_report.monitored_folders_count, 1);
}
