//! CLI argument parsing.

use clap::Parser;

/// Debug command resolution and PATH issues.
#[derive(Parser, Debug, Clone)]
#[command(name = "pathfinder")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Command to resolve
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,

    /// Additional commands for diff mode
    #[arg(value_name = "COMMANDS")]
    pub extra_commands: Vec<String>,

    /// Output as JSON
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Output without colors/Unicode
    #[arg(short = 'p', long)]
    pub plain: bool,

    /// Analyze PATH for issues (no command needed)
    #[arg(short = 'a', long)]
    pub analyze: bool,

    /// Explain resolution in plain English
    #[arg(short = 'e', long)]
    pub explain: bool,

    /// Compare multiple commands
    #[arg(short = 'd', long)]
    pub diff: bool,

    /// Version detection timeout in milliseconds
    #[arg(short = 't', long, default_value = "2000")]
    pub timeout: u64,

    /// Skip version detection
    #[arg(long)]
    pub no_version: bool,

    /// Disable colors
    #[arg(long)]
    pub no_color: bool,
}

/// Parse command line arguments.
pub fn parse_args() -> Args {
    Args::parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Args {
        Args::try_parse_from(args).unwrap()
    }

    #[test]
    fn test_basic_command() {
        let args = parse(&["pathfinder", "node"]);
        assert_eq!(args.command, Some("node".to_string()));
        assert!(!args.json);
        assert!(!args.plain);
    }

    #[test]
    fn test_json_flag() {
        let args = parse(&["pathfinder", "node", "--json"]);
        assert!(args.json);
    }

    #[test]
    fn test_json_short_flag() {
        let args = parse(&["pathfinder", "node", "-j"]);
        assert!(args.json);
    }

    #[test]
    fn test_plain_flag() {
        let args = parse(&["pathfinder", "node", "--plain"]);
        assert!(args.plain);
    }

    #[test]
    fn test_analyze_flag() {
        let args = parse(&["pathfinder", "--analyze"]);
        assert!(args.analyze);
        assert!(args.command.is_none());
    }

    #[test]
    fn test_explain_flag() {
        let args = parse(&["pathfinder", "node", "--explain"]);
        assert!(args.explain);
    }

    #[test]
    fn test_timeout_flag() {
        let args = parse(&["pathfinder", "node", "--timeout", "5000"]);
        assert_eq!(args.timeout, 5000);
    }

    #[test]
    fn test_no_version_flag() {
        let args = parse(&["pathfinder", "node", "--no-version"]);
        assert!(args.no_version);
    }
}
