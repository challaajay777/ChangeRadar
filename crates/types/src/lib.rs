use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Logical classifications of filesystem change events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
    Replaced,
    MetadataChanged,
    PermissionsChanged,
    Restored,
    SnapshotCreated,
}

impl std::fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "CREATED"),
            Self::Modified => write!(f, "MODIFIED"),
            Self::Deleted => write!(f, "DELETED"),
            Self::Renamed => write!(f, "RENAMED"),
            Self::Moved => write!(f, "MOVED"),
            Self::Replaced => write!(f, "REPLACED"),
            Self::MetadataChanged => write!(f, "METADATA_CHANGED"),
            Self::PermissionsChanged => write!(f, "PERMISSIONS_CHANGED"),
            Self::Restored => write!(f, "RESTORED"),
            Self::SnapshotCreated => write!(f, "SNAPSHOT_CREATED"),
        }
    }
}

impl std::str::FromStr for ChangeKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CREATED" => Ok(Self::Created),
            "MODIFIED" => Ok(Self::Modified),
            "DELETED" => Ok(Self::Deleted),
            "RENAMED" => Ok(Self::Renamed),
            "MOVED" => Ok(Self::Moved),
            "REPLACED" => Ok(Self::Replaced),
            "METADATA_CHANGED" => Ok(Self::MetadataChanged),
            "PERMISSIONS_CHANGED" => Ok(Self::PermissionsChanged),
            "RESTORED" => Ok(Self::Restored),
            "SNAPSHOT_CREATED" => Ok(Self::SnapshotCreated),
            _ => Err(format!("Unknown change kind: {}", s)),
        }
    }
}

/// Confidence level for heuristic change classifications (such as rename / move matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confidence {
    Exact,
    Heuristic,
    Tentative,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "EXACT"),
            Self::Heuristic => write!(f, "HEURISTIC"),
            Self::Tentative => write!(f, "TENTATIVE"),
        }
    }
}

impl std::str::FromStr for Confidence {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "EXACT" => Ok(Self::Exact),
            "HEURISTIC" => Ok(Self::Heuristic),
            "TENTATIVE" => Ok(Self::Tentative),
            _ => Err(format!("Unknown confidence: {}", s)),
        }
    }
}

/// A folder monitored by ChangeRadar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredFolder {
    pub id: String,
    pub path: String,
    pub display_name: String,
    pub is_active: bool,
    pub scan_interval_secs: u64,
    pub ignore_patterns: Vec<String>,
    pub max_storage_bytes: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MonitoredFolder {
    pub fn new(path: String, display_name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            path,
            display_name,
            is_active: true,
            scan_interval_secs: 300,
            ignore_patterns: vec![
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                "*.tmp".to_string(),
                "*.swp".to_string(),
                "~$*".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            max_storage_bytes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A tracked file metadata entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub folder_id: String,
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub file_size: u64,
    pub current_hash: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A preserved historical version of a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVersion {
    pub id: String,
    pub file_id: String,
    pub version_number: u32,
    pub content_hash: String,
    pub file_size: u64,
    pub change_kind: ChangeKind,
    pub notes: Option<String>,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
}

/// An audit and timeline event for a file change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub id: String,
    pub folder_id: String,
    pub file_id: Option<String>,
    pub event_type: ChangeKind,
    pub old_path: Option<String>,
    pub new_path: String,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub file_size_diff: i64,
    pub summary: String,
    pub confidence: Confidence,
    pub created_at: DateTime<Utc>,
}

/// A deduplicated object stored in Content-Addressed Storage (CAS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentObject {
    pub content_hash: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_algorithm: String,
    pub reference_count: u32,
    pub created_at: DateTime<Utc>,
}

/// A folder or system snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub file_count: u64,
    pub total_bytes: u64,
    pub created_at: DateTime<Utc>,
}

/// An item inside a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotItem {
    pub snapshot_id: String,
    pub file_id: String,
    pub relative_path: String,
    pub version_id: String,
    pub content_hash: String,
    pub file_size: u64,
}

/// Notification priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "LOW"),
            Self::Normal => write!(f, "NORMAL"),
            Self::High => write!(f, "HIGH"),
            Self::Urgent => write!(f, "URGENT"),
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "LOW" => Ok(Self::Low),
            "NORMAL" => Ok(Self::Normal),
            "HIGH" => Ok(Self::High),
            "URGENT" => Ok(Self::Urgent),
            _ => Err(format!("Unknown priority: {}", s)),
        }
    }
}

/// Notification rule for custom alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRule {
    pub id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub event_types: Vec<ChangeKind>,
    pub path_pattern: Option<String>,
    pub min_size_bytes: Option<u64>,
    pub priority: Priority,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Generated user notification item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationItem {
    pub id: String,
    pub rule_id: Option<String>,
    pub title: String,
    pub message: String,
    pub priority: Priority,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

/// High-level overview statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub is_monitoring: bool,
    pub active_folders: usize,
    pub total_tracked_files: u64,
    pub changes_today: u64,
    pub versions_stored: u64,
    pub total_cas_bytes: u64,
    pub uncompressed_bytes: u64,
    pub deduplication_savings_pct: f64,
    pub radar_score: u32,
}

/// Storage breakdown metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_objects: u64,
    pub compressed_bytes: u64,
    pub raw_bytes: u64,
    pub pinned_versions: u64,
    pub deduplication_ratio: f64,
    pub database_bytes: u64,
}

/// Anomaly alert indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Priority,
    pub folder_id: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Pre-flight recovery inspection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPreflight {
    pub version_id: String,
    pub content_hash: String,
    pub file_size: u64,
    pub target_destination: String,
    pub destination_exists: bool,
    pub destination_size: Option<u64>,
    pub cas_object_verified: bool,
    pub can_proceed: bool,
    pub warning: Option<String>,
}

/// Result of a recovery execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub job_id: String,
    pub destination_path: String,
    pub restored_hash: String,
    pub bytes_written: u64,
    pub backup_version_id: Option<String>,
    pub success: bool,
    pub message: String,
}

/// Comparison types supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComparatorType {
    Text,
    Json,
    Csv,
    Image,
    Pdf,
    Binary,
}

/// Universal file comparison output structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonOutput {
    pub comparator_type: ComparatorType,
    pub summary: String,
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
    pub text_diff: Option<TextDiffData>,
    pub json_diff: Option<Vec<JsonDiffItem>>,
    pub csv_diff: Option<CsvDiffData>,
    pub image_diff: Option<ImageDiffData>,
    pub binary_diff: Option<BinaryDiffData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDiffData {
    pub unified_diff: String,
    pub side_by_side: Vec<SideBySideLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideBySideLine {
    pub line_num_a: Option<usize>,
    pub line_text_a: Option<String>,
    pub line_num_b: Option<usize>,
    pub line_text_b: Option<String>,
    pub change_type: String, // "EQUAL", "INSERT", "DELETE", "MODIFY"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDiffItem {
    pub path: String,
    pub kind: String, // "ADDED", "REMOVED", "MODIFIED"
    pub old_val: Option<String>,
    pub new_val: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvDiffData {
    pub headers: Vec<String>,
    pub rows_added: usize,
    pub rows_removed: usize,
    pub rows_modified: usize,
    pub sample_changes: Vec<CsvRowChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvRowChange {
    pub row_index: usize,
    pub change_kind: String,
    pub row_data: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDiffData {
    pub old_width: u32,
    pub old_height: u32,
    pub new_width: u32,
    pub new_height: u32,
    pub dimension_match: bool,
    pub perceptual_similarity_pct: f64,
    pub exact_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDiffData {
    pub old_size: u64,
    pub new_size: u64,
    pub size_diff: i64,
    pub old_hash: String,
    pub new_hash: String,
    pub hex_sample_diff: Vec<String>,
}
