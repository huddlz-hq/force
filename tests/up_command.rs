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

// Helper functions
fn create_temp_project() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
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
    format!(
        r#"[meta]
category = "{}"

[up]
description = "Capture env vars"
run = "echo \"FORCE_FEATURE=$FORCE_FEATURE\" >> {} && echo \"FORCE_FEATURE_SLUG=$FORCE_FEATURE_SLUG\" >> {} && echo \"FORCE_PORT=$FORCE_PORT\" >> {} && echo \"FORCE_PORT_OFFSET=$FORCE_PORT_OFFSET\" >> {} && echo \"FORCE_DB_NAME=$FORCE_DB_NAME\" >> {} && echo \"FORCE_DIR=$FORCE_DIR\" >> {}"
"#,
        category,
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display(),
        output_file.display()
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

    create_script(
        project.path(),
        "capture",
        &env_capture_script("setup", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "my-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();

    assert!(output.contains("FORCE_FEATURE=my-feature"));
    assert!(output.contains("FORCE_FEATURE_SLUG=my_feature"));
    assert!(output.contains("FORCE_PORT="));
    assert!(output.contains("FORCE_PORT_OFFSET="));
    assert!(output.contains("FORCE_DB_NAME="));
    assert!(output.contains("FORCE_DIR="));
}
