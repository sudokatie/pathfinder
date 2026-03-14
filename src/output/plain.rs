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

        output.push_str(&format!("{}. {}{}\n", i + 1, m.path.display(), marker));

        // Show version, or special messages for permission denied / broken symlinks / unknown
        if !m.executable {
            output.push_str("   version: (permission denied)\n");
        } else if let Some(symlink) = &m.symlink {
            if symlink.is_broken {
                output.push_str("   version: (broken symlink)\n");
            } else if let Some(version) = &m.version {
                output.push_str(&format!("   version: {}\n", version));
            } else {
                output.push_str("   version: (version unknown)\n");
            }
        } else if let Some(version) = &m.version {
            output.push_str(&format!("   version: {}\n", version));
        } else {
            output.push_str("   version: (version unknown)\n");
        }

        // Show symlink info
        if let Some(symlink) = &m.symlink {
            if symlink.is_broken {
                if let Some(raw) = &symlink.raw_target {
                    output.push_str(&format!("   symlink: -> {} (DEAD)\n", raw.display()));
                } else {
                    output.push_str("   symlink: yes (broken)\n");
                }
            } else if symlink.is_circular {
                if let Some(raw) = &symlink.raw_target {
                    output.push_str(&format!("   symlink: -> {} (CIRCULAR)\n", raw.display()));
                } else {
                    output.push_str("   symlink: yes (circular)\n");
                }
            } else if let Some(raw) = &symlink.raw_target {
                output.push_str(&format!("   symlink: -> {}\n", raw.display()));
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

/// Format a diff comparison as plain text.
pub fn format_diff(results: &[ResolutionResult]) -> String {
    let mut output = String::new();

    output.push_str("Command Comparison:\n\n");

    // Calculate column width based on longest command name
    let max_cmd_len = results.iter().map(|r| r.command.len()).max().unwrap_or(10);
    let col_width = max_cmd_len.max(10);

    // Header row
    output.push_str(&format!("{:width$}", "", width = 12));
    for result in results {
        output.push_str(&format!("  {:^width$}", result.command, width = col_width));
    }
    output.push('\n');
    output.push_str(&format!("{:width$}", "", width = 12));
    for _ in results {
        output.push_str(&format!("  {:->width$}", "", width = col_width));
    }
    output.push('\n');

    // Status row
    output.push_str(&format!("{:width$}", "Status:", width = 12));
    for result in results {
        let status = if result.resolved.is_some() {
            "Found"
        } else {
            "Not Found"
        };
        output.push_str(&format!("  {:^width$}", status, width = col_width));
    }
    output.push('\n');

    // Path row
    output.push_str(&format!("{:width$}", "Path:", width = 12));
    for result in results {
        let path = result
            .resolved
            .as_ref()
            .map(|p| truncate_path(p, col_width))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!("  {:^width$}", path, width = col_width));
    }
    output.push('\n');

    // Version row
    output.push_str(&format!("{:width$}", "Version:", width = 12));
    for result in results {
        let version = result
            .matches
            .first()
            .and_then(|m| m.version.as_ref())
            .map(|v| truncate_str(v, col_width))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!("  {:^width$}", version, width = col_width));
    }
    output.push('\n');

    // Source directory row
    output.push_str(&format!("{:width$}", "Source Dir:", width = 12));
    let source_dirs: Vec<Option<String>> = results
        .iter()
        .map(|r| r.matches.first().map(|m| m.path_dir.display().to_string()))
        .collect();

    for dir in &source_dirs {
        let dir_str = dir
            .as_ref()
            .map(|d| truncate_path_str(d, col_width))
            .unwrap_or_else(|| "-".to_string());
        output.push_str(&format!("  {:^width$}", dir_str, width = col_width));
    }
    output.push_str("\n\n");

    // Check for mismatches
    let unique_dirs: std::collections::HashSet<_> =
        source_dirs.iter().filter_map(|d| d.as_ref()).collect();

    if unique_dirs.len() > 1 && results.iter().all(|r| r.resolved.is_some()) {
        output.push_str("Warning: Commands come from different directories!\n");
        output.push_str("This may indicate version mismatches or mixed installations.\n");
    } else if unique_dirs.len() == 1 && results.iter().all(|r| r.resolved.is_some()) {
        output.push_str("OK: All commands from same directory.\n");
    }

    output
}

/// Truncate a path for display.
fn truncate_path(path: &std::path::Path, max_len: usize) -> String {
    let s = path.display().to_string();
    truncate_path_str(&s, max_len)
}

/// Truncate a path string for display.
fn truncate_path_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        if let Some(pos) = s.rfind('/') {
            let filename = &s[pos + 1..];
            if filename.len() < max_len - 3 {
                return format!("...{}", &s[s.len() - (max_len - 3)..]);
            }
        }
        format!("{}...", &s[..max_len - 3])
    }
}

/// Truncate a string for display.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
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
                executable: true,
            }],
            path_searched: vec![PathBuf::from("/usr/bin")],
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
            path_searched: vec![PathBuf::from("/usr/bin")],
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

    #[test]
    fn test_format_diff_plain() {
        let result1 = mock_result();
        let mut result2 = mock_result();
        result2.command = "test2".to_string();

        let results = vec![result1, result2];
        let output = format_diff(&results);

        assert!(output.contains("Command Comparison"));
        assert!(output.contains("test"));
        assert!(output.contains("test2"));
        assert!(!output.contains("\x1b[")); // No ANSI codes
    }

    #[test]
    fn test_format_diff_same_source_plain() {
        let result1 = mock_result();
        let mut result2 = mock_result();
        result2.command = "test2".to_string();

        let results = vec![result1, result2];
        let output = format_diff(&results);

        assert!(output.contains("All commands from same directory"));
    }
}
