//! JSON output format.
//!
//! Structured output for scripting and piping.

use crate::analyzer::{IssueLevel, PathAnalysis, PathIssue};
use crate::resolver::{CommandMatch, ResolutionResult};
use serde::Serialize;
use std::path::PathBuf;

/// JSON-serializable resolution result.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub command: String,
    pub resolved: Option<String>,
    pub matches: Vec<JsonMatch>,
}

/// JSON-serializable command match.
#[derive(Debug, Serialize)]
pub struct JsonMatch {
    pub path: String,
    pub position: usize,
    pub path_dir: String,
    pub is_selected: bool,
    pub version: Option<String>,
    pub symlink: Option<JsonSymlink>,
}

/// JSON-serializable symlink info.
#[derive(Debug, Serialize)]
pub struct JsonSymlink {
    pub target: Option<String>,
    pub chain: Vec<String>,
    pub is_broken: bool,
    pub is_circular: bool,
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
        matches: result.matches.iter().map(|m| {
            JsonMatch {
                path: m.path.display().to_string(),
                position: m.position,
                path_dir: m.path_dir.display().to_string(),
                is_selected: m.is_selected,
                version: m.version.clone(),
                symlink: m.symlink.as_ref().map(|s| JsonSymlink {
                    target: s.target.as_ref().map(|p| p.display().to_string()),
                    chain: s.chain.iter().map(|p| p.display().to_string()).collect(),
                    is_broken: s.is_broken,
                    is_circular: s.is_circular,
                }),
            }
        }).collect(),
    };
    
    serde_json::to_string_pretty(&json_output).unwrap_or_else(|_| "{}".to_string())
}

/// Format a PATH analysis as JSON.
pub fn format_analysis(analysis: &PathAnalysis) -> String {
    let json_analysis = JsonAnalysis {
        total_entries: analysis.total_entries,
        valid_dirs: analysis.valid_dirs,
        issues: analysis.issues.iter().map(|i| {
            JsonIssue {
                path: i.path.display().to_string(),
                position: i.position,
                level: match i.level {
                    IssueLevel::Warning => "warning".to_string(),
                    IssueLevel::Error => "error".to_string(),
                },
                description: i.description.clone(),
                suggestion: i.suggestion.clone(),
            }
        }).collect(),
    };
    
    serde_json::to_string_pretty(&json_analysis).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };
        let output = format_resolution(&result);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["resolved"].is_null());
    }
}
