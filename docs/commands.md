# Commands

## force up

Spin up a new session by creating a git worktree and running all scripts.

```sh
force up <feature-name>
force u <feature-name>  # alias
```

**Example:**
```sh
force up add-login
```

This will:
1. Find the `.force/` directory (searches up from current directory)
2. Create a git worktree for the feature (or reuse existing)
3. Load all `.toml` script files
4. Run each script's `[up]` command in the worktree directory (sorted by category, priority, filename)
5. Register the session (visible via `force ls`)

## force down

Tear down a session by running `[down]` commands in reverse order and removing the worktree.

```sh
force down <feature-name>
force d <feature-name>  # alias
```

**Example:**
```sh
force down add-login
```

This will:
1. Find the `.force/` directory (searches up from current directory)
2. Load all `.toml` script files
3. Run each script's `[down]` command in the worktree directory (reverse order of `up`)
4. Scripts without a `[down]` section are skipped
5. Remove the git worktree (configurable via `remove_on_down` in config.toml)
6. Unregister the session

## force ls

List active sessions for the current project.

```sh
force ls
```

**Example output:**
```
Active sessions:
  add-login     port 4427
  fix-checkout  port 4891
```

Sessions are stored in `~/.local/state/force/` and tracked per-project.

## force init

Create a `.force/` folder with configuration and example scripts.

```sh
force init
```

**Example:**
```sh
cd my-project
force init
```

This will:
1. Create a `.force/` directory in the current folder
2. Add `config.toml` with worktree configuration options
3. Add `env.toml` script to create `.dev.local.env` and `.test.local.env`
4. Add `database.toml` script to create dev and test databases
5. Show next steps for customizing and using force
