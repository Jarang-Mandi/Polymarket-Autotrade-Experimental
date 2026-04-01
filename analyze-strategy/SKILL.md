---
name: analyze-strategy
description: Engine analisis multi-layer yang menggabungkan data market, top trader patterns, news intel, dan historical memory untuk menghasilkan sinyal strategis high-quality.
metadata: {"openclaw":{"always":true}}
---

# Strategy Analysis Engine

## Purpose
Menggabungkan semua sumber data menjadi sinyal strategis yang actionable.
Ini adalah OTAK ANALISIS — menerima data mentah dan menghasilkan insight yang terstruktur.

---

## Input Sources

1. `fetch-market-data` → Market prices, orderbook, liquidity
2. `fetch-top-traders` → Smart money positioning & patterns
3. `sports-news-intel` → News & information impact
4. `long-term-memory` → Historical patterns & lessons
5. `market-regime` → Current market conditions
6. `bayesian-update` → Updated probability estimates

---

## Analysis Pipeline

### Layer 1: Statistical Base Rate Analysis
```
Untuk setiap market:
1. Identifikasi base rate dari historical data
   - Contoh: "Incumbent presidents win re-election 60% of time"
   - Contoh: "Home team wins in NBA ~58% of time"
   - Contoh: "BTC year-end predictions miss >70% of time"

2. Compare base rate vs current market price
   base_rate_edge = base_rate - market_implied_probability
```

### Layer 2: Information Advantage Assessment
```
Evaluate: Apakah agent punya info advantage?

information_sources = [
  news_not_yet_priced_in,
  statistical_insight_market_ignores,
  pattern_from_similar_events,
  top_trader_signal,
  timing_advantage
]

info_advantage_score = weighted_sum(information_sources)
IF info_advantage_score < min_threshold: NO TRADE
```

### Layer 3: Top Trader Pattern Analysis
```
FROM fetch-top-traders data:

1. Identifikasi consensus: Are top traders aligned?
   IF 60%+ top traders on same side → strong signal
   IF split 50/50 → no signal from this source

2. Identifikasi smart money moves:
   IF whale ($10K+) enters position → moderate signal
   IF multiple whales enter → strong signal
   IF whale exits → warning signal

3. Identifikasi timing patterns:
   Top traders cenderung masuk pada timing tertentu
   Early entry vs late entry patterns
```

### Layer 4: News Impact Analysis
```
FROM sports-news-intel (expanded to all categories):

Rate each news item:
- Relevance to market: 0-1
- Reliability of source: 0-1
- Already priced in?: estimate 0-1
- Direction: positive / negative / neutral
- Magnitude: low / medium / high

Unprice_news = news yang belum fully priced → this is edge
```

### Layer 5: Market Microstructure Analysis
```
1. Orderbook imbalance:
   IF bid_depth >> ask_depth → buying pressure (bullish)
   IF ask_depth >> bid_depth → selling pressure (bearish)
   
2. Volume profile:
   IF volume increasing + price moving → real move
   IF volume decreasing + price moving → fake/weak move
   
3. Spread dynamics:
   IF spread tightening → liquidity improving, market confident
   IF spread widening → uncertainty increasing
```

### Layer 6: Timing Analysis
```
time_to_resolution = event end_date - now

IF time_to_resolution > 14 days:
    - More uncertainty, prices can swing
    - Need strong edge to enter early
    - Can get better entry if patient
    
IF time_to_resolution 3-14 days:
    - Sweet spot for entry
    - Enough time for information edge to play out
    - Good risk/reward
    
IF time_to_resolution 1-3 days:
    - High conviction needed
    - Price converging to outcome
    - Most information already priced in
    - Only trade with very clear edge
    
IF time_to_resolution < 24 hours:
    - Very risky — almost gambling
    - Only if you have DEFINITIVE info
    - Or if market clearly mispriced (>15% edge)
```

---

## Signal Synthesis

Combine all layers into final signal:

```
final_score = 
  (base_rate_edge * 0.15) +
  (info_advantage * 0.25) +
  (top_trader_signal * 0.15) +
  (news_impact * 0.20) +
  (microstructure_signal * 0.10) +
  (timing_score * 0.15)

IF final_score > strong_threshold → STRONG BUY/SELL signal
IF final_score > moderate_threshold → MODERATE signal
IF final_score > weak_threshold → WEAK signal (consider skip)
IF final_score < weak_threshold → NO signal → SKIP
```

---

## Signal Quality Classification

| Signal Quality | Criteria | Recommended Action |
|---------------|----------|-------------------|
| A+ (Excellent) | Edge >12%, 4+ layers agree, high confidence | Trade with increased size |
| A (Good) | Edge >7%, 3+ layers agree, good confidence | Trade with normal size |
| B (Decent) | Edge >5%, 2+ layers agree, moderate confidence | Trade with minimum size |
| C (Marginal) | Edge >3%, 1-2 layers agree | SKIP (not worth risk for $50) |
| D (Poor) | Edge <3% or conflicting layers | SKIP |
| F (Bad) | Negative EV or too uncertain | DEFINITELY SKIP |

**Untuk $50 bankroll**: Hanya trade signal grade A+ dan A. Grade B hanya di GROWTH mode.

---

## Cross-Validation Check

Sebelum finalisasi signal:
```
1. Query long-term-memory: Similar setups historically — how did they perform?
2. Check correlation with open positions
3. Verify market regime allows trading
4. Confirm no active cooldowns
5. Sanity check: "Would I bet $3 of my own money on this?"
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
ANALYSIS: [market_id] [category] grade=[A+|A|B|C|D|F]
LAYERS: base=X.X% info=X.X tt=X.X news=X.X micro=X.X time=X.X
PROB: model=XX% mkt=XX% edge=X.X% EV=X.XX conf=X.XX
SIDE: [YES|NO|SKIP] size=$X.XX
VALIDATION: mem=[OK|WARN] corr=[OK|FLAG] regime=[OK|CAUTION] cool=[OK|WAIT]
RISK: [warnings or CLEAR]
REASON: [1-line max 30 words]
```
