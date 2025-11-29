use assert_cmd::assert::Assert;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

// Helper to run force command
fn force_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("force"))
}

#[test]
fn test_init_creates_force_directory() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Created .force/ directory"));

    assert!(dir.path().join(".force").exists());
    assert!(dir.path().join(".force").is_dir());
}

#[test]
fn test_init_creates_example_script() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .success();

    let worktree_path = dir.path().join(".force/worktree.toml");
    assert!(worktree_path.exists());

    let content = fs::read_to_string(&worktree_path).unwrap();
    assert!(content.contains("[meta]"));
    assert!(content.contains("[up]"));
    assert!(content.contains("[down]"));
    assert!(content.contains("FORCE_FEATURE"));
}

#[test]
fn test_init_fails_if_force_dir_exists() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".force")).unwrap();

    Assert::new(
        force_cmd()
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_shows_next_steps() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("force up <feature-name>"));
}
