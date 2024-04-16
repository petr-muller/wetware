use assert_cmd::Command;

pub struct ThoughtsTableRow {
    pub id: isize,
    pub thought: String,
    pub datetime: chrono::DateTime<chrono::Utc>,
}

pub struct EntitiesTableRow {
    pub id: isize,
    pub name: String,
}

pub struct ThoughtsEntitiesTableRow {
    pub thought_id: isize,
    pub entity_id: isize,
}

pub struct TestWet {
    db: assert_fs::NamedTempFile,
}

impl TestWet {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db: assert_fs::NamedTempFile::new("wetware.db")?
        })
    }
    pub fn cmd(&self) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("wet")?;
        cmd.env("WETWARE_DB_PATH", self.db.path());
        Ok(cmd)
    }
    pub fn add(&self, thought: &str) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = self.cmd()?;
        cmd.arg("add");
        cmd.arg(thought);
        Ok(cmd)
    }

    pub fn entities(&self) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = self.cmd()?;
        cmd.arg("entities");
        Ok(cmd)
    }

    pub fn thoughts(&self) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = self.cmd()?;
        cmd.arg("thoughts");
        Ok(cmd)
    }

    fn connection(&self) -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
        let conn = rusqlite::Connection::open(self.db.path())?;
        Ok(conn)
    }

    pub fn thoughts_rows(&self) -> Result<Vec<ThoughtsTableRow>, Box<dyn std::error::Error>> {
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

    pub fn entities_rows(&self) -> Result<Vec<EntitiesTableRow>, Box<dyn std::error::Error>> {
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

    pub fn thoughts_to_entities_rows(&self) -> Result<Vec<ThoughtsEntitiesTableRow>, Box<dyn std::error::Error>> {
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