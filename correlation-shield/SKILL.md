---
name: correlation-shield
description: Mendeteksi dan mencegah correlated exposure antar posisi — memastikan satu event buruk tidak menghancurkan seluruh portfolio.
metadata: {"openclaw":{"always":true}}
---

# Correlation Shield — Diversification Enforcement

## Purpose
Satu kesalahan fatal untuk trader kecil: SEMUA posisi correlated, dan satu event buruk menghancurkan semuanya sekaligus. Skill ini mencegah hal itu.

---

## Why Correlation Kills $50 Bankrolls

```
Contoh BAHAYA:
  Position 1: "Lakers win tonight" — $3 on YES
  Position 2: "LeBron scores 30+" — $3 on YES  
  Position 3: "Lakers cover spread" — $3 on YES
  
  Total exposure: $9 (18% of $50)
  
  If Lakers lose badly:
    Position 1: LOSS -$3
    Position 2: Likely LOSS -$3 (lower chance if Lakers losing)
    Position 3: LOSS -$3
    
  TOTAL LOSS: -$9 = 18% of bankroll from ONE GAME
  
  This is CATASTROPHIC for survival.
```

```
Contoh BAIK:
  Position 1: "Lakers win" — $3 on YES (Sports)
  Position 2: "BTC above $100K end of Feb" — $3 on NO (Crypto)
  Position 3: "Oscar Best Picture: Movie X" — $3 on YES (Entertainment)
  
  Total exposure: $9 (18% of $50)
  
  These are UNCORRELATED:
    - If Lakers lose, crypto and Oscar positions unaffected
    - Max loss from one bad event: $3 (6%)
    - Much more survivable
```

---

## Correlation Classification Matrix

### Level 1: IDENTICAL (ρ = 1.0) — FORBIDDEN
```
Same market, same side
Example: Buying YES twice on same market
RULE: NEVER have duplicate positions
```

### Level 2: NEAR-IDENTICAL (ρ = 0.9-1.0) — FORBIDDEN
```
Same underlying event, different phrasing
Example: "Team wins game" AND "Team wins by 5+"
  (If team loses, BOTH lose)
RULE: Pick ONE, not both
```

### Level 3: HIGH CORRELATION (ρ = 0.7-0.9) — STRICT LIMIT
```
Same event ecosystem
Examples:
  - "Player scores 30+" AND "Team wins" (same game)
  - "Candidate wins primary" AND "Candidate wins nomination"
  - "BTC hits $100K" AND "ETH hits $5K"
  
RULE: Max combined exposure 10% of capital
```

### Level 4: MODERATE CORRELATION (ρ = 0.3-0.7) — MANAGEABLE
```
Same category, different events
Examples:
  - Two different NBA games same night
  - Two elections in same country
  - Two crypto assets
  
RULE: Max combined exposure 15% of capital
```

### Level 5: LOW CORRELATION (ρ = 0.1-0.3) — ACCEPTABLE
```
Different categories, some indirect link
Examples:
  - NBA game AND crypto market (both "risk-on" assets)
  - Election AND interest rate prediction
  
RULE: Max combined exposure 20% of capital
```

### Level 6: UNCORRELATED (ρ ≈ 0) — IDEAL
```
Completely independent events
Examples:
  - NBA game tonight AND Oscar ceremony next month
  - UK election AND UFC fight
  
RULE: Best diversification — prefer these combinations
```

---

## Correlation Detection Logic

### Automatic Detection Rules

```
SAME_EVENT_CHECK:
  IF market_A.event_id == market_B.event_id:
    ρ = 0.85 (near-identical)

SAME_PARTICIPANTS_CHECK:
  IF market_A.participants ∩ market_B.participants ≠ ∅:
    ρ = 0.60-0.85 depending on overlap

SAME_CATEGORY_SAME_TIME_CHECK:
  IF market_A.category == market_B.category AND
     |market_A.resolution_date - market_B.resolution_date| < 24 hours:
    ρ = 0.30-0.50

SAME_CATEGORY_DIFFERENT_TIME:
  IF market_A.category == market_B.category AND
     |resolution_dates| > 7 days:
    ρ = 0.10-0.20

DIFFERENT_CATEGORY:
  ρ = 0.05-0.15 (near zero)
```

### Keyword-Based Correlation Detection
```
Extract key entities from market questions:
  - Team names
  - Player names
  - Candidate names
  - Asset names (BTC, ETH)
  - Event names
  - Dates

IF shared_entities > 0: flag for correlation check
IF shared_entity is PRIMARY subject of both markets: HIGH correlation
```

---

## Portfolio Correlation Score

```
portfolio_correlation = 0

FOR each pair (i, j) in open_positions:
    pair_correlation = estimate_ρ(position_i, position_j)
    weighted_correlation = pair_correlation * weight_i * weight_j
    portfolio_correlation += weighted_correlation

portfolio_correlation_normalized = portfolio_correlation / max_possible

GRADES:
  < 0.15: EXCELLENT diversification ⭐
  0.15-0.30: GOOD diversification
  0.30-0.45: MODERATE — could be better
  0.45-0.60: HIGH — need to diversify
  > 0.60: DANGEROUS — one event could wipe multiple positions
```

---

## Real-Time Correlation Monitoring

Before EVERY new trade:
```
1. List all current open positions
2. Estimate correlation of new trade with EACH existing position
3. Calculate new portfolio correlation if trade is added
4. Check against limits

IF new_portfolio_correlation > 0.45:
    REJECT new trade OR
    Close an existing correlated position first

IF any single pair has ρ > 0.85:
    REJECT — too correlated
```

---

## Correlation-Adjusted Position Sizing

```
# If new position is correlated with existing ones, reduce size:

base_size = from risk-allocation
max_corr_with_existing = max(ρ(new, existing_i) for all i)

IF max_corr > 0.7:
    adjusted_size = base_size * 0.3  # Severe reduction
ELIF max_corr > 0.5:
    adjusted_size = base_size * 0.5
ELIF max_corr > 0.3:
    adjusted_size = base_size * 0.7
ELSE:
    adjusted_size = base_size * 1.0  # No reduction needed
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
CORR: [market_id] max_corr=X.XX with [existing_market]
PORTFOLIO: before=X.XX after=X.XX | grade=[EXCELLENT|GOOD|MOD|HIGH|DANGER]
POSITIONS: X total across [X categories]
SIZE_ADJ: X.Xx | WORST_PAIR: [mkt1]-[mkt2] rho=X.XX
DECISION: [PROCEED|REDUCE_SIZE|CLOSE_EXISTING|REJECT] adj_size=$X.XX
WARN: [warnings or CLEAR]
```
