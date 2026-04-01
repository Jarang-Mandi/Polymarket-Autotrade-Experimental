use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::db::Database;
use crate::error::{log_error, EngineError, EngineResult};
use crate::polymarket::PolymarketClient;
use crate::types::*;

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
    pub risk_thresholds: RiskThresholds,
    pub pending_risk_threshold_proposal: Option<RiskThresholdProposal>,
    pub engine_running: bool,
    pub last_scan: Option<chrono::DateTime<Utc>>,
    pub last_command: Option<chrono::DateTime<Utc>>,
    pub errors: Vec<String>,
    // Arbitrage
    pub arb_opportunities: Vec<ArbitrageOpportunity>,
    pub arb_config: ArbConfig,
    pub arb_stats: ArbStats,
    pub active_arb_count: usize,
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
            info!(
                "  Capital: ${:.2} (initial: ${:.2})",
                capital, config.initial_capital
            );
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
                } else {
                    0.0
                },
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
            risk_thresholds: RiskThresholds {
                stop_loss_pct: config.default_stop_loss_pct,
                take_profit_pct: config.default_take_profit_pct,
                auto_close_enabled: config.position_auto_close_enabled,
            },
            pending_risk_threshold_proposal: None,
            engine_running: false,
            last_scan: None,
            last_command: None,
            errors: vec![],
            arb_opportunities: vec![],
            arb_config: ArbConfig::default(),
            arb_stats: ArbStats::default(),
            active_arb_count: 0,
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
        info!(
            "Exit policy: SL {:.1}% / TP {:.1}% / auto_close={} (AI-managed close recommended)",
            self.config.default_stop_loss_pct,
            self.config.default_take_profit_pct,
            self.config.position_auto_close_enabled
        );

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

        // Main loop: scan markets + update positions + arb scan + periodic DB snapshot
        let mut scan_tick = time::interval(scan_interval);
        let mut position_tick = time::interval(position_interval);
        let mut snapshot_tick = time::interval(Duration::from_secs(3600)); // hourly
        let mut arb_tick = time::interval(Duration::from_secs(60)); // arb scan every 60s

        loop {
            tokio::select! {
                _ = scan_tick.tick() => {
                    self.market_scan_cycle().await;
                }
                _ = position_tick.tick() => {
                    self.update_positions().await;
                }
                _ = arb_tick.tick() => {
                    self.arb_scan_cycle().await;
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
                info!(
                    "Cached {} markets (available via GET /api/markets)",
                    state.cached_markets.len()
                );
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
    fn validate_trade_command(
        &self,
        cmd: &TradeCommand,
        capital: f64,
        state: &AgentState,
    ) -> EngineResult<()> {
        if cmd.price <= 0.0 || cmd.price >= 1.0 {
            return Err(EngineError::BadRequest(format!(
                "Price {:.3} must be between 0 and 1",
                cmd.price
            )));
        }
        if cmd.size <= 0.0 {
            return Err(EngineError::BadRequest("Size must be positive".into()));
        }
        if cmd.market_id.is_empty() {
            return Err(EngineError::BadRequest("market_id is required".into()));
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
        info!(
            "TRADE COMMAND: {:?} {} @ {:.3} on {}",
            cmd.side, cmd.size, cmd.price, cmd.market_id
        );

        // Read state for validation
        let state = self.state.read().await;
        let mut market = state
            .cached_markets
            .iter()
            .find(|m| m.id == cmd.market_id)
            .cloned();
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
        // If the market is not in current cached scans (e.g. closed market replay),
        // fetch it directly by ID from Gamma so command-driven dry-runs can proceed.
        if market.is_none() {
            if let Some(fetched) = self.polymarket.get_market_by_id(&cmd.market_id).await? {
                info!(
                    "Market {} loaded directly from Gamma (not in cache)",
                    cmd.market_id
                );
                let mut state = self.state.write().await;
                if !state.cached_markets.iter().any(|m| m.id == fetched.id) {
                    state.cached_markets.push(fetched.clone());
                }
                market = Some(fetched);
            }
        }

        let market = market.ok_or_else(|| EngineError::MarketNotFound(cmd.market_id.clone()))?;
        let (yes_id, no_id) = market
            .token_ids()
            .ok_or_else(|| EngineError::NoTokenIds(cmd.market_id.clone()))?;
        let token_id = match cmd.side {
            Side::Buy => yes_id,
            Side::Sell => no_id,
        };
        let question = market.question.clone().unwrap_or_default();

        let tick_size = market
            .order_price_min_tick_size
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

        info!(
            "TRADE EXECUTED: {:?} ${:.2} @ {:.3}",
            cmd.side, actual_size, cmd.price
        );

        Ok(CommandResponse {
            success: true,
            message: format!(
                "Executed {:?} ${:.2} @ {:.3}",
                cmd.side, actual_size, cmd.price
            ),
            trade_id: Some(trade_id),
            position_id: Some(position_id),
        })
    }

    /// Close a position by command from OpenClaw
    pub async fn handle_close_command(&self, cmd: CloseCommand) -> EngineResult<CommandResponse> {
        info!(
            "CLOSE COMMAND: position {} — {}",
            cmd.position_id, cmd.reason
        );

        if cmd.position_id.is_empty() {
            return Err(EngineError::BadRequest("position_id is required".into()));
        }

        let mut state = self.state.write().await;

        let pos_idx = state
            .portfolio
            .positions
            .iter()
            .position(|p| p.id == cmd.position_id)
            .ok_or_else(|| EngineError::PositionNotFound(cmd.position_id.clone()))?;

        let (pid, cost_basis, pnl, pnl_pct) = {
            let pos = &mut state.portfolio.positions[pos_idx];
            pos.status = PositionStatus::Closed;
            (
                pos.id.clone(),
                pos.cost_basis,
                pos.unrealized_pnl,
                pos.pnl_pct(),
            )
        };

        state.portfolio.capital += cost_basis + pnl;
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

        let msg = format!("Closed position. PnL: ${:.4} ({:.1}%)", pnl, pnl_pct);

        // Persist close to SQLite
        if let Err(e) = self.db.close_position(&pid, pnl) {
            warn!("DB close_position failed: {}", e);
        }

        state
            .portfolio
            .positions
            .retain(|p| p.status == PositionStatus::Open);
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

    /// Create a pending SL/TP policy proposal.
    /// Nothing is applied until user confirmation arrives.
    pub async fn propose_risk_thresholds(
        &self,
        cmd: ProposeRiskThresholdsCommand,
    ) -> EngineResult<RiskThresholdsUpdateResponse> {
        if cmd.stop_loss_pct <= 0.0 || cmd.stop_loss_pct >= 100.0 {
            return Err(EngineError::BadRequest(
                "stop_loss_pct must be between 0 and 100".into(),
            ));
        }
        if cmd.take_profit_pct <= 0.0 || cmd.take_profit_pct >= 100.0 {
            return Err(EngineError::BadRequest(
                "take_profit_pct must be between 0 and 100".into(),
            ));
        }

        let proposed_by = if cmd.proposed_by.trim().is_empty() {
            "ai"
        } else {
            cmd.proposed_by.trim()
        };
        let reason = if cmd.reason.trim().is_empty() {
            "AI requested SL/TP policy adjustment"
        } else {
            cmd.reason.trim()
        };

        let mut state = self.state.write().await;
        let proposal = RiskThresholdProposal::new(
            cmd.stop_loss_pct,
            cmd.take_profit_pct,
            cmd.auto_close_enabled,
            proposed_by,
            reason,
        );
        let proposal_id = proposal.id.clone();
        state.pending_risk_threshold_proposal = Some(proposal.clone());

        Ok(RiskThresholdsUpdateResponse {
            success: true,
            message: "Risk threshold proposal created. Awaiting explicit user confirmation.".into(),
            requires_confirmation: true,
            applied: false,
            proposal_id: Some(proposal_id),
            thresholds: state.risk_thresholds.clone(),
            pending_proposal: Some(proposal),
        })
    }

    /// Confirm or reject a pending SL/TP policy proposal.
    pub async fn confirm_risk_thresholds(
        &self,
        cmd: ConfirmRiskThresholdsCommand,
    ) -> EngineResult<RiskThresholdsUpdateResponse> {
        if cmd.proposal_id.trim().is_empty() {
            return Err(EngineError::BadRequest("proposal_id is required".into()));
        }

        let mut state = self.state.write().await;
        let pending = state
            .pending_risk_threshold_proposal
            .clone()
            .ok_or_else(|| {
                EngineError::BadRequest("No pending SL/TP proposal to confirm".into())
            })?;

        if pending.id != cmd.proposal_id {
            return Err(EngineError::BadRequest(format!(
                "Proposal ID mismatch. Pending proposal is {}",
                pending.id
            )));
        }

        if !cmd.approved {
            state.pending_risk_threshold_proposal = None;
            return Ok(RiskThresholdsUpdateResponse {
                success: true,
                message: format!(
                    "SL/TP proposal {} rejected by {}",
                    cmd.proposal_id, cmd.confirmed_by
                ),
                requires_confirmation: false,
                applied: false,
                proposal_id: Some(cmd.proposal_id),
                thresholds: state.risk_thresholds.clone(),
                pending_proposal: None,
            });
        }

        state.risk_thresholds.stop_loss_pct = pending.proposed_stop_loss_pct;
        state.risk_thresholds.take_profit_pct = pending.proposed_take_profit_pct;
        state.risk_thresholds.auto_close_enabled = pending.proposed_auto_close_enabled;
        state.pending_risk_threshold_proposal = None;

        Ok(RiskThresholdsUpdateResponse {
            success: true,
            message: format!(
                "SL/TP policy updated by {} (SL {:.1}% / TP {:.1}% / auto_close={})",
                cmd.confirmed_by,
                state.risk_thresholds.stop_loss_pct,
                state.risk_thresholds.take_profit_pct,
                state.risk_thresholds.auto_close_enabled
            ),
            requires_confirmation: false,
            applied: true,
            proposal_id: Some(cmd.proposal_id),
            thresholds: state.risk_thresholds.clone(),
            pending_proposal: None,
        })
    }

    /// AI-managed exit reasoning based on current position metrics + active SL/TP policy.
    /// If execute_close=true and recommendation is CLOSE, the engine sends a close command.
    pub async fn ai_exit_review(
        &self,
        cmd: AiExitReviewCommand,
    ) -> EngineResult<AiExitReviewResponse> {
        if cmd.position_id.trim().is_empty() {
            return Err(EngineError::BadRequest("position_id is required".into()));
        }

        let (position, thresholds, market_end_date, position_count, capital) = {
            let state = self.state.read().await;
            let position = state
                .portfolio
                .positions
                .iter()
                .find(|p| p.id == cmd.position_id)
                .cloned()
                .ok_or_else(|| EngineError::PositionNotFound(cmd.position_id.clone()))?;
            let end_date = state
                .cached_markets
                .iter()
                .find(|m| m.id == position.market_id)
                .and_then(|m| m.end_date.clone());
            (
                position,
                state.risk_thresholds.clone(),
                end_date,
                state.portfolio.positions.len(),
                state.portfolio.capital,
            )
        };

        let pnl_pct = position.pnl_pct();
        let mut should_close = false;
        let mut recommended_reason = "HOLD_SKILL".to_string();
        let mut reasoning = vec![
            format!(
                "quant-risk-engine: pnl={:.2}% vs SL=-{:.2}% TP=+{:.2}%",
                pnl_pct, thresholds.stop_loss_pct, thresholds.take_profit_pct
            ),
            format!(
                "risk-allocation: open_positions={} available_capital=${:.2}",
                position_count, capital
            ),
            format!(
                "execute-mode: auto_close_enabled={} (manual AI close preferred)",
                thresholds.auto_close_enabled
            ),
        ];

        if pnl_pct <= -thresholds.stop_loss_pct {
            should_close = true;
            recommended_reason = "AI_STOP_LOSS_SKILL".into();
        } else if pnl_pct >= thresholds.take_profit_pct {
            should_close = true;
            recommended_reason = "AI_TAKE_PROFIT_SKILL".into();
        } else if let Some(end_date) = market_end_date {
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(&end_date) {
                let secs_left = (parsed.with_timezone(&Utc) - Utc::now()).num_seconds();
                reasoning.push(format!(
                    "market-regime: secs_left={} for market resolution window",
                    secs_left
                ));
                if secs_left <= 120 {
                    should_close = true;
                    recommended_reason = "AI_TIME_BASED_EXIT_SKILL".into();
                }
            }
        }

        if let Some(ctx) = cmd.context.as_ref() {
            if !ctx.trim().is_empty() {
                reasoning.push(format!("ai-context: {}", ctx.trim()));
            }
        }

        let mut executed_close = false;
        let mut close_response = None;
        if cmd.execute_close && should_close {
            let close_cmd = CloseCommand {
                position_id: cmd.position_id.clone(),
                reason: recommended_reason.clone(),
            };
            let resp = self.handle_close_command(close_cmd).await?;
            executed_close = resp.success;
            close_response = Some(resp);
        }

        Ok(AiExitReviewResponse {
            success: true,
            position_id: cmd.position_id,
            should_close,
            recommended_reason,
            reasoning,
            entry_price: position.entry_price,
            current_price: position.current_price,
            pnl_pct,
            stop_loss_pct: thresholds.stop_loss_pct,
            take_profit_pct: thresholds.take_profit_pct,
            auto_close_enabled: thresholds.auto_close_enabled,
            executed_close,
            close_response,
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
        let entry = format!(
            "[{}] {}: {}",
            Utc::now().format("%H:%M:%S"),
            err.code(),
            err
        );
        state.errors.push(entry);
        // Keep last 50 errors
        if state.errors.len() > 50 {
            state.errors.drain(0..1);
        }
        Ok(())
    }

    // ═══════════════════════════════════════════
    // ARBITRAGE SCANNER (background loop)
    // ═══════════════════════════════════════════

    /// Full arb scan cycle — runs every 60s.
    /// Scans binary markets for YES+NO < $1, multi-outcome events, and wide spreads.
    async fn arb_scan_cycle(&self) {
        let arb_enabled = {
            let state = self.state.read().await;
            state.arb_config.enabled
        };
        if !arb_enabled {
            return;
        }

        info!("--- ARB SCAN ---");
        let mut opportunities: Vec<ArbitrageOpportunity> = vec![];

        // 1. Binary market arb scan
        match self.scan_binary_arbs().await {
            Ok(mut arbs) => {
                info!("Binary arb scan: {} opportunities", arbs.len());
                opportunities.append(&mut arbs);
            }
            Err(e) => {
                warn!("Binary arb scan failed: {}", e);
            }
        }

        // 2. Multi-outcome neg-risk arb scan
        match self.scan_multi_outcome_arbs().await {
            Ok(mut arbs) => {
                info!("Multi-outcome arb scan: {} opportunities", arbs.len());
                opportunities.append(&mut arbs);
            }
            Err(e) => {
                warn!("Multi-outcome arb scan failed: {}", e);
            }
        }

        // 3. Spread capture scan
        match self.scan_spread_capture().await {
            Ok(mut arbs) => {
                info!("Spread capture scan: {} opportunities", arbs.len());
                opportunities.append(&mut arbs);
            }
            Err(e) => {
                warn!("Spread capture scan failed: {}", e);
            }
        }

        // Sort by profit_pct * liquidity_score (best first)
        opportunities.sort_by(|a, b| {
            let score_a = a.profit_pct * a.liquidity_score;
            let score_b = b.profit_pct * b.liquidity_score;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep top 20
        opportunities.truncate(20);

        let found_count = opportunities.len() as u64;

        let mut state = self.state.write().await;
        state.arb_opportunities = opportunities;
        state.arb_stats.last_scan_at = Some(Utc::now());
        state.arb_stats.opportunities_found += found_count;

        let market_count = state.cached_markets.len() as u64;
        state.arb_stats.markets_scanned += market_count;

        if found_count > 0 {
            info!("ARB SCAN COMPLETE: {} opportunities detected", found_count);
        }
    }

    /// Scan all cached binary markets for YES_ask + NO_ask < $1.00
    async fn scan_binary_arbs(&self) -> EngineResult<Vec<ArbitrageOpportunity>> {
        let (markets, config) = {
            let state = self.state.read().await;
            let markets: Vec<Market> = state
                .cached_markets
                .iter()
                .filter(|m| {
                    // Only binary markets (not neg-risk multi-outcome)
                    !m.neg_risk.unwrap_or(false)
                        && m.enable_order_book.unwrap_or(false)
                        && m.clob_token_ids.is_some()
                        && m.liquidity.unwrap_or(0.0) >= state.arb_config.min_liquidity
                })
                .cloned()
                .collect();
            let config = state.arb_config.clone();
            (markets, config)
        };

        let mut arbs = vec![];

        for market in &markets {
            let (yes_id, no_id) = match market.token_ids() {
                Some(ids) => ids,
                None => {
                    warn!(
                        "Skipping binary market {}: missing YES/NO token IDs",
                        market.id
                    );
                    continue;
                }
            };

            // Try to get real orderbook prices
            let (yes_ask, no_ask) = match self.polymarket.get_binary_prices(&yes_id, &no_id).await {
                Ok(prices) => prices,
                Err(e) => {
                    warn!(
                        "Skipping binary market {}: orderbook unavailable ({})",
                        market.id, e
                    );
                    continue;
                } // Skip markets with no orderbook depth
            };

            let total_cost = yes_ask + no_ask;
            if !total_cost.is_finite() || total_cost <= 0.0 {
                warn!(
                    "Skipping binary market {}: invalid total_cost {:.6}",
                    market.id, total_cost
                );
                continue;
            }
            let profit_per_unit = 1.0 - total_cost;
            let profit_pct = (profit_per_unit / total_cost) * 100.0;

            if profit_pct >= config.min_profit_pct {
                let question = market.question.clone().unwrap_or_else(|| market.id.clone());
                let liquidity = market.liquidity.unwrap_or(0.0);

                let opp = ArbitrageOpportunity {
                    id: uuid::Uuid::new_v4().to_string(),
                    arb_type: ArbitrageType::BinaryMispricing,
                    legs: vec![
                        ArbLeg {
                            market_id: market.id.clone(),
                            market_question: question.clone(),
                            token_id: yes_id.clone(),
                            side: Side::Buy,
                            price: yes_ask,
                            size: 1.0, // normalized; actual size calculated at execution
                            neg_risk: false,
                        },
                        ArbLeg {
                            market_id: market.id.clone(),
                            market_question: question,
                            token_id: no_id.clone(),
                            side: Side::Buy,
                            price: no_ask,
                            size: 1.0,
                            neg_risk: false,
                        },
                    ],
                    total_cost,
                    guaranteed_payout: 1.0,
                    profit: profit_per_unit,
                    profit_pct,
                    liquidity_score: (liquidity / 10_000.0).min(10.0),
                    detected_at: Utc::now(),
                    status: ArbStatus::Detected,
                    event_slug: None,
                };

                info!(
                    "BINARY ARB DETECTED: {} | YES={:.3} + NO={:.3} = {:.3} | profit={:.2}%",
                    market.id, yes_ask, no_ask, total_cost, profit_pct
                );

                arbs.push(opp);
            }
        }

        Ok(arbs)
    }

    /// Scan neg-risk multi-outcome events for sum(YES asks) < $1.00
    async fn scan_multi_outcome_arbs(&self) -> EngineResult<Vec<ArbitrageOpportunity>> {
        let config = {
            let state = self.state.read().await;
            state.arb_config.clone()
        };

        // Fetch neg-risk events from Gamma
        let events = match self.polymarket.get_neg_risk_events(50).await {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to fetch neg-risk events: {}", e);
                return Ok(vec![]);
            }
        };

        let mut arbs = vec![];

        for event in &events {
            let slug = event["slug"].as_str().unwrap_or("");
            let title = event["title"].as_str().unwrap_or(slug);
            let markets_raw = match event["markets"].as_array() {
                Some(m) => m,
                None => continue,
            };

            if markets_raw.len() < 2 || markets_raw.len() > 8 {
                continue; // Skip too few or too many outcomes
            }

            // Parse markets and get YES ask for each
            let mut legs: Vec<ArbLeg> = vec![];
            let mut sum_yes_asks = 0.0;
            let mut min_liquidity_leg = f64::MAX;
            let mut all_ok = true;

            for m_raw in markets_raw {
                let market = match crate::polymarket::parse_market(m_raw) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Skipping event {}: market parse failed ({})", slug, e);
                        all_ok = false;
                        break;
                    }
                };

                if !market.enable_order_book.unwrap_or(false) {
                    all_ok = false;
                    break;
                }

                let (yes_id, _no_id) = match market.token_ids() {
                    Some(ids) => ids,
                    None => {
                        warn!(
                            "Skipping event {} market {}: missing token IDs",
                            slug, market.id
                        );
                        all_ok = false;
                        break;
                    }
                };

                // Get actual orderbook ask price
                let (ask_price, _depth) = match self.polymarket.get_ask_depth(&yes_id).await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!(
                            "Skipping event {} market {}: ask depth unavailable ({})",
                            slug, market.id, e
                        );
                        all_ok = false;
                        break;
                    }
                };

                let liq = market.liquidity.unwrap_or(0.0);
                if liq < config.min_liquidity {
                    all_ok = false;
                    break;
                }
                min_liquidity_leg = min_liquidity_leg.min(liq);

                sum_yes_asks += ask_price;

                legs.push(ArbLeg {
                    market_id: market.id.clone(),
                    market_question: market.question.clone().unwrap_or_default(),
                    token_id: yes_id,
                    side: Side::Buy,
                    price: ask_price,
                    size: 1.0,
                    neg_risk: true,
                });
            }

            if !all_ok || legs.len() < 2 {
                continue;
            }

            if !sum_yes_asks.is_finite() || sum_yes_asks <= 0.0 || !min_liquidity_leg.is_finite() {
                warn!(
                    "Skipping event {}: invalid aggregate values (sum_yes_asks={:.6}, min_liquidity_leg={:.2})",
                    slug, sum_yes_asks, min_liquidity_leg
                );
                continue;
            }

            // Check for under-priced arb: sum < 1.00
            let profit_per_unit = 1.0 - sum_yes_asks;
            let profit_pct = if sum_yes_asks > 0.0 {
                (profit_per_unit / sum_yes_asks) * 100.0
            } else {
                0.0
            };

            if profit_pct >= config.min_profit_pct && sum_yes_asks < 1.0 {
                info!(
                    "MULTI-OUTCOME ARB DETECTED: '{}' | {} markets | sum={:.4} | profit={:.2}%",
                    title,
                    legs.len(),
                    sum_yes_asks,
                    profit_pct
                );

                arbs.push(ArbitrageOpportunity {
                    id: uuid::Uuid::new_v4().to_string(),
                    arb_type: ArbitrageType::MultiOutcomeUnderpriced,
                    legs,
                    total_cost: sum_yes_asks,
                    guaranteed_payout: 1.0,
                    profit: profit_per_unit,
                    profit_pct,
                    liquidity_score: (min_liquidity_leg / 10_000.0).min(10.0),
                    detected_at: Utc::now(),
                    status: ArbStatus::Detected,
                    event_slug: Some(slug.to_string()),
                });
            }

            // TODO: Check for over-priced arb (sum of bids > 1.00) — requires selling YES.
            // Selling requires existing inventory or margin, so skipped for $50 bankroll.
        }

        Ok(arbs)
    }

    /// Scan liquid markets for wide bid-ask spreads (market-making lite)
    async fn scan_spread_capture(&self) -> EngineResult<Vec<ArbitrageOpportunity>> {
        let (markets, config) = {
            let state = self.state.read().await;
            let markets: Vec<Market> = state
                .cached_markets
                .iter()
                .filter(|m| {
                    m.enable_order_book.unwrap_or(false)
                        && m.volume_24hr.unwrap_or(0.0) >= state.arb_config.min_volume_24h_spread
                        && m.clob_token_ids.is_some()
                })
                .cloned()
                .collect();
            let config = state.arb_config.clone();
            (markets, config)
        };

        let mut arbs = vec![];

        for market in &markets {
            let (yes_id, _no_id) = match market.token_ids() {
                Some(ids) => ids,
                None => continue,
            };

            // Get actual spread from orderbook
            let (bid, ask) = match self.polymarket.get_spread(&yes_id).await {
                Ok(s) => s,
                Err(_) => continue,
            };

            let spread = ask - bid;

            if spread >= config.min_spread {
                let buy_price = bid + 0.01; // improve bid by 1 tick
                let sell_price = ask - 0.01; // improve ask by 1 tick
                let expected_profit = sell_price - buy_price;
                let profit_pct = if buy_price > 0.0 {
                    (expected_profit / buy_price) * 100.0
                } else {
                    0.0
                };

                if profit_pct > 0.5 {
                    let question = market.question.clone().unwrap_or_else(|| market.id.clone());
                    let volume = market.volume_24hr.unwrap_or(0.0);

                    arbs.push(ArbitrageOpportunity {
                        id: uuid::Uuid::new_v4().to_string(),
                        arb_type: ArbitrageType::SpreadCapture,
                        legs: vec![
                            ArbLeg {
                                market_id: market.id.clone(),
                                market_question: question.clone(),
                                token_id: yes_id.clone(),
                                side: Side::Buy,
                                price: buy_price,
                                size: 1.0,
                                neg_risk: market.neg_risk.unwrap_or(false),
                            },
                            ArbLeg {
                                market_id: market.id.clone(),
                                market_question: question,
                                token_id: yes_id.clone(),
                                side: Side::Sell,
                                price: sell_price,
                                size: 1.0,
                                neg_risk: market.neg_risk.unwrap_or(false),
                            },
                        ],
                        total_cost: buy_price,
                        guaranteed_payout: sell_price,
                        profit: expected_profit,
                        profit_pct,
                        liquidity_score: (volume / 100_000.0).min(10.0),
                        detected_at: Utc::now(),
                        status: ArbStatus::Detected,
                        event_slug: None,
                    });
                }
            }
        }

        Ok(arbs)
    }

    /// Execute a detected arbitrage opportunity.
    /// Places orders for all legs and tracks execution.
    pub async fn handle_execute_arb(
        &self,
        cmd: ExecuteArbCommand,
    ) -> EngineResult<ArbExecutionResponse> {
        info!("ARB EXECUTE: {}", cmd.opportunity_id);

        let (opp, config, capital) = {
            let state = self.state.read().await;

            let opp = state
                .arb_opportunities
                .iter()
                .find(|o| o.id == cmd.opportunity_id)
                .cloned()
                .ok_or_else(|| {
                    EngineError::BadRequest(format!(
                        "Arb opportunity {} not found",
                        cmd.opportunity_id
                    ))
                })?;

            if opp.status != ArbStatus::Detected {
                return Err(EngineError::BadRequest(format!(
                    "Arb opportunity {} is not executable (status: {:?})",
                    cmd.opportunity_id, opp.status
                )));
            }

            (opp, state.arb_config.clone(), state.portfolio.capital)
        };

        let max_affordable = capital * 0.8; // never use more than 80% of capital
        let unit_cost = opp.total_cost;
        if unit_cost <= 0.0 || !unit_cost.is_finite() {
            return Err(EngineError::BadRequest("Invalid arb cost".into()));
        }
        if opp.legs.is_empty() {
            return Err(EngineError::BadRequest(
                "Arb opportunity has no legs".into(),
            ));
        }

        let max_units = (max_affordable / unit_cost)
            .floor()
            .min(config.max_arb_size / unit_cost);
        if !max_units.is_finite() || max_units <= 0.0 {
            return Err(EngineError::BadRequest(
                "Insufficient capital for arb minimum unit".into(),
            ));
        }
        let actual_units = cmd
            .size_override
            .map(|s| s.min(max_units).max(1.0))
            .unwrap_or_else(|| max_units.min(5.0).max(1.0)); // default: up to $5 worth

        let total_cost = actual_units * unit_cost;
        let expected_profit = actual_units * opp.profit;

        if !total_cost.is_finite() || total_cost <= 0.0 {
            return Err(EngineError::BadRequest(
                "Invalid computed arb total_cost".into(),
            ));
        }

        if total_cost > capital * 0.95 {
            return Err(EngineError::InsufficientCapital {
                cost: total_cost,
                capital,
                pct: 95.0,
            });
        }

        // Reserve a concurrency slot and transition to executing.
        {
            let mut state = self.state.write().await;

            if state.active_arb_count >= state.arb_config.max_concurrent_arbs {
                return Err(EngineError::BadRequest(format!(
                    "Max concurrent arbs ({}) reached",
                    state.arb_config.max_concurrent_arbs
                )));
            }

            let opp_mut = state
                .arb_opportunities
                .iter_mut()
                .find(|o| o.id == cmd.opportunity_id)
                .ok_or_else(|| {
                    EngineError::BadRequest(format!(
                        "Arb opportunity {} no longer available",
                        cmd.opportunity_id
                    ))
                })?;

            if opp_mut.status != ArbStatus::Detected {
                return Err(EngineError::BadRequest(format!(
                    "Arb opportunity {} is no longer executable (status: {:?})",
                    cmd.opportunity_id, opp_mut.status
                )));
            }

            opp_mut.status = ArbStatus::Executing;
            state.active_arb_count += 1;
        }

        // Execute each leg
        let mut legs_filled = 0;
        let mut trade_ids = vec![];

        for leg in &opp.legs {
            let tick_size = "0.01".to_string(); // default tick

            let order = OrderRequest {
                token_id: leg.token_id.clone(),
                price: leg.price,
                size: actual_units * leg.size,
                side: leg.side,
                order_type: OrderType::GTC,
                neg_risk: leg.neg_risk,
                tick_size,
            };

            match self.polymarket.place_order(&order).await {
                Ok(resp) if resp.success => {
                    legs_filled += 1;
                    if let Some(oid) = resp.order_id {
                        trade_ids.push(oid);
                    }

                    // Log as trade
                    let trade = TradeLog::new(
                        &leg.market_id,
                        &leg.market_question,
                        leg.side,
                        leg.price,
                        actual_units * leg.size,
                        &format!("arb:{}", opp.arb_type.label()),
                        opp.profit_pct / 100.0,
                        1.0, // high confidence for arb
                        0.0,
                    );
                    if let Err(e) = self.db.insert_trade(&trade) {
                        warn!("DB insert_trade (arb leg) failed: {}", e);
                    }

                    let mut state = self.state.write().await;
                    state.portfolio.recent_trades.push(trade);
                    state.portfolio.total_trades += 1;
                    state.portfolio.last_trade_at = Some(Utc::now());
                }
                Ok(resp) => {
                    let msg = resp.error_msg.unwrap_or_else(|| "rejected".into());
                    warn!("Arb leg failed for {}: {}", leg.market_id, msg);
                }
                Err(e) => {
                    warn!("Arb leg order failed: {}", e);
                }
            }
        }

        let all_filled = legs_filled == opp.legs.len();

        // Update capital and stats
        {
            let mut state = self.state.write().await;

            if all_filled {
                state.portfolio.capital -= total_cost;
                state.arb_stats.opportunities_executed += 1;
                state.arb_stats.total_profit += expected_profit;
                let n = state.arb_stats.opportunities_executed as f64;
                state.arb_stats.avg_profit_pct = if n <= 1.0 {
                    opp.profit_pct
                } else {
                    ((state.arb_stats.avg_profit_pct * (n - 1.0)) + opp.profit_pct) / n
                };

                if let Some(o) = state
                    .arb_opportunities
                    .iter_mut()
                    .find(|o| o.id == cmd.opportunity_id)
                {
                    o.status = ArbStatus::Filled;
                }
            } else if legs_filled > 0 {
                state.arb_stats.partial_fills += 1;
                // Partial fill — capital deducted for filled legs only
                let filled_cost = (legs_filled as f64 / opp.legs.len() as f64) * total_cost;
                state.portfolio.capital -= filled_cost;

                if let Some(o) = state
                    .arb_opportunities
                    .iter_mut()
                    .find(|o| o.id == cmd.opportunity_id)
                {
                    o.status = ArbStatus::PartialFill;
                }
            } else {
                if let Some(o) = state
                    .arb_opportunities
                    .iter_mut()
                    .find(|o| o.id == cmd.opportunity_id)
                {
                    o.status = ArbStatus::Failed;
                }
            }

            // Release concurrency slot.
            state.active_arb_count = state.active_arb_count.saturating_sub(1);

            // Keep active detections only; executed terminal states are ephemeral and
            // should not block future opportunities.
            state
                .arb_opportunities
                .retain(|o| matches!(o.status, ArbStatus::Detected | ArbStatus::Executing));

            state.last_command = Some(Utc::now());

            if let Err(e) = self.db.save_capital(state.portfolio.capital) {
                warn!("DB save_capital (arb) failed: {}", e);
            }
        }

        let msg = if all_filled {
            format!(
                "Arb executed: {} legs filled, cost=${:.4}, expected profit=${:.4} ({:.2}%)",
                legs_filled, total_cost, expected_profit, opp.profit_pct
            )
        } else if legs_filled > 0 {
            format!(
                "Arb partial: {}/{} legs filled ({})",
                legs_filled,
                opp.legs.len(),
                opp.arb_type.label()
            )
        } else {
            format!(
                "Arb failed: 0/{} legs filled ({})",
                opp.legs.len(),
                opp.arb_type.label()
            )
        };

        info!("{}", msg);

        Ok(ArbExecutionResponse {
            success: all_filled,
            message: msg,
            legs_filled,
            legs_total: opp.legs.len(),
            total_cost: if all_filled { total_cost } else { 0.0 },
            expected_profit: if all_filled { expected_profit } else { 0.0 },
            trade_ids,
        })
    }

    // ═══════════════════════════════════════════
    // POSITION MONITORING (background loop)
    // ═══════════════════════════════════════════

    /// Update existing position prices.
    /// If auto_close_enabled=false, exits are fully manual/AI-commanded.
    /// If auto_close_enabled=true, background SL/TP auto-close is allowed.
    async fn update_positions(&self) {
        let mut state = self.state.write().await;

        if state.portfolio.positions.is_empty() {
            return;
        }

        let thresholds = state.risk_thresholds.clone();
        let mut positions_to_close: Vec<String> = vec![];

        for pos in &mut state.portfolio.positions {
            if pos.status != PositionStatus::Open {
                continue;
            }

            // TODO: fetch real midpoint from CLOB per token_id
            // self.polymarket.get_midpoint(&pos.token_id).await
            let price_change =
                (chrono::Utc::now().timestamp_millis() % 100) as f64 / 10000.0 - 0.005;
            let new_price = (pos.current_price + price_change).clamp(0.01, 0.99);
            pos.update_price(new_price);

            if thresholds.auto_close_enabled {
                let pnl_pct = pos.pnl_pct();
                if pnl_pct <= -thresholds.stop_loss_pct {
                    info!(
                        "AUTO STOP LOSS triggered: {} ({:.1}% <= -{:.1}%)",
                        pos.id, pnl_pct, thresholds.stop_loss_pct
                    );
                    positions_to_close.push(pos.id.clone());
                } else if pnl_pct >= thresholds.take_profit_pct {
                    info!(
                        "AUTO TAKE PROFIT triggered: {} ({:.1}% >= +{:.1}%)",
                        pos.id, pnl_pct, thresholds.take_profit_pct
                    );
                    positions_to_close.push(pos.id.clone());
                }
            }
        }

        for close_id in &positions_to_close {
            if let Some(pos_idx) = state
                .portfolio
                .positions
                .iter()
                .position(|p| p.id == *close_id && p.status == PositionStatus::Open)
            {
                let (pid, cost_basis, pnl) = {
                    let pos = &mut state.portfolio.positions[pos_idx];
                    pos.status = PositionStatus::Closed;
                    (pos.id.clone(), pos.cost_basis, pos.unrealized_pnl)
                };

                state.portfolio.capital += cost_basis + pnl;
                state.portfolio.total_pnl += pnl;
                state.portfolio.daily_pnl += pnl;

                if pnl > 0.0 {
                    state.portfolio.winning_trades += 1;
                    state.portfolio.last_profit_at = Some(Utc::now());
                }

                if state.portfolio.total_trades > 0 {
                    state.portfolio.win_rate = state.portfolio.winning_trades as f64
                        / state.portfolio.total_trades as f64
                        * 100.0;
                }

                // Persist auto-close to SQLite
                if let Err(e) = self.db.close_position(&pid, pnl) {
                    warn!("DB close_position (auto) failed: {}", e);
                }
            }
        }

        state
            .portfolio
            .positions
            .retain(|p| p.status == PositionStatus::Open);

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
            (state.portfolio.capital - state.portfolio.initial_capital)
                / state.portfolio.initial_capital
                * 100.0
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
