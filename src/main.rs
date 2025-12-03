mod commands;
mod data;
mod storage;

use anyhow::Result;
use clap::{Parser, Subcommand};

use storage::Storage;

#[derive(Parser)]
#[command(name = "claude-sessions")]
#[command(about = "CLI tool for managing Claude Code sessions", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a session to a label (creates label if it doesn't exist)
    Add {
        /// The label name to add the session to
        label: String,
        /// The Claude session ID
        session_id: String,
        /// Optional description for this session
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Resume a session by label
    Resume {
        /// The label name to resume
        label: String,
        /// Interactively pick from multiple sessions
        #[arg(short, long)]
        pick: bool,
    },

    /// List all labels, or sessions for a specific label
    List {
        /// Optional label name to show details for
        label: Option<String>,
    },

    /// Remove a label or a specific session from a label
    Remove {
        /// The label name
        label: String,
        /// Optional session ID to remove (removes entire label if not specified)
        session_id: Option<String>,
    },

    /// Set or update a label's description
    Describe {
        /// The label name
        label: String,
        /// The description (clears if not provided)
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show configuration info
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let storage = Storage::new()?;

    match cli.command {
        Commands::Add {
            label,
            session_id,
            description,
        } => commands::add(&storage, &label, &session_id, description),

        Commands::Resume { label, pick } => commands::resume(&storage, &label, pick),

        Commands::List { label } => commands::list(&storage, label.as_deref()),

        Commands::Remove { label, session_id } => {
            commands::remove(&storage, &label, session_id.as_deref())
        }

        Commands::Describe { label, description } => {
            commands::describe(&storage, &label, description)
        }

        Commands::Config => commands::config(&storage),
    }
}
