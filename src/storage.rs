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
        let content = serde_json::to_string_pretty(store)
            .context("Could not serialize store")?;

        fs::write(&self.path, content)
            .with_context(|| format!("Could not write data file: {:?}", self.path))?;

        Ok(())
    }
}
