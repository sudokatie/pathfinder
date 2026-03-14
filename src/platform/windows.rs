//! Windows-specific platform functions.

use std::env;
use std::fs;
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

/// Check if a path is a .lnk shortcut file.
pub fn is_lnk_file(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.to_ascii_lowercase() == "lnk")
        .unwrap_or(false)
}

/// Parse a .lnk shortcut file and return the target path.
/// Returns None if the file cannot be parsed or is not a valid .lnk file.
pub fn parse_lnk_target(path: &Path) -> Option<PathBuf> {
    // .lnk file format: https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/
    // We parse the minimum needed to extract the target path.

    let data = fs::read(path).ok()?;

    // Check minimum size and magic bytes
    if data.len() < 76 {
        return None;
    }

    // Header magic: 4C 00 00 00 (LNK signature)
    if data[0..4] != [0x4C, 0x00, 0x00, 0x00] {
        return None;
    }

    // LinkFlags at offset 0x14 (4 bytes, little-endian)
    let link_flags = u32::from_le_bytes([data[0x14], data[0x15], data[0x16], data[0x17]]);

    // HasLinkTargetIDList flag is bit 0
    let has_id_list = (link_flags & 0x01) != 0;
    // HasLinkInfo flag is bit 1
    let has_link_info = (link_flags & 0x02) != 0;

    let mut offset = 76; // End of header

    // Skip LinkTargetIDList if present
    if has_id_list {
        if offset + 2 > data.len() {
            return None;
        }
        let id_list_size = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2 + id_list_size;
    }

    // Parse LinkInfo if present (contains local path)
    if has_link_info {
        if offset + 4 > data.len() {
            return None;
        }
        let link_info_size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;

        if offset + link_info_size > data.len() || link_info_size < 28 {
            return None;
        }

        // LinkInfoFlags at offset+8
        let link_info_flags = u32::from_le_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);

        // VolumeIDAndLocalBasePath flag is bit 0
        if (link_info_flags & 0x01) != 0 {
            // LocalBasePathOffset at offset+16
            let local_path_offset = u32::from_le_bytes([
                data[offset + 16],
                data[offset + 17],
                data[offset + 18],
                data[offset + 19],
            ]) as usize;

            if local_path_offset > 0 && offset + local_path_offset < data.len() {
                // Read null-terminated string
                let path_start = offset + local_path_offset;
                let path_end = data[path_start..]
                    .iter()
                    .position(|&b| b == 0)
                    .map(|p| path_start + p)
                    .unwrap_or(data.len());

                if path_end > path_start {
                    let path_bytes = &data[path_start..path_end];
                    if let Ok(path_str) = String::from_utf8(path_bytes.to_vec()) {
                        return Some(PathBuf::from(path_str));
                    }
                }
            }
        }
    }

    None
}

/// Information about a junction point or reparse point.
#[derive(Debug, Clone)]
pub struct ReparseInfo {
    /// Whether this is a junction point.
    pub is_junction: bool,
    /// Whether this is a symlink.
    pub is_symlink: bool,
    /// The target path (if available).
    pub target: Option<PathBuf>,
}

/// Check if a path is a junction point (Windows NTFS).
/// Junction points are directory symlinks created with `mklink /J`.
#[cfg(windows)]
pub fn get_reparse_info(path: &Path) -> Option<ReparseInfo> {
    use std::os::windows::fs::MetadataExt;

    let metadata = fs::symlink_metadata(path).ok()?;
    let attrs = metadata.file_attributes();

    // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
    if (attrs & 0x400) == 0 {
        return None;
    }

    // It's a reparse point - try to determine the type
    // We can use fs::read_link to get the target for symlinks
    let target = fs::read_link(path).ok();

    // Heuristic: junctions are typically directories, symlinks can be either
    let is_dir = metadata.is_dir();

    // For now, we detect based on whether read_link succeeds and it's a directory
    // True junction detection would require DeviceIoControl with FSCTL_GET_REPARSE_POINT
    Some(ReparseInfo {
        is_junction: is_dir && target.is_some(),
        is_symlink: target.is_some() && !is_dir,
        target,
    })
}

/// Stub for non-Windows platforms.
#[cfg(not(windows))]
pub fn get_reparse_info(_path: &Path) -> Option<ReparseInfo> {
    None
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
/// On Windows, we try the command with each PATHEXT extension.
/// Also handles App Execution Aliases in WindowsApps.
pub fn find_command_in_dir(dir: &Path, command: &str) -> Option<FindResult> {
    if !dir.is_dir() {
        return None;
    }

    // First, try exact match (for commands with extension already)
    let exact = dir.join(command);
    if exact.is_file() {
        let executable = is_executable(&exact);
        return Some(FindResult {
            path: exact,
            executable,
        });
    }

    // Try each PATHEXT extension
    for ext in get_pathext() {
        let name = format!("{}{}", command, ext.to_lowercase());
        let candidate = dir.join(&name);
        if candidate.is_file() {
            return Some(FindResult {
                path: candidate,
                executable: true, // Has PATHEXT extension, so executable
            });
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
                return Some(FindResult {
                    path: candidate,
                    executable: true,
                });
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

    #[test]
    fn test_is_lnk_file() {
        assert!(is_lnk_file(Path::new(
            "C:\\Users\\Test\\Desktop\\shortcut.lnk"
        )));
        assert!(is_lnk_file(Path::new("shortcut.LNK")));
        assert!(!is_lnk_file(Path::new("program.exe")));
        assert!(!is_lnk_file(Path::new("document.txt")));
    }

    #[test]
    fn test_parse_lnk_target_invalid() {
        // Non-existent file
        let result = parse_lnk_target(Path::new("C:\\nonexistent.lnk"));
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_lnk_target_not_lnk() {
        // Test with empty/invalid data - should return None
        // We can't easily create a temp file in cross-platform tests,
        // but the function should handle invalid data gracefully
        let result = parse_lnk_target(Path::new("/dev/null"));
        assert!(result.is_none());
    }
}
