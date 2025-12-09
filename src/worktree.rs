use std::path::{Path, PathBuf};
use std::process::Command;

pub struct WorktreeResult {
    pub path: PathBuf,
    pub created: bool, // true if newly created, false if reused
}

/// Create a git worktree for the feature
pub fn create_worktree(
    project_root: &Path,
    feature_slug: &str,
    path_template: &str,
) -> Result<WorktreeResult, Box<dyn std::error::Error>> {
    let worktree_path = expand_path_template(path_template, feature_slug);
    let absolute_path = resolve_path(project_root, &worktree_path);

    // Check if worktree already exists
    if absolute_path.exists() {
        if is_valid_worktree(&absolute_path) {
            return Ok(WorktreeResult {
                path: absolute_path,
                created: false,
            });
        } else {
            return Err(format!(
                "Path {} exists but is not a valid git worktree",
                absolute_path.display()
            )
            .into());
        }
    }

    // Create parent directories if needed
    if let Some(parent) = absolute_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Create the worktree with a new branch
    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &absolute_path.to_string_lossy(),
            "-b",
            feature_slug,
        ])
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        // Try without -b in case branch already exists
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                &absolute_path.to_string_lossy(),
                feature_slug,
            ])
            .current_dir(project_root)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to create worktree. Branch '{}' may exist in another worktree.\n{}",
                feature_slug, stderr
            )
            .into());
        }
    }

    Ok(WorktreeResult {
        path: absolute_path,
        created: true,
    })
}

/// Resolve worktree path without creating it
pub fn resolve_worktree_path(
    project_root: &Path,
    feature_slug: &str,
    path_template: &str,
) -> PathBuf {
    let worktree_path = expand_path_template(path_template, feature_slug);
    resolve_path(project_root, &worktree_path)
}

/// Remove a git worktree
pub fn remove_worktree(
    project_root: &Path,
    worktree_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !worktree_path.exists() {
        return Ok(());
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "remove",
            &worktree_path.to_string_lossy(),
            "--force",
        ])
        .current_dir(project_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Failed to remove worktree at {}: {}",
            worktree_path.display(),
            stderr
        )
        .into());
    }

    Ok(())
}

fn expand_path_template(template: &str, feature_slug: &str) -> String {
    template.replace("$FORCE_FEATURE_SLUG", feature_slug)
}

fn resolve_path(project_root: &Path, relative_path: &str) -> PathBuf {
    let path = PathBuf::from(relative_path);
    if path.is_absolute() {
        path
    } else {
        project_root.join(relative_path)
    }
}

fn is_valid_worktree(path: &Path) -> bool {
    // Worktrees have a .git file (not directory) that points to the main repo
    let git_path = path.join(".git");
    git_path.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path_template() {
        assert_eq!(
            expand_path_template("../worktrees/$FORCE_FEATURE_SLUG", "my_feature"),
            "../worktrees/my_feature"
        );
        assert_eq!(
            expand_path_template(".worktrees/$FORCE_FEATURE_SLUG", "test"),
            ".worktrees/test"
        );
    }

    #[test]
    fn test_resolve_path_relative() {
        let project_root = PathBuf::from("/home/user/project");
        let resolved = resolve_path(&project_root, "../worktrees/feature");
        assert_eq!(resolved, PathBuf::from("/home/user/project/../worktrees/feature"));
    }

    #[test]
    fn test_resolve_path_absolute() {
        let project_root = PathBuf::from("/home/user/project");
        let resolved = resolve_path(&project_root, "/tmp/worktrees/feature");
        assert_eq!(resolved, PathBuf::from("/tmp/worktrees/feature"));
    }
}
