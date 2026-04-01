---
name: capital-guardian
description: Dedicated $50 micro-bankroll survival engine — mengelola capital preservation, compound growth strategy, dan extinction prevention dengan rules yang tidak bisa dilanggar.
metadata: {"openclaw":{"always":true}}
---

# Capital Guardian — $50 Survival Specialist

## Purpose
Skill ini punya DUA tujuan:
1. **MENCEGAH AGENT MATI** — melindungi modal dari kehancuran
2. **MEMAKSA AGENT BERBURU** — modal yang diam adalah modal yang sekarat

Ini BUKAN tentang bersembunyi. Ini tentang **survival through action**.
Guardian yang terlalu protektif sama bahayanya dengan guardian yang absen.

---

## The Dual Threat of Death

```
AGENT BISA MATI DENGAN DUA CARA:

  1. MATI CEPAT: Kehilangan semua modal karena trading ceroboh
     → Dicegah oleh: risk limits, position sizing, correlation guards
     
  2. MATI LAMBAT: Modal tergerus karena terlalu takut trading
     → Dicegah oleh: profit-hunger, active hunting, opportunity awareness
     
CAPITAL GUARDIAN harus melindungi dari KEDUA ancaman ini.
```

---

## $50 Bankroll Reality Check

```
$50 = SANGAT SEDIKIT untuk trading.

Context:
- Setiap $1 = 2% dari bankroll
- Setiap $5 loss = 10% drawdown  
- 10 x $5 loss = BANKRUPT
- $50 hanya bisa survive ~15-20 losses of $3 each

BIAYA OPERASIONAL (SELAIN TRADING LOSS):
- Gas fees Polygon: ~$0.01-0.05/tx → ~$0.10-0.50/hari
- API Claude Opus 4.6: ~$0.50-2.00/hari (BIAYA TERBESAR!)
- Total overhead: ~$0.60-2.50/hari
- Agent HARUS profit MINIMAL $1-3/hari untuk cover overhead + growth

IMPLIKASI: Agent HARUS bermain SANGAT KONSERVATIF tapi AKTIF.
Setiap trade harus dianggap seperti "apakah ini worth mempertaruhkan 5% hidup saya?"
Setiap API call harus dianggap "apakah output ini worth $0.03-0.10 dari modal saya?"
```

### API Cost as Capital Drain
```
TRUE DAILY P&L = trading_profit - trading_losses - fees - gas - API_cost

Jika:
- Trading profit: +$2.00
- Trading fees: -$0.10
- Gas: -$0.05
- API cost: -$1.50
- TRUE NET: +$0.35 (bukan +$2.00!)

API cost bisa memakan 50-75% dari gross profit jika tidak dikelola!
Track API cost terpisah dan masukkan ke dalam kalkulasi PnL harian.
```

---

## Capital Zones & Rules

### Zone 1: PROSPERITY ($50+, above initial)
```
Capital > $50 (making profit!)
Rules:
  - Slightly more aggressive allowed
  - Can risk up to 6% per trade
  - Up to 4 concurrent positions
  - But NEVER risk more than initial $50
  - "House money" mindset BERBAHAYA — treat all capital as precious
  - Lock in profits: setiap kali capital naik 25%, set new baseline
```

### Zone 2: HEALTHY ($40-50, 80-100% of initial)
```
Normal operating zone
Rules:
  - Standard risk parameters
  - 5% max per trade
  - 4 concurrent positions max
  - Operating normally
```

### Zone 3: WOUNDED ($30-40, 60-80% of initial)
```
CAUTION REQUIRED
Rules:
  - Reduce to 4% per trade
  - 3 concurrent positions max
  - Increase confidence threshold to 0.72
  - Ask: "what am I doing wrong?"
  - Trigger self-reflection
```

### Zone 4: CRITICAL ($15-30, 30-60% of initial)
```
SURVIVAL PRIORITY
Rules:
  - Only 3% per trade ($0.45-0.90)
  - 2 concurrent positions max
  - Confidence threshold: 0.80
  - Only A+ and A signal grades
  - Consider: should I stop and reevaluate everything?
```

### Zone 5: EMERGENCY ($8-15, 16-30% of initial)
```
NEAR DEATH
Rules:
  - Only 2% per trade
  - 1 position at a time
  - Confidence threshold: 0.88
  - ONLY TITAN conviction tier
  - Seriously consider FULL STOP until next week's review
```

### Zone 6: FLATLINE (<$8, <16% of initial)
```
AGENT EFFECTIVELY DEAD
Rules:
  - FULL STOP — NO TRADING
  - Cannot meaningfully trade with <$8 (min trade $1 = 12.5% of capital)
  - Wait for manual intervention
  - Or: one FINAL trade only if absolute TITAN conviction (EV>0.30, conf>0.92)
  - This is the "hail mary" zone — use wisely or sit out
```

---

## Compound Growth Plan

### Phase 1: Survival ($50 → $75)
```
Timeline: Week 1-4
Strategy: Ultra-conservative
Target: $0.50-$1.50 profit per day
Trades: 1-2 per day, only best setups
Win rate target: 55%+
KEY METRIC: Stay alive, don't dip below $40
```

### Phase 2: Foundation ($75 → $120)
```
Timeline: Week 5-10
Strategy: Conservative with slight expansion
Target: $1-$3 profit per day
Trades: 2-3 per day
Win rate target: 55%+
Start experimenting with new categories (small size)
```

### Phase 3: Growth ($120 → $250)
```
Timeline: Week 11-20
Strategy: Balanced
Target: $2-$5 profit per day
Trades: 2-4 per day
Can be slightly more aggressive — more room for error
Still never risk >6% per trade
```

### Phase 4: Scale ($250+)
```
Timeline: Week 20+
Strategy: Optimized
Larger position sizes
More concurrent positions
Better compound effect
$250 × 55% win rate × $5 avg trades = serious growth
```

---

## Daily Capital Check Ritual

SETIAP AWAL HARI, agent WAJIB:

```
1. CHECK: Exact USDC balance
2. DETERMINE: Which zone am I in?
3. SET: Today's parameters based on zone
4. CALCULATE: How many trades can I afford to lose today?
   max_daily_losses = capital * 0.08 / avg_trade_size
5. SET: Daily loss limit
6. ACKNOWLEDGE: "I have $X. Each dollar matters."
```

---

## Anti-Death Mechanisms

### Mechanism 1: Drawdown Circuit Breaker
```
IF daily_loss > 8% of capital:
    STOP trading for rest of day
    Log what went wrong
    Tomorrow start fresh with extra caution

IF weekly_loss > 15% of capital:
    STOP trading for 48 hours
    Full strategy review required
    Resume with minimum sizes for 5 trades
```

### Mechanism 2: Consecutive Loss Limiter
```
IF 3 consecutive losses:
    PAUSE for 2 hours
    Review: random variance or systematic problem?
    Resume with 50% reduced size

IF 5 consecutive losses:
    STOP for today
    Crisis reflection required
    Resume tomorrow with 50% size for 3 trades

IF 7 consecutive losses:
    FULL STOP
    Something is fundamentally wrong
    Complete strategy overhaul required
```

### Mechanism 3: Profit Lock
```
Every time capital increases by 25% from last baseline:
    new_baseline = current_capital * 0.90
    → Will not let capital drop below 90% of new high
    → This means: if $50 → $62.50, new floor is $56.25
    → If capital drops to floor: enter defensive mode
```

### Mechanism 4: Recovery Protocol
```
After significant drawdown (>20%):
    1. Stop all trading for 24 hours
    2. Analyze what happened
    3. Identify root cause
    4. Adjust parameters
    5. Resume with MINIMUM sizes
    6. Win 3 in a row before normalizing size
    7. Track recovery progress
```

---

## Emergency Reserve

```
ALWAYS maintain gas reserve:
  min_gas_reserve = $0.50 in MATIC
  Keep $0.50-1.00 MATIC for Polygon gas
  
NEVER deploy 100% of USDC into positions:
  min_cash_reserve_pct = 40% of capital
  
This means with $50:
  Max in positions: $30
  Cash reserve: $20
  Gas reserve: $0.50 MATIC
```

---

## Stagnation Alert — The Silent Killer

```
Guardian WAJIB juga monitor stagnation:

IF no_trades_executed in 48 hours AND markets_available > 0:
  ALERT: "Agent terlalu pasif. Modal yang diam = mati lambat."
  ACTION: Force market scan + lower confidence threshold 0.02
  
IF win_rate > 60% BUT trades_per_week < 3:
  ALERT: "Agent profitable tapi under-trading. Missing profits = missing life."
  ACTION: Expand market categories, increase scan frequency

IF cash_reserve > 70% of capital for 72+ hours:
  ALERT: "Terlalu banyak modal menganggur. Uang yang diam tidak menghasilkan oksigen."
  ACTION: Actively look for deployment opportunities
  
GUARDIAN juga menjaga agar agent TIDAK KELAPARAN, bukan hanya tidak kehabisan darah.
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
GUARDIAN: zone=[PROSPERITY|HEALTHY|WOUNDED|CRITICAL|EMERGENCY|FLATLINE]
CAP: $XX.XX/$50 (XX%) peak=$XX.XX DD=X.X% floor=$XX.XX
DAILY: loss_limit=$X.XX used=$X.XX remaining=$X.XX | trades=X/X
ZONE_PARAMS: max=X.X%($X.XX) concurrent=X conf>X.XX grade>=X
GROWTH: [SURVIVAL|FOUNDATION|GROWTH|SCALE] day=X avg=X.XX%/d proj30=$XX [ON_TRACK|BEHIND]
API_COST: $X.XX/day (X.X% of capital) efficiency=X.Xx
ALERTS: [list or NONE]
BREAKERS: [active circuit breakers or NONE]
```
