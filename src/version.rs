//! Version detection.
//!
//! Detects version of executables by running them with version flags.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Default timeout for version detection in milliseconds.
#[allow(dead_code)]
pub const DEFAULT_TIMEOUT_MS: u64 = 2000;

/// Version flags to try, in order.
const VERSION_FLAGS: &[&str] = &["--version", "-v", "-V", "version"];

/// Detect the version of an executable.
///
/// Tries various version flags and returns the first line of output.
/// Returns None if version can't be detected within the timeout.
pub fn detect_version(path: &Path, timeout_ms: u64) -> Option<String> {
    for flag in VERSION_FLAGS {
        if let Some(version) = try_version_flag(path, flag, timeout_ms) {
            return Some(version);
        }
    }
    None
}

/// Try a single version flag.
fn try_version_flag(path: &Path, flag: &str, timeout_ms: u64) -> Option<String> {
    let mut cmd = Command::new(path);
    cmd.arg(flag)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Wait with timeout
    let output = wait_with_timeout(child, timeout_ms)?;

    // Check exit status
    if !output.status.success() {
        return None;
    }

    // Try stdout first, then stderr
    let text = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else if !output.stderr.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        return None;
    };

    // Return first non-empty line that looks like version output
    text.lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && looks_like_version(l))
        .map(|s| s.to_string())
}

/// Heuristic check if a line looks like version output.
/// Filters out false positives like directory listings from `ls -v`.
fn looks_like_version(line: &str) -> bool {
    // Must contain at least one digit (versions have numbers)
    if !line.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }

    // Should not be just a filename (no path separators, not too short without keywords)
    if !line.contains(' ') && !line.contains('/') && line.len() < 10 {
        // Short single word without spaces - might be a filename
        // Accept if it contains version-like patterns
        if !line.contains('.') && !line.to_lowercase().contains('v') {
            return false;
        }
    }

    // Common version indicators
    let version_keywords = ["version", "Version", "VERSION", " v", "v.", "v:", "."];
    let has_keyword = version_keywords.iter().any(|k| line.contains(k));

    // If it has a keyword, likely a version
    if has_keyword {
        return true;
    }

    // Accept patterns like "1.2.3" or "node v20.0.0"
    // But reject things that look like filenames (single words without dots between numbers)
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() == 1 {
        // Single word - must have version-like structure (X.Y or vX)
        let word = parts[0];
        word.contains('.') || word.starts_with('v') || word.starts_with('V')
    } else {
        // Multi-word - more likely to be version info
        true
    }
}

/// Wait for a process with timeout.
fn wait_with_timeout(child: std::process::Child, timeout_ms: u64) -> Option<std::process::Output> {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    let timeout = Duration::from_millis(timeout_ms);
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => Some(output),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_ls() {
        // ls --version should work on Linux, might not on macOS
        let ls_path = Path::new("/bin/ls");
        if ls_path.exists() {
            // Just verify it doesn't panic
            let _ = detect_version(ls_path, 2000);
        }
    }

    #[test]
    fn test_detect_version_nonexistent() {
        let path = Path::new("/nonexistent/binary");
        let result = detect_version(path, 1000);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_version_timeout() {
        // Use sleep to test timeout (if available)
        let sleep_path = Path::new("/bin/sleep");
        if sleep_path.exists() {
            // With 1ms timeout, sleep should timeout
            let result = detect_version(sleep_path, 1);
            // Should be None due to timeout or invalid flag
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_try_version_flag_invalid() {
        let path = Path::new("/nonexistent/path");
        let result = try_version_flag(path, "--version", 1000);
        assert!(result.is_none());
    }

    #[test]
    fn test_version_flags_order() {
        // Verify the flags are in expected order
        assert_eq!(VERSION_FLAGS[0], "--version");
        assert_eq!(VERSION_FLAGS[1], "-v");
        assert_eq!(VERSION_FLAGS[2], "-V");
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(DEFAULT_TIMEOUT_MS, 2000);
    }

    #[test]
    fn test_looks_like_version_true_cases() {
        assert!(looks_like_version("v20.10.0"));
        assert!(looks_like_version("node v20.10.0"));
        assert!(looks_like_version("Python 3.11.0"));
        assert!(looks_like_version("rustc 1.75.0 (82e1608df 2023-12-21)"));
        assert!(looks_like_version("git version 2.42.0"));
        assert!(looks_like_version("1.2.3"));
        assert!(looks_like_version("npm 10.2.3"));
    }

    #[test]
    fn test_looks_like_version_false_cases() {
        // Filenames from `ls -v` output
        assert!(!looks_like_version("Cargo.lock"));
        assert!(!looks_like_version("README"));
        assert!(!looks_like_version("src"));
        // No digits
        assert!(!looks_like_version("hello world"));
        assert!(!looks_like_version("foobar"));
    }
}
