# Force

A force multiplier for parallel AI development. Manage multiple git worktrees with isolated ports, databases, and environments.

## Install

```sh
cargo install --path .
```

## Quick Start

1. Create a `.force/` folder in your project root
2. Add script files (see [docs/scripts.md](docs/scripts.md))
3. Run `force up my-feature`

```sh
force up add-login
# Creates worktree, database, starts services with isolated ports
```

## Documentation

- [Commands](docs/commands.md) - CLI reference
- [Scripts](docs/scripts.md) - TOML format and environment variables
