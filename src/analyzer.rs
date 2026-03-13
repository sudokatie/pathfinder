//! PATH analysis.
//!
//! Detects issues with the PATH environment variable.

use crate::platform::get_path_entries;
use std::collections::HashMap;
use std::path::PathBuf;

/// Severity of a PATH issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueLevel {
    Warning,
    Error,
}

/// A single issue found in PATH.
#[derive(Debug, Clone)]
pub struct PathIssue {
    /// The PATH entry with the issue.
    pub path: PathBuf,
    /// Position in PATH (0-indexed).
    pub position: usize,
    /// Severity level.
    pub level: IssueLevel,
    /// Description of the issue.
    pub description: String,
    /// Suggested fix.
    pub suggestion: String,
}

/// Result of PATH analysis.
#[derive(Debug, Clone)]
pub struct PathAnalysis {
    /// Total number of PATH entries.
    pub total_entries: usize,
    /// Number of valid directories.
    pub valid_dirs: usize,
    /// Issues found.
    pub issues: Vec<PathIssue>,
}

impl PathAnalysis {
    /// Check if there are any errors (not just warnings).
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.level == IssueLevel::Error)
    }
    
    /// Check if there are any issues at all.
    #[allow(dead_code)]
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

/// Analyze the PATH for issues.
pub fn analyze_path() -> PathAnalysis {
    let entries = get_path_entries();
    let mut issues = Vec::new();
    let mut valid_dirs = 0;
    let mut seen: HashMap<PathBuf, usize> = HashMap::new();
    
    for (position, path) in entries.iter().enumerate() {
        // Check for empty path (shouldn't happen after filtering, but just in case)
        if path.as_os_str().is_empty() {
            issues.push(PathIssue {
                path: path.clone(),
                position,
                level: IssueLevel::Warning,
                description: "Empty PATH entry".to_string(),
                suggestion: "Remove empty entries from PATH".to_string(),
            });
            continue;
        }
        
        // Check if directory exists
        if !path.exists() {
            issues.push(PathIssue {
                path: path.clone(),
                position,
                level: IssueLevel::Warning,
                description: "Directory does not exist".to_string(),
                suggestion: "Remove from PATH or create the directory".to_string(),
            });
            continue;
        }
        
        // Check if it's actually a directory
        if !path.is_dir() {
            issues.push(PathIssue {
                path: path.clone(),
                position,
                level: IssueLevel::Error,
                description: "Not a directory".to_string(),
                suggestion: "Remove from PATH - only directories are valid".to_string(),
            });
            continue;
        }
        
        // Check for read permission
        if std::fs::read_dir(path).is_err() {
            issues.push(PathIssue {
                path: path.clone(),
                position,
                level: IssueLevel::Warning,
                description: "Cannot read directory (permission denied)".to_string(),
                suggestion: "Fix permissions or remove from PATH".to_string(),
            });
            continue;
        }
        
        // Check for duplicates
        if let Some(&first_pos) = seen.get(path) {
            issues.push(PathIssue {
                path: path.clone(),
                position,
                level: IssueLevel::Warning,
                description: format!("Duplicate entry (first at position {})", first_pos),
                suggestion: "Remove duplicate entries".to_string(),
            });
            // Don't count as valid since it's a duplicate
            continue;
        }
        
        seen.insert(path.clone(), position);
        valid_dirs += 1;
    }
    
    PathAnalysis {
        total_entries: entries.len(),
        valid_dirs,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_path_runs() {
        let analysis = analyze_path();
        assert!(analysis.total_entries > 0, "PATH should have entries");
    }

    #[test]
    fn test_analyze_path_valid_dirs() {
        let analysis = analyze_path();
        assert!(analysis.valid_dirs > 0, "Should have some valid dirs");
    }

    #[test]
    fn test_has_errors_false_for_warnings() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 4,
            issues: vec![PathIssue {
                path: PathBuf::from("/nonexistent"),
                position: 3,
                level: IssueLevel::Warning,
                description: "test".to_string(),
                suggestion: "test".to_string(),
            }],
        };
        assert!(!analysis.has_errors());
        assert!(analysis.has_issues());
    }

    #[test]
    fn test_has_errors_true_for_errors() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 4,
            issues: vec![PathIssue {
                path: PathBuf::from("/notadir"),
                position: 3,
                level: IssueLevel::Error,
                description: "test".to_string(),
                suggestion: "test".to_string(),
            }],
        };
        assert!(analysis.has_errors());
    }

    #[test]
    fn test_has_issues_false_when_empty() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 5,
            issues: vec![],
        };
        assert!(!analysis.has_issues());
        assert!(!analysis.has_errors());
    }

    #[test]
    fn test_issue_level_equality() {
        assert_eq!(IssueLevel::Warning, IssueLevel::Warning);
        assert_eq!(IssueLevel::Error, IssueLevel::Error);
        assert_ne!(IssueLevel::Warning, IssueLevel::Error);
    }
}
