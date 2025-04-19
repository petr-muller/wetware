use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use tempfile::NamedTempFile;

#[test]
fn test_list_thoughts_empty() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();

    // Initialize the database
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thoughts (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    // List thoughts (should be empty)
    let mut cmd = Command::cargo_bin("wet").unwrap();
    cmd.arg("--database")
        .arg(db_path)
        .arg("thoughts")
        .assert()
        .success()
        .stdout(predicate::str::contains("Listing all thoughts:"))
        .stdout(predicate::str::contains("No thoughts found."));
}

#[test]
fn test_list_thoughts_with_data() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();

    // Initialize the database and add some thoughts
    let conn = Connection::open(db_path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thoughts (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL
        )",
        [],
    )
    .unwrap();

    conn.execute(
        "INSERT INTO thoughts (content) VALUES (?)",
        params!["First test thought"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO thoughts (content) VALUES (?)",
        params!["Second test thought"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO thoughts (content) VALUES (?)",
        params!["Third test thought"],
    )
    .unwrap();

    // List thoughts
    let mut cmd = Command::cargo_bin("wet").unwrap();
    cmd.arg("--database")
        .arg(db_path)
        .arg("thoughts")
        .assert()
        .success()
        .stdout(predicate::str::contains("Listing all thoughts:"))
        .stdout(predicate::str::contains("1: First test thought"))
        .stdout(predicate::str::contains("2: Second test thought"))
        .stdout(predicate::str::contains("3: Third test thought"));
}

#[test]
fn test_add_and_list_workflow() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();

    // Add thoughts
    Command::cargo_bin("wet")
        .unwrap()
        .arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("Workflow thought 1")
        .assert()
        .success();

    Command::cargo_bin("wet")
        .unwrap()
        .arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("Workflow thought 2")
        .assert()
        .success();

    // List thoughts
    Command::cargo_bin("wet")
        .unwrap()
        .arg("--database")
        .arg(db_path)
        .arg("thoughts")
        .assert()
        .success()
        .stdout(predicate::str::contains("1: Workflow thought 1"))
        .stdout(predicate::str::contains("2: Workflow thought 2"));
}
