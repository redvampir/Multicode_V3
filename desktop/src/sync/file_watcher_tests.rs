use super::{file_watcher::FileWatcher, SyncMessage};
use multicode_core::parser::Lang;
use std::fs;
use std::sync::mpsc::sync_channel;
use std::time::Duration;
use tempfile;

#[test]
fn watcher_sends_text_changed_on_modify() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("test.rs");
    fs::write(&file, "fn main() {}\n").unwrap();
    let (tx, rx) = sync_channel(1);
    let watcher = FileWatcher::new(tx);
    watcher.on_open(&file);
    fs::write(&file, "fn main() { println!(\"hi\"); }\n").unwrap();
    let msg = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    match msg.unwrap() {
        SyncMessage::TextChanged(code, lang) => {
            assert_eq!(lang, Lang::Rust);
            assert!(code.contains("println"));
        }
        other => panic!("unexpected message: {:?}", other),
    }
}

#[test]
fn watcher_ignores_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("foo.tmp");
    fs::write(&file, "a").unwrap();
    let (tx, rx) = sync_channel(1);
    let watcher = FileWatcher::new(tx);
    watcher.on_open(&file);
    fs::write(&file, "b").unwrap();
    assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
}

#[test]
fn watcher_handles_multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("a.rs");
    let file2 = dir.path().join("b.rs");
    fs::write(&file1, "fn a() {}\n").unwrap();
    fs::write(&file2, "fn b() {}\n").unwrap();
    let (tx, rx) = sync_channel(2);
    let watcher = FileWatcher::new(tx);
    watcher.on_open(&file1);
    watcher.on_open(&file2);
    fs::write(&file1, "fn a() {1}\n").unwrap();
    fs::write(&file2, "fn b() {2}\n").unwrap();
    let m1 = rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
    let m2 = rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
    assert!(matches!(m1, SyncMessage::TextChanged(_, Lang::Rust)));
    assert!(matches!(m2, SyncMessage::TextChanged(_, Lang::Rust)));
}
