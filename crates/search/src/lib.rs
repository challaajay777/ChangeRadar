use changeradar_database::{Database, DatabaseError};
use changeradar_types::{FileEvent, FileRecord, MonitoredFolder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub matched_files: Vec<FileRecord>,
    pub matched_events: Vec<FileEvent>,
    pub matched_folders: Vec<MonitoredFolder>,
}

#[derive(Clone)]
pub struct SearchEngine {
    db: Database,
}

impl SearchEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Global unified search across files, events, and folders.
    pub fn search(&self, query: &str, limit: usize) -> Result<SearchResults, SearchError> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Ok(SearchResults {
                matched_files: Vec::new(),
                matched_events: Vec::new(),
                matched_folders: Vec::new(),
            });
        }

        // Search folders
        let folders = self.db.list_folders()?;
        let matched_folders: Vec<MonitoredFolder> = folders
            .into_iter()
            .filter(|f| {
                f.display_name.to_lowercase().contains(&q) || f.path.to_lowercase().contains(&q)
            })
            .collect();

        // Search files
        let files = self.db.list_files(None, 2000)?;
        let mut matched_files = Vec::new();
        for file in files {
            if file.file_name.to_lowercase().contains(&q)
                || file.relative_path.to_lowercase().contains(&q)
                || file.extension.to_lowercase().contains(&q)
                || file.current_hash.to_lowercase().starts_with(&q)
            {
                matched_files.push(file);
                if matched_files.len() >= limit {
                    break;
                }
            }
        }

        // Search events
        let events = self.db.list_events(None, 2000, 0)?;
        let mut matched_events = Vec::new();
        for ev in events {
            if ev.new_path.to_lowercase().contains(&q)
                || ev.summary.to_lowercase().contains(&q)
                || ev.event_type.to_string().to_lowercase().contains(&q)
            {
                matched_events.push(ev);
                if matched_events.len() >= limit {
                    break;
                }
            }
        }

        Ok(SearchResults {
            matched_files,
            matched_events,
            matched_folders,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_search_across_entities() {
        let db = Database::in_memory().unwrap();
        let folder =
            MonitoredFolder::new("C:/Project/Research".to_string(), "Research".to_string());
        db.add_folder(&folder).unwrap();

        let searcher = SearchEngine::new(db);
        let results = searcher.search("research", 10).unwrap();
        assert_eq!(results.matched_folders.len(), 1);
        assert_eq!(results.matched_folders[0].display_name, "Research");

        let empty = searcher.search("", 10).unwrap();
        assert!(empty.matched_files.is_empty());
        assert!(empty.matched_folders.is_empty());
    }
}
