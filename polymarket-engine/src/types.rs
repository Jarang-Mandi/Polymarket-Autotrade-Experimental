use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ═══════════════════════════════════════════
// AGENT STATE
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Survival,    // <$15 — preserve capital at all costs
    Defensive,   // $15-30 — careful, high-conviction only
    Neutral,     // $30-60 — balanced approach
    Aggressive,  // $60-120 — compound growth mode
    Apex,        // >$120 — maximum deployment
}

impl AgentState {
    pub fn from_capital(capital: f64) -> Self {
        match capital {
            c if c < 15.0 => Self::Survival,
            c if c < 30.0 => Self::Defensive,
            c if c < 60.0 => Self::Neutral,
            c if c < 120.0 => Self::Aggressive,
            _ => Self::Apex,
        }
    }

    pub fn max_position_pct(&self) -> f64 {
        match self {
            Self::Survival => 0.02,
            Self::Defensive => 0.04,
            Self::Neutral => 0.06,
            Self::Aggressive => 0.08,
            Self::Apex => 0.10,
        }
    }

    pub fn min_edge(&self) -> f64 {
        match self {
            Self::Survival => 0.15,
            Self::Defensive => 0.12,
            Self::Neutral => 0.08,
            Self::Aggressive => 0.06,
            Self::Apex => 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum HungerLevel {
    Starving,    // No profit in 48h+
    Hungry,      // No profit in 24h
    Seeking,     // Below daily target
    Satisfied,   // Meeting target
    Feasting,    // Exceeding target
}

impl HungerLevel {
    pub fn from_hours_since_profit(hours: f64, daily_target_met: bool) -> Self {
        if hours > 48.0 { return Self::Starving; }
        if hours > 24.0 { return Self::Hungry; }
        if !daily_target_met { return Self::Seeking; }
        Self::Satisfied
    }
}

// ═══════════════════════════════════════════
// POLYMARKET TYPES
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub question: Option<String>,
    pub condition_id: Option<String>,
    pub slug: Option<String>,
    pub end_date: Option<String>,
    pub category: Option<String>,
    pub volume: Option<f64>,
    pub liquidity: Option<f64>,
    pub outcome_prices: Option<String>,
    pub outcomes: Option<String>,
    pub active: Option<bool>,
    pub clob_token_ids: Option<String>,
    pub enable_order_book: Option<bool>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub last_trade_price: Option<f64>,
    pub volume_24hr: Option<f64>,
    pub neg_risk: Option<bool>,
    pub order_price_min_tick_size: Option<f64>,
}

impl Market {
    /// Extract YES/NO token IDs from clobTokenIds string
    pub fn token_ids(&self) -> Option<(String, String)> {
        let ids = self.clob_token_ids.as_ref()?;
        // Format: "[\"token_yes_id\",\"token_no_id\"]"
        let cleaned: String = ids.chars().filter(|c| !['[', ']', '"', ' '].contains(c)).collect();
        let parts: Vec<&str> = cleaned.split(',').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }

    /// Get YES price
    pub fn yes_price(&self) -> Option<f64> {
        let prices = self.outcome_prices.as_ref()?;
        let cleaned: String = prices.chars().filter(|c| !['[', ']', '"', ' '].contains(c)).collect();
        let parts: Vec<&str> = cleaned.split(',').collect();
        parts.first()?.parse().ok()
    }

    /// Get NO price
    pub fn no_price(&self) -> Option<f64> {
        let prices = self.outcome_prices.as_ref()?;
        let cleaned: String = prices.chars().filter(|c| !['[', ']', '"', ' '].contains(c)).collect();
        let parts: Vec<&str> = cleaned.split(',').collect();
        parts.get(1)?.parse().ok()
    }

    /// Calculate spread
    pub fn spread(&self) -> Option<f64> {
        Some(self.best_ask? - self.best_bid?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    GTC,  // Good Till Cancel
    GTD,  // Good Till Date
    FOK,  // Fill Or Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: Side,
    pub order_type: OrderType,
    pub neg_risk: bool,
    pub tick_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub success: bool,
    pub order_id: Option<String>,
    pub error_msg: Option<String>,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Matched,
    Mined,
    Confirmed,
    Failed,
    Retrying,
}

// ═══════════════════════════════════════════
// POSITION TRACKING
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub market_id: String,
    pub market_question: String,
    pub token_id: String,
    pub side: Side,
    pub entry_price: f64,
    pub current_price: f64,
    pub size: f64,
    pub cost_basis: f64,
    pub unrealized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub status: PositionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PositionStatus {
    Open,
    Closed,
    Liquidated,
}

impl Position {
    pub fn new(
        market_id: &str,
        market_question: &str,
        token_id: &str,
        side: Side,
        price: f64,
        size: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            market_id: market_id.to_string(),
            market_question: market_question.to_string(),
            token_id: token_id.to_string(),
            side,
            entry_price: price,
            current_price: price,
            size,
            cost_basis: price * size,
            unrealized_pnl: 0.0,
            opened_at: Utc::now(),
            status: PositionStatus::Open,
        }
    }

    pub fn update_price(&mut self, new_price: f64) {
        self.current_price = new_price;
        match self.side {
            Side::Buy => {
                self.unrealized_pnl = (new_price - self.entry_price) * self.size;
            }
            Side::Sell => {
                self.unrealized_pnl = (self.entry_price - new_price) * self.size;
            }
        }
    }

    pub fn pnl_pct(&self) -> f64 {
        if self.cost_basis == 0.0 { return 0.0; }
        self.unrealized_pnl / self.cost_basis * 100.0
    }
}

// ═══════════════════════════════════════════
// TRADE LOG
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLog {
    pub id: String,
    pub market_id: String,
    pub market_question: String,
    pub side: Side,
    pub price: f64,
    pub size: f64,
    pub cost: f64,
    pub status: TradeStatus,
    pub reason: String,
    pub edge: f64,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub api_cost: f64,
}

impl TradeLog {
    pub fn new(
        market_id: &str,
        question: &str,
        side: Side,
        price: f64,
        size: f64,
        reason: &str,
        edge: f64,
        confidence: f64,
        api_cost: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            market_id: market_id.to_string(),
            market_question: question.to_string(),
            side,
            price,
            size,
            cost: price * size,
            status: TradeStatus::Pending,
            reason: reason.to_string(),
            edge,
            confidence,
            timestamp: Utc::now(),
            api_cost,
        }
    }
}

// ═══════════════════════════════════════════
// API COST TRACKING
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCostTracker {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_write_tokens: u64,
    pub total_cost_usd: f64,
    pub daily_cost_usd: f64,
    pub daily_budget: f64,
    pub calls_today: u32,
    pub last_reset: DateTime<Utc>,
}

impl ApiCostTracker {
    pub fn new(daily_budget: f64) -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read_tokens: 0,
            total_cache_write_tokens: 0,
            total_cost_usd: 0.0,
            daily_cost_usd: 0.0,
            daily_budget,
            calls_today: 0,
            last_reset: Utc::now(),
        }
    }

    /// Claude Opus 4.6 pricing
    pub fn add_usage(&mut self, input_tokens: u64, output_tokens: u64, cache_read: u64, cache_write: u64) {
        self.total_input_tokens += input_tokens;
        self.total_output_tokens += output_tokens;
        self.total_cache_read_tokens += cache_read;
        self.total_cache_write_tokens += cache_write;

        // Pricing per MTok: Input $5, Output $25, Cache Read $0.50, Cache Write $6.25
        let cost = (input_tokens as f64 * 5.0 / 1_000_000.0)
            + (output_tokens as f64 * 25.0 / 1_000_000.0)
            + (cache_read as f64 * 0.50 / 1_000_000.0)
            + (cache_write as f64 * 6.25 / 1_000_000.0);

        self.total_cost_usd += cost;
        self.daily_cost_usd += cost;
        self.calls_today += 1;
    }

    pub fn check_daily_reset(&mut self) {
        let now = Utc::now();
        if now.date_naive() != self.last_reset.date_naive() {
            self.daily_cost_usd = 0.0;
            self.calls_today = 0;
            self.last_reset = now;
        }
    }

    pub fn budget_remaining(&self) -> f64 {
        self.daily_budget - self.daily_cost_usd
    }

    pub fn can_call(&self) -> bool {
        self.daily_cost_usd < self.daily_budget
    }

    pub fn usage_pct(&self) -> f64 {
        if self.daily_budget == 0.0 { return 100.0; }
        (self.daily_cost_usd / self.daily_budget) * 100.0
    }
}

// ═══════════════════════════════════════════
// PORTFOLIO STATE (Dashboard payload)
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    pub capital: f64,
    pub initial_capital: f64,
    pub total_pnl: f64,
    pub total_pnl_pct: f64,
    pub daily_pnl: f64,
    pub agent_state: AgentState,
    pub hunger_level: HungerLevel,
    pub positions: Vec<Position>,
    pub recent_trades: Vec<TradeLog>,
    pub api_costs: ApiCostTracker,
    pub win_rate: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub uptime_hours: f64,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub last_profit_at: Option<DateTime<Utc>>,
    pub timestamp: DateTime<Utc>,
}

// ═══════════════════════════════════════════
// OPENCLAW COMMAND TYPES
// Commands sent from OpenClaw agent to engine
// ═══════════════════════════════════════════

/// Command from OpenClaw agent to execute a trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCommand {
    pub market_id: String,
    pub side: Side,
    pub size: f64,
    pub price: f64,
    pub edge: f64,
    pub confidence: f64,
    pub reason: String,
}

/// Command from OpenClaw to close a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseCommand {
    pub position_id: String,
    pub reason: String,
}

/// Response back to OpenClaw after executing a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub trade_id: Option<String>,
    pub position_id: Option<String>,
}

/// API cost report that OpenClaw sends to engine for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCostReport {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

// ═══════════════════════════════════════════
// WEBSOCKET MESSAGE
// ═══════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl WsMessage {
    pub fn state_update(state: &PortfolioState) -> Self {
        Self {
            msg_type: "state_update".into(),
            payload: serde_json::to_value(state).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn trade_event(trade: &TradeLog) -> Self {
        Self {
            msg_type: "trade".into(),
            payload: serde_json::to_value(trade).unwrap_or_default(),
            timestamp: Utc::now(),
        }
    }

    pub fn alert(level: &str, message: &str) -> Self {
        Self {
            msg_type: "alert".into(),
            payload: serde_json::json!({ "level": level, "message": message }),
            timestamp: Utc::now(),
        }
    }
}
