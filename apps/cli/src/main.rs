use changeradar_core::{
    ChangeRadarEngine, RetentionPolicy, UniversalComparator,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "changeradar",
    author = "ChangeRadar Team",
    version = "1.0.0",
    about = "Universal File Intelligence, Version History & Recovery Platform",
    long_about = "ChangeRadar is a local-first, privacy-first file intelligence and version-history platform.\nMonitors folders, detects changes, preserves versions, computes diffs and performs safe recovery."
)]
struct Cli {
    #[arg(short, long, global = true, help = "Output response formatted as JSON")]
    json: bool,

    #[arg(
        long,
        global = true,
        help = "Optional custom data directory instead of default OS path"
    )]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Display system overview, monitoring health and Radar Score")]
    Status,

    #[command(about = "Manage monitored folders")]
    Folder {
        #[command(subcommand)]
        action: FolderCommands,
    },

    #[command(about = "View chronological change history and timeline")]
    History {
        #[arg(short, long, default_value_t = 25, help = "Number of events to list")]
        limit: usize,
        #[arg(short, long, help = "Filter events by monitored folder ID")]
        folder: Option<String>,
    },

    #[command(about = "Search across files, events, and folders")]
    Search {
        #[arg(help = "Search query string")]
        query: String,
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },

    #[command(about = "Inspect tracked files and version history")]
    File {
        #[command(subcommand)]
        action: FileCommands,
    },

    #[command(about = "Compare two historical versions")]
    Compare {
        #[arg(help = "First version ID")]
        version_a: String,
        #[arg(help = "Second version ID")]
        version_b: String,
    },

    #[command(about = "Manage directory point-in-time snapshots")]
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },

    #[command(about = "Safely restore a historical file version")]
    Restore {
        #[arg(help = "Version ID to restore")]
        version_id: String,
        #[arg(
            short,
            long,
            help = "Optional alternate destination path (defaults to original location)"
        )]
        dest: Option<PathBuf>,
        #[arg(
            long,
            help = "Skip pre-flight confirmation check and restore immediately"
        )]
        force: bool,
    },

    #[command(about = "View Content-Addressed Storage (CAS) stats and deduplication")]
    Storage,

    #[command(about = "Preview and apply retention cleanup")]
    Cleanup {
        #[arg(long, help = "Only preview what will be removed without deleting")]
        preview: bool,
        #[arg(long, help = "Apply the cleanup policy and reclaim space")]
        apply: bool,
        #[arg(long, default_value_t = 90, help = "Maximum version age in days")]
        max_age_days: i64,
        #[arg(long, default_value_t = 50, help = "Maximum versions per file")]
        max_versions: usize,
    },

    #[command(about = "Run full system diagnostics and storage integrity checks")]
    Doctor,

    #[command(about = "View current ChangeRadar configuration")]
    Config,
}

#[derive(Subcommand)]
enum FolderCommands {
    #[command(about = "Add a folder to be monitored")]
    Add {
        #[arg(help = "Filesystem path to monitor")]
        path: PathBuf,
        #[arg(short, long, help = "Custom display name")]
        name: Option<String>,
    },
    #[command(about = "List all monitored folders")]
    List,
    #[command(about = "Remove a folder from monitoring")]
    Remove {
        #[arg(help = "Monitored folder ID")]
        id: String,
    },
    #[command(about = "Pause monitoring on a folder")]
    Pause {
        #[arg(help = "Monitored folder ID")]
        id: String,
    },
    #[command(about = "Resume monitoring on a paused folder")]
    Resume {
        #[arg(help = "Monitored folder ID")]
        id: String,
    },
}

#[derive(Subcommand)]
enum FileCommands {
    #[command(about = "List tracked files")]
    List {
        #[arg(short, long, help = "Filter by folder ID")]
        folder: Option<String>,
        #[arg(short, long, default_value_t = 50)]
        limit: usize,
    },
    #[command(about = "Show version history for a file")]
    Versions {
        #[arg(help = "File ID")]
        file_id: String,
    },
}

#[derive(Subcommand)]
enum SnapshotCommands {
    #[command(about = "Create a new snapshot")]
    Create {
        #[arg(short, long, help = "Name of the snapshot")]
        name: String,
        #[arg(short, long, help = "Optional description")]
        description: Option<String>,
        #[arg(short, long, help = "Optional specific folder ID")]
        folder: Option<String>,
    },
    #[command(about = "List all snapshots")]
    List,
    #[command(about = "Restore an entire snapshot")]
    Restore {
        #[arg(help = "Snapshot ID to restore")]
        snapshot_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let engine = ChangeRadarEngine::initialize(cli.data_dir.as_deref())?;

    match cli.command {
        Some(Commands::Status) | None => {
            let overview = engine.intelligence.get_overview(true)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&overview)?);
            } else {
                println!("\n╔══════════════════════════════════════════════════════╗");
                println!("║                CHANGERADAR STATUS                    ║");
                println!("╚══════════════════════════════════════════════════════╝");
                println!("  Monitoring State:      {}", if overview.is_monitoring { "ACTIVE [ONLINE]" } else { "PAUSED" });
                println!("  Radar Score:           {}/100", overview.radar_score);
                println!("  Active Folders:        {}", overview.active_folders);
                println!("  Total Tracked Files:   {}", overview.total_tracked_files);
                println!("  Changes Today:         {}", overview.changes_today);
                println!("  Stored Versions:       {}", overview.versions_stored);
                println!("  Compressed CAS Size:   {} MB", overview.total_cas_bytes / (1024 * 1024));
                println!("  Raw File Size:         {} MB", overview.uncompressed_bytes / (1024 * 1024));
                println!("  Deduplication Savings: {:.1}%\n", overview.deduplication_savings_pct);
            }
        }

        Some(Commands::Folder { action }) => match action {
            FolderCommands::Add { path, name } => {
                let folder = engine.add_folder(&path.to_string_lossy(), name.as_deref())?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&folder)?);
                } else {
                    println!("Successfully added monitored folder '{}' at {:?}", folder.display_name, folder.path);
                }
            }
            FolderCommands::List => {
                let list = engine.list_folders()?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&list)?);
                } else {
                    println!("\nMonitored Folders ({} total):", list.len());
                    for f in list {
                        let status = if f.is_active { "ACTIVE" } else { "PAUSED" };
                        println!("  - [{}] {} (ID: {}) -> {}", status, f.display_name, f.id, f.path);
                    }
                    println!();
                }
            }
            FolderCommands::Remove { id } => {
                engine.remove_folder(&id)?;
                if cli.json {
                    println!("{{\"success\":true}}");
                } else {
                    println!("Removed monitored folder {}", id);
                }
            }
            FolderCommands::Pause { id } => {
                engine.pause_folder(&id)?;
                println!("Paused monitoring for folder {}", id);
            }
            FolderCommands::Resume { id } => {
                engine.resume_folder(&id)?;
                println!("Resumed monitoring for folder {}", id);
            }
        },

        Some(Commands::History { limit, folder }) => {
            let events = engine.db.list_events(folder.as_deref(), limit, 0)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&events)?);
            } else {
                println!("\nRecent Change History ({} events):", events.len());
                for ev in events {
                    println!(
                        "  [{}] {:<18} {:<35} {}",
                        ev.created_at.format("%Y-%m-%d %H:%M:%S"),
                        ev.event_type.to_string(),
                        ev.new_path,
                        ev.summary
                    );
                }
                println!();
            }
        }

        Some(Commands::Search { query, limit }) => {
            let res = engine.search.search(&query, limit)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("\nSearch Results for '{}':", query);
                println!("  Matched Files:   {}", res.matched_files.len());
                for f in &res.matched_files {
                    println!("    * {} ({})", f.relative_path, f.id);
                }
                println!("  Matched Events:  {}", res.matched_events.len());
                for e in &res.matched_events {
                    println!("    * [{}] {} -> {}", e.event_type, e.new_path, e.summary);
                }
                println!();
            }
        }

        Some(Commands::File { action }) => match action {
            FileCommands::List { folder, limit } => {
                let files = engine.db.list_files(folder.as_deref(), limit)?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&files)?);
                } else {
                    println!("\nTracked Files ({} total):", files.len());
                    for f in files {
                        let del = if f.is_deleted { " [DELETED]" } else { "" };
                        println!("  - {} ({} bytes, ID: {}){}", f.relative_path, f.file_size, f.id, del);
                    }
                    println!();
                }
            }
            FileCommands::Versions { file_id } => {
                let versions = engine.db.list_versions_for_file(&file_id)?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&versions)?);
                } else {
                    println!("\nVersion History for file {} ({} versions):", file_id, versions.len());
                    for v in versions {
                        let pin = if v.is_pinned { " [PINNED]" } else { "" };
                        let note = v.notes.as_deref().unwrap_or("-");
                        println!(
                            "  #{} | {} | {} bytes | {}{}",
                            v.version_number,
                            v.created_at.format("%Y-%m-%d %H:%M"),
                            v.file_size,
                            note,
                            pin
                        );
                    }
                    println!();
                }
            }
        },

        Some(Commands::Compare { version_a, version_b }) => {
            let bytes_a = engine.store.get_version_bytes(&version_a)?;
            let bytes_b = engine.store.get_version_bytes(&version_b)?;
            let out = UniversalComparator::compare("file.txt", &bytes_a, &bytes_b);

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("\nComparison Summary: {}", out.summary);
                println!("  Added: {}, Removed: {}, Modified: {}", out.added_count, out.removed_count, out.modified_count);
                if let Some(td) = out.text_diff {
                    println!("\nDiff:\n{}", td.unified_diff);
                }
            }
        }

        Some(Commands::Snapshot { action }) => match action {
            SnapshotCommands::Create { name, description, folder } => {
                let snap = engine.snapshots.create_snapshot(folder.as_deref(), &name, description.as_deref())?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&snap)?);
                } else {
                    println!("Created snapshot '{}' (ID: {}) with {} files", snap.name, snap.id, snap.file_count);
                }
            }
            SnapshotCommands::List => {
                let snaps = engine.db.list_snapshots(None)?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&snaps)?);
                } else {
                    println!("\nSnapshots ({} total):", snaps.len());
                    for s in snaps {
                        println!("  - '{}' | {} files | {} bytes | ID: {}", s.name, s.file_count, s.total_bytes, s.id);
                    }
                    println!();
                }
            }
            SnapshotCommands::Restore { snapshot_id } => {
                let results = engine.snapshots.restore_snapshot(&snapshot_id)?;
                println!("Restored {} files from snapshot {}", results.len(), snapshot_id);
            }
        },

        Some(Commands::Restore { version_id, dest, force }) => {
            let preflight = engine.recovery.preflight_check(&version_id, dest.as_deref())?;
            if !force && !preflight.can_proceed {
                eprintln!("Preflight check failed: {:?}", preflight.warning);
                std::process::exit(1);
            }

            let res = engine.recovery.restore_version(&version_id, dest.as_deref())?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&res)?);
            } else {
                println!("Recovery succeeded! {}", res.message);
                println!("Destination: {}", res.destination_path);
            }
        }

        Some(Commands::Storage) => {
            let stats = engine.storage_mgr.get_stats()?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("\n╔══════════════════════════════════════════════════════╗");
                println!("║             CONTENT-ADDRESSED STORAGE (CAS)          ║");
                println!("╚══════════════════════════════════════════════════════╝");
                println!("  Unique CAS Objects:    {}", stats.total_objects);
                println!("  Compressed Size:       {} MB", stats.compressed_bytes / (1024 * 1024));
                println!("  Raw Original Size:     {} MB", stats.raw_bytes / (1024 * 1024));
                println!("  Deduplication Ratio:   {:.2}x", stats.deduplication_ratio);
                println!("  Pinned Versions:       {}", stats.pinned_versions);
                println!("  Metadata DB Size:      {} KB\n", stats.database_bytes / 1024);
            }
        }

        Some(Commands::Cleanup { preview, apply, max_age_days, max_versions }) => {
            let policy = RetentionPolicy {
                max_versions_per_file: Some(max_versions),
                max_age_days: Some(max_age_days),
                max_storage_bytes: None,
            };

            let prev = engine.storage_mgr.preview_cleanup(&policy)?;
            if preview || !apply {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&prev)?);
                } else {
                    println!("\nRetention Cleanup Preview:");
                    println!("  Candidate Versions to Prune: {}", prev.candidate_versions_count);
                    println!("  Estimated Space Reclaimed:   {} MB", prev.estimated_reclaimed_bytes / (1024 * 1024));
                    println!("  Files Affected:              {}", prev.affected_files_count);
                    println!("\nRun with --apply to execute cleanup and garbage collection.\n");
                }
            } else if apply {
                let gc = engine.storage_mgr.execute_cleanup(&prev)?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&gc)?);
                } else {
                    println!("Cleanup completed! Reclaimed {} bytes across {} objects.", gc.reclaimed_bytes, gc.deleted_objects_count);
                }
            }
        }

        Some(Commands::Doctor) => {
            let doc = engine.run_doctor()?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&doc)?);
            } else {
                println!("\n╔══════════════════════════════════════════════════════╗");
                println!("║                 CHANGERADAR DOCTOR                   ║");
                println!("╚══════════════════════════════════════════════════════╝");
                println!("  System Health:         {}", if doc.all_healthy { "HEALTHY (All OK)" } else { "ATTENTION NEEDED" });
                println!("  Platform:              {}", doc.platform);
                println!("  SQLite DB Integrity:   {}", doc.database_integrity);
                println!("  Database File:         {}", doc.database_path);
                println!("  CAS Valid Objects:     {}", doc.cas_integrity.valid_count);
                println!("  CAS Corrupted Objects: {}", doc.cas_integrity.corrupted_count);
                println!("  Monitored Folders:     {} (Active: {})", doc.monitored_folders_count, doc.active_folders_count);
                println!("  Tracked Files:         {}\n", doc.total_tracked_files);
            }
        }

        Some(Commands::Config) => {
            let cfg = engine.config.read().clone();
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                println!("\nChangeRadar Configuration:");
                println!("  Config File Path:      {:?}", engine.paths.config_file);
                println!("  API Port:              {}", cfg.api_port);
                println!("  Theme:                 {}", cfg.theme);
                println!("  Telemetry:             {} (Privacy guarantee: Always off)", cfg.telemetry_enabled);
                println!("  Retention Days:        {}", cfg.retention_days);
                println!("  Max Versions / File:   {}\n", cfg.max_versions_per_file);
            }
        }
    }

    Ok(())
}
