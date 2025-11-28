# Scripts

Scripts are TOML files in your `.force/` folder that define setup and teardown commands.

## Folder Structure

```
your-project/
└── .force/
    ├── worktree.toml
    ├── database.toml
    └── server.toml
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

## Examples

### Git Worktree

```toml
[meta]
category = "setup"
priority = 1

[up]
description = "Create git worktree"
run = "git worktree add ../$FORCE_FEATURE_SLUG -b $FORCE_FEATURE_SLUG"

[down]
run = "git worktree remove ../$FORCE_FEATURE_SLUG"
```

### Database (PostgreSQL)

```toml
[meta]
category = "setup"
priority = 2

[up]
description = "Create feature database"
run = "createdb $FORCE_DB_NAME || echo 'Database exists'"

[down]
run = "dropdb $FORCE_DB_NAME --if-exists"
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
