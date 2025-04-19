use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::{Connection, params};
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_add_thought() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Add a thought
    let mut cmd = Command::cargo_bin("wet").unwrap();
    cmd.arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("This is a test thought")
        .assert()
        .success()
        .stdout(predicate::str::contains("Adding thought: This is a test thought"))
        .stdout(predicate::str::contains("Thought saved with ID: 1"));

    // Verify the thought was added to the database
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("SELECT id, content FROM thoughts WHERE id = ?").unwrap();
    let thought = stmt.query_row(params![1], |row| {
        Ok((row.get::<_, i64>(0).unwrap(), row.get::<_, String>(1).unwrap()))
    }).unwrap();
    
    assert_eq!(thought, (1, "This is a test thought".to_string()));
}

#[test]
fn test_add_multiple_thoughts() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    // Add first thought
    let mut cmd1 = Command::cargo_bin("wet").unwrap();
    cmd1.arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("First thought")
        .assert()
        .success()
        .stdout(predicate::str::contains("Thought saved with ID: 1"));
    
    // Add second thought
    let mut cmd2 = Command::cargo_bin("wet").unwrap();
    cmd2.arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("Second thought")
        .assert()
        .success()
        .stdout(predicate::str::contains("Thought saved with ID: 2"));
    
    // Verify both thoughts were added to the database
    let conn = Connection::open(db_path).unwrap();
    let mut stmt = conn.prepare("SELECT id, content FROM thoughts ORDER BY id").unwrap();
    let thoughts: Vec<(i64, String)> = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0).unwrap(), row.get::<_, String>(1).unwrap()))
    }).unwrap().map(|r| r.unwrap()).collect();
    
    assert_eq!(thoughts.len(), 2);
    assert_eq!(thoughts[0], (1, "First thought".to_string()));
    assert_eq!(thoughts[1], (2, "Second thought".to_string()));
}

#[test]
fn test_add_thought_custom_database() {
    // Create a unique database filename that doesn't exist yet
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("custom_test.db");
    let db_path_str = db_path.to_str().unwrap();
    
    // Make sure it doesn't exist yet
    assert!(!db_path.exists());
    
    // Add a thought to the custom database
    let mut cmd = Command::cargo_bin("wet").unwrap();
    cmd.arg("-d")
        .arg(db_path_str)
        .arg("add")
        .arg("Thought in custom database")
        .assert()
        .success();
    
    // Verify the database was created and the thought was added
    assert!(db_path.exists());
    
    let conn = Connection::open(&db_path).unwrap();
    let mut stmt = conn.prepare("SELECT content FROM thoughts WHERE id = 1").unwrap();
    let content: String = stmt.query_row([], |row| row.get(0)).unwrap();
    
    assert_eq!(content, "Thought in custom database");
    
    // Clean up
    fs::remove_file(db_path).unwrap();
}

#[test]
fn test_add_empty_thought() {
    // Create a temporary database file
    let db_file = NamedTempFile::new().unwrap();
    let db_path = db_file.path().to_str().unwrap();
    
    let mut cmd = Command::cargo_bin("wet").unwrap();
    
    cmd.arg("--database")
        .arg(db_path)
        .arg("add")
        .arg("")
        .assert()
        .success()
        .stdout(predicate::str::contains("Adding thought:"))
        .stdout(predicate::str::contains("Thought saved with ID: 1"));
}