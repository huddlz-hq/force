# Force

A force multiplier for parallel AI development. Manage multiple git worktrees with isolated ports, databases, and environments.

## Install

### From releases (recommended)

Download the latest release for your platform:

```sh
# macOS (Apple Silicon)
curl -L https://github.com/huddlz-hq/force/releases/latest/download/force-macos-aarch64.tar.gz | tar xz
sudo mv force /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/huddlz-hq/force/releases/latest/download/force-macos-x86_64.tar.gz | tar xz
sudo mv force /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/huddlz-hq/force/releases/latest/download/force-linux-x86_64.tar.gz | tar xz
sudo mv force /usr/local/bin/

# Linux (aarch64)
curl -L https://github.com/huddlz-hq/force/releases/latest/download/force-linux-aarch64.tar.gz | tar xz
sudo mv force /usr/local/bin/
```

### From source

```sh
cargo install --path .
```

## Quick Start

```sh
# Initialize a new project
force init

# Spin up a feature session (creates worktree automatically)
force up add-login

# List active sessions
force ls

# Tear down when done (removes worktree)
force down add-login
```

## Documentation

- [Commands](docs/commands.md) - CLI reference
- [Scripts](docs/scripts.md) - TOML format and environment variables
