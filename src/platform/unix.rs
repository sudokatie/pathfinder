//! Unix-specific platform functions.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Get PATH entries (colon-separated on Unix).
pub fn get_path_entries() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Check if a file is executable on Unix.
/// Checks if file exists and has execute permission.
pub fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match fs::metadata(path) {
        Ok(meta) => {
            let perms = meta.permissions();
            // Check if any execute bit is set (owner, group, or other)
            perms.mode() & 0o111 != 0
        }
        Err(_) => false,
    }
}

/// Find a command in a directory.
/// On Unix, we look for exact name match with execute permission.
pub fn find_command_in_dir(dir: &Path, command: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }

    let candidate = dir.join(command);
    if is_executable(&candidate) {
        return Some(candidate);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path_entries() {
        let entries = get_path_entries();
        // PATH should contain at least /usr/bin on most Unix systems
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_is_executable_ls() {
        // /bin/ls or /usr/bin/ls should exist and be executable
        let ls_paths = ["/bin/ls", "/usr/bin/ls"];
        let found = ls_paths.iter().any(|p| is_executable(Path::new(p)));
        assert!(found, "ls should be executable");
    }

    #[test]
    fn test_is_executable_nonexistent() {
        assert!(!is_executable(Path::new("/nonexistent/binary")));
    }

    #[test]
    fn test_find_command_in_dir_ls() {
        let dirs = ["/bin", "/usr/bin"];
        let found = dirs
            .iter()
            .any(|d| find_command_in_dir(Path::new(d), "ls").is_some());
        assert!(found, "Should find ls in /bin or /usr/bin");
    }

    #[test]
    fn test_find_command_in_dir_nonexistent() {
        let result = find_command_in_dir(Path::new("/usr/bin"), "nonexistent_cmd_12345");
        assert!(result.is_none());
    }
}
