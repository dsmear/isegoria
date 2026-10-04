# 19 — A2: DTF across fits and the whole-test claim

| | |
|---|---|
| **Status** | Diagnosis and conditional proposition **approved** by Astra on `0519626` (below); **R1 implemented in `8fa07dd`, completed by `d13bf09` and approved by Astra on `d13bf09`** (below, §8). A2 stays open (`15`): R1 rectifies the claims, it does not realize the whole-test guarantee. §6.1 adds a decision synthesis, written on `fa11791`, linking A1, A2, B1–B3 and resources, **approved by Astra on `cebcb2e` as a documentation intervention** (below); no protocol, calibration, adoption or expenditure approved. §9 compares a reusable common calibration with a calibration per form under explicit assumptions, written on `cebcb2e`, rectified in `63b13fa` and **approved by Astra on `63b13fa` as a conditional comparison** (below); it chooses no design and does not close A2. Added after that review: the check sample's conditions and the cost hypotheses (§9.3), the matching lines of §9.4 and the error control over a search (§9.5), partly approved by Astra on `6681b03` and, after its rectifications in `b59fd03`, **approved by Astra on `b59fd03` as a conditional analysis** (below). §9.6, on one fixed form's representation, identification and inference, was partly approved by Astra on `907c245`, rectified in `00f8e2c` and **approved by Astra on `00f8e2c` as a conditional analysis** (below). §10, a decision synthesis toward an experimental protocol, is **a proposal awaiting review**. No formula, API, threshold, serialization, selection, golden output or historical result changes. |
| **Baseline** | `docs/phase1-review-alignment` at `16e862c`. Line references are to that commit; §9 names code by symbol, at `cebcb2e`. |
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

Review on `cebcb2e` (the decision synthesis of §6.1 and the recorded states), distinct from the
reviews above:

- Approved, with no blocking finding, as a documentation intervention. Not approved: any
  protocol, calibration, adoption or expenditure.
- Precisions, made after the review in §6.1: B-b's D1 rests on H-a–H-d, completed reports and C5
  included, while H-e is a distinct requirement of the contract, used for `17` §8.4's expected
  counts; H-a, H-b and H-e are not one set of "observation identities". The decisions on formats
  concern the items, not only the anchors.
- Evidence: the diff read and compared with the documents it cites; the parametric formulas
  checked algebraically; no Rust, fit or campaign run.

Review on `63b13fa` (§9 and the three rectifications of the verification of `a9700fd`), distinct
from the reviews above:

- Approved: the three rectifications — the confirmations' cost, the count of D3's selector, the
  load statement's conditions — and §9 as a conditional comparison of a common and a per-form
  calibration. Not approved: any calibration, protocol, implementation or expenditure, nor any
  guarantee as already realized. A2 stays open; this approval does not close it.
- Decision for the comparison: a form's own sample can be its check sample, with no second
  sample mandatory; `n''_T = 0` is a legitimate case, under conditions (§9.3, §9.5).
- Withdrawn: reading a stricter control as confirmations in both designs. A stricter control
  can need another bound or error budget without a second sample, and a second confirmation
  does not by itself settle the selection among the forms that pass it.
- Direction for the comparison, not an adopted policy: the probability of at least one false
  acceptance over a declared horizon, at a symbolic level `α_H`, is the analytic reference
  (§9.5).
- Evidence: the diff, the dossier and the relevant code passages read; exact rational
  calculations on the combinatorial counts, the confirmations and the selection; no Rust, fit or
  campaign run.

Review on `6681b03` (the deepening of §9 after the review above), distinct from it:

- Partial. Approved as conditional results: the per-form proof under conditional coverage and a
  budget along every path (§9.5, A); the result for a single common calibration with
  simultaneous coverage (§9.5, B); the distinction between statistical checks and the
  optimizer's internal operations; the general cost identity and its product form under the
  stated hypotheses (§9.3); `E[V/max(R, 1)] ≤ P(V ≥ 1)`, distinct from a guarantee conditional on
  one form's acceptance.
- Not approved: the deepening as a whole, pending two rectifications and a precision on the
  target; no calibration, implementation or realized guarantee. The approval of `63b13fa` is
  unchanged.
- Asked, and made after the review in §9.5: B's updates under choices that depend on the
  history; a mandatory confirmation (`A₁ ∩ A₂`) kept apart from a new opportunity of acceptance
  (`A₁ ∪ A₂`); `DTF_μ(T)` kept as the target, the envelope as a conservative majorant.
- Evidence: the diff and the updated passages read; exact calculations on the counterexamples,
  on the expected-share inequality and on the cost of a capped search; no Rust, fit or campaign
  run.

Review on `b59fd03` (§9.3–§9.5 after the rectifications above), distinct from it:

- Approved, with no blocking finding, as a conditional analysis: §9.3–§9.5, including the
  conditions for a form's own sample as its check sample; the cost comparison's links at one
  guarantee and precision; the common calibration's adaptive updates, conditional coverage and a
  budget along every path being a sufficient construction; the alternatives already stated
  (marginal coverages for a sequence fixed in advance, a guarantee joint over the whole
  procedure); a mandatory confirmation kept apart from a new opportunity of acceptance; the true
  DTF as the target, apart from the conservative envelope.
- Not approved: any calibration, protocol, implementation or expenditure, nor any guarantee as
  already realized. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Non-blocking precision, made in §9.3: the check needs coverage conditional on the history, as
  in §9.5; independence from the earlier choices is one way to obtain it, not an added
  requirement.
- Direction: before choosing a calibration, settle whether and under which hypotheses data can
  support an upper bound on the true DTF of one fixed form (§9.6).
- Evidence: the commit, its parent and the published HEAD checked; the diff and the relevant
  passages read; exact calculations with fractions — 1/400 and 39/400, the inclusion and the
  union on 1,771 joint laws, the `Z` counterexample, the contrasts ¾, ½, ¾ and the envelope 1, a
  construction with adaptive levels and a shared dependence. No Rust, fit or campaign run; the
  documentation checks Claude Code reported were not repeated.

Review on `907c245` (§9.6, one fixed form), distinct from the reviews above:

- Partial. Approved as conditional results: the separation of representation, identification and
  inference; the envelope as the supremum over the abstract family of admitted mixtures, at fixed
  curves and measure; the construction with uninformative anchors, within its limits; the
  rare-class argument through total variation, under uniform coverage and an addable component;
  the scheme "region with coverage, then a conservative supremum", with distinct statistical and
  computational obligations.
- Not approved: §9.6 as a whole; a share floor in the target; a restriction to minimal
  representations; a policy on coincident components; any calibration or implementation. The
  approval of `b59fd03` is unchanged; R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Asked, and made after the review in §9.6: the share filter applied per component is not
  invariant under duplicated components (Astra's construction, checked); identification kept apart
  from compatible values, valid bounds and their use; the rare-class result's hypotheses, its
  bound and its relation to a tolerance, with no deduction that small groups must be left out;
  the coordinate change without a double shift; the BIC selection's consequence tied to the
  acceptance rule; regularity without universal claims; the envelope's sharpness restricted to
  the declared family; the next step.
- Evidence: the commit, its parent and the published HEAD checked; the diff, the relevant code
  and the theoretical references read; exact calculations on duplicated components, the DTF, the
  population KR-20 and the affine change. No Rust, fit or campaign run.

Review on `00f8e2c` (§9.6's rectifications), distinct from the partial review above:

- Approved, as a conditional analysis, the rectifications and §9.6: the duplicated-component
  counterexample (one response law, a different filtered DTF), its limits and its attribution to
  A2 and B1; point identification kept apart from compatible values, valid bounds and their use;
  the rare-class argument under its stated hypotheses, with its consequence against the
  tolerance; no deduced need to leave small groups out; the coordinate change, the envelope and
  the convergence contract. The approval belongs to this review, not to the one of `907c245`.
- Not approved: a new target, threshold or policy on components, any calibration,
  implementation or realized guarantee. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Three non-blocking precisions, made after the review in §9.6: the final paragraph separates
  target and representation, statistical and computational validity, and precision, usefulness
  and cost; the BIC consequence concerns a procedure taking the supremum over a region restricted
  to the selected model and accepting on that condition alone; a gap vanishing at an atom can
  give a kink, not necessarily.
- Direction: a circumscribed, coherent, executable and evaluable experimental protocol, knowing
  which conclusions are sustainable and which stay hypotheses (§10).
- Evidence: the commit, its parent and the published HEAD checked; the diff and the updated text
  read; exact calculations with fractions on the standardization, the population KR-20
  (≈ 0.9983783235), the filtered DTF (≈ 0.2439024373 against 0) and the rare-class consequence.
  No Rust, fit or campaign run; the documentation checks Claude Code reported were not repeated.

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
cover, and §9 carries it out under explicit assumptions, without choosing a design. `DTF_MAX`
stays as it is; no empirical margin replaces the missing hypotheses.

### 6.1 Decision synthesis: A1, A2, B1–B3 and resources

**Status.** Written on `fa11791`, after Astra approved `17` §8 as a conditional analysis. It
links results established elsewhere (`17` §§7–8; §§2–3 here) and repeats no proof. Evidence: the
cited sections and code symbols read, with targeted static checks; no Rust, fit, script or
campaign run. It approves, adopts and funds nothing: A1, C2, A2 and Phase 1 stay open; R1 stays
approved within its limits (§8). Approved by Astra on `cebcb2e` as a documentation intervention
(design review above); that review's two precisions are made below.

**What a reviewer forecasts (A1: B-b, `17` §7.5, §§8.1–8.3).** A reference procedure is fixed per
item before the reports. Its group, the sampling of respondents and anchors, the stage-1 screen,
the model and thresholds, the attempts, the term and the treatment of the source check together
produce the outcome `Y ∈ {A, R, I}` (`17` §7.3). The reviewer forecasts `P(Y = A | Y ≠ I, F_u)`:
`A` given a conclusive outcome and the information available at the report. The binary score
acts on `A` and `R`; a verifiable terminal inconclusiveness (`17` §7.2) adds 0 at the
denominator `N_u` fixed by the assignments. A pending item is not a 0, and a missing record is
not an inconclusiveness (`17` §§8.2–8.3). The result, `17` §8.1's D1, rests on H-a–H-d: H0 with
`F_u ⊆ F_Φ` (H-a); the path identities on the conclusive coordinates (H-b); the joint invariance
of `(b, Y)` given `F_u` under the reviewer's deviation over all its reports (H-c); completed
reports and C5 (H-d). H-e, the recorded inclusion probability being the design's, is a distinct
requirement of the contract, used for the expected counts of `17` §8.4. With a zero probability
of conclusion the score is flat in the report (`17` §8.1, D1).

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
the item formats and the anchors' substantive reference; the tolerable errors and
inconclusiveness; the latency and resources available. No value is proposed here, and no choice
between designs is put to the owner before a sufficient comparison exists.

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

## 9. Common against per-form calibration (conditional analysis)

**Status.** Written on `cebcb2e`: a comparison under stated assumptions, not a choice. No
calibration, selector, threshold, policy or expenditure is proposed for adoption; R3 stays a
candidate (§5). Evidence: **L** the code named, **D** derived here, **C** exact calculations whose
formulas are given in place; no measurement, fit or run. It is separate from the A/C choice of
observation for scoring (`17` §8.5), and A1's pilot data are not calibration data (`17` §7.6).
Verified on `a9700fd` and rectified in three places in `63b13fa`: the confirmations' effect on
the break-even (§9.3, §9.4), the merged enumeration's count (§9.3, D3), the load statement's
conditions (§9.4). **Approved by Astra on `63b13fa` as a conditional comparison**, with those
rectifications (design review above); no calibration, protocol, implementation, expenditure or
realized guarantee approved, and A2 stays open. Added after that review, on its decision and
direction, in `6681b03`: the check sample's conditions and the cost hypotheses (§9.3), the
matching lines of §9.4, and §9.5; partly approved by Astra on `6681b03`, rectified in `b59fd03`,
and **approved by Astra on `b59fd03` as a conditional analysis** (design review above). §9.6,
added in `907c245` on that review's direction, was partly approved by Astra on `907c245`,
rectified in `00f8e2c` and **approved by Astra on `00f8e2c` as a conditional analysis**.

### 9.1 Common ground

- **Target** (§2), kept symbolic: for each form `T` used over a fixed horizon, `DTF_μ(T)` for the
  population `μ` of its use, on a common ability scale, over a contrast family `𝒢`, with the
  number-correct score over all of `T`, active and contested items. Nothing here chooses `μ`, `𝒢`
  or a promised protection.
- **Designs**, complete administrations only (every participation answers every column of its
  batch, as `revalidation::latent_batch` requires). *Common*: one fit `F₀` of a fixed bank `B` of
  `P` items with `A` anchors on `n_c` participations, a form `T ⊆ B` assessed with `F₀`'s curves
  and measure. *Per form*: for each of `F` forms, a fit `F_T` of its `t_T` items with `a_T`
  anchors on `n_T` participations.
- **Estimated statistics.** The plug-in `DTF_{F₀}(T)` or `DTF_{F_T}(T)`, both available as
  `ClassCurves::of(fit).dtf(indices)` on any subset of a fit's trial items (L; an API, composed by
  no runtime). The envelope `E_F` (§3) and any confidence bound are not implemented.
- **Representation and model.** §3 with a single fit: no cross-fit sum, no class matching. H1: the
  fit's measure, counted classes renormalized, is `μ` in the fit's scale; H2: `𝒢`'s groups are
  mixtures of the fit's counted classes on `T`'s items; H3: `T` lies in the fit. The two designs'
  fits need not find the same classes, and the bound needs neither labels aligned across fits nor
  the two designs' classes compared. Both rest on the D37/D43 model and on B1–B3 (§6.1).
- **Uncertainty and selection.** A form kept because its estimate is low carries that estimate's
  error (§5). A per-candidate error rate is not the rate among kept forms (D): if a share `s` of
  candidates exceeds `DTF_MAX`, each kept with probability `α`, and the others are kept with
  probability `γ`, the share over `DTF_MAX` among kept forms is `sα/(sα + (1 − s)γ)` — 1/11 at
  `s = ½`, `α = 1/20`, `γ = ½`; 9/19 at `s = 9/10` (C; illustrative values). The common design
  searches many forms on one set of estimates, whose errors they share; the per-form design
  verifies each candidate on its own sample. §9.5 separates the error per candidate, over the
  search and among accepted forms.

### 9.2 Pertinence to the target

| Condition | Common, for every form it serves | Per form, for each form |
|---|---|---|
| H1 | one population for every form of the horizon, sampled by `F₀`, without drift until each use | each form's sample drawn from the population of its use, near that use |
| H2 | `𝒢` represented by `F₀`'s counted classes on `T`; the classes chosen by BIC over all `P` items | `𝒢` represented by `F_T`'s counted classes, chosen over `t_T` items |
| H3 | `T ⊆ B`; a form mixing items of two fits returns to the cross-fit sum (§3) | by construction |
| Model | the model holding on all of `B` and the anchors; a misfitting column can move the shared parameters (`π`, `η`, histogram) of every form | the model holding on `T` and its anchors |
| Selection | a treatment valid over every form searched on `F₀` | a treatment valid over the forms drawn and verified |

**Not established either way** (nothing measured). More items give the class search more
information on membership and can separate classes that `t_T` items leave at one (§4, row 4: a
one-class zero on two leaning items); they also add parameters, let classes be chosen on items
outside `T` and spread one item's misfit to the shared parameters. A larger bank is not better by
default, and a fit per form is not valid by construction: it meets H2, the model and selection on
fewer items.

**Keeping the common reference valid**, without recalibrating at every change:

- *retirement*: under local independence the model of a subset of `B` is the same mixture with
  that subset's curves, so retired items leave the others' curves as fitted (D; C on a three-item,
  two-class rational model);
- *new items*: outside `B` until a fit includes them with every item they will share a form with;
  with complete administrations, a new fit of the updated bank;
- *population*: whether the forms' respondents are still `F₀`'s population; no statistic, data
  or tolerance for that check is defined, and anchors alone do not establish H1 (§3);
- *items*: drift from exposure or context (`05` [8]–[9]); the periodic latent re-check returns DIF
  readings (`revalidate_batch_latent`), not a recalibration of the bank;
- *procedure*: model version, floors' prior, shape and class selection pinned at `F₀`; a change
  requires re-examining the forms it affects (a change of `DTF_MAX` alone moves no estimate).

A form calibrated alone faces the same questions for its own reuse.

**Gates both designs must pass today** (L): `K_MIN = 2` items, `N_LATENT_MIN = 3,000` respondents
(a floor, not power: `15` B4–B5), anchors with KR-20 at least 0.90. `admit_templates` refuses two
columns of one template (`01` D43): one complete fit of `B` needs `P + A` distinct templates, and
a bank holding several variants of one template (`05` [9]) splits into fits, back to the cross-fit
sum.

### 9.3 Resources

`n_c` and `n_T` are set by precision requirements not yet defined; neither is assumed equal to the
other nor sufficient for a given precision. The designs are compared at one guarantee and one
precision: the same target (§9.1), the same error notion over the same horizon (§9.5) and the
same tolerable error; `n_c`, `n_T`, the attempts, the candidates per kept form and the re-fits
are what each design needs to reach them. A stricter control can change these counts and costs
without any second administration (§9.5).

**Simple case** (one calibration, every attempt conclusive, no form refused, no uncertainty cost):

| Quantity | Common | Per form |
|---|---|---|
| Participations | `n_c` | `Σ_T n_T` |
| Distinct persons per administration | `n_c` respondent pseudonyms, one per person (`NullifierSet`; `CLAUDE.md` invariant 5) | `n_T` per form; over the horizon between `max_T n_T` and `Σ_T n_T`, the overlap unspecified |
| Trial answers | `n_c P` | `Σ_T n_T t_T` |
| Anchor answers | `n_c A` | `Σ_T n_T a_T` |
| Largest load per participation | `P + A` | `max_T (t_T + a_T)` |
| Model searches | 1 | `F` |

A model search (L, `latent_dif_with` with its defaults, as `latent_batch` calls it): the
one-class model from one start; for each mixture order `g = 2, 3, 4`, two candidates (shared and
per-class `a`), each from `n_starts = 4` seeded starts; the orders stop at the first that adds no
converged BIC improvement. Hence 3, 5 or 7 candidates and 9, 17 or 25 optimizer runs per search,
each over the whole batch, with parameters linear in its columns. No time follows from these
counts.

**Break-even**, the common design strictly cheaper (D; C: the equivalences on 20,000 random
rational cases):

- participations: `n_c < Σ_T n_T`;
- trial answers: `n_c P < Σ_T n_T t_T`; with every `n_T = n_F`, `n_c/n_F < ρ`, where
  `ρ = Σ_T t_T/P` is the bank's mean use, forms per bank item;
- all answers, with equal `n_F`, `t`, `a`: `n_c/n_F < F(t + a)/(P + A)`; anchors are paid once
  per common participation, once per form participation otherwise;
- largest load: never, when the forms lie in `B` and `A ≥ a_T`;
- searches: fewer when `F > 1`, each on a larger batch; no statement on time.

Bank items that no form of the horizon uses cost answers and serve none.

**With attempts, refusals, updates and uncertainty** (D), each term explicit:

- *attempts*: an admission refusal or an unconverged fit repeats the administration on a fresh
  sample; `r_c` expected attempts per calibration, `r_T` per candidate (`17` §4.1's
  `(1 − (1 − κ)^k)/κ` for independent attempts, an illustration); `κ` is unmeasured and may change
  with the batch's size, and a common non-conclusion repeats all `P + A` columns at once;
- *refused forms*: per form, a candidate its check does not accept — a bound above `DTF_MAX`, or
  attempts ending in a terminal inconclusiveness — is refused after its administrations; `v_T` is
  the expected number of candidates verified per kept form. An inconclusive attempt followed by
  another counts once, in `r_T`; a candidate counts once, in `v_T`, whether kept, refused or
  terminally inconclusive. In the common design a refusal on `F₀` costs a recomputation, no
  answer;
- *updates*: `U` complete calibrations over the horizon, the `u`-th of `P_u` items on `n_{c,u}`
  participations;
- *uncertainty*: `b` re-fits per bound (each a whole search; a bootstrap is one construction),
  and expected confirmation participations per kept form, refused confirmations included: `n'_T`
  when the common design checks its forms on fresh samples, `n''_T` when a per-form design adds
  a sample to the one it verified on; each confirmation a search of its own. A form's own sample
  can be its check sample, with no second sample mandatory (Astra, review of `63b13fa`), so
  `n''_T = 0` is a legitimate case under §9.5's conditions: form, target and procedure fixed
  before the check's data are seen; a sampling that gives the check its coverage conditional on
  the history, as §9.5 states it (independence from the earlier choices is one way to obtain it,
  not an added requirement); a bound valid for the actual procedure, model search and
  indeterminate outcomes included; later attempts charged as the declared guarantee requires.
  `blueprint::assemble_test` gives none of these by itself. A stricter control may need another
  bound or error budget rather than a second sample, and a second confirmation does not by itself
  settle the selection among the forms that pass it (§9.5).

In answers, the common design costs `Σ_u r_{c,u} n_{c,u} (P_u + A) + Σ_T n'_T (t_T + a_T)` and the
per-form design `Σ_T v_T r_T n_T (t_T + a_T) + Σ_T n''_T (t_T + a_T)`; in searches,
`Σ_u (r_{c,u} + b)` and `Σ_T v_T (r_T + b)`, plus one per confirmation.

The product `v_T r_T` assumes the candidates for `T` of one size (`n_T`, `t_T`, `a_T`) and attempts
of one mean `r_T` given that a candidate is verified, whether it is verified depending only on the
earlier ones (Wald's identity). In general the answers of `T`'s search are
`Σ_i P(V ≥ i) E[R_i c_i | V ≥ i]`, with `V` the candidates verified, `R_i` the attempts and `c_i`
the answers per attempt of the `i`-th; the product follows when every `E[R_i c_i | V ≥ i]` equals
`r_T n_T (t_T + a_T)` (D; C on a small exact case). An error budget, a cap on candidates or a time
limit can end a search with fewer forms than wanted, or none: `v_T` per kept form presumes one is
kept, and otherwise the expected cost and the probability of keeping a form are stated apart.

A confirmation is an administration and a fit of the form alone, so with confirmations the
common design becomes a screen before per-form fits. With `n'_T = n''_T` the confirmation terms
cancel: the comparison reduces to the calibrations against the per-form administrations, the
break-even unchanged. With `n''_T = 0` and confirmations of a per-form administration's size,
`n'_T = v'_T r_T n_T` for `v'_T ≥ 1` candidates confirmed per kept form (under the product's
hypotheses), the common design saves answers only if
`Σ_u r_{c,u} n_{c,u} (P_u + A) < Σ_T (v_T − v'_T) r_T n_T (t_T + a_T)`: its screen must avoid
refused per-form administrations worth more than its calibrations, and with `v_T = 1` it saves
none (D; C on 20,000 random rational cases). These are costs at one guarantee only when the
confirmations and the per-form checks answer to the same error control, with the screen's
choice part of the history on which a confirmation's form is fixed (§9.5, A).

**Partial administrations** (extensions, no saving credited): with at most `m < P` bank items per
participation the bounds become at least `⌈P n_c/m⌉` participations and a load `m + A`, but
`scoring::latent` takes complete matrices (`latent_batch` refuses rows of unequal length): a
likelihood over the observed cells, or blocks linked across fits with H1 shown (§3), needs its
own justification.

**Contested selection (`15` D3).** The selector enumerates per fit the subsets of at most `n` of
that fit's contested members, `Σ_f S(M_f, n)` sets with `S(M, n) = Σ_{r ≤ min(M, n)} C(M, r)`, each
a DTF evaluation (L, `ContestedPool::candidates`). The bank's size `P` does not enter; the number
`M_f` of contested members recorded in one fit does. If the common fit is recorded in the pool
with `B`'s contested members, each moves to it (`ContestedPool::record`), and merging never
lowers the count of non-empty sets of a size: `C(M₁ + M₂, r) ≥ C(M₁, r) + C(M₂, r)` for `r ≥ 1`
(Vandermonde), so `S(M₁ + M₂, n) ≥ S(M₁, n) + S(M₂, n) − 1`, the `−1` being one fit's empty set.
The inequality is strict for `n ≥ 2` (the pool keeps no empty fit), so the total never falls
there; for `n ≤ 1` it falls by that empty set (D; C for `M₁, M₂ ≤ 30`, `n ≤ 15`).
At `n = 5`, two fits of 10 members visit 638 sets each, one of 20 visits 21,700; at `n = 10`, one
of 40 visits 1,221,246,132 (C; sizes illustrative). A wider common fit does not show the current
enumeration sustainable. A whole form can be assessed inside one fit with `ClassCurves::dtf`, but
no selector does so (`blueprint::assemble_test` fills quotas without DTF), and an enumeration in
the same style over the bank would visit `S(P, t)` sets. The selector is not changed here.

### 9.4 Conditional recommendation

| | Favours the common calibration | Favours a calibration per form | Neither yet justified |
|---|---|---|---|
| Population | one target population for every form of the horizon, stable until use | the population differs by form, or drifts between calibration and use | no target population named |
| Contrasts | — | — | `𝒢` unnamed, or its groups not representable by counted latent classes (H2) |
| Load | `P + A` within the tolerable load per participation | `P + A` beyond it | no tolerable load stated |
| Templates | `P + A` distinct templates | several variants per template in the bank | — |
| Horizon | high `ρ` and `F`, few bank updates, anchors heavy relative to `t` | `ρ` near 1, frequent turnover | `F`, `ρ`, `U` unknown |
| Uncertainty, selection | a simultaneous bound over the family searched, at the same `α_H`, without data per form (§9.5, B) | a form's own sample as its check, under §9.5's conditions and budget (A), where the common design would need confirmations | no treatment: both select on estimates; neither construction built (§9.5) |
| Model | — | — | misspecification and class search unassessed (B1–B3); a converged zero certifies nothing (§4, row 4) |
| Contested selection | few contested members per fit, or a selector that does not enumerate | — | `M_f` beyond what the enumeration sustains (D3) |

**What the comparison decides now.** With complete administrations, forms in `B` and `A ≥ a_T`,
the common design never lowers the load per participation and is excluded where `P + A` exceeds
the tolerable load; its saving in answers holds exactly under §9.3's inequalities; it needs
`P + A` distinct templates and forms inside one fit; retirement leaves the remaining curves as
fitted, while new items, population drift and procedure changes need §9.2's checks; it does not
make the contested enumeration sustainable. Neither design is valid by construction: both need
H1, H2, the model and a treatment of selection.

**No winner today.** The minimum information to cross the boundary, in order:

| Information | From | The choice it can change |
|---|---|---|
| tolerable items per participation | owner (product, resources) | below `P + A`, the complete common design is excluded today |
| whether one population serves every form of the horizon | owner (product) | if not, a common calibration per population at most, or per form |
| the construction under §9.5's reference (a budget over per-form checks, or a simultaneous bound over a family) and its proof obligations | Astra (design), then a targeted derivation | which checks need data per form; with confirmations for the common design only, it is a screen that saves answers only through the refused per-form administrations it avoids; with equal confirmations §9.3's break-even holds |
| `F`, `Σ_T t_T`, `A`, `a_T` and the expected bank changes | owner (resources), with the blueprint | which side of §9.3's break-even holds |
| `n_c/n_F` at equal precision, once the tolerable error is stated | a targeted calculation on the candidate model | needed only if the four lines above leave the break-even open |

Anonymity (no personal or group attribute enters; groups only through latent classes, H2) and
the recovery of contested facts (both designs keep them in forms, H3) bound both designs. The
test's neutrality is not certified, and R3 is not an approved solution.

### 9.5 Error control over a search (conditional analysis, approved on `b59fd03`)

**Status.** Added in `6681b03` after Astra's review of `63b13fa`, on its direction: the
probability of at least one false acceptance over a declared horizon, at a symbolic level `α_H`,
is the comparison's analytic reference, not an adopted policy. No estimator, bootstrap, level or
budget is built or proposed; the obligations of proof are listed. **D** derived here; **C** exact
calculations in a scratch script, formulas in place. Astra's review of `6681b03` approved as
conditional results A's proof, B's result for a single calibration, the distinction between
checks and the optimizer's internal operations, and the expected-share inequality as distinct
from a guarantee on one accepted form; not §9.5 as a whole. Rectified after it in `b59fd03`: the
target kept apart from the envelope, two checks of one form (A), B's adaptive updates. **Approved
by Astra on `b59fd03` as a conditional analysis**, with §9.3–§9.4 (design review above).

**Three notions.** A search verifies candidates `T_1, T_2, …` within a horizon and accepts some.
The true target is `DTF_μ(T)` (§2); a false acceptance accepts `T` with `DTF_μ(T) > DTF_MAX`;
`𝓗_{i−1}` is the history before the `i`-th verification.

1. *Per candidate*: `P(accept T_i | 𝓗_{i−1})` when `DTF_μ(T_i) > DTF_MAX`.
2. *Over the search*: the probability of at least one false acceptance within the horizon.
3. *Among accepted forms*: a metric to specify. The expected share of false acceptances among
   them, `E[V/max(R, 1)]` (`V` false, `R` all acceptances), is at most 2, since
   `V/max(R, 1) ≤ 1{V ≥ 1}` (D; C); the probability that a given accepted form is out of
   tolerance is bounded by neither 1 nor 2, as it depends on the candidates' mix (§9.1).

Controlling 2 bounds the chance that the search accepts any out-of-tolerance form; it is not a
probability of correctness of one accepted form. Astra's abstract example: every candidate out of
tolerance, false acceptances independent at 1/20, 20 attempts; 1 is 1/20, 2 is
`1 − (19/20)^20 ≈ 0.641514`, and every accepted form is out of tolerance (C; not a frequency of
Isegoria). Bringing 2 under `α_H` leaves that last fact unchanged: any form the search accepts
there is out of tolerance.

**Target and envelope.** Under §3's H1–H3, with one fit `F` holding `T`, `DTF_μ(T) ≤ E_F(T)`, the
envelope of §3's representation (its curves and measure, not their estimates), the majorant
needed when the groups' composition varies with ability. An upper bound valid for `E_F(T)` is
therefore a conservative bound for `DTF_μ(T)`, and A's and B's coverages may be proved for it.
The converse fails: `E_F(T) > DTF_MAX` does not imply `DTF_μ(T) > DTF_MAX` (§3's construction,
its three classes as the groups: target ¾, envelope 1; C). A false acceptance keeps its meaning
on `DTF_μ(T)`; refusing a form whose envelope alone exceeds the tolerance is a cost of
conservatism, not an error avoided.

**A. Per-form checks under an error budget.** At the `i`-th verification the form `T_i`, the
target, the procedure and `α_i` are functions of `𝓗_{i−1}`, fixed before its data are seen; its
procedure returns an upper bound `U_i` with `P(DTF_μ(T_i) > U_i | 𝓗_{i−1}) ≤ α_i`; `T_i` is
accepted only if `U_i ≤ DTF_MAX`; and `Σ_i α_i ≤ α_H` on every path (`α_i = 0` when no `i`-th
verification takes place). A false acceptance at `i`, `E_i`, lies in `{DTF_μ(T_i) > U_i}`, so
`P(E_i | 𝓗_{i−1}) ≤ α_i` and `P(⋃_i E_i) ≤ Σ_i E[P(E_i | 𝓗_{i−1})] ≤ E[Σ_i α_i] ≤ α_H` (D). Only
each check's coverage given the history and the union of the events enter; no independence
between checks is assumed (C: an exact case with a hidden factor shared by every check and a
budget spent adaptively stays within it).

- *Opportunities of acceptance.* An attempt that can accept a form the earlier attempts did not
  accept draws on the budget: a new sample after an inconclusive attempt, a retry of a refused
  form. Alternatively one `α_i` covers a candidate's whole procedure when its bound's coverage
  holds for that procedure, the choice of the attempt that concludes included. A check that can
  only confirm an acceptance opens no new opportunity (*Two checks of one form*, below).
- *Not checks.* The optimizer's seeded starts, the candidate models of one class search (§9.3)
  and a bootstrap's re-fits act on one sample inside one procedure: they draw nothing from the
  budget, but the bound's coverage must hold for the procedure that contains them.
- *The form's own sample as its check* (`n''_T = 0`, §9.3) needs: form, target and procedure
  fixed before the check's data are seen; a sampling that gives the conditional coverage above
  given the earlier choices (any overlap of respondents with earlier samples to be justified);
  a bound valid for the actual procedure, the class search, the share floor, the floors' prior
  and the indeterminate outcomes included; later attempts charged as above.
  `blueprint::assemble_test` draws by quotas from a seed and guarantees none of these.
- *Cost.* When `α_i` falls along the search, later checks are stricter: at a given sample fewer
  in-tolerance forms pass (for bounds nested in their level), raising `v_T`, or the sample grows;
  no second administration is involved. The budget can run out before the forms wanted are kept.
- *Screen, then check.* A common fit used only to choose candidates belongs to `𝓗_{i−1}` and
  spends no budget; the checks do (`n'_T`, §9.3).
- *Two checks of one form*, `A_k` the event that check `k` accepts it while out of tolerance. If
  acceptance requires both, the event is `A₁ ∩ A₂ ⊆ A₁`: a guarantee valid for the first bounds
  it, and a mandatory confirmation draws no further budget by itself. If either can accept, the
  second also after a refusal by the first, the event is `A₁ ∪ A₂`, with
  `P(A₁ ∪ A₂) ≤ P(A₁) + P(A₂)`: the added opportunity is covered by the budget or by a guarantee
  valid for the whole procedure. With independent errors of 1/20: 1/400 (0.25%) when both are
  required, 39/400 (9.75%) when either can accept (C; abstract). Independence gives these exact
  values only; the inclusion and the union bound hold without it. A second administration costs
  answers and a search (§9.3) even when it needs no further error budget.

**B. A common calibration with a simultaneous bound.** Before `F₀`'s data are seen, the family
`𝒯` of forms a later search may consider is declared (subsets of `B`); `F₀`'s procedure returns
bounds `U(T)`, `T ∈ 𝒯`, with `P(∃ T ∈ 𝒯: DTF_μ(T) > U(T)) ≤ α_B`. Any search, however adaptive, that
accepts only forms of `𝒯` with `U(T) ≤ DTF_MAX` makes a false acceptance with probability at most
`α_B`: when every bound holds, every accepted form is within tolerance (D). Searching again on
`F₀` spends nothing further; retirement keeps the guarantee, a bound over `𝒯` holding on each
subfamily; a change of `DTF_MAX` alone moves no `U(T)`.

- *Field of validity.* A form outside `𝒯` — a new item, a form mixing fits (§9.2), a template
  variant outside `B` — has no guarantee from `F₀`. A family widened after `F₀`'s data are seen
  needs a bound valid over the wider family or data not yet seen.
- *Updates.* For a sequence fixed in advance — families, procedures and deterministic levels
  that do not depend on what earlier calibrations showed — the marginal coverages
  `P(∃ T ∈ 𝒯_u: DTF_μ(T) > U_u(T)) ≤ α_{B,u}` with `Σ_u α_{B,u} ≤ α_H` suffice, by the union of
  the events (D). When the family, the procedure or the level depend on the history and later
  data may depend on it too, fixing them before the new data does not suffice. A sufficient
  construction: `𝓖_{u−1}` the history before calibration `u`; `𝒯_u`, the procedure and `α_{B,u}`
  fixed as functions of it; `P(∃ T ∈ 𝒯_u: DTF_μ(T) > U_u(T) | 𝓖_{u−1}) ≤ α_{B,u}`; and
  `Σ_u α_{B,u} ≤ α_H` on every path. A false acceptance through calibration `u`, `E^B_u`, lies in
  that event, so `P(⋃_u E^B_u) ≤ Σ_u E[P(E^B_u | 𝓖_{u−1})] ≤ E[Σ_u α_{B,u}] ≤ α_H` (D). A guarantee
  joint over the whole procedure is another admissible route; the conditional construction is
  not the only one. An abstract counterexample to marginal coverage under an adaptive choice
  (the reason A states its coverage given `𝓗_{i−1}`): `Z` uniform on 20 values and known from the
  history; each prefixed procedure `j` errs on `{Z = j}`, with probability 1/20; choosing `j = Z`
  from the history makes the error certain (C). It concerns the dependence between history and
  check, not a frequency of Isegoria, and does not claim that the declared conditional coverages
  hold. A change of model or procedure counts as a new calibration.
- *Precision.* A bound valid over `𝒯` is valid over each subfamily, so enlarging `𝒯` can only
  narrow the constructions available (D); the width this costs, and the `n_c` that recovers it,
  are not assessed.

**Obligations of proof, both constructions.** The coverage concerns the true `DTF_μ(T)`, not the
plug-in `DTF_F(T)`: a bound computed from a fit reaches the target only through H1 (the fit's
measure is `μ`), H2 (`𝒢` represented by the counted classes, with the envelope where their
composition varies with ability) and H3 (one fit holds `T`) (§3, §9.2), and through the model and
its class search (B1–B3); a bound on `E_F(T)` reaches it only under the same hypotheses. Neither
construction covers groups the model lacks or a misspecified model; a converged zero certifies
nothing (§4, row 4); drift between a check and the form's use lies outside both (§9.2). A and B
are compared at the same `α_H` over the same horizon, each at the counts it needs (§9.3); neither
is preferred here.

### 9.6 One fixed form (conditional analysis, approved on `00f8e2c`)

**Status.** Added in `907c245` on the direction of Astra's review of `b59fd03`: whether, and under
which hypotheses, data can support an upper bound on the true `DTF_μ(T)`, before any choice of
calibration. Reference simplification, not a protocol: one form `T` fixed before the data,
complete administrations, `μ` and `𝒢` symbolic, no search among forms, no drift between check and
use, a symbolic level `α`. H1–H3 are not assumed solved. **L** code read at `b59fd03`; **D**
derived here; **C** exact calculations with fractions in a scratch script; no run. Astra's review
of `907c245` approved as conditional results the separation of representation, identification
and inference; the envelope as the supremum over the abstract family of admitted mixtures, at
fixed curves and measure; the construction with uninformative anchors, within its limits; the
rare-class argument through total variation, under uniform coverage and an addable component;
the scheme "region with coverage, then a conservative supremum", with distinct statistical and
computational obligations. Not approved: §9.6 as a whole, a share floor in the target, a
restriction to minimal representations, a policy on coincident components, any calibration or
implementation. Rectified after it in `00f8e2c`: duplicated components, identification apart
from bounds and their use, the rare-class result's scope, the coordinate change, the BIC
selection, regularity, the envelope's sharpness, the next step. **Approved by Astra on `00f8e2c`
as a conditional analysis** (design review above); its three non-blocking precisions are made
below.

**Representation: what ties the target to the model.** Candidate hypotheses, none established:
- *R-a, specification*: the law of anchor and form responses is the D37/D43 model's for some
  parameter — at most four classes, one histogram shape shifted by `η_c`, local independence,
  class-invariant anchors, items `c_j + (1 − c_j) σ(a_jc(θ − b_jc))` with a floor shared by the
  classes (`latent.rs`: `Model`, `evaluate`, `LatentDif`).
- *R-b, matching*: equal ability in the target is the model's `θ`, aligned across classes by
  DIF-free anchors. `DTF_μ(T)` is unchanged by any increasing bijection of `θ` applied to curves
  and measure together (change of variables): the classes' alignment matters, the coordinates do
  not.
- *R-c, groups* (H2): if every group is a `θ`-dependent mixture of classes, `DTF_μ(T) ≤ E_F(T)`
  (§3). At fixed curves and measure, and when every class has mass wherever `μ` has, `E_F(T)` is the
  supremum over the abstract family of such mixtures, attained by mixtures taking at each `θ` the
  classes of largest and smallest expected score (D). That sharpness concerns the declared family
  at one representation: it is no sharp bound for actual prefixed groups, nor over every
  representation compatible with the data. The response law does not involve the mixtures `λ_g`,
  and invariant 1 excludes group labels: for groups other than the classes the data fix no `λ_g`.
- *R-d, measure* (H1): `μ` is the mixture of the classes' ability laws.
- *R-e, shares*: the code counts components of share at least 5% and renormalizes the measure over
  them (`MIN_CLASS_SHARE`, `dif.rs:142`; `ClassCurves::of`, `dtf.rs:161–185`). At the true
  parameters this is the target only if no protected group lies in an excluded component and
  their mass does not matter; and the count is per component, not per group (below).
- *R-f, overlap*: `with_ability` puts class `g`'s mass at `η_g + u_q` and `at` evaluates every
  class's curve at every point (`dtf.rs:107–157`): where a class carries no mass, its expected
  score is its parametric curve's extrapolation, not its respondents'.

**Identification, bounds and their use.** Four questions, kept apart: whether the response law
fixes the functional's value (point identification); the set of its values compatible with that
law; a valid upper bound; whether that bound is tight enough to decide. A supremum over the
compatible parameters can exist without point identification, and identification gives neither
finite-sample coverage nor a conservative computation.
- *Coordinates and labels* (D). With `θ = η + u`, the change `η → sη + t`, `u → su`, `a → a/s`,
  `b → sb + t` (`s > 0`) keeps `a(θ − b)` and the masses, so `DTF_F` and `E_F` do not depend on the
  coordinates; a permutation of class labels leaves them unchanged (maxima over unordered pairs).
  This is a change of coordinates: in the normalized model (`η_0 = 0`, a standardized shape) no
  affine freedom remains (`18` §3 (b)). A3 stays closed within its scope.
- *Duplicated components* (D, C). Without a share filter, splitting a component into identical
  copies that keep the measure leaves the functional unchanged: pairs of copies give 0, the other
  pairs the same integrand. With the filter applied to each component it does not (Astra's
  construction). Nodes `q = −20…20` with weights 1/41, standardized nodes `u_q = q/(2√35)` (the
  code's grid with uniform logits); `η = 0` throughout; open formats, so no floors; 60
  class-invariant anchors with `a = 2√35 ln 3`, `b = 0`; two items, in A with that `a` and `b = 0`,
  in B with that `a` and `b = 5/(2√35)`. At node `q` the anchors and A's items answer with
  `p_q = 3^q/(1 + 3^q)`, B's items with `r_q = 3^(q−5)/(1 + 3^(q−5))`. Representation 1: A and B at
  92% and 8%; representation 2: A, B₁, B₂ at 92%, 4%, 4%, B₁ and B₂ exact copies of B, ability law
  and item parameters included. With `R` the law B gives the whole response vector,
  `0.08 R = 0.04 R + 0.04 R` on every pattern, so the two laws are equal (the mixture identity, not
  an enumeration). The anchors' mean is ½ and, with `v = Var(p_q)`, their population KR-20 is
  `60v/(¼ + 59v) ≈ 0.9983783235`; the filtered DTF is `2 Σ_q (p_q − r_q)/41 ≈ 0.2439024373` before
  the split and 0 after it, only A being counted (C). Scope: a construction on the model, not a
  fit, an API call or a pilot run; a population KR-20 is not a sample passing the gate; anchors
  with equal curves can carry distinct templates, and `admit_templates` checks templates, not
  parameters; a gate on the responses has one distribution under both representations. The
  filtered functional is therefore not a function of the response law; it does not follow that a
  target defined on prefixed groups changed. No merging of components, minimal representation,
  minimum separation or new threshold is chosen here (A2, B1).
- *Uninformative anchors* (D). With every anchor at `a = 0`, moving one class `c` by
  `(η_c, b_jc) → (η_c + δ, b_jc + δ)` for all its items leaves every response probability unchanged
  (the anchors are constant, the items read `θ − b`), yet turns equal curves (DTF 0) into a DTF
  above 0 for every `δ ≠ 0` (items with `a > 0`). It lies in the model's parameter space but not
  in the admitted procedure: anchors independent of ability have a population KR-20 of 0, which
  `admit_anchors` refuses except by sampling chance. It shows no defect of the admitted
  procedure; it locates a hypothesis: informative class-invariant anchors fix the classes'
  alignment, which the DTF reads.
- *Not proved.* Whether, with informative anchors, the counted components' curves and the measure
  are fixed by the response law — Kruskal-type results for latent class models (Allman, Matias
  and Rhodes, 2009) concern unstructured models, and `18` §3 leaves the histogram's identification
  open; and, the filter being representation-dependent, which functional is to be identified.

**Inference.**
- *Rare classes, when every class counts* (D). Hypotheses: `U` has coverage `1 − α` uniformly over
  the family considered; the target counts every class whatever its share; the truth `P` has fewer
  classes than the family allows (four), so a class can be added within the model's restrictions
  (the common shape, class-invariant anchors, shared floors). Adding one of share `ε`,
  `Q_ε = (1 − ε)P + εR`, the laws of `n` respondents differ by at most `nε` in total variation, so
  under `P`, `P(U ≥ D(Q_ε)) ≥ 1 − α − nε`. With the added class's curves near 1 or near the floors
  and `ε → 0`, the limit of `D(Q_ε)` is at least `∫ Σ_j (1 − P_j0) dμ` or `∫ Σ_j (P_j0 − c_j) dμ`,
  whose sum is `Σ_{j∈T} (1 − c_j)`. The bound obtained: under `P`, `U ≥ ½ Σ_{j∈T} (1 − c_j)` with
  probability at least `1 − α`, whatever `P`'s DTF. Relative to a tolerance: when
  `½ Σ_{j∈T} (1 − c_j) > DTF_MAX` — for any non-empty form with floors at most ½ and the current
  `DTF_MAX` — a decision accepting when `U ≤ DTF_MAX` accepts with probability at most `α` under
  every such `P`. The result concerns that functional and that family. Leaving out components
  under a share is one possible change of target, not a demonstrated necessity, and a component's
  share is not in general a protected group's; no `s_min` is chosen.
- *Selection* (D, L). Inside the one-class model the functional's supremum is 0 (`dtf` returns 0
  with one class, `dtf.rs:197`). A procedure that sets `U` to the supremum over a region
  restricted to the BIC-selected model, and accepts on the sole condition `U ≤ DTF_MAX`, therefore
  accepts whenever a converged one-class model is selected: with a true DTF above `DTF_MAX`, a
  false acceptance is then at least as likely as that selection. The consequence concerns that
  procedure, not every procedure that selects one class; a region over every order, for one, does
  not return 0. §4, row 4 shows such a selection (converged, one class, true 0.78196; outside the
  gate, not a frequency).
- *Regularity.* The χ² calibration of likelihood ratios fails across mixture orders (an extra
  class's parameters are not identified under the smaller model). The functional's smoothness
  depends on where its kinks fall: a sign change integrated against a density can be smooth
  (`∫_{−1}^{1} |x − t| dx = 1 + t²` for `|t| < 1`); the code's measure is discrete, so a class gap
  vanishing at an atom can give a kink in the parameters, not necessarily (`|t²|` is
  differentiable at 0), and so can a maximum over pairs whose values tie with different
  derivatives. Where a kink occurs the delta method does not apply as such and a bootstrap needs
  its own justification. A split-likelihood region (Wasserman, Ramdas and
  Balakrishnan, 2020) has finite-sample coverage under R-a for independent respondents and any
  estimator fitted on the other half, the penalized fit included: a candidate for obligation (i)
  below, its width not assessed.
- *Penalties* (L, `18` §4). The moment penalty's weight grows with `n`, so the fit tends to a
  penalized pseudo-true parameter, which is the truth only if its grid moments are 0 and 1; the
  floors' priors keep a fixed weight, `FLOOR_PRIOR_WEIGHT = 20`. Coverage is to be argued on the
  likelihood, or the gap bounded.

**The procedure, element by element** (L, at `b59fd03`).

| Element | Code | Bears on |
|---|---|---|
| order search, BIC | `latent_dif_with` (`latent.rs:913–1031`): orders 1–4, shared or per-class `a`, stop at the first order that lowers nothing, a candidate replaces the best only if converged | inference (post-selection); representation (at most four classes) |
| regularization, priors | `penalty` (`:816–826`), `moment_penalty` (`:830–844`) | inference (the estimator's limit); representation (the family favoured) |
| 5% floor, renormalization | `MIN_CLASS_SHARE`; `latent.rs:1036` for `DIF_j`; `ClassCurves::of` for the DTF | target (which components count, per component); identification (not a function of the response law); inference (a jump at the floor) |
| measure and curves | `Grid::shape`; `ClassCurves::{with_ability, at}` | representation (H1, overlap); identification (`18` §3); inference (estimation error) |
| convergence, indeterminate outcomes | `fit_from` (`:848–877`: L-BFGS, `G_TOL`, `MAX_ITERS`, a status); four seeded starts; `LatentDif::flags` (`:156–162`) | computation (a stationary point at tolerance, not a certified maximum); inference (a non-conclusion must not accept; a retry is an opportunity, §9.5) |

`ClassCurves::of` does not read `LatentDif::status`, which `flags` reads: an observation on the
contract, not a defect observed on a production path, since none composes them today; a procedure
built on them would have to refuse a non-converged fit itself. A3 stays closed within its scope
(`18`): these are B1–B3.

**Reference logic, for the obligations only.** `U = sup {D(ϑ) : ϑ ∈ 𝒞}` over a region `𝒞` of
parameters of every candidate order bounds `DTF_μ(T)` with probability `1 − α` given: (i) the
region's coverage, `P(ϑ* ∈ 𝒞) ≥ 1 − α`, at finite samples or asymptotically and uniformly where the
decision is close, weak identification and small components included; (ii) representation,
`D(ϑ*) ≥ DTF_μ(T)` (R-a–R-f, the envelope where H2's groups vary in composition, the treatment of
small or duplicated components); (iii) a conservative supremum, a local optimizer under-estimating
it. Neither an available construction nor an approved choice; (i)–(iii) stay open.

**Where the obstacles sit.** Three kinds, kept apart. *Target and representation*: R-a–R-f and
the per-component filter decide what a bound is about. *Statistical and computational validity*:
a region's coverage across orders, regularity, the penalties' limit and a conservative supremum
decide whether a bound holds at all; a failure there invalidates the bound, it does not only
widen it. *Precision, usefulness and cost*: given a valid bound, its width against `DTF_MAX`, the
samples and searches it needs. The shared aim is a coherent, executable and evaluable
experimental protocol (§10); that direction approves no protocol and closes no finding.

## 10. Toward a circumscribed experimental protocol (proposal awaiting review)

**Status.** A proposal by Claude Code on the direction of Astra's review of `00f8e2c`, **awaiting
review**; the approval of §9.6 does not cover it. It builds on §6.1, §§9.1–9.6 and `17` §§7–8
without restating them. It adopts, funds and closes nothing; anonymity (no personal or group
attribute enters) and the recovery of contested facts bound it; the test's neutrality is not
certified. Evidence: **L** targeted reads at `00f8e2c`, named in place; no run.

**Three categories.** *A*: decisions needed before any run, because a claim's meaning or a
record's content depends on them. *B*: hypotheses a circumscribed study can measure, each with the
observation that could refute it and the conclusions the measure would not authorize. *C*:
promises excluded until their proof exists. A requirement of validity stays one when a procedure
is called experimental: the study changes which claims are made, not what those claims need.

### 10.1 A1 — the forecast outcome and how it is produced

- **A.** B-b as the score's contract (forecast `q_c`; an `I` adds 0 at the fixed `N_u`; a pending
  item is never 0) under `17` §8.1's H-a–H-e. The observation arm: A, the main analytic reference,
  C kept as its comparison, or both; none adopted (`17` §8.5). Assignment, selection, terminal
  outcome and consolidation kept as distinct records (`17` §§8.1–8.3). Rules still missing: a
  report not completed (case 6: no `p`; C4 against withholding), a freeze not reached (case 7;
  T58), the terminal-inconclusiveness record (`17` §7.2), the availability property and any
  overdue rule (`17` §8.3), what is published before consolidation, and whether study scores feed
  any reputation.
- **B.** On the study's records: every assignment in `N_u` and every case recorded as `17` §8.6
  requires, refuted by one missing assignment or one pending item read as 0 or `I`; the share of
  terminal inconclusiveness and the time to consolidation by verdict class (`R` can end before
  `A`, `17` §8.3). Not authorized by them: H-c (no frequency of outcomes shows that a reviewer
  cannot move `(b, Y)`), properness under deviations, or a value of `c` beyond the study's items
  and procedure.
- **C.** Incentive claims for the reputation in use — counters, shrinkage, cap, probation, CUSUM
  (`17` §8.4, `16` §4.5): D1 is a property of a prefixed cohort's final score, not of them.
  Unbiased intermediate values (`17` §8.3). Under arm C, any claim resting on C1 while `16` §6's
  residual stays open.

### 10.2 A2 — the target and the meaning of the DTF

Three objects, never interchanged: `D(T)`, the contested facts' admission cost (R1, §8); the
model's functional, `DTF_F` or `E_F` at one representation, filtered per component (§9.6); the
target `DTF_μ(T)` on a population (§2).

- **A.** For each statement of the study, which object it concerns. For the target, `μ` and `𝒢`
  at least symbolically, and forms fixed before the data (§9.6's reference). The calibration
  comparison stays open — common, per form, or both as arms — neither presumed valid or
  inevitable (§9). No share floor, merging or minimal representation is chosen; the per-component
  filter defines no protection of real groups (§9.6), so the study claims none.
- **B.** Diagnostic uses that could be studied: on generative scenarios with known truth, the
  plug-in `DTF_F` against the model's functional at the true parameters (`13` §7.3: an upward bias
  near 0, noise near the tolerance) and its sensitivity to the class search and to classes near
  the share floor or nearly coincident; on anonymous field data, the stability of fitted
  quantities across disjoint samples and across calibration arms. A disagreement refutes
  stability; an agreement does not show the true DTF, since both fits share model and
  representation. Duplicated components leave the law unchanged: they are a check on the
  functional, not a scenario.
- **C.** A test's DTF within `DTF_MAX`, or any certified neutrality; `D(T)` as a bound; protection
  of real groups by a form that passes the filtered functional; a bound with stated coverage
  (§9.6's obligations stay open).

### 10.3 B1–B3 — model, selection and inference

Three questions, apart: the procedure's correctness (the code computes the specified model and
functional, checked by known-answer constructions such as §9.6's); its sensitivity to the model
(priors, penalty, class search, misspecification); a bound's validity (coverage and a conservative
supremum, §9.6).

- **A.** The model version, priors, penalty and class search pinned for the study; the generative
  scenarios, misspecified populations included (`15` B1–B3's completion evidence), and the
  decision-level criteria declared before any result; any composed procedure refusing a
  non-converged fit (§9.6, `ClassCurves::of`).
- **B.** On those scenarios: verdict errors, convergence and indeterminacy, one-class selection
  where the truth has DIF (§4, row 4), the plug-in error, each refuted by observed rates. They
  verify behaviour on the scenarios run, not uniform coverage over the family and not validity in
  the field. On field data: admission, convergence and indeterminacy rates (`15` B4–B5). The true
  DTF is not observable in anonymous field data, and a comparison with another fit does not
  measure it.
- **C.** Coverage of any bound; identification beyond §9.6; BIC selecting the true order; a social
  meaning of the classes. No personal or group attribute is collected, so no field validation
  against labelled groups exists.

### 10.4 Resources and executability

Counted apart, with §6.1's and §9.3's formulas and limits: distinct persons, participations,
answers with anchors, model searches (9, 17 or 25 optimizer runs each), attempts, times; no size,
duration or performance is assumed. Enrolled users are not available respondents: a respondent
pseudonym answers one batch and epoch once (`pilot::submit_response`, `NullifierSet`), and how
many respondents are available, for how many items each, are product and resource quantities.

| Link | Executable path today (L) | None yet (L) |
|---|---|---|
| respondents | `pilot::submit_response` through `NodeState::apply` | a recruitment or availability mechanism |
| pilot | `pilot::stage1_screen`, `revalidation::latent_batch` as APIs; `orchestrator::run_item` steps the lifecycle on verdicts its caller supplies | a runtime composing responses, fits, verdicts and terminal records (`17` §1, §8.3) |
| outcomes | `Screening::Indeterminate` and `Recheck::Indeterminate` keep an item in its pilot stage (`lifecycle.rs`) | a terminal-inconclusiveness record; an attempt budget and term |
| B-b scores | `SkillTrack` (reviewed and scored counts), `panel_scores`, `ResultRecord::{ReviewerScore, ReviewerUnobserved}` | B-b's contribution; `N_u`, `O_u`, `V_u` (`17` §8.4); T58's missing-reveal rule (`PartialEpoch` freezes today) |
| DTF | `ClassCurves::dtf` on one fit's items; `ContestedPool`, composed by no runtime | a form's DTF with its active items, the envelope, any bound |
| scenarios | the characterization harness's generative studies (`DifMisspec`, `DtfError`, …; `DifDesign` with a class share `pi`) | scenarios and criteria declared for this study |

A study harness may stand in for a service where stated; it is not operational readiness and
closes no part of Phase 2.

### 10.5 Proposal for Astra's decision

My recommendation, not an approved decision.

- **Minimal perimeter.** Two separable strands. *S1*: generative scenarios with known truth, no
  respondents — §10.3's sensitivities and §10.2's diagnostic errors on declared scenarios; it
  would show how the procedure behaves there, not coverage nor field validity. *S2*: a field pilot
  on a few items fixed before the data, observation arm A, anonymous respondents, every case of
  `17` §8.2 recorded, scores computed and kept out of any reputation; it would show whether
  records, terminal outcomes and consolidation are produced, at which rates, times and resources —
  not incentives, the true DTF or neutrality.
- **Blocks today.** For S2: the pilot runtime, the terminal-inconclusiveness record and the
  attempt term; the rules for cases 6 and 7 and T58; the availability property; B-b's contribution
  and counters (§10.4). For any DTF statement beyond diagnostics: no declared `μ` and `𝒢`, no bound.
  For both strands: scenarios and criteria not yet declared; respondent availability and the
  tolerable load per participation unknown; burden and runtime unmeasured (`15` D2, D4).
- **Decisions before implementation** (Astra): S1, S2 or both; arm A, C or both; cases 6 and 7 and
  any overdue rule; the terminal record's content; scores kept out of reputation; the DTF object
  of each statement and the diagnostics reported; the scenarios and criteria fixed in advance;
  whether a calibration arm belongs to S2.
- **Owner information**, only where no symbolic scenario settles it, and not asked now: available
  anonymous respondents and tolerable items per participation; the horizon's numbers only if a
  calibration arm is kept.
- **Next intervention recommended.** A documentation-level specification of B-b's records and
  rules for cases 5–7 under arm A — the terminal-inconclusiveness record, the attempt term, case
  6's options with their incentive consequences, case 7 against T58 — presented as options for
  Astra. It lies on S2's critical path, needs neither respondents nor resource decisions and
  changes no code. The alternative is S1's declaration of scenarios and criteria, which needs no
  respondents but whose run is a campaign the owner authorizes.
