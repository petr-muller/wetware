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
        let expected_output = r"Description for entity entity:\nexpected description".to_string();
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about [entity]")?;
        add.assert().success();
        // TODO: Uncomment
        // .stdout(predicate::str::is_match(expected_output)?);

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
    fn plain_entity_describe_emits_message_when_entity_has_no_description(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about [entity]")?;
        add.assert().success();

        let entities = wet.entities_rows()?;
        assert_eq!(1, entities.len());

        let entity = &entities[0];
        assert_eq!("", entity.description);

        let expected_output = String::from("Entity entity has no description");
        let mut describe = wet.entity()?;
        describe
            .arg("describe")
            .arg("entity")
            .assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        Ok(())
    }

    #[test]
    fn plain_entity_describe_emits_description() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about [entity]")?;
        add.assert().success();

        let mut add = wet.add("Thought about [different] entity")?;
        add.assert().success();

        let mut describe = wet.entity()?;

        describe
            .arg("describe")
            .arg("entity")
            .arg("Describes entity by comparing it to a [different] entity")
            .assert()
            .success();

        let entities = wet.entities_rows()?;
        assert_eq!(2, entities.len());

        for item in &entities {
            if item.name == "entity" {
                assert_eq!(
                    "Describes entity by comparing it to a [different] entity",
                    item.description
                );
            } else {
                assert_eq!("", item.description);
            }
        }

        let expected_output =
            String::from("Describes entity by comparing it to a different entity");
        let mut describe = wet.entity()?;
        describe
            .arg("describe")
            .arg("entity")
            .assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        Ok(())
    }

    #[test]
    fn entity_describe_fails_on_bad_entity() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about entity")?;
        add.assert().success();

        let mut describe = wet.entity()?;

        describe
            .arg("describe")
            .arg("entity")
            .arg("unexpected description")
            .assert()
            .failure();

        let entities = wet.entities_rows()?;
        assert_eq!(0, entities.len());

        Ok(())
    }
    #[test]
    fn entity_describe_can_contain_entity_references() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let _ = wet.add("Thought about [entity]")?.assert().success();
        let _ = wet.add("Thought about [reference]")?.assert().success();

        let entities = wet.entities_rows()?;
        assert_eq!(2, entities.len());

        let entity = &entities[0];
        let reference = &entities[1];

        let entity_description_links = wet.entity_description_entities_rows()?;
        assert_eq!(0, entity_description_links.len());

        let expected_output = r"Description for entity entity:\nexpected description\n\nMentions:\n  - Expects entity description to link to reference".to_string();

        let mut describe = wet.entity()?;
        describe
            .arg("describe")
            .arg("entity")
            .arg("Expects entity description to link to [reference]")
            .assert()
            .success();

        let entity_description_links = wet.entity_description_entities_rows()?;
        assert_eq!(1, entity_description_links.len());

        let description_link = &entity_description_links[0];
        assert_eq!(entity.id, description_link.entity);
        assert_eq!(reference.id, description_link.to);

        Ok(())
    }
}
