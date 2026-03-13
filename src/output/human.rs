//! Human-readable output format.
//!
//! Colored output with Unicode decorations.

use crate::analyzer::{IssueLevel, PathAnalysis};
use crate::resolver::ResolutionResult;
use colored::Colorize;

/// Format a resolution result for human display.
pub fn format_resolution(result: &ResolutionResult, use_color: bool) -> String {
    colored::control::set_override(use_color);
    
    let mut output = String::new();
    
    match &result.resolved {
        Some(path) => {
            output.push_str(&format!(
                "{} {}\n\n",
                "RESOLVED:".green().bold(),
                path.display()
            ));
        }
        None => {
            output.push_str(&format!(
                "{} Command '{}' not found in PATH\n",
                "NOT FOUND:".red().bold(),
                result.command
            ));
            return output;
        }
    }
    
    output.push_str("All matches in PATH order:\n\n");
    
    for (i, m) in result.matches.iter().enumerate() {
        let marker = if m.is_selected {
            "<- SELECTED".green().to_string()
        } else {
            String::new()
        };
        
        output.push_str(&format!(
            "{}. {}  {}\n",
            i + 1,
            m.path.display(),
            marker
        ));
        
        if let Some(version) = &m.version {
            output.push_str(&format!("   version: {}\n", version.cyan()));
        }
        
        if let Some(symlink) = &m.symlink {
            if symlink.is_broken {
                output.push_str(&format!("   symlink: {} (broken)\n", "yes".red()));
            } else if symlink.is_circular {
                output.push_str(&format!("   symlink: {} (circular)\n", "yes".red()));
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

/// Format a PATH analysis for human display.
pub fn format_analysis(analysis: &PathAnalysis, use_color: bool) -> String {
    colored::control::set_override(use_color);
    
    let mut output = String::new();
    
    output.push_str(&format!("{}\n\n", "PATH Analysis:".bold()));
    output.push_str(&format!("Directories: {}\n", analysis.total_entries));
    output.push_str(&format!("Valid: {}\n", analysis.valid_dirs));
    
    if analysis.issues.is_empty() {
        output.push_str(&format!("\n{}\n", "No issues found.".green()));
        return output;
    }
    
    output.push_str(&format!("Issues found: {}\n\n", analysis.issues.len()));
    
    for (i, issue) in analysis.issues.iter().enumerate() {
        let level_str = match issue.level {
            IssueLevel::Warning => "Warning".yellow().to_string(),
            IssueLevel::Error => "Error".red().to_string(),
        };
        
        output.push_str(&format!(
            "{}. {} (position {})\n",
            i + 1,
            issue.path.display(),
            issue.position
        ));
        output.push_str(&format!("   {}: {}\n", level_str, issue.description));
        output.push_str(&format!("   Suggestion: {}\n\n", issue.suggestion.cyan()));
    }
    
    output
}

/// Format an explanation of command resolution.
pub fn format_explain(result: &ResolutionResult) -> String {
    let mut output = String::new();
    
    match &result.resolved {
        None => {
            output.push_str(&format!(
                "'{}' was not found in any PATH directory.\n",
                result.command
            ));
            return output;
        }
        Some(resolved) => {
            let first_match = &result.matches[0];
            
            output.push_str(&format!(
                "'{}' resolves to {} because\n",
                result.command,
                resolved.display()
            ));
            output.push_str(&format!(
                "{} appears at position {} in your PATH",
                first_match.path_dir.display(),
                first_match.position + 1
            ));
            
            if result.matches.len() > 1 {
                output.push_str(", before:\n");
                for m in result.matches.iter().skip(1) {
                    output.push_str(&format!(
                        "  - {} (position {})\n",
                        m.path_dir.display(),
                        m.position + 1
                    ));
                }
            } else {
                output.push_str(".\n");
            }
        }
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
                version: Some("1.0.0".to_string()),
                symlink: None,
            }],
        }
    }

    #[test]
    fn test_format_resolution_found() {
        let result = mock_result();
        let output = format_resolution(&result, false);
        assert!(output.contains("RESOLVED"));
        assert!(output.contains("/usr/bin/test"));
    }

    #[test]
    fn test_format_resolution_not_found() {
        let result = ResolutionResult {
            command: "notfound".to_string(),
            resolved: None,
            matches: vec![],
        };
        let output = format_resolution(&result, false);
        assert!(output.contains("NOT FOUND"));
    }

    #[test]
    fn test_format_analysis_no_issues() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 5,
            issues: vec![],
        };
        let output = format_analysis(&analysis, false);
        assert!(output.contains("No issues found"));
    }

    #[test]
    fn test_format_explain() {
        let result = mock_result();
        let output = format_explain(&result);
        assert!(output.contains("'test' resolves to"));
        assert!(output.contains("position 1"));
    }

    #[test]
    fn test_format_explain_not_found() {
        let result = ResolutionResult {
            command: "notfound".to_string(),
            resolved: None,
            matches: vec![],
        };
        let output = format_explain(&result);
        assert!(output.contains("was not found"));
    }

    #[test]
    fn test_format_resolution_with_version() {
        let result = mock_result();
        let output = format_resolution(&result, false);
        assert!(output.contains("version: 1.0.0"));
    }
}
