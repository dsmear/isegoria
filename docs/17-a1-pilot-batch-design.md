# 17 — A1: pilot batches (C3) and missing outcomes (C2)

| | |
|---|---|
| **Status** | Analysis. Astra's review of `164fff6` did not approve it; on `37addca` Astra approved the revised mathematical argument and the comparison of alternatives **as a conditional analysis** (below). **No batching policy is approved for implementation**; A1 stays open (`15`). Nothing here is implemented; no pilot, batching or missing-outcome policy changes; no audit or new batching is approved for implementation. §7 checks a candidate of Astra's (a three-category reference outcome): on `654dbff` Astra approved its derivations and calculations as conditional results; on `825cee3` Astra accepted §7 **as a conditional analysis, not as a protocol approved for implementation**. §8 specifies B-b as a candidate contract: on `44f0dbd` Astra approved its identities and distinctions as conditional results, **not §8 as a whole, the protocol, the implementation or the adoption of A**; §8 was rectified after that review in `fa11791`, and on `fa11791` Astra approved §8 **as a conditional analysis, with no blocking finding — not the protocol, the implementation or the adoption of A**. The decision synthesis built on §§7–8 is in [`19`](19-a2-dtf-composition-design.md) §6.1. §9 specifies cases 5–7 for a study under design A, on the direction of Astra's review of `26593e7` (`19` §10): partly approved by Astra on `d31e9fa`, with 6c and 7a decided as the basis of the study's reporting, rectified in `b2a1c2f` and **approved by Astra on `b2a1c2f` as the study's reporting specification and a conditional analysis**. §10, on the bridging input under B-b, was **partly approved by Astra on `ac06d3c`** (its results on what a report carries); its rectifications **await review**. |
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
| 4 | Stage-1 fit and verdict: point-biserial `r_pbis` on the anchor total; items with `r_pbis ≥ 0.20` fitted jointly in a one-class model, shape held normal; kept iff converged, `r_pbis ≥ 0.20`, `a ≥ 0.6`, `|b| ≤ 2.5`, floor within 0.10 of chance; at `164fff6` `screen` returned the verdicts without the fit's status, `false` for every item of a fit that did not converge; since the A11 correction every such item reads `Screening::Indeterminate` | `stage1_fit`, `stage1_verdicts`, `pilot.rs:274–355`; cuts `irt.rs:5–12` | A |
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

## 10. The bridging input under B-b (partly approved; rectifications awaiting review)

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
entry, implementation or start of the study approved. Rectified after that review, **awaiting
review**: the path (§10.1); the three events and the scale comparison (§10.2); the alternatives'
meaning and the reconstruction through an estimate (§10.3); time under A and the shared
parameters (§10.4); §10.5.

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

| | α. `q_c` as the input | β. `P(Y = A | F_u)` reconstructed |
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
