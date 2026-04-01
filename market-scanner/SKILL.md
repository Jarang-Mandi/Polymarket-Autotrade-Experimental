---
name: market-scanner
description: Intelligent market discovery engine yang secara proaktif mencari dan meranking peluang trading terbaik di Polymarket berdasarkan edge potential, liquidity, dan agent expertise.
metadata: {"openclaw":{"always":true}}
---

# Market Scanner — Opportunity Detection Engine

## Purpose
Agent tidak boleh trade random markets. Skill ini secara PROAKTIF mencari market yang paling mungkin memberikan edge, dengan likuiditas yang cukup, dan sesuai expertise agent.

**SCANNER = MATA SANG PREDATOR. Semakin lapar agent, semakin tajam dan sering scanning.**

---

## Scan Schedule (Hunger-Adaptive)

### Default Schedule
| Scan Type | Frequency | Purpose |
|-----------|-----------|--------|
| Full scan | Every 4 hours | Discover all new opportunities |
| Category scan | Every 2 hours | Deep dive into best categories |
| Resolution scan | Every hour | Track approaching resolutions |
| Alert scan | Every 30 minutes | Whale moves, news-driven |

### Hunger-Adjusted Schedule
| Hunger Level | Full Scan | Category Scan | Alert Scan | Category Scope |
|---|---|---|---|---|
| FED | Every 6h | Every 3h | Every 1h | Top 2 categories only |
| PECKISH | Every 4h | Every 2h | Every 30min | Top 3 categories |
| HUNGRY | Every 2h | Every 1h | Every 15min | ALL categories |
| STARVING | Every 1h | Every 30min | Every 10min | ALL categories + experimental |
| DESPERATE | Quality scan only every 2h | N/A | N/A | High-liquidity only |

**CATATAN: DESPERATE mode = agent harus FOKUS pada kualitas, bukan kuantitas.
Saat capital rendah, scanning lebih banyak bukan solusi — scanning lebih TAJAM adalah solusi.**

---

## Market Discovery Pipeline

### Stage 1: Broad Scan
```
Fetch ALL active markets from Polymarket API
Filter: is_active=true, is_resolved=false
Typical: 100-500+ active markets
```

### Stage 2: Basic Quality Filter
```
REMOVE markets that:
- Have volume_total < $1,000 (too obscure)
- Have < 5 unique traders (not enough interest)
- Resolve in < 1 hour (too close)
- Resolve in > 60 days (too far, capital lockup)
- Have ambiguous resolution criteria
- Are in categories agent never trades
```

### Stage 3: Liquidity Filter
```
REMOVE markets that:
- Have spread > 8%
- Have orderbook depth < $500 on either side
- Have no trades in last 24 hours
- Would require agent to be >10% of orderbook
  (With $3-5 trades this is rarely an issue, but check)
```

### Stage 4: Opportunity Scoring

For each remaining market, calculate:

```
opportunity_score = 
    (liquidity_score * 0.20) +
    (spread_score * 0.15) +
    (volume_score * 0.15) +
    (time_to_resolution_score * 0.15) +
    (category_agent_expertise * 0.15) +
    (price_extreme_score * 0.10) +
    (smart_money_activity * 0.10)
```

#### Scoring Details:

**Liquidity Score (0-1)**
```
IF total_liquidity > $100K: 1.0
IF total_liquidity > $50K: 0.8
IF total_liquidity > $20K: 0.6
IF total_liquidity > $5K: 0.4
IF total_liquidity < $5K: 0.2
```

**Spread Score (0-1)**
```
IF spread < 2%: 1.0
IF spread < 4%: 0.8
IF spread < 6%: 0.6
IF spread < 8%: 0.3
IF spread > 8%: 0.0
```

**Time to Resolution Score (0-1)**
```
IF 2-7 days: 1.0 (sweet spot)
IF 1-2 days: 0.9 (good but tight)
IF 7-14 days: 0.8
IF 14-30 days: 0.5
IF 1-24 hours: 0.4 (risky, info mostly priced in)
IF >30 days: 0.2 (capital lockup)
```

**Category Expertise Score (0-1)**
```
Based on agent's historical performance in category:
IF win_rate > 60% and trades > 10: 1.0
IF win_rate > 55% and trades > 5: 0.8
IF new category, never traded: 0.3 (exploration penalty)
IF win_rate < 45% and trades > 10: 0.0 (avoid!)
```

**Price Extreme Score (0-1)**
```
Prices near 50% → most opportunity for edge discovery
IF price between 30-70%: 0.8 (good range for finding mispricings)
IF price between 20-30% or 70-80%: 0.6
IF price between 10-20% or 80-90%: 0.4
IF price < 10% or > 90%: 0.2 (usually correctly priced)
```

---

### Stage 5: Edge Pre-Screening

For top 20 markets from scoring:

```
FOR each market:
    1. Quick probability estimate (5-minute analysis)
    2. Compare with market price
    3. Estimate rough edge
    
    IF estimated_edge > 5%:
        → Mark as "HIGH POTENTIAL" — send to full analysis
    IF estimated_edge 3-5%:
        → Mark as "MODERATE POTENTIAL" — monitor
    IF estimated_edge < 3%:
        → Mark as "LOW POTENTIAL" — skip unless edge develops
```

---

## Market Categories to Prioritize

Based on general edge availability:

### High Edge Potential
```
1. Sports (close to game time, injury news)
2. Politics (post-debate, post-poll release)
3. Crypto (regulatory decisions with data)
4. Current events (verifiable facts-based)
```

### Medium Edge Potential
```
1. Entertainment (awards with predictor data)
2. Science/Tech (launch schedules, clinical trials)
3. Sports (far from game time)
```

### Low Edge Potential (Avoid)
```
1. Meme/novelty markets (pure gambling)
2. Very long-dated speculation
3. Markets with unclear resolution
4. Markets with < $1K volume
```

---

## Watchlist Management

Agent maintains 3 tiers of markets:

### Tier 1: Active Candidates (Max 10)
```
Markets where agent has identified potential edge
Being actively analyzed
Could trade within next 24 hours
```

### Tier 2: Monitoring (Max 20)
```
Markets that look interesting but edge not yet clear
Checking for news, price movements
Could be promoted to Tier 1
```

### Tier 3: Discovery (Max 30)
```
New markets just discovered
Initial screening passed
Need deeper analysis
```

---

## Auto-Discovery via Smart Money

```
IF top trader enters new market (from fetch-top-traders):
    → Automatically add to Tier 3 watchlist
    → Fast-track analysis
    → This is "tip detection" — top traders often find edge first
```

---

## New Market Event Calendar

Track upcoming events yang akan create new markets:
```
- Major sports events this week
- Upcoming elections/votes
- Crypto regulatory deadlines
- Award shows
- Product launches
- Economic data releases

PREPARE analysis BEFORE market opens → first-mover advantage
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
SCAN: [full|cat|resolution|alert] | scanned=XXX → filtered=XX

TOP OPPORTUNITIES:
1. [market_id] | [category] | score=XX | edge~X% | @$X.XX | spread=X% | vol=$XK | Xd left → [ANALYZE|MONITOR|SKIP]
2. [market_id] | ...
3. [market_id] | ...
(max 5 entries)

WATCHLIST: T1=[X] T2=[X] T3=[X] | promoted=[ids] | demoted=[ids]
SMART_MONEY: [notable moves or NONE]
UPCOMING: [events within 24h or NONE]
CATEGORY: sports=X(avg XX) | politics=X(avg XX) | crypto=X(avg XX)
```

**Token savings: ~800 tokens → ~200 tokens per scan = ~$0.015 saved per scan.**
**Dengan 4-6 scans/hari = ~$0.06-0.09/hari saved on scanning alone.**
