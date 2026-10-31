use changeradar_core::{start_api_server, ApiState, ChangeRadarEngine};
use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(
    name = "changeradard",
    author = "ChangeRadar Team",
    version = "1.0.0",
    about = "ChangeRadar Background Monitoring Daemon"
)]
struct DaemonCli {
    #[arg(long, help = "Optional custom data directory")]
    data_dir: Option<PathBuf>,

    #[arg(long, default_value_t = 41820, help = "Developer API port")]
    port: u16,

    #[arg(long, default_value_t = 300, help = "Periodic reconciliation interval in seconds")]
    reconcile_interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = DaemonCli::parse();

    info!("Starting ChangeRadar Background Daemon v1.0.0");

    let engine = ChangeRadarEngine::initialize(args.data_dir.as_deref())?;

    // Create API state for local REST server
    let api_state = ApiState {
        db: engine.db.clone(),
        store: engine.store.clone(),
        intelligence: engine.intelligence.clone(),
        recovery: engine.recovery.clone(),
        snapshots: engine.snapshots.clone(),
        storage: engine.storage_mgr.clone(),
        search: engine.search.clone(),
        plugins: engine.plugins.clone(),
        api_token: engine.config.read().api_token.clone(),
    };

    // Spawn Developer API server on 127.0.0.1
    let port = args.port;
    tokio::spawn(async move {
        if let Err(e) = start_api_server(api_state, port).await {
            error!("Developer API server failed: {}", e);
        }
    });

    // Spawn periodic reconciliation loop
    let engine_recon = engine.clone();
    let interval_secs = args.reconcile_interval;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            let folders = engine_recon.db.list_folders().unwrap_or_default();
            for f in folders {
                if f.is_active {
                    info!("Running periodic reconciliation scan for folder '{}'", f.display_name);
                    let _ = engine_recon.event_processor.reconcile_folder(&f.id);
                }
            }
        }
    });

    info!("ChangeRadar Daemon is running in the background. Press Ctrl+C to shut down cleanly.");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down ChangeRadar Daemon cleanly...");
    Ok(())
}
