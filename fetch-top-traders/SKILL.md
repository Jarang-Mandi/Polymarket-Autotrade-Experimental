---
name: fetch_top_traders
description: Mengambil dan menganalisa data top traders dari Polymarket untuk smart money intelligence dan pattern detection.
metadata: {"openclaw":{"always":true}}
---

# Fetch Top Traders — Smart Money Intelligence

## Purpose
Top traders di Polymarket punya track record terbukti. Data mereka bukan untuk di-copy, tapi sebagai SIGNAL TAMBAHAN untuk diperhitungkan dalam analisis.

---

## Data to Fetch

### Leaderboard Data
```json
{
  "trader_id": "",
  "address": "",
  "rank": 0,
  "total_pnl": 0.00,
  "win_rate": 0.00,
  "total_trades": 0,
  "total_volume": 0.00,
  "active_positions": [],
  "avg_position_size": 0.00,
  "best_category": "",
  "roi_30d": 0.00,
  "roi_90d": 0.00
}
```

### Per-Trader Position Data
```json
{
  "trader_id": "",
  "market_id": "",
  "side": "YES | NO",
  "size": 0.00,
  "entry_price": 0.00,
  "entry_timestamp": "ISO-8601",
  "current_pnl": 0.00,
  "is_new_position": true,
  "position_change_24h": 0.00
}
```

---

## Fetch Schedule

| Data Type | Frequency | Purpose |
|-----------|-----------|---------|
| Top 100 leaderboard | Every 6 hours | Track who's performing best |
| Position changes | Every 2 hours | Detect new moves |
| Whale alerts (>$5K) | Every 30 minutes | Time-sensitive signals |

---

## Smart Money Analysis Framework

### Signal 1: Consensus Detection
```
FOR each active market:
    count_top_traders_YES = count(top_100 with YES position)
    count_top_traders_NO = count(top_100 with NO position)
    total_top_traders = count_YES + count_NO
    
    IF total_top_traders > 5:
        consensus_ratio = max(count_YES, count_NO) / total_top_traders
        IF consensus_ratio > 0.70: STRONG_CONSENSUS
        IF consensus_ratio > 0.60: MODERATE_CONSENSUS
        ELSE: NO_CONSENSUS
```

### Signal 2: Smart Money Flow
```
smart_money_flow_24h = 
    sum(new_positions_YES_by_top_traders) - sum(new_positions_NO_by_top_traders)

IF smart_money_flow > $50K: STRONG bullish signal
IF smart_money_flow < -$50K: STRONG bearish signal
IF |smart_money_flow| < $10K: NO signal
```

### Signal 3: Whale Movement Detection
```
IF single trader enters >$10K position:
    whale_alert = True
    Check trader's historical win rate
    IF whale_win_rate > 65%: HIGH quality signal
    IF whale_win_rate < 50%: IGNORE (bad whale)
```

### Signal 4: Expert Category Matching
```
Identify traders who are specialists:
    - Sports specialists (high win_rate in sports)
    - Politics specialists
    - Crypto specialists

IF specialist enters position in their specialty:
    signal_quality += 0.2
```

### Signal 5: Position Exit Detection
```
IF multiple top traders EXIT position:
    → Warning signal — they know something?
    → Check if any news explains exit
    → Consider reducing own position if aligned

IF top trader REVERSES position (was YES, now NO):
    → STRONG signal — significant new information likely
```

---

## Anti-Copy Trading Rules

**CRITICAL**: Agent TIDAK BOLEH blindly copy top traders.

Alasan:
1. Top traders punya bankroll besar — risk tolerance berbeda
2. Mereka masuk di harga berbeda — timing matters
3. Mereka mungkin hedging (posisi ini bagian dari strategy besar)
4. Past performance ≠ future results
5. Agent punya edge sendiri yang harus digunakan

**Yang boleh dilakukan:**
- Gunakan sebagai salah satu signal (weight 10-15%)
- Gunakan untuk CONFIRM suspicion yang sudah ada
- Gunakan untuk DETECT markets worth investigating
- Gunakan untuk WARNING jika kamu di opposite side dari consensus
- Gunakan specialist data untuk category-specific insights

---

## Top Trader Quality Scoring

```
trader_quality_score = 
    (win_rate * 0.30) +
    (roi_30d / max_roi * 0.25) +
    (total_trades_normalized * 0.15) +
    (consistency_score * 0.20) +
    (recency_bonus * 0.10)

Only use signals from traders with quality_score > 0.5
```

---

## Market Discovery via Top Traders

Salah satu value terbesar dari tracking top traders:
```
NEW_POSITION_SCAN:
    Setiap 2 jam, check posisi baru top traders
    IF >3 top traders masuk market yang BELUM di-watchlist agent:
        → ADD to analysis pipeline
        → This market might have edge
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
TRADERS: top10=[X updated] improved=[X] best_WR=[trader XX%]

SIGNALS:
1. [market_id] traders=X consensus=[STRONG_YES|STRONG_NO|SPLIT] flow=$XK strength=X.X [HIGH|MED|LOW]
2. ...
(max 5)

WHALE: [alerts or NONE]
NEW_MKTS: [discoveries from top traders or NONE]
EXIT_WARN: [markets top traders are exiting or NONE]
QUALITY: [GOOD|PARTIAL|STALE]
```
