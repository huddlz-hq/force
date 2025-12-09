use assert_cmd::assert::Assert;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

// Helper to run force command
fn force_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("force"))
}

// Helper functions - creates a git repo with initial commit for worktree support
fn create_temp_project() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to init git");

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to configure git name");

    // Create initial commit (required for worktrees)
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "Initial commit"])
        .current_dir(dir.path())
        .output()
        .expect("Failed to create initial commit");

    fs::create_dir(dir.path().join(".force")).expect("Failed to create .force dir");
    dir
}

fn create_script(project_dir: &Path, name: &str, content: &str) {
    let script_path = project_dir.join(".force").join(format!("{}.toml", name));
    fs::write(&script_path, content).expect("Failed to write script file");
}

fn minimal_script() -> String {
    r#"[meta]
category = "setup"

[up]
run = "echo 'up'"

[down]
run = "echo 'down'"
"#
    .to_string()
}

#[test]
fn test_ls_shows_no_sessions_initially() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script());

    Assert::new(
        force_cmd()
            .arg("ls")
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("No active sessions"));
}

#[test]
fn test_ls_shows_session_after_up() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script());

    // Run up first
    Assert::new(
        force_cmd()
            .args(["up", "ls-after-up-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Now ls should show the session
    Assert::new(
        force_cmd()
            .arg("ls")
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("ls-after-up-test"))
    .stdout(predicate::str::contains("port"));
}

#[test]
fn test_ls_removes_session_after_down() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script());

    // Run up
    Assert::new(
        force_cmd()
            .args(["up", "ls-remove-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Run down
    Assert::new(
        force_cmd()
            .args(["down", "ls-remove-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Now ls should show no sessions
    Assert::new(
        force_cmd()
            .arg("ls")
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("No active sessions"));
}

#[test]
fn test_ls_shows_multiple_sessions() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script());

    // Run up for two features
    Assert::new(
        force_cmd()
            .args(["up", "ls-multi-a"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();
    Assert::new(
        force_cmd()
            .args(["up", "ls-multi-b"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // ls should show both
    Assert::new(
        force_cmd()
            .arg("ls")
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("ls-multi-a"))
    .stdout(predicate::str::contains("ls-multi-b"));
}

#[test]
fn test_ls_fails_without_force_dir() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .arg("ls")
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains(".force/ directory not found"));
}
