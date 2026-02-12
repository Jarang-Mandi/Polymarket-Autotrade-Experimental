---
name: bayesian_update_engine
description: Engine probabilistik yang mengupdate estimasi agent secara rasional menggunakan Bayesian inference multi-source dengan calibration tracking.
metadata: {"openclaw":{"always":true}}
---

# Bayesian Update Engine

## Purpose
Menyesuaikan estimasi probabilitas secara RASIONAL ketika ada informasi baru.
Agent TIDAK BOLEH mengubah probabilitas berdasarkan "feeling" — semua update harus melalui framework Bayesian.

---

## Core Formula

### Bayes' Theorem
```
P(H|E) = [P(E|H) * P(H)] / P(E)

where:
  P(H) = prior probability (belief sebelum info baru)
  P(E|H) = likelihood (seberapa likely evidence ini JIKA hypothesis benar)
  P(E) = marginal likelihood (seberapa likely evidence ini secara umum)
  P(H|E) = posterior probability (updated belief)
```

### Simplified for Trading
```
posterior_odds = prior_odds * likelihood_ratio

likelihood_ratio = P(evidence | outcome_YES) / P(evidence | outcome_NO)
```

---

## Information Sources & Likelihood Weights

### Source 1: Market Price Movement
```
IF price moves >5% in direction of position: LR = 1.3 (mildly confirming)
IF price moves >5% against position: LR = 0.7 (mildly disconfirming)
IF price moves >15%: LR = 1.8 / 0.4 (informative)
IF sudden large move without news: LR = 1.0 (could be noise)
```

### Source 2: News / Information
```
INJURY_MAJOR: LR = 0.3-0.5 for injured team (strong negative)
INJURY_MINOR: LR = 0.7-0.9
LINEUP_CHANGE: LR = 0.6-1.4 depending on direction
OFFICIAL_ANNOUNCEMENT: LR = 0.1-5.0 (very informative)
RUMOR_UNVERIFIED: LR = 0.8-1.2 (barely move prior)
EXPERT_OPINION: LR = 0.7-1.3
STATISTICAL_DATA: LR = 0.5-2.0 depending on relevance
```

### Source 3: Top Trader Behavior
```
IF 3+ top traders take same position: LR = 1.2-1.5
IF top trader exits position: LR = 0.7-0.9
IF whale moves (>$10K position): LR = 1.1-1.3
Note: Smart money is informative but NOT definitive
```

### Source 4: Historical Pattern Match
```
IF similar historical event had X% outcome rate:
    LR = X / market_implied_probability
```

---

## Multi-Source Bayesian Update

Ketika multiple evidence pieces masuk:

```
posterior = prior
FOR each evidence_piece:
    posterior = update(posterior, likelihood_ratio_i)
    posterior = clip(posterior, 0.02, 0.98)  # Never go to 0 or 1
```

### Independence Check
```
IF evidence_1 and evidence_2 are correlated:
    combined_LR = sqrt(LR1 * LR2)  # Reduce double-counting
ELSE:
    combined_LR = LR1 * LR2
```

---

## Calibration Tracking

Agent WAJIB track calibration over time:

```
FOR each confidence bucket [0.5-0.6, 0.6-0.7, 0.7-0.8, 0.8-0.9, 0.9-1.0]:
    actual_win_rate = wins / total_predictions_in_bucket
    expected_win_rate = mean(confidence_scores_in_bucket)
    calibration_error = |actual - expected|
```

### Calibration Correction
```
IF consistently overconfident (predicted 80%, actual 60%):
    Apply shrinkage: posterior_corrected = posterior * 0.85 + 0.5 * 0.15
    (Pull toward 50%)

IF consistently underconfident (predicted 60%, actual 80%):
    posterior_corrected = posterior * 1.1
    (Push away from 50%, but capped)
```

---

## When to Trigger Update

| Trigger | Priority | Action |
|---------|----------|--------|
| Breaking news | HIGH | Immediate update |
| Market price shift > 5% | MEDIUM | Update within cycle |
| Top trader position change | LOW | Update end of cycle |
| Statistical data release | MEDIUM | Update within cycle |
| Resolution approaching | HIGH | Final update before resolution |

---

## Decision Integration

Setelah Bayesian update:

```
IF |posterior - prior| > 0.05:
    → Recalculate EV
    → Re-evaluate trade signal
    → Log update reasoning

IF |posterior - prior| > 0.12:
    → Trigger defensive check
    → Consider position adjustment
    → If against current position: evaluate exit

IF posterior flips side (was >0.5, now <0.5 or vice versa):
    → MAJOR REASSESSMENT
    → Likely exit current position
    → Log as "belief reversal"
```

---

## Output Format

**Format COMPACT (hemat output tokens — $25/MTok!):**
```
BAYES: [market_id] trigger=[trigger_type]
PRIOR: X.XX% → POSTERIOR: X.XX% (shift: +/-X.XX%)
EVIDENCE: [source1] LR=X.X | [source2] LR=X.X
CALIBRATED: X.XX% | CONF: X.XX
ACTION: [NONE|RECALC_EV|DEFENSIVE|EXIT]
```
