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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Label, Store};
    use chrono::{TimeZone, Utc};
    use std::fs;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "claude-sessions-cmd-test-{}-{}",
            name,
            std::process::id()
        ))
    }

    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    fn create_test_storage(name: &str) -> (Storage, PathBuf) {
        let path = temp_path(name);
        cleanup(&path);
        let storage = Storage::with_path(path.clone());
        (storage, path)
    }

    fn create_test_session_with_time(
        id: &str,
        path: &str,
        desc: Option<&str>,
        year: i32,
        month: u32,
        day: u32,
    ) -> Session {
        Session {
            session_id: id.to_string(),
            path: path.to_string(),
            description: desc.map(|s| s.to_string()),
            created_at: Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
        }
    }

    // ==================== Add Command Tests ====================

    #[test]
    fn test_add_creates_new_label() {
        let (storage, path) = create_test_storage("add-new-label");

        let result = add(&storage, "my-label", "session-123", None);
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        assert!(store.labels.contains_key("my-label"));

        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.sessions.len(), 1);
        assert_eq!(label.sessions[0].session_id, "session-123");

        cleanup(&path);
    }

    #[test]
    fn test_add_appends_to_existing_label() {
        let (storage, path) = create_test_storage("add-append");

        // Add first session
        add(&storage, "my-label", "session-1", None).unwrap();

        // Add second session to same label
        add(
            &storage,
            "my-label",
            "session-2",
            Some("Second session".to_string()),
        )
        .unwrap();

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.sessions.len(), 2);

        cleanup(&path);
    }

    #[test]
    fn test_add_with_description() {
        let (storage, path) = create_test_storage("add-desc");

        add(
            &storage,
            "my-label",
            "session-123",
            Some("Test description".to_string()),
        )
        .unwrap();

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(
            label.sessions[0].description,
            Some("Test description".to_string())
        );

        cleanup(&path);
    }

    #[test]
    fn test_add_saves_current_directory() {
        let (storage, path) = create_test_storage("add-path");

        add(&storage, "my-label", "session-123", None).unwrap();

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();

        // The path should be the current working directory
        let current_dir = env::current_dir().unwrap().to_string_lossy().to_string();
        assert_eq!(label.sessions[0].path, current_dir);

        cleanup(&path);
    }

    // ==================== List Command Tests ====================

    #[test]
    fn test_list_empty_store() {
        let (storage, path) = create_test_storage("list-empty");

        // Should not error on empty store
        let result = list(&storage, None);
        assert!(result.is_ok());

        cleanup(&path);
    }

    #[test]
    fn test_list_all_labels() {
        let (storage, path) = create_test_storage("list-all");

        add(&storage, "label-1", "sess-1", None).unwrap();
        add(&storage, "label-2", "sess-2", None).unwrap();

        let result = list(&storage, None);
        assert!(result.is_ok());

        cleanup(&path);
    }

    #[test]
    fn test_list_specific_label() {
        let (storage, path) = create_test_storage("list-specific");

        add(&storage, "my-label", "sess-1", None).unwrap();

        let result = list(&storage, Some("my-label"));
        assert!(result.is_ok());

        cleanup(&path);
    }

    #[test]
    fn test_list_nonexistent_label_returns_error() {
        let (storage, path) = create_test_storage("list-nonexistent");

        let result = list(&storage, Some("nonexistent"));
        assert!(result.is_err());

        cleanup(&path);
    }

    // ==================== Remove Command Tests ====================

    #[test]
    fn test_remove_entire_label() {
        let (storage, path) = create_test_storage("remove-label");

        add(&storage, "my-label", "sess-1", None).unwrap();

        let result = remove(&storage, "my-label", None);
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        assert!(!store.labels.contains_key("my-label"));

        cleanup(&path);
    }

    #[test]
    fn test_remove_specific_session() {
        let (storage, path) = create_test_storage("remove-session");

        add(&storage, "my-label", "sess-1", None).unwrap();
        add(&storage, "my-label", "sess-2", None).unwrap();

        let result = remove(&storage, "my-label", Some("sess-1"));
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.sessions.len(), 1);
        assert_eq!(label.sessions[0].session_id, "sess-2");

        cleanup(&path);
    }

    #[test]
    fn test_remove_nonexistent_label_returns_error() {
        let (storage, path) = create_test_storage("remove-nonexistent-label");

        let result = remove(&storage, "nonexistent", None);
        assert!(result.is_err());

        cleanup(&path);
    }

    #[test]
    fn test_remove_nonexistent_session_returns_error() {
        let (storage, path) = create_test_storage("remove-nonexistent-session");

        add(&storage, "my-label", "sess-1", None).unwrap();

        let result = remove(&storage, "my-label", Some("nonexistent"));
        assert!(result.is_err());

        cleanup(&path);
    }

    // ==================== Describe Command Tests ====================

    #[test]
    fn test_describe_set_description() {
        let (storage, path) = create_test_storage("describe-set");

        add(&storage, "my-label", "sess-1", None).unwrap();

        let result = describe(&storage, "my-label", Some("New description".to_string()));
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.description, Some("New description".to_string()));

        cleanup(&path);
    }

    #[test]
    fn test_describe_update_description() {
        let (storage, path) = create_test_storage("describe-update");

        add(&storage, "my-label", "sess-1", None).unwrap();
        describe(&storage, "my-label", Some("First".to_string())).unwrap();

        let result = describe(&storage, "my-label", Some("Updated".to_string()));
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.description, Some("Updated".to_string()));

        cleanup(&path);
    }

    #[test]
    fn test_describe_clear_description() {
        let (storage, path) = create_test_storage("describe-clear");

        add(&storage, "my-label", "sess-1", None).unwrap();
        describe(&storage, "my-label", Some("Has description".to_string())).unwrap();

        let result = describe(&storage, "my-label", None);
        assert!(result.is_ok());

        let store = storage.load().unwrap();
        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.description, None);

        cleanup(&path);
    }

    #[test]
    fn test_describe_nonexistent_label_returns_error() {
        let (storage, path) = create_test_storage("describe-nonexistent");

        let result = describe(&storage, "nonexistent", Some("Description".to_string()));
        assert!(result.is_err());

        cleanup(&path);
    }

    // ==================== Config Command Tests ====================

    #[test]
    fn test_config_returns_ok() {
        let (storage, path) = create_test_storage("config");

        let result = config(&storage);
        assert!(result.is_ok());

        cleanup(&path);
    }

    // ==================== Resume Command Tests ====================
    // Note: We can't fully test resume() because it executes the external `claude` command.
    // Instead, we test the error conditions and preconditions.

    #[test]
    fn test_resume_nonexistent_label_returns_error() {
        let (storage, path) = create_test_storage("resume-nonexistent");

        let result = resume(&storage, "nonexistent", false);
        assert!(result.is_err());

        cleanup(&path);
    }

    #[test]
    fn test_resume_empty_label_returns_error() {
        let (storage, path) = create_test_storage("resume-empty");

        // Create a label with no sessions (directly manipulating the store)
        let mut store = Store::new();
        store
            .labels
            .insert("empty-label".to_string(), Label::new(None));
        storage.save(&store).unwrap();

        let result = resume(&storage, "empty-label", false);
        assert!(result.is_err());

        cleanup(&path);
    }

    // ==================== Pick Session Tests ====================
    // Note: pick_session() is interactive and can't be easily unit tested.
    // We test the supporting logic through the Label's latest_session() method.

    #[test]
    fn test_latest_session_is_selected_by_default() {
        let (storage, path) = create_test_storage("latest-session");

        // We need to manually create sessions with different timestamps
        let mut store = Store::new();
        let mut label = Label::new(None);

        label.add_session(create_test_session_with_time(
            "old-session",
            "/path1",
            None,
            2023,
            1,
            1,
        ));
        label.add_session(create_test_session_with_time(
            "new-session",
            "/path2",
            None,
            2024,
            6,
            15,
        ));
        label.add_session(create_test_session_with_time(
            "middle-session",
            "/path3",
            None,
            2024,
            3,
            10,
        ));

        store.labels.insert("my-label".to_string(), label);
        storage.save(&store).unwrap();

        // Verify that latest_session() returns the newest one
        let loaded = storage.load().unwrap();
        let label = loaded.get_label("my-label").unwrap();
        let latest = label.latest_session().unwrap();

        assert_eq!(latest.session_id, "new-session");

        cleanup(&path);
    }
}
