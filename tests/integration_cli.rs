use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_no_args_shows_usage() {
    let mut cmd = Command::cargo_bin("wet").unwrap();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"))
        .stderr(predicate::str::contains("add"))
        .stderr(predicate::str::contains("thoughts"));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("wet").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Wetware - track, organize, and process thoughts",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("-d, --database <DATABASE>"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("wet").unwrap();

    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'invalid-command'",
        ));
}
