pub mod helpers;

mod integration_cli_entities {
    use crate::helpers::TestWet;
    use predicates::prelude::predicate;

    #[test]
    fn shows_no_entities_when_there_are_none() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought")?;
        second_add.assert().success();

        let expected_output = "No entities in the database\n";
        let mut entities = wet.entities()?;
        entities
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

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
        entities
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }

    #[test]
    fn entity_list_shows_all_entities_sorted_by_name() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is another thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is a thought about [a subject]")?;
        second_add.assert().success();

        let mut third_add = wet.add("This is another thought about [b subject]")?;
        third_add.assert().success();

        let expected_output = "a subject\nb subject\nsubject\n";
        let mut entities = wet.entity()?;
        entities
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::eq(expected_output));

        Ok(())
    }

    #[test]
    fn entity_describe_adds_description_to_entity() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about [entity]")?;
        add.assert().success();

        let mut describe = wet.entity()?;

        describe
            .arg("describe")
            .arg("entity")
            .arg("expected description")
            .assert()
            .success();

        let entities = wet.entities_rows()?;
        assert_eq!(1, entities.len());

        let entity = &entities[0];
        assert_eq!("expected description", entity.description);

        Ok(())
    }

    #[test]
    fn entity_describe_fails_on_missing_description() -> Result<(), Box<dyn std::error::Error>> {
        todo!("TODO")
    }

    #[test]
    fn entity_describe_fails_on_bad_entity() -> Result<(), Box<dyn std::error::Error>> {
        todo!("TODO")
    }
    #[test]
    fn entity_describe_can_contain_entity_references() -> Result<(), Box<dyn std::error::Error>> {
        todo!("TODO")
    }
}
