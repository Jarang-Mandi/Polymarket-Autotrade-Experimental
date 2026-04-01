---
name: api-cost-optimizer
description: Engine penghemat API Claude Opus 4.6 — setiap token adalah uang, setiap API call mengurangi modal trading. Agent WAJIB mengoptimalkan penggunaan otak untuk memaksimalkan profit per dollar API cost.
metadata: {"openclaw":{"always":true}}
---

# API Cost Optimizer — Brain Efficiency Engine

## Purpose
Otak agent (Claude Opus 4.6) BUKAN GRATIS. Setiap kali agent "berpikir", itu menghabiskan uang.
**API cost adalah BIAYA OPERASIONAL — sama pentingnya dengan trading fees.**
Jika API cost > trading profit, agent RUGI meskipun win rate tinggi.

**PRINSIP: Berpikir SECERDAS mungkin dengan token SESEDIKIT mungkin.**

---

## Claude Opus 4.6 — Cost Structure

| Token Type | Harga | Catatan |
|---|---|---|
| **Input (fresh)** | $5 / MTok | Setiap prompt baru |
| **Cache Write** | $6.25 / MTok | Pertama kali cache skill/system prompt |
| **Cache Read** | **$0.50 / MTok** | **10x LEBIH MURAH — GUNAKAN INI MAKSIMAL** |
| **Output** | **$25 / MTok** | **5x LEBIH MAHAL dari input — HEMAT OUTPUT** |

### Implikasi Kritis
```
1 token output = 5 token input (dari segi biaya)
1 token cache read = 0.1 token input (dari segi biaya)

ARTINYA:
- OUTPUT harus SINGKAT, PADAT, tanpa basa-basi
- System prompt/skills harus di-CACHE, bukan di-load ulang
- Verbose reasoning = MEMBAKAR UANG
```

### Estimasi Biaya per Operasi
| Operasi | Est. Tokens | Est. Cost | Frekuensi |
|---|---|---|---|
| Full market scan + analysis | ~8K in + ~2K out | ~$0.09 | 4-6x/hari |
| Trade decision (1 market) | ~4K in + ~1K out | ~$0.045 | 3-5x/hari |
| Quick price check | ~2K in + ~0.5K out | ~$0.02 | 10-20x/hari |
| Daily reflection | ~5K in + ~2K out | ~$0.075 | 1x/hari |
| Weekly deep analysis | ~10K in + ~5K out | ~$0.175 | 1x/minggu |
| Emergency assessment | ~4K in + ~1K out | ~$0.045 | Rare |

### Daily API Budget Estimation
```
Conservative agent: ~$0.50-1.00/hari
Active agent: ~$1.00-2.50/hari
Aggressive agent: ~$2.00-5.00/hari

DENGAN $50 STARTING CAPITAL:
- $1/hari API cost = 2% of capital PER HARI just for thinking
- Agent HARUS profit >2%/hari MINIMAL untuk cover biaya otak
- Jika profit hanya 1%/hari ($0.50) tapi API cost $1/hari → NET LOSS!
```

---

## WAJIB: Prompt Caching Strategy

### Apa yang HARUS di-Cache (System Prompt)
Semua 28+ skills di-load sebagai system prompt → di-cache otomatis oleh Anthropic.
- Cache write pertama: $6.25/MTok (sedikit lebih mahal)
- Semua call berikutnya: $0.50/MTok (10x lebih murah!)
- Cache duration: 5 menit default, bisa 1 jam

### Cache Optimization Rules
```
1. JANGAN ubah system prompt antar-call jika tidak perlu
   → Mengubah system prompt = cache MISS = bayar fresh input lagi
2. Keep skill content STABIL — perubahan skill = cache invalidation
3. Batch pertanyaan dalam 1 call jika bisa, bukan multiple calls
4. Jaga interval antar-call < 5 menit agar cache tidak expire
   ATAU set cache TTL = 1 jam jika frekuensi rendah
```

---

## Output Token Optimization (KRITIS — $25/MTok!)

### Rules untuk SEMUA Output
```
1. JANGAN output JSON yang verbose jika tidak perlu
2. Gunakan format COMPACT:
   BAD:  {"market_id": "abc123", "decision": "TRADE", "reasoning": "Based on my analysis of multiple factors including..."}
   GOOD: TRADE abc123 YES $3 edge=7% EV=0.12 conf=0.75

3. JANGAN ulangi informasi yang sudah ada di input
4. JANGAN jelaskan reasoning panjang lebar — gunakan shorthand
5. Internal monologue / chain-of-thought = MAHAL
   → Batasi reasoning ke max 200 tokens per keputusan
6. Gunakan abbreviasi standar:
   MKT=market, POS=position, SZ=size, ED=edge
   EV=expected value, CF=confidence, DD=drawdown
   SK=skip, TR=trade, RD=reduce, SL=stop loss
```

### Output Format Tiers

**Tier 1: MICRO (< 50 tokens) — untuk routine checks**
```
STATUS OK | CAP=$52.30 | POS=2 | DD=1.2% | HUNGER=PECKISH
```

**Tier 2: SHORT (50-200 tokens) — untuk trade decisions**
```
DECISION: TRADE
MKT: will-btc-hit-100k-by-march | YES@0.35
EDGE: 12% | EV: 0.15 | CF: 0.78 | SZ: $3(5.7%)
EXIT: TP@0.55 SL@0.25 TIME:7d
REASON: Strong momentum + ETF inflow data + top traders aligned
```

**Tier 3: MEDIUM (200-500 tokens) — untuk daily reflection**
```
DAILY: 3 trades | W2 L1 | PnL: +$1.20 (+2.3%)
BEST: btc-100k YES win +$0.80 (edge was real)
WORST: election-poll NO loss -$0.30 (news shifted post-entry)
HUNT: 5 scanned, 3 traded, 2 skipped (correct skips)
PASSIVE: NO (active day)
TOMORROW: Focus crypto + sports, avoid politics (low edge)
```

**Tier 4: LONG (500-1500 tokens) — HANYA untuk weekly review**
```
Hanya digunakan 1x per minggu untuk deep analysis.
Weekly review adalah satu-satunya operasi yang boleh verbose.
```

**Tier 5: EMERGENCY (unlimited) — HANYA saat krisis**
```
Drawdown >20%, system error, atau kondisi yang membutuhkan deep reasoning.
Ini adalah satu-satunya waktu "harga tidak masalah" — survival first.
```

---

## Decision Complexity Tiering

TIDAK SEMUA keputusan butuh full pipeline 10-step.

### Quick Decision (1-2 API calls)
Gunakan untuk:
- Market yang sudah familiar (pernah trade sukses)
- Edge yang jelas (>15%)
- Routine monitoring / price checks
- Skip decisions (jelas tidak ada edge)

```
Cost: ~$0.02-0.04 per decision
Trigger: Familiar market + clear signal
```

### Standard Decision (2-3 API calls)
Gunakan untuk:
- Market baru tapi di kategori familiar
- Edge moderate (7-15%)
- Normal trading decisions

```
Cost: ~$0.05-0.10 per decision
Trigger: New market + moderate signal
```

### Deep Decision (3-5 API calls)
Gunakan untuk:
- Market baru di kategori baru
- Big position (>5% capital)
- Conflicting signals
- Recovery trades setelah losing streak

```
Cost: ~$0.10-0.25 per decision
Trigger: High stakes + uncertainty
```

### JANGAN gunakan Deep Decision untuk:
- Trade kecil ($1-2)
- Market yang sudah jelas (edge <3% = obvious skip)
- Routine monitoring
- Checking yang bisa di-batch

---

## Batching Strategy

### Batch These Operations
```
1. Market scanning: Scan SEMUA market sekaligus, bukan satu per satu
   → 1 API call untuk 50 markets, bukan 50 calls untuk 50 markets

2. Position monitoring: Check SEMUA posisi aktif dalam 1 call
   → "Check all my positions" bukan "check position 1" "check position 2"

3. News assessment: Kumpulkan semua news dulu, analisa sekaligus
   → Bukan fetch 1 news → analyze → fetch next → analyze

4. Daily reflection: 1 comprehensive call, bukan multiple mini-reflections
```

### DO NOT Batch These (Need Separate Calls)
```
1. Trade execution: Setiap trade butuh fresh analysis (market bisa berubah)
2. Emergency assessment: Speed matters more than cost
3. Post-loss analysis: Needs focused attention
```

---

## API Budget Management

### Daily Budget Rules
```
api_budget_daily = MAX(capital * 0.02, $0.30)

Contoh dengan $50 capital:
- Budget: $1.00/hari
- Jika capital tumbuh ke $100: Budget = $2.00/hari
- Jika capital turun ke $30: Budget = $0.60/hari
- MINIMUM budget: $0.30/hari (harus bisa operasi minimal)
```

### Budget Allocation
| Operation Type | % Budget | At $1/day |
|---|---|---|
| Market scanning | 25% | $0.25 |
| Trade decisions | 40% | $0.40 |
| Monitoring | 15% | $0.15 |
| Reflection/Learning | 15% | $0.15 |
| Emergency reserve | 5% | $0.05 |

### Cost Tracking
```
Setiap API call, track:
- tokens_in (input)
- tokens_out (output)
- tokens_cached (cache hits)
- estimated_cost = (tokens_in * 5 + tokens_out * 25 + tokens_cached * 0.5) / 1_000_000
- running_daily_cost += estimated_cost

IF running_daily_cost > 80% of daily_budget:
    ALERT: "Approaching API budget limit"
    → Switch to MICRO output only
    → Skip non-essential operations
    → Batch remaining work

IF running_daily_cost > 100% of daily_budget:
    → ONLY emergency operations and trade executions
    → No scanning, no reflection until tomorrow
```

---

## Token Reduction Techniques

### 1. Abbreviated Prompts
```
INSTEAD OF: "Please analyze the following market and provide a detailed assessment of whether I should trade it, including probability estimation, edge calculation, risk assessment, and position sizing recommendation."

USE: "Analyze: [market_id] → Prob/Edge/Risk/Size → TRADE or SKIP"
```

### 2. Structured Shorthand
```
Context shorthand untuk setiap call:
CAP:[amount] POS:[count] DD:[%] STATE:[state] HUNGER:[level]

Contoh: CAP:52.30 POS:2 DD:1.2 STATE:GROWTH HUNGER:PECKISH
→ Ini menggantikan paragraf penjelasan tentang current state
```

### 3. Reference, Don't Repeat
```
INSTEAD OF: Copying full market data into every call
USE: "Re: market abc123 (data from last scan)" 
→ Agent ingat dari context sebelumnya, tidak perlu re-state
```

### 4. Progressive Detail
```
Step 1: Quick screen (5 detik thinking) → SKIP or MAYBE
Step 2: IF MAYBE → Standard analysis → SKIP or TRADE
Step 3: IF TRADE and big position → Deep verification

Kebanyakan market di-SKIP di Step 1 → hemat 80% tokens
```

---

## Cost-Profit Ratio Monitoring

### The Golden Rule
```
WAJIB maintain:
  trading_profit / api_cost > 2.0 (minimum)
  trading_profit / api_cost > 5.0 (target)
  trading_profit / api_cost > 10.0 (excellent)

Artinya: Setiap $1 yang dihabiskan untuk "berpikir" harus menghasilkan minimal $2 profit.
```

### Weekly Cost Efficiency Review
```
total_api_cost_this_week = sum(daily_costs)
total_trading_profit = sum(trade_profits) - sum(trade_losses) - sum(fees)
net_profit = total_trading_profit - total_api_cost_this_week
cost_efficiency = total_trading_profit / total_api_cost_this_week

IF cost_efficiency < 1.0:
    CRITICAL: "Agent is LOSING MONEY on thinking costs alone!"
    → Drastically reduce scan frequency
    → Use MICRO output only
    → Only trade highest-conviction opportunities

IF cost_efficiency < 2.0:
    WARNING: "Thinking costs eating too much profit"
    → Reduce to essential operations only
    → Batch more aggressively
    → Skip daily reflection, do weekly only

IF cost_efficiency > 5.0:
    GOOD: "Efficient thinking"
    → Maintain current approach
```

---

## Emergency Cost Reduction Mode

Trigger: API cost > 50% of daily profit OR approaching budget limit

Actions:
```
1. Switch ALL output to Tier 1 MICRO format
2. Stop market scanning — only monitor existing positions
3. No new trades unless edge > 15% (obvious)
4. Skip all non-essential reflection
5. Batch everything into 1-2 calls per day
6. Estimate tokens BEFORE each call — cancel if too expensive
```

---

## Output Format (This Skill's Own Output)

```
API_DAILY: cost=$0.45 budget=$1.00 remaining=$0.55 calls=12
EFFICIENCY: profit=$1.80 ratio=4.0x status=GOOD
CACHE: hit_rate=85% savings=$0.32
ACTION: Continue normal operations
```
