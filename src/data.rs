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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_session(id: &str, path: &str, desc: Option<&str>) -> Session {
        Session {
            session_id: id.to_string(),
            path: path.to_string(),
            description: desc.map(|s| s.to_string()),
            created_at: Utc::now(),
        }
    }

    fn create_session_with_time(id: &str, year: i32, month: u32, day: u32) -> Session {
        Session {
            session_id: id.to_string(),
            path: "/test/path".to_string(),
            description: None,
            created_at: Utc.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap(),
        }
    }

    // ==================== Session Tests ====================

    #[test]
    fn test_session_creation() {
        let session = create_test_session("abc123", "/home/user/project", Some("Test session"));

        assert_eq!(session.session_id, "abc123");
        assert_eq!(session.path, "/home/user/project");
        assert_eq!(session.description, Some("Test session".to_string()));
    }

    #[test]
    fn test_session_without_description() {
        let session = create_test_session("xyz789", "/tmp/test", None);

        assert_eq!(session.session_id, "xyz789");
        assert_eq!(session.path, "/tmp/test");
        assert_eq!(session.description, None);
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let session = create_test_session("sess123", "/project", Some("My session"));

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.session_id, session.session_id);
        assert_eq!(deserialized.path, session.path);
        assert_eq!(deserialized.description, session.description);
        assert_eq!(deserialized.created_at, session.created_at);
    }

    #[test]
    fn test_session_serialization_skips_none_description() {
        let session = create_test_session("sess123", "/project", None);

        let json = serde_json::to_string(&session).unwrap();

        assert!(!json.contains("description"));
    }

    // ==================== Label Tests ====================

    #[test]
    fn test_label_new_without_description() {
        let label = Label::new(None);

        assert!(label.sessions.is_empty());
        assert_eq!(label.description, None);
    }

    #[test]
    fn test_label_new_with_description() {
        let label = Label::new(Some("Test label".to_string()));

        assert!(label.sessions.is_empty());
        assert_eq!(label.description, Some("Test label".to_string()));
    }

    #[test]
    fn test_label_add_session() {
        let mut label = Label::new(None);
        let session = create_test_session("sess1", "/path1", None);

        label.add_session(session);

        assert_eq!(label.sessions.len(), 1);
        assert_eq!(label.sessions[0].session_id, "sess1");
    }

    #[test]
    fn test_label_add_multiple_sessions() {
        let mut label = Label::new(None);

        label.add_session(create_test_session("sess1", "/path1", None));
        label.add_session(create_test_session("sess2", "/path2", None));
        label.add_session(create_test_session("sess3", "/path3", None));

        assert_eq!(label.sessions.len(), 3);
    }

    #[test]
    fn test_label_latest_session_empty() {
        let label = Label::new(None);

        assert!(label.latest_session().is_none());
    }

    #[test]
    fn test_label_latest_session_single() {
        let mut label = Label::new(None);
        label.add_session(create_test_session("only-session", "/path", None));

        let latest = label.latest_session().unwrap();
        assert_eq!(latest.session_id, "only-session");
    }

    #[test]
    fn test_label_latest_session_returns_most_recent() {
        let mut label = Label::new(None);

        // Add sessions with different timestamps (oldest first)
        label.add_session(create_session_with_time("old", 2023, 1, 1));
        label.add_session(create_session_with_time("newest", 2024, 6, 15));
        label.add_session(create_session_with_time("middle", 2024, 3, 10));

        let latest = label.latest_session().unwrap();
        assert_eq!(latest.session_id, "newest");
    }

    #[test]
    fn test_label_remove_session_success() {
        let mut label = Label::new(None);
        label.add_session(create_test_session("sess1", "/path1", None));
        label.add_session(create_test_session("sess2", "/path2", None));

        let removed = label.remove_session("sess1");

        assert!(removed);
        assert_eq!(label.sessions.len(), 1);
        assert_eq!(label.sessions[0].session_id, "sess2");
    }

    #[test]
    fn test_label_remove_session_not_found() {
        let mut label = Label::new(None);
        label.add_session(create_test_session("sess1", "/path1", None));

        let removed = label.remove_session("nonexistent");

        assert!(!removed);
        assert_eq!(label.sessions.len(), 1);
    }

    #[test]
    fn test_label_remove_session_from_empty() {
        let mut label = Label::new(None);

        let removed = label.remove_session("any");

        assert!(!removed);
    }

    #[test]
    fn test_label_serialization_roundtrip() {
        let mut label = Label::new(Some("My label".to_string()));
        label.add_session(create_test_session("sess1", "/path1", Some("First")));
        label.add_session(create_test_session("sess2", "/path2", None));

        let json = serde_json::to_string(&label).unwrap();
        let deserialized: Label = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.description, label.description);
        assert_eq!(deserialized.sessions.len(), 2);
        assert_eq!(deserialized.sessions[0].session_id, "sess1");
        assert_eq!(deserialized.sessions[1].session_id, "sess2");
    }

    // ==================== Store Tests ====================

    #[test]
    fn test_store_new_is_empty() {
        let store = Store::new();

        assert!(store.labels.is_empty());
    }

    #[test]
    fn test_store_default_is_empty() {
        let store = Store::default();

        assert!(store.labels.is_empty());
    }

    #[test]
    fn test_store_get_label_not_found() {
        let store = Store::new();

        assert!(store.get_label("nonexistent").is_none());
    }

    #[test]
    fn test_store_get_label_found() {
        let mut store = Store::new();
        store
            .labels
            .insert("my-label".to_string(), Label::new(Some("Test".to_string())));

        let label = store.get_label("my-label").unwrap();
        assert_eq!(label.description, Some("Test".to_string()));
    }

    #[test]
    fn test_store_get_label_mut() {
        let mut store = Store::new();
        store
            .labels
            .insert("my-label".to_string(), Label::new(None));

        let label = store.get_label_mut("my-label").unwrap();
        label.description = Some("Modified".to_string());

        assert_eq!(
            store.get_label("my-label").unwrap().description,
            Some("Modified".to_string())
        );
    }

    #[test]
    fn test_store_get_or_create_label_creates_new() {
        let mut store = Store::new();

        let label = store.get_or_create_label("new-label");
        label.description = Some("Created".to_string());

        assert!(store.labels.contains_key("new-label"));
        assert_eq!(
            store.get_label("new-label").unwrap().description,
            Some("Created".to_string())
        );
    }

    #[test]
    fn test_store_get_or_create_label_returns_existing() {
        let mut store = Store::new();
        store.labels.insert(
            "existing".to_string(),
            Label::new(Some("Original".to_string())),
        );

        let label = store.get_or_create_label("existing");

        assert_eq!(label.description, Some("Original".to_string()));
    }

    #[test]
    fn test_store_remove_label_success() {
        let mut store = Store::new();
        store
            .labels
            .insert("to-remove".to_string(), Label::new(None));

        let removed = store.remove_label("to-remove");

        assert!(removed);
        assert!(!store.labels.contains_key("to-remove"));
    }

    #[test]
    fn test_store_remove_label_not_found() {
        let mut store = Store::new();

        let removed = store.remove_label("nonexistent");

        assert!(!removed);
    }

    #[test]
    fn test_store_serialization_roundtrip() {
        let mut store = Store::new();

        let mut label1 = Label::new(Some("Label 1".to_string()));
        label1.add_session(create_test_session("s1", "/p1", None));

        let mut label2 = Label::new(None);
        label2.add_session(create_test_session("s2", "/p2", Some("Session 2")));

        store.labels.insert("label-1".to_string(), label1);
        store.labels.insert("label-2".to_string(), label2);

        let json = serde_json::to_string(&store).unwrap();
        let deserialized: Store = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.labels.len(), 2);
        assert!(deserialized.labels.contains_key("label-1"));
        assert!(deserialized.labels.contains_key("label-2"));
        assert_eq!(
            deserialized.get_label("label-1").unwrap().description,
            Some("Label 1".to_string())
        );
    }

    #[test]
    fn test_store_empty_serialization() {
        let store = Store::new();

        let json = serde_json::to_string(&store).unwrap();
        let deserialized: Store = serde_json::from_str(&json).unwrap();

        assert!(deserialized.labels.is_empty());
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_session_id() {
        let session = create_test_session("", "/path", None);
        assert_eq!(session.session_id, "");

        // Should serialize and deserialize correctly
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "");
    }

    #[test]
    fn test_empty_label_name() {
        let mut store = Store::new();
        store.labels.insert("".to_string(), Label::new(None));

        assert!(store.labels.contains_key(""));
        assert!(store.get_label("").is_some());
    }

    #[test]
    fn test_unicode_in_session_id() {
        let session = create_test_session("セッション-123-émoji-🎉", "/path", None);

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.session_id, "セッション-123-émoji-🎉");
    }

    #[test]
    fn test_unicode_in_label_name() {
        let mut store = Store::new();
        store
            .labels
            .insert("功能-фича-🚀".to_string(), Label::new(None));

        let json = serde_json::to_string(&store).unwrap();
        let deserialized: Store = serde_json::from_str(&json).unwrap();

        assert!(deserialized.labels.contains_key("功能-фича-🚀"));
    }

    #[test]
    fn test_unicode_in_description() {
        let session = create_test_session("sess1", "/path", Some("日本語の説明 • Emoji: 💻🔧"));

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.description,
            Some("日本語の説明 • Emoji: 💻🔧".to_string())
        );
    }

    #[test]
    fn test_unicode_in_path() {
        let session = create_test_session("sess1", "/home/用户/проект/código", None);

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.path, "/home/用户/проект/código");
    }

    #[test]
    fn test_very_long_session_history() {
        let mut label = Label::new(None);

        // Add 1000 sessions
        for i in 0..1000 {
            label.add_session(create_test_session(
                &format!("session-{}", i),
                &format!("/path/{}", i),
                None,
            ));
        }

        assert_eq!(label.sessions.len(), 1000);

        // Should serialize and deserialize correctly
        let json = serde_json::to_string(&label).unwrap();
        let deserialized: Label = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sessions.len(), 1000);
    }

    #[test]
    fn test_latest_session_with_many_sessions() {
        let mut label = Label::new(None);

        // Add sessions with timestamps from 2020 to 2024
        for year in 2020..=2024 {
            for month in 1..=12 {
                label.add_session(Session {
                    session_id: format!("session-{}-{}", year, month),
                    path: "/test".to_string(),
                    description: None,
                    created_at: Utc.with_ymd_and_hms(year, month, 15, 12, 0, 0).unwrap(),
                });
            }
        }

        let latest = label.latest_session().unwrap();
        assert_eq!(latest.session_id, "session-2024-12");
    }

    #[test]
    fn test_special_characters_in_description() {
        let session = create_test_session(
            "sess1",
            "/path",
            Some("Line1\nLine2\tTabbed \"quoted\" 'single' \\backslash"),
        );

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.description,
            Some("Line1\nLine2\tTabbed \"quoted\" 'single' \\backslash".to_string())
        );
    }

    #[test]
    fn test_very_long_description() {
        let long_desc = "a".repeat(10000);
        let session = create_test_session("sess1", "/path", Some(&long_desc));

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.description.unwrap().len(), 10000);
    }

    #[test]
    fn test_many_labels_in_store() {
        let mut store = Store::new();

        // Add 100 labels, each with 10 sessions
        for i in 0..100 {
            let mut label = Label::new(Some(format!("Label {}", i)));
            for j in 0..10 {
                label.add_session(create_test_session(
                    &format!("sess-{}-{}", i, j),
                    "/path",
                    None,
                ));
            }
            store.labels.insert(format!("label-{}", i), label);
        }

        assert_eq!(store.labels.len(), 100);

        let json = serde_json::to_string(&store).unwrap();
        let deserialized: Store = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.labels.len(), 100);
        assert_eq!(
            deserialized.get_label("label-50").unwrap().sessions.len(),
            10
        );
    }

    #[test]
    fn test_label_with_special_chars_in_name() {
        let mut store = Store::new();

        // Labels with various special characters
        let special_labels = vec![
            "label-with-dash",
            "label_with_underscore",
            "label.with.dots",
            "label:with:colons",
            "label/with/slashes",
            "label@with@at",
            "label#with#hash",
            "TICKET-1234",
            "feature/auth-system",
        ];

        for label_name in &special_labels {
            store
                .labels
                .insert(label_name.to_string(), Label::new(None));
        }

        let json = serde_json::to_string(&store).unwrap();
        let deserialized: Store = serde_json::from_str(&json).unwrap();

        for label_name in &special_labels {
            assert!(
                deserialized.labels.contains_key(*label_name),
                "Missing label: {}",
                label_name
            );
        }
    }

    #[test]
    fn test_remove_from_label_with_duplicate_session_ids() {
        // Edge case: what happens if someone manually edits the JSON to have duplicate session IDs?
        let mut label = Label::new(None);
        label.add_session(create_test_session("duplicate-id", "/path1", None));
        label.add_session(create_test_session("duplicate-id", "/path2", None));
        label.add_session(create_test_session("unique-id", "/path3", None));

        assert_eq!(label.sessions.len(), 3);

        // remove_session should remove ALL sessions with that ID
        let removed = label.remove_session("duplicate-id");

        assert!(removed);
        assert_eq!(label.sessions.len(), 1);
        assert_eq!(label.sessions[0].session_id, "unique-id");
    }

    #[test]
    fn test_whitespace_in_values() {
        let session = create_test_session(
            "  session-with-spaces  ",
            "  /path/with/spaces  ",
            Some("  description with spaces  "),
        );

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        // Whitespace should be preserved
        assert_eq!(deserialized.session_id, "  session-with-spaces  ");
        assert_eq!(deserialized.path, "  /path/with/spaces  ");
        assert_eq!(
            deserialized.description,
            Some("  description with spaces  ".to_string())
        );
    }
}
