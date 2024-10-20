pub mod helpers;

mod integration_cli_thoughts {
    use predicates::prelude::predicate;
    use crate::helpers::TestWet;

    // TODO(muller): Handle empty database?

    #[test]
    fn shows_all_thoughts() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought about [subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [another subject]")?;
        third_add.assert().success();

        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(r"{0} \[1\] This is a thought about subject\n{0} \[2\] This is another thought about subject\n{0} \[3\] This is another thought about another subject\n", date_prefix_re);
        let mut thoughts = wet.thoughts()?;
        thoughts.assert().success().stdout(predicate::str::is_match(expected_output)?);

        Ok(())
    }


    #[test]
    fn shows_thoughts_on_entity() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought about [another subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [subject]")?;
        third_add.assert().success();


        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(r"{0} \[1\] This is a thought about subject\n{0} \[3\] This is another thought about subject\n", date_prefix_re);
        let mut thoughts = wet.thoughts()?;
        thoughts.arg("--on=subject")
            .assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        Ok(())
    }
}
