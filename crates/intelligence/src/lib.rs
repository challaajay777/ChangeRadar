use changeradar_database::{Database, DatabaseError};
use changeradar_types::{AnomalyAlert, ChangeKind, Priority, SystemOverview};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStats {
    pub daily_changes: Vec<DailyChangeCount>,
    pub folder_breakdown: Vec<FolderActivityCount>,
    pub top_modified_files: Vec<TopModifiedFile>,
    pub total_anomalies_detected: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyChangeCount {
    pub date: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderActivityCount {
    pub folder_id: String,
    pub folder_name: String,
    pub change_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopModifiedFile {
    pub file_id: String,
    pub file_name: String,
    pub relative_path: String,
    pub version_count: u64,
}

#[derive(Clone)]
pub struct IntelligenceEngine {
    db: Database,
}

impl IntelligenceEngine {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Calculate system overview metrics and transparent Change Radar Score (0 - 100).
    pub fn get_overview(&self, is_monitoring: bool) -> Result<SystemOverview, IntelligenceError> {
        let folders = self.db.list_folders()?;
        let active_folders = folders.iter().filter(|f| f.is_active).count();
        let total_files = self.db.count_tracked_files()?;
        let total_versions = self.db.count_total_versions()?;
        let changes_today = self.db.count_events_today()?;
        let storage_stats = self.db.get_storage_stats()?;

        let anomalies = self.detect_anomalies()?.len();

        // Transparent score calculation:
        // Base score = (changes_today * 4) + (active_folders * 6) + (anomalies * 25)
        let raw_score = (changes_today * 4) + ((active_folders as u64) * 6) + ((anomalies as u64) * 25);
        let radar_score = (raw_score.min(100)) as u32;

        let savings_pct = if storage_stats.raw_bytes > 0 && storage_stats.compressed_bytes > 0 {
            let diff = (storage_stats.raw_bytes as f64) - (storage_stats.compressed_bytes as f64);
            (diff / (storage_stats.raw_bytes as f64) * 100.0).max(0.0)
        } else {
            0.0
        };

        Ok(SystemOverview {
            is_monitoring,
            active_folders,
            total_tracked_files: total_files,
            changes_today,
            versions_stored: total_versions,
            total_cas_bytes: storage_stats.compressed_bytes,
            uncompressed_bytes: storage_stats.raw_bytes,
            deduplication_savings_pct: (savings_pct * 10.0).round() / 10.0,
            radar_score,
        })
    }

    /// Analyze historical activity patterns.
    pub fn get_activity_stats(&self) -> Result<ActivityStats, IntelligenceError> {
        let events = self.db.list_events(None, 2000, 0)?;
        let folders = self.db.list_folders()?;
        let folder_map: HashMap<String, String> = folders
            .into_iter()
            .map(|f| (f.id, f.display_name))
            .collect();

        // 1. Group events by date (last 7 days)
        let mut daily_counts: BTreeMap<String, u64> = BTreeMap::new();
        for i in 0..7 {
            let day = (Utc::now() - Duration::days(i)).format("%Y-%m-%d").to_string();
            daily_counts.insert(day, 0);
        }

        let mut folder_counts: HashMap<String, u64> = HashMap::new();

        for ev in &events {
            let day = ev.created_at.format("%Y-%m-%d").to_string();
            if let Some(c) = daily_counts.get_mut(&day) {
                *c += 1;
            }
            *folder_counts.entry(ev.folder_id.clone()).or_insert(0) += 1;
        }

        let daily_changes = daily_counts
            .into_iter()
            .map(|(date, count)| DailyChangeCount { date, count })
            .collect();

        let folder_breakdown = folder_counts
            .into_iter()
            .map(|(folder_id, change_count)| {
                let folder_name = folder_map
                    .get(&folder_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                FolderActivityCount {
                    folder_id,
                    folder_name,
                    change_count,
                }
            })
            .collect();

        let files = self.db.list_files(None, 50)?;
        let mut top_modified = Vec::new();
        for f in files {
            let versions = self.db.list_versions_for_file(&f.id)?;
            if versions.len() > 1 {
                top_modified.push(TopModifiedFile {
                    file_id: f.id,
                    file_name: f.file_name,
                    relative_path: f.relative_path,
                    version_count: versions.len() as u64,
                });
            }
        }
        top_modified.sort_by(|a, b| b.version_count.cmp(&a.version_count));
        top_modified.truncate(10);

        let anomalies = self.detect_anomalies()?;

        Ok(ActivityStats {
            daily_changes,
            folder_breakdown,
            top_modified_files: top_modified,
            total_anomalies_detected: anomalies.len(),
        })
    }

    /// Detect unusual filesystem activity heuristics:
    /// - Sudden deletion burst (> 5 deletions in the last 15 minutes)
    /// - Massive file size change (> 50 MB in a single modification)
    /// - Rapid modification burst (> 20 modifications in 5 minutes)
    pub fn detect_anomalies(&self) -> Result<Vec<AnomalyAlert>, IntelligenceError> {
        let events = self.db.list_events(None, 500, 0)?;
        let mut alerts = Vec::new();

        let recent_window = Utc::now() - Duration::minutes(15);
        let recent_events: Vec<&changeradar_types::FileEvent> = events
            .iter()
            .filter(|e| e.created_at >= recent_window)
            .collect();

        // 1. Deletion bursts
        let deletion_count = recent_events
            .iter()
            .filter(|e| e.event_type == ChangeKind::Deleted)
            .count();
        if deletion_count >= 5 {
            alerts.push(AnomalyAlert {
                id: Uuid::new_v4().to_string(),
                title: "Rapid Deletion Burst Detected".to_string(),
                description: format!(
                    "{} files were deleted in the last 15 minutes. Verify these deletions are expected.",
                    deletion_count
                ),
                severity: Priority::High,
                folder_id: None,
                detected_at: Utc::now(),
            });
        }

        // 2. Massive file change (> 50 MB)
        for ev in &recent_events {
            if ev.file_size_diff.abs() > 50 * 1024 * 1024 {
                alerts.push(AnomalyAlert {
                    id: Uuid::new_v4().to_string(),
                    title: "Large File Size Shift".to_string(),
                    description: format!(
                        "File '{}' experienced a size delta of {} MB.",
                        ev.new_path,
                        ev.file_size_diff / (1024 * 1024)
                    ),
                    severity: Priority::Normal,
                    folder_id: Some(ev.folder_id.clone()),
                    detected_at: ev.created_at,
                });
            }
        }

        // 3. Modification flood
        if recent_events.len() > 30 {
            alerts.push(AnomalyAlert {
                id: Uuid::new_v4().to_string(),
                title: "High Change Activity Spike".to_string(),
                description: format!(
                    "Observed {} change events in a 15-minute window.",
                    recent_events.len()
                ),
                severity: Priority::Normal,
                folder_id: None,
                detected_at: Utc::now(),
            });
        }

        Ok(alerts)
    }
}

use std::collections::BTreeMap;
