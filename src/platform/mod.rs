//! Platform-specific functionality.
//!
//! Abstracts PATH parsing and executable detection across Unix and Windows.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::path::{Path, PathBuf};

/// Result of finding a command in a directory.
#[derive(Debug, Clone)]
pub struct FindResult {
    /// Full path to the file.
    pub path: PathBuf,
    /// Whether the file is executable.
    pub executable: bool,
}

/// Get PATH entries as a vector of paths.
pub fn get_path_entries() -> Vec<PathBuf> {
    #[cfg(unix)]
    {
        unix::get_path_entries()
    }
    #[cfg(windows)]
    {
        windows::get_path_entries()
    }
}

/// Check if a file is executable.
#[allow(dead_code)]
pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        unix::is_executable(path)
    }
    #[cfg(windows)]
    {
        windows::is_executable(path)
    }
}

/// Find a command in a directory.
/// Returns the full path and whether it's executable.
pub fn find_command_in_dir(dir: &Path, command: &str) -> Option<FindResult> {
    #[cfg(unix)]
    {
        unix::find_command_in_dir(dir, command).map(|r| FindResult {
            path: r.path,
            executable: r.executable,
        })
    }
    #[cfg(windows)]
    {
        windows::find_command_in_dir(dir, command).map(|r| FindResult {
            path: r.path,
            executable: r.executable,
        })
    }
}

/// Get the PATH separator for the current platform.
#[allow(dead_code)]
pub fn path_separator() -> char {
    #[cfg(unix)]
    {
        ':'
    }
    #[cfg(windows)]
    {
        ';'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_path_entries_not_empty() {
        let entries = get_path_entries();
        assert!(!entries.is_empty(), "PATH should have at least one entry");
    }

    #[test]
    fn test_path_separator() {
        let sep = path_separator();
        #[cfg(unix)]
        assert_eq!(sep, ':');
        #[cfg(windows)]
        assert_eq!(sep, ';');
    }

    #[test]
    fn test_find_command_ls_or_cmd() {
        let entries = get_path_entries();
        let mut found = false;

        #[cfg(unix)]
        let command = "ls";
        #[cfg(windows)]
        let command = "cmd";

        for dir in &entries {
            if find_command_in_dir(dir, command).is_some() {
                found = true;
                break;
            }
        }

        assert!(found, "Should find {} in PATH", command);
    }

    #[test]
    fn test_find_command_executable() {
        let entries = get_path_entries();

        #[cfg(unix)]
        let command = "ls";
        #[cfg(windows)]
        let command = "cmd";

        for dir in &entries {
            if let Some(result) = find_command_in_dir(dir, command) {
                assert!(result.executable, "{} should be executable", command);
                return;
            }
        }

        panic!("Should find {}", command);
    }

    #[test]
    fn test_is_executable_nonexistent() {
        let path = Path::new("/nonexistent/path/to/binary");
        assert!(!is_executable(path));
    }

    #[test]
    fn test_find_command_nonexistent() {
        let dir = Path::new("/usr/bin");
        let result = find_command_in_dir(dir, "this_command_does_not_exist_12345");
        assert!(result.is_none());
    }
}
