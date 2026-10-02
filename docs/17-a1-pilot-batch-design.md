# 17 — A1: pilot batches (C3) and missing outcomes (C2)

| | |
|---|---|
| **Status** | Analysis. Astra's review of `164fff6` did not approve it; on `37addca` Astra approved the revised mathematical argument and the comparison of alternatives **as a conditional analysis** (below). **No batching policy is approved for implementation**; A1 stays open (`15`). Nothing here is implemented; no pilot, batching or missing-outcome policy changes; no audit or new batching is approved for implementation. |
| **Baseline** | `docs/phase1-review-alignment`; first committed at `164fff6`, revised in `37addca`. Code line references are to `164fff6`, whose code is that of `e8fdbe7`. The correction of `15` A11, implemented after `37addca` and pending design review, changes how stage 1 reads a fit that did not converge; the passages describing it say so. |
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
| 8 | Stage-2 lifecycle: `batch_size < K_MIN` refused (`BatchTooSmall`); `passed` gives `ActivePool`; else `source_verified` gives `Contested`; else `Rejected(Dif)`. No input for an indeterminate verdict (`15` A4, residual (a)) | `lifecycle.rs:501–518` | R, caller's inputs |
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
| fit not converged | stage 1: at `164fff6` no item kept, which composed into outcome 0 on both paths (§2, E-api); since the A11 correction every item indeterminate, the item pending in `Pilot1` or `Explored`, with no termination guarantee (`15` A11); stage 2: `Indeterminate`, with no lifecycle input (`15` A4, residual (a)) | — |
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
  §8.7.5). The correction (`15` A11) is implemented and awaits design review.
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
  (reuse); the retry bound and the residual-missing policy (C2); the review of A11's
  correction (`15`); whether exploration is retired for scoring.
- No decision is asked of the owner now; capacity trade-offs arise only once a design is
  chosen.

**Acceptance criteria for any implementation.**

1. The group formation rule is fixed and recorded before the reports and reads no path, report
   or gate decision (a test that it is a function of the admitted set alone).
2. An observed group is piloted in full under one procedure, whatever caused its observation;
   the recorded `π` is the inclusion probability given the freeze.
3. No missing outcome is recorded as 0 (for stage 1, `15` A11 once approved).
4. The retry bound and the residual-missing policy are those decided, applied alike on every
   path.
5. The executed comparison stays in the suite as evidence of the dependence it guards against.
