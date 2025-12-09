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
fn test_init_creates_config_and_example_scripts() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .success();

    // Check config.toml was created
    let config_path = dir.path().join(".force/config.toml");
    assert!(config_path.exists());
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("[worktree]"));
    assert!(config_content.contains("path"));

    // Check env.toml was created
    let env_path = dir.path().join(".force/env.toml");
    assert!(env_path.exists());
    let env_content = fs::read_to_string(&env_path).unwrap();
    assert!(env_content.contains(".dev.local.env"));
    assert!(env_content.contains(".test.local.env"));
    assert!(env_content.contains("DATABASE_URL"));
    assert!(env_content.contains("FORCE_PORT"));

    // Check database.toml was created
    let database_path = dir.path().join(".force/database.toml");
    assert!(database_path.exists());
    let db_content = fs::read_to_string(&database_path).unwrap();
    assert!(db_content.contains("[meta]"));
    assert!(db_content.contains("[up]"));
    assert!(db_content.contains("[down]"));
    assert!(db_content.contains("createdb"));
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
