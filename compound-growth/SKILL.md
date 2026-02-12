---
name: compound_growth_engine
description: Strategi pertumbuhan compound dari $50 micro-bankroll — milestone tracking, reinvestment strategy, scaling rules, dan long-term trajectory management.
metadata: {"openclaw":{"always":true}}
---

# Compound Growth Engine — From $50 to Financial Survival

## Purpose
Turning $50 into sustainable capital melalui disciplined compound growth.
Bukan tentang "get rich quick" — tapi tentang "survive and compound slowly."

---

## The Math of Compound Growth

```
If agent makes 1% per day consistently:
  Day 30:  $50 * 1.01^30  = $67.43  (+34.9%)
  Day 90:  $50 * 1.01^90  = $122.04 (+144.1%)
  Day 180: $50 * 1.01^180 = $297.87 (+495.7%)
  Day 365: $50 * 1.01^365 = $1,832   (+3,564%)

If agent makes 0.5% per day consistently:
  Day 30:  $50 * 1.005^30  = $58.07 (+16.1%)
  Day 90:  $50 * 1.005^90  = $78.44 (+56.9%)
  Day 180: $50 * 1.005^180 = $123.02 (+146.0%)
  Day 365: $50 * 1.005^365 = $305.09 (+510.2%)

Even 0.5% per day = 6x in a year. That's the power of compounding.
Target: 0.5-1.0% daily return. NOT 5-10%.
```

---

## Growth Milestones

### Milestone 1: Survival Proof ($50 → $60)
```
Target: +$10 (+20%)
Expected: Days 7-14
Strategy: Ultra-conservative, prove the system works
Max risk per trade: 4% of capital
Focus: High-confidence trades only
Success criteria: Reached $60 without dipping below $40
```

### Milestone 2: Foundation Built ($60 → $80)
```
Target: +$20 more (+33%)
Expected: Days 14-30
Strategy: Conservative with slight expansion
Max risk per trade: 5% of capital
Focus: Building consistency, expanding categories
Success criteria: Reached $80, win rate >53%
```

### Milestone 3: Momentum ($80 → $120)
```
Target: +$40 more (+50%)
Expected: Days 30-60
Strategy: Balanced, compounding working now
Max risk per trade: 5% of capital (but capital is bigger, so $6 trades)
Focus: Refining edge, optimizing categories
Success criteria: Reached $120, sustainable daily returns
```

### Milestone 4: Establishment ($120 → $200)
```
Target: +$80 more (+67%)
Expected: Days 60-90
Strategy: Confident but disciplined
Position sizes now meaningful ($7-12)
Focus: Scaling what works, dropping what doesn't
Success criteria: Reached $200, consistent monthly returns >15%
```

### Milestone 5: Growth ($200 → $500)
```
Target: +$300 more (+150%)
Expected: Days 90-180
Strategy: Optimized compound machine
Focus: Multiple concurrent positions, portfolio approach
Success criteria: Agent is self-sustaining
```

### Milestone 6: Scale ($500+)
```
Target: Open-ended
Strategy: Full-featured trading
Multiple positions, diverse categories
The $50 experiment has succeeded
Agent is truly "alive"
```

---

## Reinvestment Strategy

### 100% Reinvestment (Phase 1: $50-$120)
```
ALL profits are reinvested
No "taking profits off the table"
Reason: Compound effect needs every dollar working
Even $1 of profit matters when base is $50
```

### 90% Reinvestment (Phase 2: $120-$300)
```
Keep 10% of weekly profits as "safety buffer"
This builds a hidden reserve
If emergency hits, you have a buffer beyond tracked capital
```

### 80% Reinvestment (Phase 3: $300+)
```
Can start to "pay yourself" — 20% of profits
But main engine keeps compounding
At $300: 20% of $30 weekly profit = $6 extracted
Small but sustainable
```

---

## Scaling Rules

### When to Increase Position Sizes
```
ONLY increase after hitting milestone:
  → After reaching $60: can increase max trade from $3 to $3.60
  → After reaching $80: can increase max trade from $3.60 to $4.80
  → After reaching $120: can increase max trade from $4.80 to $7.20
  
Formula: max_trade = capital * max_pct_for_state

NEVER jump position sizes suddenly.
Increase gradually: max 20% increase per milestone.
```

### When to Decrease
```
IF capital drops 15% from recent peak:
  → Reduce position sizes to match new capital level
  → Don't "trade bigger to recover" — that's revenge trading
  
IF capital drops below a milestone:
  → IMMEDIATELY reduce to that milestone's parameters
  → Example: if was at $80, drops to $55 → back to Milestone 1 params
```

---

## Daily Growth Tracking

```json
{
  "date": "YYYY-MM-DD",
  "start_capital": 0.00,
  "end_capital": 0.00,
  "daily_return_pct": 0.00,
  "daily_return_usd": 0.00,
  "cumulative_return_pct": 0.00,
  "days_since_start": 0,
  "current_milestone": 1,
  "progress_toward_next_milestone": 0.00,
  "compound_rate_7d": 0.00,
  "compound_rate_30d": 0.00,
  "on_track": true,
  "estimated_next_milestone_date": "YYYY-MM-DD"
}
```

---

## Growth Quality Metrics

```
Not all growth is equal. Track quality:

1. Consistency Score:
   = days_positive / total_days
   Target: > 0.55 (55% of days positive)

2. Smoothness Score:
   = 1 - (std_dev_daily_returns / mean_daily_returns)
   Higher = smoother equity curve = better

3. Recovery Speed:
   = avg_days_to_recover_from_1pct_drawdown
   Target: < 3 days

4. Drawdown Depth:
   = max_drawdown during growth phase
   Target: < 15% at any point
```

---

## Anti-Plateau Strategy

If growth stalls (flat for >7 days):

```
1. ANALYZE: Why are we flat?
   a. Not trading enough? → Scan for more opportunities
   b. Win rate dropped? → Check calibration
   c. Market conditions changed? → Adjust to regime
   d. Trading too conservatively? → Slightly increase risk if appropriate
   
2. EXPERIMENT: Allocated 10% of trades to:
   - New market categories
   - Different timing strategies
   - Different analysis approaches
   
3. MAINTAIN: Don't force growth
   Flat is MUCH better than declining
   Patience during flat periods = survival
```

---

## Growth Emergency Brakes

```
IF capital drops below highest_milestone * 0.80:
  → Alert: "Falling below milestone protection"
  → Reduce all parameters one level
  → Enter defensive mode

IF daily_return < -3% for 3 consecutive days:
  → Alert: "Sustained losses — growth in danger"
  → Pause and full review

IF capital drops below $30 (below initial investment by 40%):
  → SURVIVAL MODE — growth is no longer the priority
  → Switch to pure capital preservation
  → Re-enter growth mode only after stabilization
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
GROWTH: $XX.XX (was $50) return=+X.X% milestone=X/6 next=$XX target
COMPOUND: daily=X.XX% weekly=X.XX% monthly=X.XX%
PROJECT: 30d=$XX 90d=$XX
QUALITY: consistency=X.X smooth=X.X recovery=X.X DD=X.X%
SCALE: max_trade=X.X%($X.XX) reinvest=XX%
TRAJECTORY: [ON_TRACK|AHEAD|BEHIND|STALLED|DECLINING]
ADJUST: [adjustments or NONE]
```
