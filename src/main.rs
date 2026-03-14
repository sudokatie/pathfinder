use pathfinder::analyzer::analyze_path;
use pathfinder::cli::parse_args;
use pathfinder::output::{
    print_analysis, print_diff, print_explain, print_resolution, OutputFormat,
};
use pathfinder::platform::{get_path_entries, is_path_empty};
use pathfinder::resolver::{resolve_command, ResolveConfig};

fn main() {
    let args = parse_args();

    // Determine output format
    let format = if args.json {
        OutputFormat::Json
    } else if args.plain {
        OutputFormat::Plain
    } else {
        OutputFormat::Human
    };

    // Determine color usage
    let use_color = !args.no_color && !args.plain && atty::is(atty::Stream::Stdout);

    // Check for empty PATH (before parsing, since empty string becomes ["."])
    if is_path_empty() {
        eprintln!("PATH environment variable is empty");
        std::process::exit(2);
    }

    let _path_entries = get_path_entries();

    // Handle analyze mode
    if args.analyze {
        let analysis = analyze_path();
        print_analysis(&analysis, format, use_color);

        // Exit with code 3 if critical issues found
        if analysis.has_errors() {
            std::process::exit(3);
        }
        return;
    }

    // Require a command for other modes
    let command = match &args.command {
        Some(cmd) => cmd,
        None => {
            eprintln!("Usage: pathfinder <command>");
            eprintln!();
            eprintln!("For more information, try '--help'.");
            std::process::exit(2);
        }
    };

    // Build resolve config
    let config = ResolveConfig {
        timeout_ms: args.timeout,
        skip_version: args.no_version,
    };

    // Handle diff mode
    if args.diff {
        let mut all_commands = vec![command.clone()];
        all_commands.extend(args.extra_commands.clone());

        if all_commands.len() < 2 {
            eprintln!("Error: --diff requires at least 2 commands");
            std::process::exit(2);
        }

        // Resolve all commands
        let results: Vec<_> = all_commands
            .iter()
            .map(|cmd| resolve_command(cmd, &config))
            .collect();

        // Print side-by-side comparison
        print_diff(&results, format, use_color);
        return;
    }

    // Standard resolution
    let result = resolve_command(command, &config);

    // Check if command was found
    if result.resolved.is_none() {
        // Per spec 7.2: error messages go to stderr (except JSON which goes to stdout)
        match format {
            OutputFormat::Json => {
                // JSON output goes to stdout even for not-found
                print!("{}", pathfinder::output::json::format_resolution(&result));
            }
            OutputFormat::Human | OutputFormat::Plain => {
                // Per spec 7.2: exact message format
                eprintln!("Command '{}' not found in PATH", command);
                // Add shell builtin note if applicable
                if result.is_builtin {
                    eprintln!(
                        "Note: '{}' is a shell builtin - it works in your shell but has no PATH executable",
                        command
                    );
                }
            }
        }
        std::process::exit(1);
    }

    // Handle explain mode (only for found commands)
    if args.explain {
        print_explain(&result, format);
    } else {
        print_resolution(&result, format, use_color);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_exit_codes_documented() {
        // Exit codes:
        // 0 - success
        // 1 - command not found
        // 2 - usage error
        // 3 - critical PATH issues
        assert!(true);
    }
}
