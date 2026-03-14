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
                "{} {}\n",
                "RESOLVED:".green().bold(),
                path.display()
            ));
            // Warn if this is also a shell builtin
            if result.is_builtin {
                output.push_str(&format!(
                    "{} '{}' is also a shell builtin - the builtin may take precedence\n",
                    "NOTE:".yellow(),
                    result.command
                ));
            }
            output.push('\n');
        }
        None => {
            output.push_str(&format!(
                "{} Command '{}' not found in PATH\n",
                "NOT FOUND:".red().bold(),
                result.command
            ));
            // Mention if it might be a builtin
            if result.is_builtin {
                output.push_str(&format!(
                    "{} '{}' is a shell builtin - it works in your shell but has no PATH executable\n",
                    "NOTE:".yellow(),
                    result.command
                ));
            }
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

        output.push_str(&format!("{}. {}  {}\n", i + 1, m.path.display(), marker));

        // Show version, or special messages for permission denied / broken symlinks / unknown
        if !m.executable {
            output.push_str(&format!("   version: {}\n", "(permission denied)".red()));
        } else if let Some(symlink) = &m.symlink {
            if symlink.is_broken {
                output.push_str(&format!("   version: {}\n", "(broken symlink)".red()));
            } else if let Some(version) = &m.version {
                output.push_str(&format!("   version: {}\n", version.cyan()));
            } else {
                output.push_str(&format!("   version: {}\n", "(version unknown)".dimmed()));
            }
        } else if let Some(version) = &m.version {
            output.push_str(&format!("   version: {}\n", version.cyan()));
        } else {
            output.push_str(&format!("   version: {}\n", "(version unknown)".dimmed()));
        }

        // Show symlink info (including Windows .lnk and junction points)
        if let Some(symlink) = &m.symlink {
            // Determine the type label
            let type_label = if symlink.is_lnk {
                "shortcut"
            } else if symlink.is_junction {
                "junction"
            } else {
                "symlink"
            };

            if symlink.is_broken {
                // Show broken symlink with DEAD marker
                if let Some(raw) = &symlink.raw_target {
                    output.push_str(&format!(
                        "   {}: {} {} {}\n",
                        type_label,
                        "->".dimmed(),
                        raw.display(),
                        "(DEAD)".red()
                    ));
                } else {
                    output.push_str(&format!(
                        "   {}: {} {}\n",
                        type_label,
                        "yes".red(),
                        "(broken)"
                    ));
                }
            } else if symlink.is_circular {
                if let Some(raw) = &symlink.raw_target {
                    output.push_str(&format!(
                        "   {}: {} {} {}\n",
                        type_label,
                        "->".dimmed(),
                        raw.display(),
                        "(CIRCULAR)".red()
                    ));
                } else {
                    output.push_str(&format!(
                        "   {}: {} {}\n",
                        type_label,
                        "yes".red(),
                        "(circular)"
                    ));
                }
            } else if let Some(raw) = &symlink.raw_target {
                output.push_str(&format!(
                    "   {}: {} {}\n",
                    type_label,
                    "->".dimmed(),
                    raw.display()
                ));
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

/// Format a side-by-side diff comparison of multiple commands.
pub fn format_diff(results: &[ResolutionResult], use_color: bool) -> String {
    colored::control::set_override(use_color);

    let mut output = String::new();

    output.push_str(&format!("{}\n\n", "Command Comparison:".bold()));

    // Calculate column width based on longest command name
    let max_cmd_len = results.iter().map(|r| r.command.len()).max().unwrap_or(10);
    let col_width = max_cmd_len.max(10);

    // Header row
    output.push_str(&format!("{:width$}", "", width = 12));
    for result in results {
        output.push_str(&format!(
            "  {:^width$}",
            result.command.bold(),
            width = col_width
        ));
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
            "Found".green().to_string()
        } else {
            "Not Found".red().to_string()
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
        output.push_str(&format!("  {:^width$}", version.cyan(), width = col_width));
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

    // Check for mismatches and highlight
    let unique_dirs: std::collections::HashSet<_> =
        source_dirs.iter().filter_map(|d| d.as_ref()).collect();

    if unique_dirs.len() > 1 && results.iter().all(|r| r.resolved.is_some()) {
        output.push_str(&format!(
            "{} Commands come from different directories!\n",
            "Warning:".yellow().bold()
        ));
        output.push_str("This may indicate version mismatches or mixed installations.\n");
    } else if unique_dirs.len() == 1 && results.iter().all(|r| r.resolved.is_some()) {
        output.push_str(&format!(
            "{} All commands from same directory.\n",
            "OK:".green()
        ));
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
        // Try to show the filename at least
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
                version: Some("1.0.0".to_string()),
                symlink: None,
                executable: true,
            }],
            path_searched: vec![PathBuf::from("/usr/bin")],
            is_builtin: false,
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
            path_searched: vec![PathBuf::from("/usr/bin")],
            is_builtin: false,
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
            path_searched: vec![],
            is_builtin: false,
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

    #[test]
    fn test_format_diff_same_source() {
        let result1 = mock_result();
        let mut result2 = mock_result();
        result2.command = "test2".to_string();

        let results = vec![result1, result2];
        let output = format_diff(&results, false);

        assert!(output.contains("Command Comparison"));
        assert!(output.contains("test"));
        assert!(output.contains("test2"));
        assert!(output.contains("All commands from same directory"));
    }

    #[test]
    fn test_format_diff_different_sources() {
        let result1 = ResolutionResult {
            command: "cmd1".to_string(),
            resolved: Some(PathBuf::from("/usr/bin/cmd1")),
            matches: vec![CommandMatch {
                path: PathBuf::from("/usr/bin/cmd1"),
                position: 0,
                path_dir: PathBuf::from("/usr/bin"),
                is_selected: true,
                version: None,
                symlink: None,
                executable: true,
            }],
            path_searched: vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")],
            is_builtin: false,
        };
        let result2 = ResolutionResult {
            command: "cmd2".to_string(),
            resolved: Some(PathBuf::from("/usr/local/bin/cmd2")),
            matches: vec![CommandMatch {
                path: PathBuf::from("/usr/local/bin/cmd2"),
                position: 1,
                path_dir: PathBuf::from("/usr/local/bin"),
                is_selected: true,
                version: None,
                symlink: None,
                executable: true,
            }],
            path_searched: vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")],
            is_builtin: false,
        };

        let results = vec![result1, result2];
        let output = format_diff(&results, false);

        assert!(output.contains("Warning"));
        assert!(output.contains("different directories"));
    }
}
