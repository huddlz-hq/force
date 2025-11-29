use std::fs;
use std::path::Path;

const WORKTREE_EXAMPLE: &str = r#"# Force Script: worktree.toml
#
# Scripts are sorted by: category (alphabetical), priority (lower first), filename
# Available environment variables:
#   FORCE_FEATURE      - Original feature name (e.g., "add-login")
#   FORCE_FEATURE_SLUG - Sanitized name (e.g., "add_login")
#   FORCE_PORT         - Assigned port (e.g., 4427)
#   FORCE_PORT_OFFSET  - Port offset 0-999 (e.g., 427)
#   FORCE_DB_NAME      - Database name (e.g., "myapp_add_login")
#   FORCE_DIR          - Path to .force/ directory

[meta]
category = "setup"    # Scripts run in category order (alphabetical)
priority = 1          # Lower numbers run first within a category

[up]
description = "Create git worktree for feature"
run = "git worktree add ../worktrees/$FORCE_FEATURE_SLUG -b $FORCE_FEATURE_SLUG 2>/dev/null || echo 'Worktree exists'"

[down]
description = "Remove git worktree"
run = "git worktree remove ../worktrees/$FORCE_FEATURE_SLUG --force 2>/dev/null || echo 'Worktree already removed'"
"#;

pub fn run_init() -> Result<(), Box<dyn std::error::Error>> {
    let force_dir = Path::new(".force");

    if force_dir.exists() {
        return Err(".force/ directory already exists".into());
    }

    fs::create_dir(force_dir)?;
    fs::write(force_dir.join("worktree.toml"), WORKTREE_EXAMPLE)?;

    println!("Created .force/ directory with example script");
    println!("  .force/worktree.toml");
    println!("\nEdit the scripts to match your project, then run:");
    println!("  force up <feature-name>");

    Ok(())
}
