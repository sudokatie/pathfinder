//! Integration tests for pathfinder CLI.

use assert_cmd::Command;
use predicates::prelude::*;

fn pathfinder() -> Command {
    Command::cargo_bin("pathfinder").unwrap()
}

#[test]
fn test_help_flag() {
    pathfinder()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Debug command resolution"));
}

#[test]
fn test_version_flag() {
    pathfinder()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("pathfinder"));
}

#[test]
fn test_resolve_existing_command() {
    pathfinder()
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("RESOLVED"));
}

#[test]
fn test_resolve_nonexistent_command() {
    pathfinder()
        .arg("this_command_does_not_exist_xyz123")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("NOT FOUND"));
}

#[test]
fn test_json_output() {
    pathfinder()
        .args(["ls", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"ls\""));
}

#[test]
fn test_plain_output() {
    pathfinder()
        .args(["ls", "--plain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RESOLVED:"));
}

#[test]
fn test_analyze_mode() {
    pathfinder()
        .arg("--analyze")
        .assert()
        .success()
        .stdout(predicate::str::contains("PATH Analysis"));
}

#[test]
fn test_no_command_error() {
    pathfinder()
        .assert()
        .code(2)
        .stderr(predicate::str::contains("No command specified"));
}

#[test]
fn test_explain_mode() {
    pathfinder()
        .args(["ls", "--explain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("resolves to"));
}

#[test]
fn test_no_version_flag() {
    pathfinder().args(["ls", "--no-version"]).assert().success();
}

#[test]
fn test_diff_mode() {
    pathfinder()
        .args(["--diff", "ls", "cat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Command Comparison"));
}

#[test]
fn test_diff_mode_json() {
    pathfinder()
        .args(["--diff", "ls", "cat", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"same_source\""));
}

#[test]
fn test_diff_requires_two_commands() {
    pathfinder()
        .args(["--diff", "ls"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("requires at least 2 commands"));
}
