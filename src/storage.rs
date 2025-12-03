use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::data::Store;

pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("", "", "claude-sessions")
            .context("Could not determine config directory")?;

        let config_dir = project_dirs.config_dir();
        fs::create_dir_all(config_dir)
            .with_context(|| format!("Could not create config directory: {:?}", config_dir))?;

        let path = config_dir.join("data.json");

        Ok(Self { path })
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn load(&self) -> Result<Store> {
        if !self.path.exists() {
            return Ok(Store::new());
        }

        let content = fs::read_to_string(&self.path)
            .with_context(|| format!("Could not read data file: {:?}", self.path))?;

        if content.trim().is_empty() {
            return Ok(Store::new());
        }

        serde_json::from_str(&content)
            .with_context(|| format!("Could not parse data file: {:?}", self.path))
    }

    pub fn save(&self, store: &Store) -> Result<()> {
        let content = serde_json::to_string_pretty(store).context("Could not serialize store")?;

        fs::write(&self.path, content)
            .with_context(|| format!("Could not write data file: {:?}", self.path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Label, Session};
    use chrono::Utc;
    use std::env;

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "claude-sessions-test-{}-{}",
            name,
            std::process::id()
        ))
    }

    fn create_test_session(id: &str) -> Session {
        Session {
            session_id: id.to_string(),
            path: "/test/path".to_string(),
            description: None,
            created_at: Utc::now(),
        }
    }

    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    // ==================== Load Tests ====================

    #[test]
    fn test_load_missing_file_returns_empty_store() {
        let path = temp_path("missing");
        cleanup(&path); // Ensure file doesn't exist

        let storage = Storage::with_path(path.clone());
        let store = storage.load().unwrap();

        assert!(store.labels.is_empty());
        cleanup(&path);
    }

    #[test]
    fn test_load_empty_file_returns_empty_store() {
        let path = temp_path("empty");
        fs::write(&path, "").unwrap();

        let storage = Storage::with_path(path.clone());
        let store = storage.load().unwrap();

        assert!(store.labels.is_empty());
        cleanup(&path);
    }

    #[test]
    fn test_load_whitespace_only_file_returns_empty_store() {
        let path = temp_path("whitespace");
        fs::write(&path, "   \n\t  \n  ").unwrap();

        let storage = Storage::with_path(path.clone());
        let store = storage.load().unwrap();

        assert!(store.labels.is_empty());
        cleanup(&path);
    }

    #[test]
    fn test_load_valid_json() {
        let path = temp_path("valid");
        let json = r#"{"labels":{"my-label":{"description":"Test","sessions":[]}}}"#;
        fs::write(&path, json).unwrap();

        let storage = Storage::with_path(path.clone());
        let store = storage.load().unwrap();

        assert!(store.labels.contains_key("my-label"));
        assert_eq!(
            store.get_label("my-label").unwrap().description,
            Some("Test".to_string())
        );
        cleanup(&path);
    }

    #[test]
    fn test_load_corrupt_json_returns_error() {
        let path = temp_path("corrupt");
        fs::write(&path, "{ not valid json }").unwrap();

        let storage = Storage::with_path(path.clone());
        let result = storage.load();

        assert!(result.is_err());
        cleanup(&path);
    }

    #[test]
    fn test_load_partial_json_returns_error() {
        let path = temp_path("partial");
        fs::write(&path, r#"{"labels":{"#).unwrap();

        let storage = Storage::with_path(path.clone());
        let result = storage.load();

        assert!(result.is_err());
        cleanup(&path);
    }

    // ==================== Save Tests ====================

    #[test]
    fn test_save_creates_file() {
        let path = temp_path("create");
        cleanup(&path); // Ensure file doesn't exist

        let storage = Storage::with_path(path.clone());
        let store = Store::new();
        storage.save(&store).unwrap();

        assert!(path.exists());
        cleanup(&path);
    }

    #[test]
    fn test_save_writes_valid_json() {
        let path = temp_path("write-json");

        let storage = Storage::with_path(path.clone());
        let mut store = Store::new();
        let mut label = Label::new(Some("My description".to_string()));
        label.add_session(create_test_session("test-session"));
        store.labels.insert("test-label".to_string(), label);

        storage.save(&store).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let loaded: Store = serde_json::from_str(&content).unwrap();

        assert!(loaded.labels.contains_key("test-label"));
        assert_eq!(
            loaded.get_label("test-label").unwrap().description,
            Some("My description".to_string())
        );
        assert_eq!(loaded.get_label("test-label").unwrap().sessions.len(), 1);
        cleanup(&path);
    }

    #[test]
    fn test_save_overwrites_existing_file() {
        let path = temp_path("overwrite");

        // Write initial content
        fs::write(&path, "old content").unwrap();

        let storage = Storage::with_path(path.clone());
        let store = Store::new();
        storage.save(&store).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("labels"));
        assert!(!content.contains("old content"));
        cleanup(&path);
    }

    // ==================== Roundtrip Tests ====================

    #[test]
    fn test_save_load_roundtrip_empty_store() {
        let path = temp_path("roundtrip-empty");

        let storage = Storage::with_path(path.clone());
        let store = Store::new();

        storage.save(&store).unwrap();
        let loaded = storage.load().unwrap();

        assert!(loaded.labels.is_empty());
        cleanup(&path);
    }

    #[test]
    fn test_save_load_roundtrip_with_data() {
        let path = temp_path("roundtrip-data");

        let storage = Storage::with_path(path.clone());

        // Create store with multiple labels and sessions
        let mut store = Store::new();

        let mut label1 = Label::new(Some("First label".to_string()));
        label1.add_session(create_test_session("sess1"));
        label1.add_session(create_test_session("sess2"));

        let mut label2 = Label::new(None);
        label2.add_session(create_test_session("sess3"));

        store.labels.insert("label-1".to_string(), label1);
        store.labels.insert("label-2".to_string(), label2);

        // Save and load
        storage.save(&store).unwrap();
        let loaded = storage.load().unwrap();

        // Verify
        assert_eq!(loaded.labels.len(), 2);

        let loaded_label1 = loaded.get_label("label-1").unwrap();
        assert_eq!(loaded_label1.description, Some("First label".to_string()));
        assert_eq!(loaded_label1.sessions.len(), 2);

        let loaded_label2 = loaded.get_label("label-2").unwrap();
        assert_eq!(loaded_label2.description, None);
        assert_eq!(loaded_label2.sessions.len(), 1);

        cleanup(&path);
    }

    // ==================== Path Tests ====================

    #[test]
    fn test_path_returns_configured_path() {
        let expected = PathBuf::from("/custom/path/data.json");
        let storage = Storage::with_path(expected.clone());

        assert_eq!(storage.path(), &expected);
    }

    #[test]
    fn test_storage_new_creates_valid_path() {
        // This test verifies Storage::new() works without errors
        let storage = Storage::new();
        assert!(storage.is_ok());

        let storage = storage.unwrap();
        let path = storage.path();
        assert!(path.ends_with("data.json"));
    }
}
