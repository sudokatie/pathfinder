//! JSON output format.
//!
//! Structured output for scripting and piping.

use crate::analyzer::{IssueLevel, PathAnalysis};
use crate::resolver::ResolutionResult;
use serde::Serialize;

/// JSON-serializable resolution result.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub command: String,
    pub resolved: Option<String>,
    pub matches: Vec<JsonMatch>,
    pub path_searched: Vec<String>,
    /// Whether the command name matches a shell builtin.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_builtin: bool,
}

/// JSON-serializable command match.
#[derive(Debug, Serialize)]
pub struct JsonMatch {
    pub path: String,
    pub selected: bool,
    pub version: Option<String>,
    pub symlink: Option<JsonSymlink>,
    pub executable: bool,
}

/// JSON-serializable symlink info.
#[derive(Debug, Serialize)]
pub struct JsonSymlink {
    /// The raw symlink target (as stored, may be relative).
    pub target: Option<String>,
    /// The fully resolved absolute path.
    pub resolved: Option<String>,
    /// Whether the symlink is broken.
    pub broken: bool,
    /// Whether this is a Windows .lnk shortcut file.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_lnk: bool,
    /// Whether this is a Windows junction point.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub is_junction: bool,
}

/// JSON-serializable PATH analysis.
#[derive(Debug, Serialize)]
pub struct JsonAnalysis {
    pub total_entries: usize,
    pub valid_dirs: usize,
    pub issues: Vec<JsonIssue>,
}

/// JSON-serializable PATH issue.
#[derive(Debug, Serialize)]
pub struct JsonIssue {
    pub path: String,
    pub position: usize,
    pub level: String,
    pub description: String,
    pub suggestion: String,
}

/// Format a resolution result as JSON.
pub fn format_resolution(result: &ResolutionResult) -> String {
    let json_output = JsonOutput {
        command: result.command.clone(),
        resolved: result.resolved.as_ref().map(|p| p.display().to_string()),
        matches: result
            .matches
            .iter()
            .map(|m| JsonMatch {
                path: m.path.display().to_string(),
                selected: m.is_selected,
                version: m.version.clone(),
                symlink: m.symlink.as_ref().map(|s| JsonSymlink {
                    target: s.raw_target.as_ref().map(|p| p.display().to_string()),
                    resolved: s.resolved.as_ref().map(|p| p.display().to_string()),
                    broken: s.is_broken,
                    is_lnk: s.is_lnk,
                    is_junction: s.is_junction,
                }),
                executable: m.executable,
            })
            .collect(),
        path_searched: result
            .path_searched
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        is_builtin: result.is_builtin,
    };

    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
}

/// Format a PATH analysis as JSON.
pub fn format_analysis(analysis: &PathAnalysis) -> String {
    let json_analysis = JsonAnalysis {
        total_entries: analysis.total_entries,
        valid_dirs: analysis.valid_dirs,
        issues: analysis
            .issues
            .iter()
            .map(|i| JsonIssue {
                path: i.path.display().to_string(),
                position: i.position,
                level: match i.level {
                    IssueLevel::Warning => "warning".to_string(),
                    IssueLevel::Error => "error".to_string(),
                },
                description: i.description.clone(),
                suggestion: i.suggestion.clone(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&json_analysis).unwrap_or_else(|_| "{}".to_string())
}

/// JSON-serializable diff comparison.
#[derive(Debug, Serialize)]
pub struct JsonDiff {
    pub commands: Vec<JsonDiffEntry>,
    pub same_source: bool,
    pub warning: Option<String>,
}

/// JSON-serializable diff entry for a single command.
#[derive(Debug, Serialize)]
pub struct JsonDiffEntry {
    pub command: String,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub source_dir: Option<String>,
}

/// Format a diff comparison as JSON.
pub fn format_diff(results: &[ResolutionResult]) -> String {
    let entries: Vec<JsonDiffEntry> = results
        .iter()
        .map(|r| JsonDiffEntry {
            command: r.command.clone(),
            found: r.resolved.is_some(),
            path: r.resolved.as_ref().map(|p| p.display().to_string()),
            version: r.matches.first().and_then(|m| m.version.clone()),
            source_dir: r.matches.first().map(|m| m.path_dir.display().to_string()),
        })
        .collect();

    // Check if all commands come from the same directory
    let source_dirs: std::collections::HashSet<_> = entries
        .iter()
        .filter_map(|e| e.source_dir.as_ref())
        .collect();

    let same_source = source_dirs.len() <= 1;
    let all_found = entries.iter().all(|e| e.found);

    let warning = if !same_source && all_found {
        Some("Commands come from different directories - possible version mismatch".to_string())
    } else {
        None
    };

    let diff = JsonDiff {
        commands: entries,
        same_source,
        warning,
    };

    serde_json::to_string_pretty(&diff).unwrap_or_else(|_| "{}".to_string())
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
            path_searched: vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")],
            is_builtin: false,
        }
    }

    #[test]
    fn test_format_resolution_valid_json() {
        let result = mock_result();
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["command"], "test");
    }

    #[test]
    fn test_format_resolution_has_matches() {
        let result = mock_result();
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["matches"].is_array());
        assert_eq!(parsed["matches"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_format_resolution_has_selected() {
        let result = mock_result();
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["matches"][0]["selected"], true);
    }

    #[test]
    fn test_format_resolution_has_executable() {
        let result = mock_result();
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["matches"][0]["executable"], true);
    }

    #[test]
    fn test_format_resolution_has_path_searched() {
        let result = mock_result();
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["path_searched"].is_array());
        assert_eq!(parsed["path_searched"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_format_analysis_valid_json() {
        let analysis = PathAnalysis {
            total_entries: 5,
            valid_dirs: 5,
            issues: vec![],
        };
        let output = format_analysis(&analysis);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["total_entries"], 5);
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
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["resolved"].is_null());
    }

    #[test]
    fn test_format_diff_valid_json() {
        let result1 = mock_result();
        let mut result2 = mock_result();
        result2.command = "test2".to_string();

        let results = vec![result1, result2];
        let output = format_diff(&results);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["commands"].is_array());
        assert_eq!(parsed["commands"].as_array().unwrap().len(), 2);
        assert!(parsed["same_source"].is_boolean());
    }

    #[test]
    fn test_format_diff_same_source() {
        let result1 = mock_result();
        let mut result2 = mock_result();
        result2.command = "test2".to_string();

        let results = vec![result1, result2];
        let output = format_diff(&results);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["same_source"], true);
        assert!(parsed["warning"].is_null());
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
        let output = format_diff(&results);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["same_source"], false);
        assert!(parsed["warning"].is_string());
    }
}
