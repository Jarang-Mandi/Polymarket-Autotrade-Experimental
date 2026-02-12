mod config;
mod db;
mod error;
mod types;
mod polymarket;
mod engine;
mod server;

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    dotenv::dotenv().ok();

    let config = config::Config::from_env()?;

    // Validate critical config
    if config.private_key.is_empty() {
        return Err(anyhow::anyhow!("POLYMARKET_PRIVATE_KEY is required but empty"));
    }

    tracing::info!("╔══════════════════════════════════════════════╗");
    tracing::info!("║   POLYMARKET EXECUTION ENGINE v0.3           ║");
    tracing::info!("║   Mode: COMMAND-DRIVEN (OpenClaw = brain)    ║");
    tracing::info!("║   Capital: ${:<10.2}                       ║", config.initial_capital);
    tracing::info!("║   DB: {:<37} ║", config.db_path);
    tracing::info!("╚══════════════════════════════════════════════╝");

    // Open SQLite database (creates + migrates if new)
    let db = db::Database::open(&config.db_path)?;

    // Create engine wrapped in Arc for sharing with server
    let engine = Arc::new(engine::TradingEngine::new(config.clone(), db));
    let state = engine.state.clone();
    let engine_for_server = engine.clone();

    // Start command + dashboard server
    let dashboard_host = config.dashboard_host.clone();
    let dashboard_port = config.dashboard_port;
    tokio::spawn(async move {
        if let Err(e) = server::start_dashboard_server(
            &dashboard_host,
            dashboard_port,
            state,
            engine_for_server,
        ).await {
            tracing::error!("Server fatal error: {}", e);
            std::process::exit(1);
        }
    });

    // Start background loops (scan + position monitor)
    engine.run().await;

    Ok(())
}
