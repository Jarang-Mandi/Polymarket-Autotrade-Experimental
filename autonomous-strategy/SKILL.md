---
name: autonomous-strategy
description: Otak utama agent — menghasilkan strategi trading mandiri berdasarkan multi-source data, probabilistic reasoning, dan survival-first logic untuk Polymarket.
metadata: {"openclaw":{"always":true}}
---

# Autonomous Strategy Engine

## Purpose
Engine utama yang menghasilkan keputusan trading secara independen. Tidak bergantung pada siapapun — agent berpikir sendiri berdasarkan data, probabilitas, dan pengalaman.

**PRINSIP INTI: Engine ini bukan hanya mesin pertahanan — ini adalah MESIN PEMBURU.**
Agent AKTIF mencari profit karena DIAM = MATI PERLAHAN. Setiap cycle tanpa trade yang menguntungkan adalah oksigen yang terbuang. Engine ini harus menyeimbangkan KELAPARAN (agresif mencari profit) dengan DISIPLIN (tidak trade sembarangan).

### Hunger-Adjusted Behavior
| Hunger Level | Scan Frequency | Min Edge Required | Max Positions | Aggression |
|---|---|---|---|---|
| FED (profit >1% hari ini) | Normal (4h) | 5% | 3 | Low — protect gains |
| PECKISH (0% hari ini) | Increased (2h) | 4% | 4 | Medium — actively seeking |
| HUNGRY (no trade 12h+) | High (1h) | 3.5% | 5 | High — lower threshold slightly |
| STARVING (no profit 48h+) | Aggressive (30min) | 3% | 5 | Very High — expand categories |
| DESPERATE (drawdown >15%) | Critical scan only | 8% | 2 | DISCIPLINED — quality over quantity |

---

## Decision Pipeline (Sequential + Hunger-Aware + Cost-Aware)

```
-1. BUDGET  → Cek remaining API budget hari ini → set thinking depth
 0. HUNGER  → Cek hunger level & oxygen clock → set aggression level
 1. OBSERVE → Kumpulkan data (BATCH semua market dalam 1 call)
 2. ANALYZE → Hitung probabilitas — depth sesuai cost tier
 3. COMPARE → Bandingkan model vs market price
 4. EVALUATE→ Hitung EV & edge (hunger + state adjusted thresholds)
 5. FILTER  → Cek risk, liquidity, correlation, API COST GATE
 6. DECIDE  → TRADE / REDUCE / SKIP (compact output format)
 7. SIZE    → Tentukan position size optimal
 8. EXECUTE → Kirim ke execution engine
 9. MONITOR → Track ALL positions dalam 1 call (batched)
10. LEARN   → Record outcome — SHORT format ke memory
```

### Step -1: API Budget Gate (WAJIB — sebelum semua)
```
api_budget_remaining = daily_budget - spent_today
api_budget_pct = api_budget_remaining / daily_budget

IF api_budget_pct > 50%:
    thinking_depth = "FULL" → all 10 steps, standard output
IF api_budget_pct 20-50%:
    thinking_depth = "EFFICIENT" → skip verbose analysis, SHORT output only
IF api_budget_pct < 20%:
    thinking_depth = "MICRO" → only trade if edge > 15%, MICRO output only
IF api_budget_pct < 5%:
    thinking_depth = "FROZEN" → emergency & execution only, 0 scanning
```

**OUTPUT FORMAT WAJIB SESUAI THINKING DEPTH:**
- FULL → Tier 2 SHORT format (max 200 tokens output)
- EFFICIENT → Tier 2 SHORT format (max 150 tokens output)
- MICRO → Tier 1 MICRO format (max 50 tokens output)
- FROZEN → Tier 1 MICRO format (max 30 tokens output)

### Step 0: Hunger Check (NEW — WAJIB sebelum setiap cycle)
```
hunger_level = get_from(profit-hunger skill)
oxygen_status = get_from(profit-hunger skill)
time_since_last_profit = now() - last_profitable_trade_time
today_pnl = sum(today_resolved_trades)

IF oxygen_status == "SUFFOCATING" OR "CRITICAL":
    mode = "EMERGENCY_HUNT" → expand scan, lower edge threshold (min 3%)
IF hunger_level >= "HUNGRY":
    mode = "ACTIVE_HUNT" → increase scan frequency, widen categories
IF hunger_level == "FED":
    mode = "SELECTIVE_HUNT" → be picky, protect today's gains
```

**PENTING: Hunger TIDAK PERNAH menurunkan standar di bawah 3% edge. Lapar bukan alasan untuk trade bodoh.**

---

## Step 1: Market Selection (OBSERVE)

Agent harus memilih market yang:
- Punya volume > $5,000 (cukup likuid)
- Spread < 5% (bid-ask tidak terlalu lebar)
- Resolution dalam 1-30 hari (sweet spot)
- Agent punya informational edge (news, data, atau pattern)
- Bukan 50/50 coin flip tanpa edge

Kategori yang boleh di-trade:
- Sports (soccer, basketball, MMA, etc.)
- Politics (elections, policy decisions)
- Crypto (price predictions, ETF approvals)
- Entertainment (awards, releases)
- Science/Tech (launches, discoveries)
- Current Events (verified facts-based)

**AVOID**: Markets yang terlalu random/unpredictable tanpa data basis.

---

## Step 2: Probability Estimation (ANALYZE)

Agent membuat estimasi independen berdasarkan:

1. **Base Rate Analysis**: Statistik historis untuk event serupa
2. **Current Information**: News terbaru, data real-time
3. **Expert Signals**: Posisi top traders (bukan sebagai copy, tapi sebagai signal)
4. **Market Implied**: Harga market saat ini = implied probability
5. **Bayesian Update**: Integrasikan semua sumber ke posterior

```
P_model = Bayesian_Posterior(base_rate, news_impact, expert_signal, market_data)
```

---

## Step 3: Edge Detection (COMPARE)

```
Market_Implied_Probability = market_price / 100
Edge = P_model - Market_Implied_Probability
```

Klasifikasi edge:
- Edge < 3%: **NO EDGE** → SKIP
- Edge 3-7%: **SMALL EDGE** → Trade with minimum size
- Edge 7-12%: **MODERATE EDGE** → Trade with normal size
- Edge 12-20%: **STRONG EDGE** → Trade with increased size (max cap applies)
- Edge > 20%: **SUSPICIOUS** → Double-check reasoning, possible mispricing ATAU model error

---

## Step 4: Expected Value Calculation (EVALUATE)

Untuk binary market (YES/NO):

```
EV_YES = (P_model * (1 - price_yes)) - ((1 - P_model) * price_yes)
EV_NO  = ((1 - P_model) * (1 - price_no)) - (P_model * price_no)

Best_side = argmax(EV_YES, EV_NO)
```

**Minimum EV threshold berdasarkan agent state × hunger level:**

| Agent State | FED (EV min) | HUNGRY (EV min) | STARVING (EV min) |
|---|---|---|---|
| GROWTH | 0.05 | 0.04 | 0.03 (floor) |
| CAUTIOUS | 0.08 | 0.06 | 0.05 |
| DEFENSIVE | 0.12 | 0.10 | 0.08 |
| SURVIVAL | 0.15 | 0.15 | 0.15 (no hunger discount) |
| CRITICAL | 0.25 | 0.25 | 0.25 (no hunger discount) |

**CATATAN**: Di SURVIVAL dan CRITICAL mode, hunger TIDAK menurunkan EV threshold.
Lapar boleh mendorong agent mencari LEBIH BANYAK market, tapi TIDAK menurunkan standar saat capital rendah.

---

## Step 5: Multi-Layer Filter (FILTER)

Semua filter harus PASS sebelum trade dieksekusi:

| Filter | Check | Fail Action |
|--------|-------|-------------|
| Risk Budget | Total exposure < max allowed? | SKIP |
| Position Limit | Under max concurrent positions? | SKIP |
| Liquidity | Spread < 5%? Volume adequate? | SKIP |
| Correlation | Not correlated with existing positions? | REDUCE or SKIP |
| **API Cost** | **Trade profit potential > 2x API cost for this decision?** | **SKIP if not worth thinking about** |
| Confidence | Above state-dependent threshold? | SKIP |
| Regime | Market regime allows trading? | Adjust size or SKIP |
| Memory | Similar past trades — positive history? | Adjust confidence |
| Cooldown | No recent loss requiring cooldown? | WAIT |

---

## Step 6: Decision Matrix

```
IF (EV > threshold AND all_filters_pass AND confidence > threshold):
    IF edge == "STRONG":
        decision = "TRADE" with increased_size
    ELIF edge == "MODERATE":
        decision = "TRADE" with normal_size
    ELIF edge == "SMALL":
        decision = "TRADE" with minimum_size
    ELSE:
        decision = "SKIP"
ELSE:
    decision = "SKIP"
    reason = first_failed_condition
```

---

## Step 7: Conviction Score

Hitung conviction score (0-100) berdasarkan:
- EV strength: 30%
- Model confidence: 25%
- Information quality: 20%
- Historical accuracy in similar markets: 15%
- Top trader alignment: 10%

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
STRATEGY: [TRADE|REDUCE|SKIP] [market_id] [YES|NO]
PROB: model=XX% mkt=XX% edge=X.X% EV=X.XX
CONV: XX/100 conf=X.XX | SZ: $X.XX (X.X%)
RR: X.Xx | FILTERS: X/X passed
EXIT: TP=$X.XX SL=$X.XX time=Xd
REASON: [1-line reasoning max 30 words]
```

Jika SKIP:
```
SKIP: [market_id] — [reason] | opp_cost=$X.XX
```

---

## Survival Override (ABSOLUTE)

Kondisi yang SELALU trigger SKIP, regardless of EV:
- Capital < 20% initial → FREEZE
- 3+ consecutive losses today → STOP for today
- Total daily loss > 8% of capital → STOP for today
- Agent state == CRITICAL → Only ultra-high conviction trades
- Correlation exposure > 15% → No new correlated positions
- Less than $3 available capital → PERMANENT FREEZE

---

## Profit-Hunger Override (PROACTIVE)

Kondisi yang MENDORONG agent untuk BERTINDAK (bukan freeze):

| Trigger | Action | Rationale |
|---|---|---|
| 0 trades dalam 24 jam & market open | Force full market scan + report opportunities | Diam = mati perlahan |
| 3+ hari tanpa profit | Enter ACTIVE_HUNT mode, expand ke semua kategori | Kelaparan harus mendorong perburuan |
| Win streak 3+ trades | Slight size increase (max +20%) selama edge masih ada | Momentum = confidence = profit |
| Capital growth >10% minggu ini | Evaluate apakah sizing masih optimal atau terlalu konservatif | Jangan biarkan uang menganggur |
| Cash reserve >60% portfolio | ALERT: terlalu banyak uang idle, cari deployment | Uang idle = peluang terbuang |

### Opportunity Cost Awareness
Setiap SKIP harus di-log dengan estimasi:
```
skip_cost = estimated_EV * would_be_position_size
running_opportunity_cost += skip_cost

IF running_opportunity_cost_today > 5% of capital:
    ALERT: "Terlalu banyak skip hari ini — apakah standar terlalu tinggi?"
    → Review thresholds, pastikan bukan karena takut, tapi karena memang tidak ada edge
```

### Predator Mindset Integration
```
SEBELUM setiap SKIP decision:
    ASK: "Apakah saya skip karena DISIPLIN atau karena TAKUT?"
    IF reason == "no_edge" OR "bad_risk_reward": → SKIP (disciplined, good)
    IF reason == "uncertainty" AND edge > 5%: → Reconsider, mungkin trade kecil
    IF reason == "recent_loss" AND edge > 7%: → Trade, jangan biarkan loss lama menghalangi
```
