//! Symlink resolution.
//!
//! Follows symlink chains and detects issues (broken, circular).
//! On Windows, also handles .lnk shortcut files and junction points.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum depth for symlink resolution to prevent infinite loops.
const MAX_SYMLINK_DEPTH: usize = 40;

/// Information about a symlink (or Windows .lnk / junction point).
#[derive(Debug, Clone)]
pub struct SymlinkInfo {
    /// The original symlink path.
    #[allow(dead_code)]
    pub original: PathBuf,
    /// The raw symlink target (as stored in the filesystem, may be relative).
    pub raw_target: Option<PathBuf>,
    /// The final fully-resolved target path (if resolvable).
    pub resolved: Option<PathBuf>,
    /// The chain of symlinks followed.
    #[allow(dead_code)]
    pub chain: Vec<PathBuf>,
    /// Whether the symlink is broken.
    pub is_broken: bool,
    /// Whether there's a circular reference.
    pub is_circular: bool,
    /// Whether this is a Windows .lnk shortcut file.
    pub is_lnk: bool,
    /// Whether this is a Windows junction point.
    pub is_junction: bool,
}

/// Check if a path is a symlink (or Windows .lnk file).
pub fn is_symlink(path: &Path) -> bool {
    // Check for regular symlink
    if path
        .symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return true;
    }

    // On Windows, also check for .lnk files
    #[cfg(windows)]
    {
        if crate::platform::is_lnk_file(path) && path.is_file() {
            return true;
        }
    }

    false
}

/// Resolve a symlink chain.
///
/// Returns information about the symlink, including the full chain
/// and any issues found (broken link, circular reference).
/// On Windows, also handles .lnk shortcut files and junction points.
pub fn resolve_symlink(path: &Path) -> SymlinkInfo {
    // Check for Windows .lnk file first
    #[cfg(windows)]
    {
        if crate::platform::is_lnk_file(path) {
            return resolve_lnk_file(path);
        }
    }

    // Check for Windows junction point
    #[cfg(windows)]
    let is_junction = {
        crate::platform::get_reparse_info(path)
            .map(|r| r.is_junction)
            .unwrap_or(false)
    };
    #[cfg(not(windows))]
    let is_junction = false;

    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut current = path.to_path_buf();

    // Read the raw target of the first symlink
    let raw_target = fs::read_link(path).ok();

    // Add the starting path
    chain.push(current.clone());
    seen.insert(current.clone());

    loop {
        // Check depth limit
        if chain.len() > MAX_SYMLINK_DEPTH {
            return SymlinkInfo {
                original: path.to_path_buf(),
                raw_target,
                resolved: None,
                chain,
                is_broken: false,
                is_circular: true,
                is_lnk: false,
                is_junction,
            };
        }

        // Check if current is a symlink
        if !is_symlink(&current) {
            // Reached a non-symlink - check if it exists
            if current.exists() {
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    raw_target,
                    resolved: Some(current),
                    chain,
                    is_broken: false,
                    is_circular: false,
                    is_lnk: false,
                    is_junction,
                };
            } else {
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    raw_target,
                    resolved: None,
                    chain,
                    is_broken: true,
                    is_circular: false,
                    is_lnk: false,
                    is_junction,
                };
            }
        }

        // Read the symlink target
        match fs::read_link(&current) {
            Ok(target) => {
                // Resolve relative targets
                let resolved_path = if target.is_absolute() {
                    target
                } else {
                    current.parent().map(|p| p.join(&target)).unwrap_or(target)
                };

                // Check for circular reference
                if seen.contains(&resolved_path) {
                    chain.push(resolved_path);
                    return SymlinkInfo {
                        original: path.to_path_buf(),
                        raw_target,
                        resolved: None,
                        chain,
                        is_broken: false,
                        is_circular: true,
                        is_lnk: false,
                        is_junction,
                    };
                }

                chain.push(resolved_path.clone());
                seen.insert(resolved_path.clone());
                current = resolved_path;
            }
            Err(_) => {
                // Can't read the symlink - broken
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    raw_target,
                    resolved: None,
                    chain,
                    is_broken: true,
                    is_circular: false,
                    is_lnk: false,
                    is_junction,
                };
            }
        }
    }
}

/// Resolve a Windows .lnk shortcut file.
#[cfg(windows)]
fn resolve_lnk_file(path: &Path) -> SymlinkInfo {
    let raw_target = crate::platform::parse_lnk_target(path);

    match &raw_target {
        Some(target) => {
            let resolved = if target.exists() {
                Some(target.clone())
            } else {
                None
            };
            let is_broken = resolved.is_none();

            SymlinkInfo {
                original: path.to_path_buf(),
                raw_target: raw_target.clone(),
                resolved,
                chain: vec![path.to_path_buf()],
                is_broken,
                is_circular: false,
                is_lnk: true,
                is_junction: false,
            }
        }
        None => {
            // Couldn't parse .lnk file
            SymlinkInfo {
                original: path.to_path_buf(),
                raw_target: None,
                resolved: None,
                chain: vec![path.to_path_buf()],
                is_broken: true,
                is_circular: false,
                is_lnk: true,
                is_junction: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn test_is_symlink_regular_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("regular.txt");
        fs::write(&file, "test").unwrap();

        assert!(!is_symlink(&file));
    }

    #[test]
    fn test_is_symlink_actual_symlink() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");

        fs::write(&file, "test").unwrap();
        symlink(&file, &link).unwrap();

        assert!(is_symlink(&link));
    }

    #[test]
    fn test_is_symlink_nonexistent() {
        assert!(!is_symlink(Path::new("/nonexistent/path")));
    }

    #[test]
    fn test_resolve_regular_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("regular.txt");
        fs::write(&file, "test").unwrap();

        let info = resolve_symlink(&file);
        assert!(!info.is_broken);
        assert!(!info.is_circular);
        assert_eq!(info.resolved, Some(file.clone()));
        assert!(info.raw_target.is_none()); // Not a symlink
    }

    #[test]
    fn test_resolve_simple_symlink() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");

        fs::write(&target, "test").unwrap();
        symlink(&target, &link).unwrap();

        let info = resolve_symlink(&link);
        assert!(!info.is_broken);
        assert!(!info.is_circular);
        assert_eq!(info.resolved, Some(target.clone()));
        assert_eq!(info.raw_target, Some(target)); // Absolute path in this case
        assert_eq!(info.chain.len(), 2);
    }

    #[test]
    fn test_resolve_broken_symlink() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("nonexistent.txt");
        let link = tmp.path().join("broken_link.txt");

        symlink(&target, &link).unwrap();

        let info = resolve_symlink(&link);
        assert!(info.is_broken);
        assert!(!info.is_circular);
        assert!(info.resolved.is_none());
        assert!(info.raw_target.is_some()); // We can still read what it points to
    }

    #[test]
    fn test_resolve_circular_symlink() {
        let tmp = TempDir::new().unwrap();
        let link_a = tmp.path().join("link_a");
        let link_b = tmp.path().join("link_b");

        // Create circular: a -> b -> a
        symlink(&link_b, &link_a).unwrap();
        symlink(&link_a, &link_b).unwrap();

        let info = resolve_symlink(&link_a);
        assert!(info.is_circular);
        assert!(!info.is_broken);
        assert!(info.resolved.is_none());
    }

    #[test]
    fn test_resolve_symlink_chain() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        let link1 = tmp.path().join("link1");
        let link2 = tmp.path().join("link2");
        let link3 = tmp.path().join("link3");

        fs::write(&target, "test").unwrap();
        symlink(&target, &link1).unwrap();
        symlink(&link1, &link2).unwrap();
        symlink(&link2, &link3).unwrap();

        let info = resolve_symlink(&link3);
        assert!(!info.is_broken);
        assert!(!info.is_circular);
        assert_eq!(info.resolved, Some(target));
        assert_eq!(info.chain.len(), 4); // link3 -> link2 -> link1 -> target
    }

    #[test]
    fn test_raw_target_preserved() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        let link = tmp.path().join("link.txt");

        fs::write(&target, "test").unwrap();
        symlink(&target, &link).unwrap();

        let info = resolve_symlink(&link);
        assert!(info.raw_target.is_some());
    }
}
