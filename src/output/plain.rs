//! Plain text output format.
//!
//! No colors, no Unicode. Suitable for piping.

use crate::analyzer::{IssueLevel, PathAnalysis};
use crate::resolver::ResolutionResult;

/// Format a resolution result as plain text.
pub fn format_resolution(result: &ResolutionResult) -> String {
    let mut output = String::new();
    
    match &result.resolved {
        Some(path) => {
            output.push_str(&format!("RESOLVED: {}\n\n", path.display()));
        }
        None => {
            output.push_str(&format!(
                "NOT FOUND: Command '{}' not found in PATH\n",
                result.command
            ));
            return output;
        }
    }
    
    output.push_str("All matches in PATH order:\n\n");
    
    for (i, m) in result.matches.iter().enumerate() {
        let marker = if m.is_selected { " <- SELECTED" } else { "" };
        
        output.push_str(&format!(
            "{}. {}{}\n",
            i + 1,
            m.path.display(),
            marker
        ));
        
        if let Some(version) = &m.version {
            output.push_str(&format!("   version: {}\n", version));
        }
        
        if let Some(symlink) = &m.symlink {
            if symlink.is_broken {
                output.push_str("   symlink: yes (broken)\n");
            } else if symlink.is_circular {
                output.push_str("   symlink: yes (circular)\n");
            } else if let Some(target) = &symlink.target {
                output.push_str(&format!("   symlink: -> {}\n", target.display()));
            }
        } else {
            output.push_str("   symlink: no\n");
        }
        
        output.push('\n');
    }
    
    output
}

/// Format a PATH analysis as plain text.
pub fn format_analysis(analysis: &PathAnalysis) -> String {
    let mut output = String::new();
    
    output.push_str("PATH Analysis:\n\n");
    output.push_str(&format!("Directories: {}\n", analysis.total_entries));
    output.push_str(&format!("Valid: {}\n", analysis.valid_dirs));
    
    if analysis.issues.is_empty() {
        output.push_str("\nNo issues found.\n");
        return output;
    }
    
    output.push_str(&format!("Issues found: {}\n\n", analysis.issues.len()));
    
    for (i, issue) in analysis.issues.iter().enumerate() {
        let level_str = match issue.level {
            IssueLevel::Warning => "Warning",
            IssueLevel::Error => "Error",
        };
        
        output.push_str(&format!(
            "{}. {} (position {})\n",
            i + 1,
            issue.path.display(),
            issue.position
        ));
        output.push_str(&format!("   {}: {}\n", level_str, issue.description));
        output.push_str(&format!("   Suggestion: {}\n\n", issue.suggestion));
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::CommandMatch;
    use std::path::PathBuf;

    fn mock_result() -> ResolutionResult {
        ResolutionResult {
            command: "test".to_string(),
            resolved: Some(PathBuf::from("/usr/bin/test")),
            matches: vec![CommandMatch {
                path: PathBuf::from("/usr/bin/test"),
                position: 0,
                path_dir: PathBuf::from("/usr/bin"),
                is_selected: true,
                version: None,
                symlink: None,
            }],
        }
    }

    #[test]
    fn test_format_resolution_plain() {
        let result = mock_result();
        let output = format_resolution(&result);
        assert!(output.contains("RESOLVED:"));
        assert!(!output.contains("\x1b[")); // No ANSI codes
    }

    #[test]
    fn test_format_resolution_not_found() {
        let result = ResolutionResult {
            command: "notfound".to_string(),
            resolved: None,
            matches: vec![],
        };
        let output = format_resolution(&result);
        assert!(output.contains("NOT FOUND"));
    }

    #[test]
    fn test_format_analysis_plain() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 5,
            issues: vec![],
        };
        let output = format_analysis(&analysis);
        assert!(output.contains("PATH Analysis:"));
        assert!(output.contains("No issues found"));
    }
}
