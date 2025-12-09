use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the state directory for a project based on its .force/ path
pub fn get_state_dir(force_dir: &Path) -> PathBuf {
    let canonical = force_dir
        .canonicalize()
        .unwrap_or_else(|_| force_dir.to_path_buf());
    let hash = simple_hash(canonical.to_string_lossy().as_ref());

    dirs::state_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"))
        .join("force")
        .join(hash)
}

/// Simple string hash for creating project identifiers
fn simple_hash(s: &str) -> String {
    let mut hash: u64 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    format!("{:016x}", hash)
}

/// Get the sessions file path
fn sessions_file(force_dir: &Path) -> PathBuf {
    get_state_dir(force_dir).join("sessions")
}

/// Add a session to the state
pub fn add_session(force_dir: &Path, feature: &str) -> Result<(), Box<dyn std::error::Error>> {
    let state_dir = get_state_dir(force_dir);
    fs::create_dir_all(&state_dir)?;

    let mut sessions = load_sessions(force_dir)?;
    sessions.insert(feature.to_string());
    save_sessions(force_dir, &sessions)?;

    Ok(())
}

/// Remove a session from the state
pub fn remove_session(force_dir: &Path, feature: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut sessions = load_sessions(force_dir)?;
    sessions.remove(feature);
    save_sessions(force_dir, &sessions)?;

    Ok(())
}

/// List all sessions for a project
pub fn list_sessions(force_dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let sessions = load_sessions(force_dir)?;
    let mut list: Vec<String> = sessions.into_iter().collect();
    list.sort();
    Ok(list)
}

/// Load sessions from file
fn load_sessions(force_dir: &Path) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let path = sessions_file(force_dir);
    if !path.exists() {
        return Ok(HashSet::new());
    }

    let content = fs::read_to_string(&path)?;
    let sessions: HashSet<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    Ok(sessions)
}

/// Save sessions to file
fn save_sessions(
    force_dir: &Path,
    sessions: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = sessions_file(force_dir);
    let content: String = sessions
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if sessions.is_empty() {
        // Remove file if no sessions
        let _ = fs::remove_file(&path);
    } else {
        fs::write(&path, content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_simple_hash_deterministic() {
        let hash1 = simple_hash("/path/to/project/.force");
        let hash2 = simple_hash("/path/to/project/.force");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simple_hash_different_for_different_paths() {
        let hash1 = simple_hash("/path/to/project1/.force");
        let hash2 = simple_hash("/path/to/project2/.force");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_add_and_list_sessions() {
        let dir = TempDir::new().unwrap();
        let force_dir = dir.path().join(".force");
        fs::create_dir(&force_dir).unwrap();

        add_session(&force_dir, "feature-a").unwrap();
        add_session(&force_dir, "feature-b").unwrap();

        let sessions = list_sessions(&force_dir).unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"feature-a".to_string()));
        assert!(sessions.contains(&"feature-b".to_string()));
    }

    #[test]
    fn test_remove_session() {
        let dir = TempDir::new().unwrap();
        let force_dir = dir.path().join(".force");
        fs::create_dir(&force_dir).unwrap();

        add_session(&force_dir, "feature-a").unwrap();
        add_session(&force_dir, "feature-b").unwrap();
        remove_session(&force_dir, "feature-a").unwrap();

        let sessions = list_sessions(&force_dir).unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions.contains(&"feature-b".to_string()));
    }

    #[test]
    fn test_list_empty_sessions() {
        let dir = TempDir::new().unwrap();
        let force_dir = dir.path().join(".force");
        fs::create_dir(&force_dir).unwrap();

        let sessions = list_sessions(&force_dir).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_add_duplicate_session() {
        let dir = TempDir::new().unwrap();
        let force_dir = dir.path().join(".force");
        fs::create_dir(&force_dir).unwrap();

        add_session(&force_dir, "feature-a").unwrap();
        add_session(&force_dir, "feature-a").unwrap();

        let sessions = list_sessions(&force_dir).unwrap();
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_state_dir_is_absolute_path() {
        let dir = TempDir::new().unwrap();
        let force_dir = dir.path().join(".force");
        fs::create_dir(&force_dir).unwrap();

        let state_dir = get_state_dir(&force_dir);

        // State dir must be an absolute path, not contain unexpanded ~ or other shell shortcuts
        assert!(
            state_dir.is_absolute(),
            "State dir should be absolute, got: {:?}",
            state_dir
        );
        assert!(
            !state_dir.to_string_lossy().contains('~'),
            "State dir should not contain literal ~, got: {:?}",
            state_dir
        );
    }
}
