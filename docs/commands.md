# Commands

## force up

Spin up a new session by running all scripts in your `.force/` folder.

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
2. Load all `.toml` script files
3. Run each script's `[up]` command in order (sorted by category, priority, filename)
4. Register the session (visible via `force ls`)

## force down

Tear down a session by running `[down]` commands in reverse order.

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
3. Run each script's `[down]` command in reverse order (opposite of `up`)
4. Scripts without a `[down]` section are skipped
5. Unregister the session

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

Create a `.force/` folder with example scripts.

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
2. Add an example `worktree.toml` script with detailed comments
3. Show next steps for customizing and using force
