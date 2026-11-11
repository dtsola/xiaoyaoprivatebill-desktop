//! P1-8 契约测试：导入、列表、删除、清空。

use std::fs;

use bill_core::DataPaths;
use bill_services::files::{
    clear_data, delete_file, import_paths, import_paths_with_limit, list_files, FileError,
    MAX_UPLOAD_SIZE,
};

const ALIPAY: &str = "交易时间,交易分类,交易对方,商品说明,收/支,金额,收/付款方式,交易状态\n\
2024-01-15 12:30:00,餐饮美食,星巴克,咖啡,支出,28.50,余额宝,交易成功\n";

fn setup() -> (tempfile::TempDir, DataPaths, tempfile::TempDir) {
    let data = tempfile::tempdir().expect("数据目录");
    let source = tempfile::tempdir().expect("来源目录");
    let paths = DataPaths::new(data.path().to_path_buf());
    paths.ensure_layout().expect("创建布局");
    (data, paths, source)
}

#[test]
fn imports_file_and_lists_it() {
    let (_data, paths, source) = setup();
    let input = source.path().join("alipay.csv");
    fs::write(&input, ALIPAY).expect("写来源文件");

    let names = import_paths(&paths, std::slice::from_ref(&input)).expect("导入成功");
    assert_eq!(names, vec!["alipay.csv"]);

    let files = list_files(&paths).expect("列出文件");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "alipay.csv");
    assert_eq!(files[0].source, "支付宝");
    assert_eq!(files[0].size, ALIPAY.len() as u64);
    assert!(paths.bills_dir().join("alipay.csv").is_file());
}

#[test]
fn chinese_name_degrades_to_extension_and_becomes_invisible() {
    // 复刻 D3：中文名清洗后只剩 "csv"，列表按扩展名过滤所以看不到。
    let (_data, paths, source) = setup();
    let input = source.path().join("账单.csv");
    fs::write(&input, ALIPAY).expect("写来源文件");

    let names = import_paths(&paths, std::slice::from_ref(&input)).expect("导入成功");
    assert_eq!(names, vec!["csv"]);
    assert!(paths.bills_dir().join("csv").is_file());
    assert!(list_files(&paths).expect("列出").is_empty());
}

#[test]
fn rejects_unsupported_extension() {
    let (_data, paths, source) = setup();
    let input = source.path().join("bill.txt");
    fs::write(&input, "x").expect("写来源文件");

    let error = import_paths(&paths, &[input]).expect_err("不支持的扩展名必须报错");
    assert!(matches!(error, FileError::Unsupported), "实际: {error:?}");
}

#[test]
fn rejects_oversized_file() {
    let (_data, paths, source) = setup();
    let input = source.path().join("big.csv");
    fs::write(&input, vec![b'a'; 64]).expect("写来源文件");

    let error = import_paths_with_limit(&paths, &[input], 32).expect_err("超过上限必须报错");
    assert!(matches!(error, FileError::TooLarge(_)), "实际: {error:?}");
    assert_eq!(MAX_UPLOAD_SIZE, 16 * 1024 * 1024);
}

#[test]
fn duplicate_import_gets_counter_suffix() {
    let (_data, paths, source) = setup();
    let input = source.path().join("alipay.csv");
    fs::write(&input, ALIPAY).expect("写来源文件");

    import_paths(&paths, std::slice::from_ref(&input)).expect("第一次导入");
    let names = import_paths(&paths, &[input]).expect("第二次导入");

    assert_eq!(names, vec!["alipay_1.csv"], "重复导入不去重（复刻 D4）");
    assert_eq!(list_files(&paths).expect("列出").len(), 2);
}

#[test]
fn delete_moves_file_to_trash_with_manifest() {
    let (_data, paths, source) = setup();
    let input = source.path().join("alipay.csv");
    fs::write(&input, ALIPAY).expect("写来源文件");
    import_paths(&paths, &[input]).expect("导入");

    delete_file(&paths, "alipay.csv").expect("删除成功");

    assert!(!paths.bills_dir().join("alipay.csv").exists());
    let trash_entries: Vec<_> = fs::read_dir(paths.trash_dir())
        .expect("读取回收站")
        .filter_map(|entry| entry.ok())
        .collect();
    assert_eq!(trash_entries.len(), 1);

    let entry = &trash_entries[0];
    assert!(
        entry.path().join("manifest.json").is_file(),
        "必须写 manifest"
    );
    assert!(entry.path().join("files").join("alipay.csv").is_file());
}

#[test]
fn delete_missing_file_reports_not_found() {
    let (_data, paths, _source) = setup();
    let error = delete_file(&paths, "missing.csv").expect_err("缺失文件必须报错");
    assert!(matches!(error, FileError::NotFound), "实际: {error:?}");
}

#[test]
fn delete_rejects_path_traversal() {
    let (_data, paths, _source) = setup();
    let error = delete_file(&paths, "../settings.json").expect_err("必须拒绝路径穿越");
    assert!(matches!(error, FileError::InvalidName), "实际: {error:?}");
}

#[test]
fn clear_moves_all_files_to_trash() {
    let (_data, paths, source) = setup();
    let a = source.path().join("a.csv");
    let b = source.path().join("b.csv");
    fs::write(&a, ALIPAY).expect("写 a");
    fs::write(&b, ALIPAY).expect("写 b");
    import_paths(&paths, &[a, b]).expect("导入");

    let moved = clear_data(&paths).expect("清空成功");

    assert_eq!(moved, 2);
    assert_eq!(list_files(&paths).expect("列出").len(), 0);
    let trash_entries: Vec<_> = fs::read_dir(paths.trash_dir())
        .expect("读取回收站")
        .filter_map(|entry| entry.ok())
        .collect();
    assert_eq!(trash_entries.len(), 1);
    assert_eq!(
        fs::read_dir(trash_entries[0].path().join("files"))
            .expect("读取回收文件")
            .count(),
        2
    );
}
