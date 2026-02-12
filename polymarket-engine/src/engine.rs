use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tracing::{info, warn, error};

use crate::config::Config;
use crate::db::Database;
use crate::error::{EngineError, EngineResult, log_error};
use crate::types::*;
use crate::polymarket::PolymarketClient;

/// Core execution engine — the "hands" of the agent.
/// OpenClaw (the "brain") makes decisions via Claude and sends commands here.
pub struct TradingEngine {
    config: Config,
    polymarket: PolymarketClient,
    db: Database,
    pub state: Arc<RwLock<EngineState>>,
}

/// Mutable engine state shared with dashboard and API
pub struct EngineState {
    pub portfolio: PortfolioState,
    pub api_costs: ApiCostTracker,
    pub cached_markets: Vec<Market>,
    pub engine_running: bool,
    pub last_scan: Option<chrono::DateTime<Utc>>,
    pub last_command: Option<chrono::DateTime<Utc>>,
    pub errors: Vec<String>,
}

impl TradingEngine {
    pub fn new(config: Config, db: Database) -> Self {
        let polymarket = PolymarketClient::new(&config);

        // Try to recover state from DB
        let recovered_capital = db.load_capital().ok().flatten();
        let recovered_positions = db.load_open_positions().unwrap_or_default();
        let recovered_trades = db.load_recent_trades(50).unwrap_or_default();
        let recovered_stats = db.load_portfolio_stats().ok().flatten();

        let capital = recovered_capital.unwrap_or(config.initial_capital);
        let position_count = recovered_positions.len();
        let trade_count = recovered_trades.len();

        let mut total_pnl = 0.0;
        let mut total_trades: u32 = 0;
        let mut winning_trades: u32 = 0;
        let mut win_rate = 0.0;

        if let Some(stats) = recovered_stats {
            total_pnl = stats["total_pnl"].as_f64().unwrap_or(0.0);
            total_trades = stats["total_trades"].as_u64().unwrap_or(0) as u32;
            winning_trades = stats["winning_trades"].as_u64().unwrap_or(0) as u32;
            win_rate = stats["win_rate"].as_f64().unwrap_or(0.0);
        }

        if position_count > 0 || recovered_capital.is_some() {
            info!("STATE RECOVERED from SQLite:");
            info!("  Capital: ${:.2} (initial: ${:.2})", capital, config.initial_capital);
            info!("  Open positions: {}", position_count);
            info!("  Trade history: {} records", trade_count);
        }

        let initial_state = EngineState {
            portfolio: PortfolioState {
                capital,
                initial_capital: config.initial_capital,
                total_pnl,
                total_pnl_pct: if config.initial_capital > 0.0 {
                    (capital - config.initial_capital) / config.initial_capital * 100.0
                } else { 0.0 },
                daily_pnl: 0.0,
                agent_state: AgentState::from_capital(capital),
                hunger_level: HungerLevel::Seeking,
                positions: recovered_positions,
                recent_trades: recovered_trades,
                api_costs: ApiCostTracker::new(config.daily_api_budget),
                win_rate,
                total_trades,
                winning_trades,
                uptime_hours: 0.0,
                last_trade_at: None,
                last_profit_at: None,
                timestamp: Utc::now(),
            },
            api_costs: ApiCostTracker::new(config.daily_api_budget),
            cached_markets: vec![],
            engine_running: false,
            last_scan: None,
            last_command: None,
            errors: vec![],
        };

        Self {
            config,
            polymarket,
            db,
            state: Arc::new(RwLock::new(initial_state)),
        }
    }

    /// Start background loops (market scanning, position monitoring, heartbeat).
    /// Trade decisions come from OpenClaw via REST API — NOT from this engine.
    pub async fn run(&self) {
        info!("=== POLYMARKET EXECUTION ENGINE STARTING ===");
        info!("Mode: COMMAND-DRIVEN (OpenClaw is the brain)");
        info!("Initial capital: ${:.2}", self.config.initial_capital);

        {
            let mut state = self.state.write().await;
            state.engine_running = true;
        }

        // Check Polymarket server status
        match self.polymarket.get_server_status().await {
            Ok(status) => info!("CLOB status: {:?}", status),
            Err(e) => warn!("Could not reach CLOB: {}", e),
        }

        let scan_interval = Duration::from_secs(self.config.market_scan_interval_secs);
        let position_interval = Duration::from_secs(self.config.position_update_interval_secs);
        let heartbeat_interval = Duration::from_secs(self.config.heartbeat_interval_secs);

        // Spawn heartbeat + periodic DB snapshot
        let hb_state = self.state.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(heartbeat_interval);
            let mut snapshot_counter: u32 = 0;
            loop {
                interval.tick().await;
                let mut s = hb_state.write().await;
                s.portfolio.timestamp = Utc::now();
                s.portfolio.agent_state = AgentState::from_capital(s.portfolio.capital);
                s.api_costs.check_daily_reset();
                s.portfolio.api_costs = s.api_costs.clone();
                snapshot_counter += 1;
                // snapshot_counter not used for DB here since we don't have db ref in this task
                // DB snapshots happen via the main loop
                let _ = snapshot_counter;
            }
        });

        // Main loop: scan markets + update positions + periodic DB snapshot
        let mut scan_tick = time::interval(scan_interval);
        let mut position_tick = time::interval(position_interval);
        let mut snapshot_tick = time::interval(Duration::from_secs(3600)); // hourly

        loop {
            tokio::select! {
                _ = scan_tick.tick() => {
                    self.market_scan_cycle().await;
                }
                _ = position_tick.tick() => {
                    self.update_positions().await;
                }
                _ = snapshot_tick.tick() => {
                    let state = self.state.read().await;
                    if let Err(e) = self.db.save_snapshot(&state.portfolio) {
                        warn!("DB snapshot failed: {}", e);
                    } else {
                        info!("Portfolio snapshot saved to DB");
                    }
                }
            }
        }
    }

    /// Scan markets for opportunities — data only, no decisions
    async fn market_scan_cycle(&self) {
        info!("--- MARKET SCAN ---");

        let state = self.state.read().await;
        let (min_vol, min_liq) = match state.portfolio.agent_state {
            AgentState::Survival => (100_000.0, 50_000.0),
            AgentState::Defensive => (50_000.0, 25_000.0),
            AgentState::Neutral => (25_000.0, 10_000.0),
            AgentState::Aggressive => (10_000.0, 5_000.0),
            AgentState::Apex => (5_000.0, 2_000.0),
        };
        drop(state);

        match self.polymarket.scan_opportunities(min_vol, min_liq).await {
            Ok(markets) => {
                let mut state = self.state.write().await;
                state.cached_markets = markets;
                state.last_scan = Some(Utc::now());
                info!("Cached {} markets (available via GET /api/markets)", state.cached_markets.len());
            }
            Err(e) => {
                log_error(&e);
                if let Err(push_err) = self.push_error(&e).await {
                    warn!("Failed to push error to state: {}", push_err);
                }
                // If retryable, don't panic — next tick will try again
                if !e.is_retryable() {
                    error!("Non-retryable scan failure: {}", e);
                }
            }
        }
    }

    // ═══════════════════════════════════════════
    // COMMAND HANDLERS — Called by REST API
    // OpenClaw sends commands, engine executes
    // ═══════════════════════════════════════════

    /// Validate a trade command before execution
    fn validate_trade_command(&self, cmd: &TradeCommand, capital: f64, state: &AgentState) -> EngineResult<()> {
        if cmd.price <= 0.0 || cmd.price >= 1.0 {
            return Err(EngineError::BadRequest(
                format!("Price {:.3} must be between 0 and 1", cmd.price),
            ));
        }
        if cmd.size <= 0.0 {
            return Err(EngineError::BadRequest(
                "Size must be positive".into(),
            ));
        }
        if cmd.market_id.is_empty() {
            return Err(EngineError::BadRequest(
                "market_id is required".into(),
            ));
        }
        let min_edge = state.min_edge();
        if cmd.edge < min_edge {
            return Err(EngineError::EdgeTooLow {
                edge: cmd.edge * 100.0,
                min: min_edge * 100.0,
                state: format!("{:?}", state),
            });
        }
        if cmd.confidence < 0.5 {
            return Err(EngineError::LowConfidence(cmd.confidence * 100.0));
        }
        let cost = cmd.size.min(self.config.max_position_size) * cmd.price;
        if cost > capital * 0.95 {
            return Err(EngineError::InsufficientCapital {
                cost,
                capital,
                pct: 95.0,
            });
        }
        Ok(())
    }

    /// Execute a trade command from OpenClaw
    pub async fn handle_trade_command(&self, cmd: TradeCommand) -> EngineResult<CommandResponse> {
        info!("TRADE COMMAND: {:?} {} @ {:.3} on {}", cmd.side, cmd.size, cmd.price, cmd.market_id);

        // Read state for validation
        let state = self.state.read().await;
        let market = state.cached_markets.iter().find(|m| m.id == cmd.market_id).cloned();
        let max_size = state.portfolio.capital * state.portfolio.agent_state.max_position_pct();
        let capital = state.portfolio.capital;
        let agent_state = state.portfolio.agent_state;
        let position_count = state.portfolio.positions.len();
        drop(state);

        // Max positions check
        if position_count >= 5 {
            return Err(EngineError::MaxPositionsReached(5));
        }

        // Validate command
        self.validate_trade_command(&cmd, capital, &agent_state)?;

        // Risk-adjusted size
        let actual_size = cmd.size.min(max_size).min(self.config.max_position_size);
        if actual_size <= 0.0 {
            return Err(EngineError::PositionSizeExceeded {
                size: cmd.size,
                limit: max_size.min(self.config.max_position_size),
            });
        }

        let cost = actual_size * cmd.price;

        // Resolve market & token ID
        let market = market.ok_or_else(|| EngineError::MarketNotFound(cmd.market_id.clone()))?;
        let (yes_id, no_id) = market.token_ids()
            .ok_or_else(|| EngineError::NoTokenIds(cmd.market_id.clone()))?;
        let token_id = match cmd.side {
            Side::Buy => yes_id,
            Side::Sell => no_id,
        };
        let question = market.question.clone().unwrap_or_default();

        let tick_size = market.order_price_min_tick_size
            .map(|t| format!("{:.3}", t))
            .unwrap_or_else(|| "0.01".to_string());

        let order = OrderRequest {
            token_id: token_id.clone(),
            price: cmd.price,
            size: actual_size,
            side: cmd.side,
            order_type: OrderType::GTC,
            neg_risk: market.neg_risk.unwrap_or(false),
            tick_size,
        };

        let resp = match self.polymarket.place_order(&order).await {
            Ok(r) => r,
            Err(e) => {
                log_error(&e);
                let _ = self.push_error(&e).await;
                return Err(e);
            }
        };

        if !resp.success {
            let err_msg = resp.error_msg.unwrap_or_else(|| "Unknown".into());
            return Err(EngineError::OrderRejected(err_msg));
        }

        // Success — update state
        let trade = TradeLog::new(
            &cmd.market_id,
            &question,
            cmd.side,
            cmd.price,
            actual_size,
            &cmd.reason,
            cmd.edge,
            cmd.confidence,
            0.0,
        );
        let trade_id = trade.id.clone();

        let position = Position::new(
            &cmd.market_id,
            &question,
            &token_id,
            cmd.side,
            cmd.price,
            actual_size,
        );
        let position_id = position.id.clone();

        // Persist to SQLite (non-blocking — warn on failure, don't abort)
        if let Err(e) = self.db.insert_trade(&trade) {
            warn!("DB insert_trade failed: {}", e);
        }
        if let Err(e) = self.db.insert_position(&position) {
            warn!("DB insert_position failed: {}", e);
        }

        let mut state = self.state.write().await;
        state.portfolio.capital -= cost;
        state.portfolio.positions.push(position);
        state.portfolio.recent_trades.push(trade);
        state.portfolio.total_trades += 1;
        state.portfolio.last_trade_at = Some(Utc::now());
        state.last_command = Some(Utc::now());

        if state.portfolio.recent_trades.len() > 50 {
            state.portfolio.recent_trades.drain(0..1);
        }

        // Persist capital + stats
        if let Err(e) = self.db.save_capital(state.portfolio.capital) {
            warn!("DB save_capital failed: {}", e);
        }
        if let Err(e) = self.db.save_portfolio_stats(&state.portfolio) {
            warn!("DB save_portfolio_stats failed: {}", e);
        }

        info!("TRADE EXECUTED: {:?} ${:.2} @ {:.3}", cmd.side, actual_size, cmd.price);

        Ok(CommandResponse {
            success: true,
            message: format!("Executed {:?} ${:.2} @ {:.3}", cmd.side, actual_size, cmd.price),
            trade_id: Some(trade_id),
            position_id: Some(position_id),
        })
    }

    /// Close a position by command from OpenClaw
    pub async fn handle_close_command(&self, cmd: CloseCommand) -> EngineResult<CommandResponse> {
        info!("CLOSE COMMAND: position {} — {}", cmd.position_id, cmd.reason);

        if cmd.position_id.is_empty() {
            return Err(EngineError::BadRequest("position_id is required".into()));
        }

        let mut state = self.state.write().await;

        let pos_idx = state.portfolio.positions.iter().position(|p| p.id == cmd.position_id)
            .ok_or_else(|| EngineError::PositionNotFound(cmd.position_id.clone()))?;

        let pos = &mut state.portfolio.positions[pos_idx];
        pos.status = PositionStatus::Closed;

        let pnl = pos.unrealized_pnl;
        state.portfolio.capital += pos.cost_basis + pnl;
        state.portfolio.total_pnl += pnl;
        state.portfolio.daily_pnl += pnl;

        if pnl > 0.0 {
            state.portfolio.winning_trades += 1;
            state.portfolio.last_profit_at = Some(Utc::now());
        }

        if state.portfolio.total_trades > 0 {
            state.portfolio.win_rate =
                state.portfolio.winning_trades as f64 / state.portfolio.total_trades as f64 * 100.0;
        }

        let msg = format!("Closed position. PnL: ${:.4} ({:.1}%)", pnl, pos.pnl_pct());
        let pid = pos.id.clone();

        // Persist close to SQLite
        if let Err(e) = self.db.close_position(&pid, pnl) {
            warn!("DB close_position failed: {}", e);
        }

        state.portfolio.positions.retain(|p| p.status == PositionStatus::Open);
        state.last_command = Some(Utc::now());

        // Persist capital + stats
        if let Err(e) = self.db.save_capital(state.portfolio.capital) {
            warn!("DB save_capital failed: {}", e);
        }
        if let Err(e) = self.db.save_portfolio_stats(&state.portfolio) {
            warn!("DB save_portfolio_stats failed: {}", e);
        }

        Ok(CommandResponse {
            success: true,
            message: msg,
            trade_id: None,
            position_id: Some(pid),
        })
    }

    /// Report API cost from OpenClaw (for dashboard tracking)
    pub async fn handle_cost_report(&self, report: ApiCostReport) {
        let mut state = self.state.write().await;
        state.api_costs.add_usage(
            report.input_tokens,
            report.output_tokens,
            report.cache_read_tokens,
            report.cache_write_tokens,
        );
        state.portfolio.api_costs = state.api_costs.clone();
    }

    /// Push error to state (visible on dashboard)
    async fn push_error(&self, err: &EngineError) -> EngineResult<()> {
        let mut state = self.state.write().await;
        let entry = format!("[{}] {}: {}", Utc::now().format("%H:%M:%S"), err.code(), err);
        state.errors.push(entry);
        // Keep last 50 errors
        if state.errors.len() > 50 {
            state.errors.drain(0..1);
        }
        Ok(())
    }

    // ═══════════════════════════════════════════
    // POSITION MONITORING (background loop)
    // ═══════════════════════════════════════════

    /// Update existing position prices + auto stop-loss/take-profit
    async fn update_positions(&self) {
        let mut state = self.state.write().await;

        if state.portfolio.positions.is_empty() {
            return;
        }

        let mut positions_to_close = vec![];

        for (i, pos) in state.portfolio.positions.iter_mut().enumerate() {
            if pos.status != PositionStatus::Open {
                continue;
            }

            // TODO: fetch real midpoint from CLOB per token_id
            // self.polymarket.get_midpoint(&pos.token_id).await
            let price_change = (chrono::Utc::now().timestamp_millis() % 100) as f64 / 10000.0 - 0.005;
            let new_price = (pos.current_price + price_change).clamp(0.01, 0.99);
            pos.update_price(new_price);

            let pnl_pct = pos.pnl_pct();
            if pnl_pct <= -15.0 {
                info!("STOP LOSS triggered: {} ({:.1}%)", pos.id, pnl_pct);
                positions_to_close.push(i);
            } else if pnl_pct >= 30.0 {
                info!("TAKE PROFIT triggered: {} ({:.1}%)", pos.id, pnl_pct);
                positions_to_close.push(i);
            }
        }

        for &i in positions_to_close.iter().rev() {
            if i < state.portfolio.positions.len() {
                let pos = &mut state.portfolio.positions[i];
                pos.status = PositionStatus::Closed;

                let pnl = pos.unrealized_pnl;
                state.portfolio.capital += pos.cost_basis + pnl;
                state.portfolio.total_pnl += pnl;
                state.portfolio.daily_pnl += pnl;

                if pnl > 0.0 {
                    state.portfolio.winning_trades += 1;
                    state.portfolio.last_profit_at = Some(Utc::now());
                }

                if state.portfolio.total_trades > 0 {
                    state.portfolio.win_rate =
                        state.portfolio.winning_trades as f64 / state.portfolio.total_trades as f64 * 100.0;
                }

                // Persist auto-close to SQLite
                if let Err(e) = self.db.close_position(&pos.id, pnl) {
                    warn!("DB close_position (auto) failed: {}", e);
                }
            }
        }

        state.portfolio.positions.retain(|p| p.status == PositionStatus::Open);

        // Persist updated capital after auto-closes
        if !positions_to_close.is_empty() {
            if let Err(e) = self.db.save_capital(state.portfolio.capital) {
                warn!("DB save_capital (auto-close) failed: {}", e);
            }
            if let Err(e) = self.db.save_portfolio_stats(&state.portfolio) {
                warn!("DB save_portfolio_stats (auto-close) failed: {}", e);
            }
        }

        state.portfolio.total_pnl_pct = if state.portfolio.initial_capital > 0.0 {
            (state.portfolio.capital - state.portfolio.initial_capital) / state.portfolio.initial_capital * 100.0
        } else {
            0.0
        };

        state.portfolio.agent_state = AgentState::from_capital(state.portfolio.capital);

        let hours_since_profit = state
            .portfolio
            .last_profit_at
            .map(|t| (Utc::now() - t).num_minutes() as f64 / 60.0)
            .unwrap_or(999.0);
        let daily_target_met = state.portfolio.daily_pnl >= state.portfolio.capital * 0.005;
        state.portfolio.hunger_level =
            HungerLevel::from_hours_since_profit(hours_since_profit, daily_target_met);
    }
}
