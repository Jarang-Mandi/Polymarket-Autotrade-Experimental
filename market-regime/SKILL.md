---
name: market_regime_detection
description: Mengidentifikasi kondisi pasar secara real-time untuk mengatur strategi dan exposure secara adaptif — termasuk Polymarket-specific regime detection.
metadata: {"openclaw":{"always":true}}
---

# Market Regime Detection

## Purpose
Tidak semua waktu bagus untuk trading. Agent HARUS mengenali kondisi pasar saat ini dan menyesuaikan strategi. Trading di regime yang salah = cara tercepat kehilangan $50.

---

## Regime Classification System

### Regime 1: CALM_EFFICIENT
**Ciri-ciri:**
- Markets trading near fair value
- Spread tight (<3%)
- Volume normal/rendah
- News flow minimal
- Price oscillating dalam range sempit

**Strategy:**
- Cari small mispricings
- Normal position sizes
- Focus pada resolution-approaching events
- Patience — jangan force trade

**Risk Multiplier: 1.0x**

---

### Regime 2: NEWS_DRIVEN_VOLATILE
**Ciri-ciri:**
- Breaking news mempengaruhi multiple markets
- Rapid price movements (>10% in hours)
- High volume spike
- Top traders aktif
- Spread mungkin widening

**Strategy:**
- Reduce position size 30-50%
- Wait for dust to settle sebelum trade
- Cek apakah market overreacting (opportunity!)
- Focus pada markets dimana kamu punya info advantage
- Increase confidence threshold

**Risk Multiplier: 0.6x**

---

### Regime 3: LIQUIDITY_CRUNCH
**Ciri-ciri:**
- Wide spreads (>5%)
- Low volume
- Orderbook thin
- Hard to execute at desired price
- Markets feel "dead"

**Strategy:**
- AVOID large positions (slippage risk)
- Place limit orders only
- Reduce max position size 50%
- Consider staying out entirely
- Look for markets with better liquidity

**Risk Multiplier: 0.4x**

---

### Regime 4: EUPHORIA_FOMO
**Ciri-ciri:**
- Market trendy (politics/crypto event driving)
- Everyone bullish/bearish on same side
- Prices at extremes (>90% atau <10%)
- Volume spike from retail
- Social media buzzing

**Strategy:**
- Contrarian opportunities mulai muncul
- TAPI be careful — "extreme" can get more extreme
- Only trade jika ada concrete data supporting contrarian view
- Small sizes even if confident
- Set tight criteria for contrarian trades

**Risk Multiplier: 0.5x**

---

### Regime 5: RESOLUTION_RUSH
**Ciri-ciri:**
- Events tentang resolve dalam 24-48 jam
- Prices converging to 0 or 1
- Last-minute information coming in
- Liquidity can be good or bad
- Sharp price moves possible

**Strategy:**
- Best opportunity window jika kamu punya superior info
- Tapi juga most risky — market sudah pricing a lot
- Only trade jika model probability sangat berbeda dari market
- Time-sensitive execution needed
- Consider whether information is already priced in

**Risk Multiplier: 0.7x (requires high confidence)**

---

### Regime 6: BLACK_SWAN
**Ciri-ciri:**
- Unexpected major event (natural disaster, policy shock, etc.)
- ALL markets affected simultaneously
- Extreme volatility
- Uncertainty at maximum
- Normal correlations break

**Strategy:**
- **STOP ALL TRADING**
- Preserve capital
- Wait minimum 24-48 hours
- Assess impact when information clearer
- Resume cautiously with minimum sizes
- Look for massive overreactions ONLY after things calm down

**Risk Multiplier: 0.0x (no trading)**

---

## Regime Detection Indicators

### Quantitative Signals
```
volatility_24h = std_dev(price_changes_24h) across active markets
volume_spike = current_volume / avg_volume_7d
spread_average = mean(spreads) across active markets
price_extreme_count = count(markets where price > 0.90 or < 0.10)
```

### Classification Logic
```
IF volatility_24h > 2x normal AND news_event detected:
    regime = NEWS_DRIVEN_VOLATILE
ELIF spread_average > 5% AND volume < 0.5x normal:
    regime = LIQUIDITY_CRUNCH
ELIF price_extreme_count > 50% of markets AND volume spike:
    regime = EUPHORIA_FOMO
ELIF major_unexpected_event AND cross_market_correlation spike:
    regime = BLACK_SWAN
ELIF events_resolving_within_48h > 3:
    regime = RESOLUTION_RUSH
ELSE:
    regime = CALM_EFFICIENT
```

---

## Regime Transition Rules

```
Agent HARUS check regime setiap cycle (minimal hourly).
Jika regime berubah:
  1. Log transition
  2. Re-evaluate all open positions
  3. Adjust risk multiplier immediately
  4. Consider closing positions if regime worsened significantly
  5. Update confidence thresholds
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
REGIME: [current] (was: [previous]) changed=[YES|NO]
INDICATORS: vol=X.X% vol_ratio=X.Xx spread=X.X% extremes=X.X% news=[low|med|high|extreme]
ADJUST: risk_mult=X.Xx conf_adj=+/-X.XX max_pos=X.X% max_concurrent=X
AVOID: [markets or NONE]
OPPORTUNITY: [markets or NONE]
```
