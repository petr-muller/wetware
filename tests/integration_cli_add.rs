mod integration_cli_add {
    use assert_cmd::cmd::Command;
    use assert_fs::NamedTempFile;
    use chrono::{DateTime, Duration, Utc};
    use rusqlite::Connection;
    use predicates::prelude::predicate;

    struct ThoughtsTableRow {
        id: isize,
        thought: String,
        datetime: DateTime<Utc>,
    }

    struct EntitiesTableRow {
        id: isize,
        name: String,
    }

    struct ThoughtsEntitiesTableRow {
        thought_id: isize,
        entity_id: isize,
    }

    struct TestWet {
        db: NamedTempFile,
    }

    impl TestWet {
        fn new() -> Result<Self, Box<dyn std::error::Error>> {
            Ok(Self {
                db: NamedTempFile::new("wetware.db")?
            })
        }
        fn cmd(&self) -> Result<Command, Box<dyn std::error::Error>> {
            let mut cmd = Command::cargo_bin("wet")?;
            cmd.env("WETWARE_DB_PATH", self.db.path());
            Ok(cmd)
        }
        fn add(&self, thought: &str) -> Result<Command, Box<dyn std::error::Error>> {
            let mut cmd = self.cmd()?;
            cmd.arg("add");
            cmd.arg(thought);
            Ok(cmd)
        }

        fn connection(&self) -> Result<Connection, Box<dyn std::error::Error>> {
            let conn = Connection::open(self.db.path())?;
            Ok(conn)
        }

        fn thoughts_rows(&self) -> Result<Vec<ThoughtsTableRow>, Box<dyn std::error::Error>> {
            let conn = self.connection()?;
            let mut stmt = conn.prepare("SELECT id, thought, datetime FROM thoughts")?;
            let rows = stmt.query_map([], |row| Ok(ThoughtsTableRow {
                id: row.get(0)?,
                thought: row.get(1)?,
                datetime: row.get(2)?,
            }))?;
            let mut thoughts = Vec::new();
            for thought in rows {
                thoughts.push(thought.unwrap())
            }
            Ok(thoughts)
        }

        fn entities_rows(&self) -> Result<Vec<EntitiesTableRow>, Box<dyn std::error::Error>> {
            let conn = self.connection()?;
            let mut stmt = conn.prepare("SELECT id, name FROM entities")?;
            let rows = stmt.query_map([], |row| Ok(EntitiesTableRow {
                id: row.get(0)?,
                name: row.get(1)?,
            }))?;
            let mut entities = Vec::new();
            for entity in rows {
                entities.push(entity.unwrap())
            }

            Ok(entities)
        }

        fn thoughts_to_entities_rows(&self) -> Result<Vec<ThoughtsEntitiesTableRow>, Box<dyn std::error::Error>> {
            let conn = self.connection()?;
            let mut stmt = conn.prepare("SELECT thought_id, entity_id FROM thoughts_entities")?;
            let rows = stmt.query_map([], |row| Ok(ThoughtsEntitiesTableRow {
                thought_id: row.get(0)?,
                entity_id: row.get(1)?,
            }))?;

            let mut links = Vec::new();
            for link in rows {
                links.push(link.unwrap())
            }

            Ok(links)
        }
    }

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
