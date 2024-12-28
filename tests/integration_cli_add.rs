pub mod helpers;

mod integration_cli_add {
    use crate::helpers::TestWet;
    use chrono::{Datelike, Local};
    use predicates::prelude::predicate;

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
        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(r"{0} \[1\] This is a simple thought\n$", date_prefix_re);
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a simple thought")?;
        add.assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

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

        assert_eq!(thought.date, Local::now().date_naive());

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_today_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let _ = wet
            .add("This is a thought with today date")?
            .arg("--date")
            .arg("today")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "This is a thought with today date");

        assert_eq!(thought.date, Local::now().date_naive());

        Ok(())
    }

    #[test]
    fn refuses_thought_with_invalid_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("Seed thought")?.assert().success();
        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);

        wet.add("This is a thought with invalid date")?
            .arg("--date")
            .arg("not-a-date")
            .assert()
            .failure();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "Seed thought");

        Ok(())
    }

    // Work only with (naive) dates for now
    #[ignore]
    #[test]
    fn stores_thought_in_database_with_given_datetime() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a thought with a given datetime")?;
        add.arg("--datetime")
            .arg("2023-10-30T00:02:42+01:00")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "This is a thought with a given date");
        let expected = chrono::NaiveDate::parse_from_str("2023-10-29", "%Y-%m-%d")?;
        assert_eq!(thought.date, expected);

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_given_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought with a date")?;
        add.arg("--date").arg("2023-10-12").assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        let expected = chrono::NaiveDate::parse_from_str("2023-10-12", "%Y-%m-%d")?;
        assert_eq!(expected, thought.date);

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_given_convenient_date(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought with a convenient date")?;
        add.arg("--date").arg("Oct 12, 2022").assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        let expected = chrono::NaiveDate::parse_from_str("2022-10-12", "%Y-%m-%d")?;
        assert_eq!(thought.date, expected);

        Ok(())
    }

    #[test]
    fn stores_thought_in_database_with_given_convenient_date_without_year(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought with a convenient date without year")?;
        add.arg("--date").arg("Oct 12").assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        let now = Local::now().year();
        let expected = chrono::NaiveDate::parse_from_str("2222-10-12", "%Y-%m-%d")?
            .with_year(now)
            .unwrap();
        assert_eq!(thought.date, expected);

        Ok(())
    }

    #[test]
    fn stores_thought_with_entity_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"{0} \[1\] This is a thought about subject\n\nMentions:\n  - subject \[NEW\]\n$",
            date_prefix_re
        );
        let wet = TestWet::new()?;
        let mut add = wet.add("This is a thought about [subject]")?;
        add.assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

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
    fn stores_thought_in_database_with_two_reference_to_one_entity(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"{0} \[1\] subject and subject\n\nMentions:\n  - subject \[NEW\]\n  - subject \[NEW\]\n$",
            date_prefix_re
        );

        let wet = TestWet::new()?;
        let mut add = wet.add("[subject] and [subject]")?;
        add.assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        let thoughts_rows = wet.thoughts_rows()?;
        assert_eq!(thoughts_rows.len(), 1);
        let thought = &thoughts_rows[0];
        assert_eq!(thought.thought, "[subject] and [subject]");

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
    fn two_thoughts_with_same_entity_adds_just_one_entity() -> Result<(), Box<dyn std::error::Error>>
    {
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
    #[test]
    fn adds_thought_with_aliased_entity() -> Result<(), Box<dyn std::error::Error>> {
        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"{0} \[1\] Thought about subject\n\nMentions:\n  - Subject With Complicated Name \| aliased as subject \[NEW\]\n$",
            date_prefix_re
        );
        let wet = TestWet::new()?;
        let mut add = wet.add("Thought about [subject](Subject With Complicated Name)")?;
        add.assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        let entities_rows = wet.entities_rows()?;
        assert_eq!(entities_rows.len(), 1);
        assert_eq!(entities_rows[0].name, "Subject With Complicated Name");

        Ok(())
    }
}
