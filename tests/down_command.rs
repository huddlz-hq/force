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

fn script_with_down(category: &str) -> String {
    format!(
        r#"[meta]
category = "{}"

[up]
run = "echo 'up'"

[down]
run = "echo 'down'"
"#,
        category
    )
}

fn script_without_down(category: &str) -> String {
    format!(
        r#"[meta]
category = "{}"

[up]
run = "echo 'up only'"
"#,
        category
    )
}

fn order_tracking_down_script(
    category: &str,
    priority: Option<i32>,
    name: &str,
    output_file: &Path,
) -> String {
    let priority_line = match priority {
        Some(p) => format!("priority = {}", p),
        None => String::new(),
    };

    format!(
        r#"[meta]
category = "{}"
{}

[up]
run = "echo 'up'"

[down]
description = "Tear down: {}"
run = "echo '{}' >> {}"
"#,
        category,
        priority_line,
        name,
        name,
        output_file.display()
    )
}

#[test]
fn test_down_with_single_script() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    Assert::new(
        force_cmd()
            .args(["down", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains(
        "Session 'test-feature' torn down.",
    ));
}

#[test]
fn test_down_with_alias() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    Assert::new(
        force_cmd()
            .args(["d", "my-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Session 'my-feature' torn down."));
}

#[test]
fn test_down_skips_scripts_without_down_section() {
    let project = create_temp_project();
    create_script(project.path(), "with_down", &script_with_down("setup"));
    create_script(
        project.path(),
        "without_down",
        &script_without_down("setup"),
    );

    Assert::new(
        force_cmd()
            .args(["down", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("(no down script, skipping)"));
}

#[test]
fn test_down_runs_scripts_in_reverse_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    // Create scripts - they should run in reverse of up order
    // Up order: services -> setup (alphabetical by category)
    // Down order: setup -> services (reverse)
    create_script(
        project.path(),
        "zebra",
        &order_tracking_down_script("services", None, "zebra", &output_file),
    );
    create_script(
        project.path(),
        "alpha",
        &order_tracking_down_script("setup", None, "alpha", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["down", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // Up order would be: zebra (services), alpha (setup)
    // Down order should be reversed: alpha, zebra
    assert_eq!(lines, vec!["alpha", "zebra"]);
}

#[test]
fn test_down_reverses_priority_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    create_script(
        project.path(),
        "first",
        &order_tracking_down_script("setup", Some(1), "first", &output_file),
    );
    create_script(
        project.path(),
        "second",
        &order_tracking_down_script("setup", Some(2), "second", &output_file),
    );
    create_script(
        project.path(),
        "third",
        &order_tracking_down_script("setup", Some(3), "third", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["down", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // Up order: first, second, third (by priority)
    // Down order should be reversed: third, second, first
    assert_eq!(lines, vec!["third", "second", "first"]);
}

#[test]
fn test_down_fails_on_script_error() {
    let project = create_temp_project();
    let script = r#"[meta]
category = "setup"

[up]
run = "echo 'up'"

[down]
run = "exit 1"
"#;
    create_script(project.path(), "failing", script);

    Assert::new(
        force_cmd()
            .args(["down", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains("failed"));
}

#[test]
fn test_down_sets_env_vars() {
    let project = create_temp_project();
    let output_file = project.path().join("env_output.txt");

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo 'up'"

[down]
run = "echo \"FORCE_FEATURE=$FORCE_FEATURE\" >> {}"
"#,
        output_file.display()
    );
    create_script(project.path(), "env_check", &script);

    Assert::new(
        force_cmd()
            .args(["down", "my-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    assert!(output.contains("FORCE_FEATURE=my-feature"));
}
