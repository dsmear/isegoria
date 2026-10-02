# 16 — A1: exploration and the evaluator score's incentive guarantee

| | |
|---|---|
| **Status** | Analysis and proposal; **A1 open** (`15`). Diagnosis confirmed; the updated analysis of the beacon's manipulability and of observations, assignments and the IPW count approved; the theorem and the overall proposal **not approved**. The band baselines (C5, §4.6) are decided by the review and implemented: implemented and verified, design review pending. The paper is not rewritten; `01` D33 carries a dated refinement and `02` §C.2 the band baselines. |
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
`d̄_uj = E[d_uj | F_Φ]`. The estimator is conditionally unbiased exactly when C1 holds.

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
| C2 | an entering item yields its outcome | partial: a pilot without enough respondents is refused, leaving the item in `Pilot1`; a stage-2 fit that does not converge has no representation (`15` A4, residual (a)) | a policy for missing outcomes, still open. Removing an inconclusive pilot from every panelist's `R_u` would not settle the selection and can contradict C4; it is not offered as a solution | to be defined with the policy |
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
to a statement about `E[Ŝ_u]` under withholding needs C1 and C2 relative to `F'_u` (the draw
independent of what `u` knows at the reveal decision), so that a revealed judgment's expected
contribution is `E[d | F'_u] ≥ −1`; a fixed denominator (C4); and the deviation leaving the
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
- **Open:** the convex weight, the cap and the CUSUM's selection. No claim on the size of their
  effects is made without a quantitative check.

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
  month against ≈17 explored, about 158 extra slots, ≈47% of 333.

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
verified, design review pending). Next: define C3's batch contract, that is, which items a
batch may hold and how batches form so that an item's outcome depends neither on its own
path nor on other items' paths. C2's missing-outcome policy and C4's denominator wait on
decisions not yet taken.

**Trade-offs that need the owner.**

1. Accept an exploration-score guarantee conditional on a stated behavioral model of the
   consortium's members, or withhold that guarantee until a source with uniqueness, a real
   DKG and a same-message fallback exists.
2. Deferred draw (more evidence; the appeal in the freeze, so later measurement of polarized
   rejections) against the audit sample (simpler dependence, a report-independent evidence
   count; on the declared scenarios about a sixth of the scored outcomes at equal cost, or
   about 47% more pilot capacity at equal evidence).
3. Whether closing A1 requires the incentives of the reputation actually used (`k_u`,
   shrinkage, weight, cap, CUSUM; §7) or only the IPW objective.

A10 remains separate. Related, outside A1: golden-item placement also derives from the public
beacon (`honeypot.rs:14–25`) and a golden item is absent from the public deposit set, so golden
and live items may be distinguishable; to be tracked separately if confirmed.
