use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub sessions: Vec<Session>,
}

impl Label {
    pub fn new(description: Option<String>) -> Self {
        Self {
            description,
            sessions: Vec::new(),
        }
    }

    pub fn add_session(&mut self, session: Session) {
        self.sessions.push(session);
    }

    pub fn latest_session(&self) -> Option<&Session> {
        self.sessions.iter().max_by_key(|s| s.created_at)
    }

    pub fn remove_session(&mut self, session_id: &str) -> bool {
        let len_before = self.sessions.len();
        self.sessions.retain(|s| s.session_id != session_id);
        self.sessions.len() < len_before
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    pub labels: HashMap<String, Label>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    pub fn get_label(&self, name: &str) -> Option<&Label> {
        self.labels.get(name)
    }

    pub fn get_label_mut(&mut self, name: &str) -> Option<&mut Label> {
        self.labels.get_mut(name)
    }

    pub fn get_or_create_label(&mut self, name: &str) -> &mut Label {
        self.labels
            .entry(name.to_string())
            .or_insert_with(|| Label::new(None))
    }

    pub fn remove_label(&mut self, name: &str) -> bool {
        self.labels.remove(name).is_some()
    }
}
