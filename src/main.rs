use chrono::{DateTime, Utc};
use clap::{Args, command, Parser, Subcommand};
use rusqlite::{params, params_from_iter};
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Parser)]
#[clap(name = "wet", version)]
pub struct Wet {
    #[clap(flatten)]
    globals: GlobalFlags,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new thought to the database
    #[command(name = "add", arg_required_else_help = true)]
    Add {
        /// The thought to add
        thought: String,
        #[arg(short, long)]
        datetime: Option<DateTime<Utc>>,
    },
    #[command(name = "thoughts")]
    Thoughts {
        #[arg(long="on")]
        entity: Option<String>,
    },
}


#[derive(Debug, Args)]
struct GlobalFlags {
    /// The path to the database
    #[arg(long, env = "WETWARE_DB_PATH", required(false))]
    db: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Wet::parse();

    let db = args.globals.db.unwrap_or_else(|| {
        eprintln!("No database path provided");
        std::process::exit(1);
    });

    match args.command {
        Commands::Thoughts { entity } => {
            let conn = rusqlite::Connection::open(db).unwrap();

            let mut stmt_lines = vec!["SELECT thought FROM thoughts"];
            let mut params = vec![];
            if let Some(entity) = entity {
                stmt_lines.append(&mut vec![
                    "JOIN thoughts_entities ON thoughts.id = thoughts_entities.thought_id",
                    "JOIN entities ON thoughts_entities.entity_id = entities.id",
                    "WHERE entities.name = ?1"]);
                params.push(entity)
            }
            stmt_lines.push("ORDER BY datetime");
            let mut stmt = conn.prepare(stmt_lines.join("\n").as_str())?;

            let rows = stmt.query_map(params_from_iter(params), |row| row.get::<usize, String>(0))?;
            for thought in rows {
                println!("{}", thought.unwrap());
            }
        }
        Commands::Add { thought, datetime } => {
            let conn = rusqlite::Connection::open(db).unwrap();

            conn.execute(
                "CREATE TABLE IF NOT EXISTS thoughts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought TEXT NOT NULL,
                    datetime INTEGER NOT NULL
                    )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEST NOT NULL UNIQUE
                    )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS thoughts_entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought_id INTEGER,
                    entity_id INTEGER,
                    FOREIGN KEY(thought_id) REFERENCES thoughts(id),
                    FOREIGN KEY(entity_id) REFERENCES entities(id),
                    UNIQUE(thought_id, entity_id)
                    )",
                (),
            )?;

            let now = datetime.unwrap_or_else(chrono::offset::Utc::now);

            conn.execute(
                "INSERT INTO thoughts (thought, datetime) VALUES (?1, ?2)",
                params![&thought, &now],
            )?;
            let thought_id = conn.last_insert_rowid();

            lazy_static! {
                static ref ENTITY_RE: Regex = Regex::new(r"\[[^\[]+\]").unwrap();
            }
            let entities: Vec<&str> = ENTITY_RE.find_iter(&thought)
                .map(|entity| entity.as_str())
                .collect();

            for entity in entities {
                let entity_name = &entity[1..entity.len() - 1];
                conn.execute(
                    "INSERT INTO entities (name) VALUES (?1)
                    ON CONFLICT(name) DO NOTHING",
                    params![entity_name],
                )?;
                let mut stmt = conn.prepare("SELECT id FROM entities WHERE name=?1")?;
                let mut rows = stmt.query_map(params![entity_name], |row| row.get::<usize, usize>(0))?;
                let entity_id = rows.next().unwrap().unwrap();
                conn.execute(
                    "INSERT INTO thoughts_entities (thought_id, entity_id) VALUES (?1, ?2)",
                    params![thought_id, entity_id],
                )?;
            }
        }
    }
    Ok(())
}
