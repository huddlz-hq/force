use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

// Default worktree configuration values
fn default_worktree_path() -> String {
    "../worktrees/$FORCE_FEATURE_SLUG".to_string()
}

fn default_remove_on_down() -> bool {
    true
}

/// Project-level Force configuration from .force/config.toml
#[derive(Debug, Deserialize, Default)]
pub struct ForceConfig {
    #[serde(default)]
    pub worktree: WorktreeConfig,
}

/// Worktree configuration options
#[derive(Debug, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default = "default_worktree_path")]
    pub path: String,
    #[serde(default = "default_remove_on_down")]
    pub remove_on_down: bool,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            path: default_worktree_path(),
            remove_on_down: default_remove_on_down(),
        }
    }
}

/// Load configuration from .force/config.toml
pub fn load_config(force_dir: &Path) -> Result<ForceConfig, Box<dyn std::error::Error>> {
    let config_path = force_dir.join("config.toml");
    if !config_path.exists() {
        return Ok(ForceConfig::default());
    }
    let content = fs::read_to_string(&config_path)?;
    let config: ForceConfig =
        toml::from_str(&content).map_err(|e| format!("Failed to parse config.toml: {}", e))?;
    Ok(config)
}

/// Parsed TOML script file
#[derive(Debug, Deserialize)]
pub struct Script {
    pub meta: ScriptMeta,
    pub up: ScriptCommand,
    pub down: Option<ScriptCommand>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptMeta {
    pub category: String,
    pub priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptCommand {
    pub run: String,
    pub description: Option<String>,
}

/// A loaded script with its file info
pub struct LoadedScript {
    pub name: String,
    pub script: Script,
}

/// Find .force/ directory by walking up from current directory
pub fn find_force_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current = std::env::current_dir()?;

    loop {
        let force_dir = current.join(".force");
        if force_dir.is_dir() {
            return Ok(force_dir);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(".force/ directory not found. Run 'force init' to create one.".into());
            }
        }
    }
}

/// Load all TOML scripts from .force/ directory
pub fn load_scripts(force_dir: &Path) -> Result<Vec<LoadedScript>, Box<dyn std::error::Error>> {
    let mut scripts = Vec::new();

    for entry in fs::read_dir(force_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process .toml files, skip config.toml
        if path.extension().is_some_and(|ext| ext == "toml") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Skip config.toml - it's not a script
            if name == "config" {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let script: Script = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

            scripts.push(LoadedScript { name, script });
        }
    }

    // Sort by category, then priority (default 0), then filename
    scripts.sort_by(|a, b| {
        let cat_cmp = a.script.meta.category.cmp(&b.script.meta.category);
        if cat_cmp != std::cmp::Ordering::Equal {
            return cat_cmp;
        }

        let pri_a = a.script.meta.priority.unwrap_or(0);
        let pri_b = b.script.meta.priority.unwrap_or(0);
        let pri_cmp = pri_a.cmp(&pri_b);
        if pri_cmp != std::cmp::Ordering::Equal {
            return pri_cmp;
        }

        a.name.cmp(&b.name)
    });

    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_toml() {
        let toml = r#"
[meta]
category = "setup"

[up]
run = "echo hello"
"#;
        let script: Script = toml::from_str(toml).unwrap();
        assert_eq!(script.meta.category, "setup");
        assert_eq!(script.meta.priority, None);
        assert_eq!(script.up.run, "echo hello");
        assert_eq!(script.up.description, None);
        assert!(script.down.is_none());
    }

    #[test]
    fn test_parse_full_toml() {
        let toml = r#"
[meta]
category = "setup"
priority = 5

[up]
run = "echo hello"
description = "Say hello"

[down]
run = "echo goodbye"
description = "Say goodbye"
"#;
        let script: Script = toml::from_str(toml).unwrap();
        assert_eq!(script.meta.category, "setup");
        assert_eq!(script.meta.priority, Some(5));
        assert_eq!(script.up.run, "echo hello");
        assert_eq!(script.up.description, Some("Say hello".to_string()));
        assert!(script.down.is_some());
        let down = script.down.unwrap();
        assert_eq!(down.run, "echo goodbye");
        assert_eq!(down.description, Some("Say goodbye".to_string()));
    }

    #[test]
    fn test_parse_missing_category_fails() {
        let toml = r#"
[meta]

[up]
run = "echo hello"
"#;
        let result: Result<Script, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_up_fails() {
        let toml = r#"
[meta]
category = "setup"
"#;
        let result: Result<Script, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_run_fails() {
        let toml = r#"
[meta]
category = "setup"

[up]
description = "No run command"
"#;
        let result: Result<Script, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_negative_priority() {
        let toml = r#"
[meta]
category = "setup"
priority = -10

[up]
run = "echo hello"
"#;
        let script: Script = toml::from_str(toml).unwrap();
        assert_eq!(script.meta.priority, Some(-10));
    }

    #[test]
    fn test_force_config_defaults() {
        let config = ForceConfig::default();
        assert_eq!(config.worktree.path, "../worktrees/$FORCE_FEATURE_SLUG");
        assert!(config.worktree.remove_on_down);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml = "";
        let config: ForceConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.worktree.path, "../worktrees/$FORCE_FEATURE_SLUG");
        assert!(config.worktree.remove_on_down);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml = r#"
[worktree]
path = ".worktrees/$FORCE_FEATURE_SLUG"
"#;
        let config: ForceConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.worktree.path, ".worktrees/$FORCE_FEATURE_SLUG");
        assert!(config.worktree.remove_on_down); // default
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[worktree]
path = "/tmp/worktrees/$FORCE_FEATURE_SLUG"
remove_on_down = false
"#;
        let config: ForceConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.worktree.path, "/tmp/worktrees/$FORCE_FEATURE_SLUG");
        assert!(!config.worktree.remove_on_down);
    }
}
