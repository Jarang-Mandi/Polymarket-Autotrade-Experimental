---
name: fetch-market-data
description: Mengambil dan menormalisasi data market dari Polymarket CLOB API termasuk harga, orderbook, volume, liquidity metrics, dan event metadata.
metadata: {"openclaw":{"always":true}}
---

# Fetch Market Data — Polymarket Data Pipeline

## Purpose
Mengambil semua data yang dibutuhkan agent untuk membuat keputusan trading. Data adalah darah kehidupan agent — tanpa data yang baik, tidak ada keputusan yang baik.

---

## Data Points to Fetch

### 1. Market Overview
```json
{
  "market_id": "",
  "question": "",
  "category": "sports | politics | crypto | entertainment | science | other",
  "subcategory": "",
  "description": "",
  "resolution_source": "",
  "end_date": "ISO-8601",
  "time_to_resolution_hours": 0,
  "is_active": true,
  "is_resolved": false,
  "resolution_value": null
}
```

### 2. Price Data
```json
{
  "market_id": "",
  "token_YES_price": 0.00,
  "token_NO_price": 0.00,
  "implied_probability_YES": 0.00,
  "implied_probability_NO": 0.00,
  "mid_price_YES": 0.00,
  "price_24h_change": 0.00,
  "price_7d_change": 0.00,
  "all_time_high": 0.00,
  "all_time_low": 0.00
}
```

### 3. Orderbook Data
```json
{
  "market_id": "",
  "best_bid_YES": 0.00,
  "best_ask_YES": 0.00,
  "spread_YES": 0.00,
  "spread_pct": 0.00,
  "bid_depth_5pct": 0.00,
  "ask_depth_5pct": 0.00,
  "total_liquidity": 0.00,
  "orderbook_imbalance": 0.00
}
```

### 4. Volume & Liquidity Metrics
```json
{
  "market_id": "",
  "volume_24h": 0.00,
  "volume_7d": 0.00,
  "volume_total": 0.00,
  "unique_traders": 0,
  "liquidity_score": "HIGH | MEDIUM | LOW | VERY_LOW",
  "avg_trade_size": 0.00,
  "large_trades_24h": 0
}
```

### 5. Market Health Assessment
```json
{
  "market_id": "",
  "is_tradeable": true,
  "health_score": 0.00,
  "flags": [],
  "minimum_recommended_size": 0.00,
  "estimated_slippage_at_1usd": 0.00,
  "estimated_slippage_at_5usd": 0.00
}
```

---

## Fetch Schedule

| Action | Frequency | Purpose |
|--------|-----------|---------|
| Full market scan | Every 4 hours | Discover new opportunities |
| Active market update | Every 30 minutes | Track positions & candidates |
| Orderbook refresh | Before each trade | Get current execution data |
| Resolution check | Every hour | Track approaching resolutions |

---

## Market Filtering Pipeline

Dari semua markets yang di-fetch, filter:

### Stage 1: Basic Filter
```
REMOVE if:
  - is_resolved == true
  - is_active == false
  - time_to_resolution_hours < 1 (too close to resolution)
  - time_to_resolution_hours > 720 (>30 days — too far)
```

### Stage 2: Liquidity Filter
```
REMOVE if:
  - volume_24h < $1,000
  - spread_pct > 8%
  - total_liquidity < $5,000
  - liquidity_score == "VERY_LOW"
```

### Stage 3: Quality Filter
```
REMOVE if:
  - unique_traders < 10
  - resolution_source unclear
  - market description ambiguous
```

### Stage 4: Opportunity Scoring
```
FOR each remaining market:
    opportunity_score = 
        (volume_score * 0.25) +
        (spread_score * 0.25) +
        (liquidity_score * 0.20) +
        (time_to_resolution_score * 0.15) +
        (category_agent_performance * 0.15)
    
    RANK by opportunity_score DESC
```

---

## API Endpoints Reference (Polymarket CLOB)

```
Base URL: https://clob.polymarket.com

GET /markets              — List all markets
GET /market/{id}          — Market details
GET /book?token_id={id}   — Orderbook
GET /prices?token_ids=... — Current prices
GET /trades?market={id}   — Recent trades
```

---

## Data Normalization Rules

1. All prices normalized to 0.00-1.00 range
2. All USD values in USDC
3. Timestamps in ISO-8601 UTC
4. Missing data flagged, not assumed
5. Stale data (>5 min old) must be refreshed before trading

---

## Error Handling

```
IF API timeout: retry 2x with 5s backoff
IF API error 429 (rate limit): wait 60s, reduce frequency
IF API returns stale data: flag and don't trade with it
IF API down: use last known data + increase confidence threshold
```

---

## Combined Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
DATA: fetched=XXX filtered=XX quality=[GOOD|PARTIAL|STALE]

TOP MARKETS:
1. [market_id] [cat] @X.XX% spread=X.X% vol=$XK liq=[score] Xh left score=X.XX
2. ...
(max 10 entries)

MY_POSITIONS: [market_ids with current data or NONE]
RESOLVING_SOON: [market_ids within 24h or NONE]
```
