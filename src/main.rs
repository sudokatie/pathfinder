mod cli;
mod platform;
mod resolver;
mod symlink;
mod version;

use cli::parse_args;

fn main() {
    let args = parse_args();
    
    if args.analyze {
        println!("PATH Analysis mode (not yet implemented)");
        return;
    }
    
    match &args.command {
        Some(cmd) => {
            println!("Resolving: {}", cmd);
        }
        None => {
            eprintln!("Error: No command specified. Use --analyze for PATH analysis.");
            std::process::exit(2);
        }
    }
}
