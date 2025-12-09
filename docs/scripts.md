# Scripts

Scripts are TOML files in your `.force/` folder that define setup and teardown commands. All scripts run in the worktree directory.

## Folder Structure

```
your-project/
└── .force/
    ├── config.toml    # Force configuration (not a script)
    ├── env.toml       # Creates .env files
    └── database.toml  # Database setup
```

## TOML Format

Each script has three sections:

```toml
[meta]
category = "setup"    # Groups scripts together
priority = 1          # Optional: lower runs first (default: 0)

[up]
description = "What this script does"
run = "shell command here"

[down]  # Optional: for teardown
run = "cleanup command here"
```

## Execution Order

Scripts run in this order:
1. Category (alphabetically)
2. Priority (lower first, default 0)
3. Filename (alphabetically)

On `force down`, scripts run in reverse order.

## Environment Variables

Force provides these variables to every script:

| Variable | Example | Description |
|----------|---------|-------------|
| `FORCE_FEATURE` | `add-login` | Original feature name |
| `FORCE_FEATURE_SLUG` | `add_login` | Sanitized (lowercase, underscores) |
| `FORCE_PORT_OFFSET` | `427` | Stable offset 0-999 from feature hash |
| `FORCE_PORT` | `4427` | Base port (4000) + offset |
| `FORCE_DB_NAME` | `myapp_add_login` | Project name + feature slug |
| `FORCE_DIR` | `/path/to/.force` | Path to .force directory |
| `FORCE_WORKTREE` | `/path/to/worktrees/add_login` | Path to worktree directory |

## Configuration

Create `.force/config.toml` to customize worktree behavior:

```toml
[worktree]
# Path template for worktrees (default shown)
path = "../worktrees/$FORCE_FEATURE_SLUG"

# Remove worktree when running `force down` (default: true)
remove_on_down = true
```

## Examples

### Environment Files

```toml
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
run = "rm -f .dev.local.env .test.local.env"
```

### Database (PostgreSQL)

```toml
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
run = """
dropdb $FORCE_DB_NAME --if-exists
dropdb ${FORCE_DB_NAME}_test --if-exists
"""
```

### Phoenix Server

```toml
[meta]
category = "services"

[up]
description = "Start Phoenix on isolated port"
run = "PORT=$FORCE_PORT mix phx.server"
```

### Rails Server

```toml
[meta]
category = "services"

[up]
description = "Start Rails on isolated port"
run = "PORT=$FORCE_PORT bundle exec rails server"
```
