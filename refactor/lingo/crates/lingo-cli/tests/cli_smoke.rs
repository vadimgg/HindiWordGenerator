#![allow(clippy::expect_used)]

mod support;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use support::TestWorkspace;

#[test]
fn root_and_command_help_are_available() {
    cargo_bin_cmd!("lingo")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("raw text -> reviewed source"))
        .stdout(predicate::str::contains("Commands:"));

    for arguments in [
        vec!["init", "--help"],
        vec!["import", "--help"],
        vec!["build", "--help"],
        vec!["check", "--help"],
        vec!["audio", "--help"],
        vec!["package", "--help"],
        vec!["export", "--help"],
        vec!["status", "--help"],
        vec!["lang", "--help"],
        vec!["lang", "list", "--help"],
        vec!["lang", "show", "--help"],
        vec!["lang", "which", "--help"],
        vec!["lang", "edit", "--help"],
        vec!["doctor", "--help"],
        vec!["viewer", "--help"],
    ] {
        cargo_bin_cmd!("lingo")
            .args(&arguments)
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn usage_errors_exit_two() {
    cargo_bin_cmd!("lingo")
        .arg("init")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--lang"));
}

#[test]
fn legacy_command_surface_is_rejected() {
    cargo_bin_cmd!("lingo")
        .args(["sentences", "generate"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn no_color_environment_suppresses_ansi_output() {
    let workspace = TestWorkspace::new();
    workspace
        .command()
        .args(["init", "--lang", "hindi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace"))
        .stdout(predicate::str::contains("\u{1b}[").not());
}

#[test]
fn color_always_forces_ansi_output() {
    let workspace = TestWorkspace::new();
    workspace
        .command()
        .args(["--color", "always", "init", "--lang", "hindi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace"))
        .stdout(predicate::str::contains("\u{1b}["));
}

#[test]
fn color_always_forces_help_ansi_output() {
    cargo_bin_cmd!("lingo")
        .args(["--help", "--color", "always"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("\u{1b}["));

    cargo_bin_cmd!("lingo")
        .args(["init", "--help", "--color", "always"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("\u{1b}["));
}
