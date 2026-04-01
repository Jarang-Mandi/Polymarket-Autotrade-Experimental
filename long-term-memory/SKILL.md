---
name: long-term-memory
description: Sistem memori persisten yang menyimpan semua pengalaman trading, pola market, dan lessons learned untuk mencegah kesalahan berulang dan meningkatkan keputusan.
metadata: {"openclaw":{"always":true}}
---

# Long-Term Memory — Agent Experience Database

## Purpose
Agent WAJIB mengingat setiap trade, setiap kesalahan, setiap market condition.
Tanpa memory, agent akan mengulangi kesalahan yang sama berulang kali.
Memory adalah kunci survival jangka panjang.

---

## Memory Categories

### 1. Trade Memory (Per Trade Record)
```json
{
  "trade_id": "",
  "timestamp": "ISO-8601",
  "market_id": "",
  "market_question": "",
  "category": "sports | politics | crypto | entertainment | other",
  "side": "YES | NO",
  "entry_price": 0.00,
  "exit_price": 0.00,
  "size_usd": 0.00,
  "model_probability": 0.00,
  "market_probability_at_entry": 0.00,
  "confidence_at_entry": 0.00,
  "expected_value_at_entry": 0.00,
  "actual_outcome": "WIN | LOSS",
  "pnl": 0.00,
  "reasoning_at_entry": "",
  "reasoning_at_exit": "",
  "news_at_time": [],
  "top_trader_alignment": 0.00,
  "post_trade_reflection": "",
  "mistake_flag": false,
  "mistake_type": ""
}
```

### 2. Market Pattern Memory
```json
{
  "pattern_id": "",
  "category": "",
  "pattern_description": "",
  "times_observed": 0,
  "times_traded": 0,
  "win_rate": 0.00,
  "average_edge": 0.00,
  "average_pnl": 0.00,
  "last_observed": "ISO-8601",
  "confidence_in_pattern": 0.00,
  "example_trades": []
}
```

### 3. News Impact Memory
```json
{
  "news_type": "",
  "category": "",
  "estimated_impact_at_time": 0.00,
  "actual_impact": 0.00,
  "impact_estimation_error": 0.00,
  "times_occurred": 0,
  "reliability_score": 0.00
}
```

### 4. Market Category Performance
```json
{
  "category": "",
  "total_trades": 0,
  "win_rate": 0.00,
  "total_pnl": 0.00,
  "average_edge": 0.00,
  "best_subcategory": "",
  "worst_subcategory": "",
  "calibration_error": 0.00,
  "last_updated": "ISO-8601"
}
```

### 5. Mistake Ledger
```json
{
  "mistake_type": "REVENGE_TRADE | OVERSIZE | LOW_EV | FOMO | CORRELATED | TIMING | OVERCONFIDENCE",
  "times_occurred": 0,
  "total_cost": 0.00,
  "last_occurred": "ISO-8601",
  "preventive_action": ""
}
```

### 6. Strategy Performance Memory
```json
{
  "strategy_type": "",
  "description": "",
  "trades_count": 0,
  "win_rate": 0.00,
  "sharpe_ratio": 0.00,
  "max_drawdown": 0.00,
  "current_status": "active | paused | abandoned",
  "effectiveness_score": 0.00
}
```

---

## Memory Retrieval Logic

### Before Any Trade Decision

```
1. SEARCH: Similar markets in history
   - Same category
   - Similar probability range
   - Similar news conditions
   
2. RETRIEVE: Pattern memory
   - Has this pattern been seen before?
   - What was the outcome?
   
3. CHECK: Mistake ledger
   - Am I about to repeat a known mistake?
   
4. ASSESS: Category performance
   - How am I doing in this category?
   - Am I calibrated for this type of market?
```

### Similarity Scoring
```
similarity_score = 
  (category_match * 0.3) +
  (probability_range_similarity * 0.2) +
  (news_condition_similarity * 0.2) +
  (market_structure_similarity * 0.15) +
  (time_to_resolution_similarity * 0.15)

IF similarity_score > 0.7: HIGH relevance
IF similarity_score 0.4-0.7: MODERATE relevance
IF similarity_score < 0.4: LOW relevance → less weight
```

---

## Memory-Informed Adjustments

### Positive Memory Match
```
IF similar_trades have win_rate > 70%:
    confidence_boost = +0.05
    reasoning: "Historically strong pattern"
```

### Negative Memory Match
```
IF similar_trades have win_rate < 35%:
    confidence_reduction = -0.10
    IF win_rate < 25%:
        recommendation = "SKIP — historically bad setup"
```

### Mistake Prevention
```
IF current_trade matches a mistake_pattern from ledger:
    ALERT: "Warning — this resembles [mistake_type]"
    Extra scrutiny required before proceeding
```

---

## Memory Consolidation (Weekly)

Setiap minggu, memory harus di-consolidate:

1. **Merge similar patterns**: Gabungkan patterns yang mirip
2. **Update statistics**: Refresh win rates, averages
3. **Decay old data**: Data > 90 hari dapat weight yang lebih rendah
4. **Highlight insights**: Top 3 most profitable patterns, Top 3 mistake types
5. **Archive**: Move resolved/irrelevant data ke cold storage

---

## Memory Size Management

Untuk efisiensi:
- Keep last 500 trades in active memory
- Summarize older data into aggregate statistics
- Keep ALL mistake ledger entries (never delete mistakes)
- Pattern memory: max 100 active patterns
- Consolidate weekly

---

## Output Format (Memory Query Response)

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
MEMORY: query=[context] found=X matches
TOP: [trade_id] sim=X.XX [WIN|LOSS] | [trade_id] sim=X.XX [WIN|LOSS]
HIST: WR=XX% avgPnL=$X.XX for similar
PATTERN: [pattern or NONE] | MISTAKE_WARN: [warning or NONE]
CATEGORY: WR=XX% cal_err=X.XX
REC: [PROCEED|CAUTION|AVOID] conf_adj=+/-X.XX
```
