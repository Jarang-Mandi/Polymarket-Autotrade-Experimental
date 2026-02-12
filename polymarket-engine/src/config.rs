use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Polymarket
    pub clob_url: String,
    pub gamma_url: String,
    pub chain_id: u64,
    pub private_key: String,
    pub funder_address: String,
    pub signature_type: u8,

    // Trading
    pub initial_capital: f64,
    pub max_position_size: f64,
    pub max_portfolio_risk: f64,
    pub daily_api_budget: f64,
    pub min_edge_threshold: f64,

    // Dashboard
    pub dashboard_host: String,
    pub dashboard_port: u16,

    // Database
    pub db_path: String,

    // Intervals
    pub heartbeat_interval_secs: u64,
    pub market_scan_interval_secs: u64,
    pub position_update_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            clob_url: env::var("POLYMARKET_CLOB_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".into()),
            gamma_url: env::var("POLYMARKET_GAMMA_URL")
                .unwrap_or_else(|_| "https://gamma-api.polymarket.com".into()),
            chain_id: env::var("POLYMARKET_CHAIN_ID")
                .unwrap_or_else(|_| "137".into())
                .parse()?,
            private_key: env::var("POLYMARKET_PRIVATE_KEY")
                .unwrap_or_else(|_| String::new()),
            funder_address: env::var("POLYMARKET_FUNDER_ADDRESS")
                .unwrap_or_else(|_| String::new()),
            signature_type: env::var("POLYMARKET_SIGNATURE_TYPE")
                .unwrap_or_else(|_| "1".into())
                .parse()?,

            initial_capital: env::var("INITIAL_CAPITAL")
                .unwrap_or_else(|_| "50.0".into())
                .parse()?,
            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "5.0".into())
                .parse()?,
            max_portfolio_risk: env::var("MAX_PORTFOLIO_RISK")
                .unwrap_or_else(|_| "0.06".into())
                .parse()?,
            daily_api_budget: env::var("DAILY_API_BUDGET")
                .unwrap_or_else(|_| "0.50".into())
                .parse()?,
            min_edge_threshold: env::var("MIN_EDGE_THRESHOLD")
                .unwrap_or_else(|_| "0.08".into())
                .parse()?,

            dashboard_host: env::var("DASHBOARD_HOST")
                .unwrap_or_else(|_| "127.0.0.1".into()),
            dashboard_port: env::var("DASHBOARD_PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()?,

            db_path: env::var("DB_PATH")
                .unwrap_or_else(|_| "data/engine.db".into()),

            heartbeat_interval_secs: env::var("HEARTBEAT_INTERVAL_SECS")
                .unwrap_or_else(|_| "30".into())
                .parse()?,
            market_scan_interval_secs: env::var("MARKET_SCAN_INTERVAL_SECS")
                .unwrap_or_else(|_| "300".into())
                .parse()?,
            position_update_interval_secs: env::var("POSITION_UPDATE_INTERVAL_SECS")
                .unwrap_or_else(|_| "60".into())
                .parse()?,
        })
    }
}
