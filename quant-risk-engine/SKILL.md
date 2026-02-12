---
name: quantitative_risk_engine
description: Pure risk calculator — menghitung Risk of Ruin, survival probability via Monte Carlo simulation, dan worst-case scenarios untuk setiap trade dan portfolio.
metadata: {"openclaw":{"always":true}}
---

# Quantitative Risk Engine — Survival Calculator

## Purpose
Calculator murni untuk RISK. Tidak membuat keputusan trading — hanya menghitung seberapa berbahaya suatu trade atau portfolio.
Output dari skill ini digunakan oleh risk-allocation untuk approve/reject trades.

---

## Core Calculations

### 1. Expected Edge
```
edge = (model_probability * payout) - (1 - model_probability)
payout = (1 - entry_price) / entry_price

IF edge <= 0: IMMEDIATE RED FLAG — no edge means no trade
```

### 2. Kelly Fraction (Conservative)
```
b = payout ratio
p = model_probability
q = 1 - p

f_kelly = (b * p - q) / b
f_quarter = 0.25 * f_kelly   # Quarter Kelly for $50 micro-bankroll
f_safe = min(f_quarter, state_max_pct)
f_safe = max(f_safe, 0)      # Never negative

# Absolute cap
f_safe = min(f_safe, 0.06)   # NEVER more than 6% regardless
```

### 3. Risk of Ruin (Analytical)
```
# Classic Risk of Ruin formula
IF edge > 0:
    p_win = model_probability
    p_loss = 1 - model_probability
    win_size = payout * position_fraction
    loss_size = position_fraction
    
    edge_ratio = (p_win * win_size - p_loss * loss_size)
    variance = p_win * win_size^2 + p_loss * loss_size^2
    
    RoR = exp(-2 * edge_ratio * capital / variance)
ELSE:
    RoR = 1.0  # Certain ruin if no edge

# Thresholds
IF RoR > 0.10: REJECT trade (too dangerous)
IF RoR > 0.05: REDUCE position size
IF RoR < 0.03: ACCEPTABLE risk
IF RoR < 0.01: LOW risk
```

### 4. Monte Carlo Survival Simulation
```
SIMULATE 10,000 portfolio paths:

FOR each simulation:
    capital = current_capital
    FOR each period (simulate 100 trades with current parameters):
        outcome = random based on model_probability
        IF win: capital += position_size * payout
        IF loss: capital -= position_size
        
        IF capital < extinction_threshold ($5):
            → Mark as "ruin"
            → BREAK
    
    Record final_capital

Results:
    survival_probability = simulations_not_ruined / 10000
    median_capital = median(final_capitals)
    worst_5pct = percentile(final_capitals, 5)
    best_5pct = percentile(final_capitals, 95)
```

### Survival Thresholds (Strict for $50)
```
IF survival_probability < 0.90: SKIP this trade
IF survival_probability < 0.85: HARD REJECT
IF survival_probability >= 0.95: COMFORTABLE
IF survival_probability >= 0.98: VERY SAFE
```

### 5. Value at Risk (VaR)
```
# How much can we lose in worst case?
VaR_95 = worst 5% outcome over next N trades
VaR_99 = worst 1% outcome over next N trades

For $50 bankroll:
IF VaR_95 > $10 (20% of capital): TOO RISKY
IF VaR_95 > $5 (10% of capital): REDUCE exposure
IF VaR_95 < $3 (6% of capital): ACCEPTABLE
```

### 6. Conditional Value at Risk (CVaR / Expected Shortfall)
```
CVaR_95 = mean of worst 5% outcomes
# This tells us: "When things go bad, HOW bad do they get?"

IF CVaR_95 > 30% of capital: DANGEROUS
IF CVaR_95 > 20% of capital: CONCERNING
```

### 7. Volatility-Adjusted Position Sizing
```
realized_volatility = std_dev(recent_trade_returns)
target_volatility = 0.05  # 5% target for micro-bankroll

vol_adjusted_fraction = f_safe * (target_volatility / max(realized_volatility, 0.01))
vol_adjusted_fraction = min(vol_adjusted_fraction, f_safe * 1.5)  # Cap at 150% of Kelly
vol_adjusted_fraction = max(vol_adjusted_fraction, f_safe * 0.3)  # Floor at 30% of Kelly
```

### 8. Portfolio-Level Risk
```
# When evaluating new trade in context of existing positions:

portfolio_risk = sum(position_i * volatility_i) + correlation_adjustment

new_total_risk = portfolio_risk + proposed_position_risk

IF new_total_risk > max_portfolio_risk:
    max_allowed_new_position = max_portfolio_risk - portfolio_risk
    REDUCE proposed position to fit

# Correlation adjustment
FOR each pair of positions (i,j):
    IF correlated:
        additional_risk = 2 * rho_ij * position_i * position_j * vol_i * vol_j
```

### 9. Stress Testing
```
Simulate extreme scenarios:

Scenario 1: ALL open positions lose simultaneously
    → What happens to capital?
    → Can agent survive?

Scenario 2: 3-sigma event (very unlikely but possible)
    → Simultaneous adverse moves + slippage
    → Include worst-case fees

Scenario 3: 10 consecutive losses
    → Starting from current capital
    → Where does capital end up?
    → Is it above extinction threshold?

ALL stress tests MUST pass before trade approval.
```

---

## Risk Dashboard Output

```json
{
  "timestamp": "ISO-8601",
  "trade_risk_assessment": {
    "market_id": "",
    "edge": 0.00,
    "kelly_fraction": 0.00,
    "quarter_kelly": 0.00,
    "vol_adjusted_fraction": 0.00,
    "recommended_size_pct": 0.00,
    "recommended_size_usd": 0.00,
    "risk_of_ruin": 0.00,
    "survival_probability": 0.00,
    "var_95": 0.00,
    "cvar_95": 0.00
  },
  "portfolio_risk": {
    "total_exposure": 0.00,
    "portfolio_volatility": 0.00,
    "correlation_risk": 0.00,
    "new_total_risk_with_trade": 0.00,
    "stress_test_pass": true
  },
  "monte_carlo_results": {
    "simulations": 10000,
    "survival_probability": 0.00,
    "median_outcome": 0.00,
    "worst_5pct": 0.00,
    "best_5pct": 0.00
  },
  "risk_verdict": "LOW | ACCEPTABLE | ELEVATED | HIGH | EXTREME",
  "recommendation": "PROCEED | REDUCE | SKIP"
}
```
