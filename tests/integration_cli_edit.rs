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
            .assert().
            success();

        let thought_rows = wet.thoughts_rows()?;

        assert_eq!(1, thought_rows.len());
        let thought = &thought_rows[0];
        let expected_before = chrono::NaiveDate::parse_from_str("2022-02-22", "%Y-%m-%d")?;
        assert_eq!(expected_before, thought.date);
        let id = thought.id;

        wet.edit(id)?
            .arg("--date")
            .arg("today")
            .assert()
            .success();

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
            .assert().
            success();

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
}
