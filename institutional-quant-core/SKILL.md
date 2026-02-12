---
name: polymarket_meta_knowledge
description: Pengetahuan mendalam tentang mekanisme platform Polymarket — CLOB, settlement, fees, token mechanics, dan platform-specific strategies.
metadata: {"openclaw":{"always":true}}
---

# Polymarket Meta Knowledge — Platform Mastery

## Purpose
Agent HARUS memahami Polymarket secara mendalam — bagaimana platform bekerja, aturan settlement, token mechanics, dan quirks yang bisa di-exploit atau harus dihindari.

---

## Platform Fundamentals

### What is Polymarket?
- Prediction market pada Polygon blockchain
- Binary outcomes: YES or NO tokens
- CLOB (Central Limit Order Book) — bukan AMM
- Settlement in USDC
- Resolution by UMA oracle atau designated resolution source

### Token Mechanics
```
Setiap market punya 2 tokens: YES dan NO
YES token price + NO token price ≈ $1.00

Membeli YES token di harga $0.60:
  → Jika outcome YES: receive $1.00 → profit $0.40
  → Jika outcome NO: receive $0.00 → loss $0.60
  
Membeli NO token di harga $0.40:
  → Jika outcome NO: receive $1.00 → profit $0.60
  → Jika outcome YES: receive $0.00 → loss $0.40

IMPLIKASI: Buy YES di harga rendah = high payout, high risk
           Buy YES di harga tinggi = low payout, low risk
```

### Price = Implied Probability
```
YES price $0.65 = market thinks 65% probability of YES outcome
NO price $0.35 = market thinks 35% probability of NO outcome

IF model thinks probability is 75% but market price is $0.65:
  → Edge = 10%
  → EV = (0.75 * 0.35) - (0.25 * 0.65) = 0.2625 - 0.1625 = +$0.10 per $1
```

---

## CLOB Mechanics

### Order Types on Polymarket
```
LIMIT ORDER:
  - You set the price
  - May not fill immediately
  - Better price execution
  - Lower/zero maker fees
  → PREFERRED for $50 bankroll

MARKET ORDER:
  - Fill immediately at best available price
  - Pay taker fees (higher)
  - Subject to slippage
  → Only use when speed critical
```

### Orderbook Understanding
```
BID side: Buyers willing to pay (sorted high to low)
ASK side: Sellers willing to sell (sorted low to high)
SPREAD: gap between best bid and best ask
MID PRICE: (best_bid + best_ask) / 2

For $50 agent:
  → Trade sizes are small ($1-5)
  → Slippage should be minimal
  → But ALWAYS check orderbook before trading
  → Wide spread = DANGER for small positions
```

### Spread as Indicator
```
Tight spread (<2%): Active, liquid market — good
Normal spread (2-5%): Acceptable
Wide spread (5-8%): Caution — slippage risk
Very wide (>8%): AVOID — illiquid, expensive to trade
```

---

## Fee Structure

```
Maker fee (limit orders): ~0% 
Taker fee (market orders): ~1-2%
Polygon gas: ~$0.01-0.05 per transaction

For $50 bankroll optimization:
  → ALWAYS use limit orders when possible
  → $3 trade with 2% taker fee = $0.06 fee
  → Over 20 trades that's $1.20 — 2.4% of bankroll!
  → LIMIT orders save significant money over time
```

---

## Settlement & Resolution

### Resolution Process
```
1. Event occurs (game played, election held, etc.)
2. Resolution source confirms outcome
3. Market is resolved: YES=1.00, NO=0.00
4. Winners can redeem tokens for $1.00 each
5. Losers get $0.00

IMPORTANT: Resolution can take hours or days after event!
Agent should NOT panic if resolution is delayed.
```

### Edge Cases to Watch
```
1. AMBIGUOUS RESOLUTION:
   - Some markets have unclear resolution criteria
   - READ the resolution source carefully before trading
   - If ambiguous → SKIP (not worth the risk)

2. EARLY RESOLUTION:
   - Some markets resolve before end_date
   - Track resolution source actively

3. DISPUTED RESOLUTION:
   - UMA oracle can be challenged
   - Market stays in pending state
   - Capital locked until resolution
   → Factor this into liquidity planning

4. N/A RESOLUTION:
   - Event cancelled or can't be determined
   - Usually resolved at 50/50 or voided
   → Avoid markets where this is likely
```

---

## Platform-Specific Strategies for $50 Bankroll

### Strategy 1: "Resolution Sniping"
```
Target markets resolving in 2-24 hours
By this point, information is clearest
If you find a mispricing this close to resolution:
  → Win rate should be very high
  → Hold period very short
  → Quick capital turnover

Risk: Market might already be efficient this close to resolution
```

### Strategy 2: "News Edge Trading"
```
Find news that hasn't been priced in yet
Act quickly but verify first
Small size — you're not sure how much is priced in
Key: SPEED of information processing

Best for: Sports (injury news), Politics (poll releases)
```

### Strategy 3: "Extreme Value Hunting"
```
Look for markets where price is at extreme (<10¢ or >90¢)
Occasionally these are mispriced
BUT usually they're correct — extreme prices exist for a reason

Only trade if you have SPECIFIC reason to disagree:
  → Data the market is ignoring
  → Calculation error by the market
  → New development not yet reflected
```

### Strategy 4: "Liquidity Providing via Limit Orders"
```
Place limit orders at slightly better than current best bid/ask
You earn the spread if both sides fill
Requires: patience, understanding of fair value
Risk: adverse selection (you get filled because price moving against you)

For $50: Only do this in markets where you have an opinion
```

### Strategy 5: "Multi-Market Arbitrage"
```
Sometimes related markets are inconsistent:
  → "Team A wins Game 1" priced at 60%
  → "Team A wins Series" priced at 80%
  → But winning Game 1 makes winning series much more likely
  
Look for logical inconsistencies across related markets
Very rare but can be profitable
```

---

## Polymarket API Quick Reference

```
REST API Base: https://clob.polymarket.com

Key Endpoints:
GET  /markets                    — List all markets
GET  /market/{condition_id}      — Market details
GET  /book?token_id={id}         — Orderbook
GET  /price?token_id={id}        — Current price
GET  /trades?market={id}         — Recent trades  
POST /order                      — Place order
DELETE /order/{id}               — Cancel order
GET  /orders?market={id}         — Your open orders

Gamma Markets API: https://gamma-api.polymarket.com
GET /markets                     — Market metadata
GET /events                      — Event groupings

Authentication: API key + signature (via Rust backend)
```

---

## Common Pitfalls to Avoid

1. **Trading illiquid markets**: Spread will eat your edge
2. **Ignoring resolution rules**: Read carefully — edge cases can cause losses
3. **Market orders on small positions**: Fee disproportionately large
4. **Holding too many positions**: Capital gets locked, can't seize new opportunities
5. **Ignoring gas costs**: Small but accumulate
6. **Not checking orderbook depth**: Your order might move the price
7. **Trading just before resolution**: High risk, information already priced in
8. **Ambiguous markets**: Skip if resolution criteria unclear
9. **Duplicate exposure**: Two markets on same underlying event = correlated loss risk
10. **Not accounting for capital lockup**: Money in positions ≠ available money

---

## Output: Market Evaluation

```json
{
  "market_id": "",
  "platform_assessment": {
    "liquidity_adequate": true,
    "spread_acceptable": true,
    "resolution_clear": true,
    "resolution_source_reliable": true,
    "fees_acceptable": true,
    "orderbook_depth_adequate": true,
    "time_to_resolution_optimal": true,
    "no_ambiguity_risk": true
  },
  "platform_warnings": [],
  "recommended_order_type": "LIMIT | MARKET",
  "recommended_entry_price": 0.00,
  "estimated_fees": 0.00,
  "estimated_slippage": 0.00,
  "platform_score": 0.00,
  "tradeable": true
}
```
