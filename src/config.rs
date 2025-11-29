use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

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

        // Only process .toml files
        if path.extension().map_or(false, |ext| ext == "toml") {
            let content = fs::read_to_string(&path)?;
            let script: Script = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

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
}
