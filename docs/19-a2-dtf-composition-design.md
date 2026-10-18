# 19 — A2: DTF across fits and the whole-test claim

| | |
|---|---|
| **Status** | Diagnosis and conditional proposition **approved** by Astra on `0519626` (below); **R1 implemented in `8fa07dd`, completed by `d13bf09` and approved by Astra on `d13bf09`** (below, §8). A2 stays open (`15`): R1 rectifies the claims, it does not realize the whole-test guarantee. §6.1 adds a decision synthesis, written on `fa11791`, linking A1, A2, B1–B3 and resources, **approved by Astra on `cebcb2e` as a documentation intervention** (below); no protocol, calibration, adoption or expenditure approved. §9 compares a reusable common calibration with a calibration per form under explicit assumptions, written on `cebcb2e`, rectified in `63b13fa` and **approved by Astra on `63b13fa` as a conditional comparison** (below); it chooses no design and does not close A2. Added after that review: the check sample's conditions and the cost hypotheses (§9.3), the matching lines of §9.4 and the error control over a search (§9.5), partly approved by Astra on `6681b03` and, after its rectifications in `b59fd03`, **approved by Astra on `b59fd03` as a conditional analysis** (below). §9.6, on one fixed form's representation, identification and inference, was partly approved by Astra on `907c245`, rectified in `00f8e2c` and **approved by Astra on `00f8e2c` as a conditional analysis** (below). §10, a decision synthesis toward an experimental protocol, was **approved by Astra on `26593e7` as a preparatory synthesis** (below), with four precisions made in it and approved on `d31e9fa` (below); the specification of cases 5–7 it led to, [`17`](17-a1-pilot-batch-design.md) §9, was partly approved by Astra on `d31e9fa` and **approved on `b2a1c2f` as the study's reporting specification**; the bridging input under `q_c` is analysed in `17` §10, **approved by Astra on `111e595` as a conditional analysis**; timing under A is specified in `17` §11, partly approved by Astra on `82bdb9a` and **approved on `cf5575a` as a conditional comparison between T1 and T2** (neither adopted); access and disclosure per object are specified in `17` §12, partly approved by Astra on `8af257d` and **approved on `7d8db08` as a conditional analysis of the informational requirements**; the observable evidence under T1 for its predicates (ii) and (iv) is specified in `17` §13, partly approved by Astra on `f38f8a4` and **approved on `176dd8c` as a conditional analysis of the observable evidence**. §10.6, a minimal perimeter for S2, was partly approved by Astra on `cbcbc67`, with decisions taken for preparing its specification only, rectified in `5af22b9` and **approved by Astra on `5af22b9` as a preparatory synthesis of the experimental perimeter** (below); the synthetic-event check it leads to is specified in [`20`](20-s2-synthetic-check.md), which that approval does not cover, **approved by Astra on `c76c035` as the check's specification**; its implementation in test code, partly reviewed on `e5e0898`, was rectified and **approved on `3559396` within the check's perimeter** (`20` §9). §11’s **candidate A2 guarantee contract approved by Astra on `5e600be`** as a specification of obligations; §12’s split-likelihood coverage proof, conditional on its model, sampling and split hypotheses, and §13.3's corollary, in its domain, are **accepted by Astra on `5102852`** (below); the five rectifications that review asked for, in §12.3 and §§13.4–13.7, and §13.5's contract are **accepted by Astra on `2aa136d`** (below); §13.5's bounded information check, run in §13.8, is **accepted by Astra on `24cdfce` as an obstacle demonstrated in its declared domain** under its numerical contract (below), without closing A2. **The owner defers A2's design decision (2026-10-14, §13.9):** the whole-test requirement is kept, A2 remains open and blocking for Phase 1 under the current criteria, and work on its certifier is suspended pending that decision. A1’s subsequent conditional closure on `1bd865c` is recorded only in `17` §14.15; it supplies none of A2’s missing obligations. No formula, API, threshold, serialization, selection, golden output or historical result changes. |
| **Baseline** | `docs/phase1-review-alignment` at `16e862c`. Line references are to that commit; §9 names code by symbol, at `cebcb2e`. |
| **Scope** | `15` A2: the contested pool's cost `D(T)`, a sum of per-fit DTFs, and the guarantee the docs attach to it for a test. B1–B3 (identification, BIC) only where A2 needs them. |
| **Evidence** | **L** read; **D** proved here; **C** recalculated (`sim/dtf_composition.py`); **E** executed. E is labelled *API* (real code on hand-built curves), *fit* (a real latent fit) or *frequency* (none here). |

**Current owner direction (2026-10-18):** foundations first, then bounded iterative
evaluation under [§13.10](#1310-owner-direction-foundations-and-iterative-testing-2026-10-18). This supersedes §13.9's
blanket work suspension only to that extent; no design option, numerical operating point
or closure is adopted. The review history below is unchanged.

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

Review on `26593e7` (the record of `00f8e2c`'s review, §9.6's precisions and §10), distinct from
the reviews above:

- Approved: the record of the review of `00f8e2c`; §9.6's three precisions; §10 as a preparatory
  synthesis, with four precisions, made in §10: blocks by strand; the bridging input's meaning
  under `q_c`; what `ClassCurves::dtf` already computes; measures refuting hypotheses only
  against criteria declared in advance.
- Not approved: the S2 protocol, any implementation, the start of an experiment, penalties or
  new reputational rules. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Direction confirmed: specify the records and rules of `17` §8.2's cases 5–7, design A as the
  study's reference and C the comparison, keeping apart the procedure's term, the terminal record
  and the close of the study's observation (`17` §9, awaiting review).
- Evidence: the commit, its parent and the published HEAD checked; the diff, the documents and
  the relevant passages of `run_item`, `lifecycle`, `ResultRecord`, `SkillTrack` and
  `ClassCurves::dtf` read; no new calculation needed; no Rust, fit or campaign run; the local
  working tree as reported by Claude Code, not checked through GitHub.

Review on `d31e9fa` (§10's precisions and `17` §9), distinct from the reviews above:

- Approved: the record of the review of `26593e7`; §10's four precisions. `17` §9 is partly
  approved and rectified there (its status records what); it awaits review.
- Decided for the study's reporting specification only: `17` §9's options 6c and 7a as its
  basis, unresolved cases left unresolved with no imputed value; not approved: S2's start, an
  implementation, penalties or reputational policies. T58's deepening is no prerequisite of that
  perimeter and stays open for a procedure that must proceed despite missing reports.
- Direction: after those rectifications, the priority link is the bridging input's meaning when
  the report forecasts `q_c` (§10.1). R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Evidence: the commit, its parent and the published HEAD checked; the diff, the documents, T58
  and the relevant lifecycle arms read; a small rational calculation on the baseline's dependence
  channel; no Rust, fit or campaign run.

Review on `cbcbc67` (the record of `176dd8c`'s review, `17` §13's precisions and §10.6), distinct
from the reviews above; the reviews of `17` §§9–13 are recorded in `17`:

- Approved: the record of the review of `176dd8c`; `17` §13's three precisions; the further
  changes to `17` §13.2's classification at (R) and (E) — an absence concerns acceptance in the
  declared prefix, without showing that the commitment or reference did not exist elsewhere.
- §10.6: **partial**. The direction accepted; the section as a whole to rectify. Asked, and made
  in §10.6: three kinds of evidence, the synthetic-event check promising no fit or search time;
  anonymity and D17 as requirements of the path, in place of a statement that they hold; α's
  meaning, in place of a claim that it is harmless without effects; the floors' sum qualified and
  the counts kept apart, the capability table corrected; the decisions for that check apart from
  those before a human study, with no circular need for the rates a pilot would measure.
- Decided, for preparing the specification only: B-b's study scores outside the reputation; the
  gate's decisions recorded, with no effect on operational pools; α the semantic reference to
  specify; the synthetic-event technical check the next deliverable, after this rectification.
  Not authorized: an implementation, recruitment, S2's start, the adoption of T1 or T2, new
  protocol policies. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Non-blocking: §10.6's length beyond an indicative 800 words; the synthesis kept compact.
- Evidence: the commit, its parent and the published HEAD checked; the diff and the relevant
  passages of the pilot, revalidation, enrollment, latent, harness and contracts read. No Rust,
  fit, benchmark or campaign run; the local working tree not checked.

Review on `5af22b9` (§10.6's rectifications), distinct from the partial review above:

- Approved, with no blocking finding: the rectifications and §10.6 as a preparatory synthesis of
  the experimental perimeter. Resolved: synthetic events, synthetic responses through the fitters
  and human participants kept apart; anonymity and D17 as requirements; α's meaning and the
  provisional thresholds' limits; the 3,300 participations qualified; the technical check's
  prerequisites apart from the human study's. Approved: the gate check corrected to
  `bridging_gate`'s decision on the fixture's inputs, with no claim on how those inputs are
  produced.
- Confirmed for the next specification: B-b's study scores outside the reputation; the gate
  recorded with no effect on operational pools; α the semantic reference; fixtures with explicit
  synthetic budgets and terms, small values chosen to exercise the cases, never parameters of the
  protocol or of a future human study.
- Not authorized: an implementation, a run, the adoption of T1 or T2, data collection or S2's
  start. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- Two editorial joins, made after the review: §10.4 names `pilot::screen` as the gated entry of
  `stage1_screen`'s computation; `17` §13.2's absent-precondition row stands on its own.
- Evidence: the commit, its parent and the published HEAD checked; the diff of the four
  documents read; the joins checked; `bridging_gate`, `supplementary_review`, `screen` and
  `stage1_screen` read again. No Rust, fit, benchmark or campaign run; the local checks and the
  working tree stay Claude Code's evidence.

Review on `5e600be`:

- **Approved, with no blocking finding:** the record of A1’s conditional closure and §11
  as a candidate A2 contract. This approves the specification of obligations, not an A2
  guarantee already demonstrated; representation, coverage and conservative computation
  remain distinct. A2, A12 and Phase 1 remain open.
- **Astra’s evidence:** diff read and compared with §§3 and 9.6. No Rust, fit or new numerical
  check. Separate from Codex’s documentation work and checks in the handoff.
- Subsequent §12’s mathematical construction/proof is not covered by that approval and
  awaits review; no implementation is supplied.

Review on `5102852` (§12 and Claude Code's §13), distinct from the reviews above:

- **Accepted within the declared scope:** §12's mathematical coverage proof, conditional on its
  explicit model, sampling and split hypotheses; §13.3's corollary for one-class truths, with
  components of arbitrarily small share admitted and a linkage allowing their contrast. Neither
  extends to other domains or to an implementation's numerical correctness.
- **A2 open:** domination, conservative computation, usefulness in any revised domain, cost and
  scope decisions are not settled by coverage. The two-point check of §13.5 is a possible proof
  of an obstacle; a "not excluded" outcome proves neither feasibility nor that a share floor
  suffices.
- **Asked:** five corrections — the region's inclusion (§12.3), the `χ²` identity (§13.4), the
  sample-size frontier's conditions, the numerical contract of an obstacle and the reading of
  the outcomes (§§13.5–13.7). Made in `2aa136d` and accepted on `2aa136d` (record below).
- Astra's evidence is not stated in the outcome transmitted to Claude Code; none is recorded here.

Review on `2aa136d` (the five rectifications and §13.5's contract), distinct from the reviews
above:

- **Accepted, with no new blocking finding:** the five rectifications made after `5102852`
  (§12.3, §§13.4–13.7) and §13.5 as the contract of the bounded information check.
- **Evidence:** the commit, the texts and the pertinent mathematical steps verified. Astra ran
  neither the rectifications' scratch script, nor a Rust test, nor §13.5's check.
- **A2 open;** the earlier mathematical acceptances keep their own domains. The check was then run
  as §13.5 specifies (§13.8); its result is accepted on `24cdfce` (record below).

Review on `24cdfce` (§13.8, the bounded information check's result), distinct from the reviews
above:

- **Accepted:** §13.8 as an obstacle demonstrated in §13.5's declared domain, under its numerical
  contract. A2 is not closed.
- **Astra's evidence:** the published commit and its parent `2aa136d` checked; the script, the
  intervals' justifications and the result read; the grid's construction compared with
  `Grid::new`, `normal_logits` and `Grid::shape`; the seven Python tests passing; the complete
  check run in 25.1 s, its output identical to the committed JSON apart from the environment and
  timings block; the tolerance's strict exceedance verified on `D`'s lower bound in all 27
  configurations. No Rust, fit or campaign run.
- **Kept, each in its own domain:** §12's conditional coverage, §13.3's corollary and §13.8's
  numerical result. None is reopened or implicitly extended.
- **Three precisions asked, made after `24cdfce`:** the logarithm's remainder (§13.8); the output's
  label, which now separates certified bounds from consistency checks (the script and the
  recorded JSON, that string only); the alternatives, where contrasts limited to present groups do
  not overcome the balanced obstacle (§§13.7–13.8).
- The owner's subsequent decision is in §13.9.

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

## 10. Toward a circumscribed experimental protocol (preparatory synthesis, approved on `26593e7`)

**Status.** Written by Claude Code in `26593e7` on the direction of Astra's review of `00f8e2c`;
**approved by Astra on `26593e7` as a preparatory synthesis** (design review above), its four
precisions made below; the S2 protocol, any implementation or experiment, penalties and new
reputational rules not approved. It builds on §6.1, §§9.1–9.6 and `17` §§7–8 without restating
them; the cases 5–7 it led to are specified in `17` §9, approved on `b2a1c2f`. It adopts,
funds and closes nothing; anonymity (no personal or group attribute enters) and the recovery of
contested facts bound it; the test's neutrality is not certified. Evidence: **L** targeted reads
at `00f8e2c`, named in place; no run. §10.6, S2's minimal perimeter on `17` §§9–13, was
approved on `5af22b9` as a preparatory synthesis; its technical check is specified in `20`,
approved on `c76c035`, its implementation approved on `3559396` within the check's
perimeter.

**Three categories.** *A*: decisions needed before any run, because a claim's meaning or a
record's content depends on them. *B*: hypotheses a circumscribed study can measure, each with the
observation that could refute it and the conclusions the measure would not authorize. *C*:
promises excluded until their proof exists. A requirement of validity stays one when a procedure
is called experimental: the study changes which claims are made, not what those claims need. An
observed rate or difference is a measure: it refutes a hypothesis only against a metric, a
tolerance and a treatment of uncertainty declared before the results; none is proposed here.

### 10.1 A1 — the forecast outcome and how it is produced

- **A.** B-b as the score's contract (forecast `q_c`; an `I` adds 0 at the fixed `N_u`; a pending
  item is never 0) under `17` §8.1's H-a–H-e. The observation arm: A, the main analytic reference,
  C kept as its comparison, or both; none adopted (`17` §8.5). Assignment, selection, terminal
  outcome and consolidation kept as distinct records (`17` §§8.1–8.3). Rules still missing: a
  report not completed (case 6: no `p`; C4 against withholding), a freeze not reached (case 7;
  T58), the terminal-inconclusiveness record (`17` §7.2), the availability property and any
  overdue rule (`17` §8.3), what is published before consolidation, and whether study scores feed
  any reputation (`17` §9, the study's reporting specification for cases 5–7, approved on
  `b2a1c2f`). The bridging input:
  what the rating means when the report forecasts `q_c`, with `τ`, the band and the polarization
  cut not validated for that meaning (`17` §7.5, §8.5); keeping study scores out of the reputation
  does not settle it, and no transformation, threshold or bypass of the bridging is chosen
  (analysed in `17` §10, approved on `111e595`; α the main reference, β the comparison).
- **B.** On the study's records: every assignment in `N_u` and every case recorded as `17` §8.6
  requires, a correctness check that one missing assignment or one pending item read as 0 or `I`
  refutes; the share of terminal inconclusiveness and the time to consolidation by verdict class
  (`R` can end before `A`, `17` §8.3), measures that refute a hypothesis only against a criterion
  declared beforehand. Not authorized by them: H-c (no frequency of outcomes shows that a reviewer
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
  quantities across disjoint samples and across calibration arms. A difference between fits does
  not by itself refute stability: that needs a metric, a tolerance and a treatment of sampling
  uncertainty declared before the results, none proposed here; an agreement does not show the
  true DTF, since both fits share model and representation. Duplicated components leave the law
  unchanged: they are a check on the functional, not a scenario.
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
  where the truth has DIF (§4, row 4), the plug-in error: measures, each refuting a hypothesis
  only against a criterion declared beforehand. They verify behaviour on the scenarios run, not
  uniform coverage over the family and not validity in the field. On field data: admission,
  convergence and indeterminacy rates (`15` B4–B5). The true DTF is not observable in anonymous
  field data, and a comparison with another fit does not measure it.
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
| pilot | `pilot::stage1_screen`, the stage-1 computation's API, entered with its size gate through `pilot::screen`; `revalidation::latent_batch` as an API; `orchestrator::run_item` steps the lifecycle on verdicts its caller supplies | an executable path composing responses, fits, verdicts and terminal records (`17` §1, §8.3); for S2 a study harness would do, not necessarily the production runtime |
| outcomes | `Screening::Indeterminate` and `Recheck::Indeterminate` keep an item in its pilot stage (`lifecycle.rs`) | a terminal-inconclusiveness record; an attempt budget and term |
| B-b scores | `SkillTrack` (reviewed and scored counts), `panel_scores`, `ResultRecord::{ReviewerScore, ReviewerUnobserved}` | B-b's contribution; `N_u`, `O_u`, `V_u` (`17` §8.4); T58's missing-reveal rule (`PartialEpoch` freezes today) |
| DTF | `ClassCurves::dtf` on any items of one fit, a form's active items included when the form lies in that fit; `ContestedPool`, composed by no runtime | the operational composition of a form with its fit; the envelope; any guarantee on the target |
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
- **Blocks today, by strand.** *S1*: scenarios and decision criteria not yet declared;
  respondent availability and human load do not block it. *S2*: an executable path from responses
  to terminal records — a study harness may serve, not necessarily the production runtime; the
  terminal-inconclusiveness record and the attempt term; the rules for cases 5–7 (`17` §9, the
  reporting specification approved on `b2a1c2f`, 6c and 7a its basis); B-b's contribution
  and counters (§10.4); the bridging input's meaning (§10.1); respondent availability and the
  tolerable load per participation, unknown. A general availability guarantee is not needed to
  study delays, which the study records; the study's end does not turn a pending item into a
  terminal `I` of the protocol (`17` §9.1). *Any DTF statement beyond diagnostics*: no declared
  `μ` and `𝒢`, no bound. Burden and runtime stay unmeasured (`15` D2, D4).
- **Decisions before implementation** (Astra): S1, S2 or both; arm A, C or both; cases 5–7 and
  any overdue rule; the terminal record's content; scores kept out of reputation; the bridging
  input's meaning under `q_c`; the DTF object of each statement and the diagnostics reported; the
  scenarios, metrics, tolerances and treatment of uncertainty fixed in advance; whether a
  calibration arm belongs to S2.
- **Owner information**, only where no symbolic scenario settles it, and not asked now: available
  anonymous respondents and tolerable items per participation; the horizon's numbers only if a
  calibration arm is kept.
- **Next intervention.** The specification of B-b's records and rules for cases 5–7 under arm A
  is `17` §9, approved on `b2a1c2f` as the study's reporting specification; the priority link
  Astra set next, the bridging input's meaning when the report forecasts `q_c` (§10.1), is
  analysed in `17` §10, approved on `111e595`; timing under A follows in `17` §11, approved on
  `cf5575a` as a conditional comparison; access and disclosure per object follow in `17` §12,
  approved on `7d8db08` as a conditional analysis; the evidence under T1 for its predicates (ii)
  and (iv) is in `17` §13, approved on `176dd8c` as a conditional analysis; S2's minimal
  perimeter, §10.6, was approved on `5af22b9` as a preparatory synthesis, and its synthetic-event
  check is specified in `20`, approved on `c76c035`, its implementation approved on `3559396`
  within the check's perimeter. S1's
  declaration of scenarios and criteria stays an alternative strand, needing no respondents but a
  campaign the owner authorizes to run.

### 10.6 S2's minimal perimeter (preparatory synthesis, approved on `5af22b9`)

**Status.** Claude Code's proposal, committed in `cbcbc67`; partly approved by Astra on
`cbcbc67` (design review above: the direction accepted, the decisions taken for preparing the
specification and what they do not authorize), rectified below in place in `5af22b9` and
**approved there as a preparatory synthesis of the experimental perimeter, with no blocking
finding**; no implementation, run, data collection, adoption of T1 or T2, or start of S2.
**L** targeted reads; **C** one sum of documented floors; no run.

**A. Question, result, kinds of evidence.** Under design A, for a few items grouped and fixed
before the data, does an executable path from each group record to its terminal records produce,
at a declared snapshot `Ω`, the records `17` §§9.5 and 13.5 need? The result is that snapshot:
records; terminal and pending outcomes by unit (`17` §9.5); which contributions exist; each
pair's documentable order (`17` §13.2); delays from named origins; the resources recorded. Three
kinds of evidence, apart:

- *events and outcomes supplied as fixtures*: the handling of records, states and
  classifications (`17` §§8.6, 13.2), and the accounting of costs the fixtures supply, not the
  cost of a fit they do not run. The next technical check is of this kind only.
- *synthetic responses through the real fitters*: a distinct statistical execution, with its own
  costs and authorization; like any benchmark, outside the next perimeter.
- *human participants*: behaviour, availability, delays and load — missing and late reports, real
  admissions, outcome frequencies on these items.

A replay or record-handling time, if reported, is named for what it measures, extrapolated
neither to fits nor to a human pilot. No kind licenses conclusions on properness, the
reputation's incentives, H-c, the true DTF or neutrality; S2 settles no part of A2. Without a
proof of H-c the study can see whether the path works, not read its scores as
incentive-compatible.

**B. Perimeter.** The approved references, none adopted as a protocol: 6c and 7a, nothing
imputed; item, assignment, group and cohort apart; `Ω` fixed before the start, a later record
changing the state, not the snapshot; α and T1 the analytic references, β and T2 the comparisons
(`17` §§9–13). As decided: B-b's study scores computed (`17` §9.5) and fed to no reputation; the
gate's inputs and decisions recorded, none applied to an operational pool. No real effect,
penalty or reputational rule enters, and no pool entry discloses `A` (`17` §12.3). The study then
leaves aside behaviour under stakes (`16` §4.5, item 4), pool entry as a channel and the pool's
share of respondents (`17` §7.3, *Load*). With `Φ_j` unchanged, appeal windows still delay T1's
start; that delay is recorded.

**α's meaning.** α stays the specification's reference (`17` §10.3), to be specified. Without
reputational or pool effects, the forecast's meaning can still move reports, gate decisions,
extra rounds, appeals and times, and so the study's observations. `τ`, `ε` and the gap stay
provisional and unvalidated for that meaning; no change of their values is approved. What is
observed concerns that experimental path, and shows neither properness nor reputational
incentives.

**C. Capabilities strictly needed.**

| Capability | Exists (L) | Missing | Synthetic check | Evidence it works there |
|---|---|---|---|---|
| group record | — | the record (`17` §11.1) | yes | precedes every `AssignReviewers` on replay |
| reviewer steps, gate | lifecycle steps through `Node::submit`, `Ledger::apply`; `bridge_scores`, `bridging_gate` | a driver filling ratings from reveals (`17` §10.1) | steps and gate inputs as fixtures | replay applies or refuses each step; each recorded decision is `bridging_gate`'s on the fixture's inputs (recomputing them runs the bridging fits, a statistical execution outside this check) |
| `Φ_j`, `Φ_G`, T1's start | computable from the log | rules naming them | yes | a known-answer log |
| pilot to terminal records | `pilot::screen` (the `N1_MIN` gate, then `stage1_screen`'s computation); `latent_batch` (its gates, then the search) | the composing path; attempt, `I` and shortfall records; budget and term; A's pilot without `Explore` (`17` §12.5) | outcomes and refusals as fixtures, under the fixture's rule (D) | `17` §8.6; an exhaustion under that rule read `I`, a silence pending, a shortfall and a refused batch recorded apart, neither `I` |
| B-b scores, counts | `SkillTrack`, `panel_scores` | the contribution; `N_u`, `O_u`, `V_u`; frozen weights recorded | yes, offline | `17` §8.6, item 5, on an exact enumeration |
| order evidence | replica, cuts, ledger; co-signing between member nodes on loopback (AT-NET-17) | an epoch logged between nodes (T79) | yes; on one feed, `17` §13.1's cross-feed case unexercised | `17` §13.2's outcomes on constructed orders |
| answers, holder | `pilot::submit_response`, `AdmitRespondent` | a holder and an access rule (`08` Q-1) | synthetic admissions, P5 declared | no answer on the log |
| pseudonyms | enrollment pipeline, BBS+ issuance, role nullifiers | for a field study: identity-bound enrollment (T20), recruitment | synthetic ids, no enrollment | none on persons |

Not needed for the check: the full distributed runtime, T58, an availability guarantee, T20,
recruitment, human availability.

**Anonymity and D17: requirements of the path**, to meet where the construction is missing. No
personal or group attribute may enter. Answers off the log and no attributes do not settle the
visibility of the reveals and other objects: access decisions remain for reports (who reads
commitments and reveals, when; every reveal is public per nym today, `08` PRIV-004, T77),
answers (a holder and an access rule, `08` Q-1) and results (attempt statuses, terminal records,
gate decisions, study scores: readers and an opening rule, `17` §§12.2, 12.5). The future path
must not extend the publication T77 contests. No cryptography is designed here. Synthetic data
alone keep real data out of the technical check, and validate nothing of a future human study's
confidentiality. Issuing no inclusion proof keeps `08` PRIV-004.1's specific vector out of the
proposed path, with no general confidentiality guarantee; PRIV-004.1's remediation stays open,
apart from A1.

**D. Decisions.** Taken, for the specification: 1, scores and pool (B); 2, α (above). Open for
the synthetic-event specification:

| Decision | Alternatives | Observable consequence | Recommendation |
|---|---|---|---|
| 3. the group procedure in the fixture | the code's provisional thresholds, declared; an attempt rule — a synthetic budget and term — or none | without a rule no exhaustion is checkable, so no case 5 | the provisional thresholds pinned (§10.3), a synthetic budget and term declared in the fixture; none becomes a parameter of the human study or of the protocol |
| 4. a recorded shortfall, or a refused stage-2 batch (`latent_batch` needs `K_MIN = 2` items) | apart; `I` by a candidate rule (`17` §9.6) | apart: no term, its cohort unconsolidated | distinct records, neither turned into `I` automatically, nothing imputed |
| 5. pseudonyms in the check | synthetic ids | records and times per synthetic pseudonym, nothing on persons | synthetic ids; T20, recruitment and human availability no prerequisite, and no enrollment chosen for the field |

*Before a human study is authorized*, not asked now: the field's budget and term per group;
pseudonyms and enrollment (T20 open), recruitment and availability; the access decisions above
and a construction meeting them; the planning hypotheses and resource limits of E.
*Deferrable*: T58, 6a, 6b; availability and overdue rules; adopting T1 or T2; arm C and the
beacon; reputational rules; any calibration or DTF statement beyond diagnostics; PRIV-004.1's
remediation; a reference binding answers at collection — without it, (iv) pairs keep `17`
§13.2's outcomes and their channel stays declared.

**E. Resources**, counted apart, with §6.1's and §9.3's formulas; `02` §B.6's floors are not
power.

- *Participations*: a stage-1 administration passes `pilot::screen`'s gate with at least
  `N1_MIN = 300` admitted respondent pseudonyms, a stage-2 batch `latent_batch`'s with at least
  `N_LATENT_MIN = 3,000`. 3,300 is the sum for two distinct administrations, one passing each
  gate, counting the participations at both stages — not the least cost of every attempt that
  reaches `Pilot2` or calls the stage-2 procedure: a batch can be refused before the fit, for
  `K_MIN` among other gates. Per item, a first panel of odd `k ∈ [7, 11]` (`AssignReviewers`),
  plus `K_EXTRA = 4` by default in the band.
- *Pseudonyms counted by the gates*: per batch, each answering once (§10.4); one pseudonym can be
  admitted to both stages' batches. *Distinct persons*: no count guarantees them (D, 5).
- *Answers*: per participation, trial items and anchors (§6.1's `t_F + a_F`), refused batches
  included.
- *Searches actually run*: a stage-1 fit per screen past its gates; per batch past
  `latent_batch`'s, with `LatentParams`' defaults, one latent search of 9, 17 or 25 optimizer
  runs (§9.3); per ratings matrix a collective bridging fit and `m = 10` bootstrap fits (`02`
  §A.4), and a fresh fit per supplementary review. *Attempts refused before the fit*: no search;
  recorded and counted apart.
- *Times*: computation (`15` D4) apart from waits — reveals, band, appeals, the slowest freeze,
  records — which no count of rounds bounds (`17` §9.4).

No saving from reusing data is assumed; no statistical power or feasibility follows from the
floors. Before a human study is authorized: planning hypotheses and resource limits — burden per
participation (`15` D2), pseudonyms available, computation. Refusal and attempt rates can be that
study's results; a preliminary collection, if wanted, is a separate activity to delimit and
authorize.

**Next.** The documentary specification of the synthetic-event technical check is in
[`20`](20-s2-synthetic-check.md), approved by Astra on `c76c035` with four precisions: its
fixtures (events, outcomes, refusals, the declared synthetic budget and term), its records, the
snapshot report at `Ω`, the checks of `17` §§8.6 and 13.2, and the accounting of the costs its
fixtures supply. Its implementation in test code, authorized by the owner, partly reviewed by
Astra on `e5e0898`, was rectified and approved on `3559396` within the check's perimeter (`20`
§9). Not a field pilot and no fit run.

## 11. Candidate A2 guarantee contract

**Status:** **approved by Astra on `5e600be` as a candidate contract**, without blocking
findings; **A2 remains open**. This specifies obligations, not a demonstrated A2 guarantee
or an implementation. The subsequent proof for b in §12 is accepted on `5102852` under its hypotheses. It reuses §3’s proposition and
§9.6’s approved conditional analysis, with §10.2’s separation of objects. Neither calibration
arm is chosen. The earlier reviews, calculations and counterexamples remain as recorded;
no new numerical evidence or component policy is introduced.

### 11.1 Fixed experiment and target

Let `T = T_active ∪ T_contested` be one finite, nonempty test, fixed before observing the
check data; `m = |T|`. Every administration includes all of `T` and the required anchors.
The total score is **unweighted number correct on all items**, `S_T = Σ_{j∈T} X_j`;
active items contribute even if each previously passed its DIF cut. No search among forms,
outcome-dependent item removal or drift between check and use is covered.

The following objects must be specified before data; their values remain symbolic here:

| Object | Meaning in the candidate contract | Boundary |
|---|---|---|
| `Π` | intended population law of complete administrations of this fixed test, in a specified use context and horizon | check administrations must represent this law; independence and identical distribution are explicit assumptions for the finite-sample reference below, not consequences of completeness; a pilot population does not automatically transport to use |
| `θ` | ability coordinate shared across the target contrasts and all contributing calibrations | curves and measure must use the same alignment; informative, class-invariant anchors and the R-b interpretation are assumptions to justify, not something a successful gate proves |
| `μ` | probability reference measure on that common ability scale, used for both sides of every contrast | its relation to `Π` must be declared; under §9.6 R-d it is the population mixture of class ability laws; if unknown, its parameters belong to the uncertainty region, rather than treating fitted weights as fixed |
| `𝒢` | a fixed, nonempty family of abstract population contrasts, with conditional mean functions `s_g(θ) = E_Π[S_T \| θ,g]` defined `μ`-almost everywhere | R-c/H2 must connect these means to the representation; no personal or group attributes are collected, and the response data do not identify arbitrary `λ_g(θ)`; fitted component labels alone do not define this target |
| `α`, `δ` | error probability and, if an acceptance decision is wanted, tolerance for the whole-test target | symbolic, no new level or threshold chosen; they have no relation by default to A1’s observation design |

Use `DTF_μ(T) = sup_{g,h∈𝒢} ∫ |s_g(θ) − s_h(θ)| dμ(θ)`, with the measurability needed for
these integrals. This is §2’s maximum when `𝒢` is finite or the maximum is attained. The
support/representation assumptions must make the conditional means meaningful at the
ability values being compared; a group with no such support is not repaired by inventing
observations. Since `0 ≤ S_T ≤ m`, `0 ≤ DTF_μ(T) ≤ m`.

Three objects have distinct names and roles:

- **`D(T)`** is the current contested-item admission cost (§8), summed from fitted contributions.
  It omits active items and is neither this target nor a confidence bound for it.
- **`B_T(ϑ)`** denotes a proposed *population-model majorant* at parameters `ϑ`; its plug-in
  value `B_T(ϑ_hat)` is an estimate, not a bound with coverage. §3 supplies a possible
  majorant `Σ_F E_F(S_F;ϑ)` for a partition of all of `T`, subject to H1–H3. A singleton
  partition and a restriction of a common calibration are possibilities, not an adopted
  calibration. For ability-dependent mixtures, the pointwise envelope is needed; a maximum
  of integrated pair gaps alone has no such justification. This does not prescribe a policy
  for fitting, merging, filtering or protecting components.
- **`DTF_μ(T)`** is the population target above, on active and contested items together.
  Equality with a majorant is unnecessary; its conservatism must remain visible.

### 11.2 Candidate guarantee and three obligations

Declare a model/representation family `Ξ` and its allowed orders before the check data.
Let `ϑ* ∈ Ξ` represent the true response law and the target linkage under the hypotheses
below. Given check data `X`, a region `𝒞(X) ⊆ Ξ` and a reported upper bound `U(X)` would
support the candidate **finite-sample, uniform** guarantee

`inf_{ϑ*∈Ξ} P_{ϑ*}[DTF_μ(T) ≤ U(X)] ≥ 1 − α`,

where the infimum is restricted to laws/target linkages satisfying the declared experiment
and representation assumptions. The randomness is the check sample (and any declared
independent construction randomness); `T`, the population target and contrasts are fixed.
Under the R-a reference family, the region must allow orders 1–4 and the admitted
shared/per-class slope alternatives, irrespective of which candidates BIC visits or selects.
That does not validate R-a or extend it to a truth outside its model restrictions.
An asymptotic or pointwise result is a different, weaker claim and must be labelled as such;
it cannot silently satisfy this finite-sample contract. No existing fit realizes it.

| Obligation | Required statement | What must be justified |
|---|---|---|
| **a. Representation** | `DTF_μ(T) ≤ B_T(ϑ*)` throughout the declared target/model family | §3 H1–H3 or another explicit domination proof: common scale and population measure, all active/contested items, conditional group means represented by admitted mixtures including every component needed for those means; R-a–R-f delimit the assumptions, none established by a fit |
| **b. Statistical coverage** | `inf_{ϑ*∈Ξ} P_{ϑ*}[ϑ* ∈ 𝒞(X)] ≥ 1 − α` | the true allowed order and all parameters used by the response law or majorant: class weights/shifts, histogram shape, item/anchor curves, shared floors, shared/per-class slope alternatives where admitted, uncertain linking and `μ`; weak identification, rare components and coincident representations cannot be removed by selection without justification |
| **c. Conservative calculation** | `U(X) ≥ sup_{ϑ∈𝒞(X)} B_T(ϑ)` whenever a nontrivial certificate is returned | an upper bound on the global supremum, including integration, truncation and numerical errors; feasible values or locally optimized values give lower bounds on the supremum, not the required upper bound |

Obligation a holds at the true representing parameter; b places it in the region with the
stated probability; c dominates its majorant on that event. These implications are the
reference logic already acquired in §9.6, not a proof of any of the three obligations.
Coverage does not require unique identification of `ϑ*`: label permutations/common coordinate
changes can be quotiented only while preserving the law, target linkage and majorant.
Nor does coverage mean every observationally equivalent parameter must be in every realized
region; it requires the uniform statement for each admitted truth, without an unjustified
choice of representation excluding truths relevant to the bound.

If deciding on `U ≤ δ`, this would control
`P_{ϑ*}(certificate returned, U ≤ δ, DTF_μ(T) > δ) ≤ α` under the same hypotheses.
The opposite decision does **not** prove excessive DTF. A bound is not a guarantee outside
`Π`, `μ`, `𝒢`, the fixed form or the stated model. Calibration data and anchor admission
must enter the sampling/coverage argument; ignoring their selection or conditioning on a
successful gate does not inherit coverage automatically. No missing administration is
silently discarded to claim the complete-data experiment.

### 11.3 Failure, usefulness and cost

A valid procedure must define its outputs even when an order is unresolved, an optimization
fails or a sample is uninformative. It may return the **non-informative upper bound `m`**,
valid directly from the score range without a fit. That is a trivial target bound, explicitly
labelled, not a successful calculation of the proposed supremum. Alternatively it returns
**no certificate**, with the failed obligation or unsupported input identified. For probability/
error bookkeeping only, set `U = +∞` on a no-certificate output; this is not a numerical
certificate. An empty numerical region, unsupported model or indeterminate computation never
implies DTF zero or acceptance. If the target population/scale/contrasts themselves are
unspecified, no claimed nontrivial certificate has a defined meaning.

**Validity** is a–c and the stated error control. **Useful precision** is whether a valid bound
can resolve the symbolic tolerance on the intended population; it may fail despite validity.
**Cost** is the administrations, answer burden, model search and certified optimization needed
for that precision. It is not measured here and remains subject to D2/D4. Bootstrap spread,
convergence and agreement across fits can be diagnostics (§10.2), not coverage or identification
proofs. No sample size, resource commitment or operational campaign follows from this contract.

### 11.4 Acquired obstacles applied to the contract

The results are those of §9.6; this table states their consequences without re-running them.

| Acquired obstacle | Consequence for this candidate | What would discharge the affected obligation |
|---|---|---|
| identical component duplication and the per-component 5% filter | the current filtered functional can change with the representation while the response law stays fixed; it cannot simply be named the target/majorant | a representation-compatible domination argument; the acquired unfiltered envelope’s invariance under exact duplication is reusable, but adopts no replacement component policy |
| rare/addable components | under §9.6’s uniform-coverage, all-components-counted family with fewer than four true classes and an admissible addable component, the acquired lower bound on `U` can preclude useful acceptance; this is not a universal result about actual contrasts or every four-class truth | retain those hypotheses when invoking the result; distinguish a valid but wide bound from a false coverage claim; no share floor or minimum separation is inferred |
| BIC order selection and penalized fitting | a region restricted to the selected order can miss the truth; selection/convergence gives no coverage | coverage over the full declared order/parameter family, or a separately proved selection-aware construction; the current search’s visited/converged candidates are not that family; penalties are not automatically likelihood uncertainty |
| ability alignment, measure and support | common anchors or numerical normalization do not establish the same target ability/measure; agreement of fits does not identify them | a–b explicitly address R-b/R-d, support and H1; any linking/measure estimated from data is covered jointly or assigned a justified simultaneous error allocation |
| local optimization and integration | a converged local fit/supremum search can underestimate the required supremum | a proved global upper bound with numerical error control; otherwise a declared trivial bound or no certificate |

A2 does not require proving a unique parameterization merely to return a valid upper bound.
It does require the target linkage and a–c; a stability experiment cannot replace them.
The comparison of common and per-test calibration in §9 stays acquired and undecided.

### 11.5 Acceptance and smallest next deliverable

**Reusable:** §3’s conditional domination/subadditivity and envelope distinction; R1’s corrected
meaning of `D(T)`; §9.6’s separation of representation, identification, inference and computation,
its coordinate/duplication observations and scoped rare-class/order-selection results; §10.2’s
diagnostic boundaries. No approved result or check is reopened.

**Still to demonstrate:** a for the declared representation/target; b for one explicit region
construction on the full declared family; c for an actual conservative bound, including
integration. Useful width and affordable execution are further questions, not substitutions
for those proofs. No one of a–c is supplied by the current APIs.

**Owner decisions genuinely needed before an operational claim:** the intended population/use
context, ability interpretation, reference measure and abstract contrasts, then the tolerated
error/DTF and resource envelope if a useful acceptance procedure is wanted. The contract can
be reviewed and conditional mathematics developed with those objects symbolic; no immediate
choice of calibration, numerical level, sample size or component policy is necessary for that
work. A change of target to evade the rare-component obstruction would need a distinct explicit
decision; none is proposed as already taken.

Verifiable acceptance criteria for a future realization are:

| Deliverable | Acceptance evidence |
|---|---|
| target and representation statement | fixed `T`, score, `Π`, scale, `μ`, `𝒢` and allowed model family specified; a proved for all included active/contested items; assumptions labelled separately from realized properties |
| region construction | exact statistic, sampling assumptions and allowed orders/nuisance parameters stated; proof of b with the claimed uniform finite-sample scope, including the irregular cases it permits; diagnostic/bootstrap evidence alone fails this criterion |
| conservative bound | specification and proof of c, certified numerical error treatment, and explicit trivial/no-certificate outputs; a local optimum or agreement across starts fails this criterion |
| decision and scope | false-acceptance statement derived from a–c; failed gates/computations cannot accept; useful-width and cost evidence distinguished from validity; no certification of another population, form search or current runtime |

**Historical next-deliverable recommendation on `5e600be`, now addressed by §12.**

**Single recommended next deliverable, after review of this contract:** a short mathematical
specification and coverage proof for **one all-order uncertainty region** for the fixed-form,
complete-administration reference, with `α` symbolic. The split-likelihood route already named
in §9.6 is the recommended candidate: state its training/check separation, the response
likelihood and the union of allowed orders, and address all nuisance parameters used by the
majorant; do not substitute the penalized objective for a likelihood. Acceptance is the exact
uniform finite-sample statement b, with its assumptions and excluded laws explicit. This is
a coverage deliverable, not a choice of calibration or an assertion that the region is narrow
or its supremum computable. Representation and conservative computation remain separate open
obligations, and model/sampling premises remain conditional until justified for use. No code,
fit, campaign or resource authorization is required by this recommended next step.

## 12. Split-likelihood region and finite-sample coverage

**Status:** mathematical specification and proof (accepted, below). This supplies a construction
for §11’s obligation **b under the assumptions below**, not a realized A2 guarantee. §11 was
approved as a candidate contract on `5e600be`; obligations a and c, useful precision and cost
remain separate. No implementation, calibration arm or operational parameter is chosen.
Reviewed by Claude Code in §13; Astra accepted the coverage proof on `5102852`, conditional on
the model, sampling and split hypotheses stated below, not as an implementation's numerical
correctness. The region-inclusion precision of §12.3 is rectified after that review and accepted
on `2aa136d`; §12.4 and §12.5 point to §13.

### 12.1 Observable family and target-linkage parameters

Keep §11.1’s fixed `T`, population `Π`, complete administrations, fixed compatible anchor
columns `H`, and `α ∈ (0,1)` symbolic. One observation is the correctness vector
`Z = (X_j : j∈J) ∈ 𝒵 = {0,1}^d`, where `J` contains all active/contested test and anchor
columns, each once, and `d = |J|`. Keys, formats and column order are fixed before data;
missing or duplicate response columns are not complete administrations of this experiment.
No personal or group attributes are observed. Assume `Z_1,…,Z_n` are i.i.d. from the law
below and that this law is the declared population’s, with no transport or drift inferred.

For the R-a/D37–D43 reference, let `β` range over **every order `k = 1,…,4`** and both
admitted alternatives for item slopes (shared or per class), including coincident orders.
For each order, `β` includes positive mixing weights `π_r` summing to one, shifts `η_r`
(`η_1 = 0` as coordinates), the common histogram shape, item and anchor slopes/difficulties,
and the format-dependent floors. No minimum share is imposed: positive shares can be
arbitrarily small. These are the finite-logit weights of `Model::pi` and `Grid::shape`,
not the 5% filtering of a fitted DTF.

More explicitly, for a finite nondegenerate base grid `v_q`, `q=1,…,Q`, `Q≥2`, fixed before data,
the shape has
positive weights `w_q` summing to one, `u_q = (v_q − Σ_t w_t v_t)/s_w`, where
`s_w² = Σ_q w_q(v_q − Σ_t w_t v_t)² > 0`. Then
`ν_{r,β} = Σ_q w_q δ_{η_r+u_q}` is a probability ability law. Grid size/range stay symbolic;
the normal-shape API option is the specified fixed-weight subcase, not permission to omit
the free shape when it is admitted. With `σ(x) = 1/(1+exp(−x))`, the response curve is
`p_{jr,β}(t) = c_j + (1 − c_j) σ(a_{jr}(t − b_{jr}))`:
anchor `a,b` are class-invariant; item `b` may vary by class, and `a` follows either admitted
alternative; `c_j = 0` for open formats and `0 < c_j < 1` for choice formats, shared across
classes. Finite curve/floor-logit parameters and a valid shape define this reference family;
no prior, penalty or admission threshold is a likelihood factor.

With `b(x;p) = p` if `x=1` and `1−p` if `x=0`, local independence conditional on class and
ability gives the **marginal observable probability**

`f_β(z) = Σ_{r=1}^k π_r ∫ ∏_{j∈J} b(z_j;p_{jr,β}(t)) dν_{r,β}(t)`
`          = Σ_r π_r Σ_q w_q ∏_{j∈J} b(z_j;p_{jr,β}(η_r+u_q))`.

Both latent variables are integrated out, not replaced by estimated respondent abilities or
hard class labels. Summing over `z∈𝒵` gives `Σ_r π_r Σ_q w_q = 1`, because each product of
Bernoulli probabilities sums to one. This is the mathematical law behind `latent::evaluate`,
`Model` and `Grid::shape`; it is not the penalized objective or a claim of exact numerical
likelihood evaluation by the APIs. Specifying a law on the fixed columns adopts neither
common nor per-test calibration.

Write the full family as `Ξ = {(β,γ): β∈⋃_{k=1,…,4; s∈{shared,per-class}} ℬ_{k,s}, γ∈Γ_β}`.
Here `γ` is the **target linkage**: any unknown scale maps, reference measure and conditional
contrast mixtures needed for §11’s target, subject to the target-linkage hypotheses explicitly
declared for that experiment (§11, H1–H3/R-b–R-d where retained). R-e/R-f locate filter/support
boundaries, not implicit restrictions on `Γ_β`. The family is fixed before data, not selected
by favourable responses or restricted by the fitted 5% filter.
The contrasts `𝒢` remain the fixed abstract family, not data-dependent labels. The law
`f_(β,γ) = f_β` does not observe arbitrary `λ_g(θ)` or external scale/measure linkages.
Those parameters must remain free within `Γ_β`. If R-d makes `μ = Σ_r π_r ν_{r,β}`, it varies
with `β` throughout the region; it is not an independent free measure or a fitted value held
fixed. An externally prescribed `μ` is fixed by that prescription; an unknown external
linkage is not estimated by this likelihood. No linkage, support condition or actual target
representation is proved merely by this definition; an operational target still needs §11.5’s
declarations. The coverage argument covers every admitted pair, whether identifiable or not.

### 12.2 Independent split and normalized training prediction

Before responses, fix disjoint index sets `I_tr, I_v` partitioning the observations, with
symbolic positive sizes `n_tr, n_v`. A randomized partition must be independent of responses;
include it in the conditioning. Let `𝒜` be the training data, partition and any fixed or
independent training seed. Validation responses retain product law
`L_β(z_v) = ∏_{i∈I_v} f_β(z_i)` conditionally on `𝒜`.

Using **only** `𝒜`, construct a measurable normalized probability mass function `q_𝒜` on
`𝒵`; for example `q_𝒜 = f_{β_hat(𝒜)}` for any measurable valid fitted parameter. Define
`Q_𝒜(z_v) = ∏_{i∈I_v} q_𝒜(z_i)`, so `Σ_{z_v} Q_𝒜(z_v) = 1`. A deterministic algorithm
with specified tie-breaking supplies measurability on the finite training alphabet. The
proof requires this normalization, measurability and independence from validation data;
it requires **neither correct BIC selection, global optimality, convergence nor consistency**.
A penalized/local estimator can supply a valid prediction; this mathematical sufficiency
changes no existing API refusal/status contract. Invalid parameters or numerical
outputs do not supply a normalized law merely because a fit returned them.

Choose the **no-certificate convention** for training failure in this mathematical
specification. Failure is `𝒜`-measurable, including inability to supply a valid `q_𝒜`.
On that branch set the bookkeeping region to `Ξ` and return no certificate (`U=+∞` for
§11.3’s probability accounting); no acceptance is possible. There is no deletion of failed
training samples or repetition of the experiment until one succeeds. This convention prescribes no production
retry/resource policy.

### 12.3 Ratio, support and proof

On a successful training branch, for **each** `ϑ=(β,γ)∈Ξ`, define
`R_ϑ(z_v;𝒜) = Q_𝒜(z_v)/L_β(z_v)` when `L_β(z_v)>0`;
if `L_β=0<Q_𝒜`, set `R_ϑ=+∞`; if both are zero, set `R_ϑ=0`.
The region is `𝒞(𝒜,z_v) = {ϑ∈Ξ: R_ϑ(z_v;𝒜) ≤ 1/α}`.
Equality is included; rejection uses `R_ϑ>1/α`. Only the **numerator** may use the training
model selector. Every order, slope alternative and admitted linkage remains in the denominator
family; no BIC winner, visited-candidate list or 5% filter restricts this inversion.

Finite parameters in §12.1 give `0<p_{jr,β}(t)<1` at each grid point, hence positive probability
for every response vector. There is no uniform lower bound away from zero across parameters.
The proof below does **not** rely on full support: it also applies if a separately declared
normalized family admits structural zeros. No boundary extension of the reference family is
adopted here. Denominator-zero events have probability zero under that candidate’s own law;
the conventions specify the region even there, not evidence that impossible data occurred
under the truth. Floating-point underflow is not a structural zero and does not justify
these conventions in a future implementation; exact likelihoods or conservative numerical
error treatment would be required. Without ratios the region is
`𝒞 = {ϑ∈Ξ: α Q_𝒜(z_v) ≤ L_β(z_v)}`, the same set under the conventions above (both zero: kept;
`L_β = 0 < Q_𝒜`: rejected). Two statements about a computed region are distinct (rectified after
the review of `5102852`, accepted on `2aa136d`):

- *Inclusion of this region.* A computed numerator `Q̃ ≤ Q_𝒜(z_v)`, below this specific `Q_𝒜` at
  the observed `z_v`, and a computed denominator `L̃_β ≥ L_β(z_v)` for every `β` tested give
  `{ϑ: α Q̃ ≤ L̃_β} ⊇ 𝒞`: the numerator rounded down, the denominator up. Over a continuum of
  `β` this belongs to obligation c.
- *Another region.* A different `𝒜`-measurable sub-probability `q'` on `𝒵`, `Q'` its product
  over the validation set (for instance the evaluated values themselves, once their total mass
  is shown at most 1), defines its own region `{ϑ: α Q'(z_v) ≤ L_β(z_v)}` with its own coverage
  by the proof below. It need not contain `𝒞`, and no statement about `𝒞` transfers to it.

**Proposition (conditional and uniform finite-sample coverage).** For every admitted truth
`ϑ*=(β*,γ*)∈Ξ`,
`P_{ϑ*}(ϑ*∈𝒞 | 𝒜) ≥ 1−α` almost surely, and consequently
`inf_{ϑ*∈Ξ} P_{ϑ*}(ϑ*∈𝒞) ≥ 1−α`.

**Proof.** Fix a training realization on the successful branch. Under the truth, validation
has law `L_{β*}` independently of that realization. For the nonnegative ratio,

`E_{ϑ*}[R_{ϑ*} | 𝒜] = Σ_{z_v:L_{β*}(z_v)>0} L_{β*}(z_v) Q_𝒜(z_v)/L_{β*}(z_v)`
`                          = Σ_{z_v:L_{β*}(z_v)>0} Q_𝒜(z_v) ≤ 1`.

Zero-probability validation vectors contribute zero to the expectation even if the assigned
ratio there is infinite; no `0·∞` arithmetic is needed. Equality holds when `Q_𝒜` places
no mass outside the truth’s support. Conditional Markov’s inequality therefore gives
`P_{ϑ*}(R_{ϑ*}>1/α | 𝒜) ≤ α E_{ϑ*}[R_{ϑ*}|𝒜] ≤ α`.
This rejection event is exactly `ϑ*∉𝒞`. On the training-failure branch `𝒞=Ξ`, so the
conditional noncoverage probability is zero. Averaging these conditional inequalities gives
`P_{ϑ*}(ϑ*∉𝒞) ≤ α P_{ϑ*}(training succeeds) ≤ α`. The proof applies to every admitted truth
with the same constant `α`; it uses no lower share bound, identification or asymptotics, hence
the stated uniform bound. It is conditional on training for each truth, not a posterior claim
or a simultaneous confidence statement about all parameters. ∎

### 12.4 What the coverage supplies, and what it does not

The region includes **whole linkage fibres** `Γ_β` when `β` passes; it cannot fix unobserved
linkages to fitted values. Representations with identical observable laws have identical
ratios and are retained/rejected together, regardless of their order or labels. Rare components,
coincident orders and weak identification do not invalidate the expectation calculation.
They do make the bound useless in §13.3's domain — a one-class truth, components of arbitrarily
small share admitted, a linkage allowing their contrast: there `U ≥ ½ Σ_{j∈T} (1 − c_j)` with
probability at least `1 − α`, whatever the training rule (accepted on `5102852` in that domain;
§9.6's rare-class result made exact for this region).

| Layer | Result / remaining boundary |
|---|---|
| region coverage, §11 b | proved above for the exact mathematical region and declared i.i.d. reference, accepted on `5102852` under these hypotheses; no implementation certified |
| target domination, §11 a | still needs a declared target linkage and `DTF_μ(T) ≤ B_T(β,γ)` on that family; §3’s conditional result is reusable, not a property of today’s filtered functional |
| conservative supremum, §11 c | still needs a certified upper bound on `sup_{ϑ∈𝒞} B_T(ϑ)`, including likelihood-region evaluation and integration errors; no local fit or convergence certificate suffices |
| useful precision and cost | not established by coverage; a trivial `m` bound or no certificate remains possible, with no sample/resource commitment |

No coverage **conditional on passing an anchor gate** follows. If a gate uses only `𝒜`, the
conditional statement can survive conditioning on that training-only event, under this same
experiment; it does not authorize a changed population or training response law for validation.
A gate using validation or pooled data has no such implication. Returning no certificate on
its failure can only reduce the *unconditional* false-certification event for this one fixed
check, but conditioning on success, discarding/replacing respondents, refitting the numerator
on validation, or reusing that set for a favourable split does not inherit the proof. Previously
seen data are eligible only if the stated training/validation independence and data-use
restrictions actually hold. Test search, sequential collection/stopping, multiple splits,
updates and retries for another chance of acceptance need their own error argument (§9.5);
none is implicitly covered, and no new procedure is designed here.

### 12.5 Exact result and one next deliverable

**Demonstrated:** an exact split-likelihood region with conditional-on-training and uniform
finite-sample coverage `1−α`, under the declared response-law family, i.i.d. complete
administrations of one fixed test with fixed anchors, an independent split, and a measurable
normalized training-only prediction (or the no-certificate failure branch). The proof has no
identification, BIC-optimality, asymptotic or positive-support prerequisite beyond those stated.
It does not validate R-a for actual responses, identify the target linkage, certify a computed
supremum, guarantee useful width/affordable execution, or close A2. **The proof is accepted on
`5102852` under these hypotheses; A2 and Phase 1 remain open.**

**Single next deliverable recommended after review:** a compact **target-linkage and majorant
specification on this `Ξ`**: state `Γ_β` for the symbolic `μ`, scale and contrasts, which parts
are determined by `β` and which remain free, and instantiate §3’s acquired domination result
for all active/contested items without relying on the fitted filter or BIC order. Acceptance:
`DTF_μ(T) ≤ B_T(β,γ)` for every admitted pair with support/measure/alignment hypotheses explicit,
and no plug-in collapse of unidentified linkages. This is obligation a; it reuses the acquired
proof instead of repeating its examples, and adopts no calibration/component policy. The
certified supremum, useful precision and cost remain subsequent distinct obligations.

**Replaced (§13.7):** in §13.3's domain this deliverable would specify a valid majorant whose
certificate is useless. §13.5's check, whose contract was accepted on `2aa136d`, is run in §13.8;
its result is accepted on `24cdfce` as an obstacle in its domain, and the owner defers A2's
design decision (§13.9). This deliverable is not started.

## 13. Review of §12 and the first feasibility contract

**Status:** written by Claude Code at `a0e30c7` (`5102852`). Reviewed by Astra on `5102852` (record
before §1): §12's coverage proof and §13.3's corollary are **accepted in their declared domains**;
the five rectifications that review asked for are made here and in §12.3 and, with §13.5's
contract, **accepted on `2aa136d`** (record before §1). §13.5's check, run in §13.8, is
**accepted on `24cdfce`** as an obstacle in its declared domain; the owner defers A2's design
decision (§13.9). A2 stays open and its certifier suspended. Evidence at
`5102852`: **L** `latent.rs` (`Model`, `Grid::{new, shape}`, `Floors`, `Data::new`, `evaluate`),
`dtf::DTF_MAX`, `revalidation::N_LATENT_MIN`, `pilot::{N1_MIN, N2_MIN}`,
`characterization::generate::dif_batch`; **D** the derivations below; **C** a scratch script, not
committed: §12.3's identity on a toy family in exact rationals (for every training outcome
`E[R | 𝒜] ≤ 1` and conditional non-coverage `≤ α E[R | 𝒜]`; at `α = 1/4` the largest conditional
non-coverage is 16/225, the unconditional 203/16875), §13.3's domination step in exact rationals,
and §13.5's product formula against exhaustive enumeration of the patterns on 200 random toys, in
floating point (agreement within `10⁻¹²` relative; the `KL`, computed in floating point by
exhaustive enumeration with ability marginalized and anchors included, at most 0.46 of the bound).
Evidence of the rectifications: **C** exact rationals, a second scratch script, not committed: on
72 idealized cases (`m` from 1 to 8, three `p`, three `d`) the definition, the product form and
`Σ_{k even ≥ 0} C(m, k) e^k` agree for `1 + χ²`, the sum from `k = 2` is `χ²`, and
`C(m, 2) e² = (m − 1) D⁴/(32 m³ p² (1 − p)²)` with `D = 2md`; on two cases with item-specific
`p_j`, `d_j` the product form equals the definition. No Rust, fit or campaign was run.

### 13.1 The six layers

| Layer | What §12 supplies | What remains |
|---|---|---|
| representation and common target | a declared response-law family `Ξ` with symbolic linkage `Γ_β` | `Γ_β`, `μ`, scale and contrasts; the linkage decides usefulness (§13.3) |
| statistical coverage (b) | exact, conditional on training, uniform over `Ξ`; accepted on `5102852` under its hypotheses | the premises: R-a, i.i.d. complete administrations, an independent split |
| domination (a) | nothing | `DTF_μ(T) ≤ B_T(β, γ)` on the admitted pairs; §3 reusable |
| conservative computation (c) | the requirement; the region's inclusion needs `Q̃ ≤ Q_𝒜` and `L̃_β ≥ L_β` (§12.3) | outer enclosure of `𝒞`, global supremum, numerical error included |
| usefulness | nothing | negative in §13.3's domain (accepted); in §13.5's domain an obstacle established under its numerical contract (§13.8, accepted on `24cdfce`); otherwise open; feasibility never shown |
| cost | nothing | unmeasured (`15` D2, D4) |

### 13.2 Verification of b and the corrections made

- **Proof.** Correct. Given `𝒜`, validation has law `L_{β*}`; `E[R_{ϑ*} | 𝒜]` is the numerator's
  mass on the truth's support, at most 1; conditional Markov bounds `P(R_{ϑ*} > 1/α | 𝒜)` by `α`;
  the failure branch has `𝒞 = Ξ`. The law does not depend on `γ`, so coverage of `β*` is coverage
  of every `(β*, γ)`. What it uses: the truth in `Ξ` (R-a), i.i.d. complete administrations from
  the declared population, a split independent of the responses, an `𝒜`-measurable
  sub-normalized numerator. Assumptions, not properties of any data.
- **Correspondence with the code** (L). `evaluate` computes `ln f_β`: per class and node,
  `ln π_g + ln w_q` plus each column's Bernoulli log-probability; nodes standardized by
  `Grid::shape` and shifted by `η_g` (`η_0 = 0`); anchors class-invariant; item `a` shared or per
  class, `b` per class; one floor `σ(γ)` per choice column shared by the classes, none for an open
  one; `nodes` equispaced points on `[−θ_max, θ_max]`. No mismatch. Two precisions: slopes are
  unconstrained reals in `Model`, and `Data::new` reads a response `≥ 0.5` as correct, so `Z` is
  that dichotomization. Neither affects the proof. A reading of the code, not a check of its
  numerical correctness.
- **Corrections in §12.** At `5102852`: §12.4's understatement replaced by §13.3; §12.5's next
  deliverable replaced (§13.7). Rectified after that review and accepted on `2aa136d`: §12.3's
  numerical remark, which conflated two statements. The inclusion of `𝒞` needs a numerator below
  this specific `Q_𝒜` and a denominator above `L_β`, point by point; another training-measurable
  sub-probability gives its own region and coverage, not that inclusion.

**Judgment on §12:** accepted as the proof of b under its hypotheses (`5102852`). Building a and c
next, as §12.5 recommended, needs a decision on the claim first (§§13.3–13.4).

### 13.3 Exact consequence for usefulness (D, accepted on `5102852` in its domain)

**Corollary.** Let the truth `β* ∈ Ξ` have one class, `ν` its ability law; let `Ξ` admit components
of arbitrarily small share, as §12.1's does, and the linkage allow a contrast between two
components, as §12.1's free `Γ_β` does. If `B_T` satisfies a and `U` satisfies c on §12's region
(`U = +∞` on the failure branch), then for every training rule
`P_{β*}(U ≥ ½ Σ_{j∈T} (1 − c_j)) ≥ 1 − α`. The certificate therefore accepts at any
`δ < ½ Σ_{j∈T} (1 − c_j)` with probability at most `α` under every such truth, DIF-free ones
included; at `DTF_MAX = 0.10`, for every form holding one item with `c_j < 0.8`.

*Proof.* Add a class `x` of share `ε`, `η_x = 0`, the same shape, anchors and floors, slopes `1`
(per-class alternative), difficulties `−M` (or `+M`): `Q_ε ∈ ℬ_{2,per-class} ⊂ Ξ` and
`f_{Q_ε} = (1 − ε) f_{β*} + ε f_x ≥ (1 − ε) f_{β*}`, so `R_{Q_ε} ≤ (1 − ε)^{−n_v} R_{β*}`. On
`{R_{β*} < 1/α}`, of conditional probability at least `1 − α`, `Q_ε ∈ 𝒞` for every small `ε`,
whatever `M`. Both classes of `Q_ε` have ability law `ν`, which is `μ` under R-d, so their contrast
has conditional means wherever `μ` has mass, without extrapolation:
`∫ |Σ_{j∈T} (p_{jx} − p_{j0})| dν`. As `M → ∞` it tends to `∫ Σ_j (1 − p_{j0}) dν` (`−M`) or to
`∫ Σ_j (p_{j0} − c_j) dν` (`+M`), whose sum is `Σ_j (1 − c_j)`. By a and c, `U` is at least the
larger. ∎

Scope. It is §9.6's rare-class result made exact for this region, at every `n` and with no
total-variation slack; neither c nor a better numerator repairs it, since §9.6's form holds for
any uniformly valid bound. The acceptance covers one-class truths only: the remark that the
construction extends to two or three true classes when curves are compared where a class carries
no mass (R-f), as `ClassCurves::dtf` and §3's envelope do, is not part of it. It is avoided only
by changing the claim, for instance contrasts restricted to groups present at some share at every
ability (the added class is present at `ε`), or a share floor on the components of `Ξ` (an
assumption on the truth). Each is an owner decision (§11.5); none is taken here. Neither avoids
§13.4's balanced alternative, whose two classes are present at ½ at every ability (§13.8).

### 13.4 Why a floor need not suffice

A two-class alternative with shares ½, one ability law and difficulties `b_j − τ`, `b_j + τ` has
both classes present at ½ at every ability: it survives any share or presence floor up to ½, and
its contrast needs no extrapolation. Anchors being class-invariant, its classes differ only in the
items' joint law given ability. Per node, with `I^±` and `I` the item product laws and
item-specific probabilities, the general product form is
`1 + χ²(½ I⁺ + ½ I⁻ ‖ I) = ¼ ∏_j s_j^{++} + ½ ∏_j s_j^{+−} + ¼ ∏_j s_j^{−−}`, with
`s_j^{xy} = p_j^x p_j^y / p_j + (1 − p_j^x)(1 − p_j^y)/(1 − p_j)` (D; checked, §13 status). In the
idealized case of one base probability `p` common to every item and common perturbations `p ± d`,
`s^{++} = s^{−−} = 1 + e` and `s^{+−} = 1 − e` with `e = d²/(p(1 − p))`, so
`1 + χ² = ½ [(1 + e)^m + (1 − e)^m] = Σ_{k even ≥ 0} C(m, k) e^k` and
`χ² = Σ_{k even ≥ 2} C(m, k) e^k`. With the contrast `D = 2md`, the leading term as `d → 0` is
`C(m, 2) e² = (m − 1) D⁴/(32 m³ p² (1 − p)²)`: asymptotically of order `D⁴/m²` per node. That order
is asymptotic and idealized — logistic curves are not `p ± d`, and their base probabilities differ
by item and node — a heuristic expectation, not a result. If the information is that small at the
sizes of reference, no certifier valid on a family containing these alternatives is useful there,
§12's or any other, and no share or presence floor up to ½ restores usefulness in that domain. The
check cannot show the converse: a floor's sufficiency would need its own argument. This is why it
is proposed before a and c.

### 13.5 Contract of the first feasibility result (rectified after `5102852`, accepted on `2aa136d`)

| Item | Content |
|---|---|
| Question | Can a certifier valid at level `α` on a family containing a DIF-free one-class truth `P_0` and the alternatives `Q_τ` below accept `P_0`'s form with probability `1 − β` at the sizes of reference? The result is a necessary condition: it can establish an obstacle in its domain, never feasibility. |
| Domain, declared before computing | Shape: the code's fixed normal subcase (`Grid::new(41, 5.0)`, `normal_logits`, standardized by `Grid::shape`), as a mathematical object, not its floating-point values in the code. Forms: `m ∈ {4, 8, 16}` items (`08` SC-2's `K` range); `a_j = 1.25`, `b_j = 0.6 Φ⁻¹((j − ½)/m)`, the centre and quantiles of `dif_batch`'s laws `U(1, 1.5)` and `0.6 N(0, 1)`; floors all 0 (open), all ¼ or all ½ (the priors' modes for four and two options). `P_0`: one class with those items, any anchors. `Q_τ`: shares ½, `η = (0, 0)`, `P_0`'s shape, anchors and floors, shared slopes, difficulties `b_j − τ` and `b_j + τ`, in `ℬ_{2,shared} ⊂ Ξ`, its two classes an admitted contrast. `δ = DTF_MAX = 0.10`, the existing provisional value, with `2δ` and `4δ` as sensitivity only. Sizes `n ∈ {N1_MIN, N2_MIN, N_LATENT_MIN, 6,000}` (300, 1,500, 3,000 and SC-2's largest), every respondent the procedure uses. |
| Input | These declarations; no data, fit or simulated respondent. |
| Computation | `D(τ) = Σ_q w_q Σ_j (p_j(u_q; b_j − τ) − p_j(u_q; b_j + τ))`, the two classes' contrast, increasing in `τ`; `τ_δ` with `D(τ_δ) > δ` established under the numerical contract; `K̄`, an upper bound of `K = Σ_q w_q ln(1 + χ²_q(τ_δ))` by §13.4's product form; `Δ̄_n = min{1, (1 − e^{−nK̄})^{1/2}, (nK̄/2)^{1/2}}`. |
| Statement used (D) | A procedure valid at `α` on a family containing `Q_τ`, `D(τ) > δ`, accepts `Q_τ`'s form at `δ` with probability at most `α`, hence `P_0`'s with probability at most `α + TV(P_0^n, Q_τ^n)`. `KL(Q_τ ‖ P_0) ≤ K` by joint convexity over the shared nodes and `KL ≤ ln(1 + χ²)`; Bretagnolle–Huber and Pinsker then give `TV ≤ Δ_n ≤ Δ̄_n`. `Δ_n` is an upper bound on the total-variation distance, not an advantage any procedure attains. Usefulness at `(α, β)` needs `1 − α − β ≤ Δ_n`. |
| Frontier conditions | `α + β ≥ 1`: the condition is void and constrains no `n`; such pairs are attained without data (accept with probability `α`), so they are not usefulness. `α + β < 1`, `K > 0`: squaring is legitimate and `n ≥ max{2(1 − α − β)², ln(1/(1 − (1 − α − β)²))}/K`; at `α + β = 0` no finite `n` suffices. `K = 0`: the two laws coincide, `Δ_n = 0`, and every `α + β < 1` is excluded at every `n`. With `K̄` in place of `K`, frontier and sizes are weaker but valid. |
| Numerical contract | An obstacle is claimed only with: a certified lower bound of `D(τ_δ)` above `δ`; a certified upper bound `K̄` at that same `τ_δ`; `Δ̄_n` and the sizes computed from `K̄` with outward rounding (each is monotone in `K`); method, precision and enclosure widths recorded. Exhaustive enumeration of the finite pattern space makes a sum complete, not its terms exact: floating-point values are not certified unless their arithmetic is exact or carries proved error bounds. Values without error control are a numerical indication, never an obstacle demonstrated. Specified, not implemented. |
| Expected result | Per form and size: `τ_δ`, `K̄`, `Δ̄_n` and the frontier `1 − Δ̄_n`; per form, the sample-size bound; each labelled demonstrated, under the numerical contract, or indicative. No level is chosen: `α`, `β` stay symbolic. |
| Control reference | The product form against exhaustive enumeration at every node of every form (at most `2¹⁶` patterns); `KL(Q_τ ‖ P_0)` by exhaustive enumeration, ability marginalized, with no anchors and with a declared small anchor set, below `K̄`; the idealized expansion of §13.4 as an order check only; optionally, one `D(τ)` against `ClassCurves::dtf` (E API). In floating point these are consistency checks, not certification. |
| Usefulness criterion | §11.3: useful at `(n, α, β)` when valid at `α` and accepting `P_0`'s form with probability at least `1 − β`. A procedure that always returns the trivial bound or no certificate fails it. |
| Outcomes | **Obstacle**, under the numerical contract only: every `(α, β)` with `α + β < 1 − Δ̄_n` is excluded at that size, for the forms, `δ`, sizes and alternative family covered and no other; no construction, §12's included, is useful there, and no floor up to ½ restores it there. **Numerical indication**: the same reading without the contract. **Not excluded**, where `(α, β)` lies above the frontier: feasibility stays open and no floor is shown sufficient; a later result would concern usefulness (§12's own acceptance probability), which does not discharge c, the conservative computation of the supremum with its numerical error. **Decision needed**: when the reading changes across forms, sizes or `δ`, the owner fixes `(α, β)`, the tolerance or the sizes with the computed trade-off; or when the numerical contract does not fit the resource limit. |
| Resource limit | One standard-library Python script in `sim/`, deterministic, under a minute on a laptop; Rust only for the optional API check; no fit, campaign or data. A failed control stops the work and is reported, with no workaround. If the certification does not fit this limit, the outcome is "decision needed" and the work is not extended. |
| Not covered | Other alternatives (per-class slopes, impact, sparse DIF) and the strongest one: the declared `Q_τ` gives a valid, possibly weaker bound. Misspecification, search over forms (§9.5), the linkage decision, §12's power. |

The obstacle, if found, concerns latent-class contrasts under invariant 1; no group attribute may
be collected to escape it.

### 13.6 A2 obligations, satisfied and missing

| Obligation | Criterion | State |
|---|---|---|
| diagnosis; declared claims rectified | `15` A2; §8 | acquired (`0519626`, R1 on `d13bf09`) |
| target measure, linking assumptions, whole-test contribution | `15` A2; §11.5 row 1 | symbolic objects acquired as a contract (§11.1); `Γ_β` unspecified, and its choice decides usefulness (§13.3) |
| a, domination | §11.2 a; §11.5 row 1 | missing; §3's conditional proposition reusable |
| b, coverage | §11.2 b; §11.5 row 2 | proof accepted on `5102852`, conditional on its model, sampling and split hypotheses; no implementation's numerical correctness |
| c, conservative computation | §11.2 c; §11.5 row 3 | missing; the region's inclusion needs the numerator below `Q_𝒜` and the denominator above `L_β`, point by point (§12.3) |
| estimation error kept apart | `15` A2 | carried by b and c once realized; missing |
| usefulness | §11.3 | negative in §13.3's domain (accepted on `5102852`); in §13.5's domain an obstacle established under its numerical contract (§13.8, accepted on `24cdfce`); otherwise open, with neither feasibility nor a floor's sufficiency shown |
| cost | §11.3; `15` D2, D4 | not assessed |
| decision and scope | §11.5 row 4 | missing; searches and sequences need §9.5 |
| model and sampling premises | `15` B1–B3; §12.1 | assumed, not validated |

### 13.7 Decisions

*State at the 2026-10-14 deferral. The 2026-10-18 update to permitted work is in
§13.10; the obstacles and unresolved substantive decisions below remain.*

- **Reviewed:** §13.8, accepted on `24cdfce` as an obstacle in its declared domain (record before
  §1); the three precisions asked there are made in §§13.7–13.8 and in the output's label. The
  rectifications made after `5102852` and §13.5's contract were accepted on `2aa136d`.
- **Next step: none on A2.** The owner defers the design decision (§13.9) and the certifier's work
  is suspended. Neither the study of §12's own acceptance probability nor obligations a and c are
  started.
- **For the owner:** the decision of §13.8 (reading), deferred (§13.9). §13.8 chooses no level,
  threshold, target or resource; its values of reference are existing provisional constants. Its
  obstacle shows that floors up to ½ do not restore usefulness in its domain.
- **After the check** (rectified after `24cdfce`). On an obstacle, the alternatives: restrict the
  claim explicitly with the owner's approval (a larger tolerance, an assumption excluding spread
  small DIF), change the mechanism (no whole-test certificate, R1's honest description kept), or
  keep A2's requirement open. Contrasts limited to groups present at every ability do not overcome
  this obstacle: both classes of `Q_τ` have share ½ at every ability (§13.4). Any future
  restriction must state which alternative it excludes and why that exclusion is justified; none
  is designed here. On a non-exclusion, feasibility stays open and a usefulness result on §12's
  own procedure would be proposed separately. Obligations a and c remain in every case. None is
  chosen here, and the problem stays under A2.
- **Stop rule:** after two reviews still blocked, §11's contract is re-examined rather than a
  further deepening added.

### 13.8 Result of the bounded information check (run on `2aa136d`, accepted on `24cdfce`)

**Status:** run by Claude Code on `2aa136d`, after §13.5's acceptance, in §13.5's domain and no
other; **accepted by Astra on `24cdfce` as an obstacle demonstrated in that domain** under the
numerical contract (record before §1), without closing A2. Three precisions asked by that review
are made below and in the output's label. A2 stays open; the owner's decision is in §13.9.
Evidence: **C**
`sim/a2_information_check.py` (standard library only) and its output
`sim/a2_information_check.json`; **E** its runs and `sim/test_a2_information_check.py`; **D** the
enclosures below; **L** `ClassCurves::dtf`, whose weight at node `q` is `w_q` for `η = (0, 0)` and
shares ½, as in `D(τ)`. The optional comparison with `ClassCurves::dtf` (E API) was not run.

**Objects.** The grid `θ_i = −5 + i/4`, `i = 0, …, 40`, exact; weights `w_i ∝ e^{−θ_i²/2}`,
normalized; the grid law's mean and standard deviation; nodes `u_i = (θ_i − mean)/sd`, as
`Grid::shape` defines them. `b_j = 0.6 Φ⁻¹((j − ½)/m)`, `a_j = 5/4`, floors `c ∈ {0, ¼, ½}`,
`p = c + (1 − c) σ(a(u − b))`, the classes at `b_j ∓ τ`. Each of the 27 configurations (`m`, `c`,
`δ ∈ {0.10, 0.20, 0.40}`) has its `τ_δ` on the lattice `2⁻³⁰ ℤ`: the first point at or above a
floating-point root of `D(τ) = δ`, plus one step. The script would add further steps, at most
eight, if the certified `D` did not exceed `δ`; none was needed. Any `τ` with `D(τ) > δ` gives a
valid bound; no other alternative is sought. At that `τ_δ` the script certifies `D(τ_δ)`, `K`
by §13.4's product form, the size coefficient `1/K̄` and, at `n ∈ {300, 1,500, 3,000, 6,000}`,
`Δ̄_n` and the frontier `1 − Δ̄_n`.

**Certification (D).** Every real quantity is an interval whose endpoints are `k/2¹²⁸`, `k` an
integer. `+`, `−`, `×` and `÷` run on exact integers, rounding the lower endpoint down and the
upper up, so each interval contains its expression's exact value whatever the cancellation. The
trust base is Python's exact integers and fractions. Floating point only seeds brackets and the
`τ` search, and runs the checks. Enclosures:

- `exp x`: `y = x/2^s`, `s` the bit length of `⌊|x|⌋` plus 3, so `|y| ≤ 1/8`. Then the Taylor
  polynomial of `N` terms, `N` least with `2 · 8^{−N}/N! ≤ 2^{−132}`, whose remainder is at most
  `8^{−N}/N! · (1 − 1/(8(N + 1)))^{−1} < 2 · 8^{−N}/N!`, followed by `s` squarings. `exp` is
  increasing, so an interval argument uses its endpoints.
- `ln y`, `y > 0`: `2 atanh t`, `t = (y − 1)/(y + 1)`. The tail of the `atanh` series after `n`
  terms is at most `|t|^{2n+1}/((2n + 1)(1 − t²))`, and `n` grows until that is `≤ 2^{−132}`.
  That bound is `atanh`'s remainder; the factor 2 applies to it as well, so `ln y`'s remainder is
  at most twice it. The script widens the `atanh` enclosure by the tail, then doubles it.
- `√`: the integer square root, floor below and ceiling above.
- `π`: Machin's `16 atan(1/5) − 4 atan(1/239)`, partial sums of 60 and 20 terms in exact
  rationals, each alternating tail bounded by its first omitted term.
- `Φ(x) = ½ + (2π)^{−½} Σ_k (−1)^k x^{2k+1}/(2^k k! (2k + 1))`: with `n` terms and `x² < 2(n + 1)`,
  the tail is at most the first omitted term over `1 − x²/(2(n + 1))`. `Φ⁻¹(p)` lies between
  `x_lo < x_hi` with `Φ(x_lo) < p < Φ(x_hi)` certified, `Φ` being increasing, after 64 bisections.
  The standard library's floating-point quantile only seeds the bracket.
- weights, mean, standard deviation and nodes: interval evaluation of their definitions. The
  mean's enclosure contains 0, as the grid's symmetry requires.
- Directions. `D(τ_δ)`'s lower endpoint is compared exactly and strictly with `δ`. `K̄` is `K`'s
  upper endpoint; `K`'s lower endpoint is positive in every configuration, so the `K = 0` branch
  does not arise. `e^{−nK̄}` is bounded below, the two square roots above, and `Δ̄_n` is their
  minimum with 1; `1 − Δ̄_n` is then exact, and `1/K̄` is rounded down. Decimals are rounded
  outward at ten significant digits; the tables below round further in the same direction.

Widths: quantiles at most `4.1·10⁻²⁸`, nodes `7.3·10⁻³⁶`, weights `2.4·10⁻³⁸`. The printed
intervals of `D` and `K` differ only in their tenth digit, through the outward rounding. The grid's
standard deviation lies in `[0.9999961902, 0.9999961903]`.

**Result at the operative `δ = 0.10`.** Bounds rounded outward; `τ_δ` rounded for display only.
The exact `τ_δ`, the intervals of `D(τ_δ)` and `K`, and the other 18 configurations are in the
JSON.

| `m` | floor | `τ_δ` | `K̄` | `1/K̄ ≥` | `1 − Δ̄_n ≥`, `n` = 300 | 1,500 | 3,000 | 6,000 |
|---|---|---|---|---|---|---|---|---|
| 4 | 0 | 0.05494 | 5.393·10⁻⁶ | 185,430 | 0.9715 | 0.9364 | 0.9100 | 0.8728 |
| 4 | ¼ | 0.07326 | 6.149·10⁻⁶ | 162,635 | 0.9696 | 0.9320 | 0.9039 | 0.8641 |
| 4 | ½ | 0.10994 | 1.156·10⁻⁵ | 86,543 | 0.9583 | 0.9069 | 0.8683 | 0.8138 |
| 8 | 0 | 0.02770 | 1.484·10⁻⁶ | 674,109 | 0.9850 | 0.9666 | 0.9528 | 0.9332 |
| 8 | ¼ | 0.03694 | 1.626·10⁻⁶ | 615,327 | 0.9843 | 0.9650 | 0.9506 | 0.9301 |
| 8 | ½ | 0.05541 | 2.845·10⁻⁶ | 351,613 | 0.9793 | 0.9538 | 0.9346 | 0.9076 |
| 16 | 0 | 0.01391 | 3.880·10⁻⁷ | 2,577,914 | 0.9923 | 0.9829 | 0.9758 | 0.9658 |
| 16 | ¼ | 0.01854 | 4.172·10⁻⁷ | 2,397,064 | 0.9920 | 0.9823 | 0.9749 | 0.9646 |
| 16 | ½ | 0.02782 | 7.039·10⁻⁷ | 1,420,773 | 0.9897 | 0.9770 | 0.9675 | 0.9540 |

The uniform frontier is the minimum over the nine forms (`m` and floor), rounded down:

| `δ` | `n` = 300 | 1,500 | 3,000 | 6,000 |
|---|---|---|---|---|
| 0.10 | 0.9583 | 0.9069 | 0.8683 | 0.8138 |
| 0.20, sensitivity | 0.8333 | 0.6274 | 0.4731 | 0.2548 |
| 0.40, sensitivity | 0.3320 | 0.0057 | 6.6·10⁻⁵ | 8.9·10⁻⁹ |

`K` grows by a factor between 16.0 and 16.1 when `δ` doubles, and falls by a factor between 3.6
and 4.1 when `m` doubles at fixed `δ`: the order of §13.4's heuristic, `D⁴/m²`.

**Consistency checks** (C, floating point). They test the implementation and certify nothing.

- Product form against exhaustive enumeration of the `2^m` patterns, at all 41 nodes of the 27
  configurations, each pattern's ratio `M(z)/I(z)` taken from the product laws' definition. The
  largest relative deviation of `χ²_q` from the certified value is `4.3·10⁻¹¹`; the declared
  tolerance is `10⁻⁸`.
- `KL(Q_τ ‖ P_0)` by exhaustive enumeration, ability marginalized over the 41 nodes, in the 27
  configurations. Without anchors it is between 0.012 and 0.088 of `K̄`; with two declared open
  anchors (`a = 5/4`, `b = ∓3/5`), between 0.027 and 0.157. All are below 1.
- §13.4's idealized identity: in 12 cases (`m ∈ {4, 8, 16}`, `p ∈ {¼, ½}`, `d ∈ {1/64, 1/16}`) the
  certified product form encloses the exact rational `½[(1 + e)^m + (1 − e)^m]`.
- Order: `K` over `Σ_q w_q (m − 1) D_q⁴/(32 m³ p̄_q² (1 − p̄_q)²)`, with `D_q` the node's contrast
  and `p̄_q` its items' mean probability, lies between 1.24 and 1.64. The declared band is
  `[0.1, 10]`.
- Interval primitives: `exp` and `ln` against 80-digit decimal references, `π`, `√2`, the
  quantile brackets, the product form against an exact enumeration, and the weights' sum and mean.
  `sim/test_a2_information_check.py`, 7 tests passing.

The ratios `KL/K̄` are a numerical indication that the per-node bound is loose by a factor between
about 6 and 80. The information is then likely below `K̄` and the certified frontier conservative;
no stronger frontier is claimed from them.

**Resources and reproducibility.** From the repository root,
`python3 sim/a2_information_check.py > sim/a2_information_check.json`. CPython 3.12.3, Linux
x86-64, AMD Ryzen 7 5800X, one process: 20.4 s wall-clock in each of three runs of the committed
version (0.7 s inputs, 4.2 s certification, 15.3 s checks), within the minute. Outputs are
identical across runs apart from the `run` block (environment and timings). The certified values
depend only on integer arithmetic. On another platform the floating-point seeds could move `τ_δ`
or the last digits; every bound printed there would still be valid at its own `τ_δ`. Tests:
`python3 -m unittest discover -s sim -p test_a2_information_check.py`, 0.6 s. After `24cdfce`
the output's `label` string was corrected in the script and in the recorded JSON alike, without a
new run: it names the certified bounds and the consistency checks apart. No value, timing or
provenance changed.

**Reading** (accepted on `24cdfce` with the result; the alternatives rectified after it):

- **Obstacle demonstrated, under the numerical contract, in §13.5's domain only.** At each
  configuration and size, every `(α, β)` with `α + β < 1 − Δ̄_n` is excluded: no procedure valid at
  `α` on a family containing `Q_{τ_δ}` accepts `P_0`'s form with probability at least `1 − β`.
  §12's certifier is included, and no share or presence floor up to ½ restores usefulness there
  (§13.4). At the operative `δ = 0.10`, every pair with `α + β < 0.8138` is excluded for all nine
  forms at all four sizes. The frontier is higher for larger `m` and smaller `n`.
- **Sizes.** At `(α, β)` with `α + β < 1`, usefulness needs
  `n ≥ max{2s², ln(1/(1 − s²))}/K`, `s = 1 − α − β`, hence at least that expression with `1/K̄` in
  place of `1/K`. At `δ = 0.10` the certified lower bound of that coefficient ranges from 86,543
  (`m = 4`, floor ½) to 2,577,914 (`m = 16`, open). Pairs with `α + β ≥ 1` constrain no `n`;
  `α = β = 0` needs an infinite `n`. With `α` and `β` symbolic, no integer size is produced: one
  would be the ceiling of a lower bound of the expression, computed once the levels are fixed.
- **Numerical indication only:** the looseness of `K̄` above. Nothing else is reported without
  certification.
- **Not excluded:** at each configuration and size, the pairs with `1 − Δ̄_n ≤ α + β < 1`. There
  feasibility stays open, no floor is shown sufficient, and §12's power is unknown. Under the
  sensitivities the frontier falls fast: at `4δ` and `m = 4` it is below 0.066 from `n = 1,500`.
- **Decision needed (owner; deferred, §13.9).** The reading changes with `δ` and, for pairs
  between the uniform frontier and the largest one, with the form and the size. Per §13.5, the
  owner fixes `(α, β)`, the tolerance or the sizes with this trade-off. Per §13.7, on an
  obstacle, the alternatives are a claim restricted with the owner's approval (a larger tolerance,
  an assumption excluding spread small DIF), another mechanism (no whole-test certificate, R1's
  honest description kept), or A2's requirement kept open. Contrasts limited to present groups do
  not overcome this obstacle, since both classes of `Q_τ` have share ½ at every ability; a future
  restriction must state which alternative it excludes and why that is justified. None is chosen
  here, and the owner defers the choice (§13.9). Obligations a and c remain in every case, and the
  study of §12's acceptance probability is not started.
- **Not covered:** what §13.5 lists (other alternatives, misspecification, search over forms, the
  linkage decision, §12's power), and sizes, tolerances or forms other than those declared.

### 13.9 Owner's decision: A2's design decision deferred (2026-10-14)

*Historical decision retained. The permitted next work was updated by the owner on
2026-10-18 in [§13.10](#1310-owner-direction-foundations-and-iterative-testing-2026-10-18); its scope and the remaining
design decisions are stated there.*

**Decision (owner).** The design decision on A2 is deferred.

- **Kept:** the requirement of a useful guarantee on the whole test (§11; `15` A2). A2 remains
  open and blocking for Phase 1's closure under the current criteria (`15` A2; `10` §1.5's
  completion qualification), which are unchanged.
- **Not approved:** any narrowing of the guarantee, new threshold, risk level, sample size, model
  assumption or resource.
- **Suspended until the decision:** the work on A2's certifier, obligations a and c and the study
  of §12's acceptance probability included. No other A2 work is authorized.
- **Elsewhere:** other Phase 1 requirements may advance where they do not depend on A2's solution.
  The requirement is not moved to Phase 2 automatically.

**State.**

- *Demonstrated, each in its own domain only:* §12's coverage under its hypotheses; §13.3's
  corollary; §13.8's obstacle in §13.5's domain under the numerical contract. At `δ = 0.10`, every
  `(α, β)` with `α + β < 0.8138` is excluded for the nine declared forms at the four declared sizes.
- *Why A2 stays open:* domination (a) and the conservative computation (c) are missing. In
  §13.8's domain no procedure is useful at a pair below the frontier of its configuration and
  size; above it feasibility is not shown, and outside that domain nothing is. Cost is not
  assessed. Going further needs a decision among §13.7's alternatives, which the owner defers.
- *Phase 1:* incomplete under the kept requirement. No narrowing of the claimed domain is approved
  (`10` §1.5).

### 13.10 Owner direction: foundations and iterative testing (2026-10-18)

**Owner direction.** Record the discussed principles as foundational before the next
test work, then proceed iteratively: test, inspect the evidence and refine the candidate.
The canonical principles are [`01` D44](01-decisions.md). The owner does not expect
perfect performance and used "75%" to express willingness to accept partial progress.
That phrase selects no metric, denominator, error budget or acceptance threshold.

**Change from §13.9.** Its blanket suspension of A2 work is lifted for this documentation,
evaluation design and bounded, predeclared tests that neither adopt the unresolved
substantive choices nor commit unapproved resources. The certifier's implementation
remains suspended pending its contract decisions. No large campaign or production work
is started or authorized by this record. A2 remains open and blocking under the kept
whole-test criterion; the established obstacles and prior scoped approvals are unchanged.
This is the owner's direction, not a new mathematical or independent design-review result.

**Decisions still open.** How D38's compensation objective treats differences in pertinent
factual knowledge; the concrete target population, competence and contrasts; whole-test
tolerance; false-certification risk and useful acceptance; sample/answer budget; and the
common-bank versus per-form design. Neither a fixed form nor a relaxed certificate is
adopted here. The existing D38 behavior remains until an explicit amendment.

**Next deliverable.** A small evaluation declaration, before running it, identifying:

1. The candidate commit, test form(s), target and input domain; reference behavior to
   compare; the relation to §13.8's already excluded combinations.
2. Synthetic cases with known truth that separate legitimate relevant-ability differences,
   nuisance effects, accumulated small DIF, pertinent factual-knowledge differences and
   insufficient information. The intended treatment of the factual-knowledge cases stays
   an explicit alternative, not an assumed new pass/fail rule.
3. Separate denominators and uncertainty for detection/false alarms, correct/incorrect
   certification if a certifier exists, indeterminate outcomes, and respondent/computation
   cost. For the current selector, measure fitted-cost compliance and true whole-test
   behavior separately; do not rename its selections certificates.
4. Construction/tuning data versus confirmation data, seeds/repetitions, resource cap
   and stop conditions; label any post-result revision as a new exploratory iteration.

The declaration may compare alternatives without implementing a new policy. Synthetic
truth is known by construction, not proof that the anonymous model can recover it in
real populations. Finite empirical success does not replace representation, coverage or
conservative computation where a guarantee is claimed. Imperfect utility is compatible
with honest limits; silently reporting an unsupported guarantee is not.

The first such declaration is
[§13.11](#1311-first-bounded-probe-declaration-not-run); it is not run.

### 13.11 First bounded probe: declaration (not run)

**Status:** written by Claude Code on `d413516` under §13.10, before any run; not reviewed, not
executed. It adopts no policy, tolerance, risk level, sample size or resource; A2 stays open and
blocking. Evidence: **L** the symbols below, read at `d413516` (engine and protocol unchanged since
`6274270`); **C** a scratch script, not committed: the generator's true DTF on reference items
(`a = 1.25`, `b_j = 0.6 Φ⁻¹((j − ½)/8)`) and over 2,000 draws of its item law, and §13.4's per-node
bound on the reference items, in floating point — indicative, no respondent, no fit. No Rust, fit
or campaign was run.

**Question.** On a few synthetic batches with known truth, what does the current path — latent
re-check, routing, contested-facts cost, draw — report, and how does it compare with the true DTF
of the forms it would let through? The current selector is not a certifier: this is a diagnostic
demonstration of its behaviour, with no validity, coverage or frequency claimed.

**Executable paths (L).**

| Level | Runs today | Missing for this probe |
|---|---|---|
| known curves | `ClassCurves::{with_floors, with_ability, dtf}`; `characterization::run::true_curves` (private) | no harness records the whole form's truth or the classes' raw-score gap |
| estimation by a fit | `generate::dif_batch` → `latent_dif` → `ClassCurves::of` (`run::dtf`; study `dtf-error`'s cells mirror only, `DTF_SETS` without the whole form, no admission field; its summary keeps converged mixtures only, so one-class zeros leave its denominators) | the cases below; the whole form; the gate's outcome |
| contested selection | `revalidation::{latent_batch, target_rechecks}` → `lifecycle::step` (`Pilot2Batch`, `source_verified` an input, T68) → `ContestedPool::{record, dtf, draw}`, chained only by `protocol/tests/contested_facts.rs` (`fitted::at_pro_08_on_batches_fitted_through_the_gate`, `calibration`) on two fixed batches | that chain on generated batches, with the outputs below |
| full composition | none: `blueprint::assemble_test` ignores DTF; no code composes active items and contested facts; what follows `NoBalancedDraw` or `OutOfDomain` is unspecified (`02` §B.7) | not built: the probe's forms are harness readings, not a composer |

**Harness, to write after review.** One entry point in `crates/characterization`, apart from the
existing studies, reusing `dif_batch`, the production gate with synthetic pseudonyms (as
`tests/production.rs`), `target_rechecks`, `lifecycle::step`, a one-fit `ContestedPool` and
`true_curves`'s construction; one JSON line per run; one smoke-size record pinned by a test. Nothing
changes in `crates/scoring`, `crates/protocol`, the existing studies, seeds, records or summaries.
It is study scaffolding, not an operational path (§10.4).

**Candidate and domain.** Engine and protocol at `6274270`, production defaults (`latent_dif`:
1–4 classes, both slope alternatives, 4 starts, 41 nodes, shape estimated), `MIXTURE_DIF_MAX = 1.0`,
`DTF_MAX = 0.10`. Every case: `N = 3,000` (`N_LATENT_MIN`), 60 anchors, `K = 8` trial items, open
format, `π = ½`, normal ability, `a_j ~ U(1, 1.5)` and `b_j ~ 0.6 N(0, 1)` drawn per run (`13`
§3.1); no floor, slope gap, skew, template or attack. One fit per form: no cross-fit sum (§4,
row 2). Truth: the generator's classes `z = ±1`, known to the harness only, on
`ClassCurves::with_floors`'s grid over the population mixture; score the number correct. The
engine receives the 0/1 matrices and nothing else (invariant 1).

**Readings, no extra fit.** Source input S0 (no key verified: a flagged item is rejected) and S1
(every key verified: it is contested). Form A: the items `step` routes to `ActivePool`. Form A+C
(S1): A with the draw of all contested items at `DTF_MAX`, or on `NoBalancedDraw` of its largest
feasible size, recorded as such. `Indeterminate` items stay in `Pilot2`, in neither form. The fixed
form, all eight items, is read as well: §11.1's target is a form fixed before the data.

| Case | Input (`dif_batch`) | True quantity (C; median [5%, 95%] over item draws) | Estimated, observed | Question |
|---|---|---|---|---|
| C0 null, reference | layout `c0`, δ 0, impact 0 | DTF 0; raw gap 0 | gate, status, classes, flags; `DTF_F` of the fixed form; routing, `D`, draw | baseline the paired cases are read against: false flags, the fitted functional at a true 0 |
| C1 ability difference, no DIF | C0 with impact 1 | DTF 0; raw gap 1.70 [1.55, 1.82] of 8 points | as C0 | is a class-wide difference in the measured ability kept out of DIF (`01` D44)? |
| C2 residual difference at equal ability | `c2`, δ 0.9: gap 1.8 on two items, one side | form 0.77 [0.67, 0.85]; each leaner 0.39 [0.32, 0.44] | as C0; S0 rejects, S1 contests | is concentrated DIF found; what is the true DTF of what passes? |
| C3 small concordant contributions | `c8`, δ 0.17: log-odds gap about 0.43 at `a = 1.25`, the ETS A boundary behind `DTF_MAX` (`02` §B.7) | form 0.61 [0.57, 0.64]; each item 0.076 [0.061, 0.087] | as C0; the true gap 0.34 is under the cut, and unflagged items are active, outside `D` by design | how large is the active items' omission from `D` (§3, H3); does `DTF_F` see it? |
| C4 compensation in one fit | `m` (`+ − + −`), δ 0.9 | form 0.13 [0.03, 0.30], above 0.10 in 64.5% of the 2,000 item draws; each leaner 0.39 [0.32, 0.44] | as C0; S1: the selections, their `D` and true DTF | does the cancellation the draw counts on hold; does selecting on the fitted cost select its error? |
| C5 insufficient information | `c8`, δ 0.04 | form 0.143 [0.133, 0.151]; each item 0.018; reference items: `K ≈ 6.4·10⁻⁶`, `TV ≤ 0.10` at `N = 3,000` (indicative) | as C0, run by run against C0 | what does the current path report where the data barely tell this form from C0 (§13.8)? |

**Pertinent factual knowledge, apart.** C2 and C4 under S1, their leaners declared sourced facts
of pertinent knowledge. Data and fits are C2's and C4's: the engine sees the same responses as for
a nuisance effect, and only the scenario knows the cause. Reported: whether the current balancing
draws each such fact, with which counterweight, and the true DTF with and without it. Two readings
stay open — D38's balancing as implemented, or the difference counted as measured knowledge — and
neither carries an expected outcome (`01` D44; §13.10).

**Metrics, kept apart.** `R` = 8 runs per case; `R_read` those the gate admits; `R_eval` those of
`R_read` whose selected fit converged.

| Metric | Per run | Denominator |
|---|---|---|
| fitted-cost compliance | each returned selection has `D ≤ DTF_MAX` (a violation is a code defect) | returned selections |
| true DTF | of the fixed form, A, A+C and the selection; `DTF_F` of the fixed form beside the first; the classes' raw-score gap | `R` (fixed form, raw gap); runs producing the form |
| reference-line exceedances | true DTF of A or A+C above 0.10; true DTF of a selection minus its `D` | runs producing that form; returned selections |
| refusals, indeterminates | gate refusal; unconverged fit; `Indeterminate` items; `NoBalancedDraw` with its feasible sizes; `OutOfDomain`. One-class readings (converged, no flag) listed apart | `R`; items of `R_read` |
| detection, false alarms | flagged leaners; flagged clean items; runs with a clean item flagged | leaners × `R` and × `R_eval`; clean items × `R_eval`; `R_eval` |
| cost | wall time, candidates fitted, answers `N(A + K)`, the process's peak resident memory | per run; per process |

No certifier exists, so no correct or incorrect certification is counted. Counts are reported as
"`k` of `r`" with per-run values, medians and ranges; no interval, pooled rate, cross-case
aggregate or single success score. Eight runs separate a systematic behaviour from an occasional
one and support no rate: 0 of 8 is compatible with a per-run frequency up to 31% (exact one-sided
95%). "75%" enters no metric; 0.10 is a reference line, not A2's tolerance.

**Seeds, phases, budget, stops.**

- *Seeds.* Run seed `s` = SHA-256 of `isegoria/a2-probe/v1`, the phase and `r`, as `grid::seed`
  builds it; the same `s` in every case at `(phase, r)`, so C1 and C5 differ from C0 only by impact
  or δ and comparisons are paired. Engine seed `engine_seed(s)`, draw seed `draw_seed("contested",
  s, 0)`.
- *Exploration:* the existing chain on its two batches (command 1), the harness test, then phase
  `explore`, `r = 0`, every case at full size. It checks and times; its values are not results and
  are not pooled. The one change it may cause is a smaller `R`, recorded before confirmation, if
  the timing breaks a cap.
- *Confirmation:* phase `confirm`, `r = 0, …, 7`, 48 fits, run once.
- *Cost (estimate, not a measurement).* Pre-D43 open fits on the container: 15–20 s at
  `N = 3,000`, `K = 8`; 75 s for `dtf-error` at 6,000 (`13` §2). D43's histogram made a floor fit
  about half as much again (T82); assumed at most double here, mixtures dearest: 30–90 s a run,
  so 54 runs take 0.5–1.4 CPU-hours, under 15 minutes on 8 workers, plus command 1's two fits.
  Memory: 3,000 × 68 answers and 3,000 × 4 posteriors per fit, tens of MiB a worker. Answers:
  204,000 a run (synthetic; in the field, one admitted batch at the gate's floor), 9,792,000 in
  confirmation.
- *Caps:* 2 CPU-hours and 45 minutes of wall time for both phases; 2 GiB peak resident memory;
  the owner's machine (Ryzen 7 5800X, 16 threads).
- *Stops.* `git diff --quiet 6274270 -- crates/scoring crates/protocol` fails; a harness check
  fails (C0 or C1 truth not exactly 0, the pool's `D` differing from `ClassCurves::dtf` rounded up,
  a returned selection above `DTF_MAX`, a malformed record); a cap is reached (no new run starts;
  partial results labelled partial with their denominators); one run passes 10 minutes (it ends,
  then the probe stops and the estimate is reported wrong). No outcome starts, stops or extends
  the probe. Changing a case, the model, a metric or a reading after the results opens a new
  exploratory iteration with a new phase tag.

**Commands, proposed; only 1 exists at `d413516`.** (1) `cargo test --release -p protocol
--features calibration --test contested_facts fitted`; (2) `cargo test --release -p
characterization --test a2_probe`; (3) `cargo run --release -p characterization --bin a2-probe --
--phase explore --replicates 1 --jobs 8 --out characterization-results/a2-probe-explore`; (4) the
same with `--phase confirm --replicates 8 --out characterization-results/a2-probe-confirm`.

**Against §13.8.** §13.8 is an obstacle for every procedure valid at `α` on its domain; this probe
evaluates a procedure that claims no validity, so no outcome moves that frontier or makes a pair
below it attainable. C5 is adjacent to that domain, not inside it: drawn `a` and `b` instead of
quantiles and one slope, a continuous normal ability, δ for a form DTF near 0.14. Its indicative
bound says the observations should resemble C0's whatever the procedure, so it can show only what
the current path *reports* there. C2–C4 lie where the bound excludes nothing, which is not
feasibility. The probe can clarify the current path's behaviour per case, the size of the active
items' omission, the in-fit cancellation and the fitted cost's error, one batch's cost, and the
stake of D38's open choice. It cannot settle domination, coverage or conservative computation
(§11.2 a–c), any certificate's usefulness, floors, cross-fit forms, misspecification, real
populations or frequencies: no simulation replaces a proof where a guarantee is kept.

**What would change the next step.**

- C1 flags more, or reads a larger `DTF_F`, than C0 on paired runs: the model takes the measured
  ability for DIF here, and B1–B3 work on impact precedes any A2 policy.
- C2 read as one class at `N = 3,000`: a question for `N_LATENT_MIN` (`15` B4–B5, T83), not for
  A2's design.
- C3 with `D = 0` and true form DTF near 0.6: a whole-test claim must cover active items (as R3
  would), or give way to §13.7's "no whole-test certificate" with R1's description; the choice
  stays the owner's. If `DTF_F` tracks the truth, a whole-form estimate — a diagnostic, not a
  bound — is available from existing APIs; if not, not even that.
- C4 selections within the fitted cost whose true DTF exceeds 0.10: selection on estimates selects
  error inside one fit, sizing the uncertainty treatment (§11.2 b) a margin alone does not supply.
- C5 like C0: the path returns cost 0 where little can be told, which bears on a distinct
  "insufficient information" output and on §13.7's alternatives. C5 unlike C0 is unexpected and is
  checked as a harness error first.
- F: how often a sourced pertinent fact is excluded, or drawn only with a counterweight, sizes what
  D38's open choice decides; the probe does not make it.

**Owner's choices needed.** Only authorizing the run within the caps above, after review of this
declaration and of the harness commit. Floors, cross-fit forms and other sizes are left to later
declarations.
