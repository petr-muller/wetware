mod integration {
    use assert_cmd::Command;
    use predicates::prelude::predicate;

    #[test]
    fn plain_shows_usage_and_fails() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Usage: wet"))
            .stderr(predicate::str::contains("add"));

        Ok(())
    }
}
