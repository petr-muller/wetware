mod integration_cli_entities {
    use assert_cmd::Command;
    use predicates::prelude::predicate;

    #[test]
    fn shows_no_entities_when_there_are_none() -> Result<(), Box<dyn std::error::Error>> {
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        let expected_output = "No entities in the database\n";
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("entities")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }

    #[test]
    fn shows_all_entities_sorted_by_name() -> Result<(), Box<dyn std::error::Error>> {
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought about [a subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [b subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        let expected_output = "a subject\nb subject\nsubject\n";
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("entities")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }
}