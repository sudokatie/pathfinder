//! Windows-specific platform functions.

use std::env;
use std::path::{Path, PathBuf};

/// Executable extensions on Windows (in priority order).
const PATHEXT_DEFAULT: &[&str] = &[".COM", ".EXE", ".BAT", ".CMD", ".VBS", ".JS", ".PS1"];

/// Get PATH entries (semicolon-separated on Windows).
pub fn get_path_entries() -> Vec<PathBuf> {
    env::var("PATH")
        .unwrap_or_default()
        .split(';')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Get executable extensions from PATHEXT or use defaults.
fn get_pathext() -> Vec<String> {
    env::var("PATHEXT")
        .unwrap_or_else(|_| PATHEXT_DEFAULT.join(";"))
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_uppercase())
        .collect()
}

/// Check if a file is executable on Windows.
/// Checks if file exists and has an executable extension.
pub fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    
    // Check if the file has an executable extension
    if let Some(ext) = path.extension() {
        let ext_upper = format!(".{}", ext.to_string_lossy().to_uppercase());
        get_pathext().iter().any(|pe| pe == &ext_upper)
    } else {
        false
    }
}

/// Find a command in a directory.
/// On Windows, we try the command with each PATHEXT extension.
pub fn find_command_in_dir(dir: &Path, command: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    
    // First, try exact match (for commands with extension already)
    let exact = dir.join(command);
    if exact.is_file() && is_executable(&exact) {
        return Some(exact);
    }
    
    // Try each PATHEXT extension
    for ext in get_pathext() {
        let name = format!("{}{}", command, ext.to_lowercase());
        let candidate = dir.join(&name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pathext() {
        let pathext = get_pathext();
        assert!(!pathext.is_empty());
        assert!(pathext.contains(&".EXE".to_string()));
    }

    #[test]
    fn test_is_executable_by_extension() {
        // Create a mock path with .exe extension
        let path = Path::new("C:\\Windows\\System32\\cmd.exe");
        // We can't fully test this without Windows, but we can test the extension logic
        if let Some(ext) = path.extension() {
            let ext_upper = format!(".{}", ext.to_string_lossy().to_uppercase());
            assert!(get_pathext().contains(&ext_upper));
        }
    }
}
