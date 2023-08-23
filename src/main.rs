use clap::{Args, command, Parser, Subcommand};

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
        Commands::Add { thought } => {
            let conn = rusqlite::Connection::open(db).unwrap();

            conn.execute(
                "CREATE TABLE IF NOT EXISTS thoughts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought TEXT NOT NULL
                    )",
                (),
            )?;

            conn.execute(
                "INSERT INTO thoughts (thought) VALUES (?)",
                [&thought],
            )?;
        }
    }
    Ok(())
}
