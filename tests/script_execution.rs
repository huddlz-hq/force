use assert_cmd::assert::Assert;
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
        output_file.display()
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
fn test_scripts_run_in_category_order() {
    let project = create_temp_project();
    let output_file = project.path().join("order.txt");

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
}

#[test]
fn test_deterministic_port_for_same_feature() {
    let project = create_temp_project();
    let mut ports = Vec::new();

    for _ in 0..2 {
        let output_file = project.path().join("port.txt");
        let _ = fs::remove_file(&output_file);

        let script = format!(
            r#"[meta]
category = "setup"

[up]
run = "echo $FORCE_PORT > {}"
"#,
            output_file.display()
        );

        let script_path = project.path().join(".force/port.toml");
        fs::write(&script_path, &script).unwrap();

        Assert::new(
            force_cmd()
                .args(["up", "consistent-feature"])
                .current_dir(project.path())
                .output()
                .unwrap(),
        )
        .success();

        let port = fs::read_to_string(&output_file).unwrap().trim().to_string();
        ports.push(port);
    }

    assert_eq!(ports[0], ports[1], "Port should be deterministic");
}

#[test]
fn test_different_features_get_different_ports() {
    let project = TempDir::new().unwrap();
    fs::create_dir(project.path().join(".force")).unwrap();

    let output_file = project.path().join("port.txt");
    let script = format!(
        r#"[meta]
category = "setup"

[up]
run = "echo $FORCE_PORT > {}"
"#,
        output_file.display()
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
