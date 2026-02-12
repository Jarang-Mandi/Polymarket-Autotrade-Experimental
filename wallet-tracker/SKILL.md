---
name: wallet_tracker
description: Sistem monitoring real-time untuk wallet balance, open positions, PnL tracking, fee tracking, dan portfolio health dashboard.
metadata: {"openclaw":{"always":true}}
---

# Wallet & Portfolio Tracker

## Purpose
Agent WAJIB selalu tahu persis berapa modal yang tersedia, berapa yang di-expose, dan berapa PnL-nya.
Tanpa tracking yang akurat, semua risk calculation akan salah.

---

## Core Tracking Data

### 1. Wallet Balance
```json
{
  "timestamp": "ISO-8601",
  "usdc_balance": 0.00,
  "usdc_available": 0.00,
  "usdc_in_positions": 0.00,
  "usdc_in_pending_orders": 0.00,
  "total_portfolio_value": 0.00,
  "polygon_matic_balance": 0.00,
  "gas_budget_remaining": 0.00
}
```

### 2. Open Positions
```json
{
  "positions": [
    {
      "position_id": "",
      "market_id": "",
      "market_question": "",
      "category": "",
      "side": "YES | NO",
      "shares": 0.00,
      "entry_price": 0.00,
      "current_price": 0.00,
      "cost_basis": 0.00,
      "current_value": 0.00,
      "unrealized_pnl": 0.00,
      "unrealized_pnl_pct": 0.00,
      "time_held": "duration",
      "time_to_resolution": "duration",
      "entry_timestamp": "ISO-8601",
      "fees_paid": 0.00
    }
  ],
  "total_positions": 0,
  "total_cost_basis": 0.00,
  "total_current_value": 0.00,
  "total_unrealized_pnl": 0.00
}
```

### 3. PnL Summary
```json
{
  "initial_capital": 50.00,
  "current_capital": 0.00,
  "total_realized_pnl": 0.00,
  "total_unrealized_pnl": 0.00,
  "total_pnl": 0.00,
  "total_pnl_pct": 0.00,
  "total_fees_paid": 0.00,
  "total_gas_spent": 0.00,
  "net_pnl_after_costs": 0.00,
  "peak_capital": 0.00,
  "trough_capital": 0.00,
  "max_drawdown": 0.00,
  "max_drawdown_pct": 0.00,
  "daily_pnl": 0.00,
  "weekly_pnl": 0.00,
  "monthly_pnl": 0.00,
  "win_count": 0,
  "loss_count": 0,
  "win_rate": 0.00,
  "average_win": 0.00,
  "average_loss": 0.00,
  "largest_win": 0.00,
  "largest_loss": 0.00,
  "profit_factor": 0.00,
  "expectancy": 0.00
}
```

### 4. Fee & Cost Tracking
```json
{
  "total_maker_fees": 0.00,
  "total_taker_fees": 0.00,
  "total_gas_fees": 0.00,
  "total_costs": 0.00,
  "costs_as_pct_of_capital": 0.00,
  "avg_cost_per_trade": 0.00,
  "fee_efficiency": "GOOD | ACCEPTABLE | HIGH | EXCESSIVE"
}
```

---

## Check Schedule

| Check | Frequency | Purpose |
|-------|-----------|---------|
| Balance snapshot | Every agent cycle | Know available capital |
| Position mark-to-market | Every 30 minutes | Track unrealized PnL |
| PnL calculation | After each trade resolved | Accurate performance tracking |
| Fee audit | Daily | Monitor cost efficiency |
| Full portfolio review | Every 4 hours | Comprehensive health check |

---

## Portfolio Health Alerts

```
IF available_capital < $5.00:
    ALERT: "CRITICAL — Near zero capital"
    ACTION: FREEZE all trading

IF unrealized_loss > 20% of capital:
    ALERT: "Large unrealized loss"
    ACTION: Evaluate positions for exit

IF daily_realized_loss > 8% of capital:
    ALERT: "Daily loss limit reached"
    ACTION: STOP trading today

IF total_fees > 5% of capital:
    ALERT: "Fee drag significant"
    ACTION: Switch to limit orders only

IF no trades winning for 7 days:
    ALERT: "Extended losing streak"
    ACTION: Trigger crisis reflection

IF portfolio_value reaches new peak:
    LOG: "New high-water mark"
    UPDATE: peak_capital for drawdown calculation
```

---

## Agent State Determination

Berdasarkan wallet data, tentukan agent state:
```
capital_ratio = current_capital / initial_capital

IF capital_ratio > 0.80: state = "GROWTH"
IF capital_ratio 0.60-0.80: state = "CAUTIOUS"
IF capital_ratio 0.40-0.60: state = "DEFENSIVE"
IF capital_ratio 0.20-0.40: state = "SURVIVAL"
IF capital_ratio < 0.20: state = "CRITICAL"
```

---

## Trade History Log

Setiap resolved trade harus di-record:
```json
{
  "trade_id": "",
  "market_id": "",
  "market_question": "",
  "category": "",
  "side": "",
  "entry_price": 0.00,
  "exit_price": 0.00,
  "shares": 0.00,
  "cost": 0.00,
  "revenue": 0.00,
  "realized_pnl": 0.00,
  "fees": 0.00,
  "net_pnl": 0.00,
  "hold_duration": "",
  "was_profitable": true,
  "entry_reasoning": "",
  "outcome_notes": ""
}
```

---

## Output Format (Dashboard)

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
WALLET: $XX.XX avail | $XX.XX in_pos | gas=$X.XX
STATE: [STATE] | ratio=XX% | DD=X.X%
POS: X open | value=$XX.XX | uPnL=+/-$X.XX
PERF: PnL=+/-$X.XX (X.X%) | WR=XX% | PF=X.Xx | Sharpe=X.X
COSTS: fees=$X.XX (X.X% of volume) | API=$X.XX
ALERTS: [list or NONE]
NEXT_RESOLVE: [market] in Xh
```

**Full JSON hanya digunakan untuk weekly deep review atau jika data perlu di-export.**
