use changeradar_core::ChangeRadarEngine;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_snapshot_engine_audit() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    let monitored_dir = tmp.path().join("monitored");
    fs::create_dir_all(&monitored_dir).unwrap();

    let engine = ChangeRadarEngine::initialize(Some(&data_dir)).unwrap();
    let folder = engine
        .add_folder(&monitored_dir.to_string_lossy(), Some("Snapshot Folder"))
        .unwrap();

    // 1. Create directory structure with nested files
    let sub = monitored_dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(monitored_dir.join("file_a.txt"), b"File A content").unwrap();
    fs::write(sub.join("file_b.txt"), b"File B content").unwrap();
    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    // 2. Create Snapshot 1 (Baseline)
    let snap1 = engine
        .snapshots
        .create_snapshot(Some(&folder.id), "Baseline Snapshot", Some("Initial setup"))
        .unwrap();
    assert_eq!(snap1.file_count, 2);

    // 3. File modifications after snapshot
    fs::write(
        monitored_dir.join("file_a.txt"),
        b"File A MODIFIED after snap1",
    )
    .unwrap();

    // 4. File additions after snapshot
    fs::write(
        monitored_dir.join("file_c.txt"),
        b"File C ADDED after snap1",
    )
    .unwrap();

    // 5. File deletions after snapshot
    fs::remove_file(sub.join("file_b.txt")).unwrap();

    engine.event_processor.reconcile_folder(&folder.id).unwrap();

    // 6. Create Snapshot 2
    let snap2 = engine
        .snapshots
        .create_snapshot(Some(&folder.id), "Post-Changes Snapshot", None)
        .unwrap();
    assert_eq!(snap2.file_count, 2); // file_a (mod) and file_c (add), file_b deleted

    // 7. Test Snapshot Comparison
    let comparison = engine
        .snapshots
        .compare_snapshots(&snap1.id, &snap2.id)
        .unwrap();
    assert_eq!(
        comparison.added_files.len(),
        1,
        "Snapshot comparison: Expected 1 added file (file_c)"
    );
    assert_eq!(
        comparison.removed_files.len(),
        1,
        "Snapshot comparison: Expected 1 removed file (file_b)"
    );
    assert_eq!(
        comparison.modified_files.len(),
        1,
        "Snapshot comparison: Expected 1 modified file (file_a)"
    );

    // 8. Test Rollback to Snapshot 1
    let restore_results = engine.snapshots.restore_snapshot(&snap1.id).unwrap();
    assert_eq!(
        restore_results.len(),
        2,
        "Snapshot restore: Expected 2 files restored"
    );

    // Verify file_a is restored to initial content
    assert_eq!(
        fs::read_to_string(monitored_dir.join("file_a.txt")).unwrap(),
        "File A content"
    );
    // Verify file_b is restored to disk
    assert_eq!(
        fs::read_to_string(sub.join("file_b.txt")).unwrap(),
        "File B content"
    );
}
