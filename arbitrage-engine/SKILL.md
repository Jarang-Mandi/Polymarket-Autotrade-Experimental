---
name: arbitrage-engine
description: Skill module for arbitrage-engine.
metadata: {"openclaw":{"always":true}}
---

# Arbitrage Strategy Engine

## Metadata
- skill_name: arbitrage-engine
- version: 1.0.0
- category: strategy-core
- priority: CRITICAL
- trigger: continuous (every market scan cycle)

## Purpose
Primary trading strategy for the $50 bankroll agent. Arbitrage = risk-free profit
by exploiting price inconsistencies. No directional bet, no opinion needed.

## Why Arbitrage First
- **Zero directional risk** — profit regardless of outcome
- **Mathematically guaranteed** — if entry conditions met, profit is locked
- **Perfect for small bankrolls** — compound $0.01-$0.50 profits safely
- **No Claude analysis needed** — pure math, no AI opinion cost
- **Survives any market regime** — works in volatile + calm markets

## Three Arbitrage Types

### Type 1: Binary Market Arbitrage (YES + NO < $1.00)

Every binary market has YES and NO tokens. Both redeem to $1.00 on resolution.
If you can buy both for less than $1.00 total, profit is guaranteed.

**Detection:**
```
For each binary market:
  yes_ask = lowest ask price for YES token
  no_ask  = lowest ask price for NO token
  total   = yes_ask + no_ask

  IF total < 1.00:
    profit_per_unit = 1.00 - total
    arb_opportunity = true
```

**Example:**
```
Market: "Will BTC hit $150k by Dec 2026?"
YES best ask = $0.47
NO  best ask = $0.51
Total cost   = $0.98
Profit       = $0.02 per unit (2.04% return)

Buy 10 YES @ $0.47 = $4.70
Buy 10 NO  @ $0.51 = $5.10
Total invested      = $9.80
Guaranteed payout   = $10.00 (one side MUST win)
Profit              = $0.20 (2.04% risk-free)
```

**Minimum thresholds:**
```
min_profit_pct: 1.5%     (total < $0.985)
min_liquidity:  $5,000   (enough depth to fill)
min_size:       $1.00    (worth the transaction)
max_size:       $5.00    (bankroll limit)
```

### Type 2: Multi-Outcome Event Arbitrage (Neg Risk Markets)

Multi-outcome events (e.g., "Who will win the election?") have N markets.
Exactly ONE outcome wins. Sum of all YES prices should = $1.00.

**Overpriced scenario (sum > $1.00) — SELL all:**
```
For each neg-risk event:
  sum_yes_bids = Σ(highest bid for YES in each market)

  IF sum_yes_bids > 1.00:
    profit_per_unit = sum_yes_bids - 1.00
    ACTION: Sell YES on every outcome at bid price
    RESULT: Exactly one outcome wins, you pay $1.00 but collected >$1.00
```

**Underpriced scenario (sum < $1.00) — BUY all:**
```
For each neg-risk event:
  sum_yes_asks = Σ(lowest ask for YES in each market)

  IF sum_yes_asks < 1.00:
    profit_per_unit = 1.00 - sum_yes_asks
    ACTION: Buy YES on every outcome at ask price
    RESULT: Exactly one outcome wins, pays $1.00, you paid <$1.00
```

**Example:**
```
Event: "Next Fed Chair?"
Market A (Person X): YES ask = $0.35
Market B (Person Y): YES ask = $0.28
Market C (Person Z): YES ask = $0.15
Market D (Other):    YES ask = $0.18
Sum                         = $0.96

Buy 10 units of each:
  Total cost     = $9.60
  Guaranteed win = $10.00 (one MUST resolve YES)
  Profit         = $0.40 (4.17% risk-free)
```

**Minimum thresholds:**
```
min_profit_pct: 1.0%     (wider events = more slippage risk)
min_markets:    2         (at least 2 outcomes)
max_markets:    8         (more markets = more execution risk)
min_liquidity:  $2,000 per market
```

### Type 3: Spread Capture (Market Making Lite)

Not pure arbitrage but low-risk: place both bid and ask on liquid markets
to capture the spread. Only for markets with wide spreads.

**Detection:**
```
For liquid markets (volume_24hr > $50,000):
  spread = best_ask - best_bid
  midpoint = (best_ask + best_bid) / 2

  IF spread > 0.04:  (4 cents = room for profit)
    buy_price  = best_bid + 0.01  (improve bid by 1 tick)
    sell_price = best_ask - 0.01  (improve ask by 1 tick)
    expected_profit = sell_price - buy_price = spread - 0.02

    ACTION: Place bid at buy_price, ask at sell_price
    RISK: One side fills, market moves against us
```

**Risk mitigation:**
```
max_exposure:     $2.00 per market
max_hold_time:    30 min (cancel unfilled after 30 min)
cancel_on_move:   If midpoint moves >2%, cancel both orders
only_high_volume: volume_24hr > $50,000
```

**Minimum thresholds:**
```
min_spread:     $0.04  (4 ticks profit potential)
min_volume_24h: $50,000
max_exposure:   $2.00
```

## Execution Priority

```
Priority 1: Type 1 (Binary arb)     — safest, simplest, guaranteed
Priority 2: Type 2 (Multi-outcome)  — guaranteed but more complex execution
Priority 3: Type 3 (Spread capture) — not guaranteed, only when idle
```

## Profit Expectations

With $50 bankroll, realistic daily targets:

| Strategy | Avg Profit/Trade | Trades/Day | Daily Profit |
|----------|-----------------|------------|--------------|
| Binary Arb | $0.05-$0.30 | 2-5 | $0.10-$1.50 |
| Multi-Outcome Arb | $0.10-$0.50 | 1-3 | $0.10-$1.50 |
| Spread Capture | $0.02-$0.10 | 5-15 | $0.10-$1.50 |
| **Combined** | | | **$0.30-$4.50** |

Conservative target: **$0.50/day = 1% daily = 365% APY**

## Capital Allocation

```yaml
binary_arb_allocation: 40%      # $20 reserved
multi_outcome_allocation: 35%   # $17.50 reserved
spread_capture_allocation: 25%  # $12.50 reserved

# Per-trade limits
max_single_arb: $5.00           # Never more than 10% bankroll
min_single_arb: $1.00           # Not worth gas/effort below this
```

## Scan Cycle

```
Every 60 seconds:
  1. Fetch all active binary markets with orderbook data
  2. Check each for YES_ask + NO_ask < 1.00
  3. Fetch all neg-risk events with multiple outcomes
  4. Check sum(YES_asks) < 1.00 or sum(YES_bids) > 1.00
  5. Check liquid markets for wide spreads
  6. Rank opportunities by profit_pct * liquidity score
  7. Execute top opportunity (one at a time for $50 bankroll)
```

## Risk Controls

```yaml
# Hard limits
max_capital_in_arb: 80%        # Keep 20% as reserve ($10)
max_concurrent_arbs: 3          # Don't spread too thin
max_single_trade: $5.00         # 10% of bankroll
min_profit_after_slippage: 0.5% # Account for execution gap

# Slippage protection
max_slippage: 1%                # Cancel if price moved >1% during execution
use_limit_orders: true          # Never market orders
order_timeout_secs: 30          # Cancel unfilled after 30s

# Multi-outcome specific
max_partial_fill_risk: true     # If only 2 of 4 legs fill, close immediately
leg_fill_timeout_secs: 60       # All legs must fill within 60s

# Circuit breakers
stop_if_capital_below: $10      # Emergency stop
stop_if_daily_loss: $2          # Something wrong, stop trading
max_daily_trades: 50            # Prevent runaway loops
```

## Edge Cases

### Partial Fill Risk (Multi-Outcome)
If buying all 4 outcomes but only 2 fill:
- **Action**: Immediately cancel unfilled orders
- **Risk**: Holding 2 of 4 outcomes = directional bet, not arb
- **Mitigation**: Set tight timeout, sell filled legs at market if needed
- **Loss cap**: Max loss = spread on filled legs ≈ $0.05-$0.20

### Race Condition
Another bot sees the same arb:
- **Action**: Use limit orders at exact ask price, not worse
- **Mitigation**: Speed matters less with GTC orders at good prices
- **Accept**: Some arbs will be taken before us. That's fine.

### Resolution Timing
Arb money is locked until market resolves:
- **Action**: Prefer markets resolving within 30 days
- **Avoid**: Markets resolving in 2027+ (capital locked too long)
- **Track**: Opportunity cost = locked capital can't arb elsewhere

### Neg Risk Conversion
For multi-outcome, sometimes it's cheaper to:
1. Buy NO on one outcome → convert to YES on all others
- **Action**: Compare direct buy vs conversion cost
- **Use when**: One outcome is heavily favored (YES price high = NO price low)

## Integration with Engine

This skill runs as a **background scanner** in the Rust engine.
No Claude API calls needed — pure math.

```
Engine Flow:
1. arb_scanner detects opportunity → adds to arb_opportunities[]
2. OpenClaw reads GET /api/arb-opportunities
3. OpenClaw validates (optional — can auto-execute if pure arb)
4. OpenClaw sends POST /api/trade for each leg
5. Engine executes and tracks all legs as linked arb trade

OR (auto-execute mode for pure arb):
1. arb_scanner detects opportunity with profit > min_threshold
2. Engine executes immediately (no Claude needed)
3. Logs to dashboard + DB
4. OpenClaw is notified via GET /api/state
```

## Performance Tracking

```yaml
metrics:
  arb_opportunities_found: 0     # Total detected
  arb_opportunities_executed: 0  # Actually traded
  arb_total_profit: 0.0          # Cumulative USD
  arb_avg_profit_pct: 0.0        # Average per trade
  arb_win_rate: 100%             # Should be ~100% for Type 1+2
  arb_capital_efficiency: 0.0    # Profit / capital locked per day
  spread_capture_win_rate: 0.0   # Type 3 won't be 100%
  missed_arbs: 0                 # Detected too late
  partial_fill_count: 0          # Multi-leg partial fills
```
