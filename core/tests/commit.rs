#![cfg(feature = "git")]
use core::git::commit;
use git2::Repository;
use tempfile::tempdir;
use std::fs;
use std::env;

#[test]
fn commit_creates_commit() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    {
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Tester").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
    }
    fs::create_dir(dir.path().join("backend")).unwrap();
    fs::write(dir.path().join("backend/file.txt"), "hello").unwrap();

    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();
    commit("initial commit").unwrap();
    env::set_current_dir(prev).unwrap();

    // A new repository should have a `main` branch pointing at the initial
    // commit and `HEAD` should reference it.
    let head_ref = repo.head().unwrap();
    assert_eq!(head_ref.shorthand(), Some("main"));
    let commit = repo
        .find_commit(repo.refname_to_id("HEAD").unwrap())
        .unwrap();
    assert_eq!(commit.summary(), Some("initial commit"));
}

#[test]
fn commit_empty_message_error() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    {
        let mut cfg = repo.config().unwrap();
        cfg.set_str("user.name", "Tester").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
    }

    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();
    let err = commit("").unwrap_err();
    env::set_current_dir(prev).unwrap();
    assert_eq!(err.message(), "commit message cannot be empty");
}

