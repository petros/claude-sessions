use anyhow::{bail, Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use std::env;
use std::process::Command;

use crate::data::Session;
use crate::storage::Storage;

pub fn add(
    storage: &Storage,
    label: &str,
    session_id: &str,
    description: Option<String>,
) -> Result<()> {
    let mut store = storage.load()?;

    let current_path = env::current_dir()
        .context("Could not get current directory")?
        .to_string_lossy()
        .to_string();

    let session = Session {
        session_id: session_id.to_string(),
        path: current_path.clone(),
        description: description.clone(),
        created_at: Utc::now(),
    };

    let label_entry = store.get_or_create_label(label);
    label_entry.add_session(session);

    storage.save(&store)?;

    println!("{} Added session to label '{}'", "✓".green(), label.cyan());
    println!("  Session: {}", session_id);
    println!("  Path: {}", current_path);
    if let Some(desc) = description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

pub fn resume(storage: &Storage, label: &str, pick: bool) -> Result<()> {
    let store = storage.load()?;

    let label_entry = store
        .get_label(label)
        .with_context(|| format!("Label '{}' not found", label))?;

    if label_entry.sessions.is_empty() {
        bail!("Label '{}' has no sessions", label);
    }

    let session = if pick && label_entry.sessions.len() > 1 {
        pick_session(label_entry)?
    } else {
        label_entry
            .latest_session()
            .context("No sessions available")?
    };

    println!(
        "{} Resuming session: {}",
        "→".blue(),
        session.session_id.cyan()
    );
    println!("  In directory: {}", session.path);
    if let Some(ref desc) = session.description {
        println!("  Description: {}", desc);
    }
    println!();

    // Change to the session's directory and run claude --resume
    let status = Command::new("claude")
        .arg("--resume")
        .arg(&session.session_id)
        .current_dir(&session.path)
        .status()
        .context("Failed to execute claude command")?;

    if !status.success() {
        bail!("Claude exited with status: {}", status);
    }

    Ok(())
}

fn pick_session(label: &crate::data::Label) -> Result<&Session> {
    let mut sessions: Vec<&Session> = label.sessions.iter().collect();
    sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let items: Vec<String> = sessions
        .iter()
        .map(|s| {
            let desc = s
                .description
                .as_ref()
                .map(|d| format!(" - {}", d))
                .unwrap_or_default();
            format!(
                "{} ({}){}",
                s.session_id.chars().take(8).collect::<String>(),
                s.created_at.format("%Y-%m-%d %H:%M"),
                desc
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a session")
        .items(&items)
        .default(0)
        .interact()
        .context("Failed to get selection")?;

    Ok(sessions[selection])
}

pub fn list(storage: &Storage, label: Option<&str>) -> Result<()> {
    let store = storage.load()?;

    match label {
        Some(label_name) => {
            let label_entry = store
                .get_label(label_name)
                .with_context(|| format!("Label '{}' not found", label_name))?;

            println!("{}", label_name.cyan().bold());
            if let Some(ref desc) = label_entry.description {
                println!("  {}", desc.dimmed());
            }
            println!();

            if label_entry.sessions.is_empty() {
                println!("  No sessions");
            } else {
                let mut sessions: Vec<&Session> = label_entry.sessions.iter().collect();
                sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

                for session in sessions {
                    println!("  {} {}", "•".green(), session.session_id);
                    println!("    Path: {}", session.path.dimmed());
                    println!(
                        "    Created: {}",
                        session
                            .created_at
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                            .dimmed()
                    );
                    if let Some(ref desc) = session.description {
                        println!("    Description: {}", desc);
                    }
                    println!();
                }
            }
        }
        None => {
            if store.labels.is_empty() {
                println!("No labels found.");
                println!(
                    "\nUse {} to add a session.",
                    "claude-sessions add <label> <session-id>".cyan()
                );
                return Ok(());
            }

            let mut labels: Vec<(&String, &crate::data::Label)> = store.labels.iter().collect();
            labels.sort_by_key(|(name, _)| *name);

            for (name, label_entry) in labels {
                let session_count = label_entry.sessions.len();
                let desc = label_entry
                    .description
                    .as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();

                println!(
                    "{} ({} session{}){}",
                    name.cyan().bold(),
                    session_count,
                    if session_count == 1 { "" } else { "s" },
                    desc.dimmed()
                );
            }
        }
    }

    Ok(())
}

pub fn remove(storage: &Storage, label: &str, session_id: Option<&str>) -> Result<()> {
    let mut store = storage.load()?;

    match session_id {
        Some(sid) => {
            let label_entry = store
                .get_label_mut(label)
                .with_context(|| format!("Label '{}' not found", label))?;

            if label_entry.remove_session(sid) {
                storage.save(&store)?;
                println!(
                    "{} Removed session '{}' from label '{}'",
                    "✓".green(),
                    sid,
                    label
                );
            } else {
                bail!("Session '{}' not found in label '{}'", sid, label);
            }
        }
        None => {
            if store.remove_label(label) {
                storage.save(&store)?;
                println!("{} Removed label '{}'", "✓".green(), label);
            } else {
                bail!("Label '{}' not found", label);
            }
        }
    }

    Ok(())
}

pub fn describe(storage: &Storage, label: &str, description: Option<String>) -> Result<()> {
    let mut store = storage.load()?;

    let label_entry = store
        .get_label_mut(label)
        .with_context(|| format!("Label '{}' not found", label))?;

    label_entry.description = description.clone();
    storage.save(&store)?;

    match description {
        Some(desc) => println!(
            "{} Updated description for '{}': {}",
            "✓".green(),
            label.cyan(),
            desc
        ),
        None => println!("{} Cleared description for '{}'", "✓".green(), label.cyan()),
    }

    Ok(())
}

pub fn config(storage: &Storage) -> Result<()> {
    println!("{}", "Configuration".cyan().bold());
    println!("  Data file: {}", storage.path().display());
    Ok(())
}
