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

## force status (coming soon)

List active sessions.

```sh
force status
```

## force init (coming soon)

Create a `.force/` folder with example scripts.

```sh
force init
```
