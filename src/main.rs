mod analyzer;
mod cli;
mod output;
mod platform;
mod resolver;
mod symlink;
mod version;

use analyzer::analyze_path;
use cli::parse_args;
use output::{print_analysis, print_diff, print_explain, print_resolution, OutputFormat};
use resolver::{resolve_command, ResolveConfig};

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
            eprintln!("Error: No command specified. Use --analyze for PATH analysis.");
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

    // Handle explain mode
    if args.explain {
        print_explain(&result, format);
    } else {
        print_resolution(&result, format, use_color);
    }

    // Exit code based on whether command was found
    if result.resolved.is_none() {
        std::process::exit(1);
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
