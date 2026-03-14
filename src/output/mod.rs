//! Output formatting.
//!
//! Formats resolution and analysis results for display.

pub mod human;
pub mod json;
pub mod plain;

use crate::analyzer::PathAnalysis;
use crate::resolver::ResolutionResult;

/// Output format selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Plain,
}

/// Print a resolution result in the specified format.
pub fn print_resolution(result: &ResolutionResult, format: OutputFormat, use_color: bool) {
    match format {
        OutputFormat::Human => print!("{}", human::format_resolution(result, use_color)),
        OutputFormat::Json => print!("{}", json::format_resolution(result)),
        OutputFormat::Plain => print!("{}", plain::format_resolution(result)),
    }
}

/// Print a PATH analysis in the specified format.
pub fn print_analysis(analysis: &PathAnalysis, format: OutputFormat, use_color: bool) {
    match format {
        OutputFormat::Human => print!("{}", human::format_analysis(analysis, use_color)),
        OutputFormat::Json => print!("{}", json::format_analysis(analysis)),
        OutputFormat::Plain => print!("{}", plain::format_analysis(analysis)),
    }
}

/// Print an explanation in the specified format.
pub fn print_explain(result: &ResolutionResult, format: OutputFormat) {
    match format {
        OutputFormat::Human | OutputFormat::Plain => {
            print!("{}", human::format_explain(result));
        }
        OutputFormat::Json => {
            print!("{}", json::format_resolution(result));
        }
    }
}

/// Print a diff comparison of multiple commands.
pub fn print_diff(results: &[ResolutionResult], format: OutputFormat, use_color: bool) {
    match format {
        OutputFormat::Human => print!("{}", human::format_diff(results, use_color)),
        OutputFormat::Json => print!("{}", json::format_diff(results)),
        OutputFormat::Plain => print!("{}", plain::format_diff(results)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_equality() {
        assert_eq!(OutputFormat::Human, OutputFormat::Human);
        assert_eq!(OutputFormat::Json, OutputFormat::Json);
        assert_ne!(OutputFormat::Human, OutputFormat::Json);
    }

    #[test]
    fn test_output_format_copy() {
        let format = OutputFormat::Human;
        let copy = format;
        assert_eq!(format, copy);
    }

    #[test]
    fn test_output_format_debug() {
        let format = OutputFormat::Json;
        let debug = format!("{:?}", format);
        assert!(debug.contains("Json"));
    }
}
