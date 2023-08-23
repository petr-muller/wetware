use clap::{command, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "wet", version)]
pub struct Wet {
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

fn main() {
    let _args = Wet::parse();
}
