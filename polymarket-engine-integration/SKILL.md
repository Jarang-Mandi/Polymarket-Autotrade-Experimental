---
name: polymarket-engine-integration
description: Orchestrates Rust execution engine + React dashboard. OpenClaw is the brain, engine is the hands.
metadata:
  openclaw:
    always: true
  author: OpenClaw Autonomous Agent
  category: infrastructure
  priority: critical
  triggers:
    - "engine"
    - "dashboard"
    - "trading system"
    - "start trading"
    - "monitor"
    - "execution"
    - "trade"
    - "position"
---

# Polymarket Engine Integration

You are an autonomous trading agent. YOU are the brain (Claude via OpenClaw).
The Rust engine is your hands — it executes orders, fetches data, monitors positions.

## Architecture (Clean Separation)

```
[OpenClaw Agent = YOU = the BRAIN]
│   Uses: Claude API (managed by OpenClaw, NOT the engine)
│   Uses: All 29 skills for analysis
│
├── GET  /api/markets   → Read market data from engine
├── GET  /api/state     → Read portfolio, capital, positions
├── POST /api/trade     → Command engine to execute trade
├── POST /api/close     → Command engine to close position
├── POST /api/report-cost → Report your Claude API usage
│
└── [Rust Engine = the HANDS]
    ├── Scans markets (Gamma API) automatically
    ├── Executes orders (CLOB API + EIP712)
    ├── Monitors positions (stop-loss, take-profit)
    ├── Serves REST API + WebSocket to dashboard
    └── Does NOT call Claude — that's YOUR job
```

**KEY: No Claude API key in engine. Zero duplicate API costs.**

## How to Trade (Your Workflow)

### Step 1: Get Market Data
```
GET http://localhost:3001/api/markets
→ Returns top 50 markets with: id, question, yes_price, volume, liquidity, spread
```

### Step 2: Analyze with Your Skills
Use your 29 skills (market-regime, bayesian-update, quant-risk-engine, etc.)
to analyze the markets returned. YOU decide. The engine doesn't think.

### Step 3: Send Trade Command
```
POST http://localhost:3001/api/trade
{
  "market_id": "0x...",
  "side": "Buy",
  "size": 2.50,
  "price": 0.450,
  "edge": 0.12,
  "confidence": 0.85,
  "reason": "Bayesian edge 12%, regime favorable"
}
→ Returns: { success: true, trade_id: "...", position_id: "..." }
```

### Step 4: Monitor
```
GET http://localhost:3001/api/state
→ capital, positions, pnl, agent_state, hunger_level
GET http://localhost:3001/api/positions
→ All open positions with live P&L
```

### Step 5: Close When Ready
```
POST http://localhost:3001/api/close
{ "position_id": "...", "reason": "Target reached" }
```

### Step 6: Report Your API Cost
After each Claude call, report usage so dashboard tracks it:
```
POST http://localhost:3001/api/report-cost
{
  "input_tokens": 1500,
  "output_tokens": 80,
  "cache_read_tokens": 4000,
  "cache_write_tokens": 0
}
```

## Engine REST API

| Endpoint | Method | Purpose |
|---|---|---|
| /api/state | GET | Full engine state |
| /api/positions | GET | Open positions |
| /api/trades | GET | Trade history |
| /api/markets | GET | Cached market data |
| /api/costs | GET | API cost tracking |
| /api/health | GET | Health check |
| /api/trade | POST | **Execute trade** |
| /api/close | POST | **Close position** |
| /api/report-cost | POST | **Report Claude usage** |
| /ws | WS | Real-time state (2s) |

## Engine Auto-Behaviors (No Command Needed)

The engine handles these autonomously:
- **Market scanning** every 5 min (Gamma API)
- **Position price updates** every 60s
- **Stop-loss** at -15% P&L (auto-close)
- **Take-profit** at +30% P&L (auto-close)
- **State machine** updates (Survival/Defensive/Neutral/Aggressive/Apex)
- **Hunger level** tracking
- **Daily reset** of API cost counters

## Risk Limits (Engine-Enforced)

Even if you send a risky command, the engine will reject it:
- Max position size: capped by `agent_state.max_position_pct()` AND $5 hard cap
- Cost cannot exceed 95% of current capital
- Market must exist in cache (scan must have run first)

## Starting Everything

```bash
# Terminal 1: Engine (the hands)
cd polymarket-engine
cp .env.example .env
# Fill: POLYMARKET_PRIVATE_KEY, POLYMARKET_FUNDER_ADDRESS
cargo build --release
./target/release/polymarket-engine

# Terminal 2: Dashboard (the eyes)
cd polymarket-dashboard
npm install && npm run dev
# → http://localhost:5173
```

## Emergency Protocols

| Condition | Action |
|---|---|
| Capital < $10 | Engine enters Survival state, max 2% position |
| Stop-loss hit | Engine auto-closes, no command needed |
| Take-profit hit | Engine auto-closes, reports P&L |
| Engine unreachable | Dashboard shows OFFLINE, retry in 3s |
| Daily API > $0.45 | You should stop calling Claude, use cached analysis |
