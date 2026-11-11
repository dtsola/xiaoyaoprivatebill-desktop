//! P2-9 收尾：`session.json` 的读写语义。

use bill_core::session::Session;
use bill_core::DataPaths;

fn setup() -> (tempfile::TempDir, DataPaths) {
    let dir = tempfile::tempdir().expect("临时目录");
    let paths = DataPaths::new(dir.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    (dir, paths)
}

#[test]
fn missing_session_file_is_inactive() {
    let (_dir, paths) = setup();
    let session = Session::load(&paths);

    assert!(!session.is_active());
    assert!(!session.is_demo);
    assert_eq!(session.user_id, None);
    assert_eq!(session.schema_version, 1);
}

#[test]
fn enter_demo_persists_demo_user() {
    let (_dir, paths) = setup();
    Session::enter_demo(&paths).expect("进入演示");

    let session = Session::load(&paths);
    assert!(session.is_active());
    assert!(session.is_demo);
    assert_eq!(session.user_id.as_deref(), Some("demo_user"));
}

#[test]
fn exit_demo_clears_demo_identity() {
    let (_dir, paths) = setup();
    Session::enter_demo(&paths).expect("进入演示");
    Session::exit_demo(&paths).expect("退出演示");

    let session = Session::load(&paths);
    assert!(!session.is_active());
    assert!(!session.is_demo);
    assert_eq!(session.user_id, None);
}

#[test]
fn mark_import_sets_local_user_only_once() {
    let (_dir, paths) = setup();
    let first = Session::mark_import(&paths).expect("首次导入");
    assert_eq!(first.user_id.as_deref(), Some("user_local"));
    let started = first.session_start.clone().expect("session_start");

    let second = Session::mark_import(&paths).expect("再次导入");
    assert_eq!(second.user_id.as_deref(), Some("user_local"));
    assert_eq!(
        second.session_start,
        Some(started),
        "session_start 不应被覆盖"
    );
}

#[test]
fn mark_import_keeps_demo_identity() {
    let (_dir, paths) = setup();
    Session::enter_demo(&paths).expect("进入演示");

    let session = Session::mark_import(&paths).expect("演示中导入");
    assert_eq!(session.user_id.as_deref(), Some("demo_user"));
    assert!(session.is_demo);
}
