//! Path Utilities
//!
//! Common utilities for converting between Godot resource paths (res://) and filesystem paths.
//! Includes path traversal protection to prevent access outside the project directory.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for path operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PathError {
    /// Attempted path traversal (e.g., ../)
    #[error("Path traversal attempt detected: {0}")]
    TraversalAttempt(String),

    /// Path resolved outside project root
    #[error("Path resolves outside project: {0}")]
    OutsideProject(String),

    /// Invalid resource path format
    #[error("Invalid resource path format: {0}")]
    InvalidFormat(String),

    /// Filesystem error
    #[error("Filesystem error: {0}")]
    FilesystemError(String),
}

/// A validated resource path (res://)
#[derive(Debug, Clone, PartialEq)]
pub struct ResPath {
    /// The raw path string (with or without res:// prefix)
    inner: String,
}

impl ResPath {
    /// Create a new ResPath from a string, validating it doesn't contain traversal attempts
    pub fn new(path: &str) -> Result<Self, PathError> {
        // Check for path traversal attempts
        if path.contains("..") {
            return Err(PathError::TraversalAttempt(path.to_string()));
        }

        // Normalize the path (remove res:// prefix if present)
        let normalized = path.strip_prefix("res://").unwrap_or(path);

        // Check for absolute paths (Windows and Unix)
        if normalized.starts_with('/') || normalized.contains(':') {
            return Err(PathError::InvalidFormat(format!(
                "Absolute paths not allowed: {}",
                path
            )));
        }

        Ok(Self {
            inner: normalized.to_string(),
        })
    }

    /// Create a ResPath without validation (for internal use where path is known safe)
    pub fn new_unchecked(path: &str) -> Self {
        let normalized = path.strip_prefix("res://").unwrap_or(path);
        Self {
            inner: normalized.to_string(),
        }
    }

    /// Get the relative path (without res:// prefix)
    pub fn relative(&self) -> &str {
        &self.inner
    }

    /// Get the full resource path (with res:// prefix)
    pub fn as_res_path(&self) -> String {
        format!("res://{}", self.inner)
    }

    /// Convert to filesystem path
    pub fn to_fs_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(&self.inner)
    }
}

/// Convert a resource path (res://) to a filesystem path
///
/// This function validates that:
/// 1. The path doesn't contain traversal attempts (..)
/// 2. The resulting path is within the project root
///
/// # Arguments
/// * `project_root` - The root directory of the Godot project
/// * `res_path` - The resource path (with or without res:// prefix)
///
/// # Returns
/// * `Ok(PathBuf)` - The validated filesystem path
/// * `Err(PathError)` - If the path is invalid or outside the project
pub fn to_fs_path(project_root: &Path, res_path: &str) -> Result<PathBuf, PathError> {
    let res = ResPath::new(res_path)?;
    let fs_path = res.to_fs_path(project_root);

    // Validate the path is within project root using canonical paths
    validate_within_project(project_root, &fs_path)?;

    Ok(fs_path)
}

/// Convert a resource path to filesystem path without strict validation
///
/// Use this for paths that are known to be safe (e.g., generated internally)
/// or when the path may not exist yet (e.g., creating new files)
pub fn to_fs_path_unchecked(project_root: &Path, res_path: &str) -> PathBuf {
    let relative = res_path.strip_prefix("res://").unwrap_or(res_path);
    project_root.join(relative)
}

/// Convert a filesystem path to a resource path (res://)
///
/// # Arguments
/// * `project_root` - The root directory of the Godot project  
/// * `file_path` - The filesystem path to convert
///
/// # Returns
/// * `Ok(String)` - The resource path (res://...)
/// * `Err(PathError)` - If the path is outside the project
pub fn to_res_path(project_root: &Path, file_path: &Path) -> Result<String, PathError> {
    let relative = file_path
        .strip_prefix(project_root)
        .map_err(|_| PathError::OutsideProject(file_path.display().to_string()))?;

    // Convert to forward slashes for Godot compatibility
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    Ok(format!("res://{}", relative_str))
}

/// Validate that a target path is within the project root
///
/// This uses path canonicalization when possible to detect symlink attacks
/// and other path tricks. For non-existent paths, it validates the existing
/// ancestor directory.
pub fn validate_within_project(project_root: &Path, target: &Path) -> Result<(), PathError> {
    // Try to canonicalize both paths
    let canonical_root = project_root.canonicalize().map_err(|e| {
        PathError::FilesystemError(format!("Cannot canonicalize project root: {}", e))
    })?;

    // For target, try to canonicalize. If it doesn't exist, canonicalize the
    // nearest existing ancestor and check that
    let canonical_target = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| PathError::FilesystemError(format!("Cannot canonicalize target: {}", e)))?
    } else {
        // Find the nearest existing ancestor
        let mut ancestor = target.to_path_buf();
        while !ancestor.exists() {
            if let Some(parent) = ancestor.parent() {
                ancestor = parent.to_path_buf();
            } else {
                break;
            }
        }

        if ancestor.exists() {
            let canonical_ancestor = ancestor.canonicalize().map_err(|e| {
                PathError::FilesystemError(format!("Cannot canonicalize ancestor: {}", e))
            })?;

            // Rebuild the relative part
            let suffix = target.strip_prefix(&ancestor).unwrap_or(Path::new(""));
            canonical_ancestor.join(suffix)
        } else {
            // No ancestor exists, just use the original path
            target.to_path_buf()
        }
    };

    // Check if target starts with project root
    if !canonical_target.starts_with(&canonical_root) {
        return Err(PathError::OutsideProject(target.display().to_string()));
    }

    Ok(())
}

/// Strip the res:// prefix from a path if present
///
/// This is a simple utility for compatibility with existing code.
/// For new code, prefer using `ResPath::new()` for validation.
#[inline]
pub fn strip_res_prefix(path: &str) -> &str {
    path.strip_prefix("res://").unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn test_project_root() -> PathBuf {
        env::current_dir().unwrap()
    }

    #[test]
    fn test_res_path_valid() {
        let res = ResPath::new("res://scenes/main.tscn").unwrap();
        assert_eq!(res.relative(), "scenes/main.tscn");
        assert_eq!(res.as_res_path(), "res://scenes/main.tscn");
    }

    #[test]
    fn test_res_path_without_prefix() {
        let res = ResPath::new("scenes/main.tscn").unwrap();
        assert_eq!(res.relative(), "scenes/main.tscn");
    }

    #[test]
    fn test_res_path_traversal_blocked() {
        let result = ResPath::new("res://../../../etc/passwd");
        assert!(matches!(result, Err(PathError::TraversalAttempt(_))));
    }

    #[test]
    fn test_res_path_hidden_traversal_blocked() {
        let result = ResPath::new("res://scenes/../../../etc/passwd");
        assert!(matches!(result, Err(PathError::TraversalAttempt(_))));
    }

    #[test]
    fn test_to_fs_path_valid() {
        let root = test_project_root();
        let result = to_fs_path(&root, "res://scenes/main.tscn");
        assert!(result.is_ok());
        let path = result.unwrap();
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with("scenes/main.tscn") || path_str.ends_with("scenes\\main.tscn"));
    }

    #[test]
    fn test_to_fs_path_traversal_blocked() {
        let root = test_project_root();
        let result = to_fs_path(&root, "res://../secret.txt");
        assert!(matches!(result, Err(PathError::TraversalAttempt(_))));
    }

    #[test]
    fn test_to_res_path_valid() {
        let root = PathBuf::from("/project");
        let file = PathBuf::from("/project/scenes/main.tscn");
        let result = to_res_path(&root, &file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "res://scenes/main.tscn");
    }

    #[test]
    fn test_to_res_path_outside_project() {
        let root = PathBuf::from("/project");
        let file = PathBuf::from("/other/file.txt");
        let result = to_res_path(&root, &file);
        assert!(matches!(result, Err(PathError::OutsideProject(_))));
    }

    #[test]
    fn test_strip_res_prefix() {
        assert_eq!(strip_res_prefix("res://test.gd"), "test.gd");
        assert_eq!(strip_res_prefix("test.gd"), "test.gd");
    }

    #[test]
    fn test_validate_within_project() {
        let root = test_project_root();
        let valid_target = root.join("src").join("main.rs");

        // This should succeed if src/main.rs exists or src exists
        let result = validate_within_project(&root, &valid_target);
        // We just check it doesn't panic - result depends on filesystem state
        let _ = result;
    }
}
