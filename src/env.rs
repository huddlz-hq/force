use std::path::{Path, PathBuf};

const BASE_PORT: u16 = 4000;

/// Environment context for scripts
pub struct ForceEnv {
    pub feature: String,
    pub feature_slug: String,
    pub port_offset: u16,
    pub port: u16,
    pub db_name: String,
    pub force_dir: PathBuf,
}

impl ForceEnv {
    pub fn new(feature: &str, force_dir: &Path) -> Self {
        let feature_slug = slugify(feature);
        let port_offset = hash_to_offset(feature);
        let port = BASE_PORT + port_offset;

        // Try to get project name from parent of .force/
        let project_name = force_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("app");

        let db_name = format!("{}_{}", slugify(project_name), feature_slug);

        Self {
            feature: feature.to_string(),
            feature_slug,
            port_offset,
            port,
            db_name,
            force_dir: force_dir.to_path_buf(),
        }
    }

    /// Convert to environment variable pairs
    pub fn to_env_vars(&self) -> Vec<(String, String)> {
        vec![
            ("FORCE_FEATURE".to_string(), self.feature.clone()),
            ("FORCE_FEATURE_SLUG".to_string(), self.feature_slug.clone()),
            (
                "FORCE_PORT_OFFSET".to_string(),
                self.port_offset.to_string(),
            ),
            ("FORCE_PORT".to_string(), self.port.to_string()),
            ("FORCE_DB_NAME".to_string(), self.db_name.clone()),
            (
                "FORCE_DIR".to_string(),
                self.force_dir.display().to_string(),
            ),
        ]
    }
}

/// Convert a feature name to a slug (lowercase ASCII, underscores)
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Hash a feature name to a port offset (0-999)
fn hash_to_offset(feature: &str) -> u16 {
    let hash: u32 = feature
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    (hash % 1000) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::path::PathBuf;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("add-login"), "add_login");
        assert_eq!(slugify("My Feature"), "my_feature");
        assert_eq!(slugify("feature_123"), "feature_123");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify("a@b#c$d"), "a_b_c_d");
    }

    #[test]
    fn test_hash_is_deterministic() {
        let offset1 = hash_to_offset("my-feature");
        let offset2 = hash_to_offset("my-feature");
        assert_eq!(offset1, offset2);
    }

    #[test]
    fn test_hash_is_in_range() {
        let offset = hash_to_offset("some-random-feature-name");
        assert!(offset < 1000);
    }

    #[test]
    fn test_hash_empty_string() {
        let offset = hash_to_offset("");
        assert!(offset < 1000);
    }

    #[test]
    fn test_force_env_to_env_vars() {
        let env = ForceEnv::new("my-feature", &PathBuf::from("/project/.force"));
        let vars = env.to_env_vars();

        assert_eq!(vars.len(), 6);

        let var_map: std::collections::HashMap<_, _> = vars.into_iter().collect();
        assert_eq!(
            var_map.get("FORCE_FEATURE"),
            Some(&"my-feature".to_string())
        );
        assert_eq!(
            var_map.get("FORCE_FEATURE_SLUG"),
            Some(&"my_feature".to_string())
        );
        assert!(var_map.contains_key("FORCE_PORT"));
        assert!(var_map.contains_key("FORCE_PORT_OFFSET"));
        assert!(var_map.contains_key("FORCE_DB_NAME"));
        assert!(var_map.contains_key("FORCE_DIR"));
    }

    #[test]
    fn test_force_env_db_name() {
        let env = ForceEnv::new("add-login", &PathBuf::from("/myproject/.force"));
        assert_eq!(env.db_name, "myproject_add_login");
    }

    // Property-based tests
    proptest! {
        #[test]
        fn prop_hash_always_in_range(s in ".*") {
            let offset = hash_to_offset(&s);
            prop_assert!(offset < 1000);
        }

        #[test]
        fn prop_hash_is_deterministic(s in ".*") {
            let offset1 = hash_to_offset(&s);
            let offset2 = hash_to_offset(&s);
            prop_assert_eq!(offset1, offset2);
        }

        #[test]
        fn prop_slugify_only_valid_chars(s in ".*") {
            let slug = slugify(&s);
            for c in slug.chars() {
                prop_assert!(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            }
        }

        #[test]
        fn prop_slugify_same_length(s in "[a-zA-Z0-9_ \\-]{0,100}") {
            // For ASCII-only input, slugify preserves character count
            let slug = slugify(&s);
            prop_assert_eq!(slug.chars().count(), s.chars().count());
        }

        #[test]
        fn prop_slugify_is_idempotent(s in ".*") {
            let slug1 = slugify(&s);
            let slug2 = slugify(&slug1);
            prop_assert_eq!(slug1, slug2);
        }

        #[test]
        fn prop_port_in_valid_range(feature in "[a-zA-Z][a-zA-Z0-9\\-]{0,50}") {
            let env = ForceEnv::new(&feature, &PathBuf::from("/test/.force"));
            prop_assert!(env.port >= 4000);
            prop_assert!(env.port < 5000);
        }
    }
}
