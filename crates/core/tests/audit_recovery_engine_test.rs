use changeradar_core::ChangeRadarEngine;
use changeradar_hashing::hash_file;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_recovery_engine_comprehensive_audit() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    let monitored_dir = tmp.path().join("monitored");
    fs::create_dir_all(&monitored_dir).unwrap();

    let engine = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let folder = engine
        .add_folder(&monitored_dir.to_string_lossy(), Some("Monitored Folder"))
        .unwrap();

    // 1. Create file with Version 1 content
    let file_path = monitored_dir.join("critical_spec.txt");
    fs::write(&file_path, b"Version 1: Production System Specification").unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let tracked = engine.db.list_files(Some(&folder.id), 10).unwrap();
    let file_rec = tracked
        .iter()
        .find(|f| f.file_name == "critical_spec.txt")
        .unwrap();
    let v1 = engine.db.get_latest_version(&file_rec.id).unwrap().unwrap();
    let v1_hash = v1.content_hash.clone();

    // 2. Overwrite file with Version 2 content
    fs::write(
        &file_path,
        b"Version 2: Unreviewed Experimental Specification",
    )
    .unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let v2 = engine.db.get_latest_version(&file_rec.id).unwrap().unwrap();
    assert_ne!(v1_hash, v2.content_hash);

    // 3. Test Recovery Pre-flight check over existing destination
    let preflight = engine.recovery.preflight_check(&v1.id, None).unwrap();
    assert!(
        preflight.can_proceed,
        "Preflight: should be able to proceed"
    );
    assert!(
        preflight.cas_object_verified,
        "Preflight: CAS object should be verified"
    );
    assert!(
        preflight.destination_exists,
        "Preflight: should detect existing file"
    );
    assert!(
        preflight.warning.is_some(),
        "Preflight: should issue overwrite warning"
    );

    // 4. Test Restore over existing file (Automatic safety backup)
    let res = engine.recovery.restore_version(&v1.id, None).unwrap();
    assert!(res.success, "Restore: operation should succeed");
    assert_eq!(
        res.restored_hash, v1_hash,
        "Restore: restored hash must match v1"
    );
    assert!(
        res.backup_version_id.is_some(),
        "Restore: existing destination should be backed up before overwrite"
    );

    // Verify disk content is restored to v1
    let disk_text = fs::read_to_string(&file_path).unwrap();
    assert_eq!(disk_text, "Version 1: Production System Specification");

    // Verify post-restore hash on disk matches BLAKE3 exactly
    let on_disk_hash = hash_file(&file_path).unwrap();
    assert_eq!(on_disk_hash, v1_hash);

    // 5. Test Restore to custom new destination path
    let custom_dest = tmp.path().join("exports").join("restored_v1.txt");
    let preflight_custom = engine
        .recovery
        .preflight_check(&v1.id, Some(&custom_dest))
        .unwrap();
    assert!(!preflight_custom.destination_exists);

    let res_custom = engine
        .recovery
        .restore_version(&v1.id, Some(&custom_dest))
        .unwrap();
    assert!(res_custom.success);
    assert!(custom_dest.exists());
    assert_eq!(
        fs::read_to_string(&custom_dest).unwrap(),
        "Version 1: Production System Specification"
    );

    // 6. Test Restore deleted file
    fs::remove_file(&file_path).unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert!(!file_path.exists());

    let res_deleted = engine.recovery.restore_version(&v1.id, None).unwrap();
    assert!(res_deleted.success);
    assert!(file_path.exists());
    assert_eq!(
        fs::read_to_string(&file_path).unwrap(),
        "Version 1: Production System Specification"
    );

    // 7. Test Unicode filename recovery
    let unicode_file = monitored_dir.join("документ_2026_中文.txt");
    fs::write(
        &unicode_file,
        "Конфигурация системы: ChangeRadar 2026 🚀".as_bytes(),
    )
    .unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let tracked_u = engine.db.list_files(Some(&folder.id), 100).unwrap();
    let u_rec = tracked_u
        .iter()
        .find(|f| f.file_name.contains("документ"))
        .unwrap();
    let u_ver = engine.db.get_latest_version(&u_rec.id).unwrap().unwrap();

    // Delete and restore unicode file
    fs::remove_file(&unicode_file).unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    let res_u = engine.recovery.restore_version(&u_ver.id, None).unwrap();
    assert!(res_u.success);
    assert!(unicode_file.exists());
    assert_eq!(
        fs::read_to_string(&unicode_file).unwrap(),
        "Конфигурация системы: ChangeRadar 2026 🚀"
    );

    // 8. Test Corrupted CAS object rejection (simulated disk corruption)
    // Find the CAS object file on disk and overwrite it with junk bytes
    let cas_sub1 = &v1_hash[0..2];
    let cas_sub2 = &v1_hash[2..4];
    let cas_obj_path = data_dir
        .join("objects")
        .join(cas_sub1)
        .join(cas_sub2)
        .join(format!("{}.zst", v1_hash));
    assert!(
        cas_obj_path.exists(),
        "CAS object file should exist at {:?}",
        cas_obj_path
    );

    // Corrupt the CAS object
    fs::write(&cas_obj_path, b"CORRUPTED_CAS_GARBAGE_PAYLOAD_1234567890").unwrap();

    // Preflight must detect corruption
    let preflight_corrupt = engine.recovery.preflight_check(&v1.id, None).unwrap();
    assert!(
        !preflight_corrupt.can_proceed,
        "Corrupt CAS preflight must not allow proceed"
    );
    assert!(
        !preflight_corrupt.cas_object_verified,
        "Corrupt CAS object must fail verification"
    );

    // Attempted restore must fail cleanly with an error, not panic or write garbage
    let restore_corrupt = engine.recovery.restore_version(&v1.id, None);
    assert!(
        restore_corrupt.is_err(),
        "Recovery must reject corrupted CAS object"
    );

    // 9. Verify Recovery Audit Trail
    let audit_events = engine.db.list_events(Some(&folder.id), 100, 0).unwrap();
    let restore_events: Vec<_> = audit_events
        .into_iter()
        .filter(|e| e.event_type == changeradar_types::ChangeKind::Restored)
        .collect();
    assert!(
        !restore_events.is_empty(),
        "Recovery events must be recorded in timeline audit"
    );
}
