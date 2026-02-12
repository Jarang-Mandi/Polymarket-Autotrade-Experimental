---
name: news_intelligence
description: Mengumpulkan dan menganalisa berita dari SEMUA kategori (sports, politics, crypto, entertainment, science) untuk mendeteksi information edge sebelum market bereaksi. 
metadata: {"openclaw":{"always":true}}
---

# News Intelligence — Multi-Category Information Engine

## Purpose
Mendeteksi informasi yang dapat mempengaruhi probabilitas market SEBELUM market bereaksi penuh.
Information edge adalah salah satu edge terbesar yang bisa dimiliki agent di Polymarket.

---

## Scope — ALL Polymarket Categories

### 1. Sports
- Soccer (liga top, Champions League, World Cup)
- Basketball (NBA, Euroleague, FIBA)
- American Football (NFL, College)
- MMA/Boxing (UFC events)
- Tennis, Cricket, dan major events lainnya

**Key information**: Injuries, lineups, weather, form, head-to-head, venue

### 2. Politics
- US Elections (Presidential, Congressional, State)
- International Elections
- Policy decisions (Fed rate, legislation)
- Government appointments
- Regulatory actions

**Key information**: Polls, endorsements, debate performance, scandal, policy announcements

### 3. Crypto
- Price predictions (BTC, ETH end-of-month/year)
- ETF approvals/rejections
- Protocol upgrades
- Regulatory decisions
- Exchange listings/delistings

**Key information**: On-chain data, exchange flows, developer activity, regulatory filings

### 4. Entertainment
- Award shows (Oscars, Grammys, Emmys)
- Box office predictions
- TV ratings
- Celebrity events

**Key information**: Critic reviews, historical voting patterns, pre-show favorites, early screenings

### 5. Science & Technology
- Space launches (SpaceX, NASA)
- FDA approvals
- Tech product launches
- AI developments

**Key information**: Test results, regulatory filings, insider reports, scientific publications

### 6. Current Events
- Weather events
- Economic indicators
- Social media trends
- Misc. verifiable facts

**Key information**: Official sources, data feeds, government reports

---

## News Impact Classification

| Impact Type | Description | Typical Probability Shift |
|------------|-------------|--------------------------|
| CRITICAL_CONFIRMED | Verified game-changer (star player out, candidate drops out) | 10-30% |
| MAJOR_CONFIRMED | Verified significant impact (key player questionable, endorsement) | 5-15% |
| MODERATE_CONFIRMED | Verified moderate impact (minor injury, policy shift) | 3-8% |
| MINOR_CONFIRMED | Verified but small impact | 1-3% |
| RUMOR_CREDIBLE | From credible source but unconfirmed | Apply 30% of estimated full impact |
| RUMOR_UNCERTAIN | Unverified, questionable source | IGNORE for trading, monitor |
| NOISE | Irrelevant or misleading | IGNORE completely |

---

## News Source Reliability Tiers

### Tier 1 (Reliability 0.95+): Official Sources
- Team official announcements
- Government press releases
- SEC filings
- NASA mission updates
- Election commission results

### Tier 2 (Reliability 0.80-0.95): Reputable Media
- ESPN, BBC Sport
- Reuters, AP
- Bloomberg, CNBC
- CoinDesk, The Block

### Tier 3 (Reliability 0.60-0.80): Credible Journalists
- Known beat reporters with track record
- Insider accounts with verified history
- Expert analysts

### Tier 4 (Reliability 0.30-0.60): Social Media & Rumors
- Twitter/X trending (verify independently)
- Reddit threads
- Telegram groups
- Weight very carefully → mostly noise

---

## News Processing Pipeline

```
1. COLLECT: Gather news from multiple sources
2. CLASSIFY: Category, impact type, reliability
3. VERIFY: Cross-reference across sources
4. ASSESS: Estimate probability impact
5. PRICE CHECK: Is this already priced in?
6. TIMING: How quickly will others react?
7. INTEGRATE: Feed into Bayesian update engine
```

### "Already Priced In" Detection
```
time_since_news = now - news_timestamp
market_movement_since_news = |price_now - price_at_news_time|

IF time_since_news > 2 hours AND market_movement_since_news > estimated_impact * 0.7:
    → Probably already priced in (70%+ absorbed)
    → Remaining edge is small
    
IF time_since_news < 30 minutes AND market_movement_since_news < estimated_impact * 0.3:
    → NOT fully priced in yet
    → OPPORTUNITY: act quickly but verify first
    
IF time_since_news < 5 minutes:
    → Very early — biggest potential edge
    → BUT also highest risk of misinformation
    → VERIFY before acting
```

---

## Category-Specific Analysis Templates

### Sports Template
```
{
  "event": "",
  "sport": "",
  "teams": [],
  "key_players_status": {},
  "injuries": [],
  "lineup_changes": [],
  "weather": "",
  "venue_factor": "",
  "recent_form": {},
  "head_to_head": {},
  "motivation_factors": [],
  "probability_impact": 0.00
}
```

### Politics Template
```
{
  "election/event": "",
  "candidates": [],
  "latest_polls": {},
  "poll_aggregate": 0.00,
  "endorsements_recent": [],
  "scandal_impact": "",
  "debate_performance": "",
  "early_voting_data": {},
  "demographic_shifts": {},
  "probability_impact": 0.00
}
```

### Crypto Template
```
{
  "asset": "",
  "event_type": "",
  "on_chain_data": {},
  "exchange_flows": {},
  "regulatory_status": "",
  "developer_activity": "",
  "market_sentiment": "",
  "technical_levels": {},
  "probability_impact": 0.00
}
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
NEWS: processed=XX actionable=X

ACTIONABLE:
1. [cat/subcat] "[headline]" src=[source] T[1-3] rel=X.XX
   impact=[inc|dec|neutral] shift=+/-X.X% priced=XX% remaining_edge=X.X%
   urgency=[HIGH|MED|LOW] → [TRADE|MONITOR|IGNORE] [affected_market_ids]
2. ...
(max 5 entries)

UPCOMING: [events within 48h or NONE]
CATEGORY_HOT: [categories with most activity]
```
