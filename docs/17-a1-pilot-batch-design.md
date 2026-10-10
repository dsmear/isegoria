# 17 — A1: pilot batches (C3) and missing outcomes (C2)

| | |
|---|---|
| **Status** | Analysis. Astra's review of `164fff6` did not approve it; on `37addca` Astra approved the revised mathematical argument and the comparison of alternatives **as a conditional analysis** (below). **No batching policy is approved for implementation**; A1 stays open (`15`). Nothing here is implemented; no pilot, batching or missing-outcome policy changes; no audit or new batching is approved for implementation. §7 checks a candidate of Astra's (a three-category reference outcome): on `654dbff` Astra approved its derivations and calculations as conditional results; on `825cee3` Astra accepted §7 **as a conditional analysis, not as a protocol approved for implementation**. §8 specifies B-b as a candidate contract: on `44f0dbd` Astra approved its identities and distinctions as conditional results, **not §8 as a whole, the protocol, the implementation or the adoption of A**; §8 was rectified after that review in `fa11791`, and on `fa11791` Astra approved §8 **as a conditional analysis, with no blocking finding — not the protocol, the implementation or the adoption of A**. The decision synthesis built on §§7–8 is in [`19`](19-a2-dtf-composition-design.md) §6.1. §9 specifies cases 5–7 for a study under design A, on the direction of Astra's review of `26593e7` (`19` §10): partly approved by Astra on `d31e9fa`, with 6c and 7a decided as the basis of the study's reporting, rectified in `b2a1c2f` and **approved by Astra on `b2a1c2f` as the study's reporting specification and a conditional analysis**. §10, on the bridging input under B-b, was partly approved by Astra on `ac06d3c`, rectified in `111e595` and **approved there as a conditional analysis**. §11, on timing under design A, was partly approved by Astra on `82bdb9a`, rectified in `cf5575a` and **approved there as a conditional comparison between T1 and T2**; T1 and T2 stay alternatives, neither adopted. §12, on access and disclosure per object, was partly approved by Astra on `8af257d`, rectified in `7d8db08` and **approved there as a conditional analysis of the informational requirements**. §13, the observable evidence for §12.4's (ii) and (iv) under T1, was partly approved by Astra on `f38f8a4`, rectified in `176dd8c` and **approved there as a conditional analysis of the observable evidence**, within its stated limits and with three precisions made in §§13.1–13.4; T1 and T2 stay alternatives, neither adopted. Astra's review of `cbcbc67` approved those precisions and the further changes to §13.2's classification (§13); the minimal perimeter for a study S2 built on §§9–13, [`19`](19-a2-dtf-composition-design.md) §10.6, was partly approved there, rectified and **approved on `5af22b9` as a preparatory synthesis**; after that review §13.2's absent-precondition row was made self-standing, its classification unchanged. The synthetic-event technical check built on §§8–13 is specified in [`20`](20-s2-synthetic-check.md), **approved by Astra on `c76c035` as the check's specification**, with four precisions made there; its implementation in test code, partly reviewed on `e5e0898`, was rectified and **approved on `3559396` within the check's perimeter** (`20` §9). §14 proposes the contract of A1's candidate guarantee, on Astra's direction after the review of `bef891b` (B-b the reference score, A the analytic reference, C the comparison, neither adopted): partly reviewed by Astra on `11deec2`, which approved its mathematical core **as a conditional result** and not §14 as a whole; its adaptive verification criterion and the absent baseline were rectified in place in `d2be7da`, and §14 **approved by Astra on `d2be7da` as a conditional candidate contract** (no protocol, adoption of A or C, substantive narrowing or closure of A1). The absent baseline's correction in `20`'s check and §14.6's verification on finite constructions, in test code (`20` §§10–11), were **approved by Astra on `d7a4449`** within that perimeter. §14.8's decision proposal on A1's path was partly reviewed by Astra on `23283f2`, which approved the record of that approval and accepted B-b's scorer as the next intervention, and rectified in place; **its rectifications were approved by Astra on `7d1b19e`**. B-b's scorer, implemented in `protocol` with no production caller (§14.9), was partly reviewed on `7d1b19e`, which accepted three of its contract's choices and did not approve it for a numeric defect on its declared domain, and on `0ca81e9`; corrected, **it was approved by Astra on `6b9f42a` as an isolated component**, within §14.9's contract and numeric limits. The contract of its future caller, §14.10, was partly reviewed by Astra on `3e34f6f`, which approved the record of the scorer's approval and accepted the direction of three of the contract's choices for the isolated component, not the section; rectified in place, it was **approved by Astra on `8dc7087` as the specification of the isolated snapshot component**, with one precision on the digest and the prefixes. That component, implemented with no production caller (§14.11), was **partly reviewed on `9c7cb23`, not approved for two findings**; corrected, **it awaits review**; nothing is adopted. |
| **Baseline** | `docs/phase1-review-alignment`; first committed at `164fff6`, revised in `37addca`. Code line references are to `164fff6`, whose code is that of `e8fdbe7`. The corrections of `15` A11 (closed on `dd842d6`) and of A4's residual (a) (closed on `af17eb3`), made after `37addca`, change how the two pilot stages read a fit that did not converge; the passages describing them say so. §7 is written on `81cd467` and rectified on `654dbff`; §8 is written on `825cee3` and rectified on `44f0dbd`; both name code by symbol. |
| **Scope** | Conditions C2 and C3 of [`16`](16-a1-incentive-design.md) §4.2 (this dossier's conditions, not the review findings of the same names in `15`): what outcome a reviewer's report predicts, how a pilot batch must be formed for that outcome to be common to the paths, and what to do when no outcome arrives. |
| **Evidence** | **L** read in the source; **D** derived here; **C** calculated with rational arithmetic, formulas given in place; **E-fit** a Rust test that runs real fits; **E-api** a Rust test that composes the APIs on inputs built by hand, no fit run (`crates/protocol/tests/a1_batch_composition.rs`; for A11, `indeterminate_screen.rs`). No runtime composes the pilot (§1): nothing here describes runtime behavior. |

## Design review (Astra)

Review on `164fff6`:

- Evidence: reading of the diff, the code and the tests, with independent recalculations; the
  Rust tests were not re-run.
- Not approved: this dossier as committed, and the group audit (§4, C). A1 stays open.
- Confirmed: a stage 1 that does not converge gives `false`, which composes into an observed
  screen rejection (outcome 0). A contract defect, recorded as `15` A11. The screening test
  builds `MaxIters` by hand; it runs no fit that fails to converge.
- To correct: the batch comparison must pin `Dif` against `NoDif`, not only `assert_ne!`; §3
  wrongly required equal outcome laws across the two paths' selected populations (abstract
  counterexample, §3); the general exclusion of dynamic batching and of fixed references is
  not demonstrated; the audit's costs must separate reuse from a double pilot; a comparison
  with prefixed groups activated by an ordinary entry or by a draw is missing.
- Recalculated on `N = 667`, `g = 0.5`, `ε = 0.05`: the current reference 350.175 slots a
  month; the audit at `α = 0.525` with full compatible reuse 508.5875 (increment 158.4125);
  validation and audit separate 683.675 (increment 333.5). The counterexample's
  `E[I d/π] = −(p − ½)²`.
- No capacity decision is asked of the owner.

This revision verifies those numbers (§4, C) and answers each point in §§2–6.

Review on `37addca`:

- Evidence: reading of the diff, the code and the tests, with independent recalculations; the
  Rust tests were not re-run.
- Approved as a conditional analysis: the revised mathematical argument (§3) and the
  comparison of the alternatives (§4). No batching policy is approved for implementation; A1
  stays open.
- The historical screen study's provenance checked by comparing the code since `ec34fa4`; its
  percentages read in the historical results, neither recalculated by Astra nor re-run; the
  study's individual records are not kept.
- Non-blocking correction, made in §4.1: sampling by groups does not always lower the
  precision of the IPW score; the variance formulas assume draws independent across groups and
  fixed assignments.

## 1. The path from the pilot to the scored outcome

R = runtime the lifecycle machine runs; A = API with no production caller; I = documented
intention; — = no policy. As for A5–A7, no epoch driver composes these steps: the pilot
functions are called only by tests and by the characterization harness (`characterization/src/run.rs:145–452`), which is measurement, not runtime (L).

| # | Step | Code | Status |
|---|---|---|---|
| 1 | Entry: `Pilot1 { appealed }` after a pass, a band pass or an appeal; `Explored` after an exploration draw | `lifecycle.rs` (`Score`, `Resolve`, `Appeal`, `Explore`) | R |
| 2 | Batch formation: which items, when, how many, with which anchors and respondents | none; `05` [6]–[7] requires batches, never a single item, templates apart | — (I for the two requirements) |
| 3 | Stage-1 admission: at least `N1_MIN = 300` admitted respondents, one row each | `pilot::screen`, `pilot.rs:20,211–226` | A |
| 4 | Stage-1 fit and verdict: point-biserial `r_pbis` on the anchor total; items with `r_pbis ≥ 0.20` fitted jointly in a one-class model, shape held normal; kept iff converged, `r_pbis ≥ 0.20`, `a ≥ 0.6`, `\|b\| ≤ 2.5`, floor within 0.10 of chance; at `164fff6` `screen` returned the verdicts without the fit's status, `false` for every item of a fit that did not converge; since the A11 correction every such item reads `Screening::Indeterminate` | `stage1_fit`, `stage1_verdicts`, `pilot.rs:274–355`; cuts `irt.rs:5–12` | A |
| 5 | Stage-1 lifecycle: too few respondents is refused (`NotEnoughRespondents`, the item stays in `Pilot1`); `passed = false` gives `Rejected(Screen)`, at `164fff6` also when the fit did not converge; since the A11 correction an indeterminate screen keeps `Pilot1` (`15` A11) | `lifecycle.rs:485–499`; booleans from `ItemVerdicts` (`orchestrator.rs:99–111,294`) | R, caller's inputs |
| 6 | Stage-2 admission and fit: at least `K_MIN = 2` items, `N_LATENT_MIN = 3000` respondents, a format per column, no template twice, anchors' KR-20 at least 0.90; one-class fit, then mixtures by BIC, only a converged candidate replaces the one-class fit, the selected fit's status reported, classes under 5% share ignored in the gaps | `revalidation.rs:15,101–157`; `latent.rs:911–1034` (`:1022`, `:1031`, `:1034`) | A |
| 7 | Stage-2 reading: `Dif`, `NoDif`, or `Indeterminate` when the selected fit did not converge | `revalidation::target_rechecks` (`15` A4) | A |
| 8 | Stage-2 lifecycle: `batch_size < K_MIN` refused (`BatchTooSmall`); `passed` gives `ActivePool`; else `source_verified` gives `Contested`; else `Rejected(Dif)`. At `164fff6` no input for an indeterminate verdict; since the correction of `15` A4's residual (a), closed on `af17eb3`, the event takes the `Recheck` and an indeterminate one keeps `Pilot2` | `lifecycle.rs:501–518` | R, caller's inputs |
| 9 | The explored path: the same two events, ending in `Measured { passed: passed ‖ source_verified }`; `passed = false` at stage 1 gives `Measured { passed: false }`; since the A11 correction an indeterminate screen keeps `Explored` | `lifecycle.rs:533–576` | R |
| 10 | Outcome for scoring: pool or contested 1, screen or DIF rejection 0, `Measured` at `π = ε` | `exploration::outcome_of`, `exploration.rs:39–58` | R |

## 2. What a verdict depends on

| Element | Stage 1 | Stage 2 | Evidence |
|---|---|---|---|
| Respondents | the sample, and its size against `N1_MIN` | the sample, and its size against `N_LATENT_MIN` | L |
| Anchors | `r_pbis` reads the anchor total; the fit uses the anchors | θ is integrated over the anchors; KR-20 gate | L |
| Other trial items | fitted jointly with the item; one convergence status for the whole fit | the mixture is fitted over all trial items; class selection and the item's gap between classes depend on them | L; E-fit below |
| Formats and templates | each format sets a floor | the same; a shared template is refused | L |
| Convergence | batch-level: if the fit does not converge, no item is kept (at `164fff6`), no item is decided (since `15` A11) | batch-level: `Indeterminate` for every item | L; E-api below |
| Class selection | — (one class) | BIC over the batch; a class under 5% share does not define gaps | L |
| Screening | — | stage 2 runs on stage-1 survivors (`05` [7]), so its batch depends on other items' screens | I |
| Source verified | — | turns a DIF failure into `Contested` (an input; the check, T68, is not in code) | L |
| Entry path | no function reads it | no function reads it | L |

That no function reads the path does not mean the verdict does not depend on it. The path
decides whether an item is piloted and, under any batching that draws from current
entrants, with which companions. Four kinds of evidence, kept apart:

- **Two real fits compared** (E-fit, `a_target_s_verdict_depends_on_its_batch`, `calibration`
  feature). One simulated population of 3,000 respondents and 60 anchors; one target whose
  answers are the same in both batches; both batches admitted by `latent_batch`'s gates. With
  three leaning companions and four clean ones the target reads `Recheck::Dif`; with seven
  clean companions, `Recheck::NoDif`. The test now asserts these two values (it asserted only
  that they differ, which a difference involving `Indeterminate` would also have satisfied).
  Run on this revision: passed, 65.97 s; the run on `164fff6` reported the same two values. It
  shows that in this realization, with companions chosen by hand, the verdict is sensitive to
  the companions. It shows no frequency, no deviation of a reviewer producing such a change of
  companions, no change in the verdict's distribution, and nothing about a batching rule.
- **The screening composition** (E-api, `an_unconverged_screen_composes_into_outcome_zero` at
  `37addca`). A `Stage1Fit` with status `MaxIters`, built by hand, through `stage1_verdicts`,
  `step` and `outcome_of`: on the entering path `Rejected(Screen)` and an observed outcome 0 at
  `π = 1`; on the explored path `Measured { passed: false }` and an observed outcome 0 at
  `π = ε`. No fit runs. The A11 correction turns it into an acceptance test of the new contract
  (`indeterminate_screen.rs::an_unconverged_screen_leaves_both_paths_pending`): both paths stay
  pending. That real stage-1 fits fail to converge is recorded apart,
  on simulated pilots, by the screen study run on the harness of `ec34fa4`: 98.9% of its fits
  converged, 85.5% with true/false items at N = 300 and 60 anchors (`13` §8.7.5; L, not
  re-run). Since `ec34fa4` the lock file, the toolchain, `protocol::pilot` and the fit of
  `scoring::latent` are unchanged, and the harness changed only in recording DIF flags and in
  the screen summary's tests; the rates are those of that simulated design, not of field use.
  The defect is `15` A11 (high).
- **Abstract counterexamples** (D, checked by C): §3's on the paths' outcome laws and `16`
  §4.2's on joint invariance. Statements about the mathematics, not models of the protocol.
- **Runtime**: none. No driver composes the pilot into the lifecycle (§1).

## 3. The contract

**Reference outcome.** `o_j` is the verdict of a specified two-stage procedure applied to a
batch `G_j` that contains `j`, with anchors and a respondent sample drawn by a rule fixed
independently of the reports and of the paths. A reviewer's report forecasts
`P(o_j = 1 | F_u)` for that specified procedure. Without a specified `G_j`-rule there is no
outcome to forecast: "the verdict" of a dynamic batch is a different quantity in each batch.

Properties, kept apart:

1. **Pathwise agreement** (strong C3): on every realization the outcome observed through the
   path taken equals `o_j`.
2. **Conditional agreement on each path's event** (what the lemma of `16` §4.3 uses; below).
3. **Joint invariance** (C6): the conditional joint law of `(b_uj, o_j)` given `F_u` is the same
   under every `σ_u`. It concerns the reference pair under `u`'s deviations, not the paths;
   with C5, (1) gives it by the construction of `16` §4.2 when `u`'s report cannot change `o_j`.
4. **Independence of the draw**: on `{D_j = rejected}`, `P(X_j = 1 | F_Φ) = ε` (C1), and `X_j`
   independent of `o_j` and of the observed outcome given `F_Φ` (the lemma of `16` §4.3).
5. **Availability after inclusion** (C2): an included item yields its outcome.

**What (2) is, and what it is not.** The earlier version required the outcome laws, on each
path's event, to be that of one reference pair. That is wrong. The paths select populations:
the gate is meant to send items with `o_j = 1` to entry more often than to rejection, so the
law of `o_j` on `{D_j = enters}` and on `{D_j = rejected}` differ, and the identity never
compares them. It compares, on each path's event and under the same conditioning, the
observed outcome with the reference one.

*Counterexample to the earlier requirement* (Astra; D, C). `o` equiprobable on `{0, 1}`;
baseline `b = ½`; ordinary entry exactly when `o = 1` (`π = 1`); when `o = 0`, an exploration
draw independent of everything, of probability `ε` (`π = ε`); the observed outcome is `o` on
both paths. The entering path's outcome is always 1, the explored path's always 0. Yet

`E[I d/π] = ½ d(1) + ½ (ε/ε) d(0) = ½[¼ − (1 − p)²] + ½[¼ − p²] = −(p − ½)²`,

which is `E[d(o)]`, uniquely maximized at `p = ½ = P(o = 1)`. Checked exactly at five reports
for three values of `ε` (both sides are quadratics in `p`). The path reads `o` directly, not
through reports: the example shows the earlier condition is not necessary, not that the
protocol meets any condition.

*The argument* (D). Fix `j ∈ R_u` and a strategy `σ_u`. The report `p = p_uj`, the baseline
`b = b_uj` and the decision `D_j` are `F_Φ`-measurable. For `O ∈ {0, 1}`, `O² = O`, so

`d(O) = (b − O)² − (p − O)² = b² − p² + 2(p − b) O`,

affine in `O` with `F_Φ`-measurable coefficients. With `O^e` the outcome observed on entry and
`O^x` the one observed after exploration, (2) is

- (2e) `E[O^e | F_Φ] = E[o_j | F_Φ]` on `{D_j = enters}`;
- (2x) `E[X_j O^x | F_Φ] = ε E[o_j | F_Φ]` on `{D_j = rejected}`, which under (4) is
  `E[O^x | F_Φ] = E[o_j | F_Φ]` there.

With C2 (`I_j = 1` on entry) and the affinity,
`E[I_j d(O)/π_j | F_Φ] = 1{enters} E[d(O^e) | F_Φ] + 1{rejected} ε⁻¹ E[X_j d(O^x) | F_Φ] = E[d(o_j) | F_Φ]`
on both events, and with H0 (`F_u ⊆ F_Φ`) the tower property gives
`E[I_j d/π_j | F_u] = E[d(o_j) | F_u]`. Both sides condition on `F_Φ`, which contains `D_j`:
each path's observed outcome is compared with the reference outcome on that path's event,
never with the other path's population nor with a law that ignores the path. (1) gives (2)
under (4); (2) can hold without (1), for instance when the observed verdict is a replicate of
the reference procedure, independent of it given `F_Φ`. (2) must hold under every strategy of
`u`, the other actors' strategies fixed (their reactions included, below). Then
`E[d(o_j) | F_u] = E[(b − o_j)² | F_u] − (p − q)² − q(1 − q)` with `q = P(o_j = 1 | F_u)`, and (3)
makes the first term and `q` the same under every `σ_u`: the theorem of `16` §4.3, whose
hypotheses (H0, C1–C6) are conditions, not properties of the protocol.

**Selection of populations, against a change of outcome caused by the deviation.** The first
is admissible: which items take which path may depend on their outcomes, through the reports;
the counterexample is the extreme case. The second is what (2) and (3) exclude: `u`'s deviation
changing, for the same item, the observed outcome against `o_j` (breaking (2)) or the reference
pair's law (breaking (3)). A batch whose companions depend on which items entered is how a
deviation could do so: `u`'s report moves one item's entry, hence another item's companions,
hence possibly its verdict. §2 shows the sensitivity to companions; no deviation was run.

**The deviation that matters is `u`'s, all of it.** A reviewer changes several reports, on
items that may share a batch; each report can change whether its item is piloted, hence the
population of a batch that another of its items is in. (2) and (3) must hold for `u`'s joint
deviation, for every `k ∈ R_u`. Other actors' strategies are held fixed: their inputs fixed,
but their reactions to published information (an author appealing after the gate, an extra
reviewer reporting after first-panel reveals) are part of the environment's response to `u`'s
deviation. An appeal that responds to a gate outcome `u` changed can change which items are
piloted.

**Dependencies under which (1) and (3) follow by construction.** The item's own content;
respondents and anchors drawn independently of reports and paths; companions chosen by a rule
fixed before the reports and blind to paths; the procedure's own randomness; companions'
stage-1 survival when the companions are fixed (it is part of the procedure on a fixed batch).

**Dependencies the construction does not cover.** Companions drawn from current entrants
(their entry depends on gate decisions, hence on reports, `u`'s included); batch timing or
population that depends on paths; a stopping or retry rule that differs by path. Under these
neither (1) nor (3) follows by construction; whether (2) or (3) actually fails needs an
admissible deviation and the verdict's distribution (§4, dynamic batching). Two further cases:

- Piloting only the selected members of a group: the verdict on `G ∩ entrants` is a different
  quantity from the verdict on `G`, so (1) fails against a reference defined on `G`.
- Reading a missing outcome as 0: applied on both paths, as stage 1 did before the A11
  correction (§2), it need not
  break (1), but it changes the quantity forecast to "kept, and the batch's fit converged",
  whose convergence part belongs to the batch, not to the item (§2). `15` A11.

## 4. Alternatives

None is approved and none is recommended here (§6). Costs on a common scenario are in §4.1.

**A — fixed groups, all piloted.** Groups of `K` admitted items fixed at admission, before any
report, by a rule on the admitted set alone (templates apart). Every group is piloted in full
through both stages; every item is observed (`π = 1`); no draw.

- (1) by construction: the reference outcome is the verdict on the fixed group, and every
  member is measured. (3): groups, anchors and samples are independent of reports, so
  `(b, o)` is pathwise invariant under `u`'s deviations (with C5). (4) is vacuous: there is no
  draw, so the beacon's selection residual (`16` §6) does not reach scoring. The outcomes must
  not be known to reviewers before the reports freeze.
- Band and appeals change pool entry only; scoring uses every outcome; the observed count
  grows by 1 per judgment whatever the report.
- Cost: 667 slots a month, 316.825 over the reference, 95.0% of the declared capacity (§4.1).
- Changes: a group record at admission; piloting every rejected item for measurement, never
  for pool entry; C2 still open.

**B — each target against a fixed reference set.** Each piloted item fitted with references
validated earlier and chosen independently of current admissions, one fit per target;
respondents may answer several targets and the references in one sitting.

- (1) by construction, if references and samples are fixed independently of reports and paths.
  Respondent cost can stay flat; fits multiply by the number of targets.
- **Not retained here, not excluded.** Its risk: items that lean together are never fitted
  together, and historical simulations show weak detection of an isolated biased item (a
  question of power, not of identifiability: `15` A8). The executed comparison is one
  realization of that risk: beside clean companions the leaning target reads `NoDif`. It
  measures no detection rate of B and does not cover every fixed-reference design. Excluding B
  would need its detection rate on campaigns against the group designs, not measured.

**C — group audit for scoring.** Groups fixed at admission as in A. After the reports freeze
(every report and the denominator `R_u`; the decision and the appeal are not needed), a draw
independent of the reports selects each group with probability `α`. A selected group is
piloted in full; only selected groups' outcomes are scored, at `π = α`. Pool entry keeps its
current path (gate, band, appeal, pilot) and its outcomes are not scored.

- (1) and (3) for scored outcomes by construction: fixed group, full measurement, a draw blind
  to reports and paths. (4) holds by construction for an ideal source; the beacon's selection
  residual of `16` §6 applies unchanged.
- **Reuse is a condition, not a given.** An entering item in a selected group can take its
  pool-entry verdict from the group's pilot only if its pool-entry procedure is the reference
  one: the same fixed group, the same sampling rule for respondents and anchors, the same two
  stages with the same retry and missing-outcome policy, at a time that does not change the
  sample's law. Sharing the item is not enough: a pool-entry pilot in another batch gives a
  different verdict (§2). Without reuse the item is piloted twice, with two verdicts; only the
  group's is scored. With reuse, an entering item's pool-entry batch depends on whether its
  group was selected: a consequence for pool entry, not for scoring, still to be decided.
- Exploration (D35) is not needed for scoring; the gate's false-negative rate comes from the
  selected groups' rejected members.
- The observed count grows by `α` per judgment whatever the report (`16` §7). An equal expected
  count is not an equal precision (§4.1).
- Changes: the group record at admission; a group draw after the freeze; piloting a selected
  group in full; scoring only selected groups; C2 still open.

**D — prefixed groups, activated by an ordinary entry or by a draw.** Groups of `K` admitted
items fixed before any report, as in A. A group's freeze `Φ_G` is the last of its members'
freezes (`16` §5: reports and extra round, decision and re-decision, appeal filed or expired).
After `Φ_G`: if at least one member enters ordinarily (`A_G = 1`), the whole group is piloted;
otherwise a draw `X_G` of probability `ε` selects it. Every member of an observed group is
scored, at `π = 1` when `A_G = 1` and `π = ε` when `A_G = 0`. Pool entry keeps its path: a
member that entered ordinarily can enter the pool on its verdict; a member rejected at the gate
is piloted for measurement only. Entry into the pool and observation for scoring stay distinct.

- *IPW identity* (D; illustrated by C). For `j ∈ G`, `I_j = A_G + (1 − A_G) X_G` and
  `π_j = A_G + (1 − A_G) ε`, with `A_G` `F_{Φ_G}`-measurable. If `P(X_G = 1 | F_{Φ_G}) = ε` and
  `X_G` is independent of the group's outcomes given `F_{Φ_G}`, then
  `E[I_j/π_j | F_{Φ_G}, o] = A_G + (1 − A_G) ε/ε = 1`, so `E[I_j d_uj/π_j | F_{Φ_G}] = E[d_uj | F_{Φ_G}]`,
  and §3's argument runs with `Φ_G` in place of `Φ_j` (H0 at `Φ_G`). `u`'s report on one member
  changes `A_G`, hence the other members' `π`; the identity conditions on `A_G` and holds. An
  exact enumeration on a two-item group, with an arbitrary joint law of outcomes and entries
  and reports fixed before the draw, gives `E[Σ I d/π] = E[Σ d] = −1/25`; the argument is the
  proof.
- *Conditions on the reference.* `o_j` is `j`'s verdict in its full group under the reference
  procedure. (1) needs the group, the respondents' and anchors' sampling, the two stages, the
  retry and the missing-outcome policy to be the same whether the group was activated by an
  entry or by a draw, and a timing that does not change the sample's law: a drawn group waits
  for the beacon round after `Φ_G`, an activated one could start at `Φ_G`. Then `u`'s reports
  move only `A_G`, not `o_j`, and with C5 (3) follows by construction. C2 stays open: an
  observed group can still yield no outcome (a fit that does not converge; fewer than `K_MIN`
  stage-1 survivors for stage 2).
- *Freeze and draw.* No member's reviewer or author may know `X_G` before `Φ_G`; the group
  waits for its slowest member (appeal windows); the beacon's selection residual (`16` §6)
  applies to `X_G`.
- *Effects on the APIs* (none implemented): a group record at admission; a group freeze
  record; one draw per non-activated group under a new tag; a transition that pilots a
  gate-rejected member of an activated group for measurement at `π = 1` (today `Explore`
  requires a beacon seed and `outcome_of` reads every `Measured` at `π = ε`), so a state must
  carry its inclusion probability; group pilot events composed into the members'
  `Pilot1Batch`/`Pilot2Batch`; entering items wait in `Pilot1` for their group's freeze, or are
  piloted apart for entry (a double pilot); `FalseNegatives` would need weights `1/π`, since
  rejections measured in activated groups are no random sample of rejections.
- *Observed count.* Its expected increment on `j` is `P(A_G = 1 | F_u) + ε P(A_G = 0 | F_u)`:
  it depends on `u`'s report on `j`, and on its reports on other members it reviews.
- *Observed share* (C, illustrative). If entries were independent with probability `g`, a group
  would be observed with probability `1 − (1 − ε)(1 − g)^K`: with `g = ½`, `ε = 0.05`, 0.525 at
  `K = 1` (the current reference), 0.7625 at 2, 0.88125 at 3, 0.940625 at 4, 0.99628906 at 8.
  Independence is an assumption for the illustration, not a fact of the project; entries can be
  correlated within a group (a campaign, a shared topic or author). Given only `g`,
  `P(no member enters)` lies between `max(0, 1 − Kg)` and `1 − g` (Fréchet bounds), so the
  observed share lies anywhere in `[g + (1 − g)ε, 1 − (1 − ε) max(0, 1 − Kg)]`: for `g = ½` and
  `K ≥ 2`, from 0.525 to 1, from the current reference to the universal pilot. D's cost needs
  the joint law of entries within groups, not `g`.

**Dynamic batching** (the third prompt) is not assessed as a candidate, and not excluded. A
batch drawn from current entrants has a population that depends on gate decisions, hence on
reports, `u`'s included, so (1) and (3) do not follow by construction. The executed comparison
shows that one target's verdict can change with its companions, in one realization, with
companions chosen by hand and no deviation run. Excluding dynamic batching would need an
admissible deviation of `u` that changes a scored item's companions with positive probability
and, under it, a change in the conditional law of the verdict (with the baseline) given `F_u`;
retaining it would need that law's invariance. Neither is done here.

### 4.1 Costs on the declared scenario

The paper's assumptions (`065-revisions.tex:229–231`, as in `16` §8): `N = 667` items reviewed a
month, half passing the gate (`g = ½`), `ε = 0.05`, and ≈333 pilot slots a month of declared
capacity, used as the denominator in the form `N g = 333.5`. A slot is one item piloted once
through the two stages: it counts items, not answers, anchors or fits. The audit draw is
independent of entry, so the expected overlap of audit and entry is `N g α`. Formulas (C):
reference `N[g + (1 − g)ε]`; audit with full compatible reuse `N(g + α − gα)`, separate
`N(g + α)`; universal pilot `N`; D under independent entries `N[1 − (1 − ε)(1 − g)^K]`.

| Design | Validation | Audit or draw | Reusable overlap | Total | Increment | Of capacity |
|---|---|---|---|---|---|---|
| Reference (gate, exploration) | 333.5 | 16.675 (exploration) | — | 350.175 | 0 | 0 |
| C, `α = 0.05`, full reuse | 333.5 | 33.35 | 16.675 | 350.175 | 0 | 0 |
| C, `α = 0.05`, separate | 333.5 | 33.35 | — | 366.85 | 16.675 | 5.0% |
| C, `α = 0.525`, full reuse | 333.5 | 350.175 | 175.0875 | 508.5875 | 158.4125 | 47.5% |
| C, `α = 0.525`, separate | 333.5 | 350.175 | — | 683.675 | 333.5 | 100.0% |
| A, universal pilot | — | — | — | 667 | 316.825 | 95.0% |
| D, `K = 2`, independent entries | 500.25 (activated) | 8.3375 (drawn) | in the group | 508.5875 | 158.4125 | 47.5% |
| D, `K = 4`, independent entries | 625.3125 | 2.084375 | in the group | 627.396875 | 277.221875 | 83.1% |
| D, `K = 8`, independent entries | 664.39453125 | 0.13027344 | in the group | 664.52480469 | 314.34980469 | 94.3% |

- `α = 0.525` equals the reference's expected scored outcomes per judgment, `g + (1 − g)ε`;
  `α = 0.05` is `16` §8's audit. The reference itself uses 105.0% of the declared capacity.
- The increment is `N(1 − g)(α − ε)` with full reuse (none at `α = ε`, exploration being
  retired) and `N[α − (1 − g)ε]` without. D's rows assume the group's verdict decides its
  entrants' pool entry (reuse inside the group); piloting entrants apart for entry adds
  `N g`. At `α = 0.525`, D with `K = 2` costs what C with reuse costs: under independence,
  `1 − α = (1 − g)(1 − ε)` there. An identity of the totals, not of the designs.
- Correction of the earlier version: its "≈175 extra slots (≈47% of 333)" at `α = 0.525`
  mixed the reusable overlap (175.0875) with the full-reuse increment (158.4125, 47.5%);
  without reuse the increment is 333.5 (100%). Its "≈17 extra slots" at `α = 0.05` is the
  separate case; with full reuse there are none.

**What a slot does not count** (L for the floors; no time is claimed).

- *Answers to items.* A batch needs at least `N1_MIN = 300` admitted respondents at stage 1 and
  `N_LATENT_MIN = 3000` at stage 2 (code floors, not power guarantees: `15` B4–B5), each
  answering the batch's items. Whether the two stages share respondents is not specified
  (da verificare).
- *Anchors.* Every respondent of a batch answers all its anchors, so anchor answers scale with
  batches times respondents, not with slots; with groups of `K` the batches are the piloted
  groups. The anchor count is a design value (the test uses 60).
- *Fits.* One stage-1 fit and one stage-2 fit (a one-class fit, then mixtures by BIC) per batch
  and per attempt. No fit time is measured on the candidate (`15` D2, D4).
- *Retries.* An attempt draws a fresh sample, so a retry repeats a batch's answers, anchor
  answers and fits. Under independent attempts converging with probability `c`, at most `k`
  attempts cost `(1 − (1 − c)^k)/c` attempts in expectation; `c` by batch and verdict class is
  not measured on admitted batches of the candidate (`13` §8.7.5 has stage 1 on simulated
  pilots only).

**An equal expected count is not an equal precision.** At `α = 0.525`, C scores as many
outcomes per judgment in expectation as the reference, but a group draw selects its members
together. The formulas below assume draws independent across groups and the assignments fixed.
For a reviewer whose items fall `m_G` in each group `G`, the observed count's variance is
`α(1 − α) Σ_G m_G²`, against `α(1 − α) N_u` for independent draws per item: it grows when several
of the reviewer's items share a draw (six items in one group: 8.9775 against 1.49625). The
variance of the IPW sum `Σ_j I_j d_uj/α` given the `d_uj` is `((1 − α)/α) Σ_G (Σ_{j ∈ G ∩ R_u} d_uj)²`,
against `((1 − α)/α) Σ_j d_uj²`: it can grow or shrink, since scores of opposite signs in one
group compensate. With `d₁ = 1` and `d₂ = −1` in one group it is 0, against `2(1 − α)/α`
(Astra's example, checked). The precision is different, to be evaluated with the group and
assignment rules, not yet specified; it is no universal loss of information. The two coincide
when no two of a reviewer's items share a group. D's inclusions are correlated in the same
way, with weights `1/ε` on drawn groups. In every batch design, the current one included, the
verdicts of one batch come from one fit and are dependent.

## 5. Missing outcomes (C2)

| Case | What the code does now (L) | Policy |
|---|---|---|
| batch not admissible | `PilotError` (`BatchTooSmall`, `NotEnoughRespondents`, `RowCountMismatch`, `BadFormats`, `UnreliableAnchors`, `SharedTemplate`) before any fit; lifecycle refuses `Pilot1Batch` without enough respondents and `Pilot2Batch` below `K_MIN`, leaving the item where it is | — |
| too few respondents | as above | — |
| fit not converged | stage 1: at `164fff6` no item kept, which composed into outcome 0 on both paths (§2, E-api); since the A11 correction every item indeterminate, the item pending in `Pilot1` or `Explored`, with no termination guarantee (`15` A11); stage 2: at `164fff6` `Indeterminate` with no lifecycle input; since the correction of `15` A4's residual (a), the item pending in `Pilot2` or `Explored`, equally with no termination guarantee | — |
| fewer than `K_MIN` stage-1 survivors in a fixed group | if stage 2 runs on survivors (`05` [7]), `Pilot2Batch` is refused (`BatchTooSmall`) and the survivor stays in `Pilot2` | — |
| a pilot that does not conclude | the item stays in `Pilot1` or `Pilot2` | — |
| withdrawal before the outcome | no lifecycle event exists | — |

Policies, with their effects:

1. *Read as 0* — converts a missing outcome into a negative one (§3). Excluded.
2. *Drop from numerator and denominator* — keeps the identity only if missingness is
   independent of the outcome and of the reports given the pre-draw information. Admission and
   convergence can depend on the item (a mixture fit may converge less often), and the
   denominator then varies per realization (C4). Not an automatic correction.
3. *Bounded retry*: the group, the anchors' design, formats and templates fixed; a fresh
   respondent sample; at most `k` attempts. Stopping at the first converged attempt selects
   samples. Illustration under stated assumptions, not a measurement: if attempts are
   independent, and a single attempt's verdict is 1 with probability `p` and converges with
   probability `c₁` when the verdict is 1 and `c₀` when it is 0, the first converged verdict is 1
   with probability `p c₁ / (p c₁ + (1 − p) c₀)`. With `p = 0.3`, `c₁ = 0.6`, `c₀ = 0.9` that is
   `0.18/0.81 = 2/9 ≈ 0.2222`, not 0.3, and the outcome stays missing after `k` attempts with
   probability `0.19^k` (C). The calculation holds under those assumptions only; it guarantees
   no availability. If the same bounded retry is applied on every path and is part of the
   reference procedure, properness does not need the single-attempt law ((1) and (3) hold for
   the procedure's verdict); the residual missingness after `k` attempts is still a C2 case.
4. *Keep pending until an outcome exists* — no termination guarantee; a pending item is not in
   `SkillTrack`'s denominator, so the same selection question as (2) arises.

No policy is free, and C2 stays open. A measurement cannot settle the independence policy (2)
needs: the outcome a failed fit would have given does not exist, so that independence cannot
be checked on the data; measuring first does not demonstrate it. Admission and convergence
rates by batch composition, format and size can be measured, and on simulated populations by
the planted truth (as `13` §8.7.5 does for stage 1); they inform plausibility under the
simulated model, not independence in the field.

## 6. Conclusion

**Demonstrated**, each within its kind of evidence:

- No code forms pilot batches; no driver composes the pilot into the lifecycle (L).
- The verdict is a batch-level quantity: one fit, one convergence status and one class
  selection per batch (L).
- In one realization, with companions chosen by hand, a target's verdict is `Dif` beside
  leaning companions and `NoDif` beside clean ones (E-fit). Sensitivity to companions, not a
  frequency and not a deviation.
- At `37addca` the APIs composed a stage 1 that did not converge into an observed outcome 0
  on both paths (E-api); real stage-1 fits failed to converge on some simulated pilots (`13`
  §8.7.5). The correction (`15` A11) is closed on `dd842d6`.
- The paths' outcome laws need not coincide (abstract counterexample; D, C). The lemma needs
  conditional agreement (2) on each path's event; with H0, C1, C2, C4 and (3), truthful
  reporting is the unique maximizer: `16` §4.3's conditional theorem.
- D's IPW identity under the conditions stated in §4 (D; an exact enumeration as illustration).
- The costs of §4.1 on the declared scenario (C).

**Dependent on hypotheses**, none of them a property of the protocol today: H0 and C1 (the
source; `16` §6's residual), C2, C4, a production caller for C5, and (2) and (3) under `u`'s
joint deviation for whatever batch rule is chosen; reuse in C (compatible group, sample,
procedure and timing); D's cost (the joint law of entries within groups) and D's identity (the
group's freeze, one procedure for both activation causes); the retry illustration (independent
attempts with the declared probabilities).

**Sustainable alternatives**, none approved or recommended:

- A: (1) and (3) by construction and no draw; 95% of the declared capacity over the reference.
- C: a constant `π` and a report-independent count; from no increment (`α = ε`, full reuse) to
  100% (`α = 0.525`, separate); a different precision per expected outcome, to be evaluated,
  when a reviewer's items share a group.
- D: every observed outcome scored, at `π ∈ {1, ε}`; between the reference and A depending on
  `K` and on the correlation of entries; a group freeze and new lifecycle inputs.
- B and dynamic batching: not shown viable, not excluded (§4).

**What is actually missing.**

- Measurements: the joint law of gate entries within candidate groups (D's cost); admission
  and convergence rates of both stages on admitted batches of the candidate, by format and size
  (retry cost; for missingness, plausibility only); detection on campaigns of fixed-reference
  designs against group designs, if B is reconsidered; respondent supply and fit time on the
  candidate (`15` D2, D4).
- Design decisions (Astra): the batch contract itself (A, C, D or another); group size and
  formation rule (templates apart); whether a group's verdict decides its entrants' pool entry
  (reuse); the retry bound and the residual-missing policy (C2); whether exploration is
  retired for scoring. (A11's correction, listed here at `37addca`, is closed on `dd842d6`.)
- No decision is asked of the owner now; capacity trade-offs arise only once a design is
  chosen.

**Acceptance criteria for any implementation.**

1. The group formation rule is fixed and recorded before the reports and reads no path, report
   or gate decision (a test that it is a function of the admitted set alone).
2. An observed group is piloted in full under one procedure, whatever caused its observation;
   the recorded `π` is the inclusion probability given the freeze.
3. No missing outcome is recorded as 0 (for stage 1, `15` A11, closed).
4. The retry bound and the residual-missing policy are those decided, applied alike on every
   path.
5. The executed comparison stays in the suite as evidence of the dependence it guards against.

## 7. Proposal awaiting review: a three-category reference outcome

**A candidate of Astra's, checked here; not approved for implementation.** Astra's review on
`654dbff` approved, as conditional results: the ternary score's derivation, its binary reduction
and the pointwise bound `[−1, 1]` (§7.1); B-b's derivation, with a fixed denominator, the joint
invariance and a positive probability of conclusion (§7.5); the abstract constructions'
calculations, within their stated limits. Not approved: §7 as a whole, and any implementation.
Rectified after that review: §7.1's path conditions, §7.5's B-b, §7.6's calibrations, and
constructions now stated as sufficient, not unique. Astra's preference for the next design
comparison is in §7.7. Review on `825cee3` (Astra): §7 accepted as a conditional analysis, not as
a protocol approved for implementation; evidence: the diff read and the IPW identity checked with
rational arithmetic on a construction whose draw and observed outcome are dependent yet meet the
corrected conditions; no Rust or fit run. B-b the main candidate, the ternary score an
alternative; A1, C2 and A2 open. Its non-blocking precision is made in §7.5 (*Selection*).
Nothing here changes code, API, threshold, golden output or current policy; A1 and A2 stay open,
R1 stays approved. Evidence: L, D and C as above; every C is an exact rational enumeration whose
formula is given in place. No fit, smoke, campaign, calibration or mutation run.

**The candidate.** (a) A reference procedure is attached to each item before any report: group,
respondents' and anchors' sampling, model, attempts and term. (b) Its outcome `Y` has three
categories: admissible `A`, rejected `R`, inconclusive within the term or budget `I`. (c) A
reviewer reports a distribution `p` on `{A, R, I}`. (d) The score is
`d = ½‖b − e_Y‖² − ½‖p − e_Y‖²`, `e_Y` the category's unit vector, `b` a distribution. Its claim:
if the joint law of `(b, Y)` given `F` is invariant to the reviewer's deviations,
`E[d | F] = C − ½‖p − q‖²`; stated as no guarantee of the protocol or of the reputation used.

### 7.1 The derivation, apart from the IPW

(D) Take `F = F_u`, `p` `F`-measurable, `q = E[e_Y | F]`. Since `‖e_Y‖ = 1`,
`d = ½(‖b‖² − ‖p‖²) + ⟨p − b, e_Y⟩`, affine in `e_Y`, and
`E[½‖p − e_Y‖² | F] = ½‖p − q‖² + ½(1 − ‖q‖²)`. Hence `E[d | F] = C − ½‖p − q‖²` with
`C = E[½‖b − e_Y‖² | F] − ½(1 − ‖q‖²)`. `C` reads the joint law of `(b, Y)` given `F` and its
marginal `q`; when that law is the same under every `σ_u` (`u`'s joint deviation over `R_u`, §3),
`C` is free of `σ_u` and `p = q` is the unique maximizer on the simplex. **The derivation holds
as stated.** `b` need not be `F`-measurable. (C: both sides equal on 2,000 random rational joint
laws of `(b, Y)`.) Also:

- with no mass on `I` and `Y ∈ {A, R}`, `d` is today's `(b − o)² − (p − o)²` exactly (C, on a
  grid of tenths): the ½ keeps the binary scale;
- `−1 ≤ d ≤ 1` pointwise, since `½‖b − e_Y‖² ≥ 0` and `½‖p − e_Y‖² ≤ 1` on the simplex (D;
  both bounds attained on a grid of sixths, C). Of `16` §4.4's missing-reveal argument this keeps
  only the step that uses `d ≥ −1`; every other condition there remains (the IPW identity
  relative to `F'_u`, C4, the deviation leaving the other terms unchanged, a penalty free of the
  outcome and the draw, the replacement's report, reputation beyond `Ŝ_u`, involuntary absences).

The identity concerns one item's reference outcome given `F_u`: it is `16` §4.3's C6 step with
`Y` for `o`. It uses no draw, path, inclusion probability, nor any independence of `I`. Moving it
to the computed `Ŝ_u = (1/N_u) Σ I_j d_uj/π_j` needs `16` §4.2's conditions and §3's, with §3's
two path conditions kept distinct, `Y^e` and `Y^x` the outcomes observed on entry and after
exploration:

- H0; C1, `E[X_j | F_Φ] = ε` on `{D_j = rejected}`; C4, the assigned `N_u`;
- C2 as a termination property: an included item's procedure ends with a recorded `Y` (§7.2);
- (2e) on `{D_j = enters}`: `E[e_{Y^e} | F_Φ] = E[e_Y | F_Φ]`;
- (2x) on `{D_j = rejected}`: `E[X_j e_{Y^x} | F_Φ] = ε E[e_Y | F_Φ]`.

With `d = α + ⟨β, e_Y⟩`, `α` and `β` `F_Φ`-measurable,
`E[X_j d(Y^x) | F_Φ] = α E[X_j | F_Φ] + ⟨β, E[X_j e_{Y^x} | F_Φ]⟩`: the constant term needs C1,
the linear term (2x); then §3's argument runs as written (C: equality on 500 random rational
laws satisfying both). (2x) follows from C1 when `X_j` is independent of the observed `Y^x` given
`F_Φ` and `Y^x` has `Y`'s conditional law there (§3's (4)). Independence of `X_j` from the
reference `Y` alone does not give it, even with equal marginals. *Counterexample* (Astra; D, C):
`X` and `Y` independent Bernoulli(½), `ε = ½`, the observed potential outcome `O = X`, so `O` and
`Y` have one law; with `p = 1` and `b = 0`, `E[d(Y)] = 0` but `E[X d(O)/ε] = 1`, and in the
order `(0, 1)` `E[X e_O] = (0, ½)` against `ε E[e_Y] = (¼, ¼)`.

The invariance and these IPW conditions are logically distinct: the invariance can hold with a
known draw (C1 fails, `16` §3), the IPW identity can hold for a `Y` that `u`'s reports move (C6
fails, §7.3).

### 7.2 What makes inconclusiveness observable

`I` must be a recorded outcome, not an absence. With a deadline alone, a missing record cannot
tell "inconclusive" from "not run", "not recorded" or "withheld by whoever runs the pilot", and
reading absence as `I` lets that party choose it. Needed together:

1. **Positive terminal records.** Each attempt within the declared budget ends with a recorded
   status: a converged verdict, or a named non-conclusion (the `PilotError` met,
   `Screening::Indeterminate`, fewer than `K_MIN` stage-1 survivors, `Recheck::Indeterminate`).
   `Y = I` is recorded when the last attempt so ends.
2. **Reproducibility.** Each status a deterministic function of recorded answers, anchors,
   seeds, model version and thresholds, which a verifier recomputes (`CLAUDE.md` invariant 7);
   the optimizer bounded by iterations, never by elapsed time.
3. **A term in log order**: log records or epochs, by a rule fixed before the reports; no
   clock enters the computation (invariant 7).
4. **Intrinsic against resource non-conclusion.** A fit failing on the data received belongs to
   the item, its group and the procedure. Data that do not arrive (respondents short of the
   floor by the term, no beacon round for a drawn group, no runner) belong to supply and timing:
   counting them as `I` keeps the invariance only if their law does not change with `u`'s
   deviations. Resources fixed independently of reports and paths (e.g. capacity reserved at
   association) are one construction that would give it, not a necessity shown here; without
   such a property they are a channel of §7.3, not an outcome.
5. **Liveness apart.** Who signs which record by when, and what follows if not, is a rule outside
   the score; until it holds a missing record is a pending item (§5, policy 4).

### 7.3 What can change `Y`

The third column gives sufficient constructions, not the only ones; a design that proves the
invariance otherwise is not excluded.

| Dependency | How it reaches `Y` | A construction giving invariance |
|---|---|---|
| Reports, via the path | entries choose companions under dynamic batching, and activate groups under D | the group fixed at association, as in the candidates A, C, D (§4) |
| Path | start time (a drawn group waits for its beacon round, §4 D); a pool-entry pilot reused (§4 C) | one procedure and timing whatever the activation cause |
| Other items | companions' content and stage-1 survival (`K_MIN`) | none while they do not depend on reports (§3); membership changes after the gate (an author's withdrawal, no lifecycle event today) do: the group's data fixed at association |
| Load | groups piloted in a period share respondents; their number follows entries (D) and appeals; under a time term a sample's size, hence admission and convergence, follows other items' reports | resources reserved per group, or a respondent budget with no time term |
| Time | the respondents' population and the anchor set drift | sampling frame and anchors fixed at association |
| Model and thresholds | a model change or T83's operating point between association and run | model version and thresholds pinned at association |
| Beacon | a coalition that aborts rounds delays a drawn group (`16` §6); under a time term, delay becomes `I` | a term in attempts, or `16` §6's member model stated |
| Source check | `Contested` counts as 1 (`outcome_of`) through `source_verified` (T68, not in code) | the check inside the procedure, or `Contested` placed by definition |
| Respondents | a person holds unlinkable evaluator and respondent pseudonyms (invariant 5), so sampling cannot exclude a reviewer from its items' samples | C6 covers the evaluator's deviation only; one respondent's effect on `Y` is not bounded here |

*Construction* (D, C; abstract: `u`'s report alone decides `k`'s entry, as in `16` §3's
simplified gate; not a model of the protocol). `u` reviews `j` and `k`, in different groups.
Within `j`'s term `j`'s group reaches its floor only if `k`'s group is not piloted in the period,
and `k`'s group is piloted exactly when `k` enters. Truthfully `Y_j` has law `(3/5, 2/5, 0)`,
`b_j` equals it, and `E[d_j] = 0`. Deviating, `u` moves `δ` of `k`'s mass from `R` to `A`, enough
to enter, and reports `p_j = e_I`: `Y_j = I` and `d_j = ½‖b_j − e_I‖² = 19/25`, at an expected
loss `δ²` on `k`. The forecast fulfils itself whenever `δ² < 19/25`: what C6 excludes. Not a
frequency, nor an attack on bridging.

### 7.4 Joins with the current mechanism

None is chosen here.

| Element | Today (L) | To decide |
|---|---|---|
| Report | one revealed `prob` in `[0, 1]`; `review::commit` hashes its eight bytes (`isegoria/commit/v2`) | a vector commitment version; a simplex check; a representation keeping that check exact and the score bit-reproducible (integers over a fixed denominator are one option; reproducibility alone does not impose it) |
| Bridging | the same number is the rating `r_uj`; `τ ≈ 0.80`, the band and the polarization cut 0.25 live on its scale (`02` §A.3) | one function of the vector as rating, or a separate rating; either way a recalibration (T25, T83) |
| Baseline | weighted means of scalar forecasts (`panel_scores`; golden items `loo_baseline`, A10's fallback) | a coordinatewise mean stays in the simplex; C5's composition carries over; A10's fallback still gives `d ≡ 0`; golden items' known outcomes (`honeypot::reviewer_skills`) stated in the three categories |
| Denominator (C4) | `SkillTrack` counts recorded items, never a pending one | the assigned `N_u` and the no-show rule, still; a termination guarantee records every included item, but snapshots before the term read a path-dependent subset |
| Missing reveals | `16` §4.4's proposal | the step that uses `d ≥ −1` holds (§7.1); every other condition of `16` §4.4 remains |
| `k_u` | observed count; increment `P(enters) + ε P(rejected)` (`16` §7) | `I` is observed, so an entered item counts at its term; the report-dependence stays; whether `I` counts toward `N_PROBATION` |
| Shrinkage, weight | `exp(γ S_u k_u/(k_u + k₀))`, `γ = 35`, `k₀ = 100`, set on the binary score | same range, another distribution (T25) |
| Cap | `3 × median(w)` | same formula on other weights |
| CUSUM | `k = 0.03`, `h = 1.5`, rates from independent binary outcomes (`08` AT-REP-07) | `I` is batch-level, so a reviewer's items in one group score dependently; the rates do not transfer |
| Recording | `outcome_of` gives 1 or 0; `FalseNegatives` counts `Measured`; no terminal state for `I` | a category type; `I` in the false-negative rate; a scoring record at the term, apart from the item's pool-entry fate |

**The transformation is a decision.** For `q = (7/10, 1/10, 1/5)`, `p_A = 7/10`,
`p_A/(p_A + p_R) = 7/8` and `p_A + p_I/2 = 4/5` (C): below, above and at `τ = 0.80` for the same
belief. A separate rating would be scored by no proper rule; today the gate's input is the scored
forecast (`01` D23, D33).

### 7.5 Keeping the binary forecast

Two binary targets use the same procedure:

- **B-a**: `o = 1{Y = A}`, so `I` scores as 0; target `P(Y = A | F_u)`, proper under the same
  invariance (§7.1 on two categories). It records an unconverged procedure as outcome 0, which A11's
  closed contract excludes for stage 1: that contract would need amending. The rating then forecasts
  "admissible within the term", mixing the item's quality with its group's convergence.
- **B-b**: `o = 1{Y = A}` on `{Y ≠ I}`, an `I` adding 0 at a fixed denominator; with
  `c = P(Y ≠ I | F_u)`, target `q_c = P(Y = A | Y ≠ I, F_u)`. (D)
  `E[1{Y ≠ I} d | F_u] = E[1{Y ≠ I}(b − o)² | F_u] − c[(p − q_c)² + q_c(1 − q_c)]`, so `q_c` is
  the unique maximizer when `c > 0` and the joint law of `(b, Y)` given `F_u` is invariant (C:
  exact on 2,000 random laws). When `c = 0` the term is 0 for every report: no report is
  preferred, `q_c` is undefined, the item carries no incentive; for small `c` the incentive
  scales with `c`. B-b keeps the scalar format, not automatically the forecast's current meaning
  (`02` §C.2): its target is `P(A | conclusive, F_u)`, not `P(A | F_u)`. Nothing scores
  conclusiveness, and each item weighs `c`, set by its group's convergence.

What stays in all three:

- **Availability.** Each needs §7.2's procedure, term and recorded `I`: B-a to record 0, B-b to
  record no contribution, the candidate to record `I`. A binary report saves none of it.
- **Selection.** A deviation of `u` that moves companions, load or timing is a possible channel
  (§7.3): the invariance fails when the move changes the joint law of `(b, Y)` given `F_u`, not
  at every organizational change; when it does, it fails whatever the target. In §7.3's
  construction B-a gains `(3/5)² = 9/25`; B-b gains when `u`'s truthful expected contribution is
  negative, e.g. `−q_c(1 − q_c) = −6/25` when the baseline already equals the conclusive outcome.
- **Statistical dependence against a change caused by a deviation.** `I` may depend statistically
  on the item's quality and on the information behind the reports (convergence is batch-level,
  §2; a mixture fit may converge less often, §5): none of the three results needs `I` independent
  of them, and that dependence is compatible with each. A change of the joint law of `(b, Y)`
  given `F_u` (conclusiveness or verdict) caused by `u`'s deviation is another matter: the
  hypothesis excludes it, and nothing here lets a reviewer move either while the proof stands
  (§7.3's construction). Only §5's policy 2 (dropping `I` from the denominator) would need
  statistical independence, to keep the unconditional identity.
- **Nothing validated.** `τ`, the band, the polarization cut, `k_u`'s probation count, the
  shrinkage, the cap and the CUSUM were set on today's binary score and reading (§7.4); neither
  B-b's unchanged format nor the ternary score validates any of them for its target.

The candidate scores `R` apart from `I` and keeps A11's contract; B-b keeps the scalar format with
a conditional target; B-a needs A11 amended.

### 7.6 Item admissibility against a form's DTF

`Y = A` is an item's verdict in its group's fit, not a statement on any form's `DTF_μ(T)` (`19` §2):
an item under the flag cut alone exceeds `DTF_MAX`, per-batch measures differ, active items lie
outside `D(T)` (`19` §3, §4 rows 1–3). One data set serving both needs its own conditions. On the
same target (`19` §2: one population `μ`, one group family, number-correct score), with `P` items in
the pool, `A` anchors, forms of `t = |T|` items:

| | Per form (`19` §5, R3's data) | Reusable common calibration |
|---|---|---|
| Respondents | `n_F` per form; `N_LATENT_MIN = 3,000` a floor, not power (`15` B4–B5); forms searched on the same estimates need a treatment of selection, e.g. an independent confirmation sample or a bound valid over the whole search (`19` §5), not an obligation for every form | at least `⌈P n_c/m⌉` per calibration when each item is answered `n_c` times and a respondent answers at most `m` items: a capacity bound under these assumptions, not a validated administration design; `m = P` is the complete design; `n_c` (power) and `m` (burden, D2) not set |
| Answers, anchors included | per form `n_F t` trial and `n_F A` anchor answers | per calibration `P n_c` trial answers with exactly `n_c` per item (the rounded respondent count times `m` can exceed it); `A` anchor answers per respondent, if each answers the anchors |
| Fits | a latent search (up to seven candidate models, `02` §B.3, `18` §2) per form, plus a bound not yet built | a search over the `P` items: today's code fits only the complete design (`m = P`), since `scoring::latent` takes complete respondent × item matrices (L); planned missing data needs an extension of the implementation and of the likelihood over the observed cells, to be justified, or a block or linking design with the cross-fit H1 to show (`19` §3); none validated; a bound valid over every form searched |
| Time | recruitment and fits before each form's use | recruitment and fit before any form; new items wait for the next calibration |
| `19` §3's hypotheses | sampling `μ` does not realize H1 exactly for the estimated measure: estimation error, counted classes and representation remain to treat; H2 on `T`'s classes; H3 by construction | H1 for all forms at once, with the same residues; H2 by the pool's classes, which may separate on items outside `T`; H3 for calibrated items only; a pool change, a population drift, a model or threshold change requires re-examining the domain of validity, without voiding every earlier calibration automatically |

Summed over forms, attempts and periods, these counts are participations, not necessarily
distinct persons. Which design needs fewer depends on quantities not fixed here (forms per
period, `n_F`, `n_c`, `m`, the pool's turnover, the treatment of selection); fit time is measured
on neither (`15` D2, D4); no cost is estimated here. The A1 pilots' answers are not counted as
calibration data: that would need one population, anchors and model, and the groups linked; none
is established.

### 7.7 Verdict

**To be rectified, not discarded** (this check's verdict; Astra's review on `654dbff` approved
the conditional results listed at the head of §7, not §7 as a whole). The scoring core is
coherent under explicit conditions: the derivation holds (§7.1) under the joint invariance of
`(b, Y)` given `F_u`; it is exactly the current score on two categories and keeps its pointwise
bounds; with the IPW conditions kept apart, it is `16` §4.3's step, not a guarantee of the
protocol or of the reputation used. As a protocol candidate it needs:

1. the term and budget in attempts and log order, `I` a positive reproducible record, and resource
   non-arrival either shown invariant to `u`'s deviations (reserved resources are one
   construction) or kept out of `I` (§7.2);
2. the procedure closed at association, with a rule for later membership changes (§7.3);
3. an explicit bridging input and commitment format, with their recalibration (§7.4);
4. an observation design that establishes H0, C1, (2e), (2x) and C4 (§4's A, C and D are
   candidates; another design that proves the conditions is not excluded), and a decision on the
   reputation's elements (§7.4), which three categories do not touch.

B-b keeps the scalar format with a conditional target and fewer joins; B-a conflicts with A11's
contract (§7.5). **Direction (Astra, after the review of `654dbff`):** B-b is to be deepened as the
main candidate in the next design comparison, the ternary score as the alternative. This approves
no protocol change and closes neither C2 nor A1. An admissible item certifies no form's DTF, and
the two calibrations stay to be compared (§7.6).

## 8. B-b as a candidate contract (conditional analysis; protocol not approved)

**Status.** A proposal of this check, built on §7.5's B-b (its derivation approved by Astra as a
conditional result on `654dbff`) and on Astra's direction (§7.7); accepted by Astra on `fa11791`
as a conditional analysis (below); **as a protocol not approved, not implemented**. Astra's
review of `44f0dbd` approved, as conditional results: B-b's identity for a prefixed cohort (D1);
the algebraic sufficiency of the identities on the `A` and `R` coordinates for the mean (D2); the
pointwise bound and the zero on conclusive verdicts (D3, D4); the distinctions between pending,
terminal inconclusiveness and non-selection, and between `N_u`, `O_u` and `V_u` (§8.2, §8.4). Not
approved by that review: §8 as a whole, the protocol, the implementation, the adoption of A.
Evidence: the diff, the dossier and the relevant code read; exact rational
calculations on the counterexamples and the costs; no Rust or fit run. Rectified after that
review, in `fa11791`: §8.3's consolidation, §8.1's weights and inclusion probabilities and the
definition of `S`, §8.4's statements on C1, §8.5's guarantees of A and its costs. Review on
`fa11791` (Astra): §8 approved **as a conditional analysis, with no blocking finding**; the
rectifications accepted — prefixed cohorts, availability and the selection of consolidated
cohorts (§8.3); algebraic weights apart from inclusion probabilities (§8.1, D2); the
delimitation of A's guarantees and the costs with their correct denominators (§8.5). Not
approved: the protocol, the implementation, the adoption of A; A stays the main analytic
reference, C the comparison even if A proves sustainable. Evidence: the diff and the documents
read; exact rational calculations on the counterexamples and the costs; no Rust or fit run. The
earlier approvals stay as recorded: the review of `44f0dbd` approved only the conditional results
listed above. §8 changes no code, API, threshold, golden output or policy and closes none of A1,
C2, A2. **H** marks a hypothesis, **D** a consequence proved here or in §7 under the stated
hypotheses (with C where an exact enumeration checks it), **Open** a decision or evidence still
needed.

### 8.1 Definition, hypotheses, consequences

- **Assignments.** `R_u`, fixed by the assignment record; `N_u = |R_u|` (C4).
- **Reference procedure.** Per item, fixed before the reports (§7): group, sampling, anchors,
  model and thresholds, attempt budget, term in log order; reference outcome `Y_j ∈ {A, R, I}`,
  `I` only on §7.2's positive records.
- **Selection.** `S_j ∈ {0, 1}` indicates that the design selected `j` for observation (its
  selection record), whatever the state of its procedure: a selected item still pending has
  `S_j = 1`. `π_j` is the inclusion probability the design records, `F_Φ`-measurable. `Y_j^obs`
  is the outcome the selected procedure yields, kept apart from the reference outcome `Y_j`.
- **Contribution.** As a function of an outcome `y`, with report `p = p_uj` and baseline
  `b = b_uj` (C5's composition, `16` §4.6): `g_uj(A) = (b − 1)² − (p − 1)²`,
  `g_uj(R) = b² − p²`, `g_uj(I) = 0` for a terminal `I`.
- **Score.** For a prefixed cohort `K ⊆ R_u` (§8.3), the final estimator
  `Ŝ_K = (1/|K|) Σ_{j ∈ K} S_j g_uj(Y_j^obs) / π_j`; `Ŝ_u` is `Ŝ_{R_u}`.
- **Target.** `q_c = P(Y_j = A | Y_j ≠ I, F_u)`, with `c = P(Y_j ≠ I | F_u)`.

**H.** (H-a) H0, `F_u ⊆ F_Φ`. (H-b) Path identities on the conclusive coordinates: on each path's
event and for `k ∈ {A, R}`, `E[S_j 1{Y_j^obs = k} | F_Φ] = π_j P(Y_j = k | F_Φ)`. (H-c) The joint
law of `(b_uj, Y_j)` given `F_u` is invariant under `u`'s joint deviation over `R_u`. (H-d) Every
report in `K` completed; C5. (H-e) The recorded probability is the design's:
`P(S_j = 1 | F_Φ) = π_j > 0`, C1 on the whole selection. H-b and H-e are distinct requirements of
the contract: one concerns the outcomes observed on selection, the other the declared probability.

**D.**

1. Under H-a–H-d, for a prefixed cohort `K`,
   `E[Ŝ_K | F_u] = (1/|K|) Σ_{j ∈ K} E[g_uj(Y_j) | F_u]`, with
   `E[g(Y) | F_u] = E[1{Y ≠ I}(b − o)² | F_u] − c[(p − q_c)² + q_c(1 − q_c)]`: `q_c` is the unique
   maximizer where `c > 0`; where `c = 0` every report gives 0 (§7.5). It concerns the final
   estimator of a set fixed in advance, not its availability nor the cohorts consolidated by a
   given time (§8.3). This mean does not use H-e (D2); the contract keeps H-e for §8.4's expected
   counts and §8.6.
2. (Algebraic.) `g_uj(y) = ⟨γ, e_y⟩`, `γ = ((b − 1)² − (p − 1)², b² − p², 0)` `F_Φ`-measurable:
   no constant term, a zero coefficient on `I`. For any `F_Φ`-measurable weight `ω > 0` with
   `E[S_j 1{Y_j^obs = k} | F_Φ] = ω P(Y_j = k | F_Φ)` for `k ∈ {A, R}`,
   `E[S_j g(Y_j^obs)/ω | F_Φ] = E[g(Y_j) | F_Φ]`, whether or not `E[S_j | F_Φ] = ω`. *Abstract
   algebraic construction* (C), not a design: `Y` uniform on `{A, R, I}`, `S = 1` on `I` and
   Bernoulli(½) otherwise, observed outcome `Y`, `b = ½`, `p = ¾`. Then `P(S = 1) = ⅔`,
   `E[g(Y)] = −1/24`, `E[S g/ω] = −1/24` with the weight `ω = ½`, and `E[S g/(2/3)] = −1/32`
   with the effective inclusion probability: there the coordinate identities fail, since the
   selection follows the outcome. It shows an identity with a normalizing weight, not a design
   whose inclusion probability is ½; it weakens neither H-b nor H-e.
3. `−1 ≤ g ≤ 1` pointwise, so a realized term `S g/π` lies in `[−1/π, 1/π]` (C, on twentieths).
4. On a verdict, with `p, b ∈ [0, 1]`, `g = 0` exactly when `p = b`: `(b − o)² = (p − o)²` gives
   `p = b` or `p = 2o − b`, and the second lies in `[0, 1]` only when `b = o` (D; C on twentieths).

### 8.2 Cases

| Case | Evidence required | Numerator | In `N_u` | Inclusion probability | Consolidable |
|---|---|---|---|---|---|
| 1. Assigned, not selected | the assignment, the completed report and the record fixing `S_j = 0`: the design's draw on its round under `16` §5's rule and, under D, no member's entry; the case does not arise under A | 0, the IPW term of an unobserved item, not an observation | yes | `π_j` recorded; no term uses it | yes, once `S_j = 0` is final |
| 2. Selected, procedure pending | the selection record (`S_j = 1`) and the association record; no terminal record | none yet, never 0 | yes | `π_j` at selection | no: provisional (§8.3) |
| 3. Positive conclusion | a reproducible terminal verdict `A` (§7.2) | `[(b − 1)² − (p − 1)²]/π_j` | yes | `π_j` | yes |
| 4. Negative conclusion | the same, verdict `R` | `(b² − p²)/π_j` | yes | `π_j` | yes |
| 5. Verifiable terminal inconclusiveness | §7.2's positive records for every attempt of the budget | 0, an observed outcome without a verdict | yes | none in the numerator; `π_j` only in the counts' expected values (§8.4) | yes |
| 6. Report not completed | the assignment, the reveal deadline and no valid reveal before it; a lost or censored reveal leaves the same absence | **Open**: no `p`, so no `g`; `16` §4.4's `s_ns ≤ −1` is a proposal, not approved | **Open**: C4 includes it; with a 0, withholding pays where the expected contribution is negative (§7.5's −6/25); excluded, C4 fails | — | no rule yet |
| 7. Freeze not reached | other reports, the decision or the appeal window not final; a missing reveal freezes the item today (`PartialEpoch`, T58 not implemented) | none yet | yes | not yet set | no |

Cases 1 and 5 and a verdict with `p = b` all add 0, for different reasons (§8.4). A pending item
is never read as 0, nor a missing record as `I` (§7.2).

### 8.3 Consolidation and availability

- **Prefixed cohort.** A set of assignments fixed by a rule on the assignment record alone (for
  instance `u`'s assignments of one epoch), never by outcomes or arrival times. It is
  *consolidated* when every member is in case 1, 3, 4 or 5 with its record.
- **Four things kept apart.** (i) The *final estimator* `Ŝ_K` (§8.1), a function of the outcomes
  the procedures yield: D1 gives its expectation under H-a–H-d, whatever the timing. (ii) The
  *availability of its value*: observable once `K` is consolidated; D1 says neither when nor
  whether. (iii) *Conditioning on consolidation*: given that `K` is consolidated by a time `t`, or
  even eventually, the conditional expectation of `Ŝ_K` can differ from D1's whenever completion
  times or the arrival of records depend on the outcomes. (iv) *The union of the cohorts
  consolidated by `t`*: a subset selected by completion times; fixing each cohort's members in
  advance does not remove that selection. D1 holds for a prefixed cohort and for any prefixed
  union of cohorts; it is not attributed to the cohorts already concluded at a given time.
- **Counterexample** (Astra; D, C). One-item cohorts, each with a completed report, `S = π = 1`,
  `b = ½`, `p = ¾`, `A` and `R` equiprobable, law independent of the report: the contribution is
  `3/16` on `A` and `−5/16` on `R`, mean `−1/16`. If `R` ends before `A` (a screen rejection ends
  at stage 1, a verdict `A` only after stage 2: §1, rows 5 and 8), at an intermediate time the
  consolidated cohorts are those ending in `R`, with mean `−5/16`.
- **Conditions kept apart.** Mathematical: H-a–H-d on a prefixed cohort for D1, H-e for §8.4's
  expected counts. Operational: every member's verifiable terminal record available, and the
  property below. Neither implies the
  other. No publication or reputation policy for intermediate values is introduced here.
- **Provisional before.** Terminal terms are fixed; with `P` the pending members, the cohort's sum
  lies within `Σ_{j ∈ P} 1/π_j` of its known part (D3): a bound on the final value, not an
  estimate of it. Cases 6 and 7 have no bound until a rule exists. Any use of a provisional value
  by the reputation is a decision outside D1.
- **Term against record.** The term is the log position by which the procedure's rule ends it; a
  terminal record is a verifiable entry. Past the term without a record the item stays in case 2:
  not `I`, not 0, not dropped.
- **Missing availability property.** Consolidation needs every selected item's terminal record
  within a bounded log interval after its term, every report completed or replaced (T58) and, in
  a drawn design, the draw's beacon round formed under its rule (`16` §5). None holds today: no
  runtime composes the pilot (§1), T58 is not implemented, a round needs `t` reveals (`16` §6).
  Without it a cohort can stay provisional indefinitely. Not solved here: an overdue rule that
  assigns a value (0, `I`, removal) lets whoever controls the late record choose that value,
  unless the value is shown invariant to that choice.

### 8.4 Information and reputation

| Count | Definition | Measures |
|---|---|---|
| `N_u` | the assignments of `R_u` | exposure |
| `O_u` | `Σ S_j` over cases 3–5 | observed terminal outcomes |
| `V_u` | `Σ S_j` over cases 3–4 | conclusive verdicts, the only evidence on `p` |

Counting them needs only the records, not C1. None of them is `Σ S_j/π_j`: under H-e its
expectation given `F_Φ` is `N_u`, an estimate of the exposure, not evidence (`16` §7); without
H-e that unbiasedness is not guaranteed. A 0 comes from a non-selection (no observation), an
observed `I` (no verdict), a verdict with `p = b` (D4: the report matched the baseline) or, if a
rule gave it, a missing reveal; only the third is evidence on `p`.

| Element | Today (L) | From D1 | Open |
|---|---|---|---|
| `SkillTrack` | `record_observed(score, π)` adds `score/π` and counts reviewed and scored; `record_unobserved` counts reviewed; a pending item counts nothing; `skill()` divides by reviewed | the numerator and the denominator of a prefixed cohort's final value | two counters for `N_u`, `O_u`, `V_u`: an observed `I` through `record_observed(0, π)` counts as scored like a verdict, through `record_unobserved` like a non-selection; reviewed equals `N_u` only once every assignment is recorded |
| Probation | `status` reads the scored count against `N_PROBATION = 30` | nothing | `O_u` (an `I` advances it without a verdict) or `V_u` (its pace set by groups' conclusiveness) |
| Shrinkage | `odds_weight(S_u, k_u)`, `k_u` the scored count | nothing: the weight's incentives lie outside the proof (`16` §4.5) | which count is `k_u`; `γ`, `k₀` on B-b's distribution (T25) |
| Cap | `3 × median` of the epoch's weights | nothing | its effect on incentives, unassessed |
| CUSUM | each observed score against the unweighted mean of observed scores, out of probation | nothing | whether observed `I` zeros enter (each adds `reference − k` while the reference exceeds `k`, so a group failing to converge moves it with no change of skill); the order of entry (§8.3); AT-REP-07's rates do not transfer |

**Counts and reports** (D, under H-b, H-c and H-e, with every terminal record available): per
judgment the expected increments of `O_u` and `V_u` are 1 and `c` under A, `α` and `α c` under C,
free of `u`'s report; under D they follow `A_G`, hence the reports (§4, D). These are statements
in expectation; the counts themselves are random.

### 8.5 Observation design

| | A — fixed groups, all piloted | C — group audit at `α` | D — activated by entry or drawn |
|---|---|---|---|
| Inclusion; information at report | `π = 1`, no observation draw; outcomes hidden until the reports freeze | `π = α`, drawn after the reports freeze; C1 and H0 need a source no one reads or selects before: `16` §6's residual | `π ∈ {1, ε}`, drawn after `Φ_G`; the same residual |
| Invariance under joint deviations | the prescribed set to pilot and `N_u` fixed before the reports: no companions drawn from entrants, no activation; the work actually done (retries, other pilots and re-validations sharing respondents), the timing and `V_u` not fixed, their effect on `Y` subordinate to H-c | audited groups report-blind; pool-entry pilots of unaudited entrants share resources unless separated or reserved | activation follows entries, so the number of groups piloted and their timing follow reports (§7.3); invariance not shown |
| Availability; consolidation | `O_u = N_u` only if every terminal record becomes available; no observation draw, while other roles of the beacon (assignment) remain | unaudited items consolidate at the draw (a beacon round); records for audited groups only | drawn groups wait for a beacon round, activated ones for their slowest member's freeze |
| Counts and reports | with every record available, `O_u`'s increment 1; `V_u`'s expected increment `c`, not fixed; free of the report under H-b, H-c, H-e | `α` and `α c`, report-independent; group draws change their variance (§4.1) | report-dependent through `A_G` |
| Costs; reuse (§4.1, declared scenario) | 667 slots a month against the reference's 350.175: +316.825, 95.0% of the declared capacity of 333.5; pool entry from the group's verdict, no reuse condition | from 350.175 (+0; `α = ε`, full reuse) to 683.675 (+333.5, 100.0% of 333.5; `α = 0.525`, separate); reuse needs one group, sampling, procedure and timing | between the reference and A, set by `K` and the unmeasured joint law of entries; reuse inside the group |

The slot counts are illustrative, on the declared scenario (§4.1): A's increment is not the
largest of the cases reported. They are no validated estimate of respondents, answers with
anchors, fits or times, which stay unmeasured (`15` D2, D4).

**Direction (Astra, review of `44f0dbd`, kept on `fa11791`): A is the main analytic reference
for deepening B-b; C stays the comparison even if A proves sustainable.** No adoption or
expenditure is approved, and no further alternative is developed now. What A gives and what it
does not:

- it removes the observation draw from the score: C1's draw and the beacon's behavioral model for
  that draw (`16` §9) no longer enter the scoring guarantee; other roles of the beacon in the
  protocol remain;
- it removes the channels of §7.3 that pass through entries: companions drawn from entrants,
  group activation, a path-dependent start;
- it fixes before the reports the prescribed set to pilot and `N_u`, not the work actually done,
  the timing or `V_u`; their effect on `Y`, and the procedure's residues of §7.3 (withdrawals,
  drift, model and thresholds, source check, respondents), stay subordinate to H-c;
- it gives `O_u = N_u` only if every terminal record becomes available (§8.3).

D is not recommended for B-b: its counts and load follow the reports.

**Open before any adoption.** (1) §8.3's availability property is missing; under A every
assignment depends on it. (2) Capacity, on §4.1's illustrative slot counts: A needs 667 slots
against the reference's 350.175 (+316.825, 95.0% of the declared 333.5); C from no increment to
+333.5 (100.0%); a resource commitment escalated with the design (`15`, working agreement 3). (3)
Residues of every design: cases 6 and 7, §8.4's counters, the procedure's closure at association
(§7.3), the bridging rating now forecasting `q_c` with `τ`, the band and the polarization cut
unvalidated for it (§7.5), and whether A1 needs the reputation's incentives (`16` §9).

### 8.6 Acceptance criteria for any implementation

1. Every assignment of `R_u` enters `N_u`; a test shows it.
2. A selected item keeps `S_j = 1` while pending and, without a terminal record, is never
   recorded as 0 or as `I`.
3. The contribution matches §8.1 in each case; an `I` adds 0 and counts in `O_u`, not in `V_u`.
4. A cohort's rule reads only the assignment record; a cohort's value is final only when every
   member is in case 1, 3, 4 or 5; no statement attributes D1 to the cohorts consolidated by a
   given time.
5. The recorded `π_j` is the design's inclusion probability (H-e); on an exact enumeration the
   IPW mean of a prefixed cohort's final value equals its expected contribution.

## 9. Cases 5–7 for a study under design A (reporting specification, approved on `b2a1c2f`)

**Status.** A proposal by Claude Code in `d31e9fa`, on the direction of Astra's review of
`26593e7` (`19` §10); it rewrites none of §§7–8. Reference, for a study only: design A (groups
prescribed before the reports, `π = 1`, §8.5), C kept as the comparison; B-b as the candidate
(§8.1); assignments and prefixed cohorts as in §8.3; any study score kept apart from the
reputation. It adopts no design, starts nothing in the field and changes no code, API,
serialization, threshold or golden output. Completed reports and a reached freeze are not assumed
where they are the problem (cases 6 and 7). **L** read at `26593e7`; **D** derived here; **C** one
exact calculation (§9.4).

Astra's review of `d31e9fa` (the commit, its parent and the published HEAD checked; the diff, the
documents, T58 and the relevant lifecycle arms read; a small rational calculation on the
baseline's dependence channel; no Rust, fit or campaign run): approved the distinctions between
the procedure's term, the terminal record and the study's snapshot; the ban on turning silence
into an outcome; the state's update kept apart from the historical snapshot. Not approved: §9 as
a whole. Decided for the study's reporting specification only: 6c and 7a as its basis, unresolved
cases left unresolved with no imputed value; this does not make the consolidated cohorts alone
representative. Not approved: S2's start, an implementation, penalties or reputational policies.
T58's deepening is no prerequisite of this perimeter; T58 stays open for a procedure that must
proceed despite missing reports. Rectified after that review in `b2a1c2f`: units and times
(§9.1), exhaustion's evidence and case 5 by unit (§9.2), freeze against consolidation and the
baseline's dependence (§9.4), reporting (§9.5), the decision's scope (§9.6).

Astra's review of `b2a1c2f` (the commit, its parent and the published HEAD checked; the diff and
the relevant documents read; the baseline example recalculated with rationals, 0 at `p = ½` and
3/16 at `p = ¾`; no Rust, fit or campaign run; the local checks and working tree are Claude
Code's evidence, not Astra's): the rectifications and §9 **approved as the study's reporting
specification and a conditional analysis, with no blocking finding** — the freeze, progress,
conclusion, record availability and consolidation kept apart; item, assignment and cohort apart;
access to reveals apart from a baseline's dependence and from a demonstrated violation of C5;
verifiable exhaustion apart from a resource shortfall and from silence; 6c and 7a imputing no
value, with no representativeness of the consolidated cohorts alone. Not approved: an
implementation, S2's start, penalties or new reputational policies; T58 stays open, no
prerequisite of this specification. Its non-blocking precision is made in §§9.2–9.3: `g(I) = 0`
does not depend on `p`, and an assignment with a missing report stays unscored for a contractual
reason.

### 9.1 Three objects, three units

- **Procedure term** `τ_j`: the log position at which the reference procedure's rule ends it
  (§7.2, item 3; §8.3), fixed before the reports, in log order. A protocol object.
- **Terminal record**: a reproducible entry stating the procedure's outcome, `A`, `R`, or `I` with
  §9.2's content. Its *existence* (the procedure produced it) and its *availability* (the log
  position at which it appears) are two facts; past `τ_j` without it, the outcome is pending.
- **Observation close** `Ω`: a log position fixed before the study starts, at which the study takes
  its snapshot (§9.5). A study object: it ends the observation, not the procedures, and changes no
  item's state.

Three units: the *item* (its procedure and outcome); a reviewer's *assignment* (`j ∈ R_u`, its
report completed or not); a *cohort* (a prefixed set of assignments). A terminal record describes
an item; §8.2's cases describe assignments; consolidation concerns a cohort. The cases are not
exclusive across units: an item can end in `I` while one of its assigned reviewers has no report,
case 6 for that assignment, which the item's record does not resolve.

Protocol rules read log order only (§7.2; invariant 7). Elapsed times may be measured as
operational quantities of the study; they enter no scoring rule. Each interval names its origin:
the association where it exists, the assignment otherwise (a freeze can precede the association);
intervals of different origins are kept apart, never substituted for each other. At `Ω` an outcome
without its terminal record is reported pending: not `I`, not 0, its assignments kept.

### 9.2 Case 5 — verifiable terminal inconclusiveness

**What the record must let a verifier check** (logical content; no schema or API proposed):

1. the item and the procedure fixed for it, with the prefixed rule that applies — attempt budget,
   term, the condition of terminal closure — by reference to the record that fixed them;
2. the group and its association record;
3. every attempt the rule requires, each with its status (a converged verdict or a named
   non-conclusion, §7.2, item 1) and the evidence to recompute it (answers, anchors, seeds, model
   version; §7.2, item 2), and no required attempt lacking that evidence;
4. no earlier conclusion that would already have ended the procedure;
5. the terminal reason and the rule's condition it meets;
6. its log position and the references needed to verify items 1–5.

An indeterminate last attempt does not by itself show exhaustion: items 1 and 3–5 do. Three
situations stay apart: *an indeterminate attempt* (the procedure continues if the rule allows);
*a procedure exhausted with evidence* (`Y_j = I`); *an attempt or a record that never arrived*
(no status: the outcome is pending). Exhaustion is never inferred from silence.

**Resources.** A shortfall of data by the term (respondents short of a floor) needs a positive,
verifiable record too: what the rule required by the term and what arrived. Three things kept
apart: that descriptive record; its classification under a candidate rule (whether it counts as
`I`); the conditions of the score's guarantees, among them §7.2, item 4's invariance, which a
declared hypothesis states and does not prove. No category is made normative here, and no
shortfall is declared to be `I`.

**Consequences, by unit, with a valid record.** *Item*: its outcome is `I` and available; nothing
about that outcome stays pending. *Assignment with a completed report*: under A, `S_j = 1` and
`π_j = 1`, so the term is `g_uj(I) = 0` (§8.1), the assignment stays in `N_u`, `O_u` increases and
`V_u` does not (§8.4); an observed outcome without a verdict, neither a non-observation (case 1,
absent under A) nor a pending outcome (case 2). *Assignment without a completed report*: case 6
stays. On the inconclusive branch `g(I) = 0` whatever `p`, so a value could be computed; the
assignment gets no term for a contractual reason: B-b is analysed for completed reports
(H-d), and no approved rule extends it to case 6. It is countable as an assignment with a missing
report whose item ended in `I`, not as a score term; this specification extends neither B-b nor
§8.4's counts to it. *Cohort*: its final value needs every member's contribution;
one member in case 6 leaves it without one, whatever the item's outcome.

### 9.3 Case 6 — report not completed

**Verifiable on the log**: the assignment (`j ∈ R_u`); a commitment, if one was recorded; the close
of the reveal period, in log order; the absence of a valid reveal before it. **Not identified by
the absence**: withholding, a reveal lost or censored, unavailability (§8.2). In every option
`p_uj` does not exist. On a conclusive outcome `g_uj` cannot be computed; on `I`, where
`g(I) = 0` needs no `p`, B-b still gives no term, since it is analysed for completed reports
(H-d) and no approved rule extends it, or D1, to this case. The case belongs to the assignment
and persists whatever its item's outcome (§9.1).

| Option | `p`, `g` | Denominator | Consolidable | Withholding; hypotheses |
|---|---|---|---|---|
| 6a. the absence reported apart, the rest scored | none; the case in a separate count of missing reports | kept in `N_u`: an implicit 0, which is a value; removed: the denominator follows `u`'s choice, against C4 | the absence is final at the reveal close; the score of the rest is not D1's estimator | with the implicit 0, withholding pays wherever the truthful expected contribution is negative (§7.5: `−6/25`); with removal, the reviewer chooses which items are scored |
| 6b. a penalty, candidate (`16` §4.4) | none; a fixed `s_ns ≤ −1` at `π = 1`, a value by rule, not a forecast score | `N_u` (C4 kept) | at the reveal close, for `u`'s term | withholding weakly dominated in `E[Ŝ_u]` only under `16` §4.4's hypotheses (the IPW identity relative to `F'_u`, C4, the deviation leaving others' terms unchanged, a penalty independent of outcome and draw); involuntary absences penalized alike, needing their own policy; a reputational rule, not approved |
| 6c. the case kept unresolved in the study | none; reported as a missing report | the cohort keeps it, with no complete final value | no: the cohort's known part only, no value or interval for the missing member | the study scores no withholding and does not model its cause: it measures how often reports are missing, not why |

`p = 0`, `p = b` (which gives `g = 0` exactly, §8.1 D4), `g = 0` or removing the item are each a
value or a selection, not a neutral reading. While no rule lets the freeze pass, the missing
report also leaves the item's freeze unreached for the other panelists: the absent reviewer's
case is 6, theirs is 7.

### 9.4 Case 7 — freeze not reached

**Today** (L). `Score` requires every panelist's reveal and otherwise returns `PartialEpoch`
(`lifecycle.rs:317–318`): the item stays in `Revealing`. T58 is decided, not implemented: at the
reveal deadline each missing panelist is replaced by a beacon-drawn reviewer of the same stratum
and the round reopens; after the second deadline the round is scored if at least 7 panelists
revealed, otherwise the item returns to the admission queue (`10` T58). The freeze can also wait
on the decision, the band's extra round or the appeal window (`16` §5).

**Five things apart**: passing the freeze; the item's progress; the procedure's conclusion; the
terminal record's availability; the availability of every contribution of a cohort. A replacement
and a quorum (7b), or a term (7c), can let the freeze pass and the item progress; they guarantee
none of the last three. Pilots can stay pending, records missing, other members' contributions
incomplete, and assignments unresolved after a re-queue; the number of rounds before a re-queue
bounds no overall delay.

| What can change | 7a. pending kept | 7b. T58's replacement and quorum | 7c. a candidate term on the present reveals |
|---|---|---|---|
| panel, baselines | unchanged | a replacement joins and reports with the others' reveals public: access to information, not by itself a dependence; C5 fails only if the panelist's deviation actually changes its baseline through the replacement's behaviour and the aggregation rule (below); a re-queue starts another panel | a smaller panel, each baseline over fewer reports |
| reviewers' information | unchanged | the replacement reports knowing the revealed reports | unchanged for those who revealed |
| reference outcome | under A, the procedure runs whatever the decision; its outcome stays hidden until the freeze (§8.5) | the gate decision can change with the replacement (`16` §4.4: a motive outside the score); a re-queue leaves open what happens to the first panel's `R_u` | as under 7b, from a rump decision that T58 itself refuses |
| group association, load | if the group's pilot waits for the freeze, its companions' timing moves (§7.3) | the same, for a delay that no count of rounds bounds | the same |
| H-a–H-e | unchanged; the cohorts holding the item stay unconsolidated | H-d's C5 at risk as above; H-c through the panel and the timing | H-c through the panel; H-d only for the scored reports |

**The baseline's dependence channel** (D, C). An abstract construction, not a simulation of the
protocol nor a call to the baselines' API, and no evidence that a real replacement would respond
so: the outcome always conclusive, Bernoulli(½), independent of the report; an effective baseline
`b(p) = min(1, max(0, 2p − ½))`; the usual differential Brier contribution. The expected
contribution is 0 at `p = ½` and 3/16 at `p = ¾`, so the truthful forecast ½ is not a maximizer.
The approved compositions of the baselines (`16` §4.6) stand; no rule including replacements is
decided.

No option settles incentives, availability or invariance by itself. *On the reviewer who does not
complete*: case 6, under §9.3's options. *On the other members of the cohort*: under 7a their
cohorts stay unconsolidated through an item they cannot complete, the block T58 names as an
attack; under 7b or 7c the freeze can pass, and their cohorts consolidate only once the procedure
concludes, its record is available and every member's contribution exists.

### 9.5 Study reporting and late records

**Describable at `Ω` without consolidation**, from the records alone, by unit. *Items*: procedures
started, attempts executed, terminal outcomes by category (`A`, `R`, `I`, and the shortfalls
recorded under §9.2 apart), outcomes pending. *Assignments*: `N_u` and totals; reports completed
and missing, with or without a commitment, each with its item's state, so that a missing report
whose item ended is counted as such. *Cohorts*: consolidated or not, and why. Pending cases by kind
(an outcome pending, case 6, case 7), each with its observation time from a named origin (§9.1).
The share concluded by `Ω` is not a probability of eventual conclusion: pending items are censored
at `Ω`, and conclusion times can depend on the outcome (§8.3's counterexample).

**Scores.** A final value only for consolidated prefixed cohorts. For a cohort whose members lack
only pending outcomes (case 2), its known part and §8.3's bound, within `Σ_{j∈P} 1/π_j` (under A,
the number of such members). Cases 6 and 7 get no value and no interval without a rule. The mean
over the cohorts consolidated by `Ω` does not inherit D1 (§8.3, (iii)–(iv)).

**Late records.** Two views, apart. The *state*: every valid record, in log order, moves its unit
to its case whenever it appears; the protocol knows no late record unless a term says so. The
*study snapshot*: what was recorded by `Ω`, reported with `Ω`; a record after `Ω` changes the
state, not the snapshot, and a later prefixed close `Ω'` gives a second snapshot, reported as
such. No deletion, substitution or retroactive value without an explicit rule.

### 9.6 Summary and decision

| Case | Evidence available | Treatment in the study's reporting | Computable | Stays pending | Decision still needed |
|---|---|---|---|---|---|
| 5 | §9.2's record: rule, every required attempt with evidence, no earlier conclusion | the item's outcome `I`; for assignments with a report, `g(I) = 0`, `O_u` up, `V_u` not; shortfalls recorded apart | the item's outcome; the terms and counts of completed reports | nothing for the item's outcome; any assignment of it in case 6, and its cohort's final value | whether a recorded shortfall is classified as `I`, under which candidate rule and evidence |
| 6 | the assignment, a commitment if any, the reveal close, the absence | 6c: a missing report, its cohort unresolved (Astra's decision for the reporting) | the counts; the other members' known terms | the cohort's final value | for any score, 6a, 6b or another rule; a penalty only as a reputational rule, not approved |
| 7 | the reveals present, the deadlines, `PartialEpoch` | 7a: pending, with its observation time (Astra's decision for the reporting) | the counts; the revealed reports | the item, and every cohort holding it | for a procedure that must proceed, T58 or another rule (open) |

**Scope of the decision.** 6c and 7a are the basis of the study's reporting specification, not a
general answer to withholding or availability. They impute no value to unresolved cases; they do
not remove selection, censoring or the dependence of times on outcomes; they do not authorize D1
on the consolidated cohorts alone.

**Next.** The review of these rectifications; then, as the priority link, the bridging input's
meaning when the report forecasts `q_c` (`19` §10.1), not developed here. The study is not ready,
and A1 stays open: these cases are made representable and verifiable, not settled by giving up
its guarantees.

## 10. The bridging input under B-b (conditional analysis, approved on `111e595`)

**Status.** A proposal by Claude Code in `ac06d3c`, on the direction of Astra's reviews of
`d31e9fa` and `b2a1c2f`; §9's approval does not cover it. It asks what information enters the
bridging when B-b elicits `q_c = P(Y = A | Y ≠ I, F_u)`. It adopts no protocol, changes no code,
API, commitment, threshold or calibration, and gives `Y` no new operational definition: a
candidate is analysed. **L** read at `ac06d3c`; **D** derived here; **C** exact calculations
with fractions in a scratch script, formulas in place.

Astra's review of `ac06d3c` (the commit, its parent and the published HEAD checked; the diff,
the contracts and the relevant code read, the harness's bootstrap path included; small exact
calculations with fractions; no Rust, fit or campaign run; Claude Code's local checks not
repeated, the working tree not checked through GitHub): approved, under the stated hypotheses,
§10.2's results — `P(Y = A | F_u) = c q_c` for `c > 0`; `q_c` alone does not fix it; at `c = 0`
the score is indifferent among reports; the expected loss of contribution `c(p − q_c)²`, a
property of the score, not a demonstrated measure of the reputational incentive or of the
reviewer's overall utility. Not approved: §10 as a whole. Direction for the next specification:
α the main reference, β a comparison with §10.3's information obligations; no threshold, pool
entry, implementation or start of the study approved. Rectified after that review in `111e595`:
the path (§10.1); the three events and the scale comparison (§10.2); the alternatives' meaning
and the reconstruction through an estimate (§10.3); time under A and the shared parameters
(§10.4); §10.5.

Astra's review of `111e595` (the commit, its parent and the published HEAD checked; the diff of
the four documents read; the relevant code and harness passages checked again; the example
checked with fractions, 9/50 and 18/25 against the public product's 9/20; no Rust, fit or
campaign run; Claude Code's local checks not repeated, the working tree not checked): the
rectifications and §10 **approved as a conditional analysis of the join between B-b and the
bridging, with no blocking finding** — the path's steps kept apart; empirical success,
conclusion and pool entry as distinct events; α keeping the scalar format while changing the
forecast's meaning; a public probability of conclusion no automatic substitute for the one given
the reviewer's information; A alone guaranteeing no invariance of resources, times and outcomes;
influence on shared parameters limited to the fit's participants, A7 preserved. Not approved: a
protocol, thresholds, pool entry, an implementation or S2's start. Direction: the documentary
specification of timing under A (§11), α the main reference and β the comparison.

### 10.1 The current path, from a report to a decision

| Step | Implemented (L) | Documentary contract | Composition today |
|---|---|---|---|
| individual report | one `prob` in `[0, 1]`; `review::commit` hashes its eight bytes (`isegoria/commit/v2`); the lifecycle refuses a reveal outside `[0, 1]` | `p_uj`, the probability that the item passes Level B validation (`02` §C.2); the same number is the rating `r_uj` (`01` D23, D33) | `review::submit_review` through `NodeState::apply` |
| ratings matrix | `orchestrator::weighted_ratings` builds `Ratings` from a matrix, with D33's weights and the axis floor (`axis_mask`) | `02` §§A.1, A.4 | no production path fills it with an epoch's reveals |
| fit, side-balanced score | `scoring::bridging::fit`: the collective fit of the participants (on the axis, positive weight), the others placed on the fixed axis; `side_balanced`: `S_j = (A_j + B_j)/2` over the participants' sides | `02` §§A.1, A.3 | — |
| robust score | `bridge_scores`: the minimum of the full fit's side-balanced score and of bootstrap subsamples' (participants' observations kept with probability `keep_frac`) | `02` §A.4: the minimum over `m = 10` subsamples, about 15% removed | only the characterization harness calls it (`BOOTSTRAPS = 10`, `KEEP = 0.85`) |
| gap, coverage | the full fit's `SideScores::gap`; `coverage` over the full fit's sides | `02` §A.3 | in the harness, from the full fit |
| first decision | `gate::bridging_gate(score, gap, coverage, τ, ε, appeal_gap)`, an API taking its values from the caller: under `MIN_COVERAGE = 1` the band; pass at `≥ τ + ε`; the band `[τ − ε, τ + ε)`; below, appealable if the gap reaches `APPEAL_GAP`, else rejected (`TAU = 0.80`, `EPS = 0.02`, `APPEAL_GAP = 0.25`) | provisional values argued on the reference simulation for the current meaning (`02` §A.3; T25) | the harness passes the robust score with the full fit's gap and coverage (`characterization::run::gate_char`); `orchestrator::run_item` steps the lifecycle on an outcome its caller supplies |
| supplementary review | `gate::supplementary_review`: a fresh `fit` over the expanded ratings, its side-balanced score against the plain `τ` if covered, else the below-band rule | `01` D26; `02` §A.3 | in the harness; in the lifecycle, a `redecide` its caller supplies |

The scored outcome today is `exploration::outcome_of`: pool, contested or retired 1, rejected at
screen or DIF 0, `Measured` its pass. Under design A every item is piloted for measurement; band
and appeals change pool entry, never scoring (§4, A).

### 10.2 What a B-b report carries

Three events, apart: the reference procedure's empirical success, `Y = A`; its conclusion,
`Y ≠ I`; the item's entry into the pool, which under A also needs the gate (§4): an item piloted
for measurement only can reach `Y = A` and stay out. `P(Y = A | F_u)` is not, in general, a
probability of entry.

With `c = c_uj = P(Y_j ≠ I | F_u)` and `c > 0`: `P(Y = A | F_u) = c q_c`,
`P(Y = R | F_u) = c(1 − q_c)`, `P(Y = I | F_u) = 1 − c` (D). `q_c` alone does not fix
`P(Y = A | F_u)`: with `q_c = 9/10` it is 9/10 at `c = 1` and 9/20 at `c = ½` (C). Set against
`τ ± ε` the two numbers fall either side of today's band, a comparison on the scale only: a
bridging decision reads the robust side-balanced score of every participant (§10.1), so it does
not show that two reports would give different decisions. At `c = 0`, `q_c` is undefined and the
score is indifferent among reports (§8.1, D1). For `c > 0`, D1 gives the expected loss of
contribution from a report `p ≠ q_c` as `c(p − q_c)²` — a shift of 1/10 costs `c/100` (D; C) — a
property of the score, not a measure of the reputational incentive or of the reviewer's overall
utility. Eliciting `c` would change the contract, the commitment and the incentive analysis (no
proper rule scores `c` under B-b, §7.5); none is introduced.

### 10.3 Two alternatives

| | α. `q_c` as the input | β. `P(Y = A \| F_u)` reconstructed |
|---|---|---|
| what the rating forecasts | the empirical success given conclusion | the empirical success, not conditioned on conclusion; neither is pool entry |
| report, commitment | the scalar format, and the current commitment can stay; the forecast's semantic contract changes | needs `c_uj`: elicited from `u` (a second report, or §7's vector with a vector commitment and a simplex check, §7.4), or replaced by an estimate (below) |
| incentives | D1 for `q_c` under H-a–H-d, the expected loss scaled by `c` (§10.2) | elicited: B-b does not score `c`, so a proper rule for it, or §7.1's ternary score, and its analysis; estimated: below |
| `τ`, band, polarization | to revalidate for the new meaning (T25, T83); no change of their values is shown necessary | the same, for the quantity actually used |

**Reconstruction through an estimate.** The identity needs `c_uj = P(Y_j ≠ I | F_u)`. A public
frequency of conclusion, per item or category, rests on information other than `F_u`; even known
exactly it need not equal `c_uj`. Astra's abstract example (C; not a fit nor a simulation of the
protocol): two equiprobable private signals give `c_uj = 1/5` or `4/5`, with `q_c = 9/10` in both;
the public frequency is ½; the individual probabilities of `A` are 9/50 and 18/25, while the
public frequency times `q_c` gives 9/20 in both. Four things apart: the public probability; the
probability given the reviewer's information; an estimate of either; its error. `ĉ_j q_c` is a
candidate indicator. Identifying it with the individual forecast needs hypotheses to be declared,
not presumed — that `F_u` carries nothing on conclusion beyond the public information, and a
treatment of the estimate's error, of pending procedures and of selection; its gap would also
mix disagreement on `q_c` with `ĉ_j`.

§7.4's transformations (`p_A`, `p_A/(p_A + p_R)`, `p_A + p_I/2`) need §7's ternary vector; under
B-b's scalar only α asks for no further elicitation, and `p_A/(p_A + p_R)` equals `q_c` (D). A
separate, unscored rating (§7.4) departs from D23 and D33 unless decided otherwise. Fitting the
scalar API validates neither meaning.

### 10.4 Can the gate's decisions move `Y`?

- **Under A** (§4, §8.5): the gate decides neither whether an item is piloted nor with which
  group; A fixes the prescribed set to pilot. It does not make invariant the work actually done,
  the samples, the timing or the outcome's law (§8.5).
- **Three things in time**: the production of the outcome; the availability and publication of
  its record; the information reviewers have before their reports. The freeze `Φ_j` includes the
  band's extra round and the appeal window (`16` §5), and outcomes must stay unknown to reviewers
  until it (§4, A). A decision that lengthens a freeze can move when a procedure runs, if it
  waits, or only when its record is published; even then, what reviewers of other items know
  before their reports is to be checked (§7.3, *Time* and *Load*) — an obligation, not a
  demonstrated violation.
- **Shared parameters.** A reviewer who participates in the collective fit (on the axis, with
  positive weight) moves the shared parameters (`μ`, `b_j`, `f_j`, the other participants'
  positions, the sides) through its ratings, and so other items' scores: a deviation on one item
  can send another to the band, hence move a freeze. Reviewers outside the collective core
  (under the axis floor, or at weight 0) are placed on the fixed axis and change none of them
  (`02` §A.4; `15` A7, closed).
- **A channel to H-c** exists only if such a change moves the joint law of `(b_uj, Y_j)` given
  `F_u`. A term in attempts and procedures that do not wait for freezes do not exclude one by
  themselves: shared resources, the samples available and the rules of execution can remain
  channels (§7.2, item 4; §7.3). A possible dependence, not a demonstrated violation. Under C or
  the reference design the gate also decides which items are piloted for entry (§8.5).
- **Keeping study scores out of the reputation** changes none of this: the gate reads the reports
  whatever their score is used for, and a reviewer's preference over an item's fate stays a
  motive (`16` §4.5, item 4) that the expected loss `c(p − q_c)²` prices within the score only.

### 10.5 For Astra

- **Specifiable now.** §10.1's layers; for a study under A, every report, ratings matrix, robust
  and full-fit score, gap, coverage, gate decision, band and appeal recorded with its log position
  beside §9's records.
- **Design decisions.** By Astra's direction α is the main reference and β the comparison. For α:
  the forecast's semantic contract restated (the empirical success given conclusion) and `τ`, `ε`
  and the gap revalidated for it (T25, T83). For β: the source of `c_uj`, elicited (with its
  contract, commitment and incentive analysis) or estimated (with §10.3's hypotheses declared).
  What the gate protects under B-b, and whether a study applies the gate's decisions to pool entry
  while it runs, stay design questions.
- **Experimental evidence**, as measures against criteria declared beforehand: the gate's scores
  and gaps under the chosen meaning; conclusion frequencies by category, public quantities that
  bear on `c_uj` only through declared hypotheses, with pending procedures, selection and
  uncertainty; the delays bands and appeals induce. None validates `τ`'s meaning by itself.
- **Next.** The review of this rectified §10; the specification of timing under A comes after it.
  Nothing here is adopted.

## 11. Timing under design A (conditional comparison, approved on `cf5575a`)

**Status.** A proposal by Claude Code in `82bdb9a`, on the direction of Astra's review of
`111e595`; §10's approval does not cover it. Analytic reference: A's prefixed groups (§4), B-b
(§8), the input α (§10), §9's reporting with 6c and 7a; none authorizes starting the study. It
aims to make the order of events verifiable and to name what a construction needs; it seeks no
general answer to availability, scheduling or T58, and changes no code, API, serialization,
threshold or golden output. **L** read at `82bdb9a`; **D** derived here; **C** two exact
calculations (§11.3).

Astra's review of `82bdb9a` (the commit, its parent and the published HEAD checked; the diff of
the four documents, the relevant contracts and the replay and results code read; exact
calculations with fractions on the timing counterexample and on the disclosure example of
§11.3; no Rust, fit or campaign run; Claude Code's local checks and working tree not checked):
approved the record of `111e595`'s approval; the setting of the comparison between T1 and T2;
the timing counterexample as an abstract construction (0 and 1/25); the pending cases kept, and
production kept apart from disclosure as problems to specify. Not approved: §11 as a whole; a
preference for T2 as the construction adopted; a protocol, an implementation, thresholds, pool
entry or S2's start. A and α stay analytic references within their recorded limits; T1 and T2
stay alternatives to compare. Rectified after that review in `cf5575a`: a partial order
with availability and disclosure apart (§§11.1–11.2); disclosure and baselines, production and
information, D1's scope (§11.3); respondents and the records' evidentiary limits (§§11.3–11.4);
§11.5.

Astra's review of `cf5575a` (the commit, its parent and the published HEAD checked; the diff of
the four documents read; `Node::submit`'s path checked again; the disclosure–baseline example
checked with fractions, 0 without the signal and −¼ with it; no Rust, fit or campaign run;
Claude Code's local documentation checks and the working tree not checked): the rectifications
and §11 **approved as a conditional comparison between T1 and T2, with no blocking finding**.
Accepted, kept apart: a partial order and one total sequence; production, availability, the
condition of opening and the actual opening; a disclosure that changes the information of later
reports and a mere selection of the observations available; the baselines' composition and the
invariance of the reports feeding them; the theorem's perimeter and actions in other roles; what
the records attest and what they do not show. Not approved: the adoption of T1 or T2, a
protocol, an implementation, thresholds, pool entry or S2's start. Its non-blocking precision is
made in §11.4: the sequence "check, apply, then write" is `Node::submit`'s.

### 11.1 Units, objects and a partial order

Units: an item `j`; an assignment `(u, j)`; a pilot group `G`, A's items fixed together; a cohort
`K` (§8.3), which may span groups. Points and objects, for `j ∈ G`:

1. **Group record**: `G` and its reference procedure (rule, attempt budget, term, model version,
   thresholds), fixed before any member's assignment (§4, A). No such record exists in code.
2. **Reports and gate**: `AssignReviewers`, `Commit`, `CloseCommits`, `Reveal`; `Score` (refused
   with `PartialEpoch` while a reveal is missing); for a band item the extra round and `Resolve`;
   for an appealable rejection `Appeal` or `AppealExpires` (`lifecycle::Event`). Under A they
   decide pool entry, not piloting (§4).
3. **`Φ_j`**, unchanged from `16` §5: the first log record after which `j`'s reports affecting
   `D_j` or a scored baseline, `D_j` and its re-decision, the appeal and the memberships
   `j ∈ R_u` are irrevocable. Computable from the log; no record names it today.
4. **`Φ_G`**: the latest `Φ_j` over `j ∈ G` — a name for a group's closure, not a change of `Φ_j`.
5. **Objects of `G`'s procedure** — answers, attempt statuses, intermediate fits, terminal
   records — each with four moments apart: its *production*; its *availability*, internal to
   whoever runs or holds it, and its entry into the replicated log, a later and distinct event;
   the *condition* that would authorize disclosing it; its *actual disclosure*.
6. **Consolidation** of a cohort, once every member is in case 1, 3, 4 or 5 (§8.3, §9).
7. **`Ω`**, the study's snapshot (§9.1).

These points form a partial order, not one sequence valid for both constructions. In both: the
group record precedes its members' assignments; a member's reports precede `Score` and
`Resolve`; `Φ_j` follows them; an object's disclosure follows both its availability and its
condition, and if either is missing the object stays pending. Production follows `Φ_G` under T1
and a position the group record fixes under T2 (§11.2), so under T2 it can precede `Φ_j`. `Ω`
can precede any production, disclosure or consolidation and photographs what is pending; a freeze
not reached by `Ω` is not a freeze that will never come. Log order, attempt counts and elapsed
time are different quantities: no finite count of rounds or attempts bounds a duration.

**Conditions of disclosure.** A record of `j` can be conditioned on `Φ_j`, or on `Φ_G`: one
group's verdicts come from one fit and are dependent (§4.1), so a record opened after `Φ_j` but
before `Φ_G` informs the members still before their freeze. Respecting `Φ_G` does not require
every object of `G` to be available: a record available after `Φ_G` opens when it is available. A
collective policy — opening `G`'s records together, so waiting for all of them — is a separate
choice. No new rule closing reports or appeals is introduced, and `Φ_j` is not redefined.

### 11.2 Two minimal constructions

| | T1. start after the freezes | T2. produce early, disclose under conditions |
|---|---|---|
| rule | `G`'s procedure starts at a log position after `Φ_G` | `G`'s procedure starts at a position the group record fixes, independent of reports; each object opens once available and its condition holds (§11.1) |
| prerequisites | the group record; `Φ_j` as a rule of the log; a start rule in log order | the same, plus a hypothesis or a construction controlling information: what is produced before its condition stays unread by reviewers and verifiable afterwards (a commitment on the log at production, opened later, is one shape; no design here) |
| missing report, freeze not reached | while `Φ_G` is not reached no member's procedure starts; at `Ω` the group's outcomes are pending, not lost | production unaffected; every object whose condition needs that member's freeze stays undisclosed |
| samples, resources, time, outcome law | the start moves with the slowest member's freeze, so with reports, bands and appeals, and samples, shared resources and drift can follow it (§7.3) | a fixed log position does not make resources, samples or the outcome's law invariant: production is decoupled from reports only if they are fixed independently of them (§7.2, item 4) |
| reviewers' information | the planned procedure does not run before `Φ_G`; reviewers may still hold private signals, answers or correlated results (§11.3) | outcomes exist before the reports; keeping them from reviewers rests on the information control |
| evidence | the log records of the reports and the gate, the start after `Φ_G`, §9.2's attempts, within the records' limits (§11.4) | the same, the commitment at production and its opening; that nothing was disclosed early is shown by no log record (§11.4) |
| excludes | the planned execution before the group's freezes | a start that depends on reports, under the resource condition |
| hypotheses left | samples and resources invariant in time; no prior information on the outcome; availability | the information control; the resource condition; the respondent channel |

### 11.3 Joins with the score's conditions

- **D1's scope.** D1's target `q_c` is conditioned on the information `F_u` declared in §8.1, and
  D1 holds under §8's hypotheses (H-a–H-d). Information wider than declared, or whose
  availability depends on a strategy, is no automatic check of those hypotheses. Under A no
  observation is drawn: that removes C1's role, not the need for H-c and C5. §4 (A) asks that
  `Y_j` stay unknown to `j`'s reviewers until `Φ_j`: T1 prevents the planned execution before
  `Φ_G`, not every source of information; T2 depends on its information control.
- **Timing and the outcome's law** (D, C; abstract, not a model of the protocol). `u`'s report
  alone sends `j` to the band when `p ≤ ½`, as in `16` §3's simplified gate; under T1 the band
  moves `Φ_G` past a log position after which the sample gives `A` with probability 2/5 instead
  of 3/5; the outcome always conclusive; `b = 3/5`. The truthful report `p = 3/5` earns 0 in
  expectation, the deviation `p = 2/5` earns 1/25: H-c fails through the outcome's law. Under T2
  the same deviation leaves production where the group record fixed it, but not necessarily the
  resources or samples (§11.2).
- **Disclosure and baselines** (D, C; Astra's abstract construction, not an attack on the
  protocol nor a gain over a cohort). `Y_j = Z`, `Z` Bernoulli(½), the same on both paths;
  `p_uj = ½` on both. Without a signal, the reports that form `b_uj` give `b_uj = ½`; with a
  signal revealing `Z` before those reports, they give `b_uj = Z`. The expected contribution is 0
  and −¼. If a deviation of `u` on another assignment decides whether that signal arrives before
  those reports — by moving a freeze that conditions a disclosure, for instance — `b_uj` and the
  joint law of `(b_uj, Y_j)` change: a channel toward C5 and H-c. Keeping `16` §4.6's composition
  of the baselines, which stays as approved, does not show that the reports it reads are
  invariant under a deviation. So moving a disclosure is not only a selection of what is
  available.
- **Three things apart.** Information legitimately available and fixed across the strategies
  compared, which a forecast includes; information whose availability depends on the deviation,
  where H-c and C5 are involved, as in the two constructions above; the selection of the
  observations or cohorts available (§8.3, (iii)–(iv)), under which D1 does not carry to the
  consolidated cohorts alone. Not every piece of additional information is a violation; each case
  names the condition involved.
- **Respondents.** Four things apart: a person may hold both roles (invariant 5: role pseudonyms
  are not linkable); the current implementation has no check keeping a reviewer from answering
  its own items (`pilot::submit_response` checks the role and one nullifier per batch and epoch);
  the theorem's deviations are a reviewer's reports, and C6 does not cover a respondent's
  (§7.3); constructions compatible with anonymity and unlinkable roles are neither excluded nor
  designed here. §7.3's statement that sampling cannot exclude a reviewer from its items' samples
  holds for the current implementation and for sampling that reads role pseudonyms alone; it is
  not shown for every such construction. Under T2 a respondent can answer before reporting, under
  T1 only after.
- **No invariance by declaration.** A start rule with no report among its arguments shows no
  invariance by itself: its inputs — freezes, resources, samples — can depend on reports.

### 11.4 Pending outcomes, records and anonymity, the gate

- §9's decisions stand: a missing report imputes nothing; an unreached freeze stays pending;
  silence and the study's end are no terminal `I`; an item's outcome does not complete an
  assignment's contribution; consolidated cohorts are not representative by construction. An
  exhaustion rule needs positive, verifiable evidence (§9.2); whether a resource shortfall counts
  as `I` is not decided here.
- **What the records show.** `NodeEvent::AdmitRespondent` admits a respondent (its role nullifier
  in the batch and epoch's set): it records no answer and proves no pilot run. Lifecycle events
  (`Score`, `Resolve`, `Pilot1Batch`, `Pilot2Batch`) carry readings their caller supplies. A
  Merkle root and its inclusion proofs (`results::{inputs_root, inclusion_proof}`) bind and verify
  inputs within those APIs' limits; they show neither secrecy, nor a correct verdict, nor the
  absence of an early disclosure. A result held internally is distinct from its event entering
  the replicated log (`Node::submit` checks and applies the event through `NodeState::apply`,
  which writes nothing, then writes it to the node's local object store and log; replication
  between nodes is a later, distinct step, `04` §Replication between nodes).
- **Publication.** Answers and ratings are meant to stay off the log, committed by a root
  (`08` PRIV-004). Each reveal, however, is a lifecycle step carrying a judge's nym and rating on
  the replicated set, the discrepancy with D17 already recorded in `08` PRIV-004 (T73, T18; T77):
  a known discrepancy, not a new A1 finding. A study must not extend it; T2's records, before and
  after their disclosure, must link no role to a person.
- **Gate and pool during S2**, not chosen here. The choice bites in three places: whether `Φ_j`
  waits for appeal windows whose only effect would be on the pool; whether pool administration
  shares respondents with pilot batches (§7.3, *Load*); what an entry tells reviewers before other
  reports.

### 11.5 For Astra

- **What the comparison shows so far.** T1 moves the dependence on reports into time and samples
  (§11.3's timing construction) and holds a group's production while any member's freeze is
  pending. T2 decouples production from reports by rule only under a resource condition and needs
  an information control the code does not provide. Under either, a disclosure whose timing
  depends on a deviation can reach baselines through later reports (§11.3). Neither is adopted;
  the earlier preference for T2 is withdrawn.
- **Open obligations.** The group record; `Φ_j` and `Φ_G` as log rules; disclosure conditions per
  object and any collective policy; the information control and its verification; the resource
  and sample condition; the respondent channel; D17 for reveals (T77); a procedure that must
  proceed despite missing reports (T58); the gate's effect on the pool during S2.
- **Next.** The review of this rectified §11. The disclosure requirement per object is not
  developed before it.

## 12. Access and disclosure per object (conditional analysis, approved on `7d8db08`)

**Status.** A proposal by Claude Code on the direction of Astra's review of `cf5575a`; §11's
approval does not cover it. References, within their recorded limits: A's prefixed groups (§4),
B-b (§8), α (§10), §9's reporting with 6c and 7a, §11's comparison, T1 and T2 alternatives and
neither assumed adopted. It states what each construction needs to handle information, object
by object; it designs no cryptography, runtime or scheduler, leaves `Φ_j` unchanged and T58
outside, and changes no code, API, serialization, threshold or golden output. **L** read at
`cf5575a`; **D** derived here; **C** one exact calculation (§12.4).

Astra's review of `8af257d` (the commit, its parent and the published HEAD checked; the diff and
the relevant code read; the result `−δ²` checked with fractions; the code's encoding and hashes
reproduced in Python on a synthetic two-leaf Merkle construction; no Rust, fit or campaign run;
the local working tree and Claude Code's documentation checks not checked): **partial**.
Approved: the record of `cf5575a`'s approval; roles as functions, kept apart from persons; the
setting of the matrix per object; the partly informative signal's result under its stated
hypotheses (§12.4: `−δ²`, so −1/16 at `δ = ¼` and −¼ at `δ = ½`). Not approved: §12 as a whole;
an implementation, a cryptographic solution, thresholds, pool entry or S2's start. Direction:
T1 the reference for the next comparison of informational requirements and T2 the comparison,
T1 not adopted as a protocol and its timing channel open; (ii) and (iv) made concrete, their
scope bounded; (i) may stay a declared hypothesis, no demonstration of H-c. The inclusion-proof
observation of §12.1 is recorded apart from A1, as `08` PRIV-004.1. Rectified after that review
in `7d8db08`: the properties and the code's write path (§12.1); the matrix, with the
code, what a requirement asks and candidate policies apart, the shortfall record and pool entry
(§§12.2–12.3); the constructions between groups, each with its channel, hypotheses, residual
channels and kind, the general sufficiency claimed in `8af257d` withdrawn (§12.4); §12.5.

Astra's review of `7d8db08` (the commit, its parent and the published HEAD checked; the diff and
the documents read; a small calculation with fractions on the conditional target; no Rust, fit,
campaign, Merkle reproduction or local documentation check; the working tree and the local
checks stay Claude Code's evidence): the substantive rectifications of §12 **approved as a
conditional analysis of the informational requirements, with no new blocking finding on its
setting** — the channels (i)–(iv) exclude, their further hypotheses and residual channels;
cryptographic properties, requirements and candidate policies apart; the local write apart from
replication; the shortfall's descriptive record apart from the refused transition and from `I`;
an entry implying `A` apart from an absence of entry; `08` PRIV-004.1's counterexample, limits
and provenance; the rectified promises of `04` and `08`. Not approved: the adoption of T1 or T2,
a protocol, an implementation, a cryptographic solution, thresholds, pool entry or S2's start.
Its two precisions are made: in §12.4, (i) states the invariance of the whole outcome's law
(below); in `08`, PRIV-004.1's outcome reads as a refuted general claim, its remediation open.

### 12.1 Roles, properties, what the code gives

**Roles are functions, not subjects.** The *runner* executes a group's procedure; a *holder*
keeps an object between its production and its opening (the runner or another); a *verifier*
recomputes from the inputs (under D17 the consortium, until proofs of the computation: `08`
PRIV-004); the *scorer* computes `g_uj` (§8.1); *reviewers*, *respondents* and the *author* act
as in the lifecycle; a *peer* reads the replicated set. One subject can hold several roles, and
naming two roles shows no two persons: role pseudonyms are unlinkable (invariant 5), and nothing
keeps a runner, holder or verifier from also being a reviewer or a respondent. No check that
would link pseudonyms, and no personal data, is proposed.

**Six properties apart.** (P1) authenticity — the object comes from the producer it names — and
integrity — it is unaltered with respect to a reference; (P2) availability: it can be obtained
when the procedure or a verifier needs it; (P3) confidentiality: who can read it before its
condition; (P4) correctness: a result is the procedure's function of its recorded inputs; (P5)
no early disclosure: the object reached no one beyond its authorized holders before its
condition; (P6) H-c's invariance (§8.1). What a primitive gives, and what it does not:

- a hash, a commitment or a Merkle root binds content to a reference value; it authenticates no
  producer by itself, and integrity checked against the reference presupposes that the
  reference's provenance is established otherwise (a signature, a log entry of a known writer);
- a signature authenticates content relative to a key — that the key's holder signed it — and
  shows neither the content's truth nor the computation's correctness (P4 needs a recomputation
  from the inputs, or a proof);
- replication and an availability rule are constructions that can give P2 (§8.3), not
  requirements of every study;
- P3 needs restricted access or a hiding construction; P6 is a property of laws, which no record
  shows.

P2–P6 stay distinct: an available record is not thereby correct, a confidential one not thereby
undisclosed early, and none of them shows P6. For P3 and P5 three levels stay apart: authorized
access (who may read), an assumption on the holders' behaviour (who does not pass it on), and
what a third party verifies — for P5, an early record when one exists. The absence of a
disclosure on the log does not show that no one learned it (§11.4).

**What the code gives** (L). `Node::submit` checks and applies through `NodeState::apply`, which
writes nothing, then writes to the node's local object store and log (`04` §A node's own disk);
replication, a later and distinct step, brings every infrastructure node the same set (`04`
§Replication between nodes), which any peer reads in the clear (`04` §Who reads, `10` T77):
today an object entering the replicated set is disclosed to every peer. `review::commit` hashes
a value with a nonce the caller supplies: binding, hiding only while the nonce is secret and
unpredictable. `results::inputs_root` binds the set of inputs; what an inclusion proof lets its
holder infer about other inputs is recorded apart from A1, as `08` PRIV-004.1 (open).

### 12.2 Objects

Five objects of a group `G` under A. Three things apart: what the code does (L); what a
requirement asks — §4 (A)'s, that outcomes stay unknown to `j`'s reviewers until `Φ_j`, with P4
and §9.2's evidence — stated conditionally on that requirement; a candidate policy chosen to meet
it, an option with its reason and consequence. No policy follows from D1.

| Object | Produced: when, by which role | Read to run; to verify | What the requirement asks | Candidate policies: reason; consequence |
|---|---|---|---|---|
| (a) answers and fit data: sheets with anchors, admissions, frame, seeds, model version, thresholds | answers by respondents during administration: under T1 after `Φ_G`, under T2 from the group record's position; admissions by the node applying `AdmitRespondent`; frame, seeds, version and thresholds in the group record, before any assignment (§11.1, §7.3) | the runner, to fit; a verifier, to recompute every status (§9.2, item 3); a respondent, its own leaf; no reviewer, to report | bearing on every member's outcome (one fit, §11.1), they reach no member's reviewer before that member's `Φ`; P4 needs a verifier to read them at some point; a respondent's check needs its own leaf only | *verifier after the group's condition*: no separation of persons is shown, so a verifier may be a member's reviewer; errors of a run surface only after `Φ_G`. *Verifier at production*: P5 then rests on verifiers' behaviour. Under either, the respondent's leaf and proof, within `08` PRIV-004.1's limits |
| (b) attempt statuses and reasons: a converged verdict or a named non-conclusion; exhaustion; a resource shortfall | by the runner, at each attempt's end | the runner, to continue or stop; a verifier, since exhaustion is shown by every required attempt (§9.2) | as (a): a status bears on the outcome (an indeterminate attempt on `I`; a screen failure is `R` and ends at stage 1, §8.3); a shortfall needs a positive descriptive record (§9.2) | as (a); under T2, logging with content and position hidden is one option (§12.3). The shortfall record stays apart from the pilot's transition and is by itself no `I` (§9.2) |
| (c) intermediate results: stage-1 fit and readings, stage-2 composition, latent fits | by the runner, inside an attempt | the runner; a verifier can recompute them instead (invariant 7) | as (a); P4 needs none received, since recomputation gives it | *none before the terminal record*: they bear on the outcome and are recomputable; no third party monitors a run in progress. *To a verifier during the run*: as the verifier at production |
| (d) terminal records: `A`, `R`, or `I` with §9.2's content | by the runner at conclusion or exhaustion: under T1 after `Φ_G`, under T2 possibly before a member's `Φ_j` | a verifier, with (a); the scorer, with `p_uj` and `b_uj`; the pool-entry rule under A | it reaches no member's reviewer before that member's `Φ`; the scorer needs it after `Φ_j`; consolidation needs it available (§8.3) | an opening at one of §12.4's conditions, with the predicates of §12.4 recorded; before it, held by its holders, on whose behaviour P5 rests |
| (e) gate and pool decisions | `Score`, `Resolve` and the appeal from the reports, before `Φ_j`; pool entry after the verdict | the author, for the appeal; the pool's administration | gate decisions are part of `Φ_j`; an entry that implies `Y = A` discloses (d) and falls under its requirement | *entry after the record's condition*: it discloses (d); a member that passed the gate waits (§7.3, *Time*, *Load*) |

| Object | Code today: on the log, kept off, capabilities | What a record attests | Missing for the requirement | T1 against T2 |
|---|---|---|---|---|
| (a) | `AdmitRespondent` (batch, epoch, the `Respond` proof and so the respondent's id); an epoch results event's root over answers and ratings; the answers kept off (D17, PRIV-004); `pilot::submit_response`, `results::{answer_leaf, inputs_root, inclusion_proof}`, the anchor and template gates | an admission: a proven id admitted once to a batch and epoch, no answer and no run (§11.4); the root: the set bound to a reference whose provenance is the results event's; none of P2–P6 | a holder and an access rule (`08` Q-1); a reference binding the answers at collection, logged (for §12.4's (iv)); a record of the frame used; under T2, a way to keep them unread before the condition (no design here). Known, not reopened: one respondent id across batches (`08` PRIV-006, T69) | T1: collected after `Φ_G`, so a member's reviewer answers only after reporting; P5 concerns other groups only (§12.4). T2: collected before the reports; runner, holders and respondents hold data bearing on outcomes before the freezes, and a reviewer can answer before reporting (§11.3); P3 and P5 rest on the holders |
| (b) | a `Pilot1Batch` or `Pilot2Batch` step carries one item's reading in the clear; an indeterminate reading leaves the state but its step is logged; the `PilotError` met is not carried; a step with too few respondents or a batch below `K_MIN` is refused, and `Node::submit` writes no refused event: a shortfall leaves no record. `Screening`, `Recheck`, `PilotError` | that a writer logged a reading for the item at a position; not that a fit ran, that the reading is the procedure's (the caller supplies it, §11.4), or exhaustion | an attempt record with reason and evidence; a positive descriptive shortfall record, apart from the pilot's transition — the refused step stays refused, and no rule classifies the record as `I` (§9.2); an attempt budget and term (the lifecycle accepts indeterminate readings without bound) | T1: produced after `Φ_G`; logged then, a status can inform other groups only. T2: produced before the freezes; logged at production it is an early disclosure to every peer today, so it stays with its holder, or enters the log with its content hidden at a position independent of the outcome (§12.3) |
| (c) | none; indirectly a stage-2 batch: if named by `pilot::batch_id` of its items (the only convention in code, called by tests only), the admissions to it show which members passed stage 1. `stage1_fit`, `stage1_verdicts`, `stage2_dif`, `revalidation::latent_batch` | nothing on the log; a recomputation from (a) attests P4 to whoever holds (a) | nothing for P4, which recomputation gives; a batch name hiding its members, if survival must stay hidden | as (b): under T2 its side effects can disclose before any record |
| (d) | the pilot steps and the states they set (`Pilot2`, `ActivePool`, `Contested`, `Rejected`, `Explored`, `Measured`) carry the outcome in the clear; no state for `I` (§7.4); the evidence kept off (PRIV-004); `exploration::outcome_of` | a record: what a writer logged and where; a commitment: content fixed by its position, hidden only under a secret nonce; neither P4 (it needs (a) and a recomputation), P5, the opening's availability (an unopened commitment is a pending outcome, §9.1) nor P6 | a terminal record with `I` and §9.2's content; an opening rule; a path piloting a gate-rejected member under A without `Explore` (§4, D's effects); under T2, a record fixed before its opening and the opening itself (no design here), and a path recording a pilot before `Score` (pilot steps are refused outside `Pilot1`, `Pilot2` and `Explored`) | T1: the record itself, logged after its condition; nothing to hide from the group's own reviewers. T2: the record kept unread until the condition, at a position independent of the outcome, and the pre-`Score` path |
| (e) | `Score`, `Resolve`, `Appeal`, `AppealExpires`; pool entry through the state (`Pilot2Batch` with `NoDif` sets `ActivePool` today); `gate::{bridging_gate, supplementary_review}` | that a writer logged the decision; not that its inputs — robust score, gap, coverage — were computed correctly (the caller supplies them, §10.1) | under A, pool entry apart from the pilot's verdict step and placed after the record's condition | T1: a member that passed the gate waits for its group (§7.3, *Time*, *Load*). T2: the verdict can exist before `Φ_j`, and an entry before its condition discloses it (§12.3) |

### 12.3 What can be learned without a record

A possibility of inference is no violation by itself; each row names what a signal can bear on
and the hypothesis it may involve, under which condition.

| Signal | Bears on | Visible today | May involve, and when |
|---|---|---|---|
| an attempt's step; the number of attempts | the chance of `I`; the stage reached | per item, in the clear | §4 (A)'s requirement that outcomes stay unknown to `j`'s reviewers until `Φ_j`, if it reaches them before; H-c and C5 if `u`'s deviation decides whether it reaches the reports forming `b_uj` (§11.3) |
| a non-conclusion's reason | intrinsic against resource; sample sizes | not carried; a refused step leaves no record | nothing is checked by an absence: silence and a shortfall stay indistinguishable without §9.2's positive descriptive record, itself no `I` |
| a record's or a commitment's position | the category, when conclusion times depend on it (`R` at the screen ends before `A`, §8.3) | under T2, a commitment logged at production would show it | as the first row; a position fixed for every member whatever its path hides it only if the commitment exists by then, and a missing one there signals a pending procedure |
| resource events: admissions per batch, a stage-2 batch | stage-1 survival (object (c)); a shortfall | admissions, in the clear | as the first row; under T1 they follow `Φ_G` |
| pool entry under A | an entry implying `Y = A` (from the group's verdict, §8.5) discloses `A`; no entry identifies `R` or `I` by itself: the gate may have refused the item, or its entry may be waiting | the state, in the clear | the entry is a disclosure of the terminal record, under its condition; an absence bears on `Y` only jointly with the gate's decision and the time elapsed |
| gate decisions on other items | under α, an aggregate forecast of their outcomes, so possibly of `Y_k` in a correlated group | `Score`, `Resolve`, in the clear | information fixed across `u`'s strategies belongs to `F` (§11.3); H-c and C5 only if `u`'s deviation decides whether it reaches the reports forming `b_uk` before they are committed |

### 12.4 Opening conditions and boundaries between groups

An object opens once available and its condition holds (§11.1). For objects (a)–(d), with `Φ_j`
unchanged:

| Condition | Complete before it | Object not yet available | Freeze not reached | Informs other groups' reports | Between groups |
|---|---|---|---|---|---|
| `Φ_j`, per member | `j`'s reports, decisions and appeal (`16` §5) | unopened, its unit pending | unopened; under T2 held | yes; and `G`'s other members before their `Φ_k`, since one fit makes `j`'s objects bear on theirs (§11.1): insufficient for a group's objects under A | `Φ_G` first, then as `Φ_G` |
| `Φ_G` | every member's `Φ_j` | opens once available, without waiting for the others; pending until then, never `I` (§9.1) | every object of `G` unopened; under T2 held for a time no count of rounds bounds (§9.4), P3 and P5 resting on the holders throughout | yes (below) | the constructions below, each for its own channel, with the channels left declared |
| collective: `G`'s objects together | `Φ_G` and every record available | every object waits for the last; one that never arrives keeps all unopened (§8.3) | as `Φ_G` | yes; the opening now also follows the slowest record, whose time can depend on outcomes (§8.3) | as `Φ_G` |

**Across groups** (D, C). Respecting `Φ_G` does not close every dependence between groups, under
T1 or T2. `G`'s objects can bear on `Y_k` for `k` in another group `G'` — shared respondents,
anchors, period, topic — and their opening follows `G`'s reports through bands and appeals. If
`u` reviews `j ∈ G` and `k ∈ G'`, a deviation on `j` can decide whether `G`'s record reaches the
reports forming `b_uk` before they are committed. A partly informative signal suffices (C;
abstract, not a model of the protocol): with `Y_k` equiprobable, a signal setting `P(Y_k = A)` to
`½ ± δ`, the reports forming `b_uk` equal to that posterior and `p_uk = ½`, `u`'s expected
contribution is 0 without the signal and `−δ²` with it: −1/16 at `δ = ¼`, −¼ at `δ = ½`
(§11.3's construction). That the reports equal the posterior is a hypothesis of the
construction, not of the analysis of incentives.

**Four bounded constructions.** The general sufficiency stated in `8af257d` — "(i) or (iii); or
(ii) with (iv)" — is withdrawn. Each construction below excludes one channel under its own
hypotheses and leaves others; a combination excludes no more than the channels its members
exclude, each under all its hypotheses. They are sufficient conditions of limited reach, not
necessities, and no construction need fix everything the same way.

| Construction | Channel it excludes | Further hypotheses | Channels left | Kind |
|---|---|---|---|---|
| (i) the opened object `O` adds no information on `Y_k` | a change of the conditional law of the whole outcome for each reviewer `v` whose report forms `b_uk`: `P(Y_k = y \| F_v, O) = P(Y_k = y \| F_v)` for `y ∈ {A, R, I}`, so `c` and `q_c` unchanged; the equality on `A` alone does not suffice (below) | that `v`'s actual report depends on `O` only through that forecast — not implied: a message uninformative on `Y_k` can still prompt strategic reactions, and an analysis of incentives cannot presume truthful reports; toward outcomes, that `O` changes no answer, selection or execution of `G'`'s procedure | the actual reports whenever the behavioural hypothesis fails; every channel to outcomes outside its second part; availability and consolidation | a strong statistical hypothesis on laws, sufficient and not shown necessary, and a behavioural one; declared, not verifiable from the order of events, no demonstration of H-c |
| (ii) the reports fixed before the opening | the opening's information reaching the values of the reports that form `b_uk` | the first panel's commitments on `k` closed (`CloseCommits`) before the opening; composition, weights and aggregation rule fixed — `16` §4.6's approved composition, the epoch's frozen weights, the weighted mean; no report added after (T58 not introduced) | a reveal withheld after the opening: the value stays fixed, the report unrevealed, case 6 and case 7 for the panel (§9.3), so availability and consolidation (§8.3, (ii)–(iv)); items whose commits close after the opening; a disclosure off the log; every channel to outcomes | a verifiable property of the order for logged openings (two log positions per pair); a disclosure off the log stays P5's behavioural hypothesis |
| (iii) an opening position fixed in advance | the opening's timing as a function of the reports | `Φ_G` before that position on every path, a bound no count of rounds gives (§9.4); the content, presence, recipients and metadata (size, attempts, production positions) of what is made available invariant under the deviation | what the position does not fix: a non-opening there (a freeze unreached, a record missing), the content through `G`'s own outcome law (under T1, §11.3's timing channel), recipients, metadata | the position verifiable on the log; the invariance of the information made available a statistical hypothesis; no mechanism proposed |
| (iv) `G'`'s answers collected before the opening | the opening changing those answers | the answers fixed at collection by a reference logged before the opening; none exists today: the root reaches the log only with an epoch's results (`04` §Events and replay), and admissions record no answer (§11.4) | the selection and inclusion of data (which sheets and rows enter, floors, gates); the procedure and its parameters unless the group record pins them (§7.3); the source check (`source_verified`, T68); the attempts and their timing; computational resources; every channel to baselines | a verifiable property of the order once such a reference exists; the rest hypotheses, or their own pinning |

**On (i)** (C; Astra's example, checked with fractions). Two laws with `P(A) = 1/4`: `P(R) = 3/4`
and `P(I) = 0` give `c = 1` and `q_c = 1/4`; `P(R) = 1/4` and `P(I) = 1/2` give `c = ½` and
`q_c = ½`. An object leaving `P(A)` unchanged can still move B-b's target, so (i) states the
invariance of the law on the three categories. The invariance of a forecast shows nothing of
the reports actually made: the behavioural hypothesis stays separate.

**(ii) and (iv) for a study**, bounded. Each is a predicate on a pair, checked on recorded
positions — not a schedule imposed on every pair:

- (ii): for each logged opening of `G`, the items `k` whose first-panel `CloseCommits` precedes
  it; for them the channel to the values forming `b_uk` is excluded under the row's hypotheses,
  for the others it stays and is reported. Which items an object bears on needs (i)'s relation,
  declared; without it the conservative reading takes every item whose commits are still open.
  Overlaps of reviewers are countable by nym but bound no exposure: a participant of the
  collective fit can move the freeze of a group it does not review, through shared parameters
  (§10.4).
- (iv): for each logged opening of `G`, the groups `G'` whose answers a logged reference fixed
  before it; checkable only once such a reference exists. It concerns those answers alone; the
  rest of `G'`'s verdict stays under the row's residual channels.

Imposing (ii) or (iv) on every pair would be a scheduling rule; a freeze common to all groups is
one way to impose (ii). Neither is shown necessary, and neither is adopted. §9 stands under every
condition: a missing report gets no 0; silence and `Ω` produce no `I`; an item's outcome does
not complete every assignment; the consolidated cohorts alone are not representative. `Ω` is no
opening condition: an object held at `Ω` is reported pending (§9.5), and opening it then would
inform the reports still open.

### 12.5 For Astra

| | Both | T1 only | T2 only |
|---|---|---|---|
| requirements | the group record; `Φ_j` and `Φ_G` as log rules; an opening rule per object, with §12.4's predicates recorded per pair and the channels left declared; a holder and an access rule for answers and evidence (`08` Q-1); terminal records with `I` and §9.2's content; positive descriptive shortfall records, apart from the pilot's transition; A's measurement-only pilot path; pool entry implying `Y = A` after the record's condition; PRIV-004 not extended (T77); no check linking roles | a start rule after `Φ_G`; resources and samples invariant in time, or declared (§11.3's timing construction: a channel through time and samples, not through disclosure) | an information control from production to opening: holders' confidentiality for an unbounded time (P3, P5), records fixed before their opening at positions independent of outcomes, side effects (admissions, stage-2 batches) kept from disclosing; a path recording a pilot before `Score`; the respondent channel before the reports |
| missing in code | the group record; freeze records; attempt records with reasons; a descriptive shortfall record (the pilot step is refused and nothing is written); `I` as a terminal state; pool entry apart from the verdict step; A's pilot without `Explore`; a holder and an access rule for answers; a logged reference binding answers at collection | a start rule | a way to keep records unread before their condition and to open them (no design here); a pre-`Score` pilot path |

**Declared, or checked only in part, by a study.** P5 for holders: declared; a record on the log
before its condition is a positive finding, its absence proves nothing. §12.4's (i): declared,
the whole outcome's law and a behavioural hypothesis, no demonstration of H-c. (ii): checkable
per pair for
logged openings; (iv): checkable per pair once a logged reference of the answers exists; neither
covers the channels left in its row. Overlaps among groups' reviewers are countable on the log
by nym; overlaps between respondents and reviewers are not, by design (invariant 5). H-c and the
invariance of resources: declared; delays, sample sizes and outcome frequencies by delay are
describable, but deviations are not observed, so no invariance is shown.

**Observable by a study**, without claiming properness or certified neutrality: per object, the
positions of production where recorded, of logging, conditions and openings; delays from named
origins (§9.1); how long objects stay held; pending cases by kind (§9.5); shortfalls with
positive descriptive records; any early record; the pairs meeting (ii) or (iv) and those that do
not; outcome frequencies by group and delay, as descriptions. Not: properness, P5 in general,
the absence of channels between groups, a neutral test.

**Direction and recommendation.** By Astra's direction T1 is the reference for the next
comparison of informational requirements and T2 the comparison; T1 is not adopted as a
protocol, and its timing channel (§11.3) stays open. My recommendation, not a decision: specify
first the common requirements, since both constructions need them and most missing capabilities
are among them. No harness's availability would show either construction's conditions.

**Still to review**: these rectifications, and the record of `08` PRIV-004.1. **Minimal next
step** after that review: for T1, the log positions a study would record to evaluate (ii) and
(iv) per pair, and the channels it would declare instead. Nothing here is adopted; no protocol
is ready and no neutrality is certified.

## 13. Observable evidence for (ii) and (iv) under T1 (conditional analysis, approved on `176dd8c`)

**Status.** A proposal by Claude Code on the direction of Astra's review of `7d8db08`; §12's
approval does not cover it. T1 is the analytic reference and T2 the comparison; T1 is not
adopted as a protocol, and its timing channel (§11.3) stays open. It states what a study would
record to evaluate §12.4's (ii) and (iv) pair by pair, and what an order shown on that evidence
concludes. No clock, global consensus, ordering protocol, API schema or cryptography is
proposed, and no code changes. **L** read at `7d8db08`; **D** derived here; no new calculation.

Astra's review of `f38f8a4` (the commit, its parent and the published HEAD checked; the diff, the
documentary links and the relevant code of `network::{cut, log, replica}`, `protocol::{ledger,
lifecycle}` and `p2p::member` read; no Rust, fit, campaign, new Merkle reproduction or local
documentation check; the working tree and the local checks stay Claude Code's evidence):
**partial**. Approved: the record of `7d8db08`'s review; §12.4's (i), the whole outcome's law
apart from the behavioural hypothesis; `08` PRIV-004.1's state and its reference among the
residual risks; in §13, the local order, the application order and disclosure apart, the
cryptographic references showing some precedences across feeds, and pairs not limited to
reviewer overlaps. Not approved: §13 as a whole. T1 stays the analytic reference and T2 the
alternative, neither adopted; no protocol, implementation or start of S2 is approved. Rectified
after that review in `176dd8c`: replay apart from information (§13.1, §§13.4–13.5); accepted,
refused and missing events, and the outcomes (§§13.2–13.3); the reference to answers for (iv)
(§§13.4–13.5). **L** read again at `f38f8a4`: `Replica::insert`, `Cut::next`, `added`,
`should_sign`, `Ledger::apply`, the lifecycle's `Commit` arm, `p2p::member`'s duties.

Astra's review of `176dd8c` (the commit, its parent and the published HEAD checked; the diff of
the four documents read; the relevant passages of replication, cuts and the ledger checked again;
no Rust, fit, campaign or Merkle reproduction, and no local check of the working tree or of the
documentation checks): the rectifications and §13 **approved as a conditional analysis of the
observable evidence**, within its stated limits and with three precisions. Resolved, the
substantive findings of the review of `f38f8a4`: a construction keeping replay apart from the
availability of information, compatible with the rules read of `Cut::next`, `added` and
`should_sign`; relevant commitments apart from mere entries named `Commit`; disclosures that can
include refused or uncounted entries; an absence in the prefix apart from a contrary order and
from incomplete evidence; a results root able to bind answers before a later disclosure without
dating their collection; the minimal result limited to the replay check. Not approved: the
adoption of T1 or T2, a protocol, an implementation or S2's start. The precisions, made in place:
(R) and (E) are distinct checks, (E) not needing every disclosure entry examined by the replay
(§13.2); (E)'s conclusions concern the entries of `x_O` its evidence covers, every disclosure
only under (I)'s coverage (§13.4); `Replica::check`'s authenticity and integrity checks (§13.1,
§13.4). **L** read at `176dd8c`: `Replica::check`, `WriterSet::check`.

Astra's review of `cbcbc67` (the commit, its parent and the published HEAD checked; the diff and
the relevant passages read; no Rust, fit, benchmark or campaign run; the local working tree not
checked): the record of the review of `176dd8c` and the three precisions **approved**; so are the
further changes to §13.2's classification at (R) and (E): an absence concerns acceptance in the
declared prefix, and does not show that the commitment or reference did not exist elsewhere.
S2's minimal perimeter (`19` §10.6) is partly approved there and rectified, awaiting review.

Astra's review of `5af22b9` (recorded in `19`'s design review) approved `19` §10.6 as a
preparatory synthesis and asked one editorial join here: §13.2's absent-precondition row now
states on its own that it concerns acceptance in the declared prefix and does not show that the
commitment or reference does not exist elsewhere; the classification is unchanged.

### 13.1 Three orders

- **Local order.** A node's own log (`Node::submit`, `04` §A node's own disk) and a writer's feed
  are hash chains: an entry commits through `prev` to every earlier entry of the same log, so
  within one log a lower position existed before a higher one — relative to a head the verifier
  trusts, since a consistent rewrite by the log's holder is caught only against a signed
  checkpoint (`08` NET-004). Positions in two different logs are not comparable by number.
- **Application order.** A `Replica` accepts an entry subject to the authenticity and integrity
  checks of `Replica::check`, which `insert` calls: the writer in the writer set and its
  signature over the entry's hash; the object within `MAX_OBJECT` and its CID equal to the
  entry's payload. None of them checks the entry's validity as a protocol transition.
  `Ledger::apply` then examines the entries a signed cut adds, in the cut's order — cut number,
  then the writers' new entries interleaved by rank (`04` §Cuts) — and applies or refuses each
  (`CutReport::{applied, refused}`); a refused entry leaves the state as it was. The order in
  which an entry is examined is not its accepted effect. Nodes holding the same cuts examine and
  apply the same order. It orders state transitions across writers, not when an entry first
  existed or became readable.
- **Effective disclosure.** An entry is readable by any peer once replicated (`04` §Who reads):
  before any cut counts it, whether the ledger later applies or refuses it, and even if no cut
  ever counts it (a feed stopped at a fork). A replica is a set and keeps no arrival time; a
  disclosure off the log leaves no record (§12.1, P5).

**Existence across feeds** (L, D). A cut's marks carry each writer's length and head, so a cut
commits to every entry it counts, and a member's signature of a cut travels on the member's own
feed with the cut's encoding (`MemberObject::CutSignature`, `04` §Members' objects). So `e₁`
existed before `e₂` when `e₂` follows, on one feed, either `e₁` itself or a cut signature whose
cut counts `e₁`. No other format carries such a reference (`NodeEvent`, `EpochResults`, the
beacon objects): otherwise the existence order of entries on different feeds is indeterminate.
Feeds, cuts and their signatures exist in code; nothing runs an epoch between nodes yet (`10`
T79), and only tests start a node (`10` T78).

**Replay is not information** (D; an abstract construction checked against the cut rules, not
an execution nor a demonstrated attack). Writer `W` logs an entry carrying `O`; it is replicated
and readable. Later, `k`'s first-panel commitments are logged on another writer's feed. The
proposer of cut `c`, whose replica does not yet hold `O`'s entry, marks that feed past the
commitments and `W` short of `O`: `Cut::next` marks each feed as the proposer's replica holds
it, and a member co-signs a cut that extends the last, names only entries it holds and counts
its own beacon messages (`should_sign`) — no rule requires a cut to count every replicated
entry. Cut `c + 1` extends `c` and counts `O`'s entry. The replay examines the commitments before
`O`, yet `O` was readable before they existed. On one feed the construction fails, since marks
are prefixes: a cut counting the commitments would count `O` too.

### 13.2 Outcomes of a pair

Each pair is classified at a stated level — the replay check (R) or documented existence
precedence (E), §13.4 — on a stated prefix: the cuts up to a named one, with the entries they
count. A prefix is *complete* when every cut up to the named one is collected and the verifier
holds every entry those cuts count. The two levels are distinct checks: (R) reads the order in
which the replay examines entries; (E) uses the prefix to identify the relevant commitments, or
the reference, and reads each entry of `x_O` on its own feed, counted by a cut or not (§13.3).

| Outcome | Evidence it needs | What it shows |
|---|---|---|
| **verified** | (R): a complete prefix counting every entry of `x_O`, in which every relevant commitment, or the reference, is accepted before the first examination of any of them; (E): the relevant commitments, or the reference, shown to exist and identified as such — accepted in a complete prefix (§13.3) — and §13.1's chains and references from each of them to each entry of `x_O`, which the replay need not have examined | the order at that level for the entries of `x_O` its evidence covers; nothing of other entries, of disclosures off the log or of the channels left in the construction's row (§12.4); no proof of H-c |
| **contrary order documented** | both events present, in the reverse order at that level: an entry of `x_O` examined (R), or shown to exist (E), before an accepted relevant commitment, or the reference | the pair's channel is not excluded by (ii) or (iv); not that it was used, nor a violation of H-c |
| **precondition absent in the prefix** | a complete prefix in which a relevant commitment, or the reference, is not yet accepted while an entry of `x_O` is examined in it (R), or held by the verifier (E) | an absence of acceptance in that declared prefix — not a later event observed, a later cut bringing it or none — and no proof that the commitment or reference does not exist elsewhere |
| **indeterminate** | anything less: at (R), an incomplete prefix or an entry of `x_O` no cut counts; at (E), a prefix too incomplete to identify the relevant commitments or the reference, or an entry of `x_O` on another feed with no reference of §13.1 to them; at either, an entry missing from the verifier's replica, no trusted head | nothing |

With several entries in `x_O`, a pair is verified only if the order holds for every entry; a
contrary order or an absent precondition for one entry is reported as such, whatever the others.
No outcome shows by itself a violation of H-c, and none turns into a value.

### 13.3 The disclosure and the pairs

Under T1 a group's procedure starts after `Φ_G`, so its objects are produced after its freezes
and opened by being logged (§11.2). For an object `O` of `G`, its *disclosure entries* `x_O` are,
conservatively, every replicated entry the verifier holds from which part of `O` can be inferred
— `O`'s own entry, a pilot step, an admission to a stage-2 batch, a pool entry (§12.3) — whether
the ledger applies it, refuses it (an orchestration event from outside the consortium, an
object that is not an event, a step the lifecycle refuses) or no cut counts it. The replay check
concerns the entries the replay examines; an entry of `x_O` that no cut counts leaves (R)
indeterminate for the pair, and (E) is judged on its feed.

**The relevant commitments** for an item `k`: its panel `P_k` fixed by its accepted
`AssignReviewers`; each accepted `Commit { nym: v, commitment }` with `v ∈ P_k`, accepted in
`InReview`; `k`'s accepted `CloseCommits`. The lifecycle refuses a commit by a non-panelist, a
second commit by one panelist and any commit after `CloseCommits`, and the ledger refuses
orchestration from outside the consortium: an entry merely named `Commit` that the ledger refused
fixes no report of the protocol. The approved composition of the baselines, the frozen weights
and the weighted mean (`16` §4.6) stay as they are; T58 is not introduced.

The pairs considered: for (ii), `O` with every item `k` reviewed up to `Ω`; for (iv), `O` with
every group `G'` administered up to `Ω`. The relation is not limited to reviewer overlaps: a
participant of the collective fit can move the freeze of a group it does not review, through
shared parameters (§10.4), so an overlap count bounds no exposure. The relation identifies no
dependence: which pairs carry information (§12.4's (i)) and through which channel stays
declared, and no algorithm finding every dependence is proposed.

### 13.4 The two predicates

Three levels, apart: **(R) the replay check** — the relevant accepted commitments, or the
reference, precede the examination of every entry of `x_O` the replay examines; **(E) documented
existence precedence** — they existed before every entry of `x_O` the evidence covers, shown by
§13.1's chains and references, whether or not the replay examined that entry; **(I) exclusion of
early information** — besides (E), the coverage of disclosures (every disclosure of `O` is an
entry of `x_O` the verifier holds) and the other declared hypotheses. (R) says nothing of
information (§13.1's construction); (E) bounds only the entries of `x_O` its evidence covers,
not every logged disclosure; (I) is reached under declared hypotheses, never verified.

| | (ii) commitments before the disclosure | (iv) answers bound before the disclosure |
|---|---|---|
| units | `O` of `G` and its entries `x_O`; an item `k`; the first-panel assignments `(v, k)`, whose reports form every baseline of `k` (`16` §4.6) | `O` and `x_O` as in (ii); a group `G'` and its answers (its batches' sheets, anchors included) |
| events compared | the relevant accepted commitments of `k` (§13.3), against each entry of `x_O` | a logged reference binding `G'`'s answers, against each entry of `x_O` |
| bound values or data | the commitment `H(prob, nonce, committer, item)` carried by an accepted `Commit` and checked at `Reveal` (`review::commit`, INV-12) | the set of answers a reference of the kind `results::inputs_root` binds (binding only, §12.1); its content and its coverage of the answers used are to be checked |
| evidence in code | `Replica::check` (writer, signature, object size and CID; no check of protocol validity) apart from `Ledger::apply` (examines, applies or refuses); the `AssignReviewers`, `Commit`, `CloseCommits` and `Reveal` steps, orchestration accepted only from consortium writers; feeds, cuts and cut signatures; the approved composition as an API (`panel_scores`: a first panelist's leave-one-out mean, an extra reviewer's first-panel mean) | `AdmitRespondent`, which records an admission and no answer (§11.4); `inputs_root` over an epoch's ratings and answers, logged only in the epoch's results event (`04` §Events and replay) |
| missing | under A, `O`'s entries for a gate-rejected member and terminal records with `I` (§12.2); a record of the weights used, showing them the epoch's frozen ones (`panel_scores` takes them from its caller, which no production code is); a runtime logging an epoch (T79) | a reference logged at collection. Without one, a results root logged before `x_O` can still bind the answers, from its own existence on: it does not show when they were collected, nor that they were bound then |
| (R) shows | the relevant commitments were accepted before every entry of `x_O` the replay examines: the state's order, not information | the reference was examined before those entries: likewise |
| (E) shows | each relevant commitment existed before every entry of `x_O` the evidence covers, so none of those entries was readable before the commitments existed; every logged disclosure only under (I)'s coverage | the reference existed before every entry of `x_O` the evidence covers: the answers it binds were fixed before those entries; before every disclosure of `O` only under (I)'s coverage |
| (I) needs, besides (E) | the coverage of disclosures (P5, declared); the approved composition, the frozen weights and the weighted mean fixed; no report added after (T58 not introduced). Then the values forming `b_uk` were fixed before `O` could inform them | the same coverage, toward `G'`'s respondents; the reference's content and coverage checked. Then `O` did not change the answers the reference binds |
| further hypotheses | the recorded weights are the frozen ones; `x_O` covers every entry bearing on `O`; the trusted head | `x_O` and the head as in (ii); any inclusion proof issued from an early reference falls under `08` PRIV-004.1 |
| channels left | values committed but not revealed: a reveal withheld after `x_O` leaves case 6, and case 7 for the panel (§9.3), so availability and consolidation, not the value; disclosures off the log; pairs not verified; every channel to outcomes; T1's timing channel | answers outside the reference; their selection and inclusion in the fit; the procedure and its parameters; the source check; the attempts; computational resources; the verdict's law; disclosures off the log to respondents; every channel to baselines |

For (ii), values committed, reveals available and consolidation stay apart: a verified pair
concerns the first, and says nothing of the other two.

### 13.5 Reporting, and the minimal result

Per pair: its outcome (§13.2), level, prefix and the evidence used; counts by outcome. No pair,
outcome or missing event becomes a value. §9 stands: a missing report gets no 0; silence and
`Ω` produce no `I`; an item's outcome does not complete every assignment; the consolidated
cohorts alone are not representative. At `Ω` a pair is classified on the prefix up to `Ω`'s last
collected cut: if that prefix is complete and an entry of `x_O` was examined while a relevant
commitment, or the reference, is not yet accepted, the outcome is *precondition absent in the
prefix*, not a contrary order; if the prefix is incomplete, *indeterminate*.

**The minimal verifiable result.** For each logged opening under T1, the replay check (R) of its
(ii) pairs on a complete prefix, from the cuts, the entries they count and the ledger's accepted
and refused entries — not a verification of (ii)'s informational predicate; documented existence
precedence (E) where §13.1's chains and references link the entries, for the entries of `x_O`
they cover, every disclosure only under (I)'s coverage. For (iv), the same two
levels for a reference logged before `x_O`, a results root included, within its checked content
and coverage; neither level says when the answers were collected. **Declared, not shown**: the
coverage of disclosures (no disclosure off the log, P5, and every replicated entry bearing on
`O` held by the verifier); which pairs carry information (§12.4's (i): the whole outcome's law,
with its behavioural hypothesis); the frozen weights, where no record shows them; the
reference's coverage of the answers used; the invariance in time of T1's resources and samples
(§11.3); H-c. **T2, the comparison**: the same predicates, with `x_O` the opening of a record
produced earlier; production before `x_O` adds the holders' P5, which no order evidence covers.

**Next.** §13 is approved on `176dd8c` as a conditional analysis, its precisions on `cbcbc67`;
S2's minimal perimeter built on §§9–13, `19` §10.6, was approved on `5af22b9` as a preparatory
synthesis, and the synthetic-event check is specified in `20`, approved on `c76c035`, its
implementation approved on `3559396` within the check's perimeter. Nothing here is
adopted; no protocol is ready and no neutrality is certified.

## 14. A1's candidate guarantee (approved on `d2be7da` as a conditional candidate contract)

**Status.** Claude Code's proposal on Astra's direction after the review of `bef891b`: B-b the
reference score, A the analytic reference, C in the comparison; no arm adopted for production, no
expenditure or sample size authorized. It assembles approved results (`16` §4; §§7–13), proves
nothing new and changes no code.

Astra's review of `11deec2` (the commit, its parent and the published branch checked; the diff,
the contracts and the relevant code read; small examples checked with rational arithmetic; no
Rust, fit or campaign run; the local checks and the working tree stay Claude Code's evidence):
**partial**. Approved **as a conditional result**: the mathematical core of §14.1 — under the
declared hypotheses B-b maximizes the expected contribution at `q_c`, a report `p` losing
`c(p − q_c)²`, and at `c = 0` the score is flat. Not approved as a whole: §14, pending the
rectification of the adaptive verification's criterion; the absent baseline needs a precision,
not blocking. On the IPW criterion, Astra's judgment is recorded in §14.7. No adoption of A or C,
no substantive narrowing of the promises and no closure of A1 approved. Rectified after that
review, in place, in `d2be7da`: §14.6's two aims (§14.3 aligned on report-driven effects); the
absent baseline (§§14.2, 14.4).

Astra's review of `d2be7da` (the commit, its parent and the published branch checked; the diffs
of the three documents, the constructions cited and the relevant code read; no Rust, fit or
campaign run; the local checks and the working tree stay Claude Code's evidence): **approved** —
the documentary rectifications; §14 **as a conditional candidate contract**; the separation of
a verification within the hypotheses from the reproduction of the contrary channels (§14.6); the
precision on the absent baseline. The finding on the adaptive criterion is resolved. Confirmed by
reading the code: the aggregation defect §14.4 records (below). Requiring a defined baseline makes
a precondition of D1's formula explicit, not a new design of the score; it authorizes no
exclusion, fallback or change of the denominator. Not approved: the protocol, an adoption of A or
C, a substantive narrowing of the promises, A1's closure. The correction of that defect and
§14.6's verification are in test code (`20` §§10–11), on the owner's assignment, approved on
`d7a4449` (below).

Astra's review of `d7a4449` (the commit, its parent and the published branch checked; the diff,
the regressions, the code and the documentary joins read; independent enumerations in Python
summing the contributions directly with exact fractions; no Rust run; the 39 passing tests,
Clippy, the regressions failing first, the probes and the local checks stay Claude Code's
evidence): **approved**, within `20` §§10–11's perimeter — the absent baseline's correction, with
assignments, denominator, counts and defined contributions kept, no value or interval where a
needed contribution is undefined and `g(I) = 0` kept; §14.6's verification on the two finite
constructions; the five contrary channels reproduced within their limits; the representation
(`Unresolved` where cases 6–7 and undefined terms coexist, `Cohort::undefined` listing the latter,
a precedence that is no reputational policy). A check of the computation and of finite
constructions: no H-c in the protocol, availability, confidentiality, reputational incentives or
neutrality. §14.8, a later proposal, is not covered.

Astra's review of `23283f2` (the commit, its parent and the published branch checked; the diff,
the documents and the relevant code read; no Rust, fit or campaign run; the local checks stay
Claude Code's evidence): **partial**. Approved: the record of `d7a4449`'s approval; §14.6's
verification reclassified as acquired within its perimeter; the editorial corrections. Accepted
as the next intervention, independent of O1–O3: B-b's scorer with no production caller. Not
approved as a whole: §14.8, rectified in place on that review's points (A: the forecast's
target, the observation design and the incentives' scope kept apart, the choice between a
narrowed guarantee under A and an IPW under C withdrawn; B: T19's properties and H-e, H-b, H-c
kept apart; D: the runtime's boundary and the caller's responsibilities; precisions on the
absent baseline, `c > 0`, O1–O3 and adoption). The approval covers neither this rectification nor
the scorer (§14.9), both reviewed on `7d1b19e` (below).

Astra's review of `7d1b19e` (the commit, its parent and the published branch checked; the code,
the diff, the tests and the documents read; the numeric defects reproduced with Python
transcriptions of the floating-point operations and rational references; an independent exact
enumeration of the 72 states on the 27 report vectors, confirming −41/1728 and −91/5184; no Rust
run; the scorer's 10 tests, the check's 39, `panel_scores`' 7, Clippy and the local checks stay
Claude Code's evidence): **partial**. Approved: the record of the review of `23283f2`; §14.8's
rectifications. Accepted in the scorer's contract: `Undrawn` apart from a final non-selection; a
frozen item whose first report is missing refused, no composition over replacements introduced;
the interval with a pending member without a baseline, only as an interval on the final value
conditional on its coming to exist, no availability guaranteed. Not approved as a whole: the
scorer, for a numeric defect on its declared domain — the weighted mean's sums overflowing or
underflowing (a finite wrong baseline, a NaN, a positive weight lost), `g/π_j` and `1/π_j` beyond
`f64` at the smallest `π_j`, a sum overflowing before its representable mean. The correction
(§14.9, *Numerics*), on the owner's assignment, was partly reviewed on `0ca81e9` (below); the
approval covers neither it nor the scorer as a whole.

Astra's review of `0ca81e9` (the code, the diff, the tests and the documents read; the
counterexample reproduced with a Python transcription of the operations and a rational
reference; no Rust run; the regressions, the 19, 9 and 39 tests, the comparison of 988 364 calls,
Clippy and the local checks stay Claude Code's evidence): **partial**. Approved: the record of
the review of `7d1b19e`; the corrections of that review's cases A–E — the baselines stabilized,
`OutOfRange` kept apart from an absent baseline, a representable mean kept where its sum leaves
the range. Not approved as a whole: the scorer, because the scale common to the defined terms and
the pending members' radii can cancel a term from `known`: a term of −2^−100 beside a pending
member at `π_j = 2^−1022` was scaled by `2^−1022` to −2^−1122, which rounds to 0, and `known` was
`Ok(0)`. The approval covers the interventions examined on `0ca81e9`, not the correction that
followed (§14.9, *Numerics*, on the owner's assignment, approved on `6b9f42a`, below) nor every
numeric claim of §14.9.

Astra's review of `6b9f42a` (the diff, the code, the regression and the documents read; the
numeric cases transcribed in Python and 1 200 sums checked against exact rationals, 210 of them
where the narrowed identity applied; no Rust run; those checks do not replace the general
derivation; the scorer's 20 tests, `panel_scores`' 9, the check's 39, Clippy and the comparative
traces stay Claude Code's evidence): **approved, with no further blocking finding** — the
correction of `known`'s scale; the documentary rectifications and the record of the review of
`0ca81e9`; B-b's scorer as an isolated component, within §14.9's contract and numeric limits;
`20`'s check through the scorer. The scorer is acquired within that perimeter: no further
enumeration or comparison is needed to consolidate this approval, and a later change needs checks
pertinent to that change. The approval does not close A1, adopts no A, C or T1, and shows none of
the hypotheses in the protocol, availability, confidentiality, reputational incentives or
neutrality. §14.10, the contract of the scorer's future caller, came after it and is not covered.

Astra's review of `3e34f6f` (the contract read and compared with the cuts, the ledger,
`NodeState`, `NodeEvent`, the lifecycle and the freeze of `20`'s check; no Rust run): **partial**.
Approved: the record of the scorer's approval on `6b9f42a`; the scorer stays acquired within its
perimeter (§14.9). Accepted for the isolated component, as its direction: cohorts `K(u, e)`;
weights recorded before the assignments; the freeze derived from today's lifecycle. These choices
show no invariance of the assignments and adopt no protocol. Not approved as a whole: §14.10, for
incomplete inputs and inconsistent acceptance criteria — the cohorts' closure reads `Cut::closes`
and the epoch's advance, which applied events carrying only their cut's number and epoch do not
give; the ledger's validated entries, its refusals and the proposed records not kept apart; a
refused record's effect on the state, and progression checks a pure function cannot make; counts
over a filtered set of items. Rectified in place after that review (§14.10); the approval covers
none of the rectifications.

Astra's review of `8dc7087` (the diff and the specification read; the pertinent passages of the
cuts, the ledger and the scorer checked again; the criterion with `|R_u| = 4`, the valid subset's
`N = 3`, `O = 2`, `V = 1`, an invalid cohort of size 3 and a second cohort `Final(0)` verified; no
Rust run): **approved, with no further blocking finding** — the record of the review of
`3e34f6f`; the four rectifications; §14.10 as the specification of the isolated snapshot
component. Accepted within that perimeter: the design record before the first assignment under C
too, as an experimental precondition; a draw before `Φ_j` refused, the state kept, a later valid
record possible under the rules on duplicates and conflicts; the proposed records' placement
supplied by the input, with the checks declared delegated upstream. Refusing an early draw does
not show it harmless to H-e. One precision, made in §14.10 (*Input*, *Prefixes and updates*): the
cut's digest identifies the ledger's prefix, not the experimental records, their slots or other
added data; the properties between prefixes hold for truncations of one complete input. The
approval covers the specification, not its implementation (§14.11), which awaits review; the
scorer stays approved.

Astra's review of `9c7cb23` (the pertinent code, specification and tests read; the effect of
the cohort's order reproduced in Python; **no Rust run**): **partial**. Approved: the previous
review's record. Accepted within the experimental component's perimeter: the draw's round
recorded but unchecked; weights late after extra assignments too; the cohort record compared
only with closed membership; the references checked structurally, their evidence left to the
external verifier. Also accepted: the ledger refusals output as `(cut, EntryId)`, their reasons
remaining in the input, with the explicit limit that the snapshot alone does not give them.
**The implementation was not approved**: cohort members followed assignment chronology rather
than item register indices, which also changes the scorer's floating-point sum; the terminal's
declared item–group association was not compared, only its rule. Corrected after that review on
the owner's assignment (§14.11); **the correction awaits review**, not covered by the partial
judgment. The scorer stays approved; A1 and Phase 1 stay open.

### 14.1 The property

For `u` and a prefixed cohort `K ⊆ R_u` (§8.3) the quantity is `E[Ŝ_K | F_u]` (§8.1; under A,
`S_j = π_j = 1`): by D1, the mean over `K` of `E[g_uj(Y_j) | F_u]`. Where
`c_uj = P(Y_j ≠ I | F_u) > 0` the unique maximizing report is `q_c = P(Y_j = A | Y_j ≠ I, F_u)`,
a report `p` losing `c_uj (p − q_c)²` (§10.2); where `c_uj = 0` every report scores 0 in
expectation: the score is flat. Kept apart: the *properness of the expected contribution*, the
property claimed; the *cohort estimator*, whose final value D1 concerns, not its availability nor
the cohorts consolidated by a given time (§8.3); *utility and reputation* — `k_u`, probation,
shrinkage, weight, cap, CUSUM, a preference over the item's fate, the gate's use of the same
report (§10.4) — outside, the study's scores kept out of the reputation (Astra, `cbcbc67`).

### 14.2 Domain

| Element | Inside | Outside, with no approved rule |
|---|---|---|
| target | `Y_j ∈ {A, R, I}`, from the reference procedure fixed for the item's group before the reports (§8.1); the rating's meaning α (§10.3) | passing Level B (`02` §C.2); pool entry |
| information | `F_u` at commitment, its private part included (`16` §4.1) | `F'_u`, at the reveal decision |
| units | the assignment for the contribution; a prefixed cohort for the estimator; the item and its group for `Y` (§9.1) | cohorts selected by outcomes or times |
| reports | completed (H-d) | case 6, a missing report; case 7, a freeze not reached (§9) |
| outcomes | `A`, `R`; `I` only on §9.2's positive records | a pending outcome, silence, `Ω`, a shortfall record |
| baselines | `b_uj` defined on `{Y_j ≠ I}` (§14.4) | `panel_scores`' `None` (`16` §4.6) |
| availability | no condition of D1 | §8.3's property; overdue rules; values before consolidation |

6c and 7a are the study's reporting (§9.6), not scores.

### 14.3 Deviations

*Covered*: `u`'s reports as any `F_u`-measurable rule chosen jointly over `R_u` (H-c, §8.1), the
other roles' behaviour given; adaptive rules on `F_u` are such rules (`16` §4.3). A report that
moves a freeze, a disclosure or a group's load — through the band, an appeal, an entry — is such
a deviation: the theorem then needs H-c and C5 to hold under it, which the contract assumes and
§§7.3, 11.3 and 12.4's constructions show can fail. *Not covered*: withholding a reveal (`16`
§4.4); direct actions on timing, not through the reports' values; actions in other roles —
respondent, runner, holder, verifier, beacon member, a participant moving shared parameters
(§§10.4, 11.3, 12.1); several reviewers deviating together. Abstract constructions show channels
for several of them, not frequencies (§§7.3, 7.5, 9.4, 11.3, 12.4). An exclusion from the
analysis shows no deviation harmless; where one acts through H-c or C5, the contract assumes it
away.

### 14.4 Conditions

Status: **code**, realized by code; **caller**, the caller's to supply; **build**, missing;
**assumed**, declared, not shown. `20`'s check derives from supplied records and shows none of
H-c, C5, properness or availability.

| Condition (source) | Function | Status | Minimal evidence | If absent |
|---|---|---|---|---|
| H-a, `F_u ⊆ F_Φ` (§8.1) | D1's tower step | assumed for private information | the information model, declared | D1 not shown |
| `Y_j` hidden from `j`'s reviewers until `Φ_j` (§4, A) | a forecast, not a reading of `Y` | build: `Φ_j`, `Φ_G` and T1's start as log rules; T1 excludes only the planned execution | those rules; §13's predicates per pair; P5 and §12.4's (i) declared | the forecast reads `Y`; an arrival a deviation decides reaches H-c and C5 (§11.3) |
| no observation draw (§8.5) | C1 and H-e at `π = 1` | build: A's measurement pilot, pool entry apart from the verdict step (§12.5) | every assigned item's procedure recorded | C's obligations return |
| the beacon's other roles (`16` §§2, 6) | admission, panels, the extra round | code; values unselected only under `16` §6's member model, assumed | — | a coalition chooses who reviews what: outside D1, which is given `R_u` |
| H-b (§8.1) | the observed outcome has the reference's law | build: the group record before the assignments, the procedure pinned (§7.3) | that record; §9.2's evidence, recomputable | §7.1's counterexample |
| H-c (§8.1), resources included (§7.2, item 4) | `c_uj` and `q_c` free of `σ_u`: uniqueness | assumed; each construction excludes named channels only (§§7.3, 12.4) | declared with its constructions and residues; no record shows it (§12.1) | §14.3's channels |
| H-d: completed reports; C5 (`16` §4.6) | `p_uj` exists; the baseline invariant | reports: caller; `panel_scores`: code; frozen weights: caller | the reveals; `panel_scores.rs` (E); the weights used recorded (§13.4) | case 6; A10's `d ≡ 0`; §9.4's 3/16 |
| C4, `N_u = \|R_u\|` (§8.6) | a denominator free of reports | build: `SkillTrack` counts recorded items | §8.6, test 1 | withholding chooses the denominator |
| `I` a positive record, a term in log order (§§7.2, 9.2) | `I` not chosen by silence | build: no `I` state; test code in `20` | §9.2's record | a late record's holder chooses `I` |
| `c_uj > 0` (§7.5) | uniqueness | uncontrolled | none; a public frequency is not `c_uj` (§10.3) | the score flat |

*Under C*, two obligations apart. Inclusion (H-e, C1): a draw after the freeze from a source no
one reads or selects before — `16` §6's member model, or uniqueness, a real DKG and a
same-message fallback. Paths (H-b): audited groups' outcomes the reference's; unaudited entrants'
pilots sharing no resources unless reserved; reuse only with one group, sampling, procedure and
timing (§8.5).

*The absent baseline* (L, `panel_scores`). Completed reports do not by themselves give a
baseline: `first_panel_baselines` and `extra_round_baseline` return `None` when the reports
`16` §4.6's composition admits carry no positive frozen weight — every other first panelist at
weight 0, or the whole first panel for an extra reviewer — and `item_scores` then gives no score.
`None` is not the reviewer's forecast, not a numerical baseline 0, and authorizes no score 0. It
is apart from a terminal `I`, whose `g(I) = 0` (§8.1) needs neither `b` nor `p`, and from
`c = 0`, where every report scores 0 in expectation: with `c_uj > 0` and no baseline the expected
contribution is undefined, neither 0 nor flat. D1 needs the quantities it uses defined: for each
member of a prefixed cohort, `p_uj` (H-d) and, on `{Y_j ≠ I}`, `b_uj`; D1 is not stated for a
cohort holding a member without them. The operational policy is open: no imputation, fallback,
penalty, exclusion from assignments or change of the denominator is decided, and no cohort is
redefined afterwards by removing the members without a baseline. `20`'s fixtures produce no such
case (K3 leaves every other first panelist a positive weight); read in its `study.rs`, a verdict
without a baseline got no contribution while its cohort still got a final value or a bound,
which read that member as 0 — outside the check's approved perimeter. Confirmed by Astra on
`d2be7da` by reading; regressed and corrected in test code (`20` §10, approved on `d7a4449`): the
cohort gets no value and no interval, the cause named apart from cases 2, 6 and 7, the counts and
the defined terms kept; the operational policy stays open.

### 14.5 Implementation

*Present*, with no production caller: `difference_score`, `panel_scores`, `review::commit`, the
lifecycle's report and gate steps, the pilots' indeterminate readings; `outcome_of` and
`record_outcome` give `SkillTrack` a binary outcome, with no `I`; since §14.9, B-b's scorer
`cohort_scores`, a library component on typed inputs, approved on `6b9f42a` as an isolated
component. *In tests*:
`panel_scores.rs` reaches `record_outcome` on one band item; `cohort_scores.rs` checks the
scorer. *Synthetic check* (`20`): B-b by case, `N`, `O`, `V`, cohorts, freezes and terminals, on
supplied records; through its scoring path, §14.6 on finite constructions (`20` §11), approved
on `d7a4449`; that path runs through the scorer since `20` §12. *Missing*: the group record; A's
pilot path; freeze and start rules; attempt and terminal records with `I`; a production caller of
the scorer, reading assignments, reveals, frozen weights, selections and terminal records
authenticated and linked (§14.8 D; its contract specified in §14.10), and of `item_scores`; an
epoch driver (`10` T79). Feeding the
score to `SkillTrack` is a reputational decision outside the contract.

### 14.6 Adaptive verification

A test through the scorer, with two aims kept apart. Implemented in test code through `20`'s
check's scoring path, on finite constructions, on the owner's assignment, and approved by Astra on
`d7a4449` (`20` §11): (A) on two constructions, P1 and P2; (B) on the five constructions below.
The result is acquired within the perimeter of those constructions; a later change of the scoring
path would need checks pertinent to that change.

**(A) Within the theorem's domain.** The comparison fixes a prefixed cohort and its denominator
(§8.3, C4); gives every strategy compared the same declared information `F_u`; admits adaptive
joint reports over `u`'s assignments — rules on `F_u` reading, besides a signal on `Y`, what is
observable before commitment (the panel, other items' gate decisions, disclosures, §12.3); and
uses constructions where H-a–H-d hold for every admitted strategy. The truthful reference is
`q_c` conditioned on the information the model actually provides. The criterion is the expected
score over the whole cohort, every contribution the deviation changes included. Expected, as in
`16` §4.3: no admitted strategy above the truthful one — strictly below where it differs from
`q_c`, with positive probability, on an assignment with `c > 0`; equal where it differs only
where `c = 0`, the score being flat there. A comparison of fixed forecasts alone does not meet
the adaptive criterion.

**(B) The channels outside the hypotheses.** For each construction §14.3 cites, the test
reproduces the effect shown, under that construction's hypotheses, and attributes no stronger
conclusion to it. The earlier request to find "its gain" on every construction is withdrawn.

| Construction | Condition that fails | Effect shown | Not shown |
|---|---|---|---|
| §7.3, read for B-b in §7.5 | H-c: `u`'s report on `k` decides `k`'s entry and, by hypothesis, whether `k`'s group is piloted in `j`'s term, hence `j`'s floor and `Y_j`'s law (an entry channel A removes, §8.5) | `j`'s contribution 0 (`Y_j = I`) against a negative truthful expectation, −6/25 when the baseline equals the conclusive outcome | a net over the cohort under B-b: `k`'s cost not counted (§7.3's net, 19/25 against `δ²`, is the ternary score's) |
| §9.4 | C5: an effective baseline `b(p)` moving with `u`'s report, through a replacement under 7b, no such rule decided | that contribution: 3/16 at `p = ¾` against 0 at the truthful ½ | that a replacement responds so; an effect under `16` §4.6's composition |
| §11.3, timing | H-c through the outcome's law: `u`'s report sends `j` to the band and moves `Φ_G` | `j`'s contribution, the report's own cost included: 1/25 against 0 | effects on other assignments, outside the construction |
| §11.3, disclosure | C5 and H-c: a deviation on another assignment decides whether a signal reaches the reports forming `b_uj` | an effect on one contribution through disclosure and baseline: 0 without the signal, −¼ with it | a gain over the cohort: the deviation's other changed contributions not counted |
| §12.4, across groups | C5 and H-c: a deviation on `j ∈ G` decides whether `G`'s record reaches the reports forming `b_uk`, assumed equal to the posterior | an effect on `u`'s contribution on `k`: 0 against `−δ²` | as above |

Kept apart: effects on freezes, disclosures or load that `u`'s reports produce, which (B)
reproduces; direct actions on timing or in other roles, outside §14.3's domain, which it does not
model. A deviation that changes the information available — to `u` at a later commitment, or to
the reports forming a baseline — is modelled as such, not as a comparison of reports on one
`F_u`. A strategy left out of the test is not thereby shown harmless.

*Limit.* An exact enumeration on finite constructions can check the computation and reproduce the
channels named; it proves neither the theorem in general, nor H-c in the protocol, nor the
frequency of an attack.

### 14.7 Judgment

Against `15` A1's criterion: the contract *defines* the information at reporting (H-a; under A,
`Y_j` hidden until `Φ_j`), its log rules unbuilt and its private part assumed. *Properness* is
proved for the expected contribution under H-a–H-d, H-c assumed; but under A every inclusion
probability is 1, so the exploration design with IPW that the criterion names is replaced, not
proved — C would keep an IPW at `α` under C1's model. Astra's judgment (review of `11deec2`):
under A, `π = 1` removes the observation draw's problem, which can be a legitimate way to correct
A1 through a change of design; a non-trivial IPW need not be kept only to honour the original
criterion's name. That verifies neither the current exploration at `ε` nor arm C; closing A1
through A would need an explicit decision on the change, its realization and the pertinent
checks, none taken. The *adaptive test* is acquired within the perimeter of the constructions
approved on `d7a4449` (§14.6, `20` §11): within the strategies enumerated none beats the truthful
reports, strictly where `c > 0`; it checks the computation, not the theorem in general nor H-c in
the protocol.

Against the declared guarantee (`01` D35, `02` §C.2, paper `prop:ipw`) the contract narrows the
promise in four places: the target, `q_c` given conclusion rather than passing Level B; every
item piloted rather than exploration at `ε` with IPW; completed reports only; the reputation
outside. Adopting it would be a decision changing those texts, and would realize nothing. A1
stays open: at least the conditions to build and H-c declared with its residues would be needed,
§14.6's test being acquired, Astra's review deciding whether they suffice; A's capacity
(+316.825 slots against the declared 333.5, §8.5) and §8.3's availability weigh on any adoption.
A proposal, not a decision; §14.8 turns it into a decision proposal.

### 14.8 A decision proposal for A1 (rectified; the rectifications approved on `7d1b19e`)

**Status.** Claude Code's proposal after Astra's approval of `d7a4449`; not approved as a whole on
`23283f2` (§14) and rectified here in place, the rectifications approved by Astra on `7d1b19e`
(§14); the substantive choices remain the owner's. It adopts nothing and changes no design; `01` D35
and `02` §C.2 stand until a decision. Question: which decisions turn §14's contract into an
implementable correction of A1, and what can each honestly guarantee?

**A. Three decisions apart.**

1. *The forecast's target*: B-b's `q_c` on `Y ∈ {A, R, I}`, `I` adding 0, against passing Level B.
   The guarantee kept is §14.1's: for completed reports with defined baselines, the expected final
   value of a prefixed cohort's score given `F_u` is maximized by `q_c`, under H-a–H-d with H-c
   declared and its residual channels named (E); uniquely where `c > 0`, while the contract admits
   `c = 0`, where the score is flat. It holds under A and under C alike: B-b under C keeps `q_c`
   and needs the same conditions on reports, baselines and invariance. Nothing on when a value
   exists, on withholding or on the reputation.
2. *The observation design*: A (`π = 1`, every reviewed item's prefixed group piloted, no
   observation draw) or C (groups audited at `α` after the freeze); D35's exploration at `ε` has
   no guarantee under the current sequence (`16` §1). They differ in H-e and in the paths H-b
   covers (B), not in the target. A is the analytic reference; C stays in the comparison even if
   A proves sustainable (Astra, `44f0dbd`, again on `23283f2`).
3. *The incentives' scope*: the contract concerns the expected contribution, not the reputation
   actually used (`k_u`, shrinkage, weight, cap, CUSUM; `16` §§4.5, 7).

Recommended: decide (1) apart from (2), since the approved results and the scorer concern `q_c`
under either design; keep (2) open, A the reference and C the comparison; keep (3) outside A1's
guarantee unless the owner decides otherwise. No adoption now: whichever the design, D's
conditions are unbuilt, and A's capacity (F) is unsettled. A circumscribed implementation needs
none of these decisions: B-b's scorer, accepted on `23283f2` and implemented in §14.9 with no
production caller; no resources or study either, S2 measuring rates and costs, not H-c (`19`
§10.6).

**B. Hypotheses and T19, apart.** H-e: the effective inclusion probability equals the recorded
one. H-b: on each path the observed outcome has the reference's law. H-c: the joint law of
baseline and outcome is invariant under `u`'s deviations. A removes the observation draw: H-e is
the design itself, every reviewed item's group piloted, which its records must show; H-b concerns
the execution of one procedure pinned before the reports. Recording that procedure does not show
that the execution followed it: §9.2's evidence makes each recorded attempt recomputable, which is
checkable evidence, not a proof of H-b. Under C, H-e rests on the draw after the freeze, on `16`
§6's member model today, and H-b covers the audited groups' paths and the entrants' pilots. T19 is
a real DKG, which the threshold-signature beacon waits for (`10`); `16` §6 keeps apart uniqueness
for a fixed message (the threshold signature), unpredictability (the DKG, fewer than `t` corrupted
shares), availability (`t` honest shares) and a fallback on the same message (no reroll). None
follows from T19 alone or from the others, and none shows H-b or H-c.

**C. Changes against the current promises.**

| §14.7 | Today | Proposed | Guarantee | Consequence |
|---|---|---|---|---|
| target | `p_uj` forecasts passing Level B (`02` §C.2); an indeterminate pilot stays pending (`15` A11) | `q_c` on `Y ∈ {A, R, I}`, `I` a positive terminal record adding 0 (α, §10.3), under A or C | modified: properness for `q_c`, unique where `c > 0`, flat where `c = 0`; none for passing | the rating's meaning changes; `τ`, the gap, the band and the polarization cut to revalidate (T25, T83) |
| observation | live outcomes and exploration at `ε`, weighted `1/ε` (`01` D35) | A: each reviewed item's prefixed group piloted under a procedure pinned at association, `π = 1`, pool entry needing the gate's or an appeal's admission and `Y = A`, T1's start; or C: groups audited at `α` after the freeze | D35's IPW replaced, not proved: under A by `π = 1`; under C an IPW at `α`, H-e on the draw's source (B) | under A, F's capacity, the gate's false negatives measurable on every rejection, `O_u` +1 per judgment once its record is available; under C, `α` and `α c` per judgment |
| completed reports | a missing reveal freezes the item (`PartialEpoch`); T58 decided, not built; `16` §4.4's `s_ns ≤ −1` a proposal | case 6 unscored, not excluded, the cohort without value (6c); case 7 likewise (7a) | left open: withholding not shown unprofitable | the missing reveal blocks its cohort and, through case 7, the item's other reviewers'; no value does not make the procedure proceed: T58 is needed, its replacement shown to keep C5 (§9.4) |
| absent baseline | `None`, its effect undecided (`02` §C.2); A10's fallback on golden items | the term undefined, the cohort without value (`20` §10); no imputation, fallback, exclusion or change of `\|K\|` | left open: D1 not stated for that cohort | it arises when an item's other first panelists all weigh 0, all on probation for instance (`review_weight`); bootstrap founders weigh 1; no frequency at the start is inferred; a policy is needed before production |
| reputation | `S_u` through `k_u`, shrinkage, cap, CUSUM (D33–D36) | B-b's scores out of `SkillTrack` | left open (`16` §§4.5, 7; §8.4) | until a separate decision B-b moves no review weight (invariant 4) |

**D. Minimal path and the runtime's boundary.** *Score*: §14.9's scorer, from typed inputs. For it,
the inputs' authenticity and semantic validity are declared preconditions. In the protocol they
bear on the guarantee itself, not only on a distributed exercise: who may produce a reveal, a
selection or a terminal record, the checks a record passes before it is read, and the link of
each record to its assignment, item, group rule and outcome decide which `p`, `π` and `Y` the
score reads. Left to the future caller: `R_u` complete and the cohorts fixed from the assignment
record alone; each reveal authenticated and linked to its commitment and assignment, its close in
log order; the epoch's frozen weights; `Φ_j` from applied steps; `S_j` and `π_j` as the design
records them, H-e included; each terminal record validated against its group's rule (§9.2) and
linked to its item. *Records*: reused `pilot::screen`, the stage gates, the lifecycle's pilot steps,
`Ledger::apply`; missing the group record, A's pilot without `Explore`, attempt and terminal
records with `I`, shortfall records, `Φ_j`, `Φ_G` and T1's start as log rules. *Progress and
availability*: T58, a term in attempts, §8.3's property; they decide when a value exists, not D1.
*T79*: A1 needs the parts realizing those responsibilities — the deadlines as cuts, the pilot
steps in log order, who publishes each step and the check before co-signing (invariant 7), how a
person's commit and reveal reach a member, linked to its assignment — not the epoch run between
node processes as a whole (T78) nor T76's view change, which bear on liveness. T77 bears on E's
disclosures. *Reputation*: outside.

**E. Hypotheses left outside.** Excluded by A with T1, once its records exist: the observation
draw; §7.3's entry channels (§7.5's −6/25 lies outside A); the planned execution before `Φ_G`.
Still assumed, under A and, with the draw's, under C: H-a's private part; H-c through time,
samples and resources, T1's start following the slowest freeze (§11.3's 1/25); disclosures
reaching the reports that form a baseline, within a group (§11.3's −¼) or across groups (§12.4's
`−δ²`), (ii) checkable per pair, (i) declared; a replacement moving baselines (§9.4); respondents
answering their reviewed items; the beacon's other roles under `16` §6's model; other roles,
coalitions, withholding. `20`'s order check is a replay check, and its synthetic tests and §14.9's
check the computation where the hypotheses hold by construction: none shows the outcome's law
invariant. The contribution's properness gives no reputational incentive.

**F. Resources and the owner's decisions.** From §§4.1 and 8.5 only: A +316.825 slots, 667 a month
against the reference's 350.175, twice the declared 333.5; C from +0 (`α = ε`, full reuse) to
+333.5 (`α = 0.525`, separate), +158.4125 with full reuse at that `α`; an equal expected count is
no equal precision. Slots are not persons, answers with anchors, fits or measured times (`15` D2,
D4). For the owner, none blocking the scorer and none needed now:

- **O1, the target**: whether A1's correction promises §14.1's properness for `q_c` rather than
  for passing Level B, under A or C alike; `01` D35, `02` §C.2 and the paper's `prop:ipw` then
  rewritten.
- **O2, the observation design and capacity**: A's capacity, or C with its draw's source and reuse
  conditions; C stays in the comparison either way.
- **O3, reputation**: whether A1's closure needs the incentives of the reputation actually used or
  only the score's mean (`16` §9).

For Astra: whether the guarantee meets `15` A1's criterion as read on `11deec2`; T1; the direction
for missing reports and absent baselines; §14.10's contract of the scorer's caller.

**Next deliverable**, accepted by Astra on `23283f2`: B-b's scorer, implemented in §14.9 and
approved on `6b9f42a` as an isolated component; then its caller's snapshot, specified in §14.10.
No protocol, A, C or T1 adopted; A1 not closed.

### 14.9 B-b's scorer (no production caller; approved on `6b9f42a` as an isolated component)

**Status.** Claude Code's implementation on the owner's assignment after Astra's review of
`23283f2`: `protocol::cohort_scores` (`crates/protocol/src/cohort_scores.rs`), a pure, deterministic
library component with **no caller in the production runtime**. Partly reviewed by Astra on
`7d1b19e` (§14): `Undrawn`, the refusal of a missing first report at the freeze and the interval
with a pending member without a baseline accepted in the contract, as stated below; the scorer not
approved, for a numeric defect on its declared domain, corrected under *Numerics*. Partly reviewed
on `0ca81e9` (§14): the corrections of cases A–E approved; the scorer not approved, the common
scale cancelling a term from `known`; `known` given its own scale under *Numerics*. **Approved on
`6b9f42a`** (§14) as an isolated component, within this contract and its numeric limits. It
adopts no A, C or T1 and changes no observation design, reputation or resource; its output
reaches no `SkillTrack`, vote weight, probation, cap or CUSUM. Reading `π_j` certifies none of
H-e, H-b, H-c. This section is its contract; the tests below and `20` §12 check it.

**Inputs**, typed; no production record format exists and none is read or imitated.

- `Item`, one per item, named by its index: `first`, the first panel in assignment order, each
  `Panelist` with its nym, `Report` and frozen review weight; `extra`, the extra round's reviewers,
  nym and report, empty without one, no weight read; `frozen`, `Φ_j` reached; `selection`.
- `Report`: `Revealed(p)`, a valid reveal before its close; `Open`, the reveal period running;
  `Missing`, closed without a valid reveal.
- `Selection`, explicit, never inferred from a terminal record's absence: `Undrawn`, a drawn
  design's selection not yet recorded; `NotSelected { inclusion }`, `S_j = 0` final;
  `Selected { inclusion, outcome }`, `S_j = 1`, `outcome` `A`, `R` or `I` once terminal, `None`
  while pending. `inclusion` is the recorded `π_j`, 1 under A.
- `Cohort`: a nym and its prefixed members, by item index, fixed by the caller from the assignment
  record alone.

**Cases**, per assignment, in this order (§8.2): a missing report, case 6, whatever else; a freeze
not reached, case 7; then by selection — `Undrawn`, a drawn design's frozen item before its draw
(§8.3), no term and no interval; not selected, case 1, term 0 for no observation, out of `O` and
`V`; selected and pending, case 2, no term, never 0; `A`, `R`, cases 3–4; `I`, case 5, term 0,
in `O`, not in `V`.

**Baselines and terms.** Composed at the freeze only, and only as `16` §4.6 decided, through
`panel_scores` over the first panel's reports and frozen weights: a first panelist against the
other first panelists, an extra reviewer against the whole first panel; no extra report enters a
baseline. On a verdict the term is `difference_score(p, b, o)/π_j` (§8.1), and `NoBaseline` where
`b` is absent (§14.4); on `I` and on a non-selection, 0 without a baseline. A frozen item whose
first panel lacks a report is refused: no composition over the reports present or a replacement
is decided (§9.4, T58).

**Output, counts and values.** Every assignment, in item order, first panel then extra round,
with its case, `π_j`, its baseline once composed and its term. `Scores::counts` gives `N_u`,
`O_u`, `V_u` over `u`'s assignments among the items supplied; a cohort gives `|K|`, its
denominator, with `O` and `V` over `K`, and its known sum, `OutOfRange` where it leaves `f64`'s
range (*Numerics*). Its value is final, `known/|K|`, once every member is in case 1, 3, 4 or 5
with its term defined; with case-2 members besides, D3's bound, the sum within `Σ 1/π_j` of its
known part (the sum's ends `OutOfRange` where one leaves the range) and the mean within
`Σ 1/π_j / |K|` of `known/|K|`, a provisional interval on the final value conditional on its
coming to exist, no availability promised (a pending member without a baseline does not block it,
its term needing one only at a verdict); otherwise `Unavailable`, listing every member in case 6,
7, undrawn, without a baseline, or out of range, in the cohort's order, all causes kept.

**Refused, never repaired** (`InputError`, the first found, items before cohorts): a report
outside `[0, 1]` or not finite; a weight negative or not finite; `π_j` outside `(0, 1]`; a
non-selection recorded at `π_j = 1`; a nym twice on one item's panels; at the freeze, a report
still open or a first report missing; a terminal outcome before the freeze; an empty cohort; a
member index out of range, repeated, or not an assignment of the cohort's nym. An error is an
input the contract excludes; a cohort without value is a legitimate state; `OutOfRange` is
neither, an input in the domain whose result no `f64` holds.

**Numerics** (after Astra's reviews of `7d1b19e` and `0ca81e9`; approved on `6b9f42a`). The scorer
computes in `f64`, deterministically; it promises no exact real arithmetic. Its domain is
unchanged: no threshold on `π_j` or the weights, no clipping, saturation, imputation, fallback to
the reviewer's report or policy; on the inputs it accepts no output is NaN or infinite.
`u = 2^−53`, `ε = 2^−52`, `γ_k = ku/(1 − ku)`.

- *Baselines* (`panel_scores`, both compositions, one function): the weights averaged are scaled
  by `2^−e`, `2^e ≤ w_max < 2^(e+1)` (`libm::ilogb`, `libm::scalbn`), before the sums. The largest
  becomes `[1, 2)`, the total `[1, 2n]` and the weighted sum `[0, 2n]` for `n` forecasts: no sum
  overflows; underflow, confined to weights or products below `2^−1022` of the largest weight,
  moves the mean by less than `n·2^−1073`; a positive weight never leaves `None`, no weight still
  does. `|b̂ − b| ≤ γ_{2n}·b + n·2^−1073`, within `2nε`.
- *Terms*: `g/π_j`, one correctly rounded division of `g ∈ [−1, 1]`, `|g/π_j| ≥ |g|`; it leaves
  the range only for `π_j` below `2^−1022`, subnormal, and is then `NoTerm::OutOfRange`, a defined
  term no `f64` holds, apart from `NoTerm::Undefined(NoBaseline)`. For `p, b ∈ [0, 1]` and
  `o ∈ {0, 1}`, `|t̂ − t| ≤ (2|b̂ − b| + 5ε)/π_j`, within `(4n + 5)ε/π_j`.
- *Cohorts*: two sums in the cohort's order, each scaled by the power of two of its own largest
  magnitude — the defined terms' by `2^−k_t`, the pending members' `1/π_j` by `2^−k_w` (`k = 0`
  for an empty or zero sum). `known` and the final value come from the terms' sum alone, scaled
  back by `2^k_t`, the final value divided by `|K|` in that scale: no pending member enters them or
  their scale (corrected after `0ca81e9`, whose scale common to both sums turned a term of
  −2^−100 beside a radius of 2^1022 into −2^−1122, rounded to 0). The bound moves both sums to the
  coarser scale `2^k`, `k = max(k_t, k_w)`, adds and subtracts them there, divides by `|K|` and
  scales back. A sum can leave the range while its mean does not: `known` and the bound's `sum`
  are `Result<_, OutOfRange>`. The final value and the bound's `mean` are never out of range (D):
  in its own scale every value is below 2 in magnitude, and moving a sum to a coarser scale only
  shrinks it; rounding is monotone, so the largest computed sum of `m` of them is that of `m`
  copies of `2 − ε`, which stays at least an ulp below `2m`; divided by `|K| ≥ m`, it rounds to at
  most `2 − ε`, which scales back to at most the largest `f64` (C, checked for every `m ≤ 20 000`
  and, for the bound's two sums, every split of `m < 1 500`).
- *Rounding and underflow of `known`* (D; `m` terms `t`, `m_u` of them rounded by their scaling):
  scaling up (`k_t ≤ 0`) is exact; scaling down (`k_t ≥ 1`) is exact for a term of at least
  `2^(k_t−1022)` in magnitude and otherwise rounds it to the subnormal grid, within
  `2^(k_t−1075)`; scaling the sum back is exact for `k_t ≥ 0`, unless `OutOfRange`, and within
  `2^−1075` for `k_t < 0`. Hence `|known − Σt| ≤ γ_(m−1) Σ|t| + (1 + γ_(m−1)) m_u 2^(k_t−1075)`,
  plus `2^−1075` for `k_t < 0`; a single term is exact. The bound is absolute: neither exactness
  nor the absence of cancellation is promised. A small sum among large opposite terms has no
  relative bound, and a term of at most `2^(k_t−1075)` rounds to 0 — terms 2^1000, −2^1000 and
  2^−100 give `known = 0`, where a direct sum in that order gives 2^−100.
- *The final value* adds its quotient's rounding, within `u` relatively or, where the scaled
  quotient is subnormal, `2^(k_t−1075)`; for `k_t < 0`, one more within `2^−1075` scaling back.
- *The interval* adds, to the two sums' bounds (the widths' as the terms', with `k_w`; a width is
  rounded by its scaling only for `k_w = 1023` and a width below 2; each `1/π_j`, one correctly
  rounded division, within `u` relatively): moving a sum to `2^k`, a rounding within `2^(k−1075)`
  where it falls below `2^(k−1022)` there; each end, a rounding within `u` relatively; each mean,
  its quotient's, within `u` relatively or `2^(k−1075)`; scaling back, `k ≥ 0`, is exact. Its
  centre is `known` only within these bounds: in the counterexample −2^−100 moved to `2^1022`
  rounds to 0, the ends ±2^1022 are the exact ends rounded, and `known` keeps −2^−100.
- *A member out of range*: a term beyond the range leaves its cohort `Unavailable` with
  `Cause::OutOfRange`, beside every other cause, and its `known` `OutOfRange`. A pending member
  whose `1/π_j` leaves the range (`π_j` below about `5.6·10^−309`) leaves its cohort
  `Unavailable` with the same cause, but not necessarily `known`, the defined terms' sum: with no
  defined term, `known = Ok(0)` is the correct sum. Where `OutOfRange` is named, an exact value may
  exist (opposite terms cancelling, `|K|` beyond `2^50`): a declared limit of the representation,
  not a rule of the protocol.
- *Identity with `7d1b19e`'s direct computation* (D, narrowed after `0ca81e9`): `known` equals the
  direct sum of the terms in the cohort's order, bit for bit, wherever that sum stays finite and
  no term is rounded by its scaling — scaling then commutes with every rounded addition, a sum
  below `2^−1022` being exact in both scales; the final value equals that sum divided by `|K|`
  where, besides, neither exact quotient, scaled or not, is nonzero below `2^−1022`. No identity
  is claimed for the interval. Withdrawn: `0ca81e9`'s identity wherever the former computation met
  neither an overflow nor a subnormal intermediate, a condition the counterexample meets. Its
  comparison of 988 364 calls stays empirical evidence on the fixtures run (E, below).

**Left to the caller, unchecked** (§14.8 D): the inputs' authenticity and links; `R_u` complete;
the weights frozen for the epoch; `Φ_j`; `S_j` and `π_j` recorded by the design before the outcome
(H-e), and when the draw happened; each terminal record validated (§9.2); the cohorts prefixed. No
validator of arbitrary flows; no H-b or H-c. Their sources, checks and gaps: §14.10; the snapshot,
§14.11.

**Verification** (E, Claude Code, 2026-10-08; values by hand and with fractions, two of them
corrected on recomputation, `20` §12): `crates/protocol/tests/cohort_scores.rs`, 10 tests — terms at
`π` 1, ½, ¼ (3/16 and −5/16 at 1; ¾ and −5/4 at ¼, beyond `[−1, 1]`); a non-selection apart from `I`
and pending; `N_u` 4 against `|K|` 2, the mean 17/128 over `|K|`; a radius of 6 = 1/½ + 1/¼, not the
2 pending; refused numbers and combinations; `16` §4.6's compositions, `panel_scores`' hand values;
no weight left; four coexisting causes; and a finite enumeration — two groups, one draw at ½ shared
by two items and one at ¼, three outcomes with a common latent, 72 states — where on all 27 report
vectors of a grid the scorer's expected cohort value equals D1's `(1/3) Σ_j E[g(Y_j)]`, computed
apart (−41/1728 at ½ everywhere), and a recorded `π` of ¾ against an effective ½ moves it to
−91/5184. That checks the arithmetic under the construction's hypotheses, not C in the protocol.
`20`'s check runs through the scorer (`20` §12).

*Numeric correction* (E, Claude Code, 2026-10-08; values analytic or exact, beforehand). Nine
regressions written against `7d1b19e`'s code and run there, all failing on their first unmet
assertion: through the scorer, A (others ½ at weight `10^308` each: baseline 0, not ½), B (others 1:
baseline NaN, not 1), C (one other ½ at the smallest positive weight: baseline 0, not ½), D (3/16 at
the smallest positive `π_j`: `Ok(Some(∞))`; pending there: `Bound` with infinite ends), E (two terms
of `1/10^−308`: `Final(∞)`; two pending at `10^−308`: an infinite mean interval); in `panel_scores`,
A and B (first and extra baselines 0 and NaN, not ½ and 1) and C (0, not ½). A temporary probe on
the same code read the values those assertions did not reach: terms ¾ (A, not 0), NaN (B, not −¼),
15/16 (C, not 3/16); `Final(∞)` (D); with two smallest weights, baselines 1 and 0 for reports 0.875
and 0.25, and 0.5 for 0.5625. After the correction all nine pass, D's and E's assertions completed
with the explicit `OutOfRange` outcomes; two checks added after it: `OutOfRange` named beside a
missing report and an absent baseline, kept apart from `NoBaseline`; at the range's edge, two terms
of `2^1022` sum to `2^1023` with mean `2^1022`, exactly, and three terms near the largest `f64` have
a finite mean within `2ε` while their sum is `OutOfRange`. A temporary instrumentation, removed,
recorded every output of `score`, `first_panel_baselines` and `extra_round_baseline` over the
earlier tests of the scorer and `panel_scores` and the check's 39 — 988 364 calls — before and
after: identical bit for bit, the new types' `Ok` normalized. `cohort_scores.rs` has 19 tests,
`panel_scores.rs` 9.

*Correction of `known`'s scale* (E, Claude Code, 2026-10-09; values exact, beforehand). A regression
through `score`, `a_pending_radius_does_not_round_away_the_known_sum`: `u`'s report 2^−50 on a
frozen item, the other first panelist's 0 at weight 1 giving the baseline 0, selected at `π = 1`
with `R`, a term of −2^−100; and a frozen item selected at `π = 2^−1022`, pending. Expected:
`known = Ok(−2^−100)` for the cohort of the first item, `Final(−2^−100)`, and for the cohort of
both, with the bound's `sum` ±2^1022 and `mean` ±2^1021, by exact equality. On `0ca81e9`'s code it
failed at the second cohort's `known`: `Ok(-0.0)` against `Ok(−2^−100)`, the first cohort's
assertions passing; a temporary trace read that cohort's interval, ±2^1022 and ±2^1021, the same
before and after. After the correction it passes. A temporary trace, removed, hashed the inputs and
output of every `score` call over the scorer's 19 earlier tests and the check's 39 (`panel_scores`'
9 make none) — 186 509 calls, 1 604 distinct inputs — before and after: the same output for every
input; it tells the regression's two outputs apart. Calculated (C), in a Python transcription of the
sums against exact rationals, not kept: the bounds above and the narrowed identity, on 60 000 random
cohorts and 20 000 built to round widths by their scaling and the radius moved to the terms' scale;
and the cancellation example. `cohort_scores.rs` has 20 tests.

### 14.10 The scorer's future caller: the snapshot's specification (approved on `8dc7087`)

**Status.** Claude Code's proposal, documentation only, on the owner's assignment after Astra's
approval of `6b9f42a`: the contract of a future caller building §14.9's inputs from records. Partly
reviewed by Astra on `3e34f6f` (§14): the direction of three of its choices accepted for the
isolated component, the section not approved. Rectified in place after that review, on its four
points, each with its acceptance criteria: the cuts' metadata the cohorts' closure reads (item 4,
*Input*); the boundary with the ledger (*Input*, *Preconditions and checks*); the state a refused
record keeps and the progression between snapshots (*Records and states*, *Prefixes and updates*);
the register of every assignment beside the counts over the items scored (item 3, *Output*).
**Approved by Astra on `8dc7087` as the specification of the isolated snapshot component** (§14),
with one precision made below on the digest and the prefixes. Its implementation was partly
reviewed on `9c7cb23`, its correction awaiting review (§14.11); this approval does not cover it.
No production caller and no production record format exist. Read (L)
at `6b9f42a`: the code cited below by symbol; §§8.2–8.3, 9.1–9.2, 11.1, 13.1, 14.4, 14.8 D, 14.9.
Read again at `3e34f6f` for the rectification: `network::cut` (`Cut`, `cut::added`), `ledger`
(`Ledger::apply`, `CutReport`, `Ledger::refused`), `node` (`NodeState::apply`, `NodeState::step`),
`events::NodeEvent`, `lifecycle::step`, `cohort_scores` (`Scores::counts`, `InputError`), the
check's `study::freeze`. **Exists** marks a source in code; **Missing**, none in code; **Proposed**,
a structure suggested here, neither built nor approved. The scorer's tests and `20`'s synthetic
records (`S0`, group, start, attempt and terminal records; K1–K8) are test inputs, cited only as
precedents of a mapping, never as production records.

**Four properties apart.** A record can be authentic, well linked and in a documented order without
showing that the procedure behind it ran as the hypotheses need.

| Property | Given today (Exists) | Not shown |
|---|---|---|
| authenticity: who wrote a record | an entry's writer signature, checked against the network's writers (`network::replica::{SignedEntry, WriterSet, Replica::check}`); lifecycle steps and results only from consortium members (`ledger::Ledger::apply`, `Refusal::NotAuthorized`); a cut applied only under `t` members' signatures (`LedgerError::Unsigned`); a judge id proven for its context (`admission::admit`, through `NodeEvent::AdmitReviewer`) | who made a commitment or a reveal: a member writes the step, and the judge's proof is bound to `review::review_context(item, epoch)`, not to the commitment |
| integrity of links | an object named by its CID; a reveal opening its commitment for that nym and item (`review::reveal`, INV-12); an assignment naming the item it moves (`node::Rejection::ItemMismatch`); panel membership, one commit and one reveal per nym (`lifecycle::step`) | that a panel is the beacon's draw; that a committing nym is the judge `AdmitReviewer` admitted; links to a group procedure, a selection or a terminal record, none of which exists |
| documented order | the cut order (`network::cut::added`): numbers in turn, epochs non-decreasing, no retraction; refused entries listed, the state unchanged (`ledger::CutReport`) | when an object was produced, held or read off the log (§13.1) |
| correctness of execution | a step the lifecycle refuses is never applied; the beacon round replayed (`Ledger::apply`); §9.2's evidence recomputable where it is recorded | H-b, H-c, H-e: a signature, a hash or a replay shows none of them (§14.8 B) |

**Per input of `score`.**

1. **Item and index** (`Item`; `Cohort::items`). *Meaning*: one reviewed item, named by its `Cid`
   (`deposit::Draft::content_id`, entered by `NodeEvent::Deposit`); the scorer's index is the
   caller's, deterministic — Proposed: an item's register index is its rank in the order of the
   items' first applied `AssignReviewers`; its scorer index, its rank among the valid items in
   that order; the snapshot links both to the `Cid` (*Output*). *Exists*:
   `NodeEvent::Step { item, .. }`; `NodeState::item`. *Checked*: a step for an item no deposit
   started is refused (`node::Rejection::UnknownItem`), an assignment naming another item too
   (`ItemMismatch`). *Missing*: nothing for identity. *If absent*: an item with no applied
   assignment enters no input; a refused step is never applied, so never read.
2. **Assignments, first panel and extra round** (`Item::first`, `Item::extra`; `j ∈ R_u`).
   *Meaning*: the nyms assigned, in assignment order, the first panel and the band's extra round
   apart, no nym in both. *Exists*: applied `lifecycle::Event::AssignReviewers { panel, item }` and
   `Event::AssignExtraReviewers { panel }`. *Checked* (`lifecycle::step`): an odd first panel in
   `[7, 11]`, distinct; an extra panel of 1 to `K_EXTRA_MAX`, distinct, outside the first, assigned
   once; written by a consortium member. *Missing*: that a panel is the beacon's stratified draw
   (`review::assign_from_beacon`, `assign_extra_from_beacon`, `assign_diverse_from_beacon`): neither
   the eligible reviewers with their positions and clusters nor the item's slot is recorded
   (`Event::Admit` carries a boolean), so no reader can recompute it; T79's check before co-signing
   (invariant 7) is where it belongs. *Not yet available*: before `AssignExtraReviewers` a band
   item's `extra` is empty and the item unfrozen. *Incoherent*: refused by the lifecycle, never
   applied; listed as applied, it makes the prefix incoherent (*Preconditions and checks*).
3. **`R_u` and the counts** (`Scores::counts`). *Meaning*: every assignment of `u` in the prefix,
   from the memberships alone, whatever its item's validity; `N_u = |R_u|` (C4). *Exists*: derived
   from 2. *Missing*: an end to a reviewer's assignments: a later cut can add one. Proposed: `R_u`
   at cut `n` is `u`'s memberships in the steps applied through cut `n`, the ledger's refusals
   excluded, kept whole in the snapshot's register of assignments, invalid items included, and
   reported with `n` (*Output*). `score` receives the valid items only: `Scores::counts` then gives
   `N`, `O` and `V` over `u`'s assignments on those items, a subset of `R_u`, and is reported as
   such, never as `|R_u|`. The register gives `|R_u|` and lists the assignments on invalid items
   with their item's reason; it states no `O` or `V` for them, their cases not computed — an
   invalid item can lack the selection that decides one. *Incoherent*: a nym twice on one item,
   refused by the lifecycle and by the scorer (`InputError::Reassigned`).
4. **Cohorts** (`Cohort`). *Meaning*: sets fixed by a rule on the assignment record alone (§8.3),
   never by reports, outcomes or the arrival of records. *Missing*: any cohort rule or record;
   `20`'s cohorts are listed in a synthetic `S0` (K2). Proposed, its direction accepted on
   `3e34f6f` for the isolated component: `K(u, e)`, `u`'s assignments, first panel or extra round,
   applied by cuts of epoch `e` (`Cut::epoch`), in register order; none exists where `u` has no
   such assignment, so no empty cohort reaches the scorer (`InputError::EmptyCohort`). *Closure*:
   `K(u, e)` is closed in the snapshot at `n` once the prefix holds a cut of epoch `e` with `closes`
   set, or a cut of a later epoch: after either, `cut::added` accepts no cut of `e`
   (`CutError::Epoch`), so no assignment can join it. The rule reads the cuts' metadata, not their
   events: a cut applying no `NodeEvent` — adding no entry, or only member objects, or only entries
   the ledger refuses — closes `K(u, e)` all the same, and two prefixes applying the same events
   under different `closes` or epochs leave it open in one and closed in the other. The snapshot
   therefore reads every cut of its prefix (*Input*). Epochs need not be consecutive: `cut::added`
   lets a cut skip any, and a skipped epoch has no cohort. *Not yet available*: before its closure
   the cohort is open — reported with its members so far, not scored: scored on the assignments
   applied so far, it would be selected by time. *Not a reveal close*: the closure fixes membership
   and says nothing of reports. `Cut::closes` closes an epoch's deposits; the window it opens in
   `Ledger::apply` is the beacon round's (`MemberObject::Reveal`), not a panel's (`Event::Reveal`).
   A closed cohort can hold `Open` reports and unfrozen items (case 7), never `Missing` (item 5).
   *Incoherent*: a cohort record supplied besides the rule, as a study's `S0` is, must equal the
   rule's membership; otherwise it is refused, the rule's membership kept.
5. **Reports** (`Report`). *Meaning*: `Revealed(p)`, a reveal applied before the reveal close and
   opening the commitment the same nym made for the item; `Open`, no close recorded; `Missing`, a
   close recorded with no valid reveal before it. *Exists*: applied
   `Event::Commit { nym, commitment }`, `CloseCommits` and `Reveal { nym, prob, nonce }`, per round.
   The first panel's reveals leave the item's state at `Score` (`State::Pilot1` and the later states
   hold none): they are read from the applied steps, not from `NodeState::item`. *Checked*
   (`lifecycle::step`): a commit only by a panelist, once; a reveal only after the commits close,
   once, by a committer, with `p ∈ [0, 1]` and `review::reveal` opening
   `commit(p, nonce, nym, item)` (INV-12); otherwise refused. *Missing*: (a) authorship: a member
   writes the step, and nothing binds a commitment to the holder of the judge's credential;
   `NodeEvent::AdmitReviewer` proves a judge id for `review_context(item, epoch)`, not for a
   commitment, `step` does not check a committing nym against that set (`NodeState::panel`), and no
   step records the epoch of an item's review; the part needed is T79's channel from a person to a
   member, linked to the assignment. (b) A reveal close: `lifecycle::Event` has no deadline, no
   deadline is a cut (T79), T58 is decided and not built, and `Score` and `Resolve` refuse a partial
   round (`PartialEpoch`): `Missing` cannot arise from today's records. *If absent*: without a close
   the caller passes `Open` — the item unfrozen, case 7 for its reviewers — never `Missing` from
   elapsed time, silence or a cohort's closure (item 4), never an imputed `p`. The caller reads the
   reveals as the replicated set holds them and publishes nothing more (`08` PRIV-004, T77).
6. **Frozen weights** (`Panelist::weight`). *Meaning*: each first panelist's review weight frozen
   for the epoch, finite and non-negative, never recomputed from the item's reports, its outcome or
   later reputation (`02` §C.2, `16` §4.6); Proposed: the epoch of the cut applying the item's first
   `AssignReviewers`. *Exists, in part*: the tracks rebuilt from applied `NodeEvent::Results`
   (`results::ResultsState::track`; `ResultRecord::{ReviewerScore, ReviewerUnobserved}`), a weight
   through `probation::SkillTrack::weight(is_founder, w_max)`, the cap
   `orchestrator::epoch_weight_cap`. *Missing*: the founder set has no record
   (`probation::FounderSet` is built in tests only), so a founder's weight 1 is not derivable; no
   record states the weights used (§13.4) or the log position at which an epoch's weights freeze.
   Proposed, its direction accepted on `3e34f6f` for the isolated component: a weights record per
   epoch, placed before the first assignment applied by a cut of that epoch (*Input*), giving each
   reviewer's weight, or the prefix of results and a recorded founder set it derives from. *If
   absent* when an assignment of its epoch is applied: an order the contract excludes, so the item
   is invalid, not pending; likewise a first panelist without a weight, or a weight negative or not
   finite (the scorer's `InputError::Weight`). A weights record placed later is refused and leaves
   its epoch without weights.
7. **Freeze** (`Item::frozen`, `Φ_j`), with the cuts and the applied order. *Meaning*: `16` §5's
   first applied record after which `j`'s reports, gate decision and re-decision, appeal and
   memberships are irrevocable. *Exists*: computable from the steps applied in cut order
   (`cut::added`, `Ledger::apply`); no record names it (§11.1). Proposed rule on today's lifecycle,
   its direction accepted on `3e34f6f` for the isolated component: the position (*Input*) of `j`'s
   first applied step whose next state is none of `Deposited`, `Admitted`, `InReview`, `Revealing`,
   `SupplementaryReview`, `AppealEligible` — `Score` on `Pass` or `Reject`, `Resolve` unless
   appealable, `Appeal` or `AppealExpires` after an appealable decision; it is the rule of `20`'s
   test-code `study::freeze`, approved there within the check's perimeter. Every
   first and extra report is then `Revealed`. *Missing*: T58's replacement and quorum, which `16` §5
   includes: once built, a replacement's assignment is a membership the rule must read. *Not yet
   available*: unfrozen, case 7. *Incoherent*: a step changing a report or a membership after the
   freeze; `lifecycle::step` has none, so one listed as applied fails the replay and makes the
   prefix incoherent (*Preconditions and checks*); a lifecycle adding one would need this contract
   revised.
8. **Selection** (`Selection`: `Undrawn`, `NotSelected`, `Selected` pending or terminal; `S_j`,
   `π_j`). *Meaning*: as in §14.9, `π_j` the inclusion probability the design records,
   `F_Φ`-measurable (§8.1). Under A, `Selected` at `π_j = 1` for each reviewed item of a prescribed
   group, from its group record, which precedes the item's assignments (§11.1, item 1); under C,
   `Undrawn` until the audit draw's record after `Φ_j`, then `Selected` or `NotSelected` at `α`.
   *Missing*: any group record, A's pilot path and any audit draw. Today's exploration
   (`Event::Explore`; `exploration::{explore_from_beacon, outcome_of}`) is no source: its draw is
   computable before the reports (`16` §1), no `ε` is recorded (the event carries a boolean) and its
   outcomes are binary. Proposed: a design record per item naming its design and `π_j`, placed
   before the item's first assignment under either shape, so that the item's validity is settled
   there (*Prefixes and updates*), and under C a draw record with its round rule (`16` §5), placed
   after `Φ_j`. Checked: those placements, `π_j ∈ (0, 1]`, no non-selection at 1, one design record
   and one draw per item; a record failing one is refused, the item keeping its state (*Records and
   states*) — after a draw refused before `Φ_j`, unfrozen, then `Undrawn` once frozen. *If absent*:
   an assigned item with no design record placed before its first assignment lies outside the
   design, so it is invalid, never `Undrawn` or `Selected` by default; under C before the draw,
   `Undrawn`, a legitimate pending state; under A no item is `Undrawn`. A record states `π_j` and
   certifies no H-e (§14.9); a draw refused before `Φ_j` is listed, not judged against H-e, and
   its refusal does not show it harmless to H-e.
9. **Terminal outcome** (`Selected { outcome }`). *Meaning*: `A`, `R` or `I` from a terminal record
   of the item's group procedure, validated as §9.2 requires and applied after `Φ_j`; `None` while
   pending; `I` only from §9.2's positive records. *Missing*: attempt and terminal records and any
   `I` state; the pilot steps (`Pilot1Batch`, `Pilot2Batch`) carry readings their caller supplies,
   lead to binary states and record no attempt (§11.4); `20`'s group, attempt and terminal records
   and its `study::validate` are test code. Proposed: a terminal record carrying §9.2's items 1–6,
   and a verifier recomputing the attempts from their evidence under the pinned procedure — a
   separate component, missing, which needs the group record. The caller checks the links (the item
   in the group, the rule the group record's), the order (after `Φ_j`, as the scorer requires,
   `InputError::OutcomeBeforeFreeze`; after a start rule once one is adopted, T1 not being) and the
   verifier's verdict. *If absent*: pending, case 2, never 0 and never `I` (§9.1); a shortfall
   record leaves it pending (§9.2). *Refused*: a terminal failing its links, its order or its
   verification, an `I` without §9.2's records, a second terminal for one item: named, it leaves the
   item in the state the prefix gave it before that record (*Records and states*), as
   `Ledger::apply` treats a refused entry — unfrozen, case 7, before `Φ_j`; pending, case 2, only
   where the item was frozen, selected and pending; the earlier valid terminal's outcome after one;
   `Undrawn` or not selected where it was so.

**The component (Proposed): a snapshot of the scorer's inputs.** A pure, deterministic function,
library code with no production caller, like the scorer.

- *Input*, three parts kept apart, and the cohort rule `K(u, e)`:
  1. *The ledger's applied prefix* (Exists; validated upstream). Every cut `0..=n` the ledger
     applied, whether or not it applied a `NodeEvent`, with its metadata: `number`, `epoch`,
     `closes`, and its digest (`Cut::digest`) as supplied, which names the prefix and enters no
     computation. The digest identifies the ledger's prefix: it authenticates neither the proposed
     records, nor their slots, nor other data added to the input, and does not identify the whole
     experimental input (Astra's precision on `8dc7087`). Per cut, the entries it applied, in their
     order (`CutReport::applied`, identifiers only: `EntryId`, its writer, sequence and hash), each
     with its object as the replica holds it (`Replica::get`), decoded as a `NodeEvent` or a member
     object (`MemberObject`: a cut signature, a beacon commit or reveal, read for nothing). An
     applied entry's position is its cut's number and its slot rank (part 3), its rank in
     `CutReport::applied` where no proposed record shares its cut; an item's first assignment, `Φ_j`
     and an assignment's epoch are read there.
  2. *The ledger's refusals, as evidence* (Exists at application only; optional). Per cut, the
     entries refused with their reasons, as `CutReport::refused` returned them; `Ledger::refused`
     keeps the identifiers without reasons. Supplied, never derived: `CutReport::applied` names the
     entries applied and nothing else, and which other entries a cut named takes the replica and
     `cut::added`, which the component does not read; `CutReport` keeps no order between an applied
     and a refused entry, and none is needed. A refusal left the state as it was (`Ledger::apply`):
     the component changes nothing for it and lists it.
  3. *Proposed records* (Proposed: experimental, with no producer). Weights, design and draw
     records, terminal records with their verifier's verdicts, a cohort record where a study
     supplies one: in-crate types of the component marked as proposals, not `NodeEvent` variants,
     never accepted by the ledger — written to a feed, an object decoding as neither a `NodeEvent`
     nor a member object is refused by `Ledger::apply` (`Refusal::NotAnEvent`). *Identity*: the
     `Cid` of the record's canonical encoding (`network::cid::cid`), two copies sharing it.
     *Placement*: per cut, the input lists the cut's slots in application order, each an applied
     entry of part 1, in `CutReport::applied`'s order, or a proposed record, interleaved where the
     input places it. A position is `(cut number, slot rank)`, a total order over the prefix's
     applied entries and proposed records, on which "before the item's first assignment", "before
     its epoch's first assignment" and "after `Φ_j`" are read. The placement states where the record
     would apply: it is no cut, signature or replica entry, it dates no production, holding or
     disclosure (§13.1), and no production record exists to compare it with. *Inclusion*: a record
     belongs to the prefix through `n` if it is placed in a cut up to `n`; nothing placed later is
     read. *Duplicates and conflicts*: a record whose identity is already placed earlier in the
     prefix is refused as a duplicate; one on the subject of an earlier accepted record — a second
     weights record for an epoch, a second design record or draw for an item, a second terminal —
     is refused as a conflict; in both the earlier record stands.
- *Preconditions and checks.* Guaranteed upstream, assumed and not checked again (the ledger's and
  the replica's, tested there): each cut signed by `t` members (`LedgerError::Unsigned`); the
  entries' order within a cut (`cut::added`); the writers' signatures and the CIDs
  (`Replica::check`); orchestration only from consortium members (`Refusal::NotAuthorized`); a
  deposit's epoch, the nullifier proofs, the quotas and admissions (`NodeState::apply`); every
  object held. The component reads no signature, key, mark or replica. Checked by the snapshot on
  its input, a failure making the prefix incoherent: cut numbers `0..=n` in turn; epochs
  non-decreasing and no cut of `e` after one closing `e` (the metadata part of `cut::added`'s rule);
  each `EntryId` once in the prefix, applied or refused, never both; each item started by one
  applied `Deposit`; each applied `Step` passing the checks `NodeState::step` makes on the state the
  prefix's earlier steps built — the item deposited, an assignment naming it, `lifecycle::step`
  accepting — a replay the component needs anyway for `Φ_j` (item 7) and the reports (item 5). A
  component test supplying an entry among the refusals, or leaving it out, checks the component's
  reading of that input, never the ledger's authentication, which the ledger's own tests check
  (`tests/ledger.rs`, `at_pro_13_orchestration_only_from_the_consortium`).
- *Output*: the snapshot at `n`, named by `n` and cut `n`'s digest. The register of assignments:
  every assignment of the prefix in position order, with its nym, its item's `Cid` and register
  index, first panel or extra round, position, cut epoch, cohort `(u, e)` and its item's validity;
  per reviewer, `|R_u|`. The scorer's `Item`s, the valid items only, in register order, with the
  table linking each scorer index to its `Cid` and register index, through which every index
  `score` reads (`Cohort::items`) or returns (`Assignment::item`) is mapped. The cohorts: closed
  and valid, passed to `score` by scorer index; open, with their members so far; invalid, holding
  an invalid item, with every member by register index and `|K|`, neither scored nor shrunk. The
  invalid items and the refused proposed records, each with its reason and position; the ledger's
  refusals as supplied. `score`'s result on the valid items and cohorts, its counts the valid
  subset's (item 3).
- *Responsibilities*: the ledger's, the preconditions above; the component's, the checks above,
  the judgment of every proposed record, the mapping to §14.9 and §14.9's list left to the caller
  as far as records allow, each gap above named in its documentation, never filled; nobody's today,
  producing the proposed records and verifying a terminal's evidence (item 9's verifier, missing).

**Records and states.** One rule for every refusal: a refused record keeps the previous state. A
proposed record the component refuses leaves the state the prefix built before its position; the
component judges each proposed record once, on that state, and nothing later in the prefix judges
it again. An entry the ledger refused left its state unchanged (`Ledger::apply`); the component,
which gives it no slot, changes nothing for it. A refusal is a correct reading of the input and
leaves the snapshot valid; an incoherent prefix is an input outside the preconditions and leaves
no snapshot. Hence a second terminal refused keeps the first valid one; a
terminal refused before `Φ_j` leaves the assignment unfrozen, case 7, pending (case 2) only once a
later step freezes the item with no valid terminal; an invalid terminal leaves an item pending only
where pending was its state before.

| Level | Examples | Effect |
|---|---|---|
| legitimate incomplete state | a report `Open`; an item unfrozen; `Undrawn`; an outcome pending; a cohort open | passed to the scorer as such, or the cohort reported open; the scorer's `Unavailable` and `Bound` read them |
| refused record | a terminal failing its links, order or verification, or a second one; a design record with `π_j` outside `(0, 1]`, or placed after its item's first assignment; a draw before `Φ_j`; a weights record placed after its epoch's first assignment; a duplicate or a conflict; a ledger refusal supplied | named with its reason and position; the previous state kept; the snapshot valid |
| invalid item | an assigned item with no design record placed before its first assignment; its epoch without a weights record before that assignment, or a first panelist without a weight in it | settled at the item's first assignment; named, not passed to the scorer, its assignments kept in the register; every cohort holding it reported invalid with all its members and `\|K\|`, neither scored nor shrunk |
| incoherent prefix | an input failing a check above: cut metadata against `cut::added`'s rule; an entry twice, or both applied and refused; a step the lifecycle refuses listed as applied | no snapshot; the first failure named |

**With the scorer.** An `InputError` from `score` on a snapshot's inputs is a defect of the
component, which checks every such condition first; `Unavailable` and `Bound` are legitimate
readings passed through; `OutOfRange` stays a limit of `f64` (§14.9), not a validation error.

**Prefixes and updates.** The component reads no earlier output and keeps nothing between calls:
the snapshot at `n` is a function of the input through cut `n`, reported with `n` and cut `n`'s
digest (§9.5's snapshot, not the state), and none is rewritten. Progression is therefore no check
against an earlier snapshot, which the input does not carry, but invariants of the function,
reconstructible from the prefix: for `n < n'` on one chain the input through `n` is the
truncation of the input through `n'`, and its snapshot can be recomputed from it. They hold for
truncations of one complete input, its proposed records and their placement in the earlier cuts
fixed; two inputs with equal digests and other records or slots are two inputs, each read on its
own data (Astra's precision on `8dc7087`). Each record being judged once on the state before it:

- the register at `n` is a prefix of the register at `n'`, items, register indices and positions
  unchanged, and the valid items at `n` are the first valid items at `n'`, scorer indices unchanged;
- an assignment's case only moves forward — `Open` to `Revealed`, unfrozen to frozen, `Undrawn` to
  a selection, pending to terminal — and a terminal stays;
- an item's validity, settled at its first assignment, never changes;
- an open cohort retains every member until it closes; a new extra assignment can add an item
  with an earlier register index, so the ordered list need not extend by prefix. After closure
  membership and item register order stay unchanged; a cohort holding an invalid item stays
  invalid;
- a refused record stays refused, for the same reason; the ledger's refusals stay as supplied.

These are properties of the component, checked by its tests on truncated prefixes; a violation is a
defect of the component, never a snapshot invalidated by comparison. That two inputs lie on one
chain is the ledger's precondition (`cut::added`, the cuts' signatures), not a check of the
component. A late record changes later snapshots only; no value bridges them, and a value can move
either way — `Bound` to `Unavailable` where a verdict meets no baseline.

**Implementable now, and the decisions it needs.**

| Part | Without O1–O3 | Production also needs |
|---|---|---|
| assignments, the register, `R_u`, counts; reports `Revealed` and `Open`; `Φ_j`; cohort membership and closure | yes, on applied `NodeEvent`s and every cut's metadata (Exists), with the Proposed cohort and freeze rules | their adoption beyond the isolated component, for which alone their direction was accepted on `3e34f6f` |
| frozen weights | yes, on a Proposed weights record | a protocol decision, not O1–O3: the record, its log position, the founder set's record |
| selection | yes, on a Proposed design record of A's or C's shape, adopting neither | O2; under A the group record and A's pilot path; under C the draw's record and source (`16` §6, T19) |
| terminal outcomes | the links and the order, on a Proposed terminal record and a verifier's verdict | the group record, attempt and terminal records with `I`, the verifier; whether a shortfall counts as `I` (§9.2) |
| case 6, `Missing` | no: without a close the component passes `Open` | a reveal close: T58's deadline or a deadline as a cut (T79); no replacement policy assumed |
| authorship of reports | no | T79's channel from a person to a member, linked to the assignment |
| panel against the draw | no | the eligible set, positions, clusters and slot recorded; T79's check before co-signing |
| the score's use | — | outside: O1, the target; O3, the reputation; nothing reaches `SkillTrack` |

No T58 replacement, selection fallback, T1 adoption or reputational rule is assumed. T79 enters only
through the three capabilities named — a person's commit and reveal reaching a member, deadlines as
cuts, the check before co-signing; T76, T77's mechanism and T78 bear on liveness and
confidentiality, not on these obligations.

**Acceptance criteria, before code.** Each checks the component's reading of its input and the
mapping to §14.9, never H-a–H-e and never what the ledger checks upstream. Cuts are of epoch 0
unless stated.

| Case | Expected | Boundary of the check |
|---|---|---|
| valid path: two items, one through the band; a weights record; design records of A's shape; verified terminal records `A` and `R`; the cohort closed | `score` on the snapshot equals `score` on the same `Item`s built by hand, `Final`; the register's `\|R_u\|` equals `Scores::counts`' `N`, every item valid | the mapping and the checks; no H-b, H-c, H-e |
| the ledger's boundary: a step from outside the consortium and a reveal not opening its commitment supplied among the ledger's refusals, not applied; a terminal naming another group's rule among the proposed records, on a pending item | the snapshot, apart from listing the two refusals, equals that of the same input without them; the report `Open`; the terminal refused and named, the item pending, its previous state | the component's reading of supplied refusals; the authorization and the opening are the ledger's and the lifecycle's, tested there (`tests/ledger.rs`; `Invalid::RevealMismatch`); a commitment's authorship stays uncheckable, stated and not tested |
| incoherent prefix: that reveal listed as applied; an `EntryId` both applied and refused; a cut of epoch 0 after one closing epoch 0 | no snapshot, the first failure named | what the input shows; no signature, writer or replica read |
| record absent: no terminal; no design record for an assigned item; no weights for the epoch | pending, case 2, `Bound`; the item and its cohorts invalid; that epoch's items invalid | absence named, never imputed |
| order: a design record placed after the item's first assignment; under C, a draw before `Φ_j`; a terminal before `Φ_j`, `Φ_j` not reached; that prefix extended past `Φ_j` | the record refused, the item invalid for want of a design record at its assignment; the draw refused, the item unfrozen, `Undrawn` once frozen; the terminal refused, the assignment unfrozen, case 7, not pending; frozen and pending, case 2, the refusal kept | log order only, not production or disclosure times (§13.1) |
| selection not yet recorded: C's shape before the draw | `Undrawn`; the scorer's `Unavailable` with `Cause::Undrawn` | the state, not the draw's source |
| terminal not valid: `I` without §9.2's records, or a verifier's failure, on a pending item; a different terminal after a valid `A`; a copy of that `A` | refused and named, the item pending; refused as a conflict, `A` kept; refused as a duplicate, `A` kept | the verdict is an input; the recomputation is the verifier's own test |
| a closing cut with no new event: cut 0, not closing, applies `u`'s assignments; cut 1, with `closes`, applies no `NodeEvent` — no entry, a cut signature, or only refused entries | `K(u, 0)` open at 0; closed at 1 and scored, its items being valid | the closure reads the cuts' metadata |
| a later epoch with no new event: the same cut 0; cut 1, of epoch 1, not closing, applies no `NodeEvent` | `K(u, 0)` open at 0; closed at 1 and scored, its items being valid | the same |
| the same events, other metadata: two inputs applying the same entries in the same cuts, cut 1 without `closes` in one and with it in the other; a report `Open` | `K(u, 0)` open in the first, closed in the second; the report `Open` in both | the cohort's closure, not a reveal close, which stays missing |
| cohort altered: a cohort record differing from the rule; outcomes or arrival order changed | the record refused, the rule's membership kept; the same membership | the rule reads assignments only; D1 is not tested |
| valid and invalid assignments of one reviewer: `u` on `j1` (A's shape, terminal `A`), `j2` (A's shape, pending), `j3` (no design record) in epoch 0, on `j4` (A's shape, terminal `I`) in epoch 1; weights for both epochs; both closed | the register: `\|R_u\| = 4`, `j3`'s assignment listed with its reason, no `O` or `V` for it; `score` on `j1`, `j2`, `j4`, scorer indices 0, 1, 2 linked to register indices 0, 1, 3; `Scores::counts(u)`, `N = 3`, `O = 2`, `V = 1`, reported as the valid subset's; `K(u, 0)` holding `j1`, `j2`, `j3`, `\|K\| = 3`, invalid, no value or bound; `K(u, 1)` holding `j4`, `Final(0)` | the counts as reported; no outcome counted for an invalid item; the denominator kept |
| no reveal close: a panelist unrevealed at every prefix | `Open`, case 7 for the item's reviewers, never `Missing`, its cohort closed or not | no time or silence read |
| progression: each case's input truncated after each of its cuts | each truncation's snapshot satisfies *Prefixes and updates*' invariants against the whole input's | a property of the function on one chain; the chain is the ledger's |

Wherever a snapshot exists `score` returns no `InputError`, and a snapshot recomputed on the same
prefix is identical bit for bit.

**Next deliverable.** The snapshot component, as library code with no production caller: it reads
real applied `NodeEvent`s with every cut's metadata, the ledger's refusals where supplied, and the
Proposed records as in-crate types marked as proposals, with the criteria above as its tests;
`cohort_scores`, `lifecycle`, `ledger` and `20`'s check unchanged. This section was approved on
`8dc7087`; the component is implemented in §14.11, partly reviewed on `9c7cb23`, its correction
awaiting review. Of the four choices it proposes,
none of O1–O3, three had their direction accepted on `3e34f6f` for the isolated component — the
cohort rule `K(u, e)`, an epoch's weights recorded before its assignments, `Φ_j` from today's
lifecycle — showing no invariance of the assignments and adopting no protocol; the fourth, the
levels of refusal and invalidity, rectified above (*Records and states*), was approved with the rest
of this section on `8dc7087`. It does not wait for T58, T79, T19 or a design: their absence leaves
reports `Open`, items `Undrawn` or invalid and outcomes pending, each named. Its output is no
production evidence while the Proposed records have no producer.

### 14.11 The snapshot component (partly reviewed on `9c7cb23`; correction awaiting review)

**Status.** Claude Code's implementation of §14.10, on the owner's assignment after Astra's
approval of `8dc7087`: `protocol::cohort_snapshot` (`crates/protocol/src/cohort_snapshot.rs`), a
pure, deterministic library component with **no production caller**. **Partly reviewed by Astra
on `9c7cb23`, not approved for the two findings recorded in §14; corrected below, awaiting
review**. The approval of §14.10 covers the specification, not this code. Its proposed records are
experimental types with no producer: no output of it is production evidence. It adopts no A, C
or T1, decides none of O1–O3, assumes no T58 policy or selection fallback, and its output reaches
no `SkillTrack`, vote weight, probation, cap or CUSUM. `cohort_scores`, `panel_scores`,
`lifecycle`, `ledger`, `NodeEvent` and `20`'s check are unchanged.

**API.** `snapshot(&[CutInput]) -> Result<Snapshot, Error>`; the snapshot at cut `n` is
`snapshot(&cuts[..=n])` on one complete input.

- `CutInput`: `number`, `epoch`, `closes`, `digest` (the ledger's prefix only, §14.10), `slots`
  and `refused` (`(EntryId, Refusal)`, as `CutReport::refused` returned them, possibly none).
- `Slot::Applied { id, object }`, an applied entry with its object's bytes as the replica holds
  them, in `CutReport::applied`'s order; `Slot::Proposed(Proposed)`, placed by the input.
- `Proposed`, separate from `NodeEvent`: `Weights { epoch, weights }` (a map from nym to weight);
  `Design { item, group, design, rule }` (`Design::A`, or `Design::C { inclusion }`, `group` and
  `rule` the declared association and procedure); `Draw { item, round, selected }`;
  `Terminal { item, group, rule, outcome, references, verified }`, the references and the verdict
  the verifier's, supplied; `Cohort { nym, epoch,
  items }` (a set). `encode` is canonical — a leading tag `0xE0`, neither `NodeEvent`'s version 1
  nor the member objects' `0xC0`, fixed-width fields, counted lists, maps and sets in key order —
  and `id` its CID, the identity duplicates are judged by.
- `Snapshot`: `cut` and `digest`; `items`, the register of items (`Cid`, first assignment's
  position and epoch, `Φ_j`'s position, `status`: the scorer index or every `Invalidity`);
  `register`, every assignment (nym, register index, extra round or not, position, epoch);
  `cohorts`, each `K(u, e)` (members by register index, closed or not, invalid members, its index
  among the scored cohorts); `records`, each proposed record's position, identity and refusal if
  any; `refusals`, the supplied ledger refusals by cut; `scored`, the register index of each scorer
  index; `inputs` and `scorer_cohorts`, what `score` received; `scores`. `Snapshot::counts(u)`
  gives `|R_u|`, the assignments on invalid items and `Scores::counts` over the valid items,
  labelled apart.
- `Error::Incoherent(Incoherence)`, an input breaking a checked precondition, no snapshot;
  `Error::Defect(InputError)`, `score` refusing what the snapshot gave it, a defect of this
  component.

**Concrete choices**, where §14.10 leaves a form open.

1. An applied object is decoded as `Ledger::apply_one` decodes it, a member object first, then a
   `NodeEvent`; one decoding as neither makes the prefix incoherent, the ledger having refused it
   (`NotAnEvent`). Member objects and `NodeEvent`s other than deposits and steps are read for
   nothing.
2. The replay calls `lifecycle::step` itself, with `NodeState::step`'s two checks before it (the
   item deposited, an assignment naming it) and one deposit per item; proofs, quotas, admissions,
   writers, signatures and deposit epochs stay upstream.
3. A draw record carries its `round`, recorded and unchecked, so that a later draw is a distinct
   record from a refused earlier one; the minimal form without it made every later draw of the
   same selection a duplicate.
4. A record's reason is its first failed check, in this order: an identity already placed
   (`Duplicate`); a subject already accepted (`Conflict`); then per kind — weights: placed after
   any assignment, first panel or extra round, applied by a cut of its epoch (`Late`); design: `π`
   outside `(0, 1]` (`Inclusion`), placed after the item's first assignment (`Late`); draw: no
   design (`NoDesign`), A's design (`Shape`), before `Φ_j` (`BeforeFreeze`), not selected at
   `π = 1` (`NotSelectedAtCertainty`); terminal: no design, another group (`Group`), another rule
   (`Rule`), before `Φ_j`,
   not selected or undrawn (`Unselected`), no reference (`Unreferenced`), not verified
   (`Unverified`); cohort record: its cohort open at its position (`CohortOpen`), another
   membership than the rule's (`Membership`).
5. A cohort record is judged once, at its position; §14.10 places it nowhere, and only a closed
   membership can be compared once and for all. It never changes the rule's membership.
6. A weights record with a negative or non-finite weight is accepted and the items it weighs are
   invalid (`Invalidity::Weight`), as §14.10 item 6 states; a nym cannot repeat in it, a map.
7. Cohorts are listed by epoch, then nym; members sorted by item register index before deriving
   invalid members and scorer inputs; the scored cohorts are passed to `score` in that order.
8. The ledger's refusals are output as `(cut, EntryId)`; their reasons stay in the input,
   `Refusal` having no `Clone` and `ledger` staying unchanged. To recover a reason, retain the
   input and look up that pair; a standalone snapshot does not carry the reason or establish
   that the supplied list of refusals is complete. Accepted with this limit on `9c7cb23`.

**Verification** (E, Claude Code, 2026-10-10; expected values by hand, fixed before the first run):
`crates/protocol/tests/cohort_snapshot.rs`, 17 tests. Through the real `Ledger` (real deposits
and proofs, signed cuts): the valid path, j1 decided `Pass` and j2 through the band and
`Resolve`, against `Item`s and `Cohort`s built by hand, `score` on them equal to the snapshot's,
with the register, the positions of `Φ_j` and the counts; the ledger's own refusals of a forged
reveal and of a step from outside the consortium, listed and changing nothing. On prefixes built
by hand as stand-ins for the ledger's output: ten incoherent prefixes, each with its first
failure; absent records (pending and `Bound`; `NoDesign`; `NoWeights`); late, duplicate,
conflicting and malformed records, the state kept; a weights record after an extra assignment of
its epoch; a draw before `Φ_j` refused and a later one counted, and the draw's design and
certainty rules; a premature, a second and invalid terminals, the cases moving Unfrozen, Pending,
Positive; four closings without a new event (an empty cut, a cut signature, refusals only, a later
epoch); the same entries under other closing metadata; §14.10's j1–j4 criterion (`|R_u| = 4`,
`N = 3`, `O = 2`, `V = 1`, `K(u, 0)` invalid with `|K| = 3`, `K(u, 1)` `Final(0)`); a report
`Open` at every prefix; equal digests with other records or slots; the cohort record; the
encoding; and §14.10's invariants on every truncation of six inputs, with recomputation identical
bit for bit. Ten temporary mutations of the component, removed: nine failed tests at once; the
tenth, an extra assignment not starting its epoch, failed none, so the weights test above was
added, and it fails there. Run with `cohort_scores`' 20 tests, `panel_scores`' 9 and the check's
39, every expected value unchanged; Clippy with `-D warnings` on those targets, `rustfmt --check`.

**Correction after the partial review of `9c7cb23`** (Codex, on the owner's assignment;
awaiting Astra's review). Only `cohort_snapshot`, its tests and pertinent documentation change:

- `Walk::finish` sorts each cohort's members by item register index before building its invalid
  list and `Cohort::items`. The assignment register keeps its chronological order. An open
  cohort's members persist as a set, with an extra assignment able to insert an earlier index;
  after closure both membership and order persist. The old test's `starts_with` for members was
  stronger than §14.10 and wrong; it is replaced by containment and strictly increasing indices,
  with exact equality after closure. `register.starts_with` remains.
- Experimental `Design` and `Terminal` carry `group: Cid`. A terminal's `item` selects its
  accepted design; its declared group and rule must both match that design, or `Group` / `Rule`
  refuses it before any terminal state changes. These are comparisons of **declared
  identifiers**, not proof that the item belongs to the referenced group, that the group pinned
  that rule, or that its procedure ran. The references' nonempty structure and the supplied
  `verified` verdict keep their prior checks; authentication, membership evidence and attempt
  recomputation remain the external verifier's responsibility. No group record producer or
  experiment verifier is built here.
- The canonical encodings of design and terminal now put the fixed-width group CID between the
  item and rule CIDs, under the same experimental tags. Their record identities therefore
  change; group changes must change bytes and CID, and a correct terminal following one refused
  for another group has a distinct identity. No production record format or compatibility
  protocol is introduced.

**Correction verification** (E, Codex, 2026-10-10; new inputs and expected values fixed before
the failing runs). Three regressions on the unchanged implementation of `9c7cb23`: j0 registered
first without u in its first panel, u assigned to j1 and j2, then to j0's extra round in the same
epoch. Membership was `[1, 2, 0]` against `[0, 1, 2]`; with j0 and j1 invalid the invalid list
was `[1, 0]` against `[0, 1]`. On valid lifecycle inputs with outcome `A`, u's `(p, b)` pairs
are `(1, 0)`, `(0, 1)`, `(1, 0)` and inclusion probabilities `2^−1000`, `2^−1000`, `1`:
terms `2^1000`, `−2^1000`, `1`. The first panel of j0 has seven unit-weight reports at 0,
u joining its extra round at 1; j1's other six reports are 1 and u's 0; j2's other six are 0
and u's 1. The prescribed order gives `known = 1`, `Final = 1/3` within `f64` rounding;
chronological membership gave `known = 0`, `Final = 0`. The test builds the scorer's inputs
independently and checks baselines, terms, membership and numeric results; the chronological
order's zero is also checked through the unchanged scorer.

The group regression requires an input the old types could not express: after adding only the
experimental field, its encoding and the refusal variant, with the old terminal judgment still
in place, a terminal with the right rule and wrong group was accepted (`None` against
`Some(Group)`). The matching-group positive test already passed. After correction all five new
tests pass: membership, the numeric scorer path, invalid-member order, the wrong-group refusal
and the matching-group acceptance. The negative test compares the entire snapshot with its
previous state after removing only the new judgment, checks a duplicate refusal, and then the
correct terminal's acceptance. Prefix checks include the extra assignment before closure and
an unchanged cohort after closure and epoch advance. Canonical bytes are checked independently
for design and terminal, group changes yielding distinct identities.

Runs before correction: `cohort_snapshot`, the 17 existing tests passed and the three order
regressions failed; the group pair, one passed and one failed as above. Runs after correction:
`cohort_snapshot` **22 passed**; `cohort_scores` **20**, `panel_scores` **9** and
`s2_synthetic_check` **39 passed**, also passing before correction. All earlier expected values
remain unchanged; the membership-prefix assertion alone is corrected as explained above, and
experimental design/terminal identities intentionally include the new field. Clippy with
`-D warnings` on the library and these four targets, `cargo fmt --all -- --check`, the repository
comment budget and `git diff --check` pass. No workspace suite, campaign or Phase 2 work run.

**Not shown.** That any production producer writes these records, or writes them in the order
the input places them; the digest's binding of anything beyond the ledger's prefix; any upstream
precondition (signatures, writers, authorization, proofs, quotas, the order within a cut); a
terminal's evidence or the truth of its declared association, verification being supplied;
H-b, H-c, H-e; a reveal close, so no `Missing`.
