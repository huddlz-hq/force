use std::fs;
use std::path::Path;

const CONFIG_EXAMPLE: &str = r#"# Force Configuration
# See: https://github.com/huddlz-hq/force

[worktree]
# Path template for worktrees (default shown)
# Available variables: $FORCE_FEATURE_SLUG
# path = "../worktrees/$FORCE_FEATURE_SLUG"

# Remove worktree when running `force down` (default: true)
# remove_on_down = true
"#;

const ENV_EXAMPLE: &str = r#"# Force Script: env.toml
#
# Creates .dev.local.env and .test.local.env in the worktree
# with PORT and DATABASE_URL for the feature branch.

[meta]
category = "setup"
priority = 1

[up]
description = "Create local env files"
run = """
cat > .dev.local.env << EOF
PORT=$FORCE_PORT
DATABASE_URL=postgres://localhost/$FORCE_DB_NAME
EOF

cat > .test.local.env << EOF
PORT=$FORCE_PORT
DATABASE_URL=postgres://localhost/${FORCE_DB_NAME}_test
EOF
"""

[down]
description = "Remove local env files"
run = "rm -f .dev.local.env .test.local.env"
"#;

const DATABASE_EXAMPLE: &str = r#"# Force Script: database.toml
#
# Creates dev and test databases for the feature branch.
# Available environment variables:
#   FORCE_FEATURE      - Original feature name (e.g., "add-login")
#   FORCE_FEATURE_SLUG - Sanitized name (e.g., "add_login")
#   FORCE_PORT         - Assigned port (e.g., 4427)
#   FORCE_PORT_OFFSET  - Port offset 0-999 (e.g., 427)
#   FORCE_DB_NAME      - Database name (e.g., "myapp_add_login")
#   FORCE_DIR          - Path to .force/ directory
#   FORCE_WORKTREE     - Path to the worktree directory

[meta]
category = "setup"
priority = 2

[up]
description = "Create feature databases"
run = """
createdb $FORCE_DB_NAME 2>/dev/null || echo 'Dev database exists'
createdb ${FORCE_DB_NAME}_test 2>/dev/null || echo 'Test database exists'
"""

[down]
description = "Drop feature databases"
run = """
dropdb $FORCE_DB_NAME --if-exists
dropdb ${FORCE_DB_NAME}_test --if-exists
"""
"#;

pub fn run_init() -> Result<(), Box<dyn std::error::Error>> {
    let force_dir = Path::new(".force");

    if force_dir.exists() {
        return Err(".force/ directory already exists".into());
    }

    fs::create_dir(force_dir)?;
    fs::write(force_dir.join("config.toml"), CONFIG_EXAMPLE)?;
    fs::write(force_dir.join("env.toml"), ENV_EXAMPLE)?;
    fs::write(force_dir.join("database.toml"), DATABASE_EXAMPLE)?;

    println!("Created .force/ directory with:");
    println!("  .force/config.toml   - Force configuration");
    println!("  .force/env.toml      - Creates .dev.local.env & .test.local.env");
    println!("  .force/database.toml - Creates dev & test databases");
    println!("\nGit worktrees are created automatically by Force.");
    println!("Edit the scripts to match your project, then run:");
    println!("  force up <feature-name>");

    Ok(())
}
