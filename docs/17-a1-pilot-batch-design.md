# 17 — A1: pilot batches (C3) and missing outcomes (C2)

| | |
|---|---|
| **Status** | Analysis and proposal, **pending design review**. A1 stays open (`15`). Nothing here is implemented; no pilot, batching or missing-outcome policy changes. |
| **Baseline** | `docs/phase1-review-alignment` at `e8fdbe7`. Line references are to that commit. |
| **Scope** | Conditions C2 and C3 of [`16`](16-a1-incentive-design.md) §4.2 (this dossier's conditions, not the review findings of the same names in `15`): what outcome a reviewer's report predicts, how a pilot batch must be formed for that outcome to be common to the paths, and what to do when no outcome arrives. |
| **Evidence** | **L** read in the source; **D** derived here; **C** calculated; **E** executed as a Rust test (`crates/protocol/tests/a1_batch_composition.rs`). |

## 1. The path from the pilot to the scored outcome

R = runtime the lifecycle machine runs; A = API with no production caller; I = documented
intention; — = no policy. As for A5–A7, no epoch driver composes these steps: the pilot
functions are called only by tests and by the characterization harness (`characterization/src/run.rs:145–452`), which is measurement, not runtime (L).

| # | Step | Code | Status |
|---|---|---|---|
| 1 | Entry: `Pilot1 { appealed }` after a pass, a band pass or an appeal; `Explored` after an exploration draw | `lifecycle.rs` (`Score`, `Resolve`, `Appeal`, `Explore`) | R |
| 2 | Batch formation: which items, when, how many, with which anchors and respondents | none; `05` [6]–[7] requires batches, never a single item, templates apart | — (I for the two requirements) |
| 3 | Stage-1 admission: at least `N1_MIN = 300` admitted respondents, one row each | `pilot::screen`, `pilot.rs:20,211–226` | A |
| 4 | Stage-1 fit and verdict: point-biserial `r_pbis` on the anchor total; items with `r_pbis ≥ 0.20` fitted jointly in a one-class model, shape held normal; kept iff converged, `r_pbis ≥ 0.20`, `a ≥ 0.6`, `|b| ≤ 2.5`, floor within 0.10 of chance | `stage1_fit`, `stage1_verdicts`, `pilot.rs:286–355`; cuts `irt.rs:5–12` | A |
| 5 | Stage-1 lifecycle: too few respondents is refused (`NotEnoughRespondents`, the item stays in `Pilot1`); `passed = false` gives `Rejected(Screen)` | `lifecycle.rs:487–497`; booleans from `ItemVerdicts` (`orchestrator.rs:99–111,294`) | R, caller's inputs |
| 6 | Stage-2 admission and fit: at least `K_MIN = 2` items, `N_LATENT_MIN = 3000` respondents, a format per column, no template twice, anchors' KR-20 at least 0.90; one-class fit, then mixtures by BIC, only a converged candidate replaces the one-class fit, the selected fit's status reported, classes under 5% share ignored in the gaps | `revalidation.rs:15,101–157`; `latent.rs:911–1034` (`:1022`, `:1031`, `:1034`) | A |
| 7 | Stage-2 reading: `Dif`, `NoDif`, or `Indeterminate` when the selected fit did not converge | `revalidation::target_rechecks` (`15` A4) | A |
| 8 | Stage-2 lifecycle: `batch_size < K_MIN` refused (`BatchTooSmall`); `passed` gives `ActivePool`; else `source_verified` gives `Contested`; else `Rejected(Dif)`. No input for an indeterminate verdict (`15` A4, residual (a)) | `lifecycle.rs:503–516` | R, caller's inputs |
| 9 | The explored path: the same two events, ending in `Measured { passed: passed ‖ source_verified }` | `lifecycle.rs:538–575` | R |
| 10 | Outcome for scoring: pool or contested 1, screen or DIF rejection 0, `Measured` at `π = ε` | `exploration::outcome_of`, `exploration.rs:39–58` | R |

## 2. What a verdict depends on

| Element | Stage 1 | Stage 2 | Evidence |
|---|---|---|---|
| Respondents | the sample, and its size against `N1_MIN` | the sample, and its size against `N_LATENT_MIN` | L |
| Anchors | `r_pbis` reads the anchor total; the fit uses the anchors | θ is integrated over the anchors; KR-20 gate | L |
| Other trial items | fitted jointly with the item; one convergence status for the whole fit | the mixture is fitted over all trial items; class selection and the item's gap between classes depend on them | L; E below |
| Formats and templates | each format sets a floor | the same; a shared template is refused | L |
| Convergence | batch-level: if the fit does not converge, no item is kept | batch-level: `Indeterminate` for every item | L; E below |
| Class selection | — (one class) | BIC over the batch; a class under 5% share does not define gaps | L |
| Screening | — | stage 2 runs on stage-1 survivors (`05` [7]), so its batch depends on other items' screens | I |
| Source verified | — | turns a DIF failure into `Contested` (an input; the check, T68, is not in code) | L |
| Entry path | no function reads it | no function reads it | L |

That no function reads the path does not mean the verdict does not depend on it. The path
decides whether an item is piloted and, under any batching that draws from current
entrants, with which companions. The companions change the verdict:

- **Executed comparison** (E, `a_target_s_verdict_depends_on_its_batch`, `calibration`
  feature, 69.5 s here): one target, one population of 3,000 respondents and 60 anchors, the
  target's answers identical. Through `latent_batch`'s gates, with three leaning companions
  and four clean ones the target reads `Dif`; with seven clean companions, `NoDif`. One
  realization on admissible batches: neither a frequency nor a universal statement.
- **Missing turned negative** (E, `an_unconverged_screen_composes_into_outcome_zero`): a
  stage-1 fit that did not converge keeps no item (`AT-PRO-15`), `Pilot1Batch { passed: false }`
  gives `Rejected(Screen)`, and `outcome_of` scores it as observed outcome 0. The composition is
  one the APIs allow, not one the runtime performs (no driver).

## 3. The contract

**Reference outcome.** `o_j` is the verdict of a specified two-stage procedure applied to a
batch `G_j` that contains `j`, with anchors and a respondent sample drawn by a rule fixed
independently of the reports and of the paths. A reviewer's report forecasts
`P(o_j = 1 | F_u)` for that specified procedure. Without a specified `G_j`-rule there is no
outcome to forecast: "the verdict" of a dynamic batch is a different quantity in each batch.

Four properties, kept apart:

1. **A pathwise common potential outcome** (strong C3): on every realization the verdict
   observed through any path equals `o_j`.
2. **Invariance of the joint law of baseline and outcome given `F_u`** (C6) under `u`'s
   deviations. This is what the theorem of `16` §4.3 uses for properness. (1) together with
   C5 is sufficient; (2) is what is needed.
3. **Independence of the draw from the quantity estimated**: the inclusion draw independent of
   `o_j` given the pre-draw information (the lemma of `16` §4.3).
4. **Availability after inclusion** (C2): an included item yields its outcome.

What the theorem needs is weaker than (1). Let the outcome observed on the entering path be
`O^e` and on the explored path `O^x`. Under C1, `E[Ŝ_u-term | F_u]` mixes `E[d(O^e)]` and
`E[d(O^x)]` with weights the report itself changes (D). Properness therefore needs the
conditional law of `(b_uj, O)` given `F_u`, on each path's event, to be that of one reference
pair, invariant under `u`'s deviations. Pathwise identity (1) is the simple sufficient way to
get it.

**The deviation that matters is `u`'s, all of it.** A reviewer changes several reports, on
items that may share a batch; each report can change whether its item is piloted, hence the
population of a batch that another of its items is in. C6 must hold for `u`'s joint
deviation, for every `k ∈ R_u`. Other actors' strategies are held fixed: their inputs fixed,
but their reactions to published information (an author appealing after the gate, an extra
reviewer reporting after first-panel reveals) are part of the environment's response to `u`'s
deviation. An appeal that responds to a gate outcome `u` changed can change which items are
piloted.

**Admissible dependencies.** The item's own content; respondents and anchors drawn
independently of reports and paths; companions chosen by a rule fixed before the reports and
blind to paths; the procedure's own randomness; companions' stage-1 survival when the
companions are fixed (it is part of the procedure on a fixed batch).

**Dependencies that break C3 or C6.** Companions drawn from current entrants (their entry
depends on gate decisions, hence on reports, `u`'s included); batch timing or population
that depends on paths; a stopping or retry rule that differs by path; reading a missing
outcome as 0; piloting only the selected members of a group. An outcome defined on a full
group is not observed by piloting part of it: the verdict on `G ∩ entrants` is a different
quantity from the verdict on `G`.

## 4. Alternatives

**A — fixed groups, all piloted.** Groups of `K` admitted items fixed at admission, before any
report, by a rule on the admitted set alone (templates apart). Every group is piloted in full
through both stages; every item is observed (`π = 1`); no exploration draw is needed.

- C3 by construction: the reference outcome is the verdict on the fixed group, and every member
  is measured. C6: groups, anchors and samples are independent of reports, so `(b, o)` is
  pathwise invariant under `u`'s deviations (with C5). The draw property is vacuous.
- Band and appeals change pool entry only; scoring uses every outcome.
- Cost on the paper's declared scenario (≈667 items reviewed a month, half passing the gate,
  ≈333 pilot slots a month; `065-revisions.tex`) (C): ≈667 piloted a month against ≈333 + ≈17
  now, about 317 more, ≈95% of the stated capacity. Respondents scale with batches.
- Changes: a group formation record at admission; piloting of every rejected item for
  measurement, never for pool entry; C2 still open.

**B — each target against a fixed reference set.** Each piloted item fitted with references
validated earlier and chosen independently of current admissions, one fit per target;
respondents may answer several targets and the references in one sitting.

- C3 by construction. Respondent cost can stay flat; fits multiply by the number of targets.
- **Discarded.** New items that lean together are never fitted together, so a coordinated
  campaign is read as isolated items, where detection is weak (`15` A8). The executed
  comparison shows the case: with clean companions the leaning target reads `NoDif`. That
  defeats what the batch-level latent model is for (`01` D37, `08` AT-DIF-12).

**C — group audit for scoring.** Groups fixed at admission as in A. After the reports freeze
(every report and the denominator `R_u`; the decision and the appeal are not needed), a draw
independent of the reports selects each group with probability `α`. A selected group is
piloted in full; only selected groups' outcomes are scored, at `π = α`. Pool entry keeps its
current path (gate, band, appeal, pilot) and its outcomes are not scored.

- C3 and C6 for scored outcomes by construction: fixed group, full measurement, a draw blind
  to reports and paths. The draw property holds by construction for an ideal source; the
  beacon's selection residual of `16` §6 applies unchanged.
- An item may be piloted twice: for pool entry and as a selected group's member, with two
  verdicts; only the group verdict is scored. Whether a selected group's verdict also decides
  its entering members' pool entry is a design choice that would avoid the double pilot.
- Exploration (D35) is no longer needed for scoring; the gate's false-negative rate comes from
  the selected groups' rejected members.
- Evidence and cost are those of `16` §8's audit: at `α = 0.05`, ≈17 extra pilot slots a month
  and ≈0.6 scored items per reviewer a month with golden items; matching the current scored
  rate needs `α ≈ 0.525`, ≈175 extra slots (≈47% of 333). The observed count's increment is `α`
  per judgment whatever the report (`16` §7).
- Changes: the group record at admission; a group draw after the freeze; piloting a selected
  group in full; scoring only selected groups; C2 still open.

**Dynamic batching under a probabilistic contract** (the third prompt) is not retained as an
alternative. A batch drawn from current entrants has a population whose law depends on gate
decisions, hence on reports, `u`'s included. Keeping C6 would then require the verdict to be
invariant to its companions, which the executed comparison contradicts once.

## 5. Missing outcomes (C2)

| Case | What the code does now (L) | Policy |
|---|---|---|
| batch not admissible | `PilotError` (`BatchTooSmall`, `NotEnoughRespondents`, `RowCountMismatch`, `BadFormats`, `UnreliableAnchors`, `SharedTemplate`) before any fit; lifecycle refuses `Pilot1Batch` without enough respondents and `Pilot2Batch` below `K_MIN`, leaving the item where it is | — |
| too few respondents | as above | — |
| fit not converged | stage 1: no item kept, which composes into outcome 0 (§2, E); stage 2: `Indeterminate`, with no lifecycle input | — |
| a pilot that does not conclude | the item stays in `Pilot1` or `Pilot2` | — |
| withdrawal before the outcome | no lifecycle event exists | — |

Policies, with their effects:

1. *Read as 0* — converts a missing outcome into a negative one. Excluded.
2. *Drop from numerator and denominator* — keeps the identity only if missingness is
   independent of the outcome and of the reports given the pre-draw information. Admission and
   convergence can depend on the item (a mixture fit may converge less often), and the
   denominator then varies per realization (C4). Not an automatic correction.
3. *Bounded retry*: the group, the anchors' design, formats and templates fixed; a fresh
   respondent sample; at most `k` attempts. Stopping at the first converged attempt selects
   samples. Illustration under stated assumptions, not a measurement: if a single attempt's
   verdict is 1 with probability `p` and converges with probability `c₁` when the verdict is 1 and
   `c₀` when it is 0, the first converged verdict is 1 with probability
   `p c₁ / (p c₁ + (1 − p) c₀)`. With `p = 0.3`, `c₁ = 0.6`, `c₀ = 0.9` that is `0.18/0.81 ≈ 0.2222`,
   not 0.3, and the outcome stays missing after `k` attempts with probability `0.19^k` (C).
   If the same bounded retry is applied on every path and is part of the reference procedure,
   properness does not need the single-attempt law (C3 and C6 hold for the procedure's
   verdict); the residual missingness after `k` attempts is still a C2 case. "It converges
   eventually" is no guarantee.
4. *Keep pending until an outcome exists* — no termination guarantee; a pending item is not in
   `SkillTrack`'s denominator, so the same selection question as (2) arises.

No policy is free. The missing measurement: admission and convergence rates of pilot
batches by verdict class, on the candidate model (owner-run characterization), to judge
whether missingness independent of the outcome is plausible.

## 6. Conclusion

**Diagnosis, verified.** No code forms pilot batches, and no driver composes the pilot into
the lifecycle (L). The verdict is a batch-level quantity (L). In one admissible realization a
target's verdict changes with its companions (E). The APIs allow a non-converged stage 1 to be
scored as outcome 0 (E). Under any batching from current entrants, C3 and C6 cannot be
assumed. Limits: one realization; no rates; no fit-time or respondent-supply measurement.

**Recommended contract: C, the group audit.** Groups fixed at admission, a group draw after the
reports freeze, full measurement of a selected group, the reference outcome the two-stage
verdict on that group under a bounded retry that is part of the procedure, scored at
`π = α`. The argument: no path enters the group, the sample or the procedure, so the outcome
is a pathwise common potential outcome (C3). It and the baseline (C5, `16` §4.6) are pathwise
invariant under `u`'s joint deviation, so their joint law is (C6). The draw is blind to reports
by construction (subject to the source, `16` §6). C2's residual missingness remains.

**What it costs.** Entering items' validation outcomes are no longer scored; at equal pilot
cost the evidence per reviewer falls to about a sixth, or matching it needs ≈47% more pilot
capacity, on the declared scenario. No alternative found keeps all three of: the guarantee,
the current cost, and the current batch-level model while scoring every observed outcome.
A keeps the guarantee and the evidence at about twice the pilot load; B keeps the cost and the
guarantee but loses campaign detection.

**Acceptance criteria for an implementation.**

1. The group formation rule is fixed and recorded at admission and reads no path, report or
   gate decision (a test that it is a function of the admitted set alone).
2. A selected group is piloted in full; scoring reads only selected groups' verdicts.
3. No missing outcome is ever recorded as 0 (the composition of §2 becomes impossible).
4. The retry bound and the residual-missing policy are those decided, applied alike on every
   path.
5. The executed comparison stays in the suite as evidence of the dependence it guards against.

**Questions Astra can settle in design.** Group size and formation rule (templates apart,
`K`); whether a selected group's verdict also decides its entering members' pool entry; the
retry bound; whether exploration is retired for scoring; the stage-1 composition into outcome 0
(an input or state for "no verdict").

**Trade-offs that need the owner.**

1. Evidence against pilot capacity: C at a small `α` (less evidence, current cost) or A (full
   evidence, about twice the pilot load).
2. Stop scoring entering items' validation outcomes, which C requires.
3. For residual missing outcomes: accept an independence assumption, or measure first.
