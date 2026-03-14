//! pathfinder - Debug command resolution and PATH issues.
//!
//! This library provides programmatic access to pathfinder's functionality.
//!
//! # Example
//!
//! ```no_run
//! use pathfinder::{resolve_command, ResolveConfig};
//!
//! let config = ResolveConfig::default();
//! let result = resolve_command("node", &config);
//!
//! if let Some(path) = &result.resolved {
//!     println!("node resolves to: {}", path.display());
//! }
//! ```

pub mod analyzer;
pub mod cli;
pub mod output;
pub mod platform;
pub mod resolver;
pub mod symlink;
pub mod version;

// Re-export main types for convenience
pub use analyzer::{analyze_path, IssueLevel, PathAnalysis, PathIssue};
pub use cli::{parse_args, Args};
pub use output::OutputFormat;
pub use platform::is_path_empty;
pub use resolver::{
    is_shell_builtin, resolve_command, CommandMatch, ResolutionResult, ResolveConfig,
};
pub use symlink::{is_symlink, resolve_symlink, SymlinkInfo};
pub use version::detect_version;
