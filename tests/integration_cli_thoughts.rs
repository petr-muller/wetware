pub mod helpers;

mod integration_cli_thoughts {
    use predicates::prelude::predicate;
    use crate::helpers::TestWet;

    // TODO(muller): Handle empty database?

    // TODO(muller): Stopped working after Ratatui
    #[ignore]
    #[test]
    fn shows_all_thoughts() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought about [subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [another subject]")?;
        third_add.assert().success();

        let expected_output = "This is a thought about [subject]\nThis is another thought about [subject]\nThis is another thought about [another subject]\n";
        let mut thoughts = wet.thoughts()?;
        thoughts.assert().success().stdout(predicate::eq(expected_output));

        Ok(())
    }

    // TODO(muller): Stopped working after Ratatui
    #[ignore]
    #[test]
    fn shows_thoughts_on_entity() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought about [another subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [subject]")?;
        third_add.assert().success();


        let expected_output = "This is a thought about [subject]\nThis is another thought about [subject]\n";
        let mut thoughts = wet.thoughts()?;
        thoughts.arg("--on=subject")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }
}
