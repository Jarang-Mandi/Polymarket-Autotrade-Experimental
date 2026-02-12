---
name: position_exit_engine
description: Engine khusus untuk mengelola exit dari posisi — profit taking, cut loss, time-based exit, dan information-based exit untuk binary Polymarket outcomes.
metadata: {"openclaw":{"always":true}}
---

# Position Exit Engine — Know When to Get Out

## Purpose
Entry is only half the battle. KAPAN dan BAGAIMANA keluar dari posisi sama pentingnya.
Di Polymarket binary markets, exit strategy berbeda dari traditional markets.

---

## Understanding Binary Market Exits

### Key Differences from Traditional Markets
```
Binary market: outcome is either $1.00 (win) or $0.00 (lose)
Traditional: price can go anywhere

Implications:
1. If you're RIGHT, holding to resolution = MAX PROFIT ($1.00 per share)
2. If you're WRONG, holding to resolution = MAX LOSS ($0.00 per share)
3. You CAN sell before resolution at current market price
4. Early exit = lock in profit/loss at market price, not waiting for resolution
```

### When to EXIT EARLY vs HOLD TO RESOLUTION
```
HOLD TO RESOLUTION if:
  - Your conviction hasn't changed
  - New information supports your position
  - Time to resolution is short (<48h)
  - Position is small (within risk limits)
  - You'd still enter this trade at current price

EXIT EARLY if:
  - New information changes your probability estimate significantly
  - Your updated EV is now negative
  - Position size has become too large relative to portfolio
  - You need the capital for a better opportunity
  - Market regime changed to defensive
  - Your agent state degraded (e.g., from GROWTH to DEFENSIVE)
```

---

## Exit Strategies

### Strategy 1: Information-Based Exit
```
TRIGGER: News or data changes probability estimate

IF |posterior_probability - prior_probability| > 0.10:
    → REASSESS position immediately

IF updated_EV < 0 (position now has negative expected value):
    → EXIT IMMEDIATELY if liquidity available
    → Place limit order at current mid-price
    → Accept small loss to avoid larger loss

IF updated_EV still positive but reduced:
    → HOLD unless better opportunity available
    → Reduce position if possible and cheap
```

### Strategy 2: Price-Based Exit (Profit Taking)
```
TRIGGER: Market price moves in your favor

IF position_pnl_pct > 30% of position value:
    EVALUATE: Is more upside likely?
    IF prob_model still supports > edge: HOLD
    IF price has moved to fair value: TAKE PROFIT
    
IF position_pnl_pct > 50%:
    → Strongly consider taking profit
    → Remaining upside is diminishing
    → Capital can be redeployed

RULE: Never turn a winning position into a losing one.
IF position was +20% now back to +5%:
    → Consider exit to lock in small profit
    → Rather than risk it going negative
```

### Strategy 3: Time-Based Exit
```
As resolution approaches:

IF time_to_resolution < 48 hours AND position is winning:
    → HOLD to resolution (max payout)
    
IF time_to_resolution < 48 hours AND position is losing:
    → EVALUATE: any chance of reversal?
    → IF <30% model probability of winning: EXIT (salvage remaining value)
    → IF 30-50% probability: small hope, HOLD if loss is small
    → IF >50% probability: HOLD (you still think you're right)

IF time_to_resolution < 4 hours:
    → Price is nearly at 0 or 1
    → Likely too late to exit meaningfully
    → HOLD to resolution unless can salvage >20% of position
```

### Strategy 4: Capital Redeployment Exit
```
TRIGGER: Better opportunity found

IF new_opportunity_EV > current_position_remaining_EV * 1.5:
    → Consider exiting current position
    → Redeploy capital to better opportunity
    → Factor in exit costs (spread, fees) in calculation

Rules:
  - Don't exit winning position just for marginal improvement
  - Only exit if new opportunity is SIGNIFICANTLY better
  - Consider partial exit (sell half)
```

### Strategy 5: Portfolio-Forced Exit
```
TRIGGER: Risk limits require reducing exposure

Priority for closure:
1. Positions with lowest remaining EV
2. Positions with highest correlation to other holdings
3. Positions closest to resolution (fastest capital return)
4. Positions with worst current PnL trajectory

Method:
  - Place limit orders at current mid-price
  - Accept up to 2% worse than mid to ensure fill
  - If urgent: market order with slippage cap
```

### Strategy 6: Stop-Loss Exit
```
TRIGGER: Position loss exceeds threshold

For $50 bankroll, position stops:

IF position_unrealized_loss > 50% of position cost:
    → EVALUATE: Has anything fundamental changed?
    → IF yes: EXIT immediately
    → IF no: binary market can recover — HOLD if conviction intact

IF position_unrealized_loss > 70% of position cost:
    → Remaining value is small ($0.30-0.90 on a $3 position)
    → EVALUATE: Is it worth selling for $0.30 or holding for small chance of $3?
    → Usually HOLD at this point (sunk cost, but option value remains)

IMPORTANT: In binary markets, positions trading at $0.10 can still win ($1.00)
Don't panic sell at extreme lows if fundamental thesis hasn't changed.
```

---

## Exit Decision Matrix

```
                    Position Winning           Position Losing
                  ┌──────────────────────┬──────────────────────┐
Resolution >7d   │ Hold or take partial  │ Exit if EV negative  │
                  │ profit if >30% gain  │ Hold if EV positive  │
                  ├──────────────────────┼──────────────────────┤
Resolution 2-7d  │ Hold (max payout      │ Evaluate info change │
                  │ approaching)          │ Exit if no hope      │
                  ├──────────────────────┼──────────────────────┤
Resolution <2d   │ HOLD to resolution    │ Exit if can salvage  │
                  │ (max payout imminent) │ >20%, else hold      │
                  └──────────────────────┴──────────────────────┘
```

---

## Position Monitoring Frequency

| Time to Resolution | Check Frequency | Action Level |
|-------------------|----------------|--------------|
| >7 days | Every 4 hours | Low — unless news |
| 3-7 days | Every 2 hours | Medium |
| 1-3 days | Every hour | High |
| <24 hours | Every 30 minutes | Very High |
| <4 hours | Every 10 minutes | Critical |

---

## Exit Execution Procedure

```
1. DECIDE to exit (based on strategy above)
2. CHECK orderbook — is there liquidity to exit?
3. CALCULATE exit cost (spread + fees)
4. VERIFY exit is rational (cost vs benefit)
5. PLACE exit order:
   - Limit order at mid-price (preferred)
   - If urgent: limit order 1-2% worse than mid
   - If very urgent: market order with 3% max slippage
6. CONFIRM execution
7. UPDATE wallet-tracker
8. LOG to activity-log with reasoning
9. STORE in long-term-memory
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
EXIT: [market_id] [HOLD|EXIT_FULL|EXIT_PARTIAL|MONITOR]
STRATEGY: [info|price|time|redeploy|portfolio|stop_loss] urgency=[IMM|SOON|CONV]
POS: [YES|NO]@X.XX now=X.XX uPnL=+/-$X.XX(X.X%) EV=X.XX Xd left
PLAN: exit@X.XX cost=$X.XX salvage=$X.XX [LIMIT|MARKET] slip<X.X%
REASON: [1-line max 20 words]
```

Jika HOLD:
```
HOLD: [market_id] @X.XX now=X.XX uPnL=+/-$X.XX EV=X.XX — on track
```
