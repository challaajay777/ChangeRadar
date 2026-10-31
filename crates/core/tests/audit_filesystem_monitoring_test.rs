use changeradar_core::ChangeRadarEngine;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_filesystem_monitoring_16_scenarios() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    let monitored_dir = tmp.path().join("monitored");
    fs::create_dir_all(&monitored_dir).unwrap();

    let engine = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let folder = engine
        .add_folder(&monitored_dir.to_string_lossy(), Some("Monitored"))
        .unwrap();

    // Scenario 1: Create a file
    let file1 = monitored_dir.join("file1.txt");
    fs::write(&file1, b"Initial content of file 1").unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(
        res.files_created, 1,
        "Scenario 1: File creation not detected"
    );

    let tracked = engine.db.list_files(Some(&folder.id), 100).unwrap();
    assert_eq!(tracked.len(), 1);
    assert_eq!(tracked[0].file_name, "file1.txt");

    // Scenario 2: Modify a file
    fs::write(&file1, b"Modified content of file 1").unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(
        res.files_modified, 1,
        "Scenario 2: Modification not detected"
    );

    // Scenario 3: Modify a file repeatedly
    for i in 1..=5 {
        fs::write(
            &file1,
            format!("Rapid modification iteration {}", i).as_bytes(),
        )
        .unwrap();
        let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
        assert_eq!(
            res.files_modified, 1,
            "Scenario 3: Repeated modification not detected at step {}",
            i
        );
    }
    let versions = engine.db.list_versions_for_file(&tracked[0].id).unwrap();
    // 1 initial + 1 modify + 5 rapid modifies = 7 versions
    assert_eq!(
        versions.len(),
        7,
        "Scenario 3: Expected 7 historical versions"
    );

    // Scenario 4: Delete a file
    fs::remove_file(&file1).unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(res.files_deleted, 1, "Scenario 4: Deletion not detected");
    let file1_rec = engine.db.get_file(&tracked[0].id).unwrap().unwrap();
    assert!(
        file1_rec.is_deleted,
        "Scenario 4: File record should be marked deleted"
    );

    // Scenario 5: Rename a file (re-create as source, then rename)
    let src_file = monitored_dir.join("old_name.txt");
    let dst_file = monitored_dir.join("new_name.txt");
    fs::write(&src_file, b"Content for rename test").unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    fs::rename(&src_file, &dst_file).unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    // In reconciliation, rename is detected as 1 deleted + 1 created
    assert_eq!(res.files_deleted, 1);
    assert_eq!(res.files_created, 1);

    // Scenario 6: Move a file between monitored directories
    let monitored_dir2 = tmp.path().join("monitored_2");
    fs::create_dir_all(&monitored_dir2).unwrap();
    let folder2 = engine
        .add_folder(&monitored_dir2.to_string_lossy(), Some("Folder 2"))
        .unwrap();

    let moved_file_src = monitored_dir.join("to_move.txt");
    let moved_file_dst = monitored_dir2.join("moved_here.txt");
    fs::write(&moved_file_src, b"Cross-folder movement content").unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    fs::rename(&moved_file_src, &moved_file_dst).unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();
    let res2 = engine
        .event_processor
        .reconcile_folder(&folder2.id)
        .unwrap();
    assert_eq!(
        res2.files_created, 1,
        "Scenario 6: Moved file not detected in target folder"
    );

    // Scenario 7 & 8: Create nested directories and files inside them
    let nested_dir = monitored_dir.join("sub1").join("sub2");
    fs::create_dir_all(&nested_dir).unwrap();
    let nested_file = nested_dir.join("deep_doc.md");
    fs::write(&nested_file, b"# Deep Document in Nested Directory").unwrap();

    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(
        res.files_created, 1,
        "Scenario 8: Nested file creation not detected"
    );

    let nested_tracked = engine.db.list_files(Some(&folder.id), 100).unwrap();
    let deep_rec = nested_tracked
        .iter()
        .find(|f| f.file_name == "deep_doc.md")
        .expect("Scenario 8: deep_doc.md not tracked");
    assert!(
        deep_rec.relative_path.contains("sub1/sub2")
            || deep_rec.relative_path.contains("sub1\\sub2")
    );

    // Scenario 9: Delete an entire directory
    fs::remove_dir_all(monitored_dir.join("sub1")).unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(
        res.files_deleted, 1,
        "Scenario 9: Deletion of entire directory not reconciled"
    );
    let deep_rec_after = engine.db.get_file(&deep_rec.id).unwrap().unwrap();
    assert!(
        deep_rec_after.is_deleted,
        "Scenario 9: Nested file should be marked deleted"
    );

    // Scenario 10: Temporary-file-and-rename saving pattern (editor atomic write)
    let atomic_target = monitored_dir.join("document.txt");
    fs::write(&atomic_target, b"Original document text").unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    // Editor writes to temp file then renames over atomic_target
    let temp_save = monitored_dir.join("document.txt.tmp");
    fs::write(&temp_save, b"Updated document text via atomic swap").unwrap();
    #[cfg(windows)]
    fs::remove_file(&atomic_target).unwrap();
    fs::rename(&temp_save, &atomic_target).unwrap();

    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert!(
        res.files_modified >= 1 || res.files_created >= 1,
        "Scenario 10: Atomic save pattern not tracked"
    );
    let content = fs::read_to_string(&atomic_target).unwrap();
    assert_eq!(content, "Updated document text via atomic swap");

    // Scenario 11: Modify a file while engine is running
    fs::write(&atomic_target, b"Updated text while running").unwrap();
    let res = engine.event_processor.reconcile_folder(&folder.id).unwrap();
    assert_eq!(
        res.files_modified, 1,
        "Scenario 11: Running modification not tracked"
    );

    // Scenario 12: Restart the daemon / engine
    drop(engine);
    let engine_restarted = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let folders = engine_restarted.list_folders().unwrap();
    assert_eq!(
        folders.len(),
        2,
        "Scenario 12: Monitored folders should persist after restart"
    );

    // Scenario 13: Large directory tree with 50 files
    let bulk_dir = monitored_dir.join("bulk_test");
    fs::create_dir_all(&bulk_dir).unwrap();
    for i in 0..50 {
        fs::write(
            bulk_dir.join(format!("item_{:03}.txt", i)),
            format!("Payload {}", i).as_bytes(),
        )
        .unwrap();
    }
    let res = engine_restarted
        .event_processor
        .reconcile_folder(&folder.id)
        .unwrap();
    assert_eq!(
        res.files_created, 50,
        "Scenario 13: Bulk files not detected"
    );

    // Scenario 14: Ignored directories
    let git_dir = monitored_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(
        git_dir.join("COMMIT_EDITMSG"),
        b"ignored git commit message",
    )
    .unwrap();

    let res = engine_restarted
        .event_processor
        .reconcile_folder(&folder.id)
        .unwrap();
    assert_eq!(
        res.files_created, 0,
        "Scenario 14: Ignored .git directory should not create tracked files"
    );

    // Scenario 15: Unicode filenames and unusual characters
    let unicode_file = monitored_dir.join("файл_тест_中文_🚀.txt");
    fs::write(
        &unicode_file,
        "UTF-8 Content: 🌟 ChangeRadar 2026 🚀".as_bytes(),
    )
    .unwrap();
    let res = engine_restarted
        .event_processor
        .reconcile_folder(&folder.id)
        .unwrap();
    assert_eq!(
        res.files_created, 1,
        "Scenario 15: Unicode file creation not detected"
    );

    let unicode_files = engine_restarted
        .db
        .list_files(Some(&folder.id), 200)
        .unwrap();
    let found = unicode_files
        .iter()
        .any(|f| f.file_name.contains("файл_тест") || f.file_name.contains("🚀"));
    assert!(
        found,
        "Scenario 15: Unicode file record not found in database"
    );

    // Scenario 16: Reconciliation detects changes made while offline
    drop(engine_restarted);
    // Make changes while engine is completely offline
    let offline_file = monitored_dir.join("offline_created.txt");
    fs::write(&offline_file, b"Created while ChangeRadar was stopped").unwrap();

    // Restart again
    let engine_resumed = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let res = engine_resumed
        .event_processor
        .reconcile_folder(&folder.id)
        .unwrap();
    assert_eq!(
        res.files_created, 1,
        "Scenario 16: Offline created file detected on restart reconciliation"
    );
}
