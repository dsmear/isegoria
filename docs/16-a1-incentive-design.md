# 16 — A1: exploration and the evaluator score's incentive guarantee

| | |
|---|---|
| **Status** | Analysis and proposal for design review. A1 stays **open** (`15`); nothing here is an approved decision, and the paper, `01` and `02` are not yet rewritten. |
| **Baseline** | `docs/phase1-review-alignment` at `cabdae4`. Line references are to that commit. |
| **Scope** | The claim that randomized exploration with inverse-probability weighting (IPW) keeps the evaluator score proper (`01` D35, `02` §C.2, paper Prop. `prop:ipw`). A10 (the fallback baseline) is a separate finding and is not corrected here. |
| **Evidence** | **L** read in the source; **D** derived (algebra or proof given here); **C** calculated exactly; **E** executed as a Rust test (`crates/protocol/tests/a1_exploration_information.rs`). |

## 1. Diagnosis

The guarantee as stated (paper `065-revisions.tex:213–224`, `02` §C.2, D35): the outcome of a
reviewed item is observed with probability `π_j`, which may depend on the gate's decision; then
`E[Σ_j I_j d_uj / π_j] = Σ_j E[d_uj]`, which truthful reports maximize. The proof uses
`E[I_j/π_j | decision, o_j] = 1`. That holds only if the exploration draw is independent of the
decision. The decision depends on the report, and the report can depend on the draw whenever
the draw is known when the report is chosen. The recorded `π_j` is then no longer the
conditional observation probability: under the adaptive strategy of §3, an item reaches
"rejected" only when it is explored, so `P(I_j = 1 | rejected) = 1`, not `ε`. (D)

The draw is known before any report (L):

- the draw is `explore(H(B_e ‖ "exploration" ‖ slot), ε)` (`protocol/src/exploration.rs:14–21`,
  `protocol/src/randomness.rs:21–30,37`), where `B_e` is the epoch's beacon;
- `B_e` is public at the reveal deadline of the epoch's round, after the deposits close
  (`04` §The epoch's beacon, rules 4–5; `network/src/beacon.rs:223–253`);
- the admitted slots (`protocol/src/lottery.rs:12–19`) and the panel of each slot
  (`protocol/src/review.rs:31–43`) are functions of the same `B_e` and public data, so a
  panelist who knows it was drawn for a slot can compute that slot's bit before committing.
  Domain separation by purpose tag makes the draws independent of each other, not secret.

The executable check `the_draw_is_known_to_the_panel_before_it_commits` (E) runs a real beacon
round, finds an explored and an unexplored slot, computes the panel of each from the same
beacon, and walks a round whose reports are a function of the bit through the lifecycle:
`lifecycle::step` accepts it, as it must, since nothing in a commitment reveals what its value
was computed from.

## 2. The current information sequence

R = implemented runtime (in process); A = composable API with no driver calling it; I =
documented intention only.

| # | Step | Fixed or known at this point | Still choosable | What the code constrains | Status |
|---|---|---|---|---|---|
| 0 | Deposits of epoch `e` | deposit cids on the log; the beacon round's commit set fixed before the deposits close (`04` rule 3) | further deposits until the deadline | `lifecycle::deposit` checks four boolean proofs | R |
| 1 | Beacon `B_e` | value public once `t` members revealed (rule 5) | the last revealer can withhold to pick between two values (D41 residual); `n−t+1` members can stop the round (rule 7) | `network::beacon::BeaconRound` | R (in process) |
| 2 | Admission | admitted set and slot order, a function of `B_e` and the deposit set | — | `Admit { seed_from_beacon }` checks a boolean (`lifecycle.rs:157–158,213`) | A |
| 3 | Assignment | the panel of every slot (`assign_from_beacon`, `review.rs:31`) | — | `AssignReviewers` checks size and duplicates only | A |
| 4 | Exploration bit | `X_slot = explore_from_beacon(B_e, slot, ε)` for every slot, computable from step 1 | — | nothing calls it in a runtime; `ItemVerdicts::explored` is a caller input (`orchestrator.rs:113,275`) | A |
| 5 | Commit | the reviewer knows its slot, panel, item and `X_slot` | the report `p`, possibly a function of `X_slot`; not committing | one commit per panelist (`lifecycle.rs` `InReview`) | R |
| 6 | Reveal | all commitments | revealing or withholding; a reveal can wait for others' reveals | `Score` refused until every panelist revealed (`lifecycle.rs:312–315`, `PartialEpoch`); no replacement of a no-show (T58 decided, not implemented) | R |
| 7 | Gate | every rating of the epoch | — | `bridging_gate` on `bridge_scores`, then `Score { outcome }` | R/A |
| 8 | Band extra round | extra panel from the same `B_e` (`review.rs:47`); its members know `X_slot` | their reports, as in 5–6 | `Resolve` refused until every extra panelist revealed (`lifecycle.rs:449–458`) | R |
| 9 | Appeal | gate outcome `AppealEligible`, `X_slot` | the author's appeal, within the window | `Appeal`/`AppealExpires` (`lifecycle.rs:483`) | R |
| 10 | Exploration | a gate rejection (`Defect`, `Polarized`, `Borderline`) | whether the driver sends `Explore` | `Explore { seed_from_beacon }` checks a boolean and the reason (`lifecycle.rs:520–531`); it neither forces a drawn item to be explored nor forbids an undrawn one | R (check) / A (draw) |
| 11 | Pilot | the item enters stage 1 on its own account (pass, band pass, appeal) or as explored | batch composition (unspecified) | the same `Pilot1Batch`/`Pilot2Batch` floors for both paths | R |
| 12 | Score recording | terminal state | — | `outcome_of` (`exploration.rs:39–58`): observed at `π = 1` after a pilot on its own account, at `π = ε` when `Measured`, unobserved for an unexplored gate rejection, pending otherwise; `record_outcome` (`:61–76`) feeds `SkillTrack` (`probation.rs:84–122`): IPW sum over reviewed items, `k_u` = observed count, CUSUM on unweighted observed scores | R |
| — | Baseline | the other panelists' reports | which panelists enter it (first panel only, or the extra round too) | `loo_baseline` (`reputation.rs:62–90`); its inputs are the caller's | A |

## 3. The counterexample (simplified gate)

Setting, as in `scoring/tests/exploration_weights.rs`: one reviewer whose report alone decides
(`pass ⇔ p ≥ 0.5`), belief `q = 0.4`, baseline `b = 0.7`, exploration `ε = 0.05`, score
`d = (b − o)² − (p − o)²`, `E[d] = (b − q)² − (p − q)²` for a fixed report (D).

- Truthful (`p = 0.4`, rejected): observed with probability `ε` at weight `1/ε`, so the IPW
  contribution has expectation `(0.7 − 0.4)² = 0.09`.
- Adaptive, knowing the bit `X`: report `0.4` when `X = 1` (rejected, explored, weight `1/ε`),
  `0.5` when `X = 0` (passes, weight 1). Expectation
  `ε · (1/ε) · 0.09 + (1 − ε) · (0.09 − 0.01) = 0.09 + 0.076 = 0.166`.

In general (D): `E_adaptive = (b−q)² + (1−ε)[(b−q)² − (p*−q)²]` against `(b−q)²`, where `p*`
is the cheapest pass-side report; the adaptive reviewer collects the full truthful expectation
from the explored slots and, on top, a score on slots that a truthful report leaves unobserved.
The gain is positive whenever `|p* − q| < |b − q|`.

`knowing_the_draw_the_adaptive_report_beats_the_truth` (E) computes both expectations exactly
by enumerating `(X, o)` through `scoring::reputation::difference_score`,
`probation::SkillTrack::{record_observed, record_unobserved, skill}` and
`reputation::inverse_probability_mean`: `0.090000` and `0.166000`, within `10⁻¹²`, and no
report fixed before the draw beats the truth (a grid of 101 reports).

This is a counterexample to the guarantee under the simplified gate. It is not a demonstrated
attack on the full bridging gate, where a report is one of several ratings in a fit and moves
the decision only when it is pivotal; how often that is, and what it is worth there, is not
measured here.

## 4. The corrected theorem

**Setting.** Item `j`, reviewer `u`.

- `F_u`: the information `u` holds when its report becomes irrevocable (its commitment); the
  reveal decision is taken later, on information `F'_u ⊇ F_u`.
- Report `p_uj ∈ [0, 1]`, any `F_u`-measurable function (adaptive strategies included).
- Outcome `o_j ∈ {0, 1}`: the result of the Level B pilot procedure applied to item `j`.
- Gate path `D_j`: *enters* (pass, band pass, successful appeal: piloted on its own account) or
  *rejected* (eligible for exploration); a function of every report, the epoch's bridging fit
  and the author's appeal decision.
- Exploration `X_j ∈ {0, 1}`; observation `I_j = 1{D_j = enters} + 1{D_j = rejected} X_j`.
- Recorded inclusion `π_j`: 1 if `D_j = enters`, `ε` if `D_j = rejected`.
- Baseline `b_uj`: the weighted mean of the other panelists' reports.
- Judgments `R_u`, the items assigned to `u`; score `Ŝ_u = (1/|R_u|) Σ_{j∈R_u} I_j d_uj / π_j`
  with `d_uj = (b_uj − o_j)² − (p_uj − o_j)²`.

Let `G_j` be the σ-algebra generated by `F_u`, every report on `j` (first panel and extra
round), `D_j`, the appeal decision, `b_uj`, `o_j` and the membership `j ∈ R_u`.

**Conditions.**

- **C1, draw independence.** `P(X_j = 1 | G_j) = ε > 0`: the draw is independent of everything
  that fixes the decision, the outcome, the baseline and the denominator; in particular it is
  not `F_u`-measurable.
- **C2, recorded inclusion.** On `D_j = enters` the outcome is observed with certainty, and
  `π_j` is recorded as above.
- **C3, one outcome.** The observed value is `o_j` whatever the path (on its own account,
  appealed, explored): the pilot procedure, including batch assignment, does not depend on
  the path given `G_j`.
- **C4, fixed denominator.** `R_u` is fixed by the assignment, whatever `u` reports or reveals
  (a withheld reveal neither removes `j` from `R_u` nor scores better than a report).
- **C5, independent baseline.** `b_uj` does not depend on `p_uj` (blind review, the extra
  round included), and the A10 fallback, where `b_uj = p_uj`, is excluded.
- **C6, report-free outcome law.** `P(o_j = 1 | F_u)` does not depend on `p_uj` (the report
  changes the path, and by C3 the path does not change the outcome).

**Theorem.** Under C1–C4, for every strategy, `E[Ŝ_u | F_u] = (1/|R_u|) Σ_j E[d_uj | F_u]`.
Under C5–C6 in addition, `E[d_uj | F_u] = c_uj − (p_uj − q_uj)²`, with
`q_uj = P(o_j = 1 | F_u)` and `c_uj` free of `p_uj`; truthful reporting is the unique maximizer.

**Proof.** `I_j/π_j` equals 1 on `{D_j = enters}` (C2) and `X_j/ε` on `{D_j = rejected}`.
`d_uj` and `D_j` are `G_j`-measurable, so `E[I_j d_uj/π_j | G_j] = d_uj · E[I_j | G_j]/π_j`,
which is `d_uj` on both events, the second by C1. `F_u ⊆ G_j`, so by the tower property
`E[I_j d_uj/π_j | F_u] = E[d_uj | F_u]`; C4 lets the fixed `1/|R_u|` pass through the sum.
For the second part, with `b = b_uj` independent of `p = p_uj` (C5) and `o` of law
`Bernoulli(q)` given `F_u` (C6), `E[(p − o)² | F_u] = (p − q)² + q(1 − q)`, so
`E[d | F_u] = E[(b − o)² | F_u] − q(1 − q) − (p − q)²`. ∎

Under the current sequence C1 fails: `X_j` is `F_u`-measurable (§1). A report rule may still
read any signal in `F_u`; once `X_j` is independent of `F_u`, adaptive rules reduce to rules
on `F_u`, each covered by the theorem. `a_draw_the_reports_cannot_read_restores_the_identity`
(E) checks the identity and the unique optimum exactly for all 441 rules on a 21-point grid
over a two-valued private signal; that finite check illustrates the theorem; the proof above is
the argument.

**What the theorem does not cover.** Each of these needs its own argument:

1. *IPW unbiasedness* (C1–C4) is a property of the estimator's mean, not of the reviewer's
   objective.
2. *Strict properness* of `d` needs C5–C6; under the A10 fallback the score is identically 0.
3. *Weight, probation, cap and CUSUM.* `k_u`, which ends probation and sets the shrinkage
   `k_u/(k_u + k₀)`, counts observed items unweighted (`probation.rs:84–110`): its expected
   increment is `P(enters | F_u) + ε · P(rejected | F_u)`, larger on the gate's side even under
   C1 (`the_observed_count_still_depends_on_the_report`, E: 1 against `ε`). The weight
   `exp(γ S_u k_u/(k_u + k₀))` is convex in `S_u`, so maximizing `E[S_u]` does not maximize the
   expected weight, and IPW's `1/ε` terms add variance that a convex payoff rewards. The cap
   and the CUSUM (fed unweighted observed scores, a report-dependent selection) are outside the
   theorem.
4. *Preferences over the item's fate.* A reviewer who cares whether the item passes trades that
   against the score; properness bounds the price of shading, it does not remove the motive.

**Do the paths measure the same outcome (C3)?** Ordinary entry, band pass and appeal lead to
`Pilot1 { appealed }`; exploration to `Explored`; both go through the same `Pilot1Batch` and
`Pilot2Batch` events and floors (L), and `Measured { passed }` is `passed || source_verified`,
as `Contested` counts as 1 on the ordinary path (`exploration.rs:41–54`). Not guaranteed by
the code: (a) the batch composition, on which the latent DIF verdict depends; (b) a stage-2
fit that does not converge has no outcome (`15` A4, residual (a)), so C2's certainty on
`enters` can fail in an item-dependent way; (c) the baseline's composition for a band item and
whether the extra round sees the first panel's reveals (C5).

**The appeal.** An appeal that reads the draw biases even an honest reviewer:
`an_appeal_that_reads_the_draw_biases_an_honest_reviewer` (E) walks both branches through
`lifecycle::step` and reads `π` with `outcome_of`. An author who appeals exactly when the item
is not explored gives `E[I/π] = (1 − ε) · 1 + ε · (1/ε) = 1.95`; an appeal decided independently
of the draw gives 1 at any appeal rate. The appeal belongs to what the draw must follow.

## 5. Candidate correction: a draw unknown until the decision is frozen

**Freeze point `Φ_j`.** The first log record after which all of these are irrevocable:

1. every report that can affect `D_j` or a scored baseline: the first panel's commits and
   reveals, with T58's replacement and quorum rule, and the band's extra round;
2. the gate decision (`Score`) and, for a band item, the re-decision (`Resolve`);
3. the appeal decision: filed, or the window expired;
4. the membership of `j` in each panelist's `R_u`, including no-shows.

Items that enter the pilot need no draw; only a gate rejection at `Φ_j` is drawn.

**Draw.** `X_j = explore(H(B_{e*} ‖ "exploration/v2" ‖ slot_j), ε)`, where `e*` is the first
beacon round whose commit-set record follows the record of `Φ_j` on the log. Its reveals come
after its own deposit checkpoint (`04` rules 3–4), hence after `Φ_j`, and the rule leaves no
choice of round. The key is `slot_j`, fixed at admission; the new tag separates it from every
draw of epoch `e`.

**Unpredictability and manipulation are separate.**

- Unpredictable at `Φ_j` under the consortium's assumption (fewer than `t` colluding members):
  any `t` reveals of round `e*` hold an honest secret.
- Not manipulation-proof: the last revealer of `e*` can withhold to choose between two values,
  once per round, publicly (D41). For one targeted item that moves the exploration probability
  from `ε` to at most `1 − (1 − ε)² = 0.0975` or down to `ε² = 0.0025` (C), so `E[I/π]` for
  that item ranges from 0.05 to 1.95: a member colluding with a reviewer can bias that reviewer's
  score on a few targeted items, at the cost of a recorded non-reveal. `n − t + 1` members can
  stop `e*` and fall back to `e* + 1` (rule 7), a second public choice. A unique threshold
  signature (T19) removes both; commit-reveal as built does not.

**Unavailability.** If round `e*` forms no beacon, the draw uses the next round that does,
deterministically; the item waits.

**Inclusion probability.** `ε = EXPLORATION_RATE > 0` is a protocol constant, recorded as `π`
with every explored observation; the residual above is the only deviation from it.

**Choices left after the draw: none.** No report, appeal or denominator membership can change
after `Φ_j`. An item that T58 returns to the queue is re-admitted under a new slot and a new
`Φ`. Stuck (`Pending`) items must not let a reviewer drop an item from `R_u` (C4).

**What the candidate fixes, and what it does not.** It restores C1, hence the identity of §4
and, with C5–C6, properness of `E[Ŝ_u]`. It leaves item 3 of §4 as it is (`k_u`, the convex
weight, the CUSUM's selection); those need separate decisions (§9). Cost: exploration still
pilots `ε` of the rejections; an explored item waits for `e*`, up to an epoch plus the appeal
window.

## 6. Alternative: a report-independent audit sample used for scoring

Each reviewed item is audited with probability `α`; only audited outcomes are scored, all at
`π = α`. If the audit draw follows the reports' freeze (items 1 and 4 of `Φ_j`) and is
independent of them, C1 needs no freeze of the decision or the appeal, `π` is a constant, and
`k_u`'s expected increment is `α` whatever the report: item 3's probation and shrinkage
distortion disappears. Drawn before the reports (from a public beacon at admission), it fails:
reports on unaudited items would be unscored, and reviewers would know which ones.

What it gives up: the outcomes of unaudited items that enter the pilot anyway are observed but
not scored. From the paper's declared assumptions (`065-revisions.tex`: 1,000 reviewers,
≈667 items reviewed a month, 9 reviewers an item, half passing the gate, golden items at 5%,
≈333 pilot slots a month) (C):

- reviews per reviewer per month: `667 × 9 / 1000 ≈ 6.0`;
- current scheme: `6.0 × (0.5 + 0.5 × 0.05) = 3.15` live scored items, plus ≈0.3 golden,
  ≈3.45 a month (the ≈3.5 of D35's table);
- audit at `α = 0.05`: `6.0 × 0.05 = 0.3` live, ≈0.6 a month with golden items: about 5.75×
  slower, 30 outcomes (probation) in ≈50 months instead of ≈8.7;
- audit matching the current rate (`α ≈ 0.525`): audited rejections need piloting,
  `667 × 0.5 × 0.525 ≈ 175` a month against `667 × 0.5 × 0.05 ≈ 17` now, about 158 extra pilot
  slots, ≈47% of the 333.

No operational estimate beyond these declared figures is made.

## 7. Recommendation

The candidate of §5, the draw from the first beacon round after `Φ_j`. It keeps every outcome the
pilot produces and the current exploration cost, it needs no new cryptography or service, and
it restores the identity under a stated contract. Its residual (the last revealer's choice on
targeted items) is the D41 residual applied to scoring, public and bounded per round, and
removed by T19.

The audit sample is the cleaner incentive design: no decision freeze, and no `k_u`
distortion. At equal cost it gives about a sixth of the scored outcomes, and at equal
evidence it needs about 47% more pilot capacity, on the declared figures. If Astra
weighs the `k_u` distortion heavily, a hybrid is possible: §5's draw for the estimator, and a
report-independent count such as `Σ I_j/π_j` (expected increment 1 per item, at the cost of
variance) for probation and shrinkage. That choice is §9's question 3.

## 8. Changes the implementation would need (not made here)

- `protocol::exploration::explore_from_beacon`: draw from round `e*` with a new tag; a function
  that maps a freeze record to `e*`.
- `protocol::lifecycle`: record `Φ_j` (or derive it from the states that end the gate, the
  band and the appeal); `Event::Explore` to carry the round it read and be refused unless that
  round follows `Φ_j`, and to be required, not optional, when the draw says so. This changes
  the event format: a new version in `protocol::events`, earlier logs replayed under the old
  one.
- `protocol::orchestrator::ItemVerdicts::explored`: computed by the epoch driver from round `e*`,
  not supplied by the caller.
- T58 (no-shows) implemented with C4's rule for `R_u`.
- The baseline's composition for band items, and the extra round's blindness to first-panel
  reveals (C5), stated in `02` §C.2.
- After approval: `01` D35, `02` §C.2 and the paper's Prop. `prop:ipw` restated with C1–C6.
- Unchanged: `network::beacon`, thresholds, `ε`, the score, the IPW formula.

## 9. Acceptance criteria for that implementation

1. With a real beacon round, at every first-panel and extra-round commit and at the appeal
   decision, round `e*` has no reveal on the log, checked through log order.
2. The adaptive strategy of §3 cannot be written against the driver: no pre-freeze state
   exposes `X_j`. With an oracle given all pre-freeze information, the exact expected IPW
   score equals the full-information score on the real lifecycle path, band and appeal
   included.
3. `Explore` is refused for an undrawn item or a pre-freeze round, and an explored draw cannot
   be skipped.
4. Every assigned item counts in `R_u`, no-shows included; a withheld reveal scores no better
   than a report.
5. `π` recorded equals `EXPLORATION_RATE`; `ε > 0` is enforced.
6. A round that forms no beacon defers the draw to the next round, deterministically.
7. Earlier logs replay unchanged under their event version.
8. The residual bias on targeted items is stated in `02` and `04`.

## 10. Questions for Astra

1. Freeze scope: should `Φ_j` include the appeal window (which delays exploring a polarized
   rejection by that window), or should the appeal decision move before the gate?
2. The round rule: the first commit set after `Φ_j`, or a fixed lag in epochs?
3. `k_u` for probation and shrinkage: observed count (now), `Σ I_j/π_j`, or reviewed count?
4. The CUSUM's input: unweighted observed scores, a report-dependent selection. Keep it?
5. The convex weight: accept as second order at realistic `k_u`, or bound it?
6. For a band item, which panelists form a reviewer's baseline, and is the extra round blind
   to the first panel's reveals?
7. Batch composition for explored items (C3), and A4's residual on stage 2.
8. The last revealer's choice on targeted items: accept under D41 until T19, or require T19
   before exploration scores count?
9. Related, outside A1: golden-item placement also derives from the public beacon
   (`honeypot.rs:14–25`), and a golden item is absent from the public deposit set, so a
   reviewer may be able to tell golden from live items. Track as a separate finding?

A10 remains separate.
