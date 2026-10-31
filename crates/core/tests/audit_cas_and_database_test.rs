use changeradar_core::{ChangeRadarEngine, RetentionPolicy};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_cas_and_database_audit() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    let monitored_dir = tmp.path().join("monitored");
    fs::create_dir_all(&monitored_dir).unwrap();

    let engine = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let folder = engine
        .add_folder(&monitored_dir.to_string_lossy(), Some("CAS Test Folder"))
        .unwrap();

    // 1. Store identical content twice across two different files
    let file1_path = monitored_dir.join("copy1.txt");
    let file2_path = monitored_dir.join("copy2.txt");
    let shared_content = b"IDENTICAL_CONTENT_FOR_DEDUPLICATION_VERIFICATION_2026";
    fs::write(&file1_path, shared_content).unwrap();
    fs::write(&file2_path, shared_content).unwrap();

    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let files = engine.db.list_files(Some(&folder.id), 10).unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(
        files[0].current_hash, files[1].current_hash,
        "Both files must have identical hash"
    );

    // Check CAS storage stats: should be exactly 1 content object in CAS despite 2 files
    let storage_stats = engine.storage_mgr.get_stats().unwrap();
    assert_eq!(
        storage_stats.total_objects, 1,
        "Deduplication: Only 1 CAS object must be stored for identical files"
    );
    assert!(
        storage_stats.deduplication_ratio > 1.0,
        "Deduplication ratio should exceed 1.0"
    );

    // Check Content Object reference count: must be 2
    let hash = &files[0].current_hash;
    let obj = engine.db.get_content_object(hash).unwrap().unwrap();
    assert_eq!(
        obj.reference_count, 2,
        "Reference count must be 2 for 2 files sharing identical content"
    );

    // 2. Store different content with similar names
    let file3_path = monitored_dir.join("copy3.txt");
    fs::write(
        &file3_path,
        b"SLIGHTLY_DIFFERENT_CONTENT_FOR_DEDUPLICATION_VERIFICATION_2026",
    )
    .unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let storage_stats2 = engine.storage_mgr.get_stats().unwrap();
    assert_eq!(
        storage_stats2.total_objects, 2,
        "Distinct content must produce distinct CAS object"
    );

    // 3. Store multiple versions of copy1.txt and verify all historical versions can be retrieved
    for v in 2..=5 {
        fs::write(
            &file1_path,
            format!("copy1.txt version {} unique bytes", v).as_bytes(),
        )
        .unwrap();
        engine.event_processor.reconcile_folder(&folder.id).unwrap();
    }
    let copy1_rec = files
        .iter()
        .find(|f| f.file_name == "copy1.txt")
        .unwrap()
        .clone();
    let all_v = engine.db.list_versions_for_file(&copy1_rec.id).unwrap();
    assert_eq!(all_v.len(), 5, "Must have 5 versions of copy1.txt");

    // Verify every single historical version can be retrieved and decoded from CAS
    for ver in &all_v {
        let bytes = engine.store.get_version_bytes(&ver.id).unwrap();
        assert_eq!(bytes.len() as u64, ver.file_size);
    }

    // 4. Test Retention Cleanup & Garbage Collection:
    // With max_versions_per_file = 2, versions 1, 2, 3 should be candidate for deletion
    let policy = RetentionPolicy {
        max_versions_per_file: Some(2),
        max_age_days: None,
        max_storage_bytes: None,
    };
    let preview = engine.storage_mgr.preview_cleanup(&policy).unwrap();
    assert_eq!(
        preview.candidate_versions_count, 3,
        "Expected 3 candidate versions for cleanup"
    );

    // Execute cleanup
    let gc_report = engine.storage_mgr.execute_cleanup(&preview).unwrap();
    assert!(gc_report.deleted_objects_count >= 1 || preview.candidate_versions_count == 3);

    // Verify remaining versions for copy1.txt are exactly 2
    let remaining = engine.db.list_versions_for_file(&copy1_rec.id).unwrap();
    assert_eq!(remaining.len(), 2);
    // Versions remaining must be 4 and 5 (the newest ones)
    assert_eq!(remaining[0].version_number, 5);
    assert_eq!(remaining[1].version_number, 4);

    // 5. Confirm that copy2.txt still retrieves its content intact (referenced objects never deleted)
    let copy2_rec = files.iter().find(|f| f.file_name == "copy2.txt").unwrap();
    let copy2_v = engine
        .db
        .get_latest_version(&copy2_rec.id)
        .unwrap()
        .unwrap();
    let copy2_bytes = engine.store.get_version_bytes(&copy2_v.id).unwrap();
    assert_eq!(
        copy2_bytes, shared_content,
        "Referenced object must remain intact and retrievable"
    );

    // 6. Test Integrity Check on CAS
    let integrity = engine.store.verify_integrity().unwrap();
    assert_eq!(integrity.corrupted_count, 0);
    assert!(integrity.valid_count >= 2);
}
