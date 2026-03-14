//! Core command resolution.
//!
//! Searches PATH entries to find all matches for a command.

use crate::platform::{find_command_in_dir, get_path_entries};
use crate::symlink::{is_symlink, resolve_symlink, SymlinkInfo};
use crate::version::detect_version;
use std::path::PathBuf;

/// Common shell builtins across bash, zsh, and other shells.
/// When a command matches one of these, we warn that the shell builtin
/// may take precedence over the PATH executable.
const SHELL_BUILTINS: &[&str] = &[
    // POSIX builtins
    ".",
    ":",
    "alias",
    "bg",
    "break",
    "cd",
    "command",
    "continue",
    "eval",
    "exec",
    "exit",
    "export",
    "false",
    "fg",
    "getopts",
    "hash",
    "jobs",
    "kill",
    "pwd",
    "read",
    "readonly",
    "return",
    "set",
    "shift",
    "test",
    "times",
    "trap",
    "true",
    "type",
    "ulimit",
    "umask",
    "unalias",
    "unset",
    "wait",
    // Bash-specific
    "builtin",
    "caller",
    "compgen",
    "complete",
    "compopt",
    "declare",
    "dirs",
    "disown",
    "echo",
    "enable",
    "help",
    "history",
    "let",
    "local",
    "logout",
    "mapfile",
    "popd",
    "printf",
    "pushd",
    "readarray",
    "shopt",
    "source",
    "suspend",
    "typeset",
    // Zsh-specific
    "autoload",
    "bindkey",
    "chdir",
    "emulate",
    "functions",
    "rehash",
    "setopt",
    "unfunction",
    "whence",
    "where",
    "which",
    "zcompile",
    "zle",
    "zmodload",
    "zparseopts",
    "zstyle",
];

/// Check if a command name matches a known shell builtin.
pub fn is_shell_builtin(command: &str) -> bool {
    SHELL_BUILTINS.contains(&command)
}

/// A single match found in PATH.
#[derive(Debug, Clone)]
pub struct CommandMatch {
    /// Full path to the executable.
    pub path: PathBuf,
    /// Position in PATH (0-indexed).
    pub position: usize,
    /// The PATH directory this was found in.
    pub path_dir: PathBuf,
    /// Whether this is the selected (first executable) match.
    pub is_selected: bool,
    /// Detected version (if available).
    pub version: Option<String>,
    /// Symlink information (if it's a symlink).
    pub symlink: Option<SymlinkInfo>,
    /// Whether the file is executable (vs permission denied).
    pub executable: bool,
}

/// Result of resolving a command.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// The command that was resolved.
    pub command: String,
    /// The resolved path (first executable match).
    pub resolved: Option<PathBuf>,
    /// All matches found in PATH order.
    pub matches: Vec<CommandMatch>,
    /// All PATH directories that were searched.
    pub path_searched: Vec<PathBuf>,
    /// Whether the command name matches a shell builtin.
    /// If true, the shell builtin may take precedence over PATH.
    pub is_builtin: bool,
}

/// Configuration for resolution.
#[derive(Debug, Clone)]
pub struct ResolveConfig {
    /// Timeout for version detection in milliseconds.
    pub timeout_ms: u64,
    /// Whether to skip version detection.
    pub skip_version: bool,
}

impl Default for ResolveConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            skip_version: false,
        }
    }
}

/// Resolve a command by searching all PATH entries.
pub fn resolve_command(command: &str, config: &ResolveConfig) -> ResolutionResult {
    let path_entries = get_path_entries();
    let path_searched = path_entries.clone();
    let mut matches = Vec::new();
    let mut resolved = None;

    for (position, dir) in path_entries.iter().enumerate() {
        if let Some(find_result) = find_command_in_dir(dir, command) {
            let path = find_result.path;
            let executable = find_result.executable;

            // Only the first executable match is "selected"
            let is_selected = executable && resolved.is_none();

            if is_selected {
                resolved = Some(path.clone());
            }

            // Get version only if executable and not skipped
            let version = if !executable || config.skip_version {
                None
            } else {
                detect_version(&path, config.timeout_ms)
            };

            // Get symlink info if it's a symlink
            let symlink = if is_symlink(&path) {
                Some(resolve_symlink(&path))
            } else {
                None
            };

            matches.push(CommandMatch {
                path,
                position,
                path_dir: dir.clone(),
                is_selected,
                version,
                symlink,
                executable,
            });
        }
    }

    ResolutionResult {
        command: command.to_string(),
        resolved,
        matches,
        path_searched,
        is_builtin: is_shell_builtin(command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_no_version() -> ResolveConfig {
        ResolveConfig {
            timeout_ms: 100,
            skip_version: true,
        }
    }

    #[test]
    fn test_resolve_ls() {
        let result = resolve_command("ls", &config_no_version());
        assert_eq!(result.command, "ls");
        assert!(result.resolved.is_some(), "ls should be found");
        assert!(!result.matches.is_empty());
        assert!(result.matches[0].is_selected);
    }

    #[test]
    fn test_resolve_nonexistent() {
        let result = resolve_command("this_command_does_not_exist_12345", &config_no_version());
        assert!(result.resolved.is_none());
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_first_match_is_selected() {
        let result = resolve_command("ls", &config_no_version());
        if !result.matches.is_empty() {
            assert!(result.matches[0].is_selected);
            for m in result.matches.iter().skip(1) {
                assert!(!m.is_selected);
            }
        }
    }

    #[test]
    fn test_resolved_equals_first_match() {
        let result = resolve_command("ls", &config_no_version());
        if let Some(resolved) = &result.resolved {
            assert_eq!(resolved, &result.matches[0].path);
        }
    }

    #[test]
    fn test_position_is_correct() {
        let result = resolve_command("ls", &config_no_version());
        if !result.matches.is_empty() {
            let path_entries = get_path_entries();
            let first_match = &result.matches[0];
            assert_eq!(path_entries[first_match.position], first_match.path_dir);
        }
    }

    #[test]
    fn test_skip_version() {
        let config = ResolveConfig {
            timeout_ms: 100,
            skip_version: true,
        };
        let result = resolve_command("ls", &config);
        if !result.matches.is_empty() {
            assert!(result.matches[0].version.is_none());
        }
    }

    #[test]
    fn test_default_config() {
        let config = ResolveConfig::default();
        assert_eq!(config.timeout_ms, 2000);
        assert!(!config.skip_version);
    }

    #[test]
    fn test_command_stored() {
        let result = resolve_command("cat", &config_no_version());
        assert_eq!(result.command, "cat");
    }

    #[test]
    fn test_path_searched_populated() {
        let result = resolve_command("ls", &config_no_version());
        assert!(!result.path_searched.is_empty());
    }

    #[test]
    fn test_executable_flag() {
        let result = resolve_command("ls", &config_no_version());
        if !result.matches.is_empty() {
            assert!(result.matches[0].executable);
        }
    }

    #[test]
    fn test_is_shell_builtin_true() {
        assert!(is_shell_builtin("cd"));
        assert!(is_shell_builtin("echo"));
        assert!(is_shell_builtin("export"));
        assert!(is_shell_builtin("type"));
        assert!(is_shell_builtin("alias"));
    }

    #[test]
    fn test_is_shell_builtin_false() {
        assert!(!is_shell_builtin("ls"));
        assert!(!is_shell_builtin("grep"));
        assert!(!is_shell_builtin("node"));
        assert!(!is_shell_builtin("python"));
    }

    #[test]
    fn test_resolve_builtin_sets_flag() {
        let result = resolve_command("cd", &config_no_version());
        assert!(result.is_builtin);
    }

    #[test]
    fn test_resolve_nonbuiltin_clears_flag() {
        let result = resolve_command("ls", &config_no_version());
        assert!(!result.is_builtin);
    }
}
