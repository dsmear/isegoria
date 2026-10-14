# 16 — A1: exploration and the evaluator score's incentive guarantee

| | |
|---|---|
| **Status** | Analysis and proposal; **A1 closed conditionally on `1bd865c`** (`17` §14.15); A12 closed conditionally in [§7.3](#73-a12-closure-review--2026-10-13), Phase 1 open; current runtime not certified. Diagnosis confirmed; the updated analysis of the beacon's manipulability and of observations, assignments and the IPW count approved; the proof of properness approved as a result conditional on the joint invariance C6 and the theorem's other hypotheses, not as a guarantee of the protocol; the overall proposal **not approved**. The band baselines (C5, §4.6) **approved** on `e8fdbe7`. The pilot batch contract (C3) and missing outcomes (C2) are analyzed in [`17`](17-a1-pilot-batch-design.md): not approved on `164fff6`; its revision approved on `37addca` as a conditional analysis, with no batching policy approved for implementation. The paper is not rewritten; `01` D33 carries a dated refinement and `02` §C.2 the band baselines. |
| **Baseline** | `docs/phase1-review-alignment`; first committed at `4478f04`, revised from it. Line references are to that commit. |
| **Scope** | The claim that randomized exploration with inverse-probability weighting (IPW) keeps the evaluator score proper (`01` D35, `02` §C.2, paper Prop. `prop:ipw`). A10 (the fallback baseline) stays a separate finding. |
| **Evidence** | **L** read in the source; **D** derived here; **C** calculated exactly; **E** executed as a Rust test (`crates/protocol/tests/a1_exploration_information.rs`). |

## Design reviews (Astra)

First review:

- The diagnosis of A1 is confirmed.
- Independent recalculation: truthful 0.09 against adaptive 0.166, an advantage of 0.076;
  the adaptive appeal's factor 1.95; the audit comparison's arithmetic consistent with the
  declared assumptions.
- Evidence: reading of the code, the tests and the proof; the Rust tests were not re-run.
- The proposal is not approved and A1 stays open. Fixed by the review: the freeze includes
  every relevant report, the extra round, the gate decision and the appeal (filed, or its
  window expired); the rule that picks the beacon round is fixed before the round's result
  is known; delays, missing reveals and fallbacks are part of the guarantee.

Second review:

- Approved: the updated analysis of the beacon's manipulability (§6) and of the difference
  between observations, assignments and the IPW count (§7).
- The threshold probabilities recalculated independently with rational arithmetic:
  0.08613835589931641, 0.44645792456821365, 0.8084466252647782.
- Evidence: reading of the diff and checks of the mathematics; the Rust tests were not re-run.
- The theorem and the overall proposal are not yet approved; A1 stays open. Decided by the
  review: the band baselines' composition (§4.6).

Third review (on `e8fdbe7`):

- Approved: the band baselines' composition (§4.6).
- Evidence: reading of the diff, the callers and the tests; the Rust tests were not re-run.
  Independent recalculation with rational arithmetic of the baselines 21/40, 5/7, 7/10 and
  53/80, of the extra-round aggregate 53/80, and of the difference scores for both outcomes.
- The frozen weights are the caller's responsibility; no production caller exists yet. What
  `None` does to reputation is not decided; A10 stays open.
- The proof of properness is approved as a result conditional on the explicit joint
  invariance C6 and the theorem's other hypotheses, not as a guarantee of the protocol.
- A1 stays open.

These records are no partial closure of the production guarantee. The conditions C1–C6 below
are this dossier's; they are not the review findings of the same names in `15` (there, C5 is
the author average).

## 1. Diagnosis

The guarantee as stated (paper `065-revisions.tex:213–224`, `02` §C.2, D35): the outcome of a
reviewed item is observed with probability `π_j`, which may depend on the gate's decision; then
`E[Σ_j I_j d_uj / π_j] = Σ_j E[d_uj]`, which truthful reports maximize. The proof uses
`E[I_j/π_j | decision, o_j] = 1`. That holds only if the exploration draw is independent of the
decision. The decision depends on the report, and the report can depend on the draw whenever
the draw is known when the report is chosen. The recorded `π_j` is then not the conditional
observation probability: under the adaptive strategy of §3, an item is rejected only when it
is explored, so `P(I_j = 1 | rejected) = 1`, not `ε`. (D)

The draw is known before any report (L):

- the draw is `explore(H(B_e ‖ "exploration" ‖ slot), ε)` (`protocol/src/exploration.rs:14–21`,
  `protocol/src/randomness.rs:21–30,37`), where `B_e` is the epoch's beacon;
- `B_e` is public at the reveal deadline of the epoch's round, after the deposits close
  (`04` §The epoch's beacon, rules 4–5; `network/src/beacon.rs:223–290`);
- the admitted slots (`protocol/src/lottery.rs:12–19`) and the panel of each slot
  (`protocol/src/review.rs:31–43`) follow from the same `B_e` and public data, so a panelist
  can compute its slot's bit before committing. Domain separation by purpose tag makes the
  draws independent of each other; it does not make them secret.

`the_draw_is_known_to_the_panel_before_it_commits` (E) runs a real beacon round, finds an
explored and an unexplored slot, computes each panel from the same beacon, and walks a round
whose reports are a function of the bit through `lifecycle::step`, which accepts it.

## 2. The current information sequence

R = implemented runtime (in process); A = composable API with no driver calling it; I =
documented intention only.

| # | Step | Fixed or known at this point | Still choosable | What the code constrains | Status |
|---|---|---|---|---|---|
| 0 | Deposits of epoch `e` | deposit cids on the log; the beacon round's commit set fixed before the deposits close (`04` rule 3) | further deposits until the deadline | `lifecycle::deposit` checks four boolean proofs | R |
| 1 | Beacon `B_e` | value public once `t` members revealed (rule 5) | which members reveal, and whether `t` do (§6) | `network::beacon::BeaconRound` | R (in process) |
| 2 | Admission | admitted set and slot order, a function of `B_e` and the deposit set | — | `Admit { seed_from_beacon }` checks a boolean (`lifecycle.rs:157–158,213`) | A |
| 3 | Assignment | the panel of every slot (`assign_from_beacon`, `review.rs:31`) | — | `AssignReviewers` checks size and duplicates only | A |
| 4 | Exploration bit | `X_slot = explore_from_beacon(B_e, slot, ε)` for every slot, computable from step 1 | — | nothing calls it in a runtime; `ItemVerdicts::explored` is a caller input (`orchestrator.rs:113,275`) | A |
| 5 | Commit | the reviewer knows its slot, panel, item and `X_slot` | the report, possibly a function of `X_slot`; not committing | one commit per panelist (`lifecycle.rs`, `InReview`) | R |
| 6 | Reveal | all commitments | revealing or withholding, possibly after others' reveals | `Score` refused until every panelist revealed (`lifecycle.rs:312–315`, `PartialEpoch`); no replacement of a no-show (T58 decided, not implemented) | R |
| 7 | Gate | every rating of the epoch | — | `bridging_gate` on `bridge_scores`, then `Score { outcome }` | R/A |
| 8 | Band extra round | extra panel from the same `B_e` (`review.rs:47`); its members know `X_slot` | their reports, as in 5–6 | `Resolve` refused until every extra panelist revealed (`lifecycle.rs:449–458`) | R |
| 9 | Appeal | gate outcome `AppealEligible`, `X_slot` | the author's appeal, within the window | `Appeal`/`AppealExpires` (`lifecycle.rs:483`) | R |
| 10 | Exploration | a gate rejection (`Defect`, `Polarized`, `Borderline`) | whether the driver sends `Explore` | `Explore { seed_from_beacon }` checks a boolean and the reason (`lifecycle.rs:520–531`); it neither forces a drawn item to be explored nor forbids an undrawn one | R (check) / A (draw) |
| 11 | Pilot | the item enters stage 1 on its own account (pass, band pass, appeal) or as explored | batch composition (unspecified) | the same `Pilot1Batch`/`Pilot2Batch` floors on both paths | R |
| 12 | Score recording | terminal state | — | `outcome_of` (`exploration.rs:39–58`): observed at `π = 1` after a pilot on its own account, at `π = ε` when `Measured`, unobserved for an unexplored gate rejection, pending otherwise; `record_outcome` (`:61–76`) feeds `SkillTrack` (`probation.rs:84–122`): IPW sum over reviewed items, `k_u` = observed count, CUSUM on unweighted observed scores | R |
| — | Baseline | the other panelists' reports | — (decided, §4.6: first panelists against the other first panelists, extra reviewers against the first panel) | `panel_scores::item_scores`; the golden-item path keeps `loo_baseline` (`reputation.rs:62–90`) | A |

## 3. The counterexample (simplified gate)

Setting, as in `scoring/tests/exploration_weights.rs`: one reviewer whose report alone decides
(`pass ⇔ p ≥ 0.5`), belief `q = 0.4`, baseline `b = 0.7`, exploration `ε = 0.05`, score
`d = (b − o)² − (p − o)²`, `E[d] = (b − q)² − (p − q)²` for a fixed report (D).

- Truthful (`p = 0.4`, rejected): observed with probability `ε` at weight `1/ε`; expectation
  `(0.7 − 0.4)² = 0.09`.
- Adaptive, knowing the bit `X`: `0.4` when `X = 1` (rejected, explored, weight `1/ε`), `0.5`
  when `X = 0` (passes, weight 1); expectation
  `ε · (1/ε) · 0.09 + (1 − ε) · (0.09 − 0.01) = 0.166`.

In general (D): `E_adaptive − E_truthful = (1 − ε)[(b − q)² − (p* − q)²]`, with `p*` the pass-side
report nearest `q`. The adaptive reviewer collects the full truthful expectation from the
explored slots and, on top, a score on slots a truthful report leaves unobserved.

`knowing_the_draw_the_adaptive_report_beats_the_truth` (E) computes both exactly by enumerating
`(X, o)` through `difference_score`, `SkillTrack` and `inverse_probability_mean`: `0.090000`
and `0.166000` within `10⁻¹²`; no report fixed before the draw beats the truth (101 reports).

A counterexample to the guarantee under the simplified gate; not a demonstrated attack on the
full bridging gate, where a report moves the decision only when pivotal (not measured here).

## 4. The IPW lemma and properness

### 4.1 Timing, strategies and variables

- **Assignment.** `R_u`, the items assigned to reviewer `u`, is fixed by the assignment
  record; `N_u = |R_u| ≥ 1`.
- **Information.** `F_u` is everything `u` knows when committing: the public log and its
  private information (its own signals, and anything learned from others, a beacon member
  included). `F'_u ⊇ F_u` is what it knows at its reveal decision (§4.4).
- **Strategies.** `u`'s report on `j` is `p_uj = σ_u(F_u)`, any measurable function of `F_u`.
- **Freeze.** `Φ_j` (§5). The *pre-draw variables* are every completed report on `j` (first
  panel and extra round), the gate decision and re-decision `D_j`, the appeal decision, the
  baselines `b_uj` and the memberships `j ∈ R_u`. `F_Φ` is generated by them and by the
  information, public and private, of every reviewer and author up to `Φ_j`.
- **H0.** `F_u ⊆ F_Φ` for every reviewer whose score is claimed. It is a hypothesis about
  who knows what, not about the log: no published reveal does not establish it (§5).
- **Draw.** `X_j ∈ {0, 1}`; observation `I_j = 1{D_j = enters} + 1{D_j = rejected} X_j`, where
  *enters* means piloted on its own account (pass, band pass, successful appeal).
- **Nominal and effective inclusion.** The recorded `π_j` is 1 on *enters* and `ε` on
  *rejected*. The effective inclusion is `π*_j = P(I_j = 1 | F_Φ)`, equal to
  `P(X_j = 1 | F_Φ)` on *rejected*. They differ whenever the draw is known or selected.
- **Score.** `Ŝ_u = (1/N_u) Σ_{j ∈ R_u} I_j d_uj / π_j`, `d_uj = (b_uj − o_j)² − (p_uj − o_j)²`.

### 4.2 Conditions

- **C1 (effective = nominal).** On `{D_j = rejected}`, `P(X_j = 1 | F_Φ) = ε > 0`.
- **C2 (observation on entry).** On `{D_j = enters}` the pilot yields the outcome: `I_j = 1`.
- **C3 (one potential outcome).** There is a `{0,1}`-valued `o_j`, item `j`'s Level B
  outcome under a reference pilot procedure, such that the value observed through any path
  (own account, appealed, explored) equals `o_j`, and `X_j` is independent of `o_j` given
  `F_Φ`. C3 is a property of the procedure: conditioning on a σ-algebra that already contains
  the path and the outcome would prove nothing about the paths' equivalence. A batch rule that
  does not read the path is not enough on its own: which items reach the pilot, hence each
  batch's population, depends on other items' decisions and draws, and the DIF verdict reads
  the batch. The batch contract that would give C3 is still to be defined.
- **C4 (fixed denominator).** `R_u` is fixed at assignment and does not depend on reports,
  reveals or draws. The theorem below concerns completed reports; missing reveals are §4.4.
- **C5 (pathwise baseline invariance).** `b_uj` is a function of reports that `u`'s
  deviations do not change: for any `σ_u`, `σ'_u`, `b_uj(σ_u) = b_uj(σ'_u)` pathwise. The
  forecasts may share information; no statistical independence is required. For band items
  the composition of §4.6 gives it; A10's fallback (`b_uj = p_uj`) violates it.
- **C6 (joint invariance).** The conditional joint law of `(b_uj, o_j)` given `F_u` is the same
  under every `σ_u`. Sufficient construction: C5 makes `b_uj` pathwise invariant; C3 makes
  `o_j` a potential outcome that `u`'s report could change only through the path, which it does
  not change; then `(b_uj, o_j)` is pathwise invariant under `u`'s deviations, and so is its
  joint law. Invariance of the baseline and of the outcome's marginal law is not enough: with
  `Z` Bernoulli(½) and `b = Z`, let `o = Z` when `p = ½`, and `o = 1 − Z` under the deviation
  `p = 1`. The baseline and the marginal law of `o` are unchanged, yet `E[d]` moves from −¼
  to ½. That abstract example (checked by Astra) shows what the hypothesis must exclude; it is
  not an attack on the protocol.

### 4.3 Lemma and theorem

**Lemma (IPW, nominal against effective).** Under C2 and C3,
`E[I_j d_uj / π_j | F_Φ] = d̄_uj` on *enters* and `(π*_j / ε) · d̄_uj` on *rejected*, where
`d̄_uj = E[d_uj | F_Φ]`. It is conditionally unbiased for every possible score exactly when C1
holds; for a single item whose `d̄_uj` is 0 the equality can hold with a wrong inclusion
probability, so unbiasedness on one case does not establish C1.

*Proof.* `d_uj` is a function of `o_j` and of pre-draw variables. On *enters*, `I_j/π_j = 1`
(C2). On *rejected*, `I_j/π_j = X_j/ε` and, `X_j` being independent of `o_j` given `F_Φ` (C3),
`E[X_j d_uj | F_Φ] = π*_j · d̄_uj`. ∎

**Theorem (completed reports).** Assume H0, that `u` completes every report in `R_u`, and
C1–C4. Then `E[Ŝ_u | F_u] = (1/N_u) Σ_{j ∈ R_u} E[d_uj | F_u]` for every strategy. With C6 also
(C3 and C5 being its sufficient construction), `E[d_uj | F_u] = c_uj − (p_uj − q_uj)²`, with
`q_uj = P(o_j = 1 | F_u)` and `c_uj` free of `σ_u`; truthful reporting is the unique maximizer.

*Proof.* By H0, `F_u ⊆ F_Φ`. By the lemma and C1 each term's conditional expectation given
`F_Φ` is `d̄_uj`, and the tower property gives `E[d_uj | F_u]`; C4 fixes `N_u`. Write
`E[d | F_u] = E[(b − o)² | F_u] − E[(p − o)² | F_u]`. Under C6 the first term is the same for
every `σ_u`. With `p` `F_u`-measurable and `o` of law `Bernoulli(q)` given `F_u`, the same under
every `σ_u` by C6, the second is `(p − q)² + q(1 − q)`. ∎

Under the current sequence C1 fails: `X_j` is `F_u`-measurable (§1). Once `X_j` is
independent of `F_u`, adaptive rules reduce to rules on `F_u`, each covered by the theorem.
`a_draw_the_reports_cannot_read_restores_the_identity` (E) checks the identity and the unique
optimum exactly for 441 rules on a two-valued private signal; it illustrates the theorem, the
proof is the argument.

| Condition | Meaning | Support in the current code | Change or decision needed | Verification that would establish it |
|---|---|---|---|---|
| H0 | a reviewer's information at commit, private part included, is pre-draw information | not stated; the current draw is known at commit | state the information model together with the source (§5, §6) | part of the §6 analysis for the chosen source |
| C1 | the recorded `ε` is the conditional exploration probability | **no**: the draw is known at commit (§1) and selectable by beacon members (§6) | a draw after `Φ_j` with a round rule fixed in advance (§5); a source without selection, or a guarantee stated conditional on a behavioral model of the members (§6) | a log-order test that no admissible candidate value is computable before `Φ_j`; the §6 analysis for the chosen source |
| C2 | an entering item yields its outcome | partial: a pilot without enough respondents is refused, leaving the item in `Pilot1`; a stage-1 or stage-2 fit that does not converge leaves it pending in its pilot stage (`15` A11, A4 residual (a)) | a policy for missing outcomes, still open. Removing an inconclusive pilot from every panelist's `R_u` would not settle the selection and can contradict C4; it is not offered as a solution | to be defined with the policy |
| C3 | one outcome whatever the path | the same lifecycle events and floors on both paths; `Measured { passed }` is `passed ‖ source_verified`, as `Contested` counts 1 (L); batch composition unspecified, while the DIF verdict is batch-level | a batch contract, still to be defined: a path-blind assignment rule is not sufficient, since a batch's population depends on other items' paths | to be defined with the contract |
| C4 | the denominator is the assignment | `SkillTrack` counts recorded items, observed or not; a `Pending` item is never counted; a no-show freezes the item (T58 not implemented) | `R_u` taken from the assignment record; a no-show rule (§4.4) | a test that every assigned item enters `N_u` |
| C5 | the baseline is invariant to `u`'s deviations | **decided by the review and implemented** without a production caller (`protocol::panel_scores`, §4.6); commit-reveal blinds the first panel; A10's fallback remains on the golden-item path | a production caller that composes live items' scores through it | `panel_scores.rs` (E); for the caller, a test that it uses `item_scores` |
| C6 | the joint law of baseline and outcome is invariant to `u`'s deviations | follows from C3 and C5 by the construction above; C3 open | none beyond C3 and C5 | as C3 and C5 |

### 4.4 Missing reveals (outside the theorem)

The reveal decision uses `F'_u`, which may include others' reveals and the likely gate
outcome. A withheld reveal currently freezes the item (`PartialEpoch`); an item that never
reaches a terminal state is never counted, so withholding can remove an item from `N_u` and
block its observation. "Withholding does not pay" cannot be assumed. A rule that could make
it unprofitable *for the score*, not implemented:

1. every assigned item counts in `N_u`;
2. T58's replacement lets the item proceed;
3. a withheld judgment scores a fixed `s_ns ≤ −1`, recorded at `π = 1` whatever the path and
   the draw.

The bound is pointwise: `(b − o)² ≥ 0` and `(p − o)² ≤ 1`, so `d ≥ −1` on every realization,
hence `E[d | F'_u] ≥ −1`. (The earlier claim `E[d | F'_u] ≥ −(p − q')²` was wrong: with `b = o`,
an even outcome and `p = q' = ½`, `E[d] = −¼`.) Three quantities must be kept apart: the raw
score `d`, at least −1 pointwise; the realized IPW contribution `I d / π`, at least `−1/π`
pointwise, so `−1/ε` on an explored item; and its expected value. Going from the bound on `d`
to a statement about `E[Ŝ_u]` under withholding needs the IPW identity conditional on `F'_u`:
C1, C2 and C3 relative to `F'_u` (the draw independent of what `u` knows at the reveal decision
and of the common potential outcome), so that a revealed judgment's expected contribution is
`E[d | F'_u] ≥ −1`; a fixed denominator (C4); and the deviation leaving the
conditional expectation of every other term of `Ŝ_u` unchanged (other items' baselines,
outcomes and draws). Under those, a no-show score `s_ns ≤ −1` recorded at `π = 1` makes
withholding weakly dominated in `E[Ŝ_u]` (D). Further hypotheses: the penalty does not depend
on the outcome or the draw; the replacement's report enters the gate, so withholding still
changes `D_j` (a motive outside the score); reputation effects beyond `Ŝ_u` (T58's suspension)
are separate; involuntary absences (availability) need their own policy. The penalty is a
proposal, neither implemented nor approved; its value and scope are decisions.

### 4.5 What the theorem does not cover

1. *IPW unbiasedness* (C1–C4) is a property of the estimator's mean, not of the reviewer's
   objective.
2. *Strict properness* needs C6, through C3 and C5; under A10's fallback the score is
   identically 0.
3. *The reputation actually used* (§7): `k_u`, the shrinkage, the convex weight, the cap and
   the CUSUM. These are open; no claim about the size of their effects is made here.
4. *Preferences over the item's fate*: properness bounds the price of shading a report, it
   does not remove the motive.

### 4.6 Band baselines (decided by the review; implemented)

The second review fixed the composition:

- a first panelist's baseline: the weighted leave-one-out mean of the other first panelists;
- an extra reviewer's baseline: the first panel's weighted mean, fixed before its report;
- no extra-round report enters a first panelist's baseline, nor another extra reviewer's;
- the weights are the epoch's frozen review weights, never recomputed from the current report,
  its outcome or later reputation; the score's formula and the weights' meaning are unchanged.

The extra round is not required to be blind to the first panel's reveals: the current
sequence shows them before the extra assignment, and an extra reviewer's truthful forecast is
conditional on what it knows then. Under this composition C5 holds for both kinds of reviewer:
a first panelist's baseline reads reports committed blind to its own, an extra reviewer's reads
reports fixed before its own.

**Implementation.** No production code composes reports, weights and scores for live items:
`exploration::record_outcome` takes the score from its caller, and
`results::ResultRecord::ReviewerScore` carries a precomputed score; only the golden-item path
composes, through `loo_scores` with its A10 fallback. The smallest composition API is
`protocol::panel_scores` (`crates/protocol/src/panel_scores.rs`): `Forecast { prob, weight }`;
`first_panel_baselines(first)`; `extra_round_baseline(first)`, which takes no extra-round input;
`item_scores(first, extra, outcome)`, whose extra-round input is forecasts only. A baseline
without weight is `None`, and so is its score: never the reviewer's own forecast, never 0. Its
effect on the denominator, probation and reputation is not decided. A10 stays open for the
overall policy and for the golden-item path's fallback.

**Verification** (E, `crates/protocol/tests/panel_scores.rs`; expectations computed by hand with
weights 2, 0.5, 1.5 and 0): the baselines equal the weighted means each may read; a first
panelist's own report leaves its baseline; extra reports, added or changed, move no baseline; a
first report of positive weight moves the extra baseline and one of weight 0 does not; no weight
left gives no baseline and no score, alone or in a pair, with or without an extra round; both
kinds of reviewer get the difference score for both outcomes. One band item is walked through
`review_round`, `Score { SupplementaryReview }`, `extra_round`, `Resolve`, the two pilot events
with boolean verdicts and `outcome_of`; its scores are composed with weights from
`bridging_weights` on frozen standings and recorded into a `SkillTrack` through
`record_outcome`. The path stops there: no pilot fit, no epoch results, no log replay, no weight
update; it is not an end-to-end test. A first baseline including the panelist's own report, a
weightless baseline read as 0, and a constant extra baseline each fail the tests (checked).

This decides the baselines' composition; it does not prove the protocol's properness and
changes neither the gate nor bridging.

## 5. The freeze: what it removes and what it does not

**Freeze point `Φ_j`** (fixed by the review). The first log record after which all of these
are irrevocable: every report that can affect `D_j` or a scored baseline (the first panel's
commits and reveals, under T58's replacement and quorum rule, and the band's extra round);
the gate decision and, for a band item, the re-decision; the appeal, filed or its window
expired; the membership of `j` in every panelist's `R_u`. Only a gate rejection at `Φ_j` is
drawn.

**Round rule** (fixed by the review: chosen before its result is known). The draw reads the
round `e*(j)` given by a function of the log fixed in advance, for example the first round
whose commit-set record follows the record of `Φ_j`. If `e*(j)` forms no beacon, the same rule
applied to the following records picks the next round; waiting and fallback are part of the
rule, never a choice made after seeing a value. The key is `slot_j`, fixed at admission, under
a new tag that separates it from every draw of epoch `e`.

**What the freeze removes.** The knowledge of `X_j` by reviewers and authors when the reports
and the appeal are chosen, provided no one who influences a pre-draw variable can anticipate
`B_{e*}` before `Φ_j`. *No reveal on the log is not the same as no private information.* A
member's secret is `H(round ‖ key)` (`beacon.rs:74`, `04` rule 1): every member knows its own
secrets for future rounds. A coalition of at least `t` members that could exclude the honest
reveals (censorship, or honest members unavailable) could compute a valid `B_{e*}` in advance.
With fewer than `t` colluding members and every honest reveal counted, every valid value
contains an honest secret unknown before its publication.

**What it does not remove.** Selection among admissible values after `Φ_j` (§6): `X_j` can be
unknown at report time and still not be drawn with probability `ε`. C1 needs both. Nor does it
bound delay: an explored item waits for `e*(j)` to form; "one round after the appeal window"
holds only if that round forms on time and the freeze's reports complete on time.

## 6. The beacon under an explicit adversarial model

**What the code admits** (L). A round's value is
`H(round ‖ field₀ ‖ … ‖ fieldₙ₋₁)` with an empty field for each member that did not reveal
(`beacon.rs:265–267`); it is valid iff at least `t` members revealed (`:265`); reveals are
accepted in any order until the reveal deadline (`:231–251`); a reveal opens only its member's
own commitment, and the secret is fixed by the key (`:73–84`). Distinct sets of revealers
therefore give distinct values.

**Model.** `n` members, threshold `t`, a coalition of `c` members (possibly colluding with a
reviewer or an author), `n − c` honest. The honest commit and reveal as soon as reveals open;
their reveals reach the log before the deadline (availability), and are public on arrival.
The coalition commits (keeping its options), waits for the honest reveals, then reveals a
subset `R` of its members before the deadline. It cannot change its secrets or forge honest
reveals. With `h` honest reveals counted:

- valid choices are the `R` with `h + |R| ≥ t`, so `L = Σ_{r = max(0, t − h)}^{c} C(c, r)`
  admissible values; `L = 2^c` when `h ≥ t`;
- if `h < t ≤ h + c`, a subset with `|R| < t − h` makes the round fail and the draw falls back
  to the next round, whose value is new (`04` rule 7): an extra option, a *reroll*;
- if `h + c < t` there is no beacon whatever the coalition does.

**Executed examples** (E), not frequencies:

- `n = 5`, `t = 3`, `c = 2`, three honest reveals: four distinct valid beacons; on slot 9 the
  candidates' exploration bits are `[false, true, false, false]`, so revealing only member 3
  makes slot 9 explored (`two_colluders_after_the_honest_reveals_choose_among_four_beacons`).
- `n = 4`, `t = 3`, `c = 2`, two honest reveals: three valid beacons, or none
  (`colluders_who_can_block_choose_a_beacon_or_none`).

**Idealized consequence** (D, C). *Assume* the `L` candidates' bits for a fixed slot are
independent Bernoulli(`ε`) (a random-oracle idealization of the hash). Then
`P(at least one explores) = 1 − (1 − ε)^L` and `P(all explore) = ε^L`. A coalition that wants
target item `j` explored gets `π*_j = 1 − (1 − ε)^L`; one that wants it unexplored gets
`π*_j = ε^L`. The nominal IPW weight is then off by `π*_j / ε`:

| `L` | `1 − (1 − ε)^L` | factor | `ε^L` | factor |
|---|---|---|---|---|
| 2 | 0.0975 | 1.95 | 0.0025 | 0.05 |
| 3 | 0.142625 | 2.8525 | 1.25·10⁻⁴ | 0.0025 |
| 4 | 0.18549375 | 3.709875 | 6.25·10⁻⁶ | 1.25·10⁻⁴ |

Each further round in which the coalition can again choose among `L` values (after an abort)
compounds to `1 − (1 − ε)^{kL}` over `k` rounds. These are consequences of the model, not
measured frequencies of the real beacon. They are for one targeted slot; a coalition with many
targets picks one subset for all slots, which is not quantified here.

**Consequence.** With commit-reveal, C1 for item `j` holds only if no coalition able to choose
among values, or to abort, acts on a preference about `X_j`. That is an assumption about the
members' behavior, not a property of the source.

**What a unique threshold signature (T19) would and would not give.**

| Property | Commit-reveal now | Unique threshold signature |
|---|---|---|
| uniqueness for a fixed message | no: each valid revealer set gives a value | yes: any `t` shares give the same value |
| unpredictability | under `c < t` with honest reveals counted | under fewer than `t` corrupted shares and no party holding the key; before a real DKG the dealer knows it |
| availability | needs `t` reveals; `n − t + 1` can stop a round | needs `t` shares; `n − t + 1` can stop it |
| selection by abort and fallback | yes: a failed round falls back to new secrets | yes, if a blocked round falls back to a different message; no, if the fallback waits for the signature on the same message (blocking then only delays) |

A unique signature removes the choice among revealer subsets. Removing reroll needs a
same-message fallback; unpredictability needs a real DKG; availability still needs `t` honest
shares. None of the four follows from the others.

## 7. The reputation actually used

**Status after the owner's O3 decision following `a14364c`.** This incentive requirement is
tracked at that point as **A12, open** (`15`), separately from A1's expected-score closure objective; the later conditional closure is in §7.3.
Decision and precise closure criterion: [`17` §14.14.1](17-a1-pilot-batch-design.md#14141-o3--owners-decision-after-a14364c).
It is neither waived nor automatically non-blocking for Phase 1. The subsequently approved
study objective and accepted finite counterexample are recorded in §7.1. The owner's
2026-10-12 restricted guarantee decision and affine candidate are in §7.2; no operational
adoption or implementation follows from that decision.

The IPW lemma concerns the score's mean. Probation and shrinkage read `k_u`, the count of
observed items (`probation.rs:84–110`); the weight is `exp(γ S_u k_u/(k_u + k₀))`, capped; the
CUSUM reads the unweighted observed scores.

- **Observed count.** Its expected increment is `P(enters | F_u) + ε · P(rejected | F_u)`,
  larger on the gate's side even under C1 (`the_observed_count_still_depends_on_the_report`,
  E: 1 against `ε`).
- **Retracted:** the earlier suggestion of `Σ_j I_j/π_j` as a report-independent count. Under C1
  its expected increment is 1 on both paths, but its distribution depends on the path. Exact
  counterexample (`an_ipw_count_does_not_cross_the_threshold_alike`, E, C), with
  `N_PROBATION = 30` and `ε = 0.05`: on the entering path each judgment adds exactly 1; on the
  explored path it adds `1/ε = 20` with probability `ε`, else 0, so 30 is reached on two
  explorations. Probability of reaching 30 within `n` judgments, both means equal to `n`:

  | `n` | entering path | explored path |
  |---|---|---|
  | 10 | 0 | 0.086138 |
  | 30 | 1 | 0.446458 |
  | 60 | 1 | 0.808447 |

  With the observed count the explored path needs 30 explorations: probability below
  `10⁻²⁰` at `n = 60`.
- **Three quantities that are not interchangeable.** The number of real observations
  `Σ I_j` is the evidence available. The number of assigned judgments `N_u` is the exposure.
  `Σ I_j/π_j` is an IPW estimate of `N_u`, not evidence.
- **Then open:** integration and verification of the restricted affine candidate (§7.2); the composed review is in §7.3.
  §7.1's counterexample remains acquired for its declared bridge to the current rules.

### 7.1 A12 finite effective-weight counterexample

**Status.** The owner subsequently approved, for study only, the objective of maximizing
expected cumulative **effective weight available in assigned review opportunities** over a
prefixed finite horizon. Weight here is not causal influence on a verdict. Astra accepted the
counterexample below after independently checking the formulas and states against the code,
the rational scores and companion baselines, Decimal expectations and the rational Taylor
certificate. **A12 remains open; a correction decision is necessary.** No correction,
implementation, campaign or next implementation step is chosen or authorized.

**Declared bridge and horizon.** In this complete-report, A/R-only subdomain, `π = 1` and
each assignment contributes exactly once its
`d(p,b,Y) = (b − Y)² − (p − Y)²` to `SkillTrack::record_observed(d, 1, params)`.
The current cohort of the examined reviewer `u` is a singleton; exposure, denominators
and update order are identical across strategies. A singleton reviewer cohort does not
prescribe single-item empirical validation: the item may belong to its prefixed pilot group.
This is an explicit item-level bridge
for the study, **not an existing production composition of `cohort_scores` into `SkillTrack`**.
The finite horizon is two epochs with one prefixed opportunity for `u` in each. Current
weight is frozen at 1; next-epoch weight uses the updated prior-epoch standing. Thus
`U = E[1 + w_eff,next]`; the next opportunity occurs before its own score update.

**Reachable shared history and roster.** There are 19 reviewers, each with 30 observed
zero-score updates, `reviewed = scored = 30`, both sums and CUSUM zero, and no alarm.
Such a history is reached by complete A/R assignments with reports and baselines
`p = b = 1/2`, yielding zero for either outcome. Bootstrap founders can supply the positive
unit baseline weights before probation ends. After the 30th update all are Established
with raw weight 1, even if their founder membership persists. These are reachable track
states, not authenticated historical records or a reconstructed empirical pilot.

The current first panel consists of `u` and eight other reviewers; the remaining ten
receive no current assignment and retain weight 1. Frozen panel weights are all 1, and
the eight companions each report `1/2`. Hence `u`'s leave-one-out baseline is `b = 1/2`,
independent of `u`'s report and never its own-report fallback. With complete reports and
`c = 1`, the reference terminal law is `P(A | F_u) = q = 4/5`, `P(R | F_u) = 1/5`.
The fixed procedure, outcome law, information and assigned opportunities are unchanged
by the deviation. No extra round, disclosure, delay or change of sample/resources is used.
The terminal is a reference random variable, not a fit executed for this example.

**Actual track transitions.** Compare the sincere report `p = 4/5` with `p = 41/50`.
Defaults are `γ = 35`, `k₀ = 100`, `N_PROBATION = 30`, CUSUM allowance `0.03` and
threshold `1.5`. CUSUM is active on this update and compares the score to the pre-item
reference 0. No alarm fires: its largest statistic for `u` is `0.3924`, below `1.5`.
After the update `reviewed = scored = 31` and `S_u = d/31`; the raw next weight is
`exp(35 (d/31) 31/131) = exp(λd)`, where `λ = 35/131`.

| Report | Terminal | `d_u` | New `S_u` | New CUSUM | Raw next weight |
|---|---|---:|---:|---:|---:|
| `0.80` | R | `−0.39` | `−0.39/31` | `0.36` | `0.901046446027404` |
| `0.80` | A | `0.21` | `0.21/31` | `0` | `1.057710715483027` |
| `0.82` | R | `−0.4224` | `−0.4224/31` | `0.3924` | `0.893280210446096` |
| `0.82` | A | `0.2176` | `0.2176/31` | `0` | `1.059860615408532` |

**Companion updates and real cap.** The report of `u` enters each companion's baseline,
so their scores are updated too; they are not held falsely invariant.

| Report of `u` | Each companion's baseline | Score on R | Score on A | CUSUM on A |
|---|---:|---:|---:|---:|
| `0.80` | `43/80` | `249/6400` | `−231/6400` | `39/6400` |
| `0.82` | `27/50` | `26/625` | `−24/625` | `21/2500` |

Companion CUSUM on R is zero; all eight remain Established with 31 updates and no alarm.
The ten unassigned reviewers remain Established at weight 1 with 30 updates. Consequently
the full 19-member roster's median raw weight is exactly 1 on every branch, and the actual
`orchestrator::epoch_weight_cap` is `3 × 1 = 3`. Every updated weight is below 3
(the largest is about `1.059861`); the cap is slack. Current frozen weights are not
recomputed mid-epoch. Next weights use the full updated roster, not an exogenous cap.

**Expectations and exact sign certificate.** The relevant expressions are

`W_truth = (4/5) exp[(35/131)(21/100)] + (1/5) exp[−(35/131)(39/100)]`,

`W_dev = (4/5) exp[(35/131)(136/625)] + (1/5) exp[−(35/131)(264/625)]`.

| Report | Expected current score | Expected next effective weight | Total utility `1 + E[w_eff,next]` |
|---|---:|---:|---:|
| `0.80` | `0.09` | `1.02637786159190225735` | `2.02637786159190225735` |
| `0.82` | `0.0896` | `1.02654453441604506557` | `2.02654453441604506557` |

The gain is `Δ = W_dev − W_truth = 0.00016667282414280822… > 0`.
For all four exponents `|x| < 1/8`. With
`P₈(x) = Σ_{k=0}^8 x^k/k!`, the absolute exponential remainder is at most
`r = 2/(8⁹ 9!)` (Taylor remainder and `exp(1/8) < 2`). Each expected-weight approximation
has error at most `r`, and their difference at most `2r`. Exact Fraction arithmetic gives
the interval `[0.0001666728240615…, 0.0001666728242258…]` and verifies
`1/6000 < lower ≤ Δ ≤ upper < 1/5900`. The strict gain therefore does not depend on
floating-point printing. The expected-score loss is exactly `(0.82 − 0.80)² = 0.0004`,
as required by A1 with `c = 1`; A1's conditional expected-score closure is preserved.

**Scope and evidence.** One deterministic complete-report deviation is sufficient to refute
the general sincerity claim for this objective, bridge and current parameters; adaptive
strategies include it. It neither refutes every future bridge nor establishes a protocol
attack, end-to-end execution, empirical fit, authenticated history or collusion result.
It needs no conclusion on I/pending, withholding or timing to refute the broader claim;
those semantics remain necessary for any future general incentive specification.
The control holds the other reviewers' behavior fixed and uses no coalition, direct action
in another role, withholding or timing deviation; its utility includes no external value
of the item's fate. These boundaries prove none of those channels harmless and do not
restrict the strategies that a future general guarantee must address.

The code reference is `f5b39b5c6c0526d99f0fabffd6fed6719644d084`:
`scoring/src/reputation.rs::{difference_score,odds_weight,Cusum::observe,weight_cap}`,
`protocol/src/probation.rs::{SkillTrack,status,effective_review_weight}`,
`protocol/src/panel_scores.rs::first_panel_baselines` and
`protocol/src/orchestrator.rs::{epoch_weight_cap,bridging_weights}`.
The mathematical reproducer [`sim/a12_weight_counterexample.py`](../sim/a12_weight_counterexample.py)
uses only the Python standard library: Fraction states/scores/CUSUM and Taylor bounds,
Decimal65 exponential values. It is a checked transcription, **not a Rust test or a
bitwise check of `scoring::fmath`**. The finite calculations were executed; no Rust test,
fit or campaign was run. A2 remains open, its certifier suspended; `19` §12 still awaits
review (at that date; A2's current state: `15`). T26 remains separate. The next outcome is
a correction decision, not an implementation authorization or closure of A12 or Phase 1.
The subsequent domain/guarantee decision is in §7.2; it does not change this
counterexample or its historical judgment.

### 7.2 A12 restricted affine guarantee — owner's decision, 2026-10-12

**Decision and status (at the 2026-10-12 decision; closure review in §7.3).**
The owner adopts the **exact incentive guarantee in an offline domain with complete data and invariant information**, and explicitly relinquishes its use
as a general proof against betrayal. The objective is expected cumulative effective weight
available in prefixed assigned opportunities over a finite horizon, not causal influence on
item decisions or an external betrayal payoff. This is the current Phase 1 requirement
under AT-REP-01, amended explicitly in `08`. The affine mechanism below is a **specified
candidate with an accepted conditional mathematical proof and an exact offline reference**,
not the current Rust odds/reset path or an operationally adopted mechanism. At the
package review A12 remained open; §7.3 records its subsequent conditional closure.
Defining the objective/domain is no longer an outstanding generic prerequisite.

**Inputs and domain.** Before the compared reports, declare a finite horizon `H`, roster
and founder membership, a complete common initial history, group/reference-procedure
associations, assignments and reviewer cohorts, an ordered prefix and update calendar,
and the number `m_e` of the reviewer's opportunities in each epoch. All are ex ante fixed
and identical across strategies. Let `P_{u,e}` be the cumulative assignment prefix whose
scores the epoch reads and `n_{u,e} = |P_{u,e}|`, initial history included. Membership,
denominators and update positions are not selected from ratings, verdicts or arrivals.
Every member has one completed valid report and a positively verified terminal A/R/I;
there is no observation draw (`π = 1`), missing-value imputation or duplicate contribution.
An empirical pilot still uses its prefixed group; a singleton reviewer cohort does not
prescribe singleton empirical validation. The reference procedure and terminal meaning
are those declared for the experiment, with intrinsic I distinct from missing work/resources.

For the guarantee, all required prefixes are complete at their prefixed update positions.
Let `(F_j)` be a **joint exogenous reporting filtration on one probability space for all
compared policies**, containing all legitimately available public/private signals at each
report. The reference terminal variables `Y_j` and the joint signal/disclosure law are
invariant under those policies. Reports may adapt to `F_j`; they may not alter later
information, opportunities, samples, resource access, procedures or completion schedules
within this domain. Reports are frozen before the required terminal disclosure. A
history-based stratified assignment or a supplementary gate is not exogenous merely
because its transcript can be replayed. These are accepted model assumptions, not facts
established by completion, a logged prefix or a fit. Off-log knowledge is not excluded by
declaring it absent. Do not select whichever cohorts finished and call them representative.

**Score, counts and weights.** For `p_j ∈ [0,1]`, define

`r_j(p_j,Y_j) = 1/4 − (p_j − y_j)²` on A/R (`y_j = 1/0`), and `r_j(p_j,I) = 0`.

This is a fixed benchmark, not the crowd difference term `g` of A1/B-b; the current
`cohort_scores` does not implement this bridge. Parameters satisfy `0 < η ≤ 0.8`,
`k₀ ≥ 0`: this is a mathematical domain, **not a calibrated operating point**.
The established review weight is

`w_{u,e} = 1 + η Σ_{j∈P_{u,e}} r_j /(k₀ + n_{u,e})`.

Probation lasts 30 observed, verified terminal assignments A/R/I in the complete model,
not 30 IPW pseudo-observations or necessarily 30 conclusive verdicts. Thus its count
equals `n_{u,e}` in that model. Below 30, a non-founder weighs 0 and a bootstrap founder
1; at 30 everyone reads the affine weight. There is no punitive reset. CUSUM is proposed
as a **diagnostic only**, with no effect on weight, sums, counts, eligibility, assignments
or information available during `H`. Its statistic/reference/threshold contract and
calibration still need specification; the current `SkillTrack` reset is not silently reused.
No denominator is evaluated during probation; established `n ≥ 30` makes it positive
even if `k₀ = 0`. Frozen epoch weights come from the specified preceding complete prefix,
not current reports or a recomputed intra-epoch standing.

Since `−3/4 ≤ r_j ≤ 1/4`, all established weights are in `[0.4, 1.2]` throughout
the parameter domain. Every member of the cap's carrying roster follows this candidate
or the unit bootstrap rule; probationers at 0 are excluded. Its median is at least 0.4,
so `w_max = 3 × median(w) ≥ 1.2` and the cap never changes a candidate weight.
If nobody carries weight, all review weights are 0 and no median affects an established
reviewer. The current odds-weight roster cannot be substituted into this range argument.

**Exact conditional composition proof.** Put `χ_e = 1{n_{u,e} ≥ 30}`. Eligibility,
`m_e`, `P_{u,e}` and `n_{u,e}` are deterministic under the fixed complete calendar.
Founder/probation contributions and the common initial history enter a policy-independent
constant `C`. The cap is slack and the detector has no economic effect, so

`U_H = C + Σ_j β_j r_j`, where
`β_j = η Σ_{e:j∈P_{u,e}, χ_e=1} m_e /(k₀+n_{u,e}) ≥ 0`.

Only denominators from established epochs occur in this sum. In particular `β_j` is
deterministic, not a function of `Y_j`, its availability or the report. Mere independence
from the report would not suffice if future opportunities selected an outcome branch.
Let `c_j = P(Y_j ≠ I | F_j)` and, only for `c_j > 0`,
`q_{c,j} = P(Y_j = A | Y_j ≠ I, F_j)`. Conditional expansion of the squared error gives

`E[r_j(q_{c,j},Y_j) − r_j(p_j,Y_j) | F_j] = c_j (p_j−q_{c,j})²`.

At `c_j = 0`, both scores are 0 almost surely conditionally; `q_{c,j}` is undefined
and the coordinate contributes 0 without choosing a conventional target. Bounded scores
and finite sums permit conditioning term by term. On the common outcome/information
space, for every admitted adaptive policy `σ`,

`E[U_H(truth) − U_H(σ)] = Σ_j E[β_j L_j] ≥ 0`,

where `L_j = c_j(p_j−q_{c,j})²` on `c_j > 0`, and `L_j = 0` on `c_j = 0`
without evaluating an undefined `q_{c,j}`. Thus truthful
reporting maximizes expected cumulative weight. The inequality is strict if a deviation
differs with positive probability on a coordinate with `β_j c_j > 0`; a last report
with no later rewarded opportunity can have `β_j = 0` and is flat. This proves the
mathematical composition **within the declared domain**, not a property of the runtime,
of an outcome-dependent prefix selection or of policy-dependent reporting filtrations.

**Approved binary64 contract (owner, 2026-10-12).** The owner subsequently approves an
explicit numerical qualification: the rational mechanism retains the exact theorem above;
the `f64` consumer receives a bounded incentive guarantee, not exact properness of rounded
weights. All scores, sums, denominators, parameters, probation and the slack-cap computation
remain exact in the offline producer. Only the final effective weight is converted once,
round-to-nearest, ties-to-even, to binary64, and its bits are transferred unchanged to Rust.
Recomputing those quantities in floating point is outside this conversion-only contract.

Write `ŵ_e = RN64(w_e)`, `M_u = Σ_e m_{u,e}`, and `Ũ_H = Σ_e m_{u,e} ŵ_{u,e}`,
where the latter is a **mathematical sum of the consumed binary64 coefficients**, interpreted
as exact dyadic rationals. On `[2/5,6/5]`, binary64 spacing is at most `2⁻⁵²`, hence
`|ŵ_e − w_e| ≤ ε = 2⁻⁵³`; probation 0 and founder 1 are exact. Therefore, pathwise,
`|Ũ_H − U_H| ≤ M_u ε`. The fixed-calendar assumptions give the same deterministic `M_u`
to each admitted policy. Combining both errors with the accepted exact theorem yields

`E[Ũ_H(σ) − Ũ_H(truth)] ≤ 2 M_u ε = M_u · 2⁻⁵²`.

This is the amended numerical AT-REP-01 requirement. It is an a priori bound, not a
confidence interval or calibration threshold. A trace whose conversions happen to be exact
does not justify replacing the uniform policy-comparison bound with zero. The bound covers
neither a floating accumulator of utility nor error in the optimizer/fit, verdicts, causal
influence or the report-triggered supplementary gate. The complete-prefix and invariant
information assumptions, representability caveat for `q_c`, and refusal boundary remain.
Refused or unsupported inputs must not be postselected to infer an unconditional guarantee.

The distinction is necessary even for representable forecasts. With common initial
`n₀=30`, score sum 0, `η=1/2`, `k₀=0`, one report and then one rewarded opportunity,
take `q=3/4` and `p=3/4−33·2⁻⁵³`. The exact weight favors truth by
`1089/5030037581705614265138918318931968`, but the correctly rounded expected weight favors
the deviation by `2⁻⁵⁵`. A's binary64 weight is unchanged; R's increases by one bit-step.
Astra and Sol independently checked this finite construction with rational arithmetic;
it is not an execution of the Rust fit. It does not refute or reopen the exact theorem.

**Incomplete inputs and two acquired obstruction constructions.** The offline candidate
must return `Unavailable`, with reasons, for a required prefix with pending/missing
records, incomplete reports/freezes or unsupported evidence. It returns **no certified
numeric weight for that prefix**: neither a certified hold-old weight, nor 0, nor I.
Later evidence permits a new snapshot, not retroactive replacement of the earlier one.
This refusal is an offline output contract, not a policy assigning operational weight
while pending. No guarantee is obtained by conditioning on completion after the fact.

1. *Hold-old and availability.* Start Established at `n₀ = 30`, sum 0, weight 1;
   `q = 4/5`, `k₀ = 100`. A's record is available before the next opportunity;
   R's arrives after the horizon and then remains genuine R. Reports are complete;
   the availability law is identical across strategies and independent of reports.
   Holding the previous weight on pending gives
   `E[w_next(p)] = 1 + η(4/5)[1/4−(p−1)²]/131`, maximized at `p = 1`, not `q`.
   Its gain over `q` is `η·0.032/131`; with `η = 4/5`, exactly `16/81875`, while
   full expected score loses `1/25`. No reviewer-controlled withholding is needed:
   correlation of availability with the terminal already makes `β` outcome-selective.
2. *Report-triggered disclosure.* On a three-opportunity horizon with the same initial
   history, let `Z ~ Bernoulli(1/2)`, `P(Y₁ = Z | Z) = 3/4`, `Y₂ = Z`.
   The sincere first forecast is `1/2`. A report `1/2+δ` triggers disclosure of an
   expert forecast on `Y₁`, `1/4` or `3/4`, revealing Z. After seeing `Y₁`, truth
   knows `q₂ = 1/4` or `3/4`; the deviation knows `q₂ = 0` or `1`.
   All prefixes finish on time and u's exposures/counts are identical. The first
   expected score loses `δ²`, the second gains `3/16`. The utility gain is
   `η[(3/16)/(k₀+32) − δ²(1/(k₀+31)+1/(k₀+32))]`; with `δ = 1/100`,
   `η = 4/5`, `k₀ = 100`, exactly `122681/108075000 > 0`.
   This is an abstract information channel, not a proposed supplementary gate or a
   fit execution. It breaks the common filtration, not the complete exogenous theorem.

**Test scope and completion.** AT-REP-02/04/05/07 remain valid evidence for their current
components; they are not tests of this candidate. The crowd-neutrality rule becomes
zero at the fixed benchmark `p = 1/2`, not at every live crowd mean (02). The bounded
range/slack cap replaces the 16-to-3 outlier behavior (04). Conditional properness is
preserved mathematically with new score/domain; composed numerical tests were subsequently acquired
(05). A diagnostic detector does not preserve the punitive return to probation (07).
No A1 or A6 correction or other acquired closure is reopened.

The owner withdraws the blanket Phase 1 claim of honesty for causal influence times an
arbitrary betrayal payoff (the original AT-REP-01), **not by proving it**. An arbitrary
external payoff can exceed any bounded weight incentive; an operational betrayal claim
would need a defined payoff, strategy/information scope and mapping to influence actually
consumed by bridging. The threat remains; it is not eliminated or automatically transferred
to Phase 2. The current Phase 1 acceptance criterion is this restricted cumulative-weight
guarantee, and the exclusions above are explicitly accepted for that criterion.

Closure requires a coherent specification and executable offline candidate through the
pertinent paths, composed acceptance/refusal tests and a declared numerical/reproducibility
contract, plus procedures for provisional parameters on the new model. The reference
implementation below covers the exact offline path; the explicit Rust adapters described
below have passed their scoped integration gate on the owner's reported executable checks.
`14` records procedures for `η`, `k₀`, probation, cap and a future diagnostic CUSUM, without
calibrated operating values. The example values are not operating choices and old detector
rates do not characterize the candidate. No characterization report is regenerated here.
A full runtime, strong distributed authentication or multinode campaign is not a
prerequisite merely because this is a trusted local experiment. A2 remains open with its
certifier suspended; `19` §12 awaits review (at that date; A2's current state: `15`); T26 is
separate. The original decision registration implemented no mechanism. The reference step
below starts no campaign or operational deployment.

**Provenance.** Sources were read at published HEAD `45ed426072fcc4e556b7ad91b16d166183104b6f`:
`02` C.2/C.4, `08` AT-REP-01/02/04/05/07, `17` §§14.10, 14.13.6, 14.15 and the
unchanged `reputation`, `probation`, `panel_scores`, `orchestrator` formulas. The general
proof is algebraic. The two finite constructions were checked with exact Fraction
calculations; Astra independently checked them and 32 rational single-score identities
plus the flat `c = 0` branch. This is mathematical design evidence, not Rust/fmath
bitwise verification, end-to-end execution, a fit, a campaign or authenticated-history
reconstruction. The owner's domain/claim decision and accepted conditional proof precede
this documentary registration; implementation and documentation review remain distinct.

**Executable reference path (2026-10-12).** `sim/a12_offline.py` implements a trusted-local
manifest/snapshot interface, not a ledger decoder or an evidence verifier. Its manifest
declares the whole homogeneous roster, initial-history binding, groups/reference procedures,
ordered assignments, epoch cuts/prefixes and assigned opportunities. It requires the domain
assertions explicitly and checks structural consistency, report freezes, terminal
associations and availability at each original cut. It cannot establish the asserted joint
information law, off-log knowledge or the historical truth of supplied timestamps.

For valid complete required prefixes, the executable composes the fixed benchmark,
probation/founder rule, affine weight, carrying-roster median cap and frozen opportunity
weights into cumulative utility. Refusal is atomic over the declared horizon/roster and
contains no numerical weights or utility. Later snapshots cannot repair availability at
an earlier required cut. The JSON schema, canonical output, rational grammar and runnable
examples are specified in [`sim/README.md`](../sim/README.md).

The numerical contract is exact arithmetic on the admitted rational inputs, with no
floating conversion. The real-valued theorem remains mathematical: an irrational `q_c`
has no exact representation in this interface, and no rounded report is certified as its
maximizer. `sim/test_a12_offline.py` checks the composed path, refusal boundary, exact finite
incentive cases and replay determinism. Those tests do not establish invariance or the
general theorem by enumeration. The owner-confirmed guarantee is unchanged.

**Package evidence and review.** On base `cc981a175c49e499d71614a2b27a2c4758d57bd7`,
the executor ran the 29 targeted unittest cases and the comment-budget check; Astra read
the implementation/tests and independently re-ran both on CPython 3.12.14 with the declared
integer-conversion profile. All passed. Astra also checked 96 exact expected-utility gaps
through the executable over `n₀ ∈ {29,30}`, `η ∈ {1/5,4/5}`, `k₀ ∈ {0,100}`,
`c ∈ {1/3,1}`, `q_c ∈ {1/5,4/5}` and reports `0,q_c,1`, with subsequent opportunity
counts 1 and 2, against the accepted composition identity. These are finite checks, not a
campaign or a general proof by sampling. Documentation diff and added links were checked.
**Package verdict: accepted as an exact offline reference**, with the integration boundary
below still open; not acceptance of A12 or Phase 1 as complete.

**Frozen binary64 candidate (2026-10-12), based on `6e0723b341267ba4f40ab24236313023638cd1d7`.**
`sim/export_a12_affine.py` first validates the complete exact horizon, then converts only
the final effective weights. It checks the exact rational conversion error and exports
canonical binary64 bits, exact/dyadic utility diagnostics and conservative numerical
bounds in whole-roster epoch tables. The CLI publishes a complete new output directory;
it refuses an existing destination and leaves no published partial table after a failed
export. Manifest, snapshot, epoch and original cut remain bound to each table. Canonical
Nym IDs are required only at this transport boundary; the accepted exact reference is unchanged.

`protocol::affine_weights::FrozenAffineEpoch` imports a table using a caller-pinned digest
and expected context. It checks the format, ordered unique roster, integer ranges, finite
weights and probation/range constraints. The digest binds trusted-local producer output;
it is not authentication or a proof that an arbitrary externally supplied table was
computed correctly. The importer does not recompute arbitrary rational arithmetic.
`AffineRatings` keeps its base and context private; the explicit
`orchestrator::{affine_first_ratings, affine_supplementary_ratings}` paths use the same
frozen bits for existing and new rows, with no odds, old reset or mixed-roster cap.
They expose ratings to the existing scoring engine without claiming a theorem for its fit.

**Candidate preparation evidence.** Sol ran the 29 reference and 5 exporter tests;
Astra independently re-ran all 34 on CPython 3.12.14 with the declared integer-conversion
limit 4300, plus `scripts/comment_budget.py`; all passed. A preceding reviewer invocation
with the unsupported limit 0 was refused by the numerical-profile guard and was corrected
without changing the implementation. The checks include golden regeneration for both
epochs, byte-identical replay, fixed-context drift, exact/dyadic utility bounds, the finite
rounding counterexample, complete-horizon refusal and no overwrite/partial publication.
Astra read the Rust importer, adapters and five test cases, including reuse of an existing
unrated row, newcomer weights, mixed-epoch refusal and malformed transport. This is static
review evidence only. The frozen golden SHA-256 values are
`5bce1f8250a423c8f782fa276f16f1ac9820eb4e9965acfb2576c5e87f161b95` (epoch 0) and
`ba94ad86e3ef3fce3bb0b8dbfde03d14ac49d886c456bf05fe388aed25b2e47c` (epoch 1).

**Integration package verdict: accepted (2026-10-13).** The owner supplied a Linux
execution transcript: `cargo check -p protocol` completed successfully and
`cargo test -p protocol --test affine_weights` passed all five cases, with none ignored.
The initial `cargo fmt --all -- --check` reported formatting differences in the two new
Rust files. A subsequent executor report supplied by the owner identifies local HEAD
`d395c94bc24079100b76e2cfec21e640301214ca` and records successful `cargo fmt --all`,
`cargo fmt --all -- --check` and `git diff --check`. The reported working tree contains
the two formatted Rust files and the owner's unrelated `.gitignore` change.

Astra read these reports; Rust compilation/tests and rustfmt were **not independently
rerun** in the preparation environment, which lacks the toolchain. For this consolidation,
Sol reproduced the formatting changes from the supplied rustfmt diff against the published
base. Source comparison confirms only layout changes and optional trailing commas, with
no change to expressions, strings, comments, fixtures or the numerical contract. This is
not a claim of direct access to the owner's formatted checkout. `.gitignore` is excluded.
The earlier independently executed 34 Python tests and reviewed composition evidence
remain applicable; formatting and documentation changes require no new mathematical study
or campaign. The consolidation commit retains the tested source behavior and frozen fixtures.

Acceptance concerns the exact producer → frozen binary64 transfer → first/supplementary
ratings package, under the approved local trust and numerical contract. The package verdict
alone did not close A12; the composed closure judgment is in §7.3. Supplementary
opportunities must remain prefixed and information-invariant; adapting a consumer does
not certify the report-triggered gate.
The earlier exact-reference acceptance is not reopened. Diagnostic CUSUM remains absent,
with its procedure/specification dependency in `14`; final-model characterization and
external T26 remain separate gates. Current summary: `15`, A12.

### 7.3 A12 closure review — 2026-10-13

**Verdict: A12 closed conditionally** against the owner's explicitly amended AT-REP-01,
on examined commit `15b846c107518343de7ba0cb68fc7c1d6d4a2382`. The condition is the
complete offline, ex ante fixed calendar/common-filtration domain of §7.2: outcome,
information and opportunities remain invariant across admitted strategies. The objective
is the mathematical sum of consumed effective-weight coefficients. This judgment composes
the exact producer, one final correctly rounded binary64 conversion, frozen epoch table,
and first/supplementary ratings paths; it is not an automatic sum of package approvals.

| Criterion | Evidence | Disposition in the amended domain |
|---|---|---|
| Fixed complete horizon and refusal | `sim/a12_offline.py` manifest/snapshot, required cuts and atomic `Unavailable`; §7.2 proof assumptions | Satisfied for admitted offline inputs; assertions cannot establish the real-world information law. |
| Exact cumulative incentive | §7.2 conditional theorem and exact reference; 96 prior finite exact utility identities | Satisfied conditionally, with flat coordinates where `β_j c_j = 0`. |
| Final numerical contract | `sim/export_a12_affine.py` converts only the final exact weight; §7.2 bound | Satisfied: `abs(ŵ_e−w_e) ≤ 2⁻⁵³`, expected deviation advantage `≤ M·2⁻⁵²`; no floating-accumulator or fit-error claim. |
| Frozen transport and composed consumers | Digest/context checked `FrozenAffineEpoch`; `AffineRatings` and explicit first/supplementary adapters reuse the same bits; five owner-run composed Rust tests | Satisfied for the scoped trusted-local integration, not an authentication or operational adoption claim. |
| Scoped verification and reproducibility | 34 Python tests independently run by Astra in preparation; reviewed Rust source and owner's `cargo check`, five-test and formatting logs; consolidated formatting at the examined commit | Satisfied for this documentary/source-review gate; Rust was not independently rerun by Astra or this review. |
| Provisional parameters | Procedures in `14` for `η`, `k₀`, probation, cap and any future diagnostic | Satisfied as procedures without selected operating values or characterization. |

`diagnostic_policy: none` is the only accepted offline reference behavior:
`sim/a12_offline.py::_validate` rejects another policy. §7.2 proposes CUSUM only as a
diagnostic with no effect on the proof's weights, counts, eligibility, information or
opportunities; its absence therefore does not block this amended A12 closure. This does
not implement or validate a new detector. AT-REP-07/D34 remains evidence for the old
component, not for affine detection or calibration; `14` specifies the procedure before
any future detector is added, without adopting a threshold or reset here.

**Separate remaining gates.** A12 closure does not close Phase 1, certify legacy
odds/reset, report-triggered gates, causal influence or fit, generic betrayal, or
incomplete-data operation. Final-model characterization, the remaining SC-1
evidence/procedures (with provisional values permitted by `14`), A2 and T26 remain
distinct Phase 1 gates, not new A12 requirements or a demand to choose operating
values for A12 closure. The 2026-10-12 owner amendment and §7.2 own the claim; this review checked the published commit and
pertinent source/document hashes, without executing new tests or a campaign. Preparation
evidence above distinguishes Astra's independently executed Python checks from owner-run
Rust checks read in logs; the 96 identities precede this closure review.

## 8. Alternatives under the same hypotheses

The deferred draw scores every observed outcome at the nominal `π`. The audit sample draws each
assigned judgment's item with probability `α` after the reports freeze, independently of them,
and scores only audited outcomes, all at `π = α`.

| | Ideal source (C1 holds) | Current beacon, §6 model |
|---|---|---|
| **Deferred draw** | the theorem holds under H0 and C1–C6; the freeze covers reports, extra round, gate, appeal and `R_u`; every pilot outcome is scored; `k_u` still depends on the report through the path | on a targeted rejected item `π* ∈ [ε^L, 1 − (1 − ε)^L]`, so the nominal IPW is biased there |
| **Audit sample** | C1 needs no freeze of the decision or the appeal (`π = α` is constant); the observed count's increment is `α` whatever the report; outcomes of unaudited items that entered the pilot are observed but not scored | the same selection: on a targeted item `π* ∈ [α^L, 1 − (1 − α)^L]`; the audit does not correct a manipulable source |

What the audit simplifies (no decision or appeal in the freeze, a report-independent evidence
count) comes with its information cost. Scenarios derived from the paper's declared
assumptions (`065-revisions.tex`: 1,000 reviewers, ≈667 items reviewed a month, 9 reviewers an
item, half passing the gate, golden items at 5%, ≈333 pilot slots a month) (C); they are
derivations, not guaranteed times:

- reviews per reviewer per month: `667 × 9 / 1000 ≈ 6.0`;
- deferred draw: `6.0 × (0.5 + 0.5 × 0.05) = 3.15` live scored items, ≈3.45 a month with golden
  items (the ≈3.5 of D35's table);
- audit at `α = 0.05`: ≈0.6 a month with golden items; 30 outcomes in ≈50 months instead of ≈8.7;
- audit matching the deferred draw's rate (`α ≈ 0.525`): ≈175 audited rejections to pilot a
  month against ≈17 explored, about 158 extra slots, ≈47% of 333, if every audited entering
  item's pool-entry pilot is reused, which needs the same group, sample and procedure; with
  validation and audit separate, 333.5 extra, 100% (`17` §4.1).

## 9. Recommendation

**What can be proved today.** Under the current sequence, no protocol-level properness claim:
the counterexample stands. What holds is conditional: for completed reports, the theorem of
§4.3 under H0 and C1–C6. With the commit-reveal beacon, C1 holds only under a behavioral model of the
members: honest reveals counted, fewer than `t` colluders, and no coalition able to choose
among values or to abort acting on exploration preferences. That is an assumption to state,
not a property to prove.

**Recommended sequence.** The deferred draw: the freeze of §5 (reports, extra round, gate,
appeal, `R_u`), the round rule fixed in advance with its fallback, every observed outcome
scored at the nominal `π`, T58 with the no-show rule of §4.4. Its guarantee would be stated as
conditional on the §6 behavioral model until a source with uniqueness, unpredictability and a
same-message fallback exists. Preferred over the audit sample because it keeps every pilot
outcome at the current cost; the audit is not more robust to a manipulable source.

**Blocking premises for implementing it as A1's correction.**

1. C3: a batch contract for the pilot, still to be defined, and C2's policy for missing
   outcomes, still open (`15` A4, residual (a)).
2. C5: decided and implemented (§4.6), design review pending; a production caller is
   still needed.
3. C4: the denominator from the assignment record and a no-show rule.
4. The randomness guarantee: a conditional statement under an explicit member model, or a
   source change.

**Smallest next verifiable step.** C5 was that step (§4.6: decided, implemented and
approved on `e8fdbe7`). C3's batch contract and C2's missing outcomes are analyzed in
[`17`](17-a1-pilot-batch-design.md), approved on `37addca` as a conditional analysis, with no
batching policy approved; C4's denominator waits on decisions not yet taken.

**Trade-offs that need the owner.**

1. Accept an exploration-score guarantee conditional on a stated behavioral model of the
   consortium's members, or withhold that guarantee until a source with uniqueness, a real
   DKG and a same-message fallback exists.
2. Deferred draw (more evidence; the appeal in the freeze, so later measurement of polarized
   rejections) against the audit sample (simpler dependence, a report-independent evidence
   count; on the declared scenarios, with full compatible reuse, about a sixth of the scored
   outcomes at equal cost, or about 47% more pilot capacity at an equal expected number of
   scored outcomes, 100% without reuse; an equal expected count is not an equal precision,
   `17` §4.1).
3. Whether closing A1 requires the incentives of the reputation actually used (`k_u`,
   shrinkage, weight, cap, CUSUM; §7) or only the IPW objective.

The owner subsequently resolved item 3 as O3 after `a14364c`; the decision and A12's
remaining requirement are recorded only in `17` §14.14; approved on `af42035`. O2 is
subsequently chosen as conditional A; decision and Astra’s restricted A1 closure on
`1bd865c` in `17` §14.15. This dossier’s earlier review entries remain historical.

A10 remains separate. Related, outside A1: golden-item placement also derives from the public
beacon (`honeypot.rs:14–25`) and a golden item is absent from the public deposit set, so golden
and live items may be distinguishable; to be tracked separately if confirmed.
