# 19 — A2: DTF across fits and the whole-test claim

| | |
|---|---|
| **Status** | Diagnosis and conditional proposition **approved** by Astra on `0519626` (below); **R1 implemented in `8fa07dd`, completed by `d13bf09` and approved by Astra on `d13bf09`** (below, §8). A2 stays open (`15`): R1 rectifies the claims, it does not realize the whole-test guarantee. §6.1 adds a decision synthesis, written on `fa11791`, linking A1, A2, B1–B3 and resources: static readings and checks only, not yet reviewed by Astra; it adopts and funds nothing. No formula, API, threshold, serialization, selection, golden output or historical result changes. |
| **Baseline** | `docs/phase1-review-alignment` at `16e862c`. Line references are to that commit. |
| **Scope** | `15` A2: the contested pool's cost `D(T)`, a sum of per-fit DTFs, and the guarantee the docs attach to it for a test. B1–B3 (identification, BIC) only where A2 needs them. |
| **Evidence** | **L** read; **D** proved here; **C** recalculated (`sim/dtf_composition.py`); **E** executed. E is labelled *API* (real code on hand-built curves), *fit* (a real latent fit) or *frequency* (none here). |

## Design review (Astra)

Review on `0519626`:

- Approved: the central diagnosis (the contested facts' cost does not certify the whole test's
  DTF); §3's conditional proposition — a partition of the items, a common measure and a
  representation of the groups; the per-pair maximum of integrals for mixtures constant in
  ability, the integral of the pointwise maximum in general; no need to match class labels under
  those hypotheses; R1 as the rectification of the declared guarantees.
- Not approved: R3 as a solution already able to realize the guarantee; the need for a batch per
  test form; any new calibration, selection or group policy.
- Evidence: reading of the code, the tests and the docs; a check of the proof; a run of
  `sim/dtf_composition.py` and an exact rational enumeration of the draw. Neither the Rust tests
  nor the real fit with a true DTF of 0.78196 were re-run.
- Preliminary reserve resolved: the proposition already partitions the test among the summed
  terms (`T = ⋃_F S_F`, H3).
- Corrected after the review: subadditivity is for disjoint sets; the ¾-against-1 example is an
  abstract construction; R3 stays a candidate (§5); the coverage under selection; the draw's law
  (§1); the owner trade-off withdrawn (§6); the real fit converged and is not an admitted pilot
  (§4).

Review on `d13bf09` (R1, §8), distinct from the review above:

- Approved, with no blocking finding: R1 as implemented in `8fa07dd` and completed by `d13bf09`,
  a rectification of the declared guarantees; it does not realize the whole-test DTF guarantee.
- Not approved: R3, a mandatory batch per test form, any new calibration, selection or group
  policy. A2 and Phase 1 stay open.
- Evidence: the diff of `d13bf09` and the updated documents read; no Rust test or fit run.

## 1. What the code computes (L)

- **Per fit.** `ClassCurves::of` keeps the classes with share `π_g ≥ 0.05` (`dtf.rs:158–184`,
  `dif.rs:142`) and builds, by `with_ability` (`dtf.rs:104–133`), the measure
  `μ_F = Σ_{c∈C_F} π̃_c Σ_q w_q δ_{η_c + u_q}`: every counted class's mean plus the fit's
  histogram, masses normalized (`at`, `:135–156`), so the excluded classes' mass is redistributed.
  `with_floors` (`:80–102`, behind `ClassCurves::new`) uses instead normal densities at 41 fixed
  nodes; no persisted path uses it. Curves `P_jc(θ) = c_j + (1 − c_j) σ(a_jc(θ − b_jc))` are
  evaluated for every class at every point.
- **The statistic.** `DTF_F(S) = max_{g,h∈C_F} ∫ |Σ_{j∈S} (P_jg − P_jh)(θ)| dμ_F(θ)`
  (`dtf.rs:194–223`): the unsigned expected-score gap on the number-correct scale, worst pair of
  counted classes, over the batch's estimated population. 0 with one counted class.
- **The pool.** `ContestedPool` holds contested facts grouped by the fit that last measured them
  (`contested.rs:31–36,75–105`). `dtf(T) = Σ_F DTF_F(T ∩ F)`, each term rounded up to `2⁻³²`
  (`:12–17,126–143`); `draw` visits the fits in a seeded random order and at each picks uniformly
  among its subsets that the later fits can still complete within the tolerance, by an exact
  dynamic programme (`:145–235`): every admissible selection has a positive probability, not an
  equal one, since a fit's options are not weighted by their completions. One fact from fits
  `[a]` and `[b, c]`, every cost 0: `a` 5/12, `b` and `c` 7/24 each (C, exact enumeration in the
  script; Astra's figures). `02` §B.7 and `10` T55 describe this local rule; no current contract
  promises a uniform law over the selections, so only this dossier's earlier wording was wrong.
  Only contested facts
  enter; active-pool items never do. The `2⁻³²` rounding makes the computed sum an upper bound of
  the real sum of the per-fit values; it says nothing about what those values bound.
- **Persistence.** A results record `ContestedFit` carries `π`, `η`, `a`, `b`, `c`, the histogram
  and the members; it is rebuilt with `with_ability` directly, without the 5% filter
  (`results.rs:51–61,237–254`), so the record must already hold the counted classes.
- **What runs.** `draw` and `draw_from_beacon` are called only by tests
  (`protocol/tests/contested_facts.rs`); `blueprint::assemble_test` fills quotas without any DTF
  (`blueprint.rs:87–121`); no code composes active items and contested facts into a test. The
  composition is an API, not a runtime.

**Claims attached to it at the dossier's baseline (`16e862c`).** `01` D38's implementation note:
"a test's DTF is bounded by the sum of its per-fit DTFs" (`01:782–785`); `08` DIF-011 "RESOLVED …
drawn into a test only in selections whose DTF bound … is at most `DTF_MAX`" (`08:122–131`) and
its status row (`08:1512`); `ARCHITECTURE.md:185` "DTF bound (the sum of per-fit DTFs)"; the
module doc and `dtf`'s doc (`contested.rs:1–3,126`). `02` §B.7 (`:596–600,622–643`) and `05` [7b]
(`:179–184`) already state the whole-test guarantee as open. The paper
(`065-revisions.tex:321–328`) still asserted that the whole test stays within tolerance; R1
corrects that assertion (§8).

## 2. The quantity one would want to bound

A test `T` (active items `A` and contested facts `C`) is answered by a **target population** with
ability `θ` on a **common scale** and reference measure `μ`; the **groups** are a family of
subpopulations `𝒢` (the latent classes the batches find, or any social partition); the **score**
is the number correct `S = Σ_{j∈T} X_j`, unweighted. The target is
`DTF_μ(T) = max_{g,h∈𝒢} ∫ |E[S | θ, g] − E[S | θ, h]| dμ(θ)`.
The docs leave open which population (the test's respondents or each batch's), which groups
(fitted classes are statistical components, `02` §B.3) and which scale (each fit fixes its own:
mean 0 and variance 1 for its first class's shape). The proposition below names each.

## 3. A sufficient proposition (D)

Within one fit, the per-fit statistic is subadditive over disjoint sets: for `S ∩ S' = ∅`,
pointwise `|Δ(S ∪ S')| ≤ |Δ(S)| + |Δ(S')|` on the same measure and pairs, so
`DTF_F(S ∪ S') ≤ DTF_F(S) + DTF_F(S')` (E API:
`scoring/tests/dtf.rs::the_dtf_is_bounded_and_subadditive`). Across fits:

**Proposition.** Let `T = ⋃_F S_F` be partitioned over fits `F`. Assume
- **H1 (common measure).** For each `F` an increasing affine `τ_F` with `μ_F = μ ∘ τ_F⁻¹`: the
  fit's measure, counted classes renormalized, is the target population's in `F`'s scale.
- **H2 (group representation).** For each `F`, `g` and `θ`, probability weights `λ^F_g(θ)` on
  `C_F` with `E[Σ_{j∈S_F} X_j | θ, g] = Σ_c λ^F_{gc}(θ) Σ_{j∈S_F} P^F_jc(τ_F θ)`.
- **H3 (coverage).** Every item of `T`, active or contested, belongs to some `S_F`.

Then `DTF_μ(T) ≤ Σ_F E_F(S_F)`, with the envelope `E_F(S) = ∫ max_{c,c'} |Δ_cc'(S; t)| dμ_F(t)`;
and if every `λ^F_g` is constant in `θ` (a group is a class, or a fixed mixture of classes),
`DTF_μ(T) ≤ Σ_F DTF_F(S_F)`.

*Proof.* For groups `g, h`, `E[S|θ,g] − E[S|θ,h] = Σ_F Σ_c (λ^F_gc − λ^F_hc)(θ) Σ_{j∈S_F} P^F_jc`.
Two probability vectors differ by `Σ_{c,c'} γ_cc' (e_c − e_c')` with `γ ≥ 0`, `Σγ ≤ 1` (a coupling
of their difference), so fit `F`'s term is at most `Σ γ |Δ_cc'(S_F; τ_F θ)| ≤ max_{c,c'} |Δ_cc'|` in
absolute value. Summing over `F` at the same `θ` and integrating over `μ`, H1 turns each integral
into one over `μ_F`: the envelope. With `γ` constant, `∫ Σγ|Δ_cc'| = Σγ ∫|Δ_cc'| ≤ DTF_F(S_F)`. ∎

**No label matching is needed** for this bound: each fit's maximum over its own pairs dominates
whatever pair the groups map to, so classes of different fits never have to be identified. The
price is conservatism: the worst pairs of two fits may be different contrasts. **No linking is
computed either**: the integral is invariant under a common affine change of curves and measure;
H1 and H2 only require that one `τ_F` exists for both. Anchors with class-invariant parameters
make such a `τ_F` plausible when the anchors are DIF-free; they do not establish H1, which is
about the populations, nor H2, which is about the classes.

**The envelope is needed when group composition varies with ability** (D, C). Three classes at
two equally weighted points, curves `(1, 0, ½)` and `(½, 0, 1)`: the pairs' DTFs are `¾, ½, ¾`, so
`DTF_F = ¾`; the envelope is 1; a group that is class 0 at the first point and class 2 at the
second, against class 1, has gap 1. An abstract mathematical construction — the curves' values are
given — not a call to the API nor a fit of the model.

**What the current APIs guarantee of H1–H3: none.**
- H1: each `μ_F` is one batch's estimated population; nothing makes batches share a population,
  and dropping classes under 5% changes the measure even within a fit.
- H2: classes under 5% are left out (`scoring/tests/dtf.rs::a_class_below_the_share_floor_does_not_count`),
  so a group they carry is not represented; a one-class fit represents no contrast at all.
- H3: active items are not in the cost. Passing the cut `DIF_j ≤ 1.0` is no zero contribution.
- And the estimate: the curves are fitted; T24 measured an upward bias near 0 and noise within
  0.05 of the tolerance (`13` §7.3, §8.7.3); a draw that selects sets by their fitted cost also
  selects their estimation error.

## 4. Counterexamples and evidence

| # | What it isolates | Kind | Result |
|---|---|---|---|
| 1 | the integration measure | E API, `scoring/tests/dtf_composition.rs::one_item_s_dtf_depends_on_the_population_it_is_integrated_over` | one item, `a = 1.25`, gap 1.8, two equal classes: 0.09419 at difficulty 3 (a batch for which it is hard), 0.34381 at difficulty 1 (a population two units abler) |
| 2 | the pool's sum under different measures | E API, `protocol/tests/a2_dtf_composition.rs::per_batch_dtfs_can_admit_a_test_its_population_would_refuse` | two facts, each at difficulty 4 for its own batch: cost 0.06876, drawn; leaning the same way at difficulty 2 on a common abler population: 0.41551 |
| 3 | an active item under the cut | E API, `dtf_composition.rs::an_item_under_the_flag_cut_can_exceed_the_tolerance`; C | gap 0.9 under the cut 1.0, floor 0.2: 0.1698619472 on the 41-node grid (`15` A2's figure), 0.1698619004 by continuous quadrature, over `DTF_MAX` alone |
| 4 | a zero imposed by one class | E fit, `dtf_composition.rs::a_one_class_fit_reads_zero_where_two_items_lean` | `golden.rs`'s open batch (n = 1,500, two items leaning `δ = 0.9`): the fit converges and selects one class, fitted DTF of the two items 0; their true curves 0.78196 |
| 5 | a class left out | E API, existing `dtf.rs::a_class_below_the_share_floor_does_not_count` | a class under 5% does not count and the rest is renormalized |
| 6 | max of integrals against the envelope | D, C (rationals) | `¾` against 1, §3 |

Rows 1–3 and 5 are constructions run through the real code; row 6 is an abstract construction;
row 4 is a real, converged fit with known truth, on a fixture of 1,500 respondents that does not
pass the protocol's admission gate (`N_LATENT_MIN = 3,000`): neither a frequency nor an admitted
production pilot. **No frequency is measured**. Row 2 needs H1 to fail —
batches whose populations differ by two units of ability — which is a construction, not a
measured distribution of batches. Label non-comparability is **not** a defect of its own here:
under H2 the per-fit maximum absorbs it (§3); it costs only conservatism.

## 5. Three directions

**R1 — say what is computed (rectification, no guarantee).** Call `D(T)` the *contested-facts
cost*: the sum of per-batch plug-in DTFs of a test's contested facts, each over its batch's
fitted population and counted classes, an admissibility heuristic for that subset. Remove
"bound" and "the test's DTF" from `01` D38's note, `08` DIF-011 (status no longer RESOLVED for
the test-level claim) and its row, `ARCHITECTURE.md`, and the docs of `contested.rs` (module and
`dtf`). Behaviour, records and selection unchanged.
- Guarantee: none on the test; honest description. Residues: all of H1–H3 and estimation.
- Acceptance: no active document or comment calls `D(T)` a bound on a test's DTF; the existing
  tests pass unchanged.

**R2 — co-measured contested facts (removes the cross-fit sum).** Draw a test's contested facts
only from one fit's members (or re-fit the drawn set jointly before use, as `05` [7b]'s
re-measurement already does at re-validation), so the cost is a single `DTF_F` or `E_F`.
- Guarantee: §3 with one fit, H1 reduced to "the batch's population is the target's"; H2 still
  assumed; H3 still fails (active items); estimation open.
- Data: the pool already groups members by fit; no linking. Effects: `draw` restricted to one
  fit (or a re-fit step), fewer admissible selections, possibly `NoBalancedDraw` more often;
  records unchanged. Cost: the per-fit enumeration only.
- Acceptance: a test with members of two fits is refused (or re-fitted) by a test on the API.

**R3 — a test-level fit (a candidate, not a solution).** Fit the whole assembled
test, active items and contested facts, jointly with anchors on a batch of the target population;
accept it when an upper confidence bound of `E_F(T)` (or of `DTF_F(T)` under constant-composition
groups) is within `DTF_MAX`.
- What it would give: §3 with one fit and `S_F = T` covers the items (H3) and removes the
  cross-fit sum. It does not make the estimated measure exact — sampling the target population
  estimates `μ`, with error — and H2 stays a model assumption. A confidence bound on `E_F(T)` needs
  its own construction and justification; it does not cover groups the model lacks or a
  misspecified model, and a fitted zero certifies nothing (§4, row 4).
- Coverage under selection: if forms are searched and kept by the same estimates that are then
  bounded, the kept forms are those whose estimate happened to be low, and a nominal confidence
  bound loses its coverage unless the selection is accounted for (fresh data for the check, or a
  bound valid over the search).
- Data: as sketched, one admissible batch (`N_LATENT_MIN` respondents, reliable anchors) per
  test form; a reusable common calibration is an alternative to compare, not evaluated here.
  Effects: new API (test-level fit, envelope, confidence bound), a form record,
  selection by draw-then-verify. Cost: one latent fit per form and a bootstrap or equivalent for
  the bound; respondents per form.
- Acceptance: on a constructed population with known curves, the accepted forms' true DTF is
  within the tolerance up to the stated confidence; the cases of §4 are refused.

## 6. Recommendation

**R1 now**, since no hypothesis of §3 is guaranteed and the docs still call `D(T)` a bound; it is a
rectification, not a fix of the guarantee. R3 stays a candidate whose guarantee would still rest
on representation, specification and an uncertainty bound yet to be built. R2 removes only the
cross-fit sum and leaves active items out. No cost is put to the owner yet: a reusable common
calibration and a calibration per form, with more conservative bounds and a reduced declared
guarantee as further options, must first be compared; §6.1 states what the comparison has to
cover, without carrying it out. `DTF_MAX` stays as it is; no empirical margin replaces the
missing hypotheses.

### 6.1 Decision synthesis: A1, A2, B1–B3 and resources

**Status.** Written on `fa11791`, after Astra approved `17` §8 as a conditional analysis. It
links results established elsewhere (`17` §§7–8; §§2–3 here) and repeats no proof. Evidence: the
cited sections and code symbols read, with targeted static checks; no Rust, fit, script or
campaign run. It approves, adopts and funds nothing: A1, C2, A2 and Phase 1 stay open; R1 stays
approved within its limits (§8).

**What a reviewer forecasts (A1: B-b, `17` §7.5, §§8.1–8.3).** A reference procedure is fixed per
item before the reports. Its group, the sampling of respondents and anchors, the stage-1 screen,
the model and thresholds, the attempts, the term and the treatment of the source check together
produce the outcome `Y ∈ {A, R, I}` (`17` §7.3). The reviewer forecasts `P(Y = A | Y ≠ I, F_u)`:
`A` given a conclusive outcome and the information available at the report. The binary score
acts on `A` and `R`; a verifiable terminal inconclusiveness (`17` §7.2) adds 0 at the
denominator `N_u` fixed by the assignments. A pending item is not a 0, and a missing record is
not an inconclusiveness (`17` §§8.2–8.3). The result keeps its hypotheses: the observation
identities (H-a, H-b, H-e) and the joint invariance of `(b, Y)` given `F_u` under the reviewer's
deviation over all its reports (H-c); with a zero probability of conclusion the score is flat
in the report (`17` §8.1, D1).

**What the DTF must protect (A2, §§2–3).** The target names a population, a measure of ability
on a common scale, a family of contrasts and the whole test's score, active and contested items
included (§2; H3, §3). An item's admissibility (`Y = A` in its group's fit) certifies no form's DTF
(`17` §7.6; §4, rows 1–3). Estimated classes are statistical components, not social groups by
default (`02` §B.3); common anchors alone establish neither a common population (H1) nor the
groups' representation (H2) (§3). `D(T)` stays R1's admission cost (§8).

**What the model and selection condition (B1–B3, `15`).** Identification and finite
information, the guessing floor's prior, the anchors' reference and invariance, the histogram's
penalty, misspecification, and the search and selection of classes condition the verdicts,
hence `Y`, and the DTF. A3 is closed within its scope (`15` A3); these statistical effects stay
open. Convergence and reproducibility do not show validity (§4, row 4: a converged fit reads 0
where the true curves give 0.78196, on a fixture outside the admission gate). Forms selected on
the same estimates that then bound their DTF need a treatment of coverage after selection (§5,
R3).

**Two dimensions, kept apart.**

1. *Observation for scoring: A against C (`17` §8.5).* A removes the observation draw from the
   score and the entry-dependent channels listed there; it guarantees neither the records'
   availability, nor their timing, nor conclusiveness, nor the invariance of the work actually
   done. C modulates reputational observation at `α`, but needs a valid draw after the freeze
   (C1, H0; `16` §6's residual) and the management of resources shared with pool-entry pilots.
   Reusing one pilot for audit and entry is conditional on compatible procedures (`17` §4, C),
   not presumed.
2. *DTF calibration: reusable common against per form (`17` §7.6).* Both judged on the same
   target (§2). A common calibration can amortize its cost over several forms, but needs the
   items' coverage, a domain of validity and control of selection; a calibration per form covers
   the assembled items directly, but repeats collection and fitting and keeps the problems of
   model, representation and uncertainty. R3 is not approved as a solution; a batch per form is
   not a demonstrated necessity.

**Resources, counted apart** (C, parametric, under the assumptions stated in each line). Distinct
persons, participations, trial answers, anchor answers, attempts, model searches and times are
different quantities.

- *Common:* `P` items, each answered `n_c` times, at most `m` trial items per participation: at
  least `⌈P n_c/m⌉` participations and `P n_c` trial answers, plus the anchor answers actually
  administered.
- *Per form:* each participation answers all of its form's `t_F` trial items and `a_F` anchors;
  summed over forms `F` and their attempts, the participations (`n_F` per complete
  administration) and `n_F (t_F + a_F)` answers per complete administration.
- Sums across forms, stages and periods count participations; they do not identify distinct
  persons automatically.
- Both counts include batches not admitted (for the pilot, `PilotError`), inconclusive
  attempts, retries (`17` §4.1) and the cost of the uncertainty control or of a confirmation
  sample (§5).
- Fitting counts apart the model searches, the candidate models in each (up to seven, `02`
  §B.3, `18` §2) and the seeded starts per candidate (`LatentParams::n_starts`). Latency includes
  recruitment, the freeze (`16` §5), the computation and the records' availability (`17` §8.3).
- `N1_MIN = 300` and `N_LATENT_MIN = 3,000` are floors, not power guarantees (`15` B4–B5).
- `scoring::latent` takes complete respondent × item matrices: planned missingness and links
  between blocks need their own justification (`17` §7.6; H1, §3).
- A1's pilot data are not reused for A2's calibration automatically (`17` §7.6).
- `17` §4.1's slot counts are illustrative, not validated estimates of the resources actually
  needed; they are not converted here into persons or times.

**Direction (Astra).** B-b and A stay the main analytic references, C the comparison; the two
calibrations are to be compared before any choice. No adoption or expenditure is authorized.
The incentives of the reputation actually used (`k_u`, shrinkage, cap, CUSUM; `16` §7, `17`
§8.4) stay to be assessed: the result on the score's mean (`17` §8.1, D1) does not close them.

**Decisions still missing (product and resources).** The population and contrasts to protect;
the anchors' formats and substantive reference; the tolerable errors and inconclusiveness; the
latency and resources available. No value is proposed here, and no choice between designs is
put to the owner before a sufficient comparison exists.

**Constraints.** Anonymity (`CLAUDE.md` invariant 1: no personal or group attribute enters; the
bias analysis runs on latent axes) and the recovery of contested facts (`05` [7b], `01` D38)
bound every option. The test's neutrality is not certified.

## 7. Open

The groups the guarantee is about (latent classes, social groups, both); H2's plausibility for
social groups (classes are statistical components); the uncertainty treatment of a fitted DTF and
of selection on it; whether small classes should count in a guarantee (they are excluded because
their parameters are poorly identified, `02` §B.3); privacy of contested facts (`05` [7b], `10`
T69), untouched here.

## 8. R1 as implemented (after `0519626`, approved on `d13bf09`)

`D(T)` is described as the contested facts' admission cost — the sum of each fit's estimated DTF
over its own distribution and counted classes — and nowhere as a certified bound on the whole
test's DTF: `01` D38 (dated clarification after the implementation note), `08` DIF-011 (the
classification and the selector implemented, the test-level balance open), its status row,
AT-PRO-08 and the `Contested` row of §9.1, `02` §B.7 (the cross-fit paragraph states §3's
sufficient condition; "cost" for "bound"), `10` T55 (dated clarification), `ARCHITECTURE.md`,
`contested.rs` (module, `SCALE`, `dtf`, `draw`, the table's comment), `dtf.rs` (`DTF_MAX`), and the
paper (`065-revisions.tex`: the tolerance is an aim the current cost does not guarantee).
Within-fit results are kept. No formula, API name, threshold, serialization, selection, golden
output or historical result changes. A2 stays open: the whole-test guarantee is not realized.

**Completion against published `8fa07dd` (`d13bf09`, 2026-10-03).** The follow-up
rectifies D38's choice and rationale as an aim, the current countermeasure in `06`, T55's
acceptance cell in `10`, and the open-work cell of the DIF-011 matrix in `08`. These were
remaining statements of A2's test-level guarantee, not a separate draw-law finding.
The baseline description above also distinguishes the paper's former claim from its R1
correction. No behavior or historical measurement changes. Evidence and access limits are
recorded in `phase1-handoff.md` §8. Astra approved R1 on `d13bf09` (design review above).
