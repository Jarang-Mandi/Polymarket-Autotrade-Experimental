# Polymarket Execution Engine

High-performance Rust execution engine for Polymarket trading. Receives commands from OpenClaw AI agent (the "brain"), executes trades, monitors positions, and serves a real-time dashboard.

## Architecture

```
┌─────────────┐    Commands     ┌──────────────────────────────────┐
│   OpenClaw   │───────────────→│     Execution Engine (Rust)       │
│  (Claude AI) │  POST /trade   │                                   │
│   = Brain    │  POST /close   │  ┌──────────┐    ┌────────────┐  │
│              │←───────────────│  │ Polymarket│    │ SQLite DB  │  │
│  Makes all   │  JSON response │  │ CLOB API  │    │ Persistence│  │
│  decisions   │                │  └──────────┘    └────────────┘  │
└─────────────┘                │         ↕                ↕        │
                               │  ┌──────────┐    ┌────────────┐  │
┌─────────────┐    WS + REST   │  │ Position  │    │ Risk       │  │
│    React     │←───────────────│  │ Monitor   │    │ Validation │  │
│  Dashboard   │  GET /state    │  └──────────┘    └────────────┘  │
│   = Eyes     │  WS /ws (2s)  │         ↕                         │
└─────────────┘                │  ┌──────────────────┐             │
                               │  │ REST + WS Server  │             │
                               │  │ (Axum 0.7)        │             │
                               │  └──────────────────┘             │
                               └──────────────────────────────────┘
```

**OpenClaw = Brain** — Makes all trading decisions via Claude AI  
**Rust Engine = Hands** — Executes trades, scans markets, monitors positions  
**React Dashboard = Eyes** — Real-time monitoring UI  

## Modules

| Module | Description |
|--------|-------------|
| `main.rs` | Entry point, config validation, service startup |
| `config.rs` | Environment-based configuration (`.env`) |
| `types.rs` | All data types: AgentState, Market, Position, TradeLog, ApiCostTracker |
| `polymarket.rs` | Polymarket Gamma + CLOB API client with retry logic |
| `engine.rs` | Command handlers + background loops (scan, position monitor) |
| `server.rs` | Axum REST API + WebSocket server |
| `db.rs` | SQLite persistence layer (rusqlite, WAL mode) |
| `error.rs` | 25+ typed error variants, severity classification, Axum error responses |

## Setup

```bash
cp .env.example .env
# Edit .env with your Polymarket private key + config
cargo build --release
cargo run --release
```

## API Endpoints

### Read Endpoints (Dashboard + OpenClaw)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/state` | GET | Full portfolio state + agent status |
| `/api/positions` | GET | Open positions |
| `/api/trades` | GET | Recent trade history |
| `/api/markets` | GET | Cached market opportunities |
| `/api/costs` | GET | Claude API cost tracking |
| `/api/health` | GET | Engine health check |
| `/ws` | WebSocket | Real-time state updates (2s interval) |

### Command Endpoints (OpenClaw → Engine)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/trade` | POST | Execute a trade command |
| `/api/close` | POST | Close a position |
| `/api/ai-exit-review` | POST | AI reasoning for hold/close + optional close execution |
| `/api/risk-thresholds/propose` | POST | Propose SL/TP/auto-close change (pending user confirm) |
| `/api/risk-thresholds/confirm` | POST | Approve/reject pending SL/TP proposal |
| `/api/report-cost` | POST | Report Claude API usage |

### Exit Policy Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/risk-thresholds` | GET | Current SL/TP policy + pending proposal |

### Trade Command Example

```json
POST /api/trade
{
  "market_id": "0x...",
  "side": "Buy",
  "size": 3.0,
  "price": 0.45,
  "edge": 0.12,
  "confidence": 0.75,
  "reason": "Strong polling data favoring YES outcome"
}
```

### Error Response Format

```json
{
  "success": false,
  "error": "Cost $4.50 exceeds 95% of capital $4.00",
  "code": "INSUFF_CAPITAL",
  "severity": "Warning",
  "retryable": false
}
```

## Background Behaviors

| Loop | Interval | Action |
|------|----------|--------|
| Market Scan | 5min | Fetch markets from Gamma API, cache for OpenClaw |
| Position Monitor | 60s | Update position prices; auto-close only if `POSITION_AUTO_CLOSE_ENABLED=true` |
| Heartbeat | 30s | Update timestamps, recalculate agent state |
| DB Snapshot | 1hr | Save portfolio snapshot for historical charts |

## Risk Limits

| Parameter | Value |
|-----------|-------|
| Max position size | $5.00 |
| Max portfolio risk | 6% per trade |
| Max concurrent positions | 5 |
| Stop-loss | Runtime policy (`DEFAULT_STOP_LOSS_PCT`) |
| Take-profit | Runtime policy (`DEFAULT_TAKE_PROFIT_PCT`) |
| Auto-close mode | `POSITION_AUTO_CLOSE_ENABLED` (default: `false`) |
| Min edge (Neutral state) | 8% |
| Min confidence | 50% |
| Daily API budget | $0.50 |

## SL/TP Confirmation Flow

1. AI proposes threshold change via `POST /api/risk-thresholds/propose`.
2. Engine stores proposal as pending and does **not** apply it yet.
3. User confirms/rejects via `POST /api/risk-thresholds/confirm`.
4. Only after approval, new thresholds become active.

This keeps SL/TP adjustments user-controlled while still allowing AI-generated reasoning.

## Agent States

| State | Capital | Max Position % | Min Edge |
|-------|---------|----------------|----------|
| Survival | <$15 | 2% | 15% |
| Defensive | $15-30 | 4% | 12% |
| Neutral | $30-60 | 6% | 8% |
| Aggressive | $60-120 | 8% | 6% |
| Apex | >$120 | 10% | 5% |

## SQLite Persistence

Engine state survives restarts via SQLite (`data/engine.db`):

- **trades** — Full trade history with PnL
- **positions** — Open/closed positions
- **portfolio_snapshots** — Hourly capital/PnL snapshots (for charts)
- **daily_summary** — Per-day stats (for weekly-learning skill)
- **engine_state** — Key-value store (capital, stats recovery)

Write-through pattern: memory is primary (fast), SQLite is durable backup. DB failures = warning log, engine continues.

## Deployment

See [DEPLOY_LINUX.md](DEPLOY_LINUX.md) for full Linux server setup:
- systemd service (auto-restart)
- Nginx reverse proxy + HTTPS
- SQLite backup cron
- Monitoring commands

## Tech Stack

| Component | Version |
|-----------|---------|
| Rust | 2021 edition |
| Tokio | 1.x (async runtime) |
| Axum | 0.7 (HTTP + WebSocket) |
| reqwest | 0.12 (HTTP client) |
| rusqlite | 0.31 (SQLite, bundled) |
| ethers | 2.x (Ethereum signing) |
| thiserror | 1.x (error types) |
| tracing | 0.1 (structured logging) |
