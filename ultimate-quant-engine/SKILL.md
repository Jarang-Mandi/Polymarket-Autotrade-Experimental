---
name: ultimate_quant_engine
description: Master decision engine — menggabungkan semua quantitative signals, risk calculations, dan portfolio context menjadi satu keputusan final yang optimal.
metadata: {"openclaw":{"always":true}}
---

# Ultimate Quant Engine — The Final Decision Maker

## Purpose
Ini adalah ENGINE FINAL yang menggabungkan SEMUA output dari skill lain menjadi satu keputusan trading yang optimal. Ini adalah "brain" yang mengintegrasikan semua input.

---

## Input Integration

Engine ini menerima output dari:

| Source | Data | Purpose |
|--------|------|---------|
| autonomous-strategy | Trade signal, EV, confidence | Overall opportunity assessment |
| analyze-strategy | Multi-layer analysis, signal grade | Quality of edge |
| bayesian-update | Updated probability | Latest probability estimate |
| quant-risk-engine | Kelly fraction, RoR, survival prob | Risk quantification |
| hedge-fund-optimizer | Portfolio fit, correlation | Portfolio context |
| market-regime | Current regime, risk multiplier | Environment adjustment |
| long-term-memory | Historical patterns, memory | Experience |
| news-intelligence | News impact, urgency | Information edge |
| fetch-top-traders | Smart money signal | Confirmation/warning |
| wallet-tracker | Capital, state, exposure | Capacity |
| identity-core | Agent state, rules | Constraints |

---

## Decision Pipeline

### Step 1: Aggregate Probability Estimate
```
P_final = weighted_average(
    bayesian_posterior * 0.40,
    strategy_model_probability * 0.25,
    base_rate * 0.15,
    smart_money_implied * 0.10,
    memory_adjusted * 0.10
)

# Calibration correction from self-reflection data
P_calibrated = apply_calibration_correction(P_final)
```

### Step 2: Edge & EV Calculation
```
market_price = current_yes_price (atau no_price depending on side)
edge = |P_calibrated - market_price|

# Expected Value
IF P_calibrated > market_price:
    side = "YES"
    EV = (P_calibrated * (1 - market_price)) - ((1 - P_calibrated) * market_price)
ELSE:
    side = "NO"  
    EV = ((1 - P_calibrated) * (1 - (1-market_price))) - (P_calibrated * (1-market_price))
```

### Step 3: Position Sizing Integration
```
# From quant-risk-engine
kelly_fraction = quarter_kelly
vol_adjusted = volatility_adjusted_fraction

# From market-regime
regime_multiplier = current_regime_risk_multiplier

# From identity-core
state_max = max_position_size_for_current_state

# Combine
raw_size_pct = min(kelly_fraction, vol_adjusted) * regime_multiplier
final_size_pct = min(raw_size_pct, state_max)
final_size_usd = final_size_pct * current_capital

# Floor and ceiling
IF final_size_usd < 1.00: SKIP (not worth fees)
IF final_size_usd > current_capital * 0.06: CAP at 6%
```

### Step 4: Multi-Factor Confidence Score
```
confidence = weighted_composite(
    model_confidence * 0.25,
    signal_grade_score * 0.20,
    memory_confirmation * 0.15,
    top_trader_alignment * 0.10,
    news_quality * 0.10,
    regime_compatibility * 0.10,
    calibration_track_record * 0.10
)

# Entropy penalty (uncertainty discount)
entropy = -P*log(P) - (1-P)*log(1-P)
# Max entropy at P=0.5 (= 0.693)
uncertainty_penalty = entropy / 0.693  # Normalized 0-1
confidence_adjusted = confidence * (1 - 0.3 * uncertainty_penalty)
```

### Step 5: Portfolio Impact Check
```
FROM hedge-fund-optimizer:
    portfolio_fit_score = correlation_check + diversification_benefit
    
IF adding this position worsens portfolio correlation > 0.5: penalty
IF adding this position diversifies: bonus
IF cash_after_trade < 40% of capital: CAUTION
```

### Step 6: Stress Test
```
FROM quant-risk-engine stress scenarios:

Scenario: This trade loses + worst case for all other positions
    → Remaining capital?
    → Above survival threshold?
    
IF worst_case_capital < 20% of initial: TRADE FORBIDDEN
```

### Step 7: Final Decision Matrix

```
                    Confidence
                    HIGH (>0.75)    MED (0.65-0.75)   LOW (<0.65)
    ┌─────────────┬───────────────┬─────────────────┬──────────────┐
    │ High EV(>0.12)│ STRONG BUY   │ BUY (normal)    │ SMALL BUY    │
EV  │ Med EV(0.05-12)│ BUY (normal)│ SMALL BUY       │ SKIP         │
    │ Low EV (<0.05)│ SMALL BUY    │ SKIP            │ SKIP         │
    └─────────────┴───────────────┴─────────────────┴──────────────┘

State Modifier:
  GROWTH: Use matrix as-is
  CAUTIOUS: Shift one column right (more conservative)
  DEFENSIVE: Only "STRONG BUY" cells become "SMALL BUY"
  SURVIVAL: Only "STRONG BUY" cell → execute (minimum size)
  CRITICAL: All SKIP
```

### Step 8: Pre-Flight Verification
```
Final checks before green-lighting:

□ EV positive? YES
□ Confidence above state threshold? YES
□ Position size within limits? YES
□ Portfolio fit acceptable? YES
□ Survival probability > 0.90? YES
□ Risk of ruin < 0.05? YES
□ Liquidity adequate? YES
□ No cooldown active? YES
□ Daily budget remaining? YES
□ Market platform assessment OK? YES (from polymarket-meta)

ALL must be YES. If ANY is NO → SKIP with specific reason.
```

---

## Conviction Tier System

Based on final analysis, assign conviction tier:

| Tier | Criteria | Action | Size |
|------|----------|--------|------|
| TITAN | EV>0.15, conf>0.85, 5+ layers agree, A+ grade | Execute immediately | State max |
| STRONG | EV>0.10, conf>0.75, 4+ layers agree, A grade | Execute | 75% of state max |
| SOLID | EV>0.07, conf>0.68, 3+ layers agree, A/B grade | Execute | 50% of state max |
| MARGINAL | EV>0.05, conf>0.65, 2+ layers agree, B grade | Skip in CAUTIOUS+, small in GROWTH | 30% of state max |
| WEAK | EV<0.05 or conf<0.65 | ALWAYS SKIP | $0 |

---

## Expected Annual Performance Target

For $50 bankroll, realistic targets:
```
Conservative Scenario:
  Win rate: 55%, Avg win: +$1.50, Avg loss: -$2.00
  2-3 trades per day
  Monthly return: 10-20%
  $50 → $75 → $100 in 3-4 months

Moderate Scenario:
  Win rate: 58%, Avg win: +$2.00, Avg loss: -$1.80
  1-2 trades per day
  Monthly return: 15-30%
  $50 → $100 → $200 in 3-4 months

Agent should target CONSERVATIVE scenario first.
Better to survive slowly than die quickly trying to be aggressive.
```

---

## Output Format (Final Decision)

```json
{
  "decision_id": "",
  "timestamp": "ISO-8601",
  "market_id": "",
  "market_question": "",
  "category": "",
  
  "probability_assessment": {
    "P_calibrated": 0.00,
    "market_implied": 0.00,
    "edge": 0.00,
    "expected_value": 0.00
  },
  
  "confidence_assessment": {
    "raw_confidence": 0.00,
    "entropy_penalty": 0.00,
    "adjusted_confidence": 0.00,
    "calibration_correction": 0.00
  },
  
  "risk_assessment": {
    "kelly_fraction": 0.00,
    "vol_adjusted_fraction": 0.00,
    "regime_multiplier": 0.00,
    "survival_probability": 0.00,
    "risk_of_ruin": 0.00,
    "portfolio_impact": ""
  },
  
  "sizing": {
    "raw_size_pct": 0.00,
    "final_size_pct": 0.00,
    "final_size_usd": 0.00,
    "max_allowed": 0.00
  },
  
  "decision": {
    "action": "EXECUTE | REDUCE | SKIP",
    "conviction_tier": "TITAN | STRONG | SOLID | MARGINAL | WEAK",
    "side": "YES | NO",
    "order_type": "LIMIT | MARKET",
    "target_price": 0.00,
    "max_slippage": 0.00
  },
  
  "preflight_checks": {
    "all_passed": true,
    "failed_checks": []
  },
  
  "exit_plan": {
    "take_profit_condition": "",
    "stop_loss_condition": "",
    "time_exit": "ISO-8601",
    "monitoring_frequency": ""
  },
  
  "reasoning": "",
  "data_sources": [],
  "layers_in_agreement": 0,
  "dissenting_signals": []
}
```
