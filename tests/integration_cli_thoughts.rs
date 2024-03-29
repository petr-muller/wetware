mod integration_cli_thoughts {
    use assert_cmd::Command;
    use predicates::prelude::predicate;

    // TODO(muller): Handle empty database?

    #[test]
    fn shows_all_thoughts() -> Result<(), Box<dyn std::error::Error>> {
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought about [subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [another subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        let expected_output = "This is a thought about [subject]\nThis is another thought about [subject]\nThis is another thought about [another subject]\n";
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("thoughts")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }

    #[test]
    fn shows_thoughts_on_entity() -> Result<(), Box<dyn std::error::Error>> {
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought about [subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [another subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is another thought about [subject]")
            .assert()
            .success();
        let mut cmd = Command::cargo_bin("wet")?;
        let expected_output = "This is a thought about [subject]\nThis is another thought about [subject]\n";
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("thoughts")
            .arg("--on=subject")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }
}