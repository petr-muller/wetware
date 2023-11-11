mod integration {
    use assert_cmd::Command;
    use chrono::{DateTime, Duration, Utc};
    use predicates::prelude::predicate;
    use rusqlite::{Connection};

    #[test]
    fn plain_shows_usage_and_fails() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Usage: wet"))
            .stderr(predicate::str::contains("add"));

        Ok(())
    }

    #[test]
    fn add_plain_shows_usage_and_fails() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.arg("add")
            .assert()
            .failure()
            .stderr(predicate::str::contains("Usage: wet add"))
            .stderr(predicate::str::contains(" <THOUGHT> "));

        Ok(())
    }

    #[test]
    fn add_stores_thought_in_database() -> Result<(), Box<dyn std::error::Error>> {
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a simple thought")
            .assert()
            .success();

        let conn = Connection::open(db.path())?;
        let mut stmt = conn.prepare("SELECT thought FROM thoughts")?;
        let rows = stmt.query_map([], |row| row.get::<usize, String>(0))?;
        let mut thoughts = Vec::new();
        for thought in rows {
            thoughts.push(thought);
        }

        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].as_ref().unwrap(), "This is a simple thought");

        Ok(())
    }

    #[test]
    fn add_stores_thought_in_database_with_default_date() -> Result<(), Box<dyn std::error::Error>> {
        struct ThoughtWithDate {
            thought: String,
            datetime: DateTime<Utc>,
        }

        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought with a default date")
            .assert()
            .success();
        let conn = Connection::open(db.path())?;
        let mut stmt = conn.prepare("SELECT thought, datetime FROM thoughts")?;
        let rows = stmt.query_map([], |row| Ok(ThoughtWithDate {
            thought: row.get(0)?,
            datetime: row.get(1)?,
        }))?;
        let mut thoughts = Vec::new();
        for thought in rows {
            thoughts.push(thought)
        }

        assert_eq!(thoughts.len(), 1);
        let thought = thoughts[0].as_ref().unwrap();
        assert_eq!(thought.thought, "This is a thought with a default date");
        let now = chrono::Utc::now();
        let age = now - thought.datetime;
        assert!(!age.is_zero());
        assert!(age < Duration::seconds(1));

        Ok(())
    }

    #[test]
    fn add_stores_thought_in_database_with_given_date() -> Result<(), Box<dyn std::error::Error>> {
        struct ThoughtWithDate {
            thought: String,
            datetime: DateTime<Utc>,
        }

        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("--datetime")
            .arg("2023-10-30T00:02:42+01:00")
            .arg("This is a thought with a given date")
            .assert()
            .success();
        let conn = Connection::open(db.path())?;
        let mut stmt = conn.prepare("SELECT thought, datetime FROM thoughts")?;
        let rows = stmt.query_map([], |row| Ok(ThoughtWithDate {
            thought: row.get(0)?,
            datetime: row.get(1)?,
        }))?;
        let mut thoughts = Vec::new();
        for thought in rows {
            thoughts.push(thought)
        }

        assert_eq!(thoughts.len(), 1);
        let thought = thoughts[0].as_ref().unwrap();
        assert_eq!(thought.thought, "This is a thought with a given date");
        let expected = chrono::DateTime::parse_from_rfc3339("2023-10-29T23:02:42+00:00").unwrap();
        assert_eq!(thought.datetime, expected);

        Ok(())
    }

    #[test]
    fn add_stores_thought_with_entity_in_database() -> Result<(), Box<dyn std::error::Error>> {
        struct StringWithId {
            id: isize,
            content: String,
        }
        let db = assert_fs::NamedTempFile::new("wetware.db")?;
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", db.path())
            .arg("add")
            .arg("This is a thought about [subject]")
            .assert()
            .success();
        let conn = Connection::open(db.path())?;
        let mut stmt = conn.prepare("SELECT id, thought FROM thoughts")?;
        let rows = stmt.query_map([], |row| Ok(StringWithId {
            id: row.get(0)?,
            content: row.get(1)?,
        }))?;
        let mut thoughts = Vec::new();
        for thought in rows {
            thoughts.push(thought)
        }
        assert_eq!(thoughts.len(), 1);
        let thought = thoughts[0].as_ref().unwrap();
        assert_eq!(thought.content, "This is a thought about [subject]");

        let mut stmt = conn.prepare("SELECT id, name FROM entities")?;
        let rows = stmt.query_map([], |row| Ok(StringWithId {
            id: row.get(0)?,
            content: row.get(1)?,
        }))?;
        let mut entities = Vec::new();
        for entity in rows {
            entities.push(entity)
        }
        assert_eq!(entities.len(), 1);
        let entity = entities[0].as_ref().unwrap();
        assert_eq!(entity.content, "subject");

        struct ManyToManyItem {
            left: isize,
            right: isize,
        }

        let mut stmt = conn.prepare("SELECT thought_id, entity_id FROM thoughts_entities")?;
        let rows = stmt.query_map([], |row| Ok(ManyToManyItem {
            left: row.get(0)?,
            right: row.get(1)?,
        }))?;
        let mut links = Vec::new();
        for link in rows {
            links.push(link)
        }
        assert_eq!(links.len(), 1);
        let link = links[0].as_ref().unwrap();
        assert_eq!(thought.id, link.left);
        assert_eq!(entity.id, link.right);

        Ok(())
    }
    //TODO(muller): Adding two thoughts with single entity creates just one entity row
}

