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

    // Use absolute path since scripts run in worktree directory
    let abs_path = output_file.canonicalize().unwrap_or_else(|_| output_file.to_path_buf());

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
        abs_path.display()
    )
}

#[test]
fn test_down_with_single_script() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-single-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["down", "down-single-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains(
        "Session 'down-single-test' torn down.",
    ));
}

#[test]
fn test_down_with_alias() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-alias-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["d", "down-alias-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Session 'down-alias-test' torn down."));
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

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-skip-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["down", "down-skip-test"])
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

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    // First run up to create the worktree, then down to test ordering
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

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-reverse-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["down", "down-reverse-test"])
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

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

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

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-priority-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["down", "down-priority-test"])
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

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "feature"])
        .current_dir(project.path())
        .output()
        .unwrap();

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

    // Create the file first so we can get its absolute path
    fs::write(&output_file, "").unwrap();
    let abs_path = output_file.canonicalize().unwrap();

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo 'up'"

[down]
run = "echo \"FORCE_FEATURE=$FORCE_FEATURE\" >> {}"
"#,
        abs_path.display()
    );
    create_script(project.path(), "env_check", &script);

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-env-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    Assert::new(
        force_cmd()
            .args(["down", "down-env-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    assert!(output.contains("FORCE_FEATURE=down-env-test"));
}

#[test]
fn test_down_removes_worktree() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    // Run up first to create the worktree
    force_cmd()
        .args(["up", "down-remove-wt-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    let worktree_path = project
        .path()
        .parent()
        .unwrap()
        .join("worktrees/down_remove_wt_test");
    assert!(worktree_path.exists(), "Worktree should exist after up");

    // Run down
    Assert::new(
        force_cmd()
            .args(["down", "down-remove-wt-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Verify worktree was removed
    assert!(
        !worktree_path.exists(),
        "Worktree should be removed after down"
    );
}

#[test]
fn test_down_preserves_worktree_when_configured() {
    let project = create_temp_project();
    create_script(project.path(), "hello", &script_with_down("setup"));

    // Create config to preserve worktree
    let config = r#"[worktree]
remove_on_down = false
"#;
    fs::write(project.path().join(".force/config.toml"), config).unwrap();

    // Run up first
    force_cmd()
        .args(["up", "down-preserve-test"])
        .current_dir(project.path())
        .output()
        .unwrap();

    let worktree_path = project
        .path()
        .parent()
        .unwrap()
        .join("worktrees/down_preserve_test");
    assert!(worktree_path.exists(), "Worktree should exist after up");

    // Run down
    Assert::new(
        force_cmd()
            .args(["down", "down-preserve-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Verify worktree was NOT removed
    assert!(
        worktree_path.exists(),
        "Worktree should still exist when remove_on_down = false"
    );
}
