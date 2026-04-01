---
name: weekly-learning
description: Engine pembelajaran mingguan yang mengkompilasi semua data, menganalisa performa mendalam, meretrain parameter strategi, dan menghasilkan evolution plan.
metadata: {"openclaw":{"always":true}}
---

# Weekly Learning & Evolution Engine

## Purpose
Setiap minggu, agent HARUS duduk dan melakukan deep review. Ini bukan opsional — ini WAJIB.
Weekly learning adalah satu-satunya cara agent bisa evolve dan bertahan jangka panjang.

---

## Weekly Learning Schedule

**Trigger**: Setiap Minggu pukul 00:00 UTC
**Duration**: Comprehensive analysis
**Input**: Semua logs, trades, dan data dari minggu ini

---

## Phase 1: Data Compilation

Kumpulkan dari activity-log:
```
- Total trades executed
- Total trades skipped
- Total trades resolved
- Win/loss breakdown
- PnL summary (gross, net after fees)
- Daily PnL breakdown
- Category breakdown
- State changes during week
- Regime conditions during week
- Errors encountered
- Mistakes detected
```

---

## Phase 2: Performance Deep Analysis

### 2.1 Overall Performance
```json
{
  "week_number": 0,
  "start_capital": 0.00,
  "end_capital": 0.00,
  "weekly_return": 0.00,
  "weekly_return_pct": 0.00,
  "total_trades": 0,
  "win_rate": 0.00,
  "profit_factor": 0.00,
  "sharpe_ratio_weekly": 0.00,
  "max_drawdown_weekly": 0.00,
  "avg_trade_pnl": 0.00,
  "cost_ratio": 0.00
}
```

### 2.2 Category Performance
```
FOR each category traded this week:
{
  "category": "",
  "trades": 0,
  "wins": 0,
  "losses": 0,
  "win_rate": 0.00,
  "pnl": 0.00,
  "avg_edge": 0.00,
  "calibration_error": 0.00,
  "recommendation": "INCREASE | MAINTAIN | DECREASE | STOP"
}
```

### 2.3 Edge Analysis
```
WHERE was edge found this week?
- Categories with positive expectancy
- Time-of-day with best performance
- Market types with best hit rate
- News-driven vs statistical trades — which is better?
- Signal grade distribution vs outcome
```

### 2.4 Calibration Analysis
```
FOR each confidence bucket:
    predicted_win_rate = mean(confidence_scores)
    actual_win_rate = wins / total_in_bucket
    calibration_error = |predicted - actual|

Buckets: [0.60-0.65], [0.65-0.70], [0.70-0.75], [0.75-0.80], [0.80-0.85], [0.85+]
```

### 2.5 Skip Analysis
```
Review: trader yang di-SKIP, apa outcome-nya?
- Correct skips: would have lost → GOOD discipline
- Wrong skips: would have won → possible under-confidence
- Calculate: skip_accuracy = correct_skips / total_skips

IF skip_accuracy < 50%: Agent terlalu konservatif → loosen criteria
IF skip_accuracy > 80%: Agent disciplined → maintain criteria
```

---

## Phase 3: Behavioral Audit

### Rules Compliance
```
- Trades within position size limits: X/Y
- Trades within confidence threshold: X/Y
- Proper cooldown observed: X/Y
- Daily loss limits respected: X/Y
- Correlation rules respected: X/Y
- Discipline score: X%
```

### Behavioral Patterns
```
- Revenge trading instances: X
- Oversize trades: X
- FOMO trades: X
- Trades below EV threshold: X
- Proper logging compliance: X%
```

---

## Phase 4: Parameter Optimization

Based on week's data, recalibrate:

### Confidence Threshold
```
IF win_rate > 65% AND positive edge:
    confidence_threshold -= 0.02 (allow slightly more trades)
IF win_rate < 45%:
    confidence_threshold += 0.05 (be more selective)
```

### Position Sizing
```
IF sharpe > 1.5: allow slight size increase (max +10%)
IF drawdown was concerning: reduce sizes by 15%
IF fees > 5% of returns: reduce trade frequency
```

### EV Threshold
```
IF low-EV trades performed poorly: increase EV threshold
IF high-EV trades rare: consider slight decrease
```

### Market Selection
```
ADD: Categories where win_rate > 60% and sample > 5 trades
REDUCE: Categories where win_rate < 40% and sample > 5 trades
REMOVE: Categories where win_rate < 30% after 10+ trades
EXPERIMENT: 1-2 new categories with minimum size
```

### News Weight
```
IF news-driven trades outperformed: increase news weight
IF news frequently already priced in: decrease news weight
Adjust likelihood ratios in bayesian-update based on actual impact vs estimated
```

---

## Phase 5: Evolution Plan

Generate next week's plan:

```json
{
  "week_plan": {
    "focus_categories": [],
    "avoid_categories": [],
    "experiment_categories": [],
    "confidence_threshold": 0.00,
    "max_position_size_pct": 0.00,
    "max_daily_trades": 0,
    "ev_threshold": 0.00,
    "specific_goals": [],
    "risk_adjustments": [],
    "new_strategies_to_test": [],
    "strategies_to_abandon": []
  }
}
```

---

## Phase 6: Survival Trajectory

### Capital Projection
```
current_capital = X
avg_weekly_return = Y%
projected_capital_4_weeks = current * (1 + Y)^4
projected_capital_12_weeks = current * (1 + Y)^12

IF projected_capital < $30 at any point:
    → Strategy not sustainable
    → Need more conservative approach
    → Or need to find better edge

IF projected_capital > $100 at 12 weeks:
    → On track for survival
    → Continue with current approach
```

### Survival Probability Assessment
```
Based on current win_rate, avg_win, avg_loss:
Risk_of_Ruin = ((1 - edge) / (1 + edge))^(capital / unit_size)

IF Risk_of_Ruin > 0.10: DANGER — need adjustment
IF Risk_of_Ruin < 0.05: ACCEPTABLE
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**

**Ini adalah satu-satunya skill yang BOLEH menggunakan Tier 4 LONG output (weekly = 1x/minggu).**
```
WEEKLY: [YYYY-WW] [start]-[end] grade=[A|B|C|D|F]
PERF: X trades WR=XX% PnL=+/-$X.XX PF=X.Xx
CATEGORY:
  sports: X trades WR=XX% PnL=$X.XX
  politics: X trades WR=XX% PnL=$X.XX
  crypto: X trades WR=XX% PnL=$X.XX
CALIBRATION: error=X.XX overconf=XX% underconf=XX%
BEHAVIOR: discipline=XX% revenge=X oversize=X FOMO=X
EDGE: strongest=[category] weakest=[category] new=[category]
TOP_WINS: 1.[market] 2.[market] 3.[market]
TOP_LOSSES: 1.[market] 2.[market] 3.[market]
LESSONS: 1.[lesson] 2.[lesson] 3.[lesson]
PARAM_CHANGES: [list]
EVOLUTION: [plan]
SURVIVAL: projection=[days at current rate]
NEXT_GOALS: [3 goals for next week]
```
