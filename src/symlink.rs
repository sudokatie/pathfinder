//! Symlink resolution.
//!
//! Follows symlink chains and detects issues (broken, circular).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum depth for symlink resolution to prevent infinite loops.
const MAX_SYMLINK_DEPTH: usize = 40;

/// Information about a symlink.
#[derive(Debug, Clone)]
pub struct SymlinkInfo {
    /// The original symlink path.
    pub original: PathBuf,
    /// The final resolved target (if resolvable).
    pub target: Option<PathBuf>,
    /// The chain of symlinks followed.
    pub chain: Vec<PathBuf>,
    /// Whether the symlink is broken.
    pub is_broken: bool,
    /// Whether there's a circular reference.
    pub is_circular: bool,
}

/// Check if a path is a symlink.
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Resolve a symlink chain.
///
/// Returns information about the symlink, including the full chain
/// and any issues found (broken link, circular reference).
pub fn resolve_symlink(path: &Path) -> SymlinkInfo {
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut current = path.to_path_buf();
    
    // Add the starting path
    chain.push(current.clone());
    seen.insert(current.clone());
    
    loop {
        // Check depth limit
        if chain.len() > MAX_SYMLINK_DEPTH {
            return SymlinkInfo {
                original: path.to_path_buf(),
                target: None,
                chain,
                is_broken: false,
                is_circular: true,
            };
        }
        
        // Check if current is a symlink
        if !is_symlink(&current) {
            // Reached a non-symlink - check if it exists
            if current.exists() {
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    target: Some(current),
                    chain,
                    is_broken: false,
                    is_circular: false,
                };
            } else {
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    target: None,
                    chain,
                    is_broken: true,
                    is_circular: false,
                };
            }
        }
        
        // Read the symlink target
        match fs::read_link(&current) {
            Ok(target) => {
                // Resolve relative targets
                let resolved = if target.is_absolute() {
                    target
                } else {
                    current.parent()
                        .map(|p| p.join(&target))
                        .unwrap_or(target)
                };
                
                // Check for circular reference
                if seen.contains(&resolved) {
                    chain.push(resolved);
                    return SymlinkInfo {
                        original: path.to_path_buf(),
                        target: None,
                        chain,
                        is_broken: false,
                        is_circular: true,
                    };
                }
                
                chain.push(resolved.clone());
                seen.insert(resolved.clone());
                current = resolved;
            }
            Err(_) => {
                // Can't read the symlink - broken
                return SymlinkInfo {
                    original: path.to_path_buf(),
                    target: None,
                    chain,
                    is_broken: true,
                    is_circular: false,
                };
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
        assert_eq!(info.target, Some(file.clone()));
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
        assert_eq!(info.target, Some(target));
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
        assert!(info.target.is_none());
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
        assert!(info.target.is_none());
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
        assert_eq!(info.target, Some(target));
        assert_eq!(info.chain.len(), 4); // link3 -> link2 -> link1 -> target
    }
}
