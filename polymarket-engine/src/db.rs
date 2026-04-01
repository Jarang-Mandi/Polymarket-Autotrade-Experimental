use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use tracing::{info, warn};

use crate::error::{EngineError, EngineResult};
use crate::types::*;

// ═══════════════════════════════════════════
// DATABASE LAYER — SQLite persistence
// Write-through: memory is primary, DB is async backup
// On restart: load last state from DB into memory
// ═══════════════════════════════════════════

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) SQLite database and run migrations
    pub fn open(path: &str) -> EngineResult<Self> {
        // Ensure directory exists
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| EngineError::Database(format!("Cannot create db directory: {}", e)))?;
        }

        let conn = Connection::open(path)
            .map_err(|e| EngineError::Database(format!("Failed to open {}: {}", path, e)))?;

        // Performance pragmas for single-writer trading engine
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -2000;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| EngineError::Database(format!("Pragma failed: {}", e)))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;

        info!("SQLite database ready: {}", path);
        Ok(db)
    }

    /// Run schema migrations
    fn migrate(&self) -> EngineResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| EngineError::Database(format!("Lock poisoned: {}", e)))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS trades (
                id              TEXT PRIMARY KEY,
                market_id       TEXT NOT NULL,
                market_question TEXT NOT NULL,
                side            TEXT NOT NULL,
                price           REAL NOT NULL,
                size            REAL NOT NULL,
                cost            REAL NOT NULL,
                status          TEXT NOT NULL DEFAULT 'Pending',
                reason          TEXT NOT NULL DEFAULT '',
                edge            REAL NOT NULL DEFAULT 0.0,
                confidence      REAL NOT NULL DEFAULT 0.0,
                api_cost        REAL NOT NULL DEFAULT 0.0,
                pnl             REAL,
                closed_at       TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS positions (
                id              TEXT PRIMARY KEY,
                market_id       TEXT NOT NULL,
                market_question TEXT NOT NULL,
                token_id        TEXT NOT NULL,
                side            TEXT NOT NULL,
                entry_price     REAL NOT NULL,
                current_price   REAL NOT NULL,
                size            REAL NOT NULL,
                cost_basis      REAL NOT NULL,
                unrealized_pnl  REAL NOT NULL DEFAULT 0.0,
                status          TEXT NOT NULL DEFAULT 'Open',
                opened_at       TEXT NOT NULL DEFAULT (datetime('now')),
                closed_at       TEXT
            );

            CREATE TABLE IF NOT EXISTS portfolio_snapshots (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                capital         REAL NOT NULL,
                total_pnl       REAL NOT NULL,
                daily_pnl       REAL NOT NULL,
                total_trades    INTEGER NOT NULL,
                winning_trades  INTEGER NOT NULL,
                win_rate        REAL NOT NULL,
                position_count  INTEGER NOT NULL,
                agent_state     TEXT NOT NULL,
                api_cost_daily  REAL NOT NULL DEFAULT 0.0,
                api_cost_total  REAL NOT NULL DEFAULT 0.0,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS daily_summary (
                date            TEXT PRIMARY KEY,
                opening_capital REAL NOT NULL,
                closing_capital REAL NOT NULL,
                pnl             REAL NOT NULL,
                trades_count    INTEGER NOT NULL,
                wins            INTEGER NOT NULL,
                losses          INTEGER NOT NULL,
                best_trade_pnl  REAL,
                worst_trade_pnl REAL,
                api_cost        REAL NOT NULL DEFAULT 0.0,
                notes           TEXT
            );

            CREATE TABLE IF NOT EXISTS engine_state (
                key             TEXT PRIMARY KEY,
                value           TEXT NOT NULL,
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_trades_market ON trades(market_id);
            CREATE INDEX IF NOT EXISTS idx_trades_created ON trades(created_at);
            CREATE INDEX IF NOT EXISTS idx_positions_status ON positions(status);
            CREATE INDEX IF NOT EXISTS idx_snapshots_created ON portfolio_snapshots(created_at);
            ",
        )
        .map_err(|e| EngineError::Database(format!("Migration failed: {}", e)))?;

        info!("Database migrations applied");
        Ok(())
    }

    // ═══════════════════════════════════════════
    // TRADE PERSISTENCE
    // ═══════════════════════════════════════════

    /// Insert a new trade record
    pub fn insert_trade(&self, trade: &TradeLog) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO trades (id, market_id, market_question, side, price, size, cost, status, reason, edge, confidence, api_cost, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                trade.id,
                trade.market_id,
                trade.market_question,
                format!("{:?}", trade.side),
                trade.price,
                trade.size,
                trade.cost,
                format!("{:?}", trade.status),
                trade.reason,
                trade.edge,
                trade.confidence,
                trade.api_cost,
                trade.timestamp.to_rfc3339(),
            ],
        ).map_err(|e| EngineError::Database(format!("Insert trade failed: {}", e)))?;
        Ok(())
    }

    /// Update trade with PnL on close
    pub fn close_trade(&self, trade_id: &str, pnl: f64) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE trades SET status = 'Closed', pnl = ?1, closed_at = ?2 WHERE id = ?3",
            params![pnl, Utc::now().to_rfc3339(), trade_id],
        )
        .map_err(|e| EngineError::Database(format!("Close trade failed: {}", e)))?;
        Ok(())
    }

    /// Load recent trades (for state recovery)
    pub fn load_recent_trades(&self, limit: usize) -> EngineResult<Vec<TradeLog>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, market_id, market_question, side, price, size, cost, status, reason, edge, confidence, api_cost, created_at
             FROM trades ORDER BY created_at DESC LIMIT ?1"
        ).map_err(|e| EngineError::Database(format!("Prepare failed: {}", e)))?;

        let trades = stmt
            .query_map(params![limit as i64], |row| {
                Ok(TradeLog {
                    id: row.get(0)?,
                    market_id: row.get(1)?,
                    market_question: row.get(2)?,
                    side: parse_side(&row.get::<_, String>(3)?),
                    price: row.get(4)?,
                    size: row.get(5)?,
                    cost: row.get(6)?,
                    status: parse_trade_status(&row.get::<_, String>(7)?),
                    reason: row.get(8)?,
                    edge: row.get(9)?,
                    confidence: row.get(10)?,
                    api_cost: row.get(11)?,
                    timestamp: parse_datetime(&row.get::<_, String>(12)?),
                })
            })
            .map_err(|e| EngineError::Database(format!("Query trades failed: {}", e)))?;

        let mut result = Vec::new();
        for trade in trades {
            match trade {
                Ok(t) => result.push(t),
                Err(e) => warn!("Skip corrupt trade row: {}", e),
            }
        }
        result.reverse(); // oldest first
        Ok(result)
    }

    // ═══════════════════════════════════════════
    // POSITION PERSISTENCE
    // ═══════════════════════════════════════════

    /// Insert a new position
    pub fn insert_position(&self, pos: &Position) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO positions (id, market_id, market_question, token_id, side, entry_price, current_price, size, cost_basis, unrealized_pnl, status, opened_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                pos.id,
                pos.market_id,
                pos.market_question,
                pos.token_id,
                format!("{:?}", pos.side),
                pos.entry_price,
                pos.current_price,
                pos.size,
                pos.cost_basis,
                pos.unrealized_pnl,
                format!("{:?}", pos.status),
                pos.opened_at.to_rfc3339(),
            ],
        ).map_err(|e| EngineError::Database(format!("Insert position failed: {}", e)))?;
        Ok(())
    }

    /// Close a position in DB
    pub fn close_position(&self, position_id: &str, pnl: f64) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE positions SET status = 'Closed', unrealized_pnl = ?1, closed_at = ?2 WHERE id = ?3",
            params![pnl, Utc::now().to_rfc3339(), position_id],
        ).map_err(|e| EngineError::Database(format!("Close position failed: {}", e)))?;
        Ok(())
    }

    /// Load open positions (for restart recovery)
    pub fn load_open_positions(&self) -> EngineResult<Vec<Position>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, market_id, market_question, token_id, side, entry_price, current_price, size, cost_basis, unrealized_pnl, opened_at
             FROM positions WHERE status = 'Open' ORDER BY opened_at ASC"
        ).map_err(|e| EngineError::Database(format!("Prepare failed: {}", e)))?;

        let positions = stmt
            .query_map([], |row| {
                Ok(Position {
                    id: row.get(0)?,
                    market_id: row.get(1)?,
                    market_question: row.get(2)?,
                    token_id: row.get(3)?,
                    side: parse_side(&row.get::<_, String>(4)?),
                    entry_price: row.get(5)?,
                    current_price: row.get(6)?,
                    size: row.get(7)?,
                    cost_basis: row.get(8)?,
                    unrealized_pnl: row.get(9)?,
                    opened_at: parse_datetime(&row.get::<_, String>(10)?),
                    status: PositionStatus::Open,
                })
            })
            .map_err(|e| EngineError::Database(format!("Query positions failed: {}", e)))?;

        let mut result = Vec::new();
        for pos in positions {
            match pos {
                Ok(p) => result.push(p),
                Err(e) => warn!("Skip corrupt position row: {}", e),
            }
        }
        Ok(result)
    }

    // ═══════════════════════════════════════════
    // PORTFOLIO SNAPSHOTS (hourly, for charts)
    // ═══════════════════════════════════════════

    /// Save a portfolio snapshot
    pub fn save_snapshot(&self, portfolio: &PortfolioState) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO portfolio_snapshots (capital, total_pnl, daily_pnl, total_trades, winning_trades, win_rate, position_count, agent_state, api_cost_daily, api_cost_total)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                portfolio.capital,
                portfolio.total_pnl,
                portfolio.daily_pnl,
                portfolio.total_trades,
                portfolio.winning_trades,
                portfolio.win_rate,
                portfolio.positions.len() as i32,
                format!("{:?}", portfolio.agent_state),
                portfolio.api_costs.daily_cost_usd,
                portfolio.api_costs.total_cost_usd,
            ],
        ).map_err(|e| EngineError::Database(format!("Save snapshot failed: {}", e)))?;
        Ok(())
    }

    // ═══════════════════════════════════════════
    // ENGINE STATE (key-value for recovery)
    // ═══════════════════════════════════════════

    /// Save a key-value pair
    pub fn save_state(&self, key: &str, value: &str) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO engine_state (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
            params![key, value, Utc::now().to_rfc3339()],
        )
        .map_err(|e| EngineError::Database(format!("Save state '{}' failed: {}", key, e)))?;
        Ok(())
    }

    /// Load a key-value pair
    pub fn load_state(&self, key: &str) -> EngineResult<Option<String>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT value FROM engine_state WHERE key = ?1")
            .map_err(|e| EngineError::Database(format!("Prepare failed: {}", e)))?;

        let result = stmt.query_row(params![key], |row| row.get(0)).ok();
        Ok(result)
    }

    /// Save capital value (called after every trade/close)
    pub fn save_capital(&self, capital: f64) -> EngineResult<()> {
        self.save_state("capital", &format!("{:.6}", capital))
    }

    /// Load capital on restart
    pub fn load_capital(&self) -> EngineResult<Option<f64>> {
        match self.load_state("capital")? {
            Some(s) => s
                .parse()
                .map(Some)
                .map_err(|_| EngineError::Database("Corrupt capital value in DB".into())),
            None => Ok(None),
        }
    }

    /// Save portfolio stats
    pub fn save_portfolio_stats(&self, portfolio: &PortfolioState) -> EngineResult<()> {
        let stats = serde_json::json!({
            "total_pnl": portfolio.total_pnl,
            "daily_pnl": portfolio.daily_pnl,
            "total_trades": portfolio.total_trades,
            "winning_trades": portfolio.winning_trades,
            "win_rate": portfolio.win_rate,
        });
        self.save_state("portfolio_stats", &stats.to_string())
    }

    /// Load portfolio stats on restart
    pub fn load_portfolio_stats(&self) -> EngineResult<Option<serde_json::Value>> {
        match self.load_state("portfolio_stats")? {
            Some(s) => serde_json::from_str(&s)
                .map(Some)
                .map_err(|e| EngineError::Database(format!("Corrupt portfolio_stats: {}", e))),
            None => Ok(None),
        }
    }

    // ═══════════════════════════════════════════
    // DAILY SUMMARY (for weekly-learning skill)
    // ═══════════════════════════════════════════

    /// Upsert daily summary
    pub fn save_daily_summary(
        &self,
        date: &str,
        opening: f64,
        closing: f64,
        trades: i32,
        wins: i32,
        losses: i32,
        api_cost: f64,
    ) -> EngineResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO daily_summary (date, opening_capital, closing_capital, pnl, trades_count, wins, losses, api_cost)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(date) DO UPDATE SET
                closing_capital = ?3, pnl = ?4, trades_count = ?5, wins = ?6, losses = ?7, api_cost = ?8",
            params![date, opening, closing, closing - opening, trades, wins, losses, api_cost],
        ).map_err(|e| EngineError::Database(format!("Save daily summary failed: {}", e)))?;
        Ok(())
    }

    // ═══════════════════════════════════════════
    // STATS QUERIES (for dashboard / skills)
    // ═══════════════════════════════════════════

    /// Total lifetime stats
    pub fn get_lifetime_stats(&self) -> EngineResult<serde_json::Value> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), SUM(pnl), MIN(pnl), MAX(pnl)
             FROM trades WHERE pnl IS NOT NULL"
        ).map_err(|e| EngineError::Database(format!("Stats query failed: {}", e)))?;

        let stats = stmt
            .query_row([], |row| {
                Ok(serde_json::json!({
                    "total_closed": row.get::<_, i64>(0)?,
                    "total_wins": row.get::<_, i64>(1).unwrap_or(0),
                    "total_pnl": row.get::<_, f64>(2).unwrap_or(0.0),
                    "worst_trade": row.get::<_, f64>(3).unwrap_or(0.0),
                    "best_trade": row.get::<_, f64>(4).unwrap_or(0.0),
                }))
            })
            .map_err(|e| EngineError::Database(format!("Stats query failed: {}", e)))?;

        Ok(stats)
    }

    // ═══════════════════════════════════════════
    // INTERNAL HELPERS
    // ═══════════════════════════════════════════

    fn lock(&self) -> EngineResult<std::sync::MutexGuard<Connection>> {
        self.conn
            .lock()
            .map_err(|e| EngineError::Database(format!("DB lock poisoned: {}", e)))
    }
}

// Parsing helpers for reading enum values back from text columns
fn parse_side(s: &str) -> Side {
    match s {
        "Sell" => Side::Sell,
        _ => Side::Buy,
    }
}

fn parse_trade_status(s: &str) -> TradeStatus {
    match s {
        "Matched" => TradeStatus::Matched,
        "Mined" => TradeStatus::Mined,
        "Confirmed" => TradeStatus::Confirmed,
        "Failed" => TradeStatus::Failed,
        "Retrying" => TradeStatus::Retrying,
        "Closed" => TradeStatus::Confirmed, // closed = was confirmed
        _ => TradeStatus::Pending,
    }
}

fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
