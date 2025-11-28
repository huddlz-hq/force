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

#[test]
fn test_finds_force_dir_in_current() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    Assert::new(force_cmd().args(["up", "feature"]).current_dir(project.path()).output().unwrap())
        .success()
        .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_finds_force_dir_in_parent() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    let subdir = project.path().join("src");
    fs::create_dir(&subdir).unwrap();

    Assert::new(force_cmd().args(["up", "feature"]).current_dir(&subdir).output().unwrap())
        .success()
        .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_finds_force_dir_in_grandparent() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    let subdir = project.path().join("src/components");
    fs::create_dir_all(&subdir).unwrap();

    Assert::new(force_cmd().args(["up", "feature"]).current_dir(&subdir).output().unwrap())
        .success()
        .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_error_when_no_force_dir() {
    let dir = TempDir::new().unwrap();

    Assert::new(force_cmd().args(["up", "feature"]).current_dir(dir.path()).output().unwrap())
        .failure()
        .stderr(predicate::str::contains(".force/ directory not found"));
}

#[test]
fn test_uses_closest_force_dir() {
    let outer = create_temp_project();
    let outer_output = outer.path().join("outer.txt");
    create_script(
        outer.path(),
        "outer",
        &format!(
            r#"[meta]
category = "setup"

[up]
run = "echo outer >> {}"
"#,
            outer_output.display()
        ),
    );

    let inner_dir = outer.path().join("inner");
    fs::create_dir(&inner_dir).unwrap();
    fs::create_dir(inner_dir.join(".force")).unwrap();
    let inner_output = inner_dir.join("inner.txt");
    fs::write(
        inner_dir.join(".force/inner.toml"),
        format!(
            r#"[meta]
category = "setup"

[up]
run = "echo inner >> {}"
"#,
            inner_output.display()
        ),
    )
    .unwrap();

    Assert::new(force_cmd().args(["up", "feature"]).current_dir(&inner_dir).output().unwrap())
        .success();

    assert!(inner_output.exists(), "Inner script should have run");
    assert!(!outer_output.exists(), "Outer script should NOT have run");
}
