---
name: reinforcement_survival_engine
description: Self-improving engine yang menggunakan reward-based evaluation untuk mengoptimalkan survival probability dan risk-adjusted returns dari setiap keputusan.
metadata: {"openclaw":{"always":true}}
---

# Reinforcement Survival Engine

## Purpose
Agent belajar dari setiap keputusan melalui reward system yang menyeimbangkan SURVIVAL dan GROWTH.
Profit adalah oksigen — tanpa profit, agent mati perlahan. Tapi reckless trading membunuh lebih cepat.

**FILOSOFI: Reward untuk BERTINDAK CERDAS, penalti untuk DIAM TANPA ALASAN dan TRADE TANPA EDGE.**

---

## Reward Function (Multi-Objective)

```
Total_Reward = 
    (w1 * Survival_Score)
  + (w2 * Profit_Score)
  - (w3 * Drawdown_Penalty)
  - (w4 * Volatility_Penalty)
  + (w5 * Consistency_Bonus)
  + (w6 * Discipline_Bonus)
  - (w7 * Reckless_Penalty)
  + (w8 * Hunting_Bonus)
  - (w9 * Stagnation_Penalty)
  + (w10 * Cost_Efficiency_Bonus)
```

### Weight Configuration ($50 Survival + Growth Mode)
| Component | Weight | Description |
|-----------|--------|-------------|
| Survival_Score | 0.18 | Masih hidup = reward positif |
| Profit_Score | 0.20 | ROI per trade — PROFIT IS OXYGEN |
| Drawdown_Penalty | 0.14 | Penalti untuk drawdown besar |
| Volatility_Penalty | 0.07 | Penalti untuk equity curve volatile |
| Consistency_Bonus | 0.09 | Bonus untuk profit konsisten kecil |
| Discipline_Bonus | 0.07 | Bonus untuk skip trade yang seharusnya di-skip |
| Reckless_Penalty | 0.07 | Penalti untuk trade tanpa edge |
| Hunting_Bonus | 0.06 | Bonus untuk AKTIF mencari dan mengeksekusi trades yang profitable |
| Stagnation_Penalty | 0.05 | Penalti untuk terlalu lama DIAM tanpa alasan |
| **Cost_Efficiency_Bonus** | **0.07** | **Bonus untuk profit/API_cost ratio tinggi — berpikir efisien = reward** |

> **PRINSIP**: Setiap komponen reward HARUS memperhitungkan API cost.
> Profit $2 dengan API cost $0.50 (4x efficiency) > Profit $3 dengan API cost $2 (1.5x efficiency).
> Agent harus HIDUP **dan** BERTUMBUH, bukan hanya bertahan.

---

## Scoring Detail

### Survival Score
```
IF capital >= initial_capital: score = 1.0
IF capital >= 80% initial: score = 0.8
IF capital >= 60% initial: score = 0.5
IF capital >= 40% initial: score = 0.2
IF capital >= 20% initial: score = -0.5
IF capital < 20% initial: score = -2.0 (catastrophic)
```

### Profit Score
```
trade_roi = (output - input) / input
IF trade_roi > 0: score = min(trade_roi * 10, 1.0)
IF trade_roi < 0: score = max(trade_roi * 15, -1.5) # Losses punished harder
```

### Drawdown Penalty
```
max_drawdown = (peak_capital - current_capital) / peak_capital
penalty = max_drawdown^2 * 10  # Squared — exponential punishment
```

### Consistency Bonus
```
IF 5+ consecutive profitable trades (any size): bonus = 0.5
IF win_rate > 60% over last 20 trades: bonus += 0.3
IF average_profit > 0 for 7 consecutive days: bonus += 0.5
```

### Discipline Bonus
```
FOR each SKIPPED trade that would have been a loss:
    bonus += 0.3
FOR each properly sized trade (within limits):
    bonus += 0.1
FOR each stop-loss respected:
    bonus += 0.2
```

### Reckless Penalty
```
FOR each trade with EV < 0 at execution:
    penalty += 0.5
FOR each trade exceeding position limit:
    penalty += 1.0
FOR each revenge trade (within 30min of loss):
    penalty += 0.8
```

### Hunting Bonus (NEW — Reward Proactive Profit-Seeking)
```
FOR each trade executed WITH edge > 3% that was profitable:
    bonus += 0.4
FOR each new market category explored successfully:
    bonus += 0.3
FOR scanning and finding opportunity within 2 hours of market opening:
    bonus += 0.2
FOR executing during HUNGRY state and profiting:
    bonus += 0.5 (reward hunting under pressure)
```

### Stagnation Penalty (NEW — Punish Unjustified Inaction)
```
IF 0 trades in 24 hours AND markets were available:
    penalty += 0.3
IF 0 trades in 48 hours:
    penalty += 0.8 (SEVERE — agent is dying from inaction)
IF cash_reserve > 70% for 24+ hours AND opportunities exist:
    penalty += 0.4 (money sitting idle = wasted oxygen)
IF 5+ consecutive SKIPs on markets with edge > 5%:
    penalty += 0.6 (FEAR penalty — agent terlalu takut)
```

**PENTING: Stagnation penalty HANYA berlaku jika ada peluang yang terlewat.
Jika market benar-benar tidak ada edge, diam = disiplin, bukan stagnation.**

### Cost Efficiency Bonus (NEW — Reward Smart Thinking)
```
cost_efficiency_ratio = daily_trading_profit / daily_api_cost

IF cost_efficiency_ratio > 10.0: bonus = 1.0 (EXCELLENT — thinking very efficiently)
IF cost_efficiency_ratio > 5.0: bonus = 0.6 (GOOD)
IF cost_efficiency_ratio > 2.0: bonus = 0.3 (ACCEPTABLE)
IF cost_efficiency_ratio > 1.0: bonus = 0.0 (BREAK-EVEN — needs improvement)
IF cost_efficiency_ratio < 1.0: bonus = -0.5 (BAD — spending more thinking than earning)
IF cost_efficiency_ratio < 0.5: bonus = -1.0 (CRITICAL — brain cost exceeds profit)

ADDITIONAL:
FOR using compact output format consistently: bonus += 0.2
FOR batching operations (scan all markets in 1 call): bonus += 0.2
FOR making profitable trade with Quick Decision (1 API call): bonus += 0.3
FOR using Deep Decision on a $1 trade: penalty += 0.3 (overkill)
```

---

## Parameter Adaptation

Berdasarkan cumulative reward trajectory, adjust parameters:

### If Reward Trending Positive (Last 20 trades)
```
confidence_threshold -= 0.02 (slightly more aggressive)
position_size_multiplier += 0.05
ev_threshold -= 0.01
```

### If Reward Trending Negative (Last 20 trades)
```
confidence_threshold += 0.05 (much more conservative)
position_size_multiplier -= 0.10
ev_threshold += 0.03
cooldown_period += 30 minutes
```

### If Reward Flat/Stagnant (DANGER — SLOW DEATH)
```
ALERT: "Stagnation detected — agent may be dying slowly"
Expand market categories to discover new edges
Lower edge threshold by 0.01 (min 0.03)
Increase scan frequency by 2x
Review skip history — am I skipping too many good opportunities?
IF stagnant for 5+ days:
    CRITICAL: Force deep strategy review
    Consider: category rotation, timing shift, new data sources
    Run market-scanner at maximum breadth
```

---

## Adaptation Boundaries (Hard Limits)

Parameters TIDAK BOLEH di-adjust melewati boundaries ini:

```
confidence_threshold: min=0.60, max=0.95
position_size_multiplier: min=0.3, max=1.5
ev_threshold: min=0.03, max=0.30
max_concurrent_positions: min=1, max=5
daily_trade_limit: min=1, max=10
```

---

## Learning Triggers

Evaluasi reward dilakukan pada:
1. **Per Trade**: Segera setelah trade resolved
2. **Daily**: End of day summary
3. **Weekly**: Comprehensive weekly review (weekly-learning)
4. **After Drawdown**: Setelah drawdown > 10%
5. **After Win Streak**: Setelah 5+ consecutive wins (check overconfidence)

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**

**Per-trade eval:**
```
REWARD: [market_id] score=X.XX | surv=X.X prof=X.X DD=-X.X disc=X.X hunt=X.X stag=-X.X
CUM: X.XX trend=[pos|neg|flat]
```

**Daily/Weekly eval:**
```
RL_EVAL: [daily|weekly] reward=X.XX trend=[pos|neg|flat]
COMPONENTS: S=X.X P=X.X DD=-X.X V=-X.X C=X.X D=X.X R=-X.X H=X.X ST=-X.X
ADJUST: conf X.XX→X.XX | size X.Xx→X.Xx | ev X.XX→X.XX
INSIGHTS: [1-2 key insights]
ACTIONS: [1-2 recommended actions]
```
