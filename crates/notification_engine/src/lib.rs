use changeradar_database::{Database, DatabaseError};
use changeradar_types::{FileEvent, NotificationItem, Priority};
use chrono::Utc;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

#[derive(Clone)]
pub struct NotificationEngine {
    db: Database,
}

impl NotificationEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Evaluate rules for an incoming event and dispatch notifications if matched.
    pub fn evaluate_event(&self, event: &FileEvent) -> Result<Vec<NotificationItem>, NotificationError> {
        let rules = self.db.list_notification_rules()?;
        let mut dispatched = Vec::new();

        for rule in rules {
            if !rule.is_active {
                continue;
            }

            // Check folder filter
            if let Some(ref fid) = rule.folder_id {
                if fid != &event.folder_id {
                    continue;
                }
            }

            // Check event type filter
            if !rule.event_types.is_empty() && !rule.event_types.contains(&event.event_type) {
                continue;
            }

            // Check path pattern filter
            if let Some(ref pattern) = rule.path_pattern {
                if !event.new_path.contains(pattern) {
                    continue;
                }
            }

            // Check size filter
            if let Some(min_size) = rule.min_size_bytes {
                if event.file_size_diff.unsigned_abs() < min_size {
                    continue;
                }
            }

            let notif = NotificationItem {
                id: Uuid::new_v4().to_string(),
                rule_id: Some(rule.id),
                title: format!("Rule Alert: {}", rule.name),
                message: format!(
                    "{} on '{}': {}",
                    event.event_type, event.new_path, event.summary
                ),
                priority: rule.priority,
                is_read: false,
                created_at: Utc::now(),
            };

            self.db.create_notification(&notif)?;
            info!("Created notification: {}", notif.title);
            dispatched.push(notif);
        }

        Ok(dispatched)
    }

    /// Create a direct system notification (e.g. for snapshot, backup, or recovery results).
    pub fn notify_system(
        &self,
        title: &str,
        message: &str,
        priority: Priority,
    ) -> Result<NotificationItem, NotificationError> {
        let notif = NotificationItem {
            id: Uuid::new_v4().to_string(),
            rule_id: None,
            title: title.to_string(),
            message: message.to_string(),
            priority,
            is_read: false,
            created_at: Utc::now(),
        };
        self.db.create_notification(&notif)?;
        Ok(notif)
    }
}
