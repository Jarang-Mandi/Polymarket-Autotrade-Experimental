---
name: execute_trade
description: Mengirim perintah eksekusi ke Polymarket CLOB API dengan slippage protection, order management, dan execution verification.
metadata: {"openclaw":{"always":true}}
---

# Execute Trade — Polymarket CLOB Execution Engine

## Purpose
Mengeksekusi trade di Polymarket Central Limit Order Book (CLOB) dengan presisi, safety checks, dan cost optimization.

---

## Pre-Execution Checklist (ALL must pass)

1. ✅ Trade sudah diapprove oleh risk-allocation
2. ✅ Wallet balance mencukupi (amount + gas buffer)
3. ✅ Market masih active & belum resolved
4. ✅ Current price masih dalam range yang acceptable (< 2% slippage dari signal price)
5. ✅ Liquidity adequate di orderbook
6. ✅ Tidak ada FREEZE state aktif
7. ✅ Daily loss limit belum tercapai

---

## Order Types

### 1. LIMIT ORDER (Default — Preferred)
- Place order di harga yang kita mau
- Lebih murah (maker fee lebih rendah)
- Risiko: mungkin tidak terisi

```json
{
  "order_type": "LIMIT",
  "price": 0.00,
  "size": 0.00,
  "side": "BUY | SELL",
  "token_id": "",
  "expiration": "ISO-8601"
}
```

### 2. MARKET ORDER (Urgent situations only)
- Langsung terisi tapi bayar taker fee
- Hanya gunakan jika harga bergerak cepat dan edge masih ada
- WAJIB set max_slippage

```json
{
  "order_type": "MARKET",
  "size": 0.00,
  "side": "BUY | SELL",
  "token_id": "",
  "max_slippage": 0.03
}
```

### 3. GTC (Good Till Cancel)
- Order tetap aktif sampai terisi atau dibatalkan
- Berguna untuk "sniping" harga yang diinginkan

---

## Slippage Protection

```
max_acceptable_price = signal_price * (1 + max_slippage)
min_acceptable_price = signal_price * (1 - max_slippage)

IF current_best_price > max_acceptable_price:
    ABORT execution
    REASON: "Slippage exceeds threshold"

IF current_best_price < min_acceptable_price:
    LOG: "Price improved — proceed"
```

**Slippage limits by state:**
- GROWTH: max 3% slippage
- CAUTIOUS: max 2% slippage
- DEFENSIVE: max 1.5% slippage
- SURVIVAL: max 1% slippage

---

## Execution Flow

```
1. Receive approved trade plan from risk-allocation
2. Fetch current orderbook snapshot
3. Verify price is still within acceptable range
4. Verify wallet balance sufficient
5. Calculate optimal order placement:
   - If spread tight: limit order at mid-price
   - If spread wide: limit order at better-than-mid
   - If urgent: market order with slippage cap
6. Submit order to Polymarket CLOB API via Rust backend
7. Wait for confirmation (max 30 seconds)
8. Verify execution:
   - Order filled → log + update portfolio
   - Order partially filled → evaluate remainder
   - Order rejected → log error + alert
   - Timeout → cancel + retry once
9. Update wallet-tracker with new balance
10. Log to activity-log
```

---

## Fee Awareness ($50 Bankroll Context)

Polymarket fees:
- Maker fee: ~0% (placing limit orders)
- Taker fee: ~1-2% (market orders)
- Polygon gas: ~$0.01-0.05 per transaction

**Strategy**: SELALU prefer LIMIT orders untuk minimize fees.
Dengan $3-5 trades, bahkan 2% fee = $0.06-0.10 per trade.
Over 50 trades = $3-5 in fees → 6-10% of bankroll!

---

## Position Size Validation

Sebelum execute, WAJIB validate:
```
IF trade_size < $0.50:
    SKIP — too small, fees not worth it
IF trade_size > max_allowed_by_state:
    REDUCE to max_allowed
IF trade_size > available_balance * 0.90:
    SKIP — must keep buffer
```

---

## Error Handling

| Error | Action |
|-------|--------|
| Insufficient balance | ABORT, log, alert |
| Market resolved | ABORT, update market status |
| API timeout | Retry 1x after 5 seconds |
| Slippage exceeded | ABORT, wait for better price |
| Order rejected | Log reason, evaluate re-entry |
| Network error | Wait 30s, retry 1x |
| Unknown error | ABORT, log full error, alert |

---

## Post-Execution Output

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
EXEC: [FILLED|PARTIAL|REJECTED] [market_id] [BUY|SELL]
PRICE: req=$X.XX exec=$X.XX slip=X.X%
SIZE: $X.XX → X.XX shares | FEE: $X.XX | GAS: $X.XX
BAL: $X.XX | POS: X open
```

**Gunakan JSON hanya jika data perlu di-parse programmatically oleh tool lain.**

---

## Cancel & Amend Logic

Agent dapat cancel open orders jika:
- Harga market bergerak > 5% dari saat order ditempatkan
- News baru mengubah probabilitas
- Risk exposure berubah karena posisi lain
- Agent state berubah ke lebih defensif
