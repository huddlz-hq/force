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

fn minimal_script(category: &str) -> String {
    format!(
        r#"[meta]
category = "{}"

[up]
run = "echo 'running'"
"#,
        category
    )
}

fn failing_script(category: &str) -> String {
    format!(
        r#"[meta]
category = "{}"

[up]
run = "exit 1"
"#,
        category
    )
}

fn env_capture_script(category: &str, output_file: &Path) -> String {
    // Use absolute path since scripts run in worktree directory
    let abs_path = output_file
        .canonicalize()
        .unwrap_or_else(|_| output_file.to_path_buf());
    format!(
        r#"[meta]
category = "{}"

[up]
description = "Capture env vars"
run = "echo \"FORCE_FEATURE=$FORCE_FEATURE\" >> {} && echo \"FORCE_FEATURE_SLUG=$FORCE_FEATURE_SLUG\" >> {} && echo \"FORCE_PORT=$FORCE_PORT\" >> {} && echo \"FORCE_PORT_OFFSET=$FORCE_PORT_OFFSET\" >> {} && echo \"FORCE_DB_NAME=$FORCE_DB_NAME\" >> {} && echo \"FORCE_DIR=$FORCE_DIR\" >> {} && echo \"FORCE_WORKTREE=$FORCE_WORKTREE\" >> {}"
"#,
        category,
        abs_path.display(),
        abs_path.display(),
        abs_path.display(),
        abs_path.display(),
        abs_path.display(),
        abs_path.display(),
        abs_path.display()
    )
}

#[test]
fn test_up_with_single_script() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &minimal_script("setup"));

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Session 'test-feature' is ready!"));
}

#[test]
fn test_up_with_alias() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &minimal_script("setup"));

    Assert::new(
        force_cmd()
            .args(["u", "my-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Session 'my-feature' is ready!"));
}

#[test]
fn test_up_fails_without_force_dir() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains(".force/ directory not found"));
}

#[test]
fn test_up_fails_on_invalid_toml() {
    let project = create_temp_project();
    let script_path = project.path().join(".force/invalid.toml");
    fs::write(&script_path, "this is not valid toml [[[").unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains("Failed to parse"));
}

#[test]
fn test_up_fails_on_missing_category() {
    let project = create_temp_project();
    let script_path = project.path().join(".force/nocategory.toml");
    fs::write(
        &script_path,
        r#"
[meta]

[up]
run = "echo hello"
"#,
    )
    .unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains("Failed to parse"));
}

#[test]
fn test_up_fails_on_script_error() {
    let project = create_temp_project();
    create_script(project.path(), "failing", &failing_script("setup"));

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains("failed"));
}

#[test]
fn test_up_shows_script_description() {
    let project = create_temp_project();
    let script = r#"[meta]
category = "setup"

[up]
description = "My custom description"
run = "echo hello"
"#;
    create_script(project.path(), "described", script);

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("My custom description"));
}

#[test]
fn test_up_sets_all_env_vars() {
    let project = create_temp_project();
    let output_file = project.path().join("env_output.txt");

    // Create the file so canonicalize works
    fs::write(&output_file, "").unwrap();

    create_script(
        project.path(),
        "capture",
        &env_capture_script("setup", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "up-env-vars-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();

    assert!(output.contains("FORCE_FEATURE=up-env-vars-test"));
    assert!(output.contains("FORCE_FEATURE_SLUG=up_env_vars_test"));
    assert!(output.contains("FORCE_PORT="));
    assert!(output.contains("FORCE_PORT_OFFSET="));
    assert!(output.contains("FORCE_DB_NAME="));
    assert!(output.contains("FORCE_DIR="));
    assert!(output.contains("FORCE_WORKTREE="));
}

#[test]
fn test_up_creates_worktree() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &minimal_script("setup"));

    Assert::new(
        force_cmd()
            .args(["up", "worktree-create-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Verify worktree directory was created
    let worktree_path = project
        .path()
        .parent()
        .unwrap()
        .join("worktrees/worktree_create_test");
    assert!(worktree_path.exists(), "Worktree directory should exist");
    assert!(
        worktree_path.join(".git").exists(),
        "Worktree should have .git"
    );
}

#[test]
fn test_up_reuses_existing_worktree() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &minimal_script("setup"));

    // Run up twice - should succeed both times
    Assert::new(
        force_cmd()
            .args(["up", "worktree-reuse-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    Assert::new(
        force_cmd()
            .args(["up", "worktree-reuse-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Reusing worktree at:"));
}

#[test]
fn test_up_creates_worktree_with_custom_path() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &minimal_script("setup"));

    // Create custom config
    let config = r#"[worktree]
path = ".worktrees/$FORCE_FEATURE_SLUG"
"#;
    fs::write(project.path().join(".force/config.toml"), config).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "custom-path-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Verify worktree was created at custom path
    let worktree_path = project.path().join(".worktrees/custom_path_test");
    assert!(
        worktree_path.exists(),
        "Worktree should be at custom path: {:?}",
        worktree_path
    );
}
