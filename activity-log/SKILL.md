---
name: activity_log
description: Sistem logging komprehensif yang mencatat setiap keputusan, action, dan reasoning agent untuk audit trail, debugging, dan improvement.
metadata: {"openclaw":{"always":true}}
---

# Activity Log — Complete Audit Trail

## Purpose
SETIAP keputusan agent harus ter-record. Log ini adalah memori operasional dan audit trail.
Jika sesuatu salah, log harus bisa menjelaskan MENGAPA agent membuat keputusan itu.

---

## Log Types

### 1. TRADE_SIGNAL
Ketika strategy engine menghasilkan signal:
```json
{
  "log_type": "TRADE_SIGNAL",
  "timestamp": "ISO-8601",
  "market_id": "",
  "signal": "BUY | SELL | SKIP",
  "model_probability": 0.00,
  "expected_value": 0.00,
  "confidence": 0.00,
  "reasoning": "",
  "data_sources_used": []
}
```

### 2. RISK_DECISION
Ketika risk engine approve/reject trade:
```json
{
  "log_type": "RISK_DECISION",
  "timestamp": "ISO-8601",
  "trade_id": "",
  "decision": "APPROVED | REDUCED | REJECTED",
  "reason": "",
  "gates_passed": [],
  "gates_failed": [],
  "capital_at_time": 0.00,
  "exposure_at_time": 0.00
}
```

### 3. TRADE_EXECUTION
Ketika trade dieksekusi:
```json
{
  "log_type": "TRADE_EXECUTION",
  "timestamp": "ISO-8601",
  "execution_id": "",
  "market_id": "",
  "side": "",
  "size": 0.00,
  "price": 0.00,
  "order_type": "",
  "status": "FILLED | PARTIAL | REJECTED",
  "slippage": 0.00,
  "fee": 0.00
}
```

### 4. TRADE_RESOLVED
Ketika posisi resolved:
```json
{
  "log_type": "TRADE_RESOLVED",
  "timestamp": "ISO-8601",
  "trade_id": "",
  "market_id": "",
  "outcome": "WIN | LOSS",
  "pnl": 0.00,
  "entry_price": 0.00,
  "resolution_price": 0.00,
  "model_was_correct": true,
  "post_mortem": ""
}
```

### 5. STATE_CHANGE
Ketika agent state berubah:
```json
{
  "log_type": "STATE_CHANGE",
  "timestamp": "ISO-8601",
  "old_state": "",
  "new_state": "",
  "trigger": "",
  "capital_at_change": 0.00
}
```

### 6. BAYESIAN_UPDATE
Ketika probabilitas di-update:
```json
{
  "log_type": "BAYESIAN_UPDATE",
  "timestamp": "ISO-8601",
  "market_id": "",
  "prior": 0.00,
  "posterior": 0.00,
  "trigger": "",
  "evidence": ""
}
```

### 7. NEWS_DETECTED
Ketika news penting terdeteksi:
```json
{
  "log_type": "NEWS_DETECTED",
  "timestamp": "ISO-8601",
  "headline": "",
  "category": "",
  "impact_type": "",
  "affected_markets": [],
  "action_taken": ""
}
```

### 8. REGIME_CHANGE
Ketika market regime berubah:
```json
{
  "log_type": "REGIME_CHANGE",
  "timestamp": "ISO-8601",
  "old_regime": "",
  "new_regime": "",
  "risk_multiplier_change": 0.00
}
```

### 9. REFLECTION
Ketika agent melakukan self-reflection:
```json
{
  "log_type": "REFLECTION",
  "timestamp": "ISO-8601",
  "reflection_type": "per_trade | daily | weekly | crisis",
  "key_findings": [],
  "parameter_changes": {},
  "action_plan": []
}
```

### 10. ERROR
Ketika ada error:
```json
{
  "log_type": "ERROR",
  "timestamp": "ISO-8601",
  "error_type": "",
  "error_message": "",
  "context": "",
  "recovery_action": ""
}
```

### 11. SKIP_DECISION
Ketika agent memutuskan SKIP (SAMA PENTING dengan trade):
```json
{
  "log_type": "SKIP_DECISION",
  "timestamp": "ISO-8601",
  "market_id": "",
  "reason": "",
  "model_probability": 0.00,
  "expected_value": 0.00,
  "was_skip_correct": null
}
```

---

## Log Storage

```
Primary: Structured JSON files per day
  logs/2026-02-12.json
  logs/2026-02-13.json

Index: Summary file updated daily
  logs/index.json → quick access to daily stats

Archive: Weekly consolidation
  logs/archive/week-2026-07.json
```

---

## Log Query Capabilities

Agent harus bisa query logs untuk:
1. "Show me all trades in last 7 days"
2. "Show me all losses this week"  
3. "Show me all SKIP decisions that were correct"
4. "Show me all state changes"
5. "Show me trade history for market category X"
6. "Show me all errors in last 24 hours"

---

## Daily Summary Auto-Generation

Setiap hari pukul 23:55 UTC, generate daily summary:

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
DAILY_LOG: [YYYY-MM-DD]
TRADES: exec=X skip=X resolved=X | W=X L=X
PnL: gross=+/-$X.XX fees=$X.XX API=$X.XX net=+/-$X.XX
CAP: start=$XX.XX end=$XX.XX (+/-X.X%)
STATE: [start_state] → [end_state] | REGIME: [conditions]
EVENTS: [key events or NONE]
MISTAKES: [detected or NONE]
LESSONS: [1-2 lessons or NONE]
```

---

## Logging Rules

1. **LOG EVERYTHING** — setiap keputusan, termasuk keputusan untuk NOT trade
2. Timestamps WAJIB ISO-8601 UTC
3. Reasoning WAJIB included — "what was I thinking?"
4. NEVER delete logs — even embarrassing mistakes
5. Log BEFORE action (intention) dan AFTER action (result)
6. Include capital state di setiap log entry yang melibatkan money
