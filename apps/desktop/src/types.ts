export type ChangeKind =
  | 'CREATED'
  | 'MODIFIED'
  | 'DELETED'
  | 'RENAMED'
  | 'MOVED'
  | 'REPLACED'
  | 'METADATA_CHANGED'
  | 'PERMISSIONS_CHANGED'
  | 'RESTORED'
  | 'SNAPSHOT_CREATED';

export type Priority = 'LOW' | 'NORMAL' | 'HIGH' | 'URGENT';

export interface MonitoredFolder {
  id: string;
  path: string;
  display_name: string;
  is_active: boolean;
  scan_interval_secs: number;
  ignore_patterns: string[];
  max_storage_bytes?: number;
  created_at: string;
  updated_at: string;
}

export interface FileRecord {
  id: string;
  folder_id: string;
  relative_path: string;
  file_name: string;
  extension: string;
  file_size: number;
  current_hash: string;
  is_deleted: boolean;
  created_at: string;
  updated_at: string;
}

export interface FileVersion {
  id: string;
  file_id: string;
  version_number: number;
  content_hash: string;
  file_size: number;
  change_kind: ChangeKind;
  notes?: string;
  is_pinned: boolean;
  created_at: string;
}

export interface FileEvent {
  id: string;
  folder_id: string;
  file_id?: string;
  event_type: ChangeKind;
  old_path?: string;
  new_path: string;
  old_hash?: string;
  new_hash?: string;
  file_size_diff: number;
  summary: string;
  confidence: 'EXACT' | 'HEURISTIC' | 'TENTATIVE';
  created_at: string;
}

export interface SystemOverview {
  is_monitoring: boolean;
  active_folders: number;
  total_tracked_files: number;
  changes_today: number;
  versions_stored: number;
  total_cas_bytes: number;
  uncompressed_bytes: number;
  deduplication_savings_pct: number;
  radar_score: number;
}

export interface StorageStats {
  total_objects: number;
  compressed_bytes: number;
  raw_bytes: number;
  pinned_versions: number;
  deduplication_ratio: number;
  database_bytes: number;
}

export interface Snapshot {
  id: string;
  folder_id?: string;
  name: string;
  description?: string;
  file_count: number;
  total_bytes: number;
  created_at: string;
}

export interface NotificationItem {
  id: string;
  rule_id?: string;
  title: string;
  message: string;
  priority: Priority;
  is_read: boolean;
  created_at: string;
}

export interface RecoveryPreflight {
  version_id: string;
  content_hash: string;
  file_size: number;
  target_destination: string;
  destination_exists: boolean;
  destination_size?: number;
  cas_object_verified: boolean;
  can_proceed: boolean;
  warning?: string;
}

export interface RecoveryResult {
  job_id: string;
  destination_path: string;
  restored_hash: string;
  bytes_written: number;
  backup_version_id?: string;
  success: boolean;
  message: string;
}

export interface ComparisonOutput {
  comparator_type: 'TEXT' | 'JSON' | 'CSV' | 'IMAGE' | 'PDF' | 'BINARY';
  summary: string;
  added_count: number;
  removed_count: number;
  modified_count: number;
  text_diff?: {
    unified_diff: string;
    side_by_side: Array<{
      line_num_a?: number;
      line_text_a?: string;
      line_num_b?: number;
      line_text_b?: string;
      change_type: 'EQUAL' | 'INSERT' | 'DELETE' | 'MODIFY';
    }>;
  };
  json_diff?: Array<{
    path: string;
    kind: 'ADDED' | 'REMOVED' | 'MODIFIED';
    old_val?: string;
    new_val?: string;
  }>;
  csv_diff?: {
    headers: string[];
    rows_added: number;
    rows_removed: number;
    rows_modified: number;
    sample_changes: Array<{
      row_index: number;
      change_kind: string;
      row_data: string[];
    }>;
  };
  image_diff?: {
    old_width: number;
    old_height: number;
    new_width: number;
    new_height: number;
    dimension_match: boolean;
    perceptual_similarity_pct: number;
    exact_match: boolean;
  };
  binary_diff?: {
    old_size: number;
    new_size: number;
    size_diff: number;
    old_hash: string;
    new_hash: string;
    hex_sample_diff: string[];
  };
}

export interface AnomalyAlert {
  id: string;
  title: string;
  description: string;
  severity: Priority;
  folder_id?: string;
  detected_at: string;
}

export interface DoctorReport {
  database_integrity: string;
  database_path: string;
  storage_stats: StorageStats;
  cas_integrity: {
    total_checked: number;
    valid_count: number;
    corrupted_count: number;
    missing_count: number;
    corrupted_hashes: string[];
  };
  monitored_folders_count: number;
  active_folders_count: number;
  total_tracked_files: number;
  platform: string;
  all_healthy: boolean;
}

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  capabilities: string[];
  is_enabled: boolean;
}

export interface ActivityStats {
  daily_changes: Array<{ date: string; count: number }>;
  folder_breakdown: Array<{ folder_id: string; folder_name: string; change_count: number }>;
  top_modified_files: Array<{
    file_id: string;
    file_name: string;
    relative_path: string;
    version_count: number;
  }>;
  total_anomalies_detected: number;
}

export interface CleanupPreview {
  candidate_versions_count: number;
  estimated_reclaimed_bytes: number;
  affected_files_count: number;
  candidate_version_ids: string[];
}
