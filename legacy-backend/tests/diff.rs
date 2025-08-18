use backend::git::{commit, diff};
use git2::{Repository, ErrorCode};
use tempfile::tempdir;
use std::fs;
use std::env;

#[test]
fn diff_shows_changes() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    {
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Tester").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
    }
    fs::create_dir(dir.path().join("backend")).unwrap();
    let file_path = dir.path().join("backend/file.txt");
    fs::write(&file_path, "line1\n").unwrap();

    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();
    commit("initial").unwrap();
    fs::write(&file_path, "line1\nline2\n").unwrap();
    let out = diff().unwrap();
    env::set_current_dir(prev).unwrap();

    assert!(out.contains("line2"));
    assert!(out.contains("backend/file.txt"));
}

#[test]
fn diff_errors_outside_repo() {
    let dir = tempdir().unwrap();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();
    let err = diff().unwrap_err();
    env::set_current_dir(prev).unwrap();
    assert_eq!(err.code(), ErrorCode::NotFound);
}

