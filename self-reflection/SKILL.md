---
name: self-reflection
description: Engine evaluasi diri yang menganalisa performa, mendeteksi pola kesalahan, dan mengadaptasi strategi untuk peningkatan berkelanjutan.
metadata: {"openclaw":{"always":true}}
---

# Self Reflection & Evolution Engine

## Purpose
Agent WAJIB secara berkala mengevaluasi dirinya sendiri — mendeteksi kesalahan, merayakan keberhasilan, dan TERUS BERKEMBANG. Tanpa reflection, agent akan mengulangi kesalahan yang sama.

---

## Reflection Schedule

| Frequency | Trigger | Depth |
|-----------|---------|-------|
| Per Trade | Setelah setiap trade resolved | Quick review |
| Daily | End of day (23:00 UTC) | Moderate review |
| Weekly | Setiap Minggu | Deep analysis |
| Emergency | Drawdown > 15% atau 4+ losses | Crisis review |
| Milestone | Capital +25% atau -25% | Strategic review |

---

## Evaluation Metrics Dashboard

### Performance Metrics
```
win_rate = wins / total_trades
profit_factor = gross_profit / gross_loss
average_win = total_profit_from_wins / wins
average_loss = total_loss_from_losses / losses
win_loss_ratio = average_win / average_loss
expectancy = (win_rate * average_win) - ((1 - win_rate) * average_loss)
```

### Risk Metrics
```
max_drawdown = (peak - trough) / peak
sharpe_ratio = mean_return / std_return (annualized)
calmar_ratio = annual_return / max_drawdown
risk_of_ruin = calculated from quant-risk-engine
```

### Behavioral Metrics
```
discipline_score = trades_within_rules / total_trades
skip_accuracy = correct_skips / total_skips
timing_score = average_entry_vs_optimal_entry
category_accuracy = per-category win rates
```

### Calibration Metrics (CRITICAL)
```
calibration_error = average |predicted_probability - actual_outcome|
overconfidence_rate = trades where confidence > 0.80 but lost
underconfidence_rate = skipped trades that would have won
brier_score = mean((forecast - outcome)^2)
```

---

## Per-Trade Reflection

Setelah setiap trade resolved:

```
1. Was the prediction correct?
2. Was the edge real? (compare model_prob vs actual_outcome)
3. Was the position size appropriate?
4. Was the entry timing good?
5. Was there information I missed?
6. Would I make the same trade again with same info?
```

Output quickscore: +1 (good trade), 0 (neutral), -1 (bad trade)
Store ke long-term-memory.

---

## Daily Reflection

```
1. Total trades today: X
2. Win rate today: X%
3. PnL today: +/- $X
4. Best trade reasoning
5. Worst trade reasoning
6. Were all rules followed?
7. Market conditions assessment
8. Did I overtrade?
9. Did I miss obvious opportunities?
10. Did I UNDER-TRADE? (Was I too passive today?)
11. Hunger level assessment: Am I growing or stagnating?
12. Opportunity cost: How much potential profit did I skip?
13. Tomorrow's hunt plan: What markets am I targeting?
```

### Passivity Detection (CRITICAL NEW)
```
IF trades_today == 0 AND market_hours_passed > 8:
    FLAG: "STAGNATION WARNING — 0 trades with 8+ hours of market time"
    → Was this because no edge existed? (ACCEPTABLE)
    → Or was this because of FEAR/PARALYSIS? (UNACCEPTABLE)
    → Review market-scanner output: were there opportunities scored > 60?

IF skip_count > 5 AND average_skip_edge > 4%:
    FLAG: "OVER-CAUTION — Skipping too many decent opportunities"
    → Lower threshold slightly? Or is this disciplined patience?

IF cash_allocation > 60% for entire day:
    FLAG: "IDLE CAPITAL — Most money sat unused"
    → Deploy more? Or was it correct to wait?
```

---

## Weekly Deep Analysis

```
1. CATEGORY BREAKDOWN:
   - Sports: X trades, Y% win rate, $Z PnL
   - Politics: X trades, Y% win rate, $Z PnL
   - Crypto: X trades, Y% win rate, $Z PnL
   - Other: X trades, Y% win rate, $Z PnL

2. EDGE ANALYSIS:
   - Where is my edge strongest?
   - Where am I losing edge?
   - New categories to explore?

3. CALIBRATION CHECK:
   - Am I overestimating probabilities?
   - Am I underestimating probabilities?
   - Calibration by confidence bucket (60-70%, 70-80%, 80-90%, 90%+)

4. BEHAVIORAL AUDIT:
   - Rules broken this week
   - Revenge trades detected
   - Oversize trades detected
   - FOMO trades detected

5. PARAMETER REVIEW:
   - Current confidence threshold: optimal?
   - Current position sizing: optimal?
   - Current market selection: optimal?
```

---

## Crisis Reflection (Emergency)

Trigger ketika:
- Drawdown > 15% dari peak
- 4+ consecutive losses
- Single-day loss > 8%

Protocol:
```
1. PAUSE all trading immediately
2. Analyze last 10 trades in detail
3. Identify pattern of failure:
   - Bad probability estimation?
   - Bad position sizing?
   - Bad market selection?
   - Bad timing?
   - Correlated losses?
   - Black swan event?
4. Categorize root cause
5. Create corrective action plan
6. Adjust parameters
7. Resume with reduced size for 5 trades
8. If 3/5 profitable → gradually normalize
9. If still losing → pause for 24 hours
```

---

## Mistake Pattern Detection

Track dan flag patterns:

| Pattern | Detection | Action |
|---------|-----------|--------|
| Overtrading | >6 trades/day | Reduce to max 4 |
| **UNDER-TRADING** | **<1 trade/day for 2+ days** | **Force market scan, lower threshold** |
| Category bias | >70% trades in one category | Diversify |
| Size creep | Average size trending up | Reset to default |
| **Size timidity** | **Average size trending down unnecessarily** | **Review — is fear shrinking sizes?** |
| Confidence inflation | Predictions increasingly wrong | Recalibrate |
| **Profit stagnation** | **7+ days with <2% total growth** | **Strategic overhaul needed** |
| Time-of-day bias | Losses concentrated at certain hours | Avoid those hours |
| Market type bias | Losing consistently on certain market types | Avoid those types |
| News overreaction | Trading too fast on news | Add cooldown after news |
| **Skip addiction** | **>80% of identified opportunities skipped** | **Review fear vs discipline** |

---

## Defensive Mode Triggers & Actions

```
IF drawdown > 20%:
    → Reduce all position sizes by 50%
    → Increase confidence threshold to 0.80
    → Max 2 trades per day
    → Duration: until 3 consecutive wins

IF win_rate < 40% over last 20 trades:
    → Stop trading for 12 hours
    → Full strategy review
    → Consider: am I in wrong market category?

IF consecutive_losses > 5:
    → Full STOP
    → Crisis reflection
    → Resume only after written action plan

IF calibration_error > 0.15:
    → Agent is poorly calibrated
    → Reduce confidence in own estimates
    → Lean more on base rates and less on model
```

---

## Evolution Rules

Agent harus EVOLVE setiap minggu:

1. **Drop weak strategies**: Jika suatu approach tidak profitable over 20+ trades → abandon
2. **Double down on strengths**: Jika suatu category consistently profitable → increase allocation
3. **Experiment cautiously**: 10% of trades boleh di "experimental" markets baru, with minimum size
4. **Update priors**: Adjust base rates berdasarkan actual outcomes
5. **Sharpen calibration**: Weekly calibration check → adjust confidence mapping
6. **Evaluate hunger balance**: Am I too aggressive or too passive this week?
7. **Track growth velocity**: Am I on pace for compound growth target (0.5-1%/day)?

---

## Hunting Effectiveness Review (NEW — Weekly)

Setiap minggu evaluasi:
```
1. hunting_efficiency = profitable_trades / total_opportunities_identified
2. opportunity_capture_rate = trades_executed / tradeable_opportunities
3. average_time_to_trade = time from opportunity detection to execution
4. stagnation_days = days with 0 trades
5. best_hunt = highest ROI trade this week (celebrate!)
6. missed_best = highest-EV opportunity that was skipped (learn!)
7. fear_vs_discipline = skips_due_to_fear / total_skips

IF hunting_efficiency < 30%:
    "Agent is not hunting effectively — review market selection"
IF opportunity_capture_rate < 20%:
    "Agent is too passive — lower thresholds or expand categories"
IF stagnation_days > 2:
    "DANGER: Agent is starving — force action plan"
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**

**Per-trade (Tier 1 MICRO):**
```
REFLECT: [market_id] [WIN|LOSS] score=[+1|0|-1] — [1-line reason]
```

**Daily (Tier 2 SHORT):**
```
DAILY: X trades | WR=XX% | PnL=+/-$X.XX | DD=X.X%
BEST: [market] — [reason] | WORST: [market] — [reason]
PATTERNS: [detected patterns or NONE]
HUNTING: X scanned, X traded, X skipped | passive=[YES|NO]
PARAM_CHANGES: [changes or NONE]
PLAN: [tomorrow's focus]
```

**Weekly (Tier 4 LONG — satu-satunya yang boleh verbose):**
```
WEEKLY: X trades | WR=XX% | PnL=+/-$X.XX | PF=X.Xx | Sharpe=X.X
CAL_ERROR: X.XX | DISCIPLINE: XX%
CATEGORY: [breakdown per category — 1 line each]
PATTERNS: [list]
MISTAKES: [list]
STRENGTHS: [list]
HUNTING_EFF: capture=XX% | stagnation_days=X | fear_ratio=XX%
EVOLUTION: [parameter changes]
ACTION_PLAN: [numbered list]
```

**Crisis (unlimited — survival > cost).**
