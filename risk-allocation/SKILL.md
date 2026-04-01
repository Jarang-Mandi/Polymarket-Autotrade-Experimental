---
name: risk-allocation
description: Risk manager utama — mengatur exposure, position sizing, stop-loss, survival logic, dan daily risk budget untuk $50 micro-bankroll.
metadata: {"openclaw":{"always":true}}
---

# Risk & Allocation — The Gatekeeper

## Purpose
Ini adalah GATEKEEPER. TIDAK ADA trade yang boleh lewat tanpa approval dari skill ini.
Setiap trade HARUS melewati semua risk checks sebelum eksekusi.

---

## $50 Micro-Bankroll Position Sizing

### Kelly Criterion (Modified for Micro-Bankroll)

```
f_kelly = (bp - q) / b

where:
  b = odds (payout ratio = (1 - price) / price)
  p = model probability
  q = 1 - p

f_safe = 0.25 * f_kelly   # Quarter Kelly (BUKAN half Kelly — lebih konservatif untuk $50)
f_final = min(f_safe, max_allowed_by_state)
```

### Position Size by Agent State

| State | Max Per Trade | Max Total Exposure | Max Concurrent |
|-------|--------------|-------------------|----------------|
| GROWTH | 6% ($3.00) | 25% ($12.50) | 4 |
| CAUTIOUS | 4% ($2.00) | 20% ($10.00) | 3 |
| DEFENSIVE | 3% ($1.50) | 15% ($7.50) | 2 |
| SURVIVAL | 2% ($1.00) | 10% ($5.00) | 1 |
| CRITICAL | 0% | 0% | 0 |

**Minimum trade size: $1.00** (di bawah ini, fees membuat trade tidak worth it)

---

## Risk Budget System

### Daily Risk Budget
```
daily_risk_budget = capital * 0.08  # Max 8% capital boleh hilang per hari
remaining_budget = daily_risk_budget - losses_today

IF remaining_budget <= 0:
    STOP trading for today
```

### Weekly Risk Budget
```
weekly_risk_budget = capital * 0.20  # Max 20% per minggu
```

---

## Trade Approval Process

Setiap candidate trade harus melewati checklist ini:

### Gate 1: Capital Check
```
IF available_capital < $3.00: REJECT ("Insufficient capital")
IF trade_size > max_per_trade: REDUCE to max
IF trade_size < $1.00: REJECT ("Below minimum size")
```

### Gate 2: Exposure Check
```
current_exposure = sum(all_open_positions)
IF current_exposure + trade_size > max_total_exposure: REJECT
IF open_positions_count >= max_concurrent: REJECT
```

### Gate 3: Risk/Reward Check
```
potential_loss = trade_size * price_paid  # Max you can lose
potential_gain = trade_size * (1 - price_paid)  # Max you can gain
risk_reward = potential_gain / potential_loss

IF risk_reward < 1.5: REJECT ("Risk/reward inadequate")
```

### Gate 4: Correlation Check
```
FOR each open_position:
    IF new_trade is correlated with open_position:
        combined_exposure += open_position.size
IF combined_correlated_exposure > 15% of capital: REJECT
```

### Gate 5: Drawdown Check
```
IF drawdown_today > 5%: INCREASE confidence threshold
IF drawdown_today > 8%: STOP trading today
IF drawdown_week > 15%: ENTER defensive mode
IF drawdown_total > 40%: ENTER survival mode
```

### Gate 6: Cooldown Check
```
IF last_loss < 60 minutes ago AND consecutive_losses >= 2: REJECT ("Cooldown active")
IF last_trade < 15 minutes ago: WARN ("Consider spacing trades")
```

### Gate 7: Survival Probability
```
Use quant-risk-engine Monte Carlo
IF survival_probability < 0.85: REJECT
IF risk_of_ruin > 0.05: REJECT
```

---

## Stop-Loss / Exit Rules

Polymarket adalah binary market, jadi stop-loss bekerja berbeda:

### Time-Based Exit
```
IF event resolution < 2 hours AND position losing > 30%:
    EVALUATE: apakah informasi berubah?
    IF tidak: HOLD (binary pays 0 or 1)
    IF ya: EXIT

IF market becomes illiquid: EXIT if possible
```

### Information-Based Exit
```
IF news impact shifts probability > 10% against position:
    RE-EVALUATE immediately
    IF new EV < 0: EXIT
```

### Portfolio-Level Stop
```
IF total portfolio drawdown > 10% in 24 hours:
    CLOSE weakest position
    REDUCE all position sizes by 30%
```

---

## Anti-Correlation Matrix

Track correlation between open positions:
- Same event different outcomes → HIGHLY correlated
- Same category (e.g., all sports) → MODERATE correlation
- Same timeframe → LOW correlation
- Different category, different time → UNCORRELATED

```
Max correlated exposure: 15% of capital
Max single-category exposure: 20% of capital
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
RISK: [APPROVED|REDUCED|REJECTED] [market_id] [YES|NO]
SIZE: req=$X.XX → approved=$X.XX | RR=X.Xx
EXPOSURE: current=X.X% new=X.X% | budget_left=$X.XX
GATES: [passed]/[total] | CORR: [OK|FLAG] | SURV: XX%
PLAN: [LIMIT|MARKET]@$X.XX exit:TP=$X.XX SL=$X.XX
```

Jika REJECTED:
```
RISK: REJECTED [market_id] — [reason]
GATES_FAILED: [list of failed gates]
```
