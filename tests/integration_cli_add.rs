pub mod helpers;

mod integration_cli_add {
    use chrono::Duration;
    use predicates::prelude::predicate;
    use crate::helpers::TestWet;


    #[test]
    fn plain_shows_usage_and_fails() -> Result<(), Box<dyn std::error::Error>> {
        let mut wet = TestWet::new().unwrap().cmd()?;
        wet.arg("add")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage: wet add"))
            .stderr(predicate::str::contains(" <THOUGHT> "));

        Ok(())
    }

    #[test]
    fn stores_thought_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a simple thought")?;
        add.assert().success();

        let thoughts_rows = wet.thoughts_rows()?;

        assert_eq!(thoughts_rows.len(), 1);
        assert_eq!(thoughts_rows[0].thought, "This is a simple thought");

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_default_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a thought with a default date")?;
        add.assert().success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "This is a thought with a default date");

        let age = chrono::Utc::now() - thought.datetime;
        assert!(!age.is_zero());
        assert!(age < Duration::try_seconds(1).unwrap());

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_given_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a thought with a given date")?;
        add.arg("--datetime")
            .arg("2023-10-30T00:02:42+01:00")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "This is a thought with a given date");
        let expected = chrono::DateTime::parse_from_rfc3339("2023-10-29T23:02:42+00:00").unwrap();
        assert_eq!(thought.datetime, expected);

        Ok(())
    }

    #[test]
    fn stores_thought_with_entity_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a thought about [subject]")?;
        add.assert().success();

        let thoughts_rows = wet.thoughts_rows()?;
        assert_eq!(thoughts_rows.len(), 1);
        let thought = &thoughts_rows[0];
        assert_eq!(thought.thought, "This is a thought about [subject]");

        let entities_rows = wet.entities_rows()?;

        assert_eq!(entities_rows.len(), 1);
        let entity = &entities_rows[0];
        assert_eq!(entity.name, "subject");

        let links = wet.thoughts_to_entities_rows()?;

        assert_eq!(links.len(), 1);
        let link = &links[0];
        assert_eq!(thought.id, link.thought_id);
        assert_eq!(entity.id, link.entity_id);

        Ok(())
    }

    #[test]
    fn two_thoughts_with_same_entity_adds_just_one_entity() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut first_add = wet.add("This is a thought about [subject]")?;
        first_add.assert().success();

        let mut second_add = wet.add("This is another thought about [subject]")?;
        second_add.assert().success();

        let entities_rows = wet.entities_rows()?;

        assert_eq!(entities_rows.len(), 1);
        assert_eq!(entities_rows[0].name, "subject");

        let links = wet.thoughts_to_entities_rows()?;
        assert_eq!(links.len(), 2);

        Ok(())
    }
}
