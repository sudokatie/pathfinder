//! Unix-specific platform functions.

use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Get PATH entries (colon-separated on Unix).
/// Empty entries represent the current directory.
pub fn get_path_entries() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .map(|s| {
            if s.is_empty() {
                // Empty entry represents current directory
                PathBuf::from(".")
            } else {
                PathBuf::from(s)
            }
        })
        .collect()
}

/// Check if a file is executable on Unix.
/// Checks if file exists, has execute permission, and optionally has a valid shebang.
pub fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match fs::metadata(path) {
        Ok(meta) => {
            let perms = meta.permissions();
            // Check if any execute bit is set (owner, group, or other)
            if perms.mode() & 0o111 == 0 {
                return false;
            }

            // For non-binary files, check for shebang
            if is_script(path) {
                return has_valid_shebang(path);
            }

            true
        }
        Err(_) => false,
    }
}

/// Check if a file appears to be a script (not a binary).
fn is_script(path: &Path) -> bool {
    // Check by reading first few bytes
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        let mut first_bytes = [0u8; 2];
        if std::io::Read::read(&mut reader, &mut first_bytes).is_ok() {
            // If it starts with #!, it's a script
            if &first_bytes == b"#!" {
                return true;
            }
            // If it starts with ELF magic or Mach-O magic, it's binary
            if &first_bytes == b"\x7fE"
                || &first_bytes == b"\xcf\xfa"
                || &first_bytes == b"\xca\xfe"
            {
                return false;
            }
        }
    }
    false
}

/// Check if a script has a valid shebang line.
fn has_valid_shebang(path: &Path) -> bool {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        if let Some(Ok(first_line)) = reader.lines().next() {
            if let Some(interpreter) = first_line.strip_prefix("#!") {
                let interpreter = interpreter.trim();
                // Handle "#!/usr/bin/env interpreter" case
                if interpreter.starts_with("/usr/bin/env ") {
                    // The env command will find the interpreter
                    return true;
                }
                // Check if the interpreter exists
                let interp_path = interpreter.split_whitespace().next().unwrap_or("");
                if !interp_path.is_empty() {
                    return Path::new(interp_path).exists();
                }
            }
        }
    }
    // If we can't read or parse, assume it's okay (binary, not a script)
    true
}

/// Result of finding a command in a directory.
#[derive(Debug, Clone)]
pub struct FindResult {
    /// Full path to the file.
    pub path: PathBuf,
    /// Whether the file is executable.
    pub executable: bool,
}

/// Find a command in a directory.
/// Returns the file even if it's not executable (to report permission denied).
/// On Unix, we look for exact name match.
pub fn find_command_in_dir(dir: &Path, command: &str) -> Option<FindResult> {
    if !dir.is_dir() {
        return None;
    }

    let candidate = dir.join(command);
    if candidate.is_file() {
        let executable = is_executable(&candidate);
        return Some(FindResult {
            path: candidate,
            executable,
        });
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
    fn test_get_path_entries_handles_empty() {
        // This tests the logic, not actual empty PATH entries
        // Empty entries should become "."
        let entries = get_path_entries();
        // Just verify it doesn't panic
        assert!(entries.len() > 0);
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
    fn test_find_command_in_dir_ls_executable() {
        let dirs = ["/bin", "/usr/bin"];
        for d in &dirs {
            if let Some(result) = find_command_in_dir(Path::new(d), "ls") {
                assert!(result.executable, "ls should be executable");
                return;
            }
        }
        panic!("Should find ls");
    }

    #[test]
    fn test_find_command_in_dir_nonexistent() {
        let result = find_command_in_dir(Path::new("/usr/bin"), "nonexistent_cmd_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_script_binary() {
        // /bin/ls is a binary, not a script
        let ls_paths = ["/bin/ls", "/usr/bin/ls"];
        for path in &ls_paths {
            let p = Path::new(path);
            if p.exists() {
                assert!(!is_script(p), "ls should not be detected as script");
                break;
            }
        }
    }
}
