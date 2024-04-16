pub mod helpers;

mod integration_cli_entities {
    use predicates::prelude::predicate;
    use crate::helpers::TestWet;

    #[test]
    fn shows_no_entities_when_there_are_none() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought")?;
        second_add.assert().success();

        let expected_output = "No entities in the database\n";
        let mut entities = wet.entities()?;
        entities.assert().success().stdout(predicate::eq(expected_output));

        Ok(())
    }

    #[test]
    fn shows_all_entities_sorted_by_name() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is another thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is a thought about [a subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [b subject]")?;
        third_add.assert().success();

        let expected_output = "a subject\nb subject\nsubject\n";
        let mut entities = wet.entities()?;
        entities.assert().success().stdout(predicate::eq(expected_output));

        Ok(())
    }
}