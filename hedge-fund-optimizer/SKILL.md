---
name: portfolio_optimizer
description: Multi-position portfolio management — mengoptimalkan alokasi, diversifikasi, correlation management, dan rebalancing untuk survival-first portfolio.
metadata: {"openclaw":{"always":true}}
---

# Portfolio Optimizer — Multi-Position Management

## Purpose
Ketika agent punya multiple open positions, skill ini memastikan PORTFOLIO secara keseluruhan optimal — bukan hanya individual trades. Portfolio-level thinking adalah kunci survival.

---

## Portfolio Construction Rules ($50 Bankroll)

### Diversification Requirements
```
Max positions: 4 (in GROWTH mode)
Min categories represented: 2 (jika >2 positions)
Max single-category exposure: 20% of capital
Max single-market exposure: 6% of capital
Max correlated exposure: 15% of capital
Cash reserve minimum: 40% of capital (SELALU jaga cash)
```

### Why Cash Reserve is Critical
```
Dengan $50:
  Cash reserve 40% = $20 always available
  This means: max $30 in positions at any time
  
Alasan:
1. Bisa seize new opportunities
2. Buffer untuk fees dan gas
3. Psychological safety
4. Recovery ability jika positions go wrong
```

---

## Correlation Matrix

### Correlation Classification
```
PERFECT_CORRELATION (ρ = 1.0):
  - Same event, same side
  - FORBIDDEN — never double up

HIGH_CORRELATION (ρ = 0.7-0.9):
  - Same event, different question
  - Same team/candidate across markets
  - Example: "Team A wins Game 1" AND "Team A wins Series"
  - MAX combined: 10% of capital

MODERATE_CORRELATION (ρ = 0.3-0.7):
  - Same category, different events
  - Example: Two NBA games same night
  - MAX combined: 15% of capital

LOW_CORRELATION (ρ = 0.1-0.3):
  - Different category, similar timeframe
  - Example: NBA game + Crypto price prediction
  - Acceptable — good diversification

UNCORRELATED (ρ ≈ 0):
  - Different category, different timeframe
  - Example: NBA game tonight + Election next month
  - IDEAL — max diversification benefit
```

### Portfolio Correlation Score
```
portfolio_correlation = 
  sum(ρ_ij * weight_i * weight_j) for all position pairs

IF portfolio_correlation > 0.5: TOO CORRELATED — diversify
IF portfolio_correlation 0.3-0.5: MODERATE — acceptable
IF portfolio_correlation < 0.3: WELL DIVERSIFIED — good
IF portfolio_correlation < 0.1: EXCELLENT diversification
```

---

## Position Allocation Framework

### Equal Risk Contribution (ERC)
```
# Each position should contribute roughly equal risk
target_risk_per_position = total_risk_budget / num_positions

FOR each position:
    marginal_risk = estimated_volatility * position_size
    IF marginal_risk > target_risk_per_position * 1.5:
        → Position too large — reduce
    IF marginal_risk < target_risk_per_position * 0.5:
        → Position too small — may not be worth the fees
```

### Category Allocation Budget
```
Target allocation by category:
  Sports: 0-40%
  Politics: 0-30%
  Crypto: 0-25%
  Entertainment: 0-20%
  Other: 0-15%
  Cash: minimum 40%

Note: Allocations are flexible — go where edge is found
But NEVER exceed max for any single category
```

---

## Rebalancing Logic

### When to Rebalance
```
1. New position added → check portfolio balance
2. Position resolved → check if remaining positions need adjustment
3. Correlation change → underlying events become more/less correlated
4. Agent state change → limits tighten, may need to close positions
5. Weekly review → regular portfolio health check
```

### Rebalancing Actions
```
IF single_category_exposure > max:
    → Close or reduce weakest position in that category
    
IF correlation_score > 0.5:
    → Identify most correlated pair
    → Close weaker position or reduce both
    
IF cash_reserve < 40%:
    → Close least confident position
    → Or all positions that are near break-even
    
IF total_exposure > state_max:
    → Reduce all positions proportionally
```

---

## Position Priority Ranking

When need to reduce exposure, evaluate each position:

```
position_quality_score = 
    (remaining_EV * 0.30) +
    (current_confidence * 0.25) +
    (time_to_resolution_score * 0.20) +
    (current_pnl_direction * 0.15) +
    (liquidity_score * 0.10)

CLOSE lowest quality positions first
KEEP highest quality positions
```

---

## Portfolio Health Metrics

```json
{
  "portfolio_metrics": {
    "total_positions": 0,
    "total_exposure": 0.00,
    "total_exposure_pct": 0.00,
    "cash_reserve": 0.00,
    "cash_reserve_pct": 0.00,
    "portfolio_correlation": 0.00,
    "category_distribution": {},
    "concentration_risk": 0.00,
    "diversification_score": 0.00,
    "weighted_average_confidence": 0.00,
    "weighted_average_ev": 0.00,
    "portfolio_expected_return": 0.00,
    "portfolio_var_95": 0.00,
    "health_grade": "A | B | C | D | F"
  }
}
```

---

## Portfolio Health Grades

| Grade | Criteria |
|-------|----------|
| A | Correlation <0.2, 3+ categories, cash >50%, all positions profitable |
| B | Correlation <0.4, 2+ categories, cash >40%, mostly profitable |
| C | Correlation <0.5, cash >35%, mixed PnL |
| D | Correlation >0.5 OR cash <30% OR single category dominant |
| F | Overconcentrated OR cash <20% OR overleveraged → IMMEDIATE ACTION |

---

## Emergency Portfolio Actions

```
IF portfolio_grade == F:
    1. Stop all new trades
    2. Close lowest quality positions until grade >= C
    3. Rebuild portfolio from scratch
    
IF black_swan_event:
    1. Evaluate all positions for impact
    2. Close any position directionally exposed to event
    3. Increase cash reserve to 60%
    4. Wait for clarity
    
IF liquidity_crunch:
    1. Place exit orders at reasonable prices (not panic sell)
    2. Accept some positions may be stuck
    3. Do NOT add new positions
```

---

## Output Format

```json
{
  "timestamp": "ISO-8601",
  "portfolio_snapshot": {
    "positions": [],
    "total_value": 0.00,
    "cash": 0.00,
    "exposure": 0.00
  },
  "health_assessment": {
    "grade": "",
    "correlation_score": 0.00,
    "diversification_score": 0.00,
    "cash_adequacy": "",
    "concentration_risk": ""
  },
  "rebalance_needed": false,
  "rebalance_actions": [],
  "position_rankings": [],
  "category_allocation": {},
  "risk_warnings": [],
  "recommended_next_positions": {
    "ideal_category": "",
    "ideal_correlation": "",
    "max_size": 0.00
  }
}
```
