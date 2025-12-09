use assert_cmd::assert::Assert;
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

fn order_tracking_script(
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
description = "Track order: {}"
run = "echo '{}' >> {}"
"#,
        category,
        priority_line,
        name,
        name,
        abs_path.display()
    )
}

fn env_capture_script(category: &str, output_file: &Path) -> String {
    // Use absolute path since scripts run in worktree directory
    let abs_path = output_file.canonicalize().unwrap_or_else(|_| output_file.to_path_buf());
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
fn test_scripts_run_in_category_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    // Create scripts in reverse alphabetical order
    create_script(
        project.path(),
        "zebra",
        &order_tracking_script("services", None, "zebra", &output_file),
    );
    create_script(
        project.path(),
        "alpha",
        &order_tracking_script("setup", None, "alpha", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // "services" comes before "setup" alphabetically
    assert_eq!(lines, vec!["zebra", "alpha"]);
}

#[test]
fn test_scripts_run_in_priority_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    create_script(
        project.path(),
        "second",
        &order_tracking_script("setup", Some(2), "second", &output_file),
    );
    create_script(
        project.path(),
        "first",
        &order_tracking_script("setup", Some(1), "first", &output_file),
    );
    create_script(
        project.path(),
        "third",
        &order_tracking_script("setup", Some(3), "third", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines, vec!["first", "second", "third"]);
}

#[test]
fn test_scripts_run_in_filename_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    create_script(
        project.path(),
        "charlie",
        &order_tracking_script("setup", None, "charlie", &output_file),
    );
    create_script(
        project.path(),
        "alpha",
        &order_tracking_script("setup", None, "alpha", &output_file),
    );
    create_script(
        project.path(),
        "bravo",
        &order_tracking_script("setup", None, "bravo", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines, vec!["alpha", "bravo", "charlie"]);
}

#[test]
fn test_full_sorting_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    create_script(
        project.path(),
        "svc_low",
        &order_tracking_script("services", Some(1), "svc_low", &output_file),
    );
    create_script(
        project.path(),
        "setup_high",
        &order_tracking_script("setup", Some(10), "setup_high", &output_file),
    );
    create_script(
        project.path(),
        "setup_low",
        &order_tracking_script("setup", Some(1), "setup_low", &output_file),
    );
    create_script(
        project.path(),
        "svc_high",
        &order_tracking_script("services", Some(10), "svc_high", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = output.lines().collect();

    // services before setup, then by priority
    assert_eq!(
        lines,
        vec!["svc_low", "svc_high", "setup_low", "setup_high"]
    );
}

#[test]
fn test_script_receives_env_vars() {
    let project = create_temp_project();
    let output_file = project.path().join("env.txt");

    // Create the file first so canonicalize works
    fs::write(&output_file, "").unwrap();

    create_script(
        project.path(),
        "env_check",
        &env_capture_script("setup", &output_file),
    );

    Assert::new(
        force_cmd()
            .args(["up", "test-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();

    assert!(output.contains("FORCE_FEATURE=test-feature"));
    assert!(output.contains("FORCE_FEATURE_SLUG=test_feature"));
    assert!(output.contains("FORCE_PORT="));
    assert!(output.contains("FORCE_PORT_OFFSET="));
    assert!(output.contains("FORCE_DB_NAME="));
    assert!(output.contains("FORCE_DIR="));
    assert!(output.contains("FORCE_WORKTREE="));
}

#[test]
fn test_deterministic_port_for_same_feature() {
    let project = create_temp_project();
    let output_file = project.path().join("port.txt");

    // Create the file first so we can get absolute path
    fs::write(&output_file, "").unwrap();
    let abs_path = output_file.canonicalize().unwrap();

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo $FORCE_PORT > {}"
"#,
        abs_path.display()
    );

    let script_path = project.path().join(".force/port.toml");
    fs::write(&script_path, &script).unwrap();

    // Run twice with same feature name - the worktree will be reused
    Assert::new(
        force_cmd()
            .args(["up", "consistent-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let port1 = fs::read_to_string(&output_file).unwrap().trim().to_string();

    // Clear the file
    fs::write(&output_file, "").unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "consistent-feature"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let port2 = fs::read_to_string(&output_file).unwrap().trim().to_string();

    assert_eq!(port1, port2, "Port should be deterministic");
}

#[test]
fn test_different_features_get_different_ports() {
    let project = create_temp_project();
    let output_file = project.path().join("port.txt");

    // Create the file first so we can get absolute path
    fs::write(&output_file, "").unwrap();
    let abs_path = output_file.canonicalize().unwrap();

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo $FORCE_PORT > {}"
"#,
        abs_path.display()
    );
    fs::write(project.path().join(".force/port.toml"), &script).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "feature-a"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();
    let port_a = fs::read_to_string(&output_file).unwrap().trim().to_string();

    // Clear file for next run
    fs::write(&output_file, "").unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "feature-b"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();
    let port_b = fs::read_to_string(&output_file).unwrap().trim().to_string();

    assert_ne!(
        port_a, port_b,
        "Different features should get different ports"
    );
}

#[test]
fn test_scripts_run_in_worktree_directory() {
    let project = create_temp_project();
    let output_file = project.path().join("pwd_output.txt");

    // Create the file first so we can get absolute path
    fs::write(&output_file, "").unwrap();
    let abs_path = output_file.canonicalize().unwrap();

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "pwd > {}"
"#,
        abs_path.display()
    );
    fs::write(project.path().join(".force/pwd.toml"), &script).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "pwd-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let pwd_output = fs::read_to_string(&output_file).unwrap();
    let pwd_output = pwd_output.trim();

    // pwd should show worktree path, not project path
    assert!(
        pwd_output.contains("worktrees/pwd_test"),
        "Script should run in worktree directory, got: {}",
        pwd_output
    );
    assert!(
        !pwd_output.ends_with(project.path().file_name().unwrap().to_str().unwrap()),
        "Script should NOT run in project directory"
    );
}

#[test]
fn test_force_worktree_env_var_matches_pwd() {
    let project = create_temp_project();
    let output_file = project.path().join("worktree_check.txt");

    // Create the file first so we can get absolute path
    fs::write(&output_file, "").unwrap();
    let abs_path = output_file.canonicalize().unwrap();

    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo \"WORKTREE=$FORCE_WORKTREE\" > {} && echo \"PWD=$(pwd)\" >> {}"
"#,
        abs_path.display(),
        abs_path.display()
    );
    fs::write(project.path().join(".force/check.toml"), &script).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "worktree-env-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let output = fs::read_to_string(&output_file).unwrap();
    assert!(output.contains("WORKTREE="), "Should have FORCE_WORKTREE");
    assert!(output.contains("PWD="), "Should have pwd output");

    // Extract both values and compare
    let lines: Vec<&str> = output.lines().collect();
    let worktree_line = lines.iter().find(|l| l.starts_with("WORKTREE=")).unwrap();
    let pwd_line = lines.iter().find(|l| l.starts_with("PWD=")).unwrap();

    let worktree_val = worktree_line.strip_prefix("WORKTREE=").unwrap();
    let pwd_val = pwd_line.strip_prefix("PWD=").unwrap();

    // pwd and FORCE_WORKTREE should point to the same location
    // (allow for symlink resolution differences)
    assert!(
        worktree_val.contains("worktree_env_test") && pwd_val.contains("worktree_env_test"),
        "Both FORCE_WORKTREE and pwd should reference the worktree\nWORKTREE={}\nPWD={}",
        worktree_val,
        pwd_val
    );
}
