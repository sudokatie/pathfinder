//! Windows-specific platform functions.

use std::env;
use std::path::{Path, PathBuf};

/// Executable extensions on Windows (in priority order).
/// Per SPECS: .COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC;.PS1
const PATHEXT_DEFAULT: &[&str] = &[
    ".COM", ".EXE", ".BAT", ".CMD", ".VBS", ".VBE", ".JS", ".JSE", ".WSF", ".WSH", ".MSC", ".PS1",
];

/// Get PATH entries (semicolon-separated on Windows).
/// Handles quoted paths with spaces.
pub fn get_path_entries() -> Vec<PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();
    parse_windows_path(&path_var)
}

/// Parse a Windows PATH string, handling quoted paths.
fn parse_windows_path(path_str: &str) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in path_str.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ';' if !in_quotes => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    entries.push(PathBuf::from(trimmed));
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Don't forget the last entry
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        entries.push(PathBuf::from(trimmed));
    }

    entries
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

/// Check if a path is in the WindowsApps directory (UWP app aliases).
fn is_windows_apps_alias(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("windowsapps") || path_str.contains("microsoft\\windowsapps")
}

/// Find a command in a directory.
/// On Windows, we try the command with each PATHEXT extension.
/// Also handles App Execution Aliases in WindowsApps.
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

    // For WindowsApps directory, also check for app execution aliases
    // These are special reparse points that may not show up as regular files
    if is_windows_apps_alias(dir) {
        // Try common alias patterns
        for ext in &[".exe"] {
            let name = format!("{}{}", command, ext);
            let candidate = dir.join(&name);
            // App aliases exist as zero-byte files
            if candidate.exists() {
                return Some(candidate);
            }
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
    fn test_pathext_has_all_extensions() {
        let pathext = get_pathext();
        // Verify all required extensions are present in defaults
        let required = [
            ".COM", ".EXE", ".BAT", ".CMD", ".VBS", ".VBE", ".JS", ".JSE", ".WSF", ".WSH", ".MSC",
            ".PS1",
        ];
        for ext in &required {
            assert!(
                pathext.contains(&ext.to_string()),
                "Missing extension: {}",
                ext
            );
        }
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

    #[test]
    fn test_parse_windows_path_simple() {
        let path = "C:\\Windows;C:\\Windows\\System32";
        let entries = parse_windows_path(path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], PathBuf::from("C:\\Windows"));
        assert_eq!(entries[1], PathBuf::from("C:\\Windows\\System32"));
    }

    #[test]
    fn test_parse_windows_path_with_quotes() {
        let path = "\"C:\\Program Files\";C:\\Windows";
        let entries = parse_windows_path(path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], PathBuf::from("C:\\Program Files"));
        assert_eq!(entries[1], PathBuf::from("C:\\Windows"));
    }

    #[test]
    fn test_parse_windows_path_quoted_with_semicolon() {
        // A path that contains a semicolon inside quotes
        let path = "\"C:\\Some;Path\";C:\\Windows";
        let entries = parse_windows_path(path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], PathBuf::from("C:\\Some;Path"));
    }

    #[test]
    fn test_parse_windows_path_empty_entries() {
        let path = "C:\\Windows;;C:\\System32";
        let entries = parse_windows_path(path);
        assert_eq!(entries.len(), 2); // Empty entries should be skipped
    }

    #[test]
    fn test_is_windows_apps_alias() {
        assert!(is_windows_apps_alias(Path::new(
            "C:\\Users\\Test\\AppData\\Local\\Microsoft\\WindowsApps"
        )));
        assert!(!is_windows_apps_alias(Path::new("C:\\Windows\\System32")));
    }
}
