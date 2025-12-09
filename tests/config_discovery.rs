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

#[test]
fn test_finds_force_dir_in_current() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    Assert::new(
        force_cmd()
            .args(["up", "config-current-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_finds_force_dir_in_parent() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    let subdir = project.path().join("src");
    fs::create_dir(&subdir).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "config-parent-test"])
            .current_dir(&subdir)
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_finds_force_dir_in_grandparent() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    let subdir = project.path().join("src/components");
    fs::create_dir_all(&subdir).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "config-grandparent-test"])
            .current_dir(&subdir)
            .output()
            .unwrap(),
    )
    .success()
    .stdout(predicate::str::contains("Found .force/"));
}

#[test]
fn test_error_when_no_force_dir() {
    let dir = TempDir::new().unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(dir.path())
            .output()
            .unwrap(),
    )
    .failure()
    .stderr(predicate::str::contains(".force/ directory not found"));
}

#[test]
fn test_uses_closest_force_dir() {
    let outer = create_temp_project();
    let outer_output = outer.path().join("outer.txt");

    // Create outer output file and get absolute path
    fs::write(&outer_output, "").unwrap();
    let outer_abs = outer_output.canonicalize().unwrap();

    create_script(
        outer.path(),
        "outer",
        &format!(
            r#"[meta]
category = "setup"

[up]
run = "echo outer >> {}"
"#,
            outer_abs.display()
        ),
    );

    let inner_dir = outer.path().join("inner");
    fs::create_dir(&inner_dir).unwrap();
    fs::create_dir(inner_dir.join(".force")).unwrap();

    // Initialize git in inner dir too
    Command::new("git")
        .args(["init"])
        .current_dir(&inner_dir)
        .output()
        .expect("Failed to init git");
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(&inner_dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&inner_dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", "Initial commit"])
        .current_dir(&inner_dir)
        .output()
        .unwrap();

    let inner_output = inner_dir.join("inner.txt");
    // Create inner output file and get absolute path
    fs::write(&inner_output, "").unwrap();
    let inner_abs = inner_output.canonicalize().unwrap();

    fs::write(
        inner_dir.join(".force/inner.toml"),
        format!(
            r#"[meta]
category = "setup"

[up]
run = "echo inner >> {}"
"#,
            inner_abs.display()
        ),
    )
    .unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "feature"])
            .current_dir(&inner_dir)
            .output()
            .unwrap(),
    )
    .success();

    let inner_content = fs::read_to_string(&inner_output).unwrap();
    assert!(
        inner_content.contains("inner"),
        "Inner script should have run"
    );
    let outer_content = fs::read_to_string(&outer_output).unwrap();
    assert!(
        !outer_content.contains("outer"),
        "Outer script should NOT have run"
    );
}

#[test]
fn test_default_config_when_no_config_toml() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    // No config.toml - should use defaults
    Assert::new(
        force_cmd()
            .args(["up", "default-config-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Default worktree path is ../worktrees/$FORCE_FEATURE_SLUG
    let worktree_path = project
        .path()
        .parent()
        .unwrap()
        .join("worktrees/default_config_test");
    assert!(
        worktree_path.exists(),
        "Worktree should be at default path: {:?}",
        worktree_path
    );
}

#[test]
fn test_config_toml_custom_path() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    // Create config with custom path
    let config = r#"[worktree]
path = "custom-trees/$FORCE_FEATURE_SLUG"
"#;
    fs::write(project.path().join(".force/config.toml"), config).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "custom-config-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    let worktree_path = project.path().join("custom-trees/custom_config_test");
    assert!(
        worktree_path.exists(),
        "Worktree should be at custom path: {:?}",
        worktree_path
    );
}

#[test]
fn test_config_toml_partial() {
    let project = create_temp_project();
    create_script(project.path(), "test", &minimal_script("setup"));

    // Create config with only some fields - others should use defaults
    let config = r#"[worktree]
remove_on_down = false
"#;
    fs::write(project.path().join(".force/config.toml"), config).unwrap();

    Assert::new(
        force_cmd()
            .args(["up", "partial-config-test"])
            .current_dir(project.path())
            .output()
            .unwrap(),
    )
    .success();

    // Should use default path since only remove_on_down was specified
    let worktree_path = project
        .path()
        .parent()
        .unwrap()
        .join("worktrees/partial_config_test");
    assert!(
        worktree_path.exists(),
        "Worktree should still be at default path when only other settings are customized"
    );
}
