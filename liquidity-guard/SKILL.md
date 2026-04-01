---
name: liquidity-guard
description: Menganalisa likuiditas market sebelum trading untuk mencegah slippage, stuck positions, dan execution problems yang bisa menghancurkan micro-bankroll.
metadata: {"openclaw":{"always":true}}
---

# Liquidity Guard — Don't Trade What You Can't Exit

## Purpose
Dengan $50, setiap cent matters. Liquidity guard memastikan agent HANYA trade di markets yang cukup liquid untuk masuk DAN keluar tanpa excessive cost.

---

## The Liquidity Problem for Small Bankrolls

```
Contoh buruk:
  Market: "Will X happen?" — spread 12%
  Buy YES at $0.56 (best ask)
  Immediately sell YES at $0.44 (best bid)
  → Instant loss of 21% of position value!
  → On a $3 trade: -$0.63 lost immediately
  
Contoh baik:
  Market: "Will Y happen?" — spread 2%
  Buy YES at $0.51 (best ask)  
  Immediately sell YES at $0.49 (best bid)
  → Only 3.9% loss if you need to exit
  → On a $3 trade: -$0.12 manageable
```

---

## Liquidity Assessment Checklist

### Check 1: Spread Analysis
```
spread_pct = (best_ask - best_bid) / mid_price * 100

TRADEABLE:
  spread < 2%: EXCELLENT — trade freely
  spread 2-4%: GOOD — trade with limit orders
  spread 4-6%: ACCEPTABLE — limit orders only, small size
  spread 6-8%: POOR — only if edge is very large (>10%)
  spread > 8%: DO NOT TRADE
```

### Check 2: Orderbook Depth
```
# How much can you trade without moving the price?

depth_at_1pct = total_value_within_1%_of_best_price
depth_at_3pct = total_value_within_3%_of_best_price
depth_at_5pct = total_value_within_5%_of_best_price

For $50 bankroll (trades of $1-5):
  IF depth_at_1pct > $50: GOOD (your trade won't move price)
  IF depth_at_1pct > $20: ACCEPTABLE (minimal impact)
  IF depth_at_1pct < $10: CAUTION (your trade IS the market)
  IF depth_at_1pct < $5: DO NOT TRADE (you'll move the price)
```

### Check 3: Volume Assessment
```
volume_24h: Recent trading activity

  > $10,000/day: ACTIVE market
  $5,000-10,000: MODERATE activity
  $1,000-5,000: LOW activity — caution
  < $1,000: DEAD market — avoid

avg_trade_size = volume_24h / trade_count_24h
  IF avg_trade_size < $5: Many small traders → good for our size
  IF avg_trade_size > $100: Few large traders → potential adverse selection
```

### Check 4: Orderbook Imbalance
```
bid_total = sum(all_bid_orders)
ask_total = sum(all_ask_orders)
imbalance = (bid_total - ask_total) / (bid_total + ask_total)

  imbalance > 0.3: Heavy buying interest (might be good for YES)
  imbalance < -0.3: Heavy selling interest (might be good for NO)
  |imbalance| < 0.1: Balanced — neutral signal
  |imbalance| > 0.6: Very one-sided — be cautious of adverse selection
```

### Check 5: Slippage Estimation
```
FOR proposed trade of $X:

Walk the orderbook:
  remaining_to_fill = X
  weighted_avg_price = 0
  slippage_cost = 0
  
  FOR each level in orderbook:
    fillable = min(remaining_to_fill, level_quantity)
    weighted_avg_price += fillable * level_price
    remaining_to_fill -= fillable
    IF remaining_to_fill == 0: break
  
  weighted_avg_price /= X
  slippage = |weighted_avg_price - best_price| / best_price

  IF slippage > 3%: REJECT trade
  IF slippage > 1%: USE limit order (no market order)
  IF slippage < 0.5%: Free to use market order if needed
```

### Check 6: Exit Liquidity
```
IMPORTANT: Can you EXIT this position later?

Check bid side (if you're buying YES, you'll sell on bid side):
  bid_depth = total bid volume within 5% of current price
  
  IF bid_depth < 2 * position_size: EXIT WILL BE DIFFICULT
  → Reduce position size or avoid entirely
  
  IF bid_depth > 10 * position_size: EXIT WILL BE EASY
  → Proceed normally
```

---

## Liquidity Score (Composite)

```
liquidity_score = 
    (spread_score * 0.30) +
    (depth_score * 0.25) +
    (volume_score * 0.20) +
    (exit_liquidity_score * 0.25)

Rating:
  >= 0.8: EXCELLENT — trade freely
  0.6-0.8: GOOD — standard precautions
  0.4-0.6: ADEQUATE — limit orders only, reduce size
  0.2-0.4: POOR — only with very high edge, tiny size
  < 0.2: ILLIQUID — DO NOT TRADE
```

---

## Liquidity-Adjusted Position Sizing

```
# Reduce position size for less liquid markets:

liquidity_adjusted_size = base_size * liquidity_multiplier

WHERE:
  liquidity_multiplier = 
    1.0 if score >= 0.8
    0.7 if score 0.6-0.8
    0.5 if score 0.4-0.6
    0.3 if score 0.2-0.4 (with high edge only)
    0.0 if score < 0.2 (no trade)
```

---

## Markets to Auto-Blacklist

```
Automatically blacklist markets that:
1. Have not had a trade in 48+ hours
2. Have orderbook with < $100 total
3. Have spread > 15%
4. Have been flagged as potentially manipulated
5. Resolution source is unclear or unreliable
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
LIQ: [market_id] rating=[EXCELLENT|GOOD|ADEQUATE|POOR|ILLIQUID]
SPREAD: X.X%[grade] | DEPTH: bid=$XK ask=$XK | VOL: $XK/24h[grade]
IMBAL: X.XX | SLIP@$X: ~X.X% | EXIT_LIQ: [OK|WARN]
SCORE: X.XX/1.00
REC: tradeable=[YES|NO] max=$X.XX [LIMIT|MARKET] adj=X.Xx
WARN: [warnings or CLEAR]
```
