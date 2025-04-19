pub mod helpers;

mod integration_cli_edit {
    use crate::helpers::TestWet;
    use chrono::Local;
    use predicates::prelude::predicate;

    #[test]
    fn plain_shows_usage_and_fails() -> Result<(), Box<dyn std::error::Error>> {
        let mut wet = TestWet::new()?.cmd()?;
        wet.arg("edit")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage: wet edit"))
            .stderr(predicate::str::contains(" <THOUGHT_ID> "));

        Ok(())
    }

    #[test]
    fn sets_today_date_on_thought_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("This is a thought with date")?
            .arg("--date")
            .arg("2022-02-22")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(1, thought_rows.len());
        let thought = &thought_rows[0];
        let expected_before = chrono::NaiveDate::parse_from_str("2022-02-22", "%Y-%m-%d")?;
        assert_eq!(expected_before, thought.date);
        let id = thought.id;

        wet.edit(id)?.arg("--date").arg("today").assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let thought = &thought_rows[0];
        let expected_after = Local::now().date_naive();
        assert_eq!(expected_after, thought.date);

        Ok(())
    }
    #[test]
    fn sets_given_date_on_thought_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("This is a thought without date")?
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(1, thought_rows.len());
        let thought = &thought_rows[0];
        let before = thought.date;
        let id = thought.id;

        wet.edit(id)?
            .arg("--date")
            .arg("2022-02-22")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let thought = &thought_rows[0];
        let expected = chrono::NaiveDate::parse_from_str("2022-02-22", "%Y-%m-%d")?;
        assert_eq!(expected, thought.date);
        assert_ne!(before, thought.date);

        Ok(())
    }

    #[test]
    fn edits_simple_thought_without_altering_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("This is a thought")?
            .arg("--date")
            .arg("2022-02-02")
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let before = &thought_rows[0];

        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"Before: {0} \[1\] This is a thought\nAfter:  {0} \[1\] Changed thought",
            date_prefix_re
        );

        wet.edit(before.id)?.arg("Changed thought").assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let after = &thought_rows[0];

        assert_eq!(before.date, after.date);
        assert_eq!("Changed thought", after.thought);

        Ok(())
    }

    #[test]
    fn adds_entity_refs() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("This is a thought")?.assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let before = &thought_rows[0];

        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"Before: {0} \[1\] This is a thought\nAfter:  {0} \[1\] This is a thought\n\nMentions:\n  - This \[NEW\]\n  - thought \[NEW\]",
            date_prefix_re
        );

        wet.edit(before.id)?.arg("[This] is a [thought]").assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let after = &thought_rows[0];

        assert_eq!(before.date, after.date);
        assert_eq!("[This] is a [thought]", after.thought);

        let entities_rows = wet.entities_rows()?;
        assert_eq!(entities_rows.len(), 2);

        let links = wet.thoughts_to_entities_rows()?;
        assert_eq!(links.len(), 2);

        Ok(())
    }

    #[test]
    fn removes_entity_refs() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("[This] is a [thought] about [subject]")?
            .assert()
            .success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let before = &thought_rows[0];

        let entities_rows = wet.entities_rows()?;
        assert_eq!(entities_rows.len(), 3);

        let links = wet.thoughts_to_entities_rows()?;
        assert_eq!(links.len(), 3);

        let date_prefix_re = r"\d{4} [A-Z][a-z]{2} \d{2}";
        let expected_output = format!(
            r"Before: {0} \[1\] This is a thought about subject\nAfter:  {0} \[1\] This is a thought about subject with some details\n\nMentions:\n  - subject",
            date_prefix_re
        );

        wet.edit(before.id)?
            .arg("This is a thought about [subject] with some details")
            .assert()
            .success()
            .stdout(predicate::str::is_match(expected_output)?);

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let after = &thought_rows[0];

        assert_eq!(
            "This is a thought about [subject] with some details",
            after.thought
        );

        let entities_rows = wet.entities_rows()?;
        assert_eq!(entities_rows.len(), 3);

        let links = wet.thoughts_to_entities_rows()?;
        assert_eq!(links.len(), 1);

        Ok(())
    }

    #[test]
    fn refuses_bad_date() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("Original")?.assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let before = &thought_rows[0];

        wet.edit(before.id)?
            .arg("--date")
            .arg("not-a-date")
            .assert()
            .failure();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "Original");

        Ok(())
    }

    #[test]
    fn refuses_bad_thought() -> Result<(), Box<dyn std::error::Error>> {
        let wet = TestWet::new()?;
        wet.add("Original")?.assert().success();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(1, thought_rows.len());
        let before = &thought_rows[0];

        wet.edit(before.id)?
            .arg("Changed [to invalid")
            .assert()
            .failure();

        let thought_rows = wet.thoughts_rows()?;
        assert_eq!(thought_rows.len(), 1);
        let thought = &thought_rows[0];
        assert_eq!(thought.thought, "Original");

        Ok(())
    }
}
