//! P4-7 契约测试：清空前快照。

use bill_core::DataPaths;
use bill_services::snapshot::create_snapshot;

fn setup() -> (tempfile::TempDir, DataPaths) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    (dir, paths)
}

#[test]
fn snapshot_copies_every_bill_file() {
    let (_dir, paths) = setup();
    std::fs::write(paths.bills_dir().join("a.csv"), b"alpha").expect("写 a");
    std::fs::write(paths.bills_dir().join("b.xlsx"), b"beta").expect("写 b");
    std::fs::write(paths.bills_dir().join("note.txt"), b"skip").expect("写 note");

    let (id, count) = create_snapshot(&paths).expect("快照");

    assert!(!id.is_empty());
    assert_eq!(count, 2, "只快照 csv/xlsx");
    let dir = paths.snapshots_dir().join(&id);
    assert_eq!(
        std::fs::read(dir.join("files").join("a.csv")).expect("a"),
        b"alpha"
    );
    assert_eq!(
        std::fs::read(dir.join("files").join("b.xlsx")).expect("b"),
        b"beta"
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).expect("manifest"))
            .expect("JSON");
    assert_eq!(manifest["files"].as_array().expect("文件").len(), 2);
}

#[test]
fn snapshot_of_empty_bills_still_records_entry() {
    let (_dir, paths) = setup();
    let (id, count) = create_snapshot(&paths).expect("快照");

    assert_eq!(count, 0);
    assert!(paths
        .snapshots_dir()
        .join(&id)
        .join("manifest.json")
        .exists());
}
