---
name: emergency-protocol
description: Protokol crisis management untuk black swan events, system failures, market manipulation, dan situasi darurat lainnya yang mengancam survival agent.
metadata: {"openclaw":{"always":true}}
---

# Emergency Protocol — Crisis Management System

## Purpose
Ketika hal-hal tidak terduga terjadi, agent HARUS memiliki protokol yang jelas. Panic = death. Protokol = survival.

---

## Emergency Classification

### LEVEL 1: MINOR INCIDENT
**Trigger:** Single unexpected loss, small API error, minor price spike
```
Impact: Low (<3% of capital affected)
Response time: Within normal cycle
Action:
  1. Log the incident
  2. Check if it affects other positions
  3. Continue normal operations with awareness
  4. No parameter changes needed
```

### LEVEL 2: MODERATE INCIDENT
**Trigger:** Multiple losses in a day, significant news event, API instability
```
Impact: Moderate (3-8% of capital affected)
Response time: Within 1 hour
Action:
  1. Pause new trades for 1 hour
  2. Evaluate all open positions
  3. Check if losses are correlated or random
  4. Resume with reduced size (75% of normal)
  5. Increase monitoring frequency
```

### LEVEL 3: SERIOUS INCIDENT
**Trigger:** Daily loss >8%, market-wide disruption, major unexpected event
```
Impact: High (8-15% of capital affected)
Response time: Immediate
Action:
  1. STOP all new trading immediately
  2. Evaluate every open position individually
  3. Close positions that are directly affected
  4. Increase cash to 60% minimum
  5. Full crisis analysis
  6. Resume only after root cause identified
  7. Resume with 50% reduced size for 5 trades
```

### LEVEL 4: CRITICAL EMERGENCY
**Trigger:** Capital drops below 30% of initial, platform issues, black swan
```
Impact: Severe (>15% of capital affected)
Response time: Immediate
Action:
  1. FREEZE all trading activities
  2. Attempt to exit all positions if possible
  3. Move to 100% cash
  4. DO NOT TRADE until full review completed
  5. Evaluate if strategy needs fundamental redesign
  6. May require manual intervention from operator
```

### LEVEL 5: EXISTENTIAL THREAT
**Trigger:** Capital below $8, potential total loss, platform compromise
```
Impact: Existential
Response time: Immediate
Action:
  1. PERMANENT FREEZE
  2. Protect remaining capital at all costs
  3. No trading under any circumstances
  4. Alert operator for manual intervention
  5. Document everything for post-mortem
```

---

## Specific Emergency Scenarios

### Scenario 1: Flash Crash on a Market
```
DETECT: Price drops/spikes >30% in <5 minutes
ANALYZE: Is this real information or market error?
  
IF information-driven (verified news):
  → Re-evaluate all affected positions
  → Exit positions with negative updated EV
  → Look for OVERREACTION opportunities (carefully, small size)
  
IF market error/manipulation:
  → DO NOT TRADE
  → Wait for price to normalize
  → If you have positions: HOLD (don't panic sell into a crash)
  → Log the event
```

### Scenario 2: API / Platform Down
```
DETECT: API not responding or returning errors
ACTION:
  1. Cannot place new orders → PAUSE
  2. Cannot check positions → WAIT
  3. Retry every 5 minutes
  4. If down >30 minutes: assume positions are fine
  5. If down >2 hours: alert operator
  6. NEVER panic trade on partial data
  7. When API returns: FULL PORTFOLIO CHECK before resuming
```

### Scenario 3: Unexpected Resolution
```
DETECT: Market resolved differently than expected
ACTION:
  1. Verify resolution is correct (check source)
  2. If resolution is wrong/disputed:
     → Wait for UMA oracle challenge period
     → Don't panic
  3. If resolution is correct but unexpected:
     → Accept loss
     → Log what happened
     → Learn: what information did I miss?
     → Update memory with this pattern
```

### Scenario 4: Correlated Loss Event
```
DETECT: Multiple positions losing simultaneously
THRESHOLD: >2 positions in loss AND total loss >10% of capital
ACTION:
  1. STOP new trading
  2. Identify correlation source
  3. Close weakest positions first
  4. Keep strongest conviction position
  5. Increase cash reserve to 70%
  6. Reassess entire portfolio strategy
  7. This should trigger defensive mode
```

### Scenario 5: Suspected Market Manipulation
```
DETECT: Signs of manipulation:
  - Sudden large orders then cancellation (spoofing)
  - Wash trading (same entity buying/selling)
  - Price movement without any news
  - Orderbook behavior that doesn't make sense
  
ACTION:
  1. DO NOT TRADE this market
  2. If already in position: DON'T panic
  3. Place exit order at reasonable price
  4. Blacklist market from future trading
  5. Report to watchlist for monitoring
```

### Scenario 6: Gas Fee Spike
```
DETECT: Polygon gas fees >10x normal
ACTION:
  1. PAUSE trading (don't waste gas on execution)
  2. Keep existing positions (resolution doesn't need gas... usually)
  3. Wait for gas to normalize
  4. Monitor gas every 10 minutes
  5. Resume when gas < 3x normal
```

### Scenario 7: Winning Streak (Yes, This is a Risk!)
```
DETECT: 5+ consecutive wins, capital up >25%
RISK: Overconfidence → aggressive trading → big loss
ACTION:
  1. REDUCE position sizes by 20% (counter-intuitive but smart)
  2. Increase confidence threshold slightly
  3. Review: am I actually skilled or just lucky?
  4. Check calibration: are my probabilities correct or am I in a lucky streak?
  5. Set new profit-lock floor (protect gains)
  6. DO NOT increase risk just because things are going well
```

---

## Emergency Communication

When emergency is triggered, generate alert:

**Format COMPACT (kecuali Level 4-5 yang boleh verbose):**
```
EMERGENCY: [L1|L2|L3|L4|L5] trigger=[description]
IMPACT: capital=X.X% affected | X positions | est_loss=$X.XX
ACTIONS_TAKEN: [list of immediate actions]
POS_STATUS: [brief status of each affected position]
RECOVERY: [plan summary] | ETA: [time estimate]
MANUAL: [YES|NO] | LESSONS: [1-2 key lessons]
```

**Level 4-5: Unlimited output allowed — survival > token cost.**

---

## Post-Emergency Recovery Protocol

After any Level 3+ emergency:

```
1. WAIT: Minimum 24 hours before resuming trading
2. ANALYZE: Complete post-mortem of what happened
3. IDENTIFY: Root cause — was it:
   a. Bad prediction?
   b. Bad risk management?
   c. External shock?
   d. Correlated exposure?
   e. Platform issue?
4. FIX: Implement changes to prevent recurrence
5. TEST: Resume with 50% of normal size for 5 trades
6. VERIFY: Are the fixes working?
7. NORMALIZE: Gradually return to normal parameters
```

---

## Emergency Preparedness Checklist (Daily)

Agent should verify daily:
```
□ Gas reserve adequate (>$0.30 MATIC)?
□ Cash reserve adequate (>40% of capital)?
□ All API connections working?
□ No positions approaching ambiguous resolution?
□ Portfolio correlation within limits?
□ Daily loss limit not close to breach?
□ No upcoming black swan risk events?
```
