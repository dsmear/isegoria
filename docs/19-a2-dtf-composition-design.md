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

Static reading of `5dbc66e` (§13.11, the first probe's declaration), distinct from the reviews
above:

- **Accepted:** the diagnostic setting, not all the declaration's inferences.
- **Astra's evidence:** the declaration, `generate::dif_batch`, `ClassCurves`, the `fitted` path
  of `contested_facts.rs` and the pertinent parts of `characterization` read. No Rust, fit or
  scratch script run.
- **Asked:** a reproducible oracle; proportionate conclusions; C5's distance read for its own
  objects; reduced and empty forms recorded as such; an enforceable budget. Made in §13.11 with
  the harness, which this reading does not cover.

Review of `7ce9746` (§13.11's harness), distinct from the reading above:

- **Static review:** the harness's stops and processes, its integrity checks and the analysis's
  stop conditions found insufficient; corrections asked, made in §13.11 after `7ce9746`.
- **Astra's executions:** the 10 Python tests of `sim/test_a2_probe.py` at `7ce9746` and
  reproductions on constructed records. No Rust and no fit.
- **Not approved:** the execution of the probe; its corrections were reviewed next, on `464531e`.

Review of `464531e` (those corrections), distinct from the reviews above:

- **Static review:** three findings — a child spawned before its guard, so a failed `start` write
  left it running; a failure's diagnostic record built before the other children were stopped; a
  stop named in the `session` line not binding without the `stop` line after it. Corrected in
  §13.11 after `464531e`.
- **Astra's executions:** the 17 Python tests of `sim/test_a2_probe.py` and a reproduction of the
  stop in the `session` line. No Rust and no fit.
- **Not approved:** the execution of the probe; the corrections are not reviewed.

Approval of `9511ce1` (§13.11's harness, after the corrections above), distinct from the reviews
above:

- **Approved:** the harness, for this bounded probe only.
- **Astra's evidence:** a static reading and the 18 Python tests of `sim/test_a2_probe.py`. No Rust
  and no fit.
- **Not covered:** the probe's results (§13.12), its reading and any next step.

Review of `ee5cf3b` (§13.12, the probe's results), distinct from the reviews above:

- **Accepted:** the results, as bounded diagnostic evidence. A2 stays open and blocking.
- **Astra's evidence:** the preserved records checked — `check` passing on 736 values, explore and
  confirm complete — and the confirm report, `confirm-details.json` and `confirm-factual.json`
  reproduced exactly. No Rust and no new fit.
- **Rectifications asked, made after `ee5cf3b`:** C3's contested cost is absent, not zero (no item
  was contested), the whole-form estimate being the zero; the aggregate memory reads 41.6 and
  55.0 MiB (42,568 and 56,356 KiB). The raw artefacts are unchanged.

Static review of `d88853a` (§13.13, the proposal), distinct from the reviews above:

- **Astra's evidence:** the proposal and the pertinent code read. No test and no fit.
- **Asked:** the experiment declared as exploratory on the first probe's data; readings
  proportionate both ways; comparisons with a non-converged baseline or candidate kept apart.
  Made in §13.13 and §13.14.
- **Owner's authorization:** a bounded, read-only change to `scoring::latent` exposing the
  candidates already computed, and the harness to record them; not the 24 runs at full size.

Approval of `2e2572c` (§13.14's diagnostics and harness), distinct from the reviews above:

- **Approved:** for this exploratory look only.
- **Astra's evidence:** the code, the gate and the checks read; the 21 Python tests of
  `sim/test_a2_probe.py` run. No Rust and no fit.
- **Then:** the owner authorized the 24 runs of phase `candidates` alone (§13.15).

Verification of `54d44fe` (§13.15, the candidates' results), distinct from the reviews above:

- **Confirmed:** the execution and the numbers. Astra re-ran `check` on the 312 values and
  `compare-first` on the 24 records, and reproduced `candidates-report.md` and `summary.json`
  exactly. No Rust and no new fit.
- **Rectifications asked, made after `54d44fe`:** the lack of a consistent ordering holds for the
  likelihood gains only; C3's per-class gain maximum exceeds C0's; the candidates' DTF orders C3
  above C0 in 5 of 6 comparable pairs in each variant, and is read proportionately; the deduction
  that a check on this model at this size must report C3 as not evaluable is withdrawn.

Review of `55373cb` (§§13.15–13.17), distinct from the reviews above:

- **Accepted:** the rectifications of §13.15 and the re-reading of §13.16; §13.17 approved as a
  descriptive check on new data.
- **Astra's evidence:** the declaration read; `paired-dtf.json` reproduced exactly with
  `paired_dtf.py`, the main counts and the subsets without zeros confirmed. No Rust and no fit.
- **Owner's authorization:** §13.17's harness phase and small tests (§13.18); not its 32 runs.

Review of `89b7163` (§13.18's implementation), distinct from the reviews above:

- **Finding:** `dtf-check` analysed a phase other than the declared one. With a plan of only
  `dtf-check-C0-0` and `dtf-check-C3-0`, ledger and records consistent, it exited 0 with "Audit:
  complete, 2 of 2 runs" and one pair of 16 analysed, even with the phase line declaring 16.
- **Astra's evidence:** the diff and the pertinent paths read; the 24 Python tests run, all
  passing; the defect reproduced on constructed records. No Rust and no fit.
- **Owner's direction:** correct the preliminary check alone, regressions first; the runs await
  the correction's verification. Corrected in §13.18's analysis; not yet verified.

Review of `821f52d` (§13.18's correction), distinct from the reviews above:

- **Accepted:** the correction of the preliminary check, for this descriptive check alone; the
  approval does not concern A2's closure.
- **Astra's evidence:** the diff and the preliminary check read; the 29 Python tests run, all
  passing, the historical re-reading included. No Rust and no fit.
- **Owner's authorization:** the 32 runs of `dtf-check` on `821f52d`, with their caps (§13.19).

Review of `307530a` (§13.19), distinct from the reviews above:

- **Accepted:** §13.19's results, as bounded diagnostic evidence.
- **Astra's evidence:** the 32 tasks' contract verified and the audit complete; `check` rerun on
  the 416 values, passing; `dtf-check-report.md` and `dtf-check.json` reproduced exactly from the
  records. No Rust and no new fit.
- **Reading approved:** with shared slopes C3's predominance over C0 reappears descriptively on the
  new data, 14 greater, 1 smaller, 1 equal; with per-class slopes the result is more mixed, 8
  greater, 5 smaller, 2 equal, one pair not comparable; the variants are not summed. These counts
  do not measure a classifier's accuracy on a single test; the point value is shown neither an
  accurate estimate of the generating DTF nor a guaranteed bound; no change to the BIC or the
  selector is justified by this check alone.
- **Rectified:** §13.19's "one start a candidate". The path runs `LatentParams::default()`, four
  seeded starts for each multi-class candidate, and the diagnostics keep only the chosen solution.
  The original artefacts are unchanged.
- **Owner's direction:** a short declaration of the next diagnostic study, documentation only
  (§13.20); no run, fit or code change.

Review of `53f2427` (§13.20), distinct from the reviews above:

- **Astra's evidence:** the documentation diff read; the table's intervals recomputed with SciPy.
  No Rust and no fit.
- **Accepted:** the separation of development, calibration and validation.
- **Rectified in §13.20:** (1) the range `TV` 0.077–0.120 concerns §13.11's item draws, not every
  new batch, and bounds probabilities, not observed frequencies: "a larger excess would point to an
  error" and the precision claim resting on 0.12 as a universal maximum are withdrawn; C5 stays a
  weak-signal scenario. (2) Scientific not-evaluable outcomes are kept apart from technical
  failures. (3) At `n = 50`, [0.005, 0.137] is `k = 2`, 4%: every interval now shows its `k/n`.
  (4) C2 and C4 stay out to contain the design, not because characterizing them needs D38 settled.
- **Owner's configuration:** calibration of 100 runs each of C0 and C1; validation of 100 each of
  C0, C1, C3 and C5; new seeds disjoint across stages, cases and earlier probes; the shared
  indicator already chosen, no change to `scoring` or `protocol`; a nominal α = 0.05 on the neutral
  cases; one threshold, the larger of those set apart on the two neutral cases; no band; not
  evaluable reported apart, with no operational behaviour given to it.
- **Owner's direction:** fix the threshold's order rule before the code; implement the harness and
  small tests; plan, cost and cumulative caps for the 600 runs (§13.21). No calibration or
  validation run: implementation and budget return to review before any authorization.

Review of `7bbfddb` (§13.21), distinct from the reviews above:

- **Astra's evidence:** the code, the threshold rule and the binding of validation to calibration
  read; the 36 Python tests run, all passing; the ranks compared with the integer formula for
  `n` = 0 to 100, no difference; the intervals checked against SciPy, largest difference about
  `9·10⁻¹⁶`. No Rust and no fit.
- **Approved:** the calibration alone; `ind-val` is not authorized.
- **Owner's authorization:** the 200 calibration runs on `7bbfddb` with their caps; `threshold.tsv`
  written only if the contract, the complete audit and the numerical check pass, its exit code
  verified (§13.22).

Review of `185ec93` (§13.22), distinct from the reviews above:

- **Astra's evidence:** the calibration's contract and complete audit verified; `check` rerun on
  the 2,600 values, passing; `rank-check.txt` reproduced exactly; the three files' digests
  verified; `threshold.tsv` recomputed and reproduced byte for byte. No Rust and no fit.
- **Accepted:** the calibration, as the application of the declared rule, not as a validation
  result.
- **Owner's authorization:** the 400 validation runs on `185ec93` against the calibration frozen
  there, with their caps (§13.23).

Review of `ccb8f05` (§13.23), distinct from the reviews above:

- **Astra's evidence:** the contract, the complete audit and the binding to the calibration
  verified; `check` rerun on the 5,200 values, passing; the report reproduced exactly save the
  calibration's local path; the JSON reproduced byte for byte; a separate recomputation of the
  counts, intervals and separations confirmed. No Rust and no new fit.
- **Accepted:** the results, within the declared scope. A2 is not closed.
- **Reading approved:** with the frozen threshold C3 has 23 detections and 73 misses among 96
  evaluable runs; the observed ordering does not make this check sufficient for the whole-test
  objective; C5 shows no useful separation in these data, with no deduction about an absence of
  information in general; the separation of 0.72–0.76 is not classification accuracy.
- **Owner's direction:** an exploratory reading of the trade-off between false alarms and
  detections on the existing records, with no new fit or run (§13.24).

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
[§13.11](#1311-first-bounded-probe-declaration-and-harness-not-run); its harness is
implemented, and the probe is not run.

### 13.11 First bounded probe: declaration and harness (not run)

**Status:** declared by Claude Code on `d413516` (`5dbc66e`); Astra's static reading of `5dbc66e`
accepted the diagnostic setting, not all its inferences, and the declaration was rectified with the
harness in `7ce9746`. Astra's reviews of `7ce9746` and `464531e` asked corrections to its stops,
integrity checks and analysis stops, made below; Astra approved the harness on `9511ce1` for this
bounded probe, after a static reading and the 18 Python tests, with no Rust and no fit (records
before §1). The owner authorized explore and confirm; they ran on `9511ce1` (§13.12). It adopts no
policy, tolerance, risk level, sample size or resource; A2 stays open and blocking.

**Question.** On a few synthetic batches with known truth, what does the current path — latent
re-check, routing, contested-facts cost, draw — report, and how does it compare with the true DTF
of the forms it lets through? The current selector is not a certifier: the runs are diagnostic
examples of its behaviour, with no validity, coverage or frequency claimed.

**Executable paths (L).**

| Level | Runs today | What the probe adds |
|---|---|---|
| known curves | `ClassCurves::{with_floors, with_ability, dtf}`; `run::true_curves` | the generating law's DTF and raw-score gap, apart from the grid's approximation |
| estimation by a fit | `generate::dif_batch` → `latent_dif` → `ClassCurves::of` (`run::dtf`; `dtf-error`'s cells mirror only, `DTF_SETS` without the whole form, no admission field, one-class zeros outside its summary's denominators) | the six cases through the production gate `latent_batch`, every outcome recorded |
| contested selection | `target_rechecks` → `lifecycle::step` (`Pilot2Batch`, `source_verified` an input, T68) → `ContestedPool::{record, dtf, draw}`, chained before only by `contested_facts.rs`'s `fitted` test on two fixed batches | that chain on generated batches (`probe::assess`) |
| full composition | none: `blueprint::assemble_test` ignores DTF; no code composes active items and contested facts; what follows `NoBalancedDraw` or `OutOfDomain` is unspecified (`02` §B.7) | none: the probe's forms are harness readings, not a composer |

**Harness (not reviewed).** `crates/characterization/src/probe.rs`: cases, seeds, the generating
DTF, routing, a one-fit pool, forms, the JSON record. `src/probe_jobs.rs`: the supervisor.
`src/bin/a2-probe.rs`: `plan`, `run`, `one`. `tests/a2_probe.rs` (readings and records) and
`tests/a2_probe_jobs.rs` (the supervisor). `sim/a2_probe.py`: the oracle, `audit`, `check`, `budget`
and `report`, with `sim/test_a2_probe.py` and the committed `sim/a2_probe_reference.json`. Outside
them: `run::true_curves` made public; `network` (for `Cid`) and `libc` (the parent-death signal and
the lock), already in `Cargo.lock`, dependencies of `characterization`. Nothing changes in
`crates/scoring`, `crates/protocol`, the existing studies, seeds, records or summaries. It is study
scaffolding, not an operational path (§10.4).

**Candidate and domain.** Engine and protocol at `6274270`, production defaults (`latent_dif`:
1–4 classes, both slope alternatives, 4 starts, 41 nodes, shape estimated), `MIXTURE_DIF_MAX = 1.0`,
`DTF_MAX = 0.10`. Every case: `N = 3,000` (`N_LATENT_MIN`), 60 anchors, `K = 8` trial items, open
format, `π = ½`, normal ability, `a_j ~ U(1, 1.5)` and `b_j ~ 0.6 N(0, 1)` drawn per run (`13`
§3.1); no floor, slope gap, skew, template or attack. One fit per form: no cross-fit sum (§4,
row 2). The classes `z = ±1` are the generator's, known to the harness only; the engine receives
the 0/1 matrices and nothing else (invariant 1). The smoke phase draws the same cases at `N = 400`
with 10 anchors.

**Truth and its numerical approximations.**

- *The truth* is the generating law's DTF of a set: `Σ_g π_g ∫ |Σ_j (P_j⁺ − P_j⁻)(θ)| dN(η_g, 1)`,
  `P_j^z = σ(a_j(θ − b_j − δ s_j z))`, `η = (0, impact)`; the raw gap is `E[S | +] − E[S | −]`, each
  class over its own ability law.
- *Method A*, the harness's (`probe::generating_dtf`): composite Simpson, 4,000 panels over
  `η_g ± 10`, not split where the gap changes sign. *Method B*, the oracle's
  (`sim/a2_probe.py`): composite five-point Gauss–Legendre, panels of at most 0.25 over
  `η_g ± 12`, split at the gap's sign changes (scan step 0.05, bisection). The 41-node value of
  `run::true_curves` is the engine grid's approximation, recorded beside the truth, not as it.
- *Precision check, predeclared:* `check` recomputes every generating DTF and raw gap of the
  records by B, tolerance `10⁻⁵`, and every grid value by a replica, `10⁻⁹`. A failure stops the
  analysis: nothing is then read against the line. Agreement of two discretizations and two
  implementations is a consistency check, not a certified error bound.
- *Observed (C, floating point):* on the reference items (`a = 1.25`, `b_j = 0.6 Φ⁻¹((j − ½)/8)`),
  `|A − B| ≤ 3·10⁻¹⁵` where the gap keeps one sign and `1.5·10⁻⁷` on C4's kinks; B at half its
  panel moves by at most `7·10⁻¹⁶`; the grid departs from B by up to `2·10⁻⁷`, and by `5.2·10⁻⁵`
  on C4. On the 12 smoke records' 36 values: `|A − B| ≤ 5.0·10⁻⁸`, grid `2.2·10⁻¹⁶`.
- *Near the line:* a generating DTF within 0.001 of 0.10 is "near", counted apart from above and
  below. The band is a reading convention, not a tolerance.

**Cases.** True quantities by method B: median [5%, 95%] over 500 draws of the item law (seed
20261018); `python3 sim/a2_probe.py reference` reproduces them in about 40 s.

| Case | Input (`dif_batch`) | True quantity | Estimated, observed | Question |
|---|---|---|---|---|
| C0 null, reference | layout `c0`, δ 0, impact 0 | DTF 0; raw gap 0 | gate, status, classes, flags; `DTF_F` of the fixed form; routing, `D`, draw | baseline the paired cases are read against |
| C1 ability difference, no DIF | C0 with impact 1 | DTF 0; raw gap 1.70 [1.55, 1.82] of 8 points | as C0 | is a class-wide difference in the measured ability kept out of DIF (`01` D44)? |
| C2 residual difference at equal ability | `c2`, δ 0.9: gap 1.8 on two items, one side | form 0.767 [0.675, 0.843]; each leaner 0.386 [0.318, 0.435] | as C0; S0 rejects, S1 contests | is concentrated DIF found; what is the true DTF of what passes? |
| C3 small concordant contributions | `c8`, δ 0.17: log-odds gap about 0.43 at `a = 1.25`, the ETS A boundary behind `DTF_MAX` (`02` §B.7) | form 0.607 [0.567, 0.641]; each item 0.077 [0.061, 0.087] | as C0; unflagged items are active, outside `D` by design | how large is the active items' omission from `D` (§3, H3); does `DTF_F` see it? |
| C4 compensation in one fit | `m` (`+ − + −`), δ 0.9 | form 0.125 [0.029, 0.289]: above the line in 310 draws, near in 6, below in 184; each leaner 0.387 [0.319, 0.436] | as C0; S1: the selections, their `D` and true DTF | does the cancellation the draw counts on hold; how large is the error on the selections drawn? |
| C5 weak signal | `c8`, δ 0.04 | form 0.143 [0.133, 0.151]; each item 0.018 [0.014, 0.021] | as C0, paired with C0 by replicate | what does the current path report when the form exceeds the line by about 0.04 and the signal is weak? |

**C5's motivation and its limits.** §13.4's per-ability bound gives `KL ≤ 6.4·10⁻⁶` per respondent
between C5's law and C0's on the reference items, hence `TV ≤ 0.098` at `N = 3,000`; over the 500
item draws, each bound computed for its own items, `TV ≤` 0.077 to 0.120. Floating point,
indicative, and about those item laws only, not about every draw of the case. A small distance
bounds how much the probability of any event can differ between the two laws; it does not make
paired outcomes equal, and a C5 run unlike its C0 pair is an observation, not a sign of a harness
error. C5 stays a weak-signal case to observe.

**Pertinent factual knowledge, apart.** C2 and C4 under S1, their leaners declared sourced facts
of pertinent knowledge. Data and fits are C2's and C4's: the engine sees the same responses as for
a nuisance effect, and only the scenario knows the cause. Reported: whether the current balancing
draws each such fact, with which counterweight, and the true DTF with and without it. Two readings
stay open — D38's balancing as implemented, or the difference counted as measured knowledge — and
neither carries an expected outcome (`01` D44; §13.10).

**Readings and forms, harness readings.** Source input S0 (no key verified: a flagged item is
rejected) and S1 (every key verified: it is contested); no extra fit. Form A: the items `step`
routes to `ActivePool`. Form A+C: A with the draw of every contested item at `DTF_MAX`; on
`NoBalancedDraw`, the harness draws the largest feasible size, recorded as `reduced` — its own
reading, not a protocol policy. The selection: the contested items drawn. Every record carries the
fixed form, A, A+C and the selection with members, size and completeness against its reference —
the eight items, or the contested ones for the selection: *empty* (no member), *incomplete* or
*complete* — and each missing item's reason (rejected, indeterminate, contested, not drawn, draw
failed). A zero DTF reached by removing items is an incomplete or empty form, never a success.

**Metrics, kept apart.** Every run ends `read`, `refused` (the gate), `killed` or `failed`, and
each has a record with the design's truth. `R` is the runs of a case; `R_read` those with a fit;
`R_eval` those of `R_read` whose fit converged.

| Metric | Per run | Denominator |
|---|---|---|
| fitted-cost compliance | each returned selection has `D ≤ DTF_MAX` (a violation is a code defect) | returned selections, full or reduced |
| true DTF | of the fixed form, A, A+C and the selection; `DTF_F` of the fixed form beside the first; the raw gap | `R` (fixed form, raw gap); `R_read` for the other forms, the runs without one counted beside |
| reference-line comparison | each form's true DTF below, near or above 0.10; a selection's true DTF minus its `D` | `R_read`; returned selections |
| refusals, indeterminates | outcomes over `R`; not converged over `R_read`; one counted class over `R_eval`, listed apart; `NoBalancedDraw` (feasible sizes), `OutOfDomain`, a failed pool record | `R`, `R_read`, `R_eval`; runs with contested items |
| detection, false alarms | flagged leaners; flagged clean items; runs with a clean item flagged | leaners × `R_eval`, and leaners × `R` beside; clean items × `R_eval`; `R_eval` |
| cost | wall and CPU time, peak resident set; candidates fitted; answers `N(A + K)` | per run; per session and cumulative |

No certifier exists, so no correct or incorrect certification is counted. Counts are reported as
"`k` of `r`" with per-run values; no interval, pooled rate, cross-case aggregate or success score.
Under a binomial model — independent runs sharing one probability — 0 of 8 would leave that
probability's exact one-sided 95% upper bound at about 31.23%; outside that model the eight runs
are diagnostic examples only. "75%" enters no metric; 0.10 is a reference line, not A2's tolerance.

**Seeds, phases, cost and stops.**

- *Seeds.* Run seed `s` = the first eight bytes of SHA-256 over `isegoria/a2-probe/v1`, the phase
  and `r`, length-prefixed (`probe_seed`); the same `s` in every case at `(phase, r)`, so the cases
  differ only by layout, δ or impact and comparisons are paired. Engine seed `engine_seed(s)`,
  draw seed `draw_seed("contested", s, 0)`.
- *Phases.* Smoke: the plumbing, no fit (run). Explore: `r = 0`, every case at full size, to
  measure cost; its values are not results and are not pooled. Confirm: `r = 0, …, R − 1`.
- *Replicates, by cost only.* After explore, the rule sets
  `R = min(8, ⌊(C − S)/(1.5 Σ_c c_c)⌋)`: `C` the CPU cap, `S` the CPU already spent, `c_c` each
  case's explore CPU. No confirmation with `R < 2`, an explore run missing, killed or failed, or a
  stop recorded. No outcome enters the rule.
- *Cost estimate, not a measurement.* Historical container times, pre-D43 (`13` §2): 15 s for a
  null batch at `N = 3,000`, `K = 8`; 75 s for `dtf-error` at `N = 6,000`, where mixtures were
  selected in 86–100% of the fits. Assumed factors: ×1.45 for D43 (T82: a two-class floor fit, 619
  evaluations of 12.7 ms against 579 of 9.4 ms); ×½ from `N = 6,000` to 3,000, the per-evaluation
  cost linear in `N` and the iterations unchanged. Hence about 22 s for a run where no mixture is
  likely selected (C0, C1, C5) and 54 s where one is (C2–C4); an allowance ×1.6 for machine and
  run-to-run spread gives 35 s and 87 s. Doubling 15–20 s did not explain the earlier "30–90 s";
  these lines replace it. Per replicate about 230 s, 370 s with the allowance; nine replicates,
  34–55 CPU-minutes. On 8 workers the wall time would be near an eighth of that only if they
  did not slow one another (SMT, frequency): an estimate, not a guarantee; the caps bind instead.
- *Perimeter.* A run's CPU is its child process's user and system time, launch to exit —
  generation, gate, fit, truth, routing, draws, record — read by the child from `/proc/self/stat`
  (10 ms ticks); the supervisor's own CPU is added; compiling, `check` and `report` are outside.
  Totals are cumulative over the sessions of one output directory (`ledger.tsv`). Memory: each
  run's peak (`VmHWM`) is recorded; the aggregate is the largest sampled sum of the running
  children's and the supervisor's current resident sets (`VmRSS`, once a loop), not a sum of peaks;
  a spike between samples can be missed.
- *Caps*, `a2-probe run`'s defaults: 2 CPU-hours, 45 minutes of wall time, 10 minutes a run, 2 GiB
  aggregate resident, cumulative over the sessions of one directory.
- *What stops a session.* A technical failure — a child exiting with an error, or a record missing,
  invalid, another run's, or carrying the blocking defect — a supervisor error, the run limit or a
  cap. A technical failure ends the poll where it is found. Any stop starts no new run and kills
  every running child at once (SIGKILL, then reaped); only then are the diagnostic records built
  and written: the failed run's, its rejected part moved to `rejected/`, and the killed runs',
  `killed` with the design's truth and the polled cost. A child found already completed is
  collected as usual. The ledger records the stop, and the command exits 3 (1 on a supervisor error
  or a refusal). A gate refusal (`refused`) and an inconclusive fit (`read`, its items
  `Indeterminate`) are observations of the probe: they stop nothing.
- *Supervision.* Each child is spawned with `PR_SET_PDEATHSIG = SIGKILL` and checks its parent
  before `exec`, so the kernel kills it if the supervisor dies, even by SIGKILL (Linux). It is under
  a guard from the spawn on, before its `start` line is written: any error after the spawn kills and
  reaps it before the error returns. A second supervisor on the directory is refused (`flock` on
  `out/lock`, released at death).
- *Margin of the sampled checks.* Run limit, CPU, wall and memory are checked once a loop: the
  100 ms sleep, then the loop's own work — `/proc` reads, collecting ended runs (validating, then
  appending and syncing record and ledger lines), launching runs. Each session records its largest
  gap between two checks: 196–198 ms in the stand-in tests where three runs ended in one loop
  (their syncs), 170 ms in the smoke phase. Within one gap a cap can be passed by up to that gap of
  wall time, by up to `jobs` times it of CPU (single-threaded children) plus 10 ms a process of
  tick rounding, and a memory spike shorter than it can be missed; the children's resident sets
  are read one after another, not at one instant. A killed run's CPU is its last sample.
- *Integrity.* `plan-<phase>.tsv` holds the configuration — the candidate (`HEAD:crates` and
  `HEAD:Cargo.lock` tree ids, clean for explore and confirm; `crates/scoring` and `crates/protocol`
  identical to `6274270`), the caps, the schema, the designs, the constants — and the phase's runs
  with their seeds. A session refuses another configuration, another plan for the phase, a stop —
  the `stop` line, or the reason in a `session` line when the process ended before writing the
  `stop` line — an unfinished run, spent caps, or a ledger that is not read strictly (header, field
  counts, counts as integers, each run started once and ended once): never a zero cost. Confirm's
  `R` is set once after explore by the rule (`confirm_replicates`, cross-checked by `budget`) and
  fixed in its plan; an explicit `--replicates` is refused. Each session records its provenance,
  `HEAD` and the executable's SHA-256, not compared.
- *No automatic extension.* No retry and no resumption after a stop or an unfinished run; going on
  needs an explicit act and authorization.
- *Analysis stops.* `audit` compares plan, ledger and records: duplicates, records missing,
  inconsistent (seed, outcome) or unplanned, runs started and never ended, never started, not
  observed (killed or failed), the stop. `check` passes only with values verified, no failure and no
  invalid audit. `report` exits 0 on a complete phase; 2 on a partial one, declared with those
  counts; 1, without analysis, on an empty or invalid phase, a failed precision check (its
  diagnostics printed) or the blocking defect — a returned selection whose cost exceeds `DTF_MAX`,
  in a record, from the supervisor or in `rejected/`. `budget` proposes `R` only for an explore
  phase that is complete, passes the precision check and shows no blocking defect. A generating DTF
  above 0.10 is a result to report, not a defect.
- *Other stops.* A harness test failing. No outcome starts, stops or extends the probe; changing a
  case, the model, a metric or a reading after the results opens a new exploratory iteration with
  a new phase tag.

**Checks run here** (E, on the owner's machine). At `7ce9746`, before correcting, each defect was
shown by a stand-in or constructed check: a failed child did not stop the launches (outcomes
`failed` 1, `read` 5, no stop); an invalid record was accepted (`unknown` 2); a supervisor error
left a child running; two supervisors on one directory wrote 12 records for 6 runs; a ledger's
`NaN` read as a zero cost and an unknown line was ignored; children survived the supervisor's
SIGKILL; `check` passed on an empty file; `report` analysed after a failed precision check and
passed a selection cost of 0.2, declaring no partial phase. By reading, a stop exited 0. At
`464531e`: a ledger ending on a `session` line naming a stop gave no stop to `read_ledger`, `R = 8`
to `confirm_replicates` and to `budget`, a resumed session, a complete audit and a `COMPLETE`
report; the other two findings by reading, the spawn before its guard and the diagnostic record
before the kills.

After correcting: `cargo test -p characterization --test a2_probe --test a2_probe_jobs`, 9 and 19
tests. The first file reads constructed fits, plus one smoke fit (`N = 400`) outside the gate as
plumbing. The second runs the supervisor on stand-in children (`true`, `false`, `sleep`) and the
binary's smoke phase; a journal in memory refuses a `start` line after the spawn, the child dead
when the error returns, and records at the failed run's record that the other child was already
dead. Those tests now spawn one at a time: two of them failed intermittently, a child reaped by
`init` read as alive in its transient `X` state, and a `Locked` refusal from a fork in another test
thread holding a copy of the directory's lock until its `exec`, which one supervisor process does
not meet. Twelve consecutive runs of the second file and 150 of the orphan test then passed.
`python3 -m unittest discover -s sim -p test_a2_probe.py`, 18 tests on constructed records, plans
and ledgers. The smoke phase end to end, then `check`, `report` (complete) and `budget` (refused:
no explore). `cargo fmt`, `cargo clippy --all-targets` and `scripts/comment_budget.py`. Not re-run:
the crate's whole suite, whose `production.rs` fits batches of `N = 3,000` (run once at `7ce9746`,
67 tests passing).

**Commands, in order.** All exist. Only the checks of the first line and the smoke phase, into a
scratch directory, were run. `run` starts from the repository; explore and confirm need a clean
commit. After explore, confirm starts only if `check` passes, `report --phase explore` exits 0
(complete) and `budget` proposes `R`: an incomplete, invalid, imprecise or defective explore gives
no `R`. The `R` confirm uses comes from the cost rule alone, computed by the supervisor itself.

```
cargo test -p characterization --test a2_probe --test a2_probe_jobs; python3 -m unittest discover -s sim -p test_a2_probe.py
cargo run --release -p characterization --bin a2-probe -- run --phase smoke --replicates 2 --jobs 4 --out characterization-results/a2-probe-smoke
cargo run --release -p characterization --bin a2-probe -- run --phase explore --jobs 8 --out characterization-results/a2-probe
python3 sim/a2_probe.py check characterization-results/a2-probe
python3 sim/a2_probe.py report characterization-results/a2-probe --phase explore
python3 sim/a2_probe.py budget characterization-results/a2-probe
cargo run --release -p characterization --bin a2-probe -- run --phase confirm --jobs 8 --out characterization-results/a2-probe
python3 sim/a2_probe.py check characterization-results/a2-probe
python3 sim/a2_probe.py report characterization-results/a2-probe --phase confirm
```

**Against §13.8.** §13.8 is an obstacle for every procedure valid at `α` on its domain; this probe
evaluates a procedure that claims no validity, so no outcome moves that frontier or makes a pair
below it attainable. C5 is adjacent to that domain, not inside it: drawn `a` and `b` instead of
quantiles and one slope, a continuous normal ability, δ for a form DTF near 0.14. C2–C4 lie where
the indicative bound excludes nothing, which is not feasibility. The probe can describe the
current path's behaviour per case, the size of the active items' omission, the in-fit
cancellation and the error on the selections drawn, one batch's cost, and the stake of D38's open
choice. It cannot settle domination, coverage or conservative computation (§11.2 a–c), any
certificate's usefulness, floors, cross-fit forms, misspecification, real populations or
frequencies: no simulation replaces a proof where a guarantee is kept.

**What would change the next step.**

- C1 flagging more, or reading a larger `DTF_F`, than C0 on paired runs: a signal to investigate —
  impact read as DIF is one explanation, estimation noise or the shape's estimation others; it
  proves no confusion by itself. B1–B3 work on impact would come before relying on such verdicts.
- C2 read as one class: an observation on detection at this size and design; by itself it does
  not place the cause in `N_LATENT_MIN` rather than the anchors, the model or the search.
- C3 with no contested cost (`D` absent: nothing contested) and true form DTF near 0.6: a whole-test
  claim must cover active items (as R3 would), or give way to §13.7's "no whole-test certificate"
  with R1's description; the choice stays the owner's. If `DTF_F` tracks the truth, a whole-form
  estimate — a diagnostic, not a bound — is available from existing APIs; if not, not even that.
- C4 selections within the fitted cost whose true DTF exceeds the line: they document error on the
  selections drawn. Without an appropriate comparison, such as the same sets' error unselected,
  they do not isolate what selecting on the fitted cost contributes to it.
- C5: what the path reports under a weak signal bears on whether an "insufficient information"
  output is worth specifying, and on §13.7's alternatives.
- F: how often a sourced pertinent fact is excluded, or drawn only with a counterweight, sizes what
  D38's open choice decides; the probe does not make it.

**Owner's choices needed.** Only authorizing the explore and confirm runs within the caps above,
after review of this declaration and of the harness. Floors, cross-fit forms and other sizes are
left to later declarations.

### 13.12 First bounded probe: result (run on `9511ce1`)

**Status:** run by Claude Code on 2026-10-19 on `9511ce1`, as §13.11 declares, after Astra's
approval of the harness and the owner's authorization; accepted by Astra on `ee5cf3b` as bounded
diagnostic evidence, its two rectifications made below (records before §1). Evidence:
**E** the two phases; **C** the oracle's precision check of every truth; the raw records, plans,
ledger, checks and reports in
[`verification/reports/a2-probe-9511ce1/`](../verification/reports/a2-probe-9511ce1/README.md).
Diagnostic examples of the current path on six synthetic cases: no frequency, certification of the
whole test or causal conclusion. A2 stays open and blocking.

**Execution.** Candidate checked first: `HEAD` = `origin` = `9511ce1`, `HEAD:crates` `d0cdf4e` and
`HEAD:Cargo.lock` `e661541` clean, the engine identical to `6274270`, a new run directory. Explore:
6 of 6 runs `read`, exit 0. Then `check` exit 0 (82 values, largest `|A − B|` `4.7·10⁻⁸`), `report
--phase explore` exit 0 (complete, no blocking defect), `budget` exit 0. `R` by the cost rule alone:
`Σ c = 172.75` s of explore CPU (C0 21.66, C1 21.54, C2 44.41, C3 25.51, C4 37.63, C5 22.00),
`min(8, ⌊(7,200 − 172.87)/(1.5 × 172.75)⌋) = 8`, the supervisor's own computation giving the same.
Confirm, without `--replicates`: 48 of 48 runs `read`, exit 0; `check` exit 0 (736 values, largest
`|A − B|` `4.8·10⁻⁷`), `report --phase confirm` exit 0 (complete). No stop, refusal, killed or
failed run or rejected part; no run missing. Explore's values serve its cost only and are not
pooled below.

**Cost, measured.** CPU 172.9 s (explore) and 1,551.1 s (confirm), 1,724.0 s in all — 28.7 of the
120 minutes; wall time 44.6 s and 220.0 s, 4.4 of the 45 minutes; largest sampled resident sum 41.6
and 55.0 MiB (42,568 and 56,356 KiB), each run's peak 6.2–6.6 MiB; largest gap between checks 134
and 147 ms; 8 workers. CPU per confirm run, mean and range: C0 21.5 s (10.9–30.4), C1 23.7
(17.1–31.3), C2 48.9 (11.8–64.2), C3 22.4 (16.1–28.6), C4 56.3 (32.8–82.5), C5 21.1 (12.9–26.5). The
estimate (22 s without a mixture, 54 s with one, up to 35 and 87 s) held; C3 cost as a null case
because it selected one class. Answers: 204,000 a run, 9,792,000 in confirm, synthetic.

**Results, confirm.** `R = 8` runs per case, all read; `R_eval` 8 except C3 (6).

| Case | Fits | Flags | Fixed form: generating DTF; fitted `DTF_F` | Forms A, A+C | contested cost `D`, draws (S1) |
|---|---|---|---|---|---|
| C0 | 8 converged, 8 one class | clean 0 of 64 | 0; `DTF_F` 0 | 8 items in 8, DTF 0 | `D` absent: no contested item |
| C1 | as C0, identical to its C0 pair in 8 of 8 | clean 0 of 64 | 0, raw gap 1.57–1.81; `DTF_F` 0 | 8 items in 8, DTF 0 | `D` absent |
| C2 | 8 converged: 7 mixtures, 1 one class (r5) | leaners 14 of 16, both in each mixture, none in r5; clean 0 of 48 | 0.610–0.873; `DTF_F` off by −0.030 to +0.095 in the mixtures, 0 in r5 | S0 and S1 alike: 6 items in 7 runs, DTF 0; r5 8 items, DTF 0.610 | 7 runs with 2 contested: `NoBalancedDraw`, feasible {0}, reduced to none, `D` 0 |
| C3 | 6 converged, all one class; 2 `max_iters`, every item `Indeterminate` | leaners 0 of 48 | 0.560–0.652; `DTF_F` 0 | 8 items in 6, DTF 0.560–0.652; **empty** in 2 | `D` absent |
| C4 | 8 converged, all mixtures | leaners 32 of 32; clean 0 of 32 | 0.047–0.271, above the line in 6, below in 2; `DTF_F` off by −0.050 to +0.102 | S0: 4 clean items in 8, DTF 0; S1: 6 items in 7, 8 in 1, DTF 0.028–0.088 | 8 runs with 4 contested: all drawn in 1; 7 reduced to a pair of opposite signs, feasible {0, 2}; `D` 0.031–0.079; selection's DTF minus `D` −0.025 to +0.034 |
| C5 | 8 converged, all one class, identical to its C0 pair in 8 of 8 | leaners 0 of 64 | 0.132–0.154, above in 8; `DTF_F` 0 | 8 items in 8, DTF 0.132–0.154 | `D` absent |

No form's generating DTF lies within 0.001 of 0.10. C3's two empty forms read "below" in the
report's count against the line: removing every item, they are no result. Every returned selection
had `D ≤ DTF_MAX`; no blocking defect.

**Pertinent factual knowledge, both readings (no decision).** C2 under S1: with D38's balancing as
implemented, both sourced leaners were left out in 7 of 7 runs that flagged them — a one-fit pool
holds no counterweight — and the form kept 6 items, DTF 0; counted as measured knowledge, every
sourced fact kept, the form is the fixed one, DTF 0.729–0.873. In r5 nothing was flagged and both
stayed active, DTF 0.610, in either reading. C4 under S1: the balancing drew a pair of opposite
signs in 7 runs, two sourced facts out each time, and all four in 1, DTF 0.028–0.088; every fact
kept gives the fixed form, 0.047–0.271. In C4 r2 the drawn pair's DTF (0.085) exceeds the whole
form's (0.066). Neither reading is preferred here; D38's treatment stays the open decision.

**Against §13.11's "What would change the next step".**

- C1: no difference from C0 in 8 paired runs — one class, no flag, `DTF_F` 0 — so no signal here
  of impact read as DIF; nothing beyond this scenario.
- C2: one run of eight read as one class with two strong leaners at `N = 3,000`, fitted 0 for a
  true 0.610 (§4 row 4 saw it at 1,500): an observation on detection; it does not place its cause.
- C3: no contested cost (`D` absent, nothing contested) and a whole-form estimate `DTF_F = 0` in 6
  of 6 converged runs, for forms whose generating DTF is 0.56–0.65; no reading in 2 of 8. The
  whole-form estimate of existing APIs did not track the truth here: "if not, not even that". With
  the whole-test objective kept, what this implies is set out in §13.13.
- C4: no selection within the fitted cost passed the line (0 of 8); the cost under-estimated the
  selection's true DTF in 2 of 8, by up to 0.034. Without the sets' error unselected, this
  documents error on these selections only.
- C5: outputs identical to C0's in 8 of 8 while every form is 0.03–0.05 above the line: no contested
  cost (`D` absent), `DTF_F = 0` and no flag under this weak signal. Identical pairs are an
  observation, which the distance bound does not require; it bears on an "insufficient information"
  output and §13.7's alternatives.
- F: D38's balancing excluded every uncompensated sourced fact (C2) and two of four in most C4 runs;
  what that should be is D38's open choice.

**Limits.** Eight runs per case are diagnostic examples: under a binomial model 0 of 8 leaves a
per-run probability up to 31.23% (exact one-sided 95%), and no other count here is read as a rate.
One fit per form; open format, no floors, cross-fit forms or misspecification beyond the cases; the
truth is the generating law; 0.10 is a reference line, not A2's tolerance; the composition and the
reduced draw are harness readings. Nothing here bears on domination, coverage, conservative
computation or any certificate's usefulness (§11.2), nor on real populations.

**Next step.** Superseded after Astra's review of `ee5cf3b` by §13.13, which keeps the whole-test
objective and separates composition, detection and uncertainty.

### 13.13 After the probe: composition, detection, uncertainty (proposal, not run)

**Status:** proposed by Claude Code on `ee5cf3b`, after Astra accepted §13.12; statically reviewed
by Astra on `d88853a`, rectified after it and implemented in §13.14 (records before §1); not run.
The adopted direction stands: a guarantee on the whole test (§11;
`15` A2). Nothing here selects a tolerance, risk level, sample size, policy or calibration, or
presents extending the contested cost to active items as sufficient.

**What the probe established for C3.** The whole-form estimate was computed: `DTF_F` of the fixed
form — all eight items, in one fit — was 0 in the 6 converged runs, for a generating DTF of
0.56–0.65. No item was contested, so the contested cost `D` was absent, not zero. Three questions
are kept apart.

- *Composition, which items enter the check.* In C3 it was complete: the whole-form functional
  covered every item. Extending the contested cost to the active items would compute this same
  functional, 0 here, so it is not sufficient by itself. Composition stays open across fits — a
  form drawn from several batches, the cross-fit sum (§4 row 2), common against per-form
  calibration (§9) — which this proposal does not address.
- *Detection, why a fit returns one class and a zero difference.* With one class every between-class
  contrast is zero by construction: `DTF_F = 0` says the selected model has no classes to compare,
  not that classes have equal curves. In every C3 run `latent_dif` fitted both two-class candidates,
  shared and per-class slopes — 3 candidates with the one-class fit, as in C0 and C5 — and set them
  aside; a mixture replaces the one-class fit only if it converges and lowers the BIC (`02` §B.3).
  The probe recorded the selected fit and the candidates' count, not what the candidates estimated
  nor how much their likelihood gained. It cannot tell a gain below the penalty from no gain at this
  size, or a candidate at another solution.
- *Uncertainty, what would separate a form balanced enough from one the data cannot evaluate.* A
  zero from a one-class selection is neither: it states nothing about how large a difference the
  data could have shown. Diagnostically, the evidence is the scale at which the candidates'
  statistics vary under the null at the same design (C0, paired seeds) and where C3 falls against
  it. For a guarantee it is §12's all-order region with obligations a and c (§11.2), whose
  usefulness §13.8 limits for small spread DIF. "Balanced enough" stays relative to the symbolic
  tolerance `δ`.

**The two `max_iters` runs.** In C3 r0 and r2 the selected fit — the one-class fit, which the
search keeps unless a converged mixture lowers the BIC — reached its iteration budget unconverged;
every item read `Indeterminate` and the forms were empty. They belong with B1–B3 (search and
optimizer stability, the shape and its penalty; `15`); no cause is attributed here. The experiment
below records each candidate's status on those seeds, which can locate the non-convergence among
the candidates, not explain it.

**One bounded experiment.**

| Item | Content |
|---|---|
| Question | On the probe's own confirm seeds, do the target model's candidates hold a between-class signal for C3, and for C5, that C0's do not? |
| Comparison | C0, C3 and C5 at the 8 confirm replicates, paired by seed; §13.11's design, model, defaults and gate |
| What is new | the probe kept only the selected fit; this keeps every candidate: classes, slope alternative, convergence status, log-likelihood, parameter count and, for each mixture candidate, the whole-form `DTF_F` over its counted classes |
| Built-in check | the fits are deterministic (invariant 7): each re-fit's selected model must reproduce its record — rechecks, classes, status, `DTF_F` — or the run stops as a technical failure |
| Metrics, per run and paired with C0 by seed | `2 Δ log L` of the best converged two-class candidate over the one-class fit, beside its penalty `Δk ln N`; each converged mixture candidate's `DTF_F` of the fixed form, beside the generating DTF; each candidate's status, notably on C3's two `max_iters` seeds. Counts "`k` of `r`" and per-run values only; denominators: runs, candidates fitted, candidates converged |
| Needs | a read-only accessor in `scoring::latent` returning every candidate's fit — no change to selection, flags or any output of `latent_dif` — and the harness recording it: a change to `crates/scoring` for the owner to authorize explicitly |
| Cost, estimated | 24 runs at the confirm costs of these three cases (21–24 s a run): about 9 CPU-minutes, under §13.11's caps and supervisor |

**Possible consequences, with no threshold** (rectified after `d88853a`). The experiment is an
exploratory look at the first probe's own batches, refitted on their seeds: no independent
confirmation, and every reading below is about those 24 batches.

- Comparisons are read only where both the one-class baseline and the candidate converged. A
  candidate against a non-converged baseline, or a non-converged candidate, is listed apart; its
  gain is no evidence about the penalty.
- C3's comparable candidates separate from C0's on the paired seeds — larger likelihood gains,
  mixture `DTF_F` nearer the generating DTF: the batches carry a between-class signal that the
  selected order does not show. That does not by itself place the cause in the BIC: the penalty,
  the moment penalty, the candidates' starts, a candidate at another solution and the search's stop
  are all on the path. A whole-test check would still need every admitted order with its
  uncertainty (§11.2 b, as §12's region does); the BIC is not retuned on this evidence.
- C3's comparable candidates resemble C0's: on these batches the candidates show no signal that
  C0's do not. That does not show the information absent, nor the model generally unable: other
  starts, sizes or estimators are untried, and a §13.8-type bound for C3-like alternatives would be
  a separate question. A whole-test check must still be able to report a form it cannot evaluate.
- Mixed results, or C5 separated: reported as observed, run by run.

In every case composition and the guarantee's obligations remain open; the experiment describes
where these fits lose the signal, and supplies no bound.

**D38, apart.** The probe showed what the implemented balancing does with sourced facts: in C2 it
left out every uncompensated one. The open choice is semantic. When a contested fact's class
difference reflects pertinent factual knowledge, the whole-test target either treats it as
differential functioning to compensate, D38 as implemented, or counts it as measured competence,
outside the difference the target constrains. The second needs the target to tell pertinent
knowledge from nuisance, which the responses do not identify: only review and the source check can
mark it (`01` D44). The choice changes which contrasts the whole-test target counts, not the
experiment above, and is not taken here.

### 13.14 Candidates' diagnostics: implementation and declaration (not run)

**Status:** implemented by Claude Code on `d88853a` under the owner's authorization: the read-only
change to `scoring::latent` and the harness, committed as `2e2572c`; approved by Astra on `2e2572c`
for this exploratory look (records before §1); its 24 runs authorized and run (§13.15). A2 stays
open and blocking; D38 is not decided.

**What changes in `scoring::latent`.** `latent_dif_with` now calls the same search with no
collector; `latent_dif_candidates` calls it with one and returns the identical fit with every
candidate the search fitted, in its order. Per candidate, from its chosen start (`Candidate`):
order and slope alternative, status, unpenalized log-likelihood, the BIC's parameter count, BIC,
`Verdict` (`Selected`, `Superseded`, `NotConverged`, `BicNotLower`, by the current rule) and the
candidate's parameters as a fit without posterior. The final assembly is factored into one function
with the same expressions. Search, starts, order of operations, random stream, stops, thresholds,
selection and flags are untouched, and no fit is added. Limit: the other starts of a candidate are
not kept, nor candidates the search did not try.

**What the harness adds.** Phase `candidates` in `crates/characterization`: C0, C3 and C5, the 8
confirm replicates, under keys `candidates-…` and confirm's seeds reused on purpose (`Task::seed`).
Its run applies `latent_batch`'s gate, its checks in its order (`probe::gate`, built on
`protocol::pilot`'s public admissions), then `latent_dif_candidates` with `latent_dif`'s settings;
`protocol` is unchanged. The record is the first probe's record of the same fit, plus a
`candidates` array: each candidate's fields above, its classes counted at `MIN_CLASS_SHARE`, its
proportions, `gain = 2 (log L − log L₁)`, `penalty = (k − k₁) ln N`, `delta_bic` and whether the
one-class fit converged, and its whole-form `DTF_F` — a number, `0` with "one counted class", or
`null` "unavailable". The candidate check now allows exactly one file to differ from `6274270`,
`crates/scoring/src/latent.rs`, and only for `candidates` and smoke; explore and confirm still
require `6274270`'s engine; the plan records that file's blob. Supervisor, audit and stops are the
reviewed ones. `sim/a2_probe.py` adds `compare-first` and `candidates`.

**Checks run here** (E). `cargo test --release -p characterization --test latent_candidates`, kept
outside `crates/scoring` so that `latent.rs` stays its only change: 2 tests on four small batches
(`N` 400–600, no gate) — the diagnostic fit equals the ordinary one in every field, floats by their
bits; each candidate matches the search's list, its BIC, the rule's verdict; exactly one is
selected, the returned fit; a mixture was selected in at least one batch. `cargo test --release -p
scoring --test golden`: its two pinned latent fits (`N = 1,500`) unchanged. `cargo test -p
characterization --test a2_probe`, 12 tests: the phase's identity and seeds; `probe::gate` refusing
as `latent_batch` on six refusal branches, error for error, before any fit — the admitted branch
shown equal by reading only; the diagnostic record equal to the ordinary record of the same fit plus
its candidates, on one smoke fit outside the gate. `--test a2_probe_jobs`, 19 tests, the supervisor
unchanged. `python3 -m unittest … test_a2_probe.py`, 21 tests. The smoke phase end to end;
`candidates` refused on an uncommitted tree and with 4 replicates. `cargo fmt`, `cargo clippy
--all-targets` (scoring, characterization) and the comment budget. No full suite, fitted test or fit
at `N = 3,000`.

**Declaration, before any run.**

| Item | Content |
|---|---|
| Runs | 24: C0, C3 and C5 at the 8 confirm replicates of §13.12, their seeds; `N = 3,000`, §13.11's design and defaults |
| Identity | phase `candidates`, its own plan, ledger and records in a new directory; §13.12's evidence is not written to |
| Reproduction check, binding | `compare-first`: every record's `schema`, seeds, design, items, truth, KR-20, outcome, fit, rechecks, fixed form and readings equal to the first probe's confirm record of its case and replicate; keys, phase, time, memory and candidates excluded. Any difference or missing twin stops the analysis: no candidate is read |
| Metrics, per run | per candidate: verdict, status, `gain`, `penalty`, `delta_bic`, classes counted, proportions, whole-form `DTF_F` or why none; beside the run's generating DTF |
| Paired comparison | per two-class variant — shared and per-class slopes, each on its own — the gain minus C0's at the same replicate, only where both baselines and both candidates converged; otherwise listed "not comparable", with no reading of the penalty |
| Denominators | runs and runs read per case; candidates fitted and converged; comparable pairs per variant |
| Other stops | the precision check; the audit not complete; the supervisor's stops |
| Cost, estimated | the first probe's confirm costs for these cases — C0 21.5 s a run (10.9–30.4), C3 22.4 (16.1–28.6), C5 21.1 (12.9–26.5) — give about 520 s, 8.7 CPU-minutes; collecting adds one parameter copy and one assembly per candidate, negligible beside the fit. On 8 workers about 1.5 minutes of wall time if they do not slow one another, no guarantee. Each run's peak near §13.12's 6.6 MiB |
| Caps, binding | CPU 30 minutes (`--max-cpu-s 1800`), wall 15 minutes (`--max-wall-s 900`), 10 minutes a run, 2 GiB aggregate resident, 8 workers; cumulative in the directory, no extension, any stop refusing a resumption |
| Not covered | other starts, sizes, models or estimators; frequencies; the BIC's change; D38; any bound |

**Commands, after review and authorization.**

```
cargo run --release -p characterization --bin a2-probe -- run --phase candidates --jobs 8 --max-cpu-s 1800 --max-wall-s 900 --out characterization-results/a2-candidates
python3 sim/a2_probe.py check characterization-results/a2-candidates
python3 sim/a2_probe.py compare-first characterization-results/a2-candidates verification/reports/a2-probe-9511ce1/records.jsonl
python3 sim/a2_probe.py candidates characterization-results/a2-candidates verification/reports/a2-probe-9511ce1/records.jsonl
```

### 13.15 Candidates' diagnostics: result (run on `2e2572c`)

**Status:** run by Claude Code on 2026-10-19 on `2e2572c`, as §13.14 declares, after Astra's
approval for this exploratory look and the owner's authorization of its 24 runs; execution and
numbers verified by Astra on `54d44fe`, its readings rectified after that verification (records
before §1). Evidence: **E** the run; **C** the oracle's check and the reproduction check; the raw
records, plan, ledger, checks and reports in
[`verification/reports/a2-candidates-2e2572c/`](../verification/reports/a2-candidates-2e2572c/README.md).
An exploratory look at the first probe's own 24 batches: no independent confirmation, no
frequency, no bound. A2 stays open and blocking; D38 is not decided.

**Execution and preconditions.** Candidate checked: `HEAD` = `origin` = `2e2572c`, no later commit,
`crates/scoring/src/latent.rs` the engine's only difference from `6274270`, a clean tree, a new
directory. The declared command, with its caps: exit 0, 24 of 24 runs `read`, no stop. Every
condition held before any candidate was read: `check` exit 0 (312 values, largest `|A − B|`
`1.9·10⁻¹⁵`); the audit of phase `candidates` complete, 24 records of 24, no duplicate, missing,
failed or killed run; `compare-first` exit 0 against the first probe's records, 24 compared, no
missing twin, no declared field differing. At full size, then, the diagnostic path reproduced the
first probe's outputs for these batches. The candidates report exited 0.

**Cost, measured.** CPU 599.4 s, 10.0 of the 30 minutes; wall time 90.2 s of the 15 minutes;
largest sampled resident sum 53.2 MiB (54,480 KiB), each run's peak 6.2–6.5 MiB; largest gap
between checks 150 ms; 8 workers. CPU a run, mean and range: C0 24.8 s (13.6–35.6), C3 25.9
(18.4–32.5), C5 24.2 (14.7–30.5), for an estimate of 21–22 s: 10.0 CPU-minutes for an estimated
8.7. The same fits took 21–22 s a run in §13.12: their time varies, their outputs do not.

**Results.** Every run fitted three candidates — one class; two classes with shared slopes; two
with per-class slopes — and selected the one-class fit; every two-class candidate was set aside
as "BIC not lower", so no three-class candidate was tried. Ranges are over comparable candidates:
the one-class baseline and the candidate both converged.

| Case (generating DTF) | Baseline converged | Variant | Converged; comparable | Gain `2 Δ log L` | Penalty `Δk ln N` | `Δ BIC` | Two classes counted; their `DTF_F` | Gain minus C0's, paired: range; positive |
|---|---|---|---|---|---|---|---|---|
| C0 (0) | 8 of 8 | shared | 8; 8 | 10.65–26.46 | 80.06 | 53.61–69.41 | 4 of 8; 0.313–0.433 | — |
| | | per-class | 8; 8 | 22.58–43.57 | 144.11 | 100.55–121.54 | 7 of 8; 0.104–0.736 | — |
| C3 (0.560–0.652) | 6 of 8; r0, r2 `max_iters` | shared | 8; 6 | 11.88–19.59 | 80.06 | 60.47–68.19 | 5 of 6; 0.467–0.871 | −8.83 to +3.51; 3 of 6 |
| | | per-class | 8; 6 | 23.42–44.67 | 144.11 | 99.45–120.70 | 5 of 6; 0.540–1.358 | −12.88 to +7.15; 3 of 6 |
| C5 (0.132–0.154) | 8 of 8 | shared | 8; 8 | 10.46–24.96 | 80.06 | 55.11–69.60 | 4 of 8; 0.444–0.741 | −8.62 to +9.57; 4 of 8 |
| | | per-class | 8; 8 | 25.55–41.81 | 144.11 | 102.31–118.57 | 6 of 8; 0.182–0.734 | −6.88 to +4.58; 3 of 8 |

Every two-class candidate has two nominal classes; where one held under 5% it counted one, and its
`DTF_F` is 0 by construction, "one counted class"; no `DTF_F` was unavailable. Proportions are in
the records and the report. C3's r0 and r2 are listed apart: their baseline did not converge,
both two-class candidates did, with `DTF_F` 0.52–0.97; their gains are not read.

**Readings, proportionate** (rectified after the verification of `54d44fe`).

- *Likelihood gains.* On these batches the paired gains of C3's comparable two-class candidates show
  no consistent ordering against C0's: 3 of 6 differences positive in each variant. C3's shared
  gains (11.88–19.59) lie within C0's (10.65–26.46); its per-class gains reach 44.6667, above C0's
  maximum, 43.5671. C5: 4 of 8 and 3 of 8 positive. This holds for the gains alone, and shows
  neither the information absent nor the model generally unable: other starts, sizes and estimators
  are untried.
- *The candidates' whole-form `DTF_F`* does not share that reading. Paired by replicate, C3's is
  above C0's in 5 of 6 comparable pairs in each variant, zeros from a single counted class included
  (§13.16). With shared slopes, the values with two classes counted are 0.313–0.433 in C0 (4 of 8
  runs, the other 4 zero) and 0.467–0.871 in C3 (5 of 6 comparable runs, 1 zero). These ranges are
  descriptions of these runs, not a threshold. The point value is neither an accurate measure — it
  reaches 0.74 under C0's null and 1.36 for a generating DTF of 0.56 in C3 — nor a certificate; its
  possible diagnostic use remains to be checked (§13.17).
- *Penalty.* Every two-class gain (10.5–44.7) stayed below its penalty (80.06 and 144.11), `Δ BIC`
  53.6–121.5. With no consistent ordering of the gains, this does not show a signal the penalty
  discards; nothing here calls for a change of the BIC, and none is made.
- *C3's two `max_iters` runs:* the one-class fit did not converge while both two-class candidates
  did. This locates the non-convergence in the baseline on those two batches — B1–B3 — and
  attributes no cause.

**Against §13.13's three questions** (rectified after `54d44fe`). Composition is unchanged by this
look. Detection: the selected fit's whole-form estimate is 0 in C3's six converged runs; among the
non-selected candidates the likelihood gains do not order C3 against C0, while their whole-form
`DTF_F` does in 5 of 6 comparable pairs, post hoc, on these batches. Uncertainty: neither shows how
large a difference these data could resolve, and no conclusion follows here on whether a check on
this model at this size can or cannot evaluate forms like C3's; the earlier deduction that it must
report them as not evaluable is withdrawn.

**Limits.** The 24 batches are §13.12's: exploratory, 8 replicates a case, no rate. One start per
candidate kept; only the candidates tried; one design (`N = 3,000`, 60 anchors, eight open items);
no bound, no change to the BIC, the model or the search. Questions left open: whether the
candidates' DTF ordering recurs on independent seeds (proposed in §13.17, not run), whether C3-like
spread DIF is distinguishable at this size at all, and whether another size or estimator
separates it.

### 13.16 Paired re-reading of the candidates' DTF (post hoc, exploratory)

**Status:** an analysis by Claude Code on `54d44fe`, after the results, of the records preserved in
`verification/reports/a2-candidates-2e2572c/` (`paired_dtf.py`, its output `paired-dtf.json`); not
reviewed. No fit and no run; the original artefacts are unchanged. Post hoc and exploratory: it was
not declared before the results, and the two variants refit the same batches, so they are not
independent replicates.

Per replicate, each two-class candidate's whole-form `DTF_F`, its classes counted in brackets;
`0 (1)` is the zero of a single counted class. A pair is comparable when C0's and C3's one-class
baselines and both candidates converged; every C0 fit and every C3 candidate converged here.

| Replicate | Shared: C0 | Shared: C3 | C3 against C0 | Per-class: C0 | Per-class: C3 | C3 against C0 |
|---|---|---|---|---|---|---|
| r0 | 0 (1) | 0.681 (2), baseline `max_iters` | not comparable | 0.736 (2) | 0.793 (2), baseline `max_iters` | not comparable |
| r1 | 0.313 (2) | 0.723 (2) | greater | 0.323 (2) | 0.540 (2) | greater |
| r2 | 0 (1) | 0.975 (2), baseline `max_iters` | not comparable | 0.435 (2) | 0.521 (2), baseline `max_iters` | not comparable |
| r3 | 0 (1) | 0.871 (2) | greater | 0.104 (2) | 0.790 (2) | greater |
| r4 | 0.380 (2) | 0.629 (2) | greater | 0.570 (2) | 0.645 (2) | greater |
| r5 | 0.433 (2) | 0 (1) | smaller | 0.303 (2) | 1.358 (2) | greater |
| r6 | 0 (1) | 0.743 (2) | greater | 0.448 (2) | 0 (1) | smaller |
| r7 | 0.418 (2) | 0.467 (2) | greater | 0 (1) | 0.978 (2) | greater |

C3 against C0: greater in 5 of 6 comparable pairs, smaller in 1, with shared and with per-class
slopes; zeros are counted as values. Excluding the pairs with a zero would leave 3 of 3 greater
(shared) and 4 of 4 (per-class) — a smaller and a selected denominator, given only beside the
counts above. C5 against C0, from the same file: shared 4 greater, 1 smaller, 3 equal (both zero) of
8; per-class 3 greater, 4 smaller, 1 equal of 8. No threshold, rate or test is drawn from these
counts.

### 13.17 Next check: the candidates' DTF on independent seeds (proposal, not run)

**Status:** proposed by Claude Code on `54d44fe` after Astra's verification; approved by Astra on
`55373cb` as a descriptive check on new data (record before §1); implemented in §13.18, run in
§13.19.
It asks one question that §13.16 raised after the fact, on fresh data, without changing the
model, the BIC, the search or any threshold.

| Item | Content |
|---|---|
| Question | Does the ordering of §13.16 — C3's two-class candidates' whole-form `DTF_F` above C0's on the same seed — recur on seeds independent of the first probe's? |
| Design | C0 and C3, §13.11's design and defaults (`N = 3,000`, 60 anchors, eight open items), paired by seed; 16 replicates each, seeds from a new phase tag, none shared with §§13.12–13.15 |
| Path | the reviewed diagnostic path, `latent_dif_candidates` behind the gate replica; a new harness phase for the new tag and the two cases, `crates/scoring` unchanged |
| Primary metric | per variant, among comparable pairs: C3's `DTF_F` greater than, smaller than or equal to C0's, zeros from a single counted class counted as values |
| Pre-declared beside it | the same counts with the pairs holding a zero left out, their denominator stated; each `DTF_F` beside the generating DTF; the paired likelihood gains, as in §13.15; counted classes and proportions |
| Zeros and non-convergence | a zero from one counted class is a value of 0 with its basis recorded; `DTF_F` unavailable, or any of the pair's four fits not converged, makes the pair not comparable: listed with its statuses, not ordered |
| Denominators | runs and runs read per case; candidates fitted and converged; comparable pairs per variant; pairs with a zero |
| Reading | counts per variant set beside §13.16's 5 of 6, as a description; no threshold drawn from the observed ranges, no test with a decision rule, no rate. A recurring ordering would not make the point value a measure or a certificate; its absence would not show the information missing |
| Budget, proposed | 32 runs at §13.15's 24–26 s a run: about 14 CPU-minutes; caps CPU 30 minutes, wall 15 minutes, 10 minutes a run, 2 GiB aggregate resident, 8 workers; the reviewed supervisor, audit and stops |
| Needs | the harness phase and its tests; review of that code; the owner's authorization of the runs |

Possible consequences, with no threshold: if the ordering recurs, the candidates' `DTF_F` would
merit a declared study of its diagnostic properties — its null spread, its relation to the
generating DTF, its dependence on size — still with no certificate; if it does not, §13.16's counts
are read as a feature of those 24 batches. Either way A2's obligations (§11.2) remain, and D38 stays
apart.

### 13.18 The check on independent seeds: implementation and declaration (not run)

**Status:** implemented by Claude Code on `55373cb` under the owner's authorization of the harness
phase and small tests; reviewed by Astra on `89b7163`, its preliminary check corrected, the
correction approved on `821f52d` for this descriptive check (records before §1). Run as declared
in §13.19. A2 stays open and blocking; D38 stays apart.

**Phase `dtf-check`** (`crates/characterization`). C0 and C3, 16 replicates each, paired by seed:
the two cases of a replicate share its seed, `probe_seed` under the new tag `dtf-check`, fixed
before any run, and every seed differs from smoke's, explore's, confirm's and `candidates`'. Keys
`dtf-check-…`, its own plan, ledger and directory; earlier evidence is not written to. Its runs
take the reviewed diagnostic path — `probe::gate`, then `latent_dif_candidates` — and the reviewed
supervisor, caps, audit and numerical check. `--replicates` other than 16 is refused.

**Engine check.** `candidates` and `dtf-check` now require `crates/scoring` and `crates/protocol`
identical to `2e2572c`, the reviewed diagnostic engine — no longer any `latent.rs` with the rest
as at `6274270`. Explore and confirm still require `6274270`'s engine; smoke, which fits nothing,
keeps its allowance on `latent.rs` alone. No file of `crates/scoring` or `crates/protocol` changes.

**Analysis, declared before any run** (`python3 sim/a2_probe.py dtf-check DIR`). It first requires
the declared phase exactly: the plan declaring 16 replicates; its tasks the 32 of C0 and C3 at
replicates 0 to 15, no other; every record of the phase matching its key's phase, case and
replicate; C0's seed equal to C3's at each replicate; the audit complete on that set — 32 records
of 32, no duplicate, missing, failed or killed run. Then the numerical check must pass. Otherwise
it stops with exit 1, printing every reason (with `--json` too), and orders no pair. Runs the gate
refused and fits not converged, observed as planned, are not stops: they are reported as below.
The analysis itself, `paired_dtf(D, phase, pairs)`, rereads another phase apart (the historical
check below) under the audit and numerical check alone; that reading is not the command.
`compare-first` does not apply: the new batches have no twins in the first probe. Then, for each
variant apart, shared and per-class slopes:

| Item | Rule |
|---|---|
| Pair | C0 and C3 at the same replicate, each by its two-class candidate of that variant |
| Ordered | C3's `DTF_F` greater than, smaller than or equal to C0's, on the unrounded values |
| Zeros | in the primary counts as values, each with its origin (`dtf_basis`: one counted class, or counted classes) |
| Not comparable | either baseline or candidate not converged, a candidate missing, `DTF_F` unavailable, or a run without a reading: listed with the reason per side, not ordered |
| Denominators | the 16 planned pairs; the comparable ones; the not comparable, by reason |
| Conditional reading | the same counts over the comparable pairs with no zero, stated as conditional on that subset and its denominator |
| Beside each pair | both `DTF_F`, C3's generating DTF, classes counted, proportions, statuses, and C3's gain minus C0's |
| Not done | summing the variants, which refit the same batches; a threshold from §§13.15–13.16's ranges; a success rule set after the results; more replicates |

The reading sets the counts per variant beside §13.16's 5 of 6, as a description (§13.17).

**Caps, binding.** CPU 30 minutes (`--max-cpu-s 1800`), wall 15 minutes (`--max-wall-s 900`), 10
minutes a run, 2 GiB aggregate resident, 8 workers. A stop authorizes no recovery and no new run.
Cost, estimated from §13.15's 24–26 s a run: about 13–14 CPU-minutes.

**Checks run here** (E). `python3 -m unittest discover -s sim -p test_a2_probe.py`: 29 tests, eight
on this phase, their constructed records declaring 16 replicates. Two count: ordering on unrounded
values (a difference of `10⁻¹²` reads "greater"), zeros with their origin and the conditional
reading; every reason a pair is not comparable, and the denominators. Five stop with exit 1 and
their reason, also with `--json`: a plan of two runs, declaring 16 or 8 replicates; a plan of 32
with one key substituted (`dtf-check-C3-16` or `dtf-check-C5-15`) or declaring 8; a record whose
replicate or case differs from its key; C3's seed differing from C0's; an incomplete audit or a
failed numerical check. The first four fail on `89b7163`'s `sim/a2_probe.py`, exiting 0 with an
analysis. The eighth rereads `2e2572c`'s preserved records with `paired_dtf(D, "candidates", 8)`:
`paired-dtf.json`'s C3 rows and §13.16's counts — per variant 5 greater and 1 smaller of 6
comparable, C3's two unconverged baselines not comparable, 3 of 3 (shared) and 4 of 4 (per-class)
without zeros, a check of the code, not evidence for §13.17's question — and `dtf-check` stops on
them. On `89b7163`, unchanged since: `cargo test -p characterization --test a2_probe`, 13 tests,
one new — the phase's 32 runs, its pairing, its 16 seeds new against smoke's, explore's and
confirm's first 64 and `candidates`'; `--test a2_probe_jobs`, 20 tests, one new — the binary plans
the 32 runs and refuses 8 replicates; `run --phase dtf-check` refused on an uncommitted tree
before any directory; the engine checks by `git diff`: identical to `2e2572c`, different from
`6274270`; `cargo fmt` and `cargo clippy --all-targets`. On the correction, the comment budget
again. No fit at `N = 3,000`, full suite or campaign.

**Commands, after review and authorization** (`D` a new directory,
`characterization-results/a2-dtf-check-<reviewed commit>`).

```
cargo run --release -p characterization --bin a2-probe -- run --phase dtf-check --jobs 8 --max-cpu-s 1800 --max-wall-s 900 --out D
python3 sim/a2_probe.py check D
python3 sim/a2_probe.py dtf-check D
python3 sim/a2_probe.py dtf-check D --json
```

### 13.19 The check on independent seeds: result (run on `821f52d`)

**Status:** run by Claude Code on 2026-10-19 on `821f52d`, as §§13.17–13.18 declare, after Astra's
approval of the corrected harness for this descriptive check and the owner's authorization of its
32 runs (record before §1). The results were accepted by Astra on `307530a` as bounded diagnostic
evidence, with the reading recorded before §1 and below; one limit rectified. Evidence: **E** the
run; **C** the
oracle's check; the raw records, plan, ledger, checks and reports in
[`verification/reports/a2-dtf-check-821f52d/`](../verification/reports/a2-dtf-check-821f52d/README.md).
A descriptive check on 16 new seeds a case: no rate, no test, no bound, no certificate. A2 stays
open and blocking; D38 is not decided.

**Execution and preconditions.** Candidate checked: `HEAD` = `origin` = `821f52d` after a fetch,
`crates/scoring` and `crates/protocol` identical to `2e2572c`, `crates` and `Cargo.lock` clean, a
new directory. The authorized command — §13.18's, with the per-run and resident caps written out
at their declared values, `--max-run-s 600 --max-rss-mib 2048`; the plan records all four caps:
exit 0, 32 of 32 runs `read`, no stop.
Before any reading: `check` exit 0 (416 values, largest `|A − B|` `1.7·10⁻¹⁵`); the declared
phase's contract met — 16 replicates, the 32 tasks, records matching their keys, seeds paired —
and its audit complete, 32 records of 32, no duplicate, missing, failed or killed run. Both
`dtf-check` commands exited 0. `compare-first` does not apply (§13.18).

**Cost, measured.** CPU 645.3 s, 10.8 of the 30 minutes; wall time 91.1 s of the 15 minutes;
largest sampled resident sum 53.3 MiB (54,544 KiB), each run's peak 6.2–6.5 MiB; largest gap
between checks 155 ms; 8 workers. CPU a run, mean and range: C0 20.5 s (10.7–28.3), C3 19.9
(14.8–29.2), for an estimate of 13–14 CPU-minutes.

**Fits.** Every run fitted the three candidates and selected the one-class fit, converged; all 32
one-class baselines converged. Every two-class candidate converged but C0's per-class one at r10
(`max_iters`), set aside as not converged; every other was set aside as "BIC not lower": gains
11.38–45.72 with shared slopes against a penalty of 80.06, 24.41–73.10 with per-class slopes
against 144.11. Every zero is the zero of one counted class — one of the two classes under 5%,
`DTF_F` 0 by construction; no `DTF_F` was unavailable. C0's generating DTF is 0, C3's 0.569–0.628.

**Results, per variant** (`dtf-check-report.md`, `dtf-check.json`). Values to three decimals,
classes counted in brackets, `0 (1)` the zero of one counted class; "share" the smaller class's
proportion; the gain is C3's `2 Δ log L` minus C0's, read on comparable pairs only. Every baseline
and candidate converged unless a status is shown.

Shared slopes: 16 pairs planned, 16 comparable.

| Replicate | C3 generating DTF | C0 `DTF_F` | C0 share | C3 `DTF_F` | C3 share | C3 against C0 | Gain minus C0's |
|---|---|---|---|---|---|---|---|
| r0 | 0.598 | 0.138 (2) | 0.459 | 0.412 (2) | 0.329 | greater | +1.07 |
| r1 | 0.616 | 0.841 (2) | 0.286 | 1.114 (2) | 0.294 | greater | +23.20 |
| r2 | 0.595 | 0 (1) | 0.020 | 0 (1) | 0.017 | equal | −3.90 |
| r3 | 0.601 | 0 (1) | 0.017 | 0.793 (2) | 0.061 | greater | −5.16 |
| r4 | 0.606 | 0.495 (2) | 0.327 | 0.555 (2) | 0.359 | greater | +7.20 |
| r5 | 0.615 | 1.127 (2) | 0.053 | 1.324 (2) | 0.061 | greater | +4.71 |
| r6 | 0.627 | 0.614 (2) | 0.144 | 0.631 (2) | 0.217 | greater | +3.06 |
| r7 | 0.619 | 0.109 (2) | 0.112 | 0.129 (2) | 0.166 | greater | +0.88 |
| r8 | 0.569 | 0.117 (2) | 0.414 | 0.506 (2) | 0.489 | greater | −5.54 |
| r9 | 0.583 | 0.229 (2) | 0.099 | 0 (1) | 0.041 | smaller | +5.66 |
| r10 | 0.591 | 0.353 (2) | 0.248 | 0.745 (2) | 0.060 | greater | +0.68 |
| r11 | 0.628 | 0 (1) | 0.048 | 0.436 (2) | 0.445 | greater | −13.80 |
| r12 | 0.574 | 0 (1) | 0.023 | 0.658 (2) | 0.066 | greater | +2.56 |
| r13 | 0.614 | 0 (1) | 0.050 | 0.066 (2) | 0.193 | greater | +2.11 |
| r14 | 0.575 | 0.579 (2) | 0.402 | 0.740 (2) | 0.403 | greater | +5.72 |
| r15 | 0.573 | 0 (1) | 0.040 | 0.775 (2) | 0.104 | greater | +1.02 |

Per-class slopes: 16 pairs planned, 15 comparable; not comparable 1, r10, C0's candidate not
converged.

| Replicate | C3 generating DTF | C0 `DTF_F` | C0 share | C3 `DTF_F` | C3 share | C3 against C0 | Gain minus C0's |
|---|---|---|---|---|---|---|---|
| r0 | 0.598 | 0.248 (2) | 0.089 | 0.227 (2) | 0.084 | smaller | −5.88 |
| r1 | 0.616 | 0.726 (2) | 0.241 | 1.145 (2) | 0.243 | greater | +13.77 |
| r2 | 0.595 | 0.274 (2) | 0.435 | 0.312 (2) | 0.112 | greater | −12.47 |
| r3 | 0.601 | 0.610 (2) | 0.167 | 0 (1) | 0.044 | smaller | +4.75 |
| r4 | 0.606 | 0.398 (2) | 0.080 | 0.417 (2) | 0.445 | greater | −3.06 |
| r5 | 0.615 | 0 (1) | 0.029 | 0 (1) | 0.017 | equal | +1.06 |
| r6 | 0.627 | 0 (1) | 0.043 | 1.282 (2) | 0.084 | greater | −1.35 |
| r7 | 0.619 | 0 (1) | 0.040 | 1.035 (2) | 0.065 | greater | +2.97 |
| r8 | 0.569 | 0.523 (2) | 0.070 | 0.604 (2) | 0.125 | greater | −1.36 |
| r9 | 0.583 | 0.888 (2) | 0.054 | 0.846 (2) | 0.057 | smaller | −5.55 |
| r10 | 0.591 | 0.439 (2), `max_iters` | 0.055 | 0.641 (2) | 0.123 | not comparable | — |
| r11 | 0.628 | 0.281 (2) | 0.067 | 0.648 (2) | 0.124 | greater | +0.69 |
| r12 | 0.574 | 0.712 (2) | 0.062 | 0 (1) | 0.036 | smaller | +3.15 |
| r13 | 0.614 | 0.807 (2) | 0.087 | 0.766 (2) | 0.093 | smaller | +17.95 |
| r14 | 0.575 | 0.683 (2) | 0.133 | 0.938 (2) | 0.120 | greater | +12.84 |
| r15 | 0.573 | 0 (1) | 0.043 | 0 (1) | 0.048 | equal | +9.01 |

C0's share at r13 (shared) is 0.049998, under 5% by `2.5·10⁻⁶`: one class counted.

| Variant | Planned; comparable | C3 greater, smaller, equal | Pairs with a zero | Without zeros, conditional: pairs; greater, smaller, equal | Gain minus C0's: positive; range |
|---|---|---|---|---|---|
| shared | 16; 16 | 14, 1, 1 | 7 (C0 6, C3 2, both at r2) | 9; 9, 0, 0 | 12 of 16; −13.80 to +23.20 |
| per-class | 16; 15 | 8, 5, 2 | 6 (C0 4, C3 4, both at r5 and r15) | 9; 6, 3, 0 | 9 of 15; −12.47 to +17.95 |

The equal pairs are pairs of zeros. With two classes counted, `DTF_F` spans 0.109–1.127 in C0 (10
runs) and 0.066–1.324 in C3 (14) with shared slopes; 0.248–0.888 in C0 (12) and 0.227–1.282 in C3
(12) with per-class slopes. The variants refit the same batches: they are not summed.

**Beside §13.16, descriptively.** On the first probe's batches C3's candidate `DTF_F` was greater
than C0's in 5 of 6 comparable pairs in each variant, 3 of 3 and 4 of 4 without zeros. On these 16
new seeds: with shared slopes 14 greater, 1 smaller, 1 equal of 16, 9 of 9 without zeros; with
per-class slopes 8 greater, 5 smaller, 2 equal of 15, 6 greater and 3 smaller of 9 without zeros.
The two variants read differently on the same batches; the difference is described, not explained.
No threshold or success rule was declared, and none is set after the results: whether this is
§13.17's "ordering recurring", and what follows from it, is left to the review.

**Readings, proportionate.**

- *Not a measure.* Under C0's null the candidates' `DTF_F` reaches 1.127 (shared) and 0.888
  (per-class); in C3 it spans 0.066–1.324 for generating values of 0.569–0.628. The point value
  estimates neither the generating DTF nor a bound; §13.15's ranges are no reference — C0's shared
  maximum was 0.433 there.
- *Gains and penalty.* The paired gain differences are positive in 12 of 16 (shared) and 9 of 15
  (per-class) comparable pairs; every two-class gain stayed below its penalty and every run kept
  the one-class fit. Nothing here calls for a change of the BIC, the model or the search; none is
  made.
- *What does not follow.* A repeated ordering would not certify the DTF of a form; a different one
  would not show the information absent in general. The check does not bear on composition or on
  D38.

**Reading approved on `307530a`** (record before §1). With shared slopes C3's predominance over C0
reappears descriptively on the new data, 14 greater, 1 smaller, 1 equal; with per-class slopes the
result is more mixed, 8 greater, 5 smaller, 2 equal, one pair not comparable. The variants are not
summed. These counts do not measure a classifier's accuracy on a single test; the point value is
shown neither an accurate estimate of the generating DTF nor a guaranteed bound; no change to the
BIC or the selector is justified by this check alone.

**Limits** (rectified after the review of `307530a`). One design (`N = 3,000`, 60 anchors, eight
open items), C0 and C3 only, 16 replicates; the default `LatentParams` — one start for the
one-class baseline, four seeded starts for each two-class candidate, of which the diagnostics keep
only the chosen solution; only the candidates tried; no rate, interval, test or bound; replicates
not extended. No further experiment is started.

### 13.20 Next study: the shared-slope candidate's DTF on a single test (proposal, not implemented)

**Status:** proposed by Claude Code on `307530a` at the owner's direction; reviewed by Astra on
`53f2427`, its data separation accepted and four points rectified below (record before §1). The
owner's configuration and the implementation are in §13.21; not run. §13.10's "75%" is no target.
A2 stays open and blocking; D38 stays apart.

**Indicator, chosen from data already seen.** `I` is the whole-form `DTF_F` of the two-class
candidate with shared slopes, as `latent_dif_candidates` returns it on the reviewed path
(`LatentParams::default()`, four seeded starts, the chosen solution kept), and 0 when one class is
counted. It is chosen because that variant ordered C3 above C0 in §13.16 (5 of 6) and §13.19 (14
of 16) where the per-class one was mixed (8 of 15): a choice made after those results, which can
have suggested it and cannot validate it. The per-class candidate stays in the records, unread.
The selector is unchanged: `I` reads a candidate the BIC set aside in every run so far.

**Question.** Given one batch alone, with no C0 batch on the same seed, how well does `I` separate a
neutral case from an unbalanced one? §§13.16 and 13.19 compared each C3 batch with the C0 batch
sharing its seed; a single test has no such counterpart. On §13.19's batches, read unpaired, the
values overlap — C0's reach 1.127, C3's include 0.066 and two zeros — so the answer is open.

**Data.** No threshold is drawn from any range observed so far.

| Set | Content | Use |
|---|---|---|
| Development | every run already seen: the smoke runs, §13.12's 54, §13.15's 24, §13.19's 32 | chose `I`, the cases and this design; never evaluates a threshold |
| Calibration | new seeds under their own tag | fixes a threshold `t` by a rule declared before it runs; `t` committed before validation starts |
| Validation | new seeds under another tag, disjoint from every earlier one | evaluates `t` once, as fixed; a revision after its results is a new exploratory iteration with new validation seeds (§13.10, item 4) |

**Cases** (§13.11's design and defaults: `N = 3,000`, 60 anchors, eight open items).

| Case | Role | Why |
|---|---|---|
| C0 | neutral: no DIF, no impact | false alarms under the plain null |
| C1 | neutral: an ability difference of 1, no DIF | a legitimate difference in the measured ability must not be flagged (`01` D44); a false alarm here reads impact as bias |
| C3 | unbalanced: diffuse DIF, form DTF 0.607 [0.567, 0.641] | the case `I` was chosen on; detections and misses |
| C5 | unbalanced, weak: form DTF 0.143 [0.133, 0.151] | just above `DTF_MAX = 0.10`, where a miss matters and detection is hardest: a weak-signal scenario. §13.11's indicative `TV` of 0.077–0.120 against C0 concerns the item draws computed there, not every new batch, and bounds probabilities, not the frequencies a study observes (rectified after `53f2427`) |

C2 and C4 stay out to contain the design (rectified after `53f2427`); characterizing them
numerically would not by itself require D38's semantic choice, which stays apart.

**Outcomes, declared before any run** (rectified after `53f2427`). Each planned run ends in
exactly one outcome: *flagged* (`I > t`); *not flagged* (`I ≤ t`, zeros included, with their
origin); *not evaluable*, a scientific observation — the gate refused the batch, the one-class
baseline or the shared candidate did not converge, the candidate is missing or its `DTF_F`
unavailable — with its reason. A run missing, failed or killed is a technical failure, not an
outcome: it leaves its stage incomplete, which stops that stage's analysis; the stops and the
absence of automatic resumption are kept.

| Case | Flagged | Not flagged | Not evaluable |
|---|---|---|---|
| C0, C1 | false alarm | correct non-flag | counted apart, by reason |
| C3, C5 | detection | miss | counted apart, by reason |

Denominators: the planned runs, which the three outcomes sum to; then the evaluable runs, every
rate over them labelled conditional. No case is pooled with another, C0 with C1 or C3 with C5, and
no outcome leaves the counts. Each proportion carries an exact (Clopper–Pearson) 95% interval, a
description of sampling error on these designs, not a guarantee; the zeros of `I` are counted per
case beside.

**Size, precision and cost.** Per case, over `n` evaluable runs:

| `n` | 95% half-width at ½ (normal approximation) | Exact 95% interval, `k/n` | Exact 95% interval, `k/n` | CPU at 20–26 s a run |
|---|---|---|---|---|
| 50 | ±0.139 | 0/50: [0, 0.071] | 2/50 (4%): [0.005, 0.137] | 17–22 min |
| 100 | ±0.098 | 0/100: [0, 0.036] | 5/100 (5%): [0.016, 0.113] | 33–43 min |
| 200 | ±0.069 | 0/200: [0, 0.018] | 10/200 (5%): [0.024, 0.090] | 67–87 min |

The CPU a run spans the per-case means of C0, C1, C3 and C5 in §§13.12–13.19, 19.9–25.9 s; wall time
is about a seventh of it with 8 workers (§13.19: 645 s in 91 s). A difference of two proportions,
such as C5's detections against C0's false alarms, has a half-width of ±0.139 at 100 runs a case
when both are near ½ (about 770 a case for ±0.05), and less when they are small: ±0.085 at 100 and
±0.060 at 200 for 0.05 against 0.17. (A claim of precision resting on 0.12 as C5's maximal excess is
withdrawn after `53f2427`.) A threshold at the (1 − α) quantile of `m` neutral calibration runs
rests on about `α·m` runs above it (§13.21 fixes the rank: the 96th of 100). Examples, with the rule
calibrated on the neutral cases and one `n` for both stages: `n = 100`, 200 calibration runs and 400
validation runs, 3.3–4.3 CPU-hours, 28–37 minutes of wall time; `n = 200`, 1,200 runs, 6.7–8.7
CPU-hours, 56–73 minutes. A rule weighing misses adds C3, and C5 if chosen, to calibration: 700 or
800 runs at `n = 100`. Every run is a fit at `N = 3,000`; runs this long belong on the owner's
machine, with explicit authorization.

**Choices still needed, not adopted here** (the owner's, made after `53f2427`, are in §13.21).

1. The rule for `t`: a false-alarm target α on the neutral cases (the (1 − α) quantile over C0 and
   C1 apart, or the larger of the two), or a weight `w` between false alarms and misses on a
   calibration including C3 (and C5 or not); the value of α or `w`.
2. Whether a band without decision around `t` is allowed; its runs would be a fourth outcome,
   counted apart.
3. How a consumer would treat *not evaluable* — as a flag, a pass or its own outcome; the study
   reports it apart whatever is chosen.
4. `n` per case and stage, the caps, and whether cases share seeds by replicate (harmless to each
   rate; it correlates the case estimates).
5. Whether validation adds a threshold-free description of separation, such as the unpaired
   probability that a C3 run's `I` exceeds a C0 run's, ties at zero counted ½.

**What this indicator is not.** Even validated, `I` with `t` would be an empirical screening
statistic whose error frequencies hold for these four synthetic designs, at this size, inside the
model's own family. It is not §11.2's guarantee: no region `𝒞(X)`, no bound `U(X)`, no uniform
finite-sample statement over `Ξ`; a candidate at its chosen start's local optimum bounds no
supremum (§11.2 c). It neither uses nor extends §12's coverage, which concerns the region, not a
point value. A2's open obligations — domination, conservative computation, usefulness, cost,
decision and scope (§13.6) — and the whole-test objective remain. A flag would not prove a form
biased, nor its absence a form fair. It changes no selection, admission or compensation: no
protocol path reads it, and any operational use would need its own decision.

**Needs, if approved.** A harness phase per stage, with seed-disjointness and contract checks as
in §13.18; an unpaired reading in `sim/a2_probe.py` with its tests; review; the owner's choices
above; authorization of calibration and then, with `t` committed, of validation.

### 13.21 The single-test study: declaration and implementation (not run)

**Status:** declared and implemented by Claude Code on `53f2427`, under the owner's configuration
and direction; approved by Astra on `7bbfddb` for the calibration alone (records before §1). The
calibration ran in §13.22, and the validation, against it, in §13.23. The order rule below was
fixed before the code. "No calibration or validation run, no fit at `N = 3,000`", here and under
the checks below, describes the state at the implementation, on `7bbfddb`. A2 stays open and
blocking: this
study replaces none of the whole-test guarantee's obligations (§13.20, "What this indicator is
not"). D38 stays apart.

**Configuration (the owner's).**

| Item | Fixed |
|---|---|
| Indicator | `I`, the `DTF_F` of the two-class candidate with shared slopes (§13.20), on the reviewed diagnostic path; `crates/scoring` and `crates/protocol` exactly as at `2e2572c` |
| Calibration, phase `ind-cal` | C0 and C1, 100 runs each |
| Validation, phase `ind-val` | C0, C1, C3 and C5, 100 runs each |
| Seeds | by stage, case and replicate: none shared between stages, cases or earlier probes |
| Target | nominal α = 0.05 on the neutral cases |
| Threshold | one: the larger of the thresholds set apart on C0 and C1 |
| Band | none |
| Not evaluable | reported apart, with no operational behaviour given to it |

**Order rule for the threshold, fixed before the code.**

1. *Values.* For each neutral case `c` of the calibration, its evaluable runs give `n_c` values of
   `I`, the doubles exactly as recorded; zeros are values and ties are kept.
2. *Rank.* `k_c = ⌈(1 − α)(n_c + 1)⌉ = ⌈19 (n_c + 1) / 20⌉`, computed in integers.
3. *A case's threshold.* If `k_c ≤ n_c`, `t_c` is the `k_c`-th smallest value, ties counted: with
   100 evaluable runs, the 96th. If `k_c > n_c` — fewer than 19 evaluable runs — the case gives no
   finite threshold and the study stops before validation, with no fallback value.
4. *Threshold.* `t = max(t_C0, t_C1)`.
5. *Decision.* A validation run is flagged when `I > t`, strictly: `I = t` is not flagged, and since
   `I ≥ 0` and `t ≥ 0`, neither is a zero.

**What α = 0.05 is.** A nominal target, not a guarantee obtained. If a case's evaluable calibration
values and a new evaluable value of the same case were exchangeable, the rank would make the
probability that the new value exceeds `t_c` at most `1 − k_c / (n_c + 1)` — 5/101 at `n_c = 100`
— ties only lowering it, and `t ≥ t_c` lowering it further. That is a property of the rule under
that hypothesis, marginal over calibration and validation; it does not bound the frequency
observed given the realized `t`, and says nothing outside these designs. Validation measures the
frequencies.

**Outcomes and stops** (§13.20, rectified). Gate refusals, a baseline or shared candidate not
converged, a missing candidate and an unavailable `DTF_F` are observations, counted per case by
reason. A run missing, failed or killed, or a stop, leaves its stage incomplete: calibration then
yields no threshold and validation no reading. Nothing resumes automatically; another session
needs the owner's authorization.

**Calibration's reading** (`python3 sim/a2_probe.py threshold D`). It requires the plan declaring
100 replicates, its tasks exactly `ind-cal-C0-0` to `ind-cal-C1-99` each with its declared seed,
records matching their keys, the audit complete and the numerical check passing. It then prints
`threshold.tsv`: schema, rule, per case the planned and evaluable runs, `k_c` and `t_c`, the
not-evaluable counts by reason, `t`, and the SHA-256 of `records.jsonl`, `plan-ind-cal.tsv` and
`ledger.tsv`; every double as its 64 bits in hexadecimal and its shortest decimal. Otherwise it
prints why to standard error, nothing to standard output, and exits 1.

**Freezing.** The calibration's plan, ledger, records and `threshold.tsv` are copied to
`verification/reports/a2-ind-cal-<commit>/` and committed after review: that commit identifies the
threshold and the data it came from. Validation starts only after it.

**Validation's binding** (`a2-probe run --phase ind-val --calibration DIR`). Before any run the
harness refuses:

- no `--calibration`, or `--calibration` with another phase;
- `DIR`'s four files not tracked by git, modified, or beside untracked files;
- a `threshold.tsv` not in the declared form: its schema, rule or α; a rank other than
  `⌈19 (n + 1) / 20⌉` or above `n`; counts not summing to 100; `t` not the larger `t_c`; a decimal
  other than its bits;
- a SHA-256 other than its calibration file's;
- a calibration plan other than the declared one: phase, 100 replicates, the 200 tasks and seeds;
- a calibration configuration — candidate trees, record schema, designs, constants — other than
  the validation's own; only the caps may differ.

It then writes into the validation plan a `calibration` line with `threshold.tsv`'s SHA-256 and
`t`'s bits, and the supervisor's plan check binds the directory to them. The harness does not
recompute `t`; the reading below does.

**Validation's reading** (`python3 sim/a2_probe.py ind-val D CAL [--json]`, `CAL` the frozen
directory). It stops with exit 1, printing why (also with `--json`), unless the validation's plan,
400 tasks, seeds and records are the declared ones and its audit complete; the numerical check
passes on both stages; `t` recomputed from `CAL` by the rule gives `threshold.tsv` byte for byte;
the validation plan's `calibration` line names that file and `t`; the two configurations agree but
for the caps and that line. Then, per case, never pooled:

| Item | Content |
|---|---|
| Counts | planned; not evaluable by reason; evaluable; flagged and not flagged — false alarms and correct non-flags in C0 and C1, detections and misses in C3 and C5; zeros of `I` |
| Proportions | not evaluable over planned; flagged over evaluable, labelled conditional; flagged over planned beside it |
| Intervals | Clopper–Pearson, exact two-sided 95%, each marginal. Hypotheses: a case's runs independent and identically distributed given the frozen `t`, so the count is binomial; conditional on `t` and, over evaluable runs, on their number; no adjustment across intervals; sampling error in these designs, not a guarantee |
| Separation without threshold | C3 and C5 each against C0 and C1 each: the share of pairs of evaluable runs where the unbalanced run's `I` exceeds the neutral one's, ties counted ½; conditional on evaluable; four values, no interval |
| Not done | pooling cases; changing `t`, the rule or a reading after validation; a band; an operational meaning for not evaluable; a success rule |

**Seeds.** A run of `ind-cal` or `ind-val` takes the first 8 bytes, little-endian, of the SHA-256
of `isegoria/a2-probe/v1`, the stage's name and the case's, each after its length, and the
replicate: no two runs share a batch, across stages, cases and the earlier phases. The Python
reading recomputes every planned seed; `ind-cal-C0-0` takes 14817136108943293105 and `ind-val-C5-99`
12356750456368884976.

**Plan, cost and caps, proposed for review** (not authorized).

| Stage | Runs | CPU at 19.9–25.9 s a run | Wall, about a seventh | Caps proposed: CPU; wall |
|---|---|---|---|---|
| Calibration | 200 | 3,980–5,180 s (66–86 min) | 9–12 min | 2 h (7,200 s); 30 min (1,800 s) |
| Validation | 400 | 7,960–10,360 s (2.2–2.9 h) | 19–24 min | 4 h (14,400 s); 60 min (3,600 s) |
| Both | 600 | 3.3–4.3 h | 28–37 min | 6 h; 90 min, cumulative |

Each stage also 10 minutes a run, 2 GiB aggregate resident, 8 workers; the caps are cumulative
over the sessions of the stage's directory (§13.11). The longest run so far took 35.6 s, each run's
peak about 6.5 MiB; the estimate assumes an otherwise idle machine. A stop authorizes no
resumption, seed change or new run.

**Implementation.** `crates/characterization`: phases `ind-cal` and `ind-val` in `probe.rs`, on
`gate_and_diagnose` as `dtf-check`, with `case_seed`; `probe_calibration.rs`, the check of
`threshold.tsv` and of the calibration's plan, data and configuration (`calibration_line`, `rank`);
in `bin/a2-probe.rs`, 100 replicates and no other, `--calibration`, the freeze check by git and the
`calibration` line, the engine check as for `dtf-check`. `sim/a2_probe.py`: `probe_seed` and
`case_seed` recomputed, `indicator`, `rank`, `case_threshold`, `calibrate` and `threshold`,
`clopper_pearson`, `separation`, `ind_val` and `ind-val`. No file of `crates/scoring` or
`crates/protocol` changes; the supervisor, its caps, audit and stops are reused as they are.

**Commands, after review and authorization** (`C` and `V` new run directories, `F` the frozen copy
`verification/reports/a2-ind-cal-<commit>/`).

```
cargo run --release -p characterization --bin a2-probe -- run --phase ind-cal --jobs 8 --max-cpu-s 7200 --max-wall-s 1800 --max-run-s 600 --max-rss-mib 2048 --out C
python3 sim/a2_probe.py check C
python3 sim/a2_probe.py threshold C > C/threshold.tsv
# copy C's plan, ledger, records and threshold.tsv to F; review; commit
cargo run --release -p characterization --bin a2-probe -- run --phase ind-val --calibration F --jobs 8 --max-cpu-s 14400 --max-wall-s 3600 --max-run-s 600 --max-rss-mib 2048 --out V
python3 sim/a2_probe.py check V
python3 sim/a2_probe.py ind-val V F
python3 sim/a2_probe.py ind-val V F --json
```

**Checks run here** (E), at the implementation. No calibration or validation run then, no fit at
`N = 3,000`.

- `cargo test -p characterization --test a2_probe`: 14 tests, one new — the stages' 200 and 400
  runs, 600 distinct seeds, none of smoke's, explore's or confirm's first 64, `candidates`' or
  `dtf-check`'s, two known values.
- `--test a2_calibration`, new: 3 tests — the rank (96 at 100, 19 at 19, none at 18); the declared
  calibration bound whatever the caps; refusals for a data file changed after the threshold, `t`
  not the larger, a wrong rank, 18 evaluable runs, counts not summing, another α, a decimal unlike
  its bits, a plan of 2 runs, a changed seed, another candidate tree or design, no threshold file.
- `--test a2_probe_jobs`: 23 tests, three new — the binary plans 200 and 400 runs and refuses 50
  replicates; `ind-val` without `--calibration`, with an unfrozen one, and `--calibration` with
  `ind-cal` refused before any directory; in a scratch git repository a committed calibration
  passes the freeze check (the next refusal is the engine's), and one changed or beside an
  untracked file is refused.
- `python3 -m unittest discover -s sim -p test_a2_probe.py`: 36 tests, seven new, at 22 replicates
  a case with the quadratures memoized — seeds equal to the harness's; the rule's rank, ties and
  zeros; exact intervals against their closed forms and the values reviewed on `53f2427`;
  separation with ties ½; calibration's lines, not-evaluable counts and digests; its stops on a
  killed run, 18 evaluable runs and a changed seed, with nothing on standard output; validation
  per case — a value equal to `t` not flagged, a gate refusal and a non-convergence counted apart —
  and its stops on a killed or missing run, an unbound plan, another configuration and an edited
  `threshold.tsv`, also with `--json`.
- Once, not kept: a `threshold.tsv` written by `sim/a2_probe.py` for a constructed calibration of
  100 runs a case gave, in `calibration_line`, the line the oracle computes, and another
  configuration was refused.
- `run --phase ind-cal` refused on an uncommitted tree before any directory; `crates/scoring` and
  `crates/protocol` identical to `2e2572c`; `cargo fmt`, `cargo clippy --all-targets`, the
  comment budget.

**Limits.** The harness checks the threshold's form and identity, not its value: `t` is recomputed
from the records by the validation's reading alone. The freeze check's positive path was exercised
in a scratch repository, not on a real calibration. Costs are estimates from §§13.12–13.19. The
study's results would hold for these four designs at this size, inside the model's family, with
none of §11.2's or §12's guarantees.

### 13.22 The single-test study: calibration (run on `7bbfddb`)

**Status:** run by Claude Code on 2026-10-19 on `7bbfddb`, as §13.21 declares, after Astra's
approval limited to the calibration and the owner's authorization of its 200 runs (record before
§1). Accepted by Astra on `185ec93` as the application of the declared rule, not as a validation
result (record before §1); the validation ran against it in §13.23. Evidence: **E** the run; **C**
the oracle's check, the threshold reading and a recomputation apart from it;
the records, plan, ledger, threshold and checks in
[`verification/reports/a2-ind-cal-7bbfddb/`](../verification/reports/a2-ind-cal-7bbfddb/README.md).
Calibration proportions are not validation results. A2 stays open and blocking; D38 is not
decided.

**Execution and preconditions.** Candidate checked: `HEAD` = `origin` = `7bbfddb` after a fetch,
`crates/scoring` and `crates/protocol` identical to `2e2572c`, `crates` and `Cargo.lock` clean, a
new directory. The authorized command with its caps: exit 0, 200 of 200 runs `read`, no stop.
`check` exit 0: 2,600 values, largest `|A − B|` `4.3·10⁻¹⁴`; the audit complete, 200 records of
200. `threshold` exit 0, which it gives only with the declared stage — 100 replicates, the 200
tasks and seeds, records matching their keys — a complete audit and a passing check; its output
went to a temporary file, renamed `threshold.tsv` after the exit code was read, with nothing on
standard error. Run on the frozen copy, `threshold` gives the same file byte for byte.

**Cost, measured.** CPU 4,790.6 s, 79.8 of the 120 minutes; wall time 609.1 s, 10.2 of the 30
minutes; largest sampled resident sum 54.3 MiB (55,564 KiB), each run's peak 6.2–6.7 MiB; largest
gap between checks 150 ms; 8 workers. CPU a run, mean and range: C0 23.2 s (10.0–38.4), C1 24.6
(10.9–39.2), against an estimate of 66–86 CPU-minutes and 9–12 minutes of wall time.

**Threshold** (`threshold.tsv`; `rank-check.txt` recomputes it from the records apart from
`sim/a2_probe.py`, with the same values). Ranks `k − 3` to `k + 3` around each case's threshold:

| Case | Planned; observed; evaluable | Not evaluable | Zeros of `I` | `k` | `t_c` | Values at ranks `k − 3` … `k + 3` | Equal to `t_c`; above |
|---|---|---|---|---|---|---|---|
| C0 | 100; 100; 98 | 2: one-class baseline `max_iters` (r26, r72) | 21 | 95 | 0.7984473698688216 (r67) | 0.6716, 0.7669, 0.7904, **0.7984**, 0.8446, 0.8939, 0.9843 | 1, itself; 3 |
| C1 | 100; 100; 98 | 2: one-class baseline `max_iters` (r43, r85) | 23 | 95 | 0.7164196166579272 (r65) | 0.6807, 0.6827, 0.7144, **0.7164**, 0.7757, 0.8373, 0.8819 | 1, itself; 3 |

`k = ⌈19 · 99 / 20⌉ = 95` in each case. `t = max(t_C0, t_C1) = 0.7984473698688216` (bits
`3fe98ce17fa53de3`), C0's. No tie sits at either threshold, so 98 − 95 = 3 values lie above each:
a property of the rank at `n = 98`, not a false-alarm frequency. The gate refused no batch. The
four not-evaluable runs are the one-class baselines that did not converge — their shared
candidates converged, and the rule leaves them out unread. Every shared two-class candidate
converged and was set aside as "BIC not lower"; the selected fit kept one class in all 200 runs.
Every zero is the zero of one counted class.

**What is fixed, and what is not known.** The threshold and the data it came from are identified by
`threshold.tsv` and its three digests, and are frozen when this directory is committed. Nothing is
known yet about false alarms on new neutral runs or about detections in C3 and C5: that is the
validation's question, on its own seeds. No rule, rank, seed or metric changed after the result.

**Needs, before any validation.** Review of this calibration; the owner's authorization of
`ind-val`, run with `--calibration verification/reports/a2-ind-cal-7bbfddb` from a commit whose
`crates` tree and `Cargo.lock` are this calibration's (`b618aaa`, `e661541`) — the harness refuses
another configuration — with this directory committed and unmodified.

### 13.23 The single-test study: validation (run on `185ec93`)

**Status:** run by Claude Code on 2026-10-19 on `185ec93`, as §13.21 declares, against the
calibration frozen there, after Astra's acceptance of the calibration and the owner's authorization
of the 400 runs (record before §1). The results were accepted by Astra on `ccb8f05` within the
declared scope, with the reading recorded before §1 and below; A2 is not closed. Evidence: **E** the
run; **C** the
oracle's check, the declared reading and a recomputation apart from it, its intervals by SciPy; the
records, plan, ledger, checks and readings in
[`verification/reports/a2-ind-val-185ec93/`](../verification/reports/a2-ind-val-185ec93/README.md).
No new threshold, no pooling of cases, no success criterion. A2 stays open and blocking; D38 is not
decided.

**Execution and preconditions.** Candidate checked: `HEAD` = `origin` = `185ec93` after a fetch;
its `crates` tree `b618aaa` and `Cargo.lock` `e661541`, the calibration's; `crates/scoring` and
`crates/protocol` identical to `2e2572c`; `verification/reports/a2-ind-cal-7bbfddb/` committed
and unmodified before and after; a new directory; the calibration's binary. The authorized
command: the harness accepted the calibration — frozen, in form, its digests, plan and
configuration — and bound the plan to it (`calibration`, `threshold.tsv`'s SHA-256
`34fb80c3…cd840`, bits `3fe98ce17fa53de3`); exit 0, 400 of 400 runs `read`, no stop. `check`
exit 0: 5,200 values, largest `|A − B|` `4.0·10⁻¹⁴`; the audit complete, 400 records of 400.
`ind-val` exit 0 as text and as JSON, which it gives only with the declared stage — 400 tasks and
seeds, records matching their keys — both stages' checks passing, the threshold recomputed from
the calibration equal to the frozen file byte for byte, the plan bound to it and the two
configurations alike but for the caps.

**Cost, measured.** CPU 8,553.9 s, 142.6 of the 240 minutes; wall time 1,083.4 s, 18.1 of the 60
minutes; largest sampled resident sum 53.9 MiB (55,236 KiB), each run's peak 6.2–6.7 MiB; largest
gap between checks 145 ms; 8 workers. CPU a run, mean and range: C0 21.3 s (7.5–35.8), C1 21.5
(6.5–34.8), C3 21.3 (9.4–38.9), C5 21.4 (9.7–35.0), against an estimate of 133–173 CPU-minutes
and 19–24 minutes of wall time. With the calibration: 13,344.5 s CPU (222.4 minutes) of the 6
hours, 28.2 minutes of wall time of the 90.

**Results, per case** (`ind-val-report.md`, `ind-val.json`; `val-check.txt` recomputes every count,
zero and share from the records and the frozen threshold, with the same values, and every interval
by SciPy, within `1.4·10⁻¹⁶`). `t = 0.7984473698688216`; flagged when `I > t`; no value equals `t`.

| Case | Role | Planned; observed; evaluable | Not evaluable, by reason | Flagged | Not flagged | Zeros of `I` |
|---|---|---|---|---|---|---|
| C0 | neutral | 100; 100; 99 | 1: candidate `max_iters` (r77) | 5 false alarms | 94 correct non-flags | 30 |
| C1 | neutral | 100; 100; 99 | 1: baseline `max_iters` (r9) | 2 false alarms | 97 correct non-flags | 21 |
| C3 | unbalanced | 100; 100; 96 | 4: baseline `max_iters` (r0, r58, r90), candidate `max_iters` (r91) | 23 detections | 73 misses | 18 |
| C5 | unbalanced | 100; 100; 96 | 4: baseline `max_iters` (r13, r36, r48, r49) | 4 detections | 92 misses | 30 |

| Case | Flagged over evaluable, conditional; 95% | Flagged over planned; 95% | Not evaluable over planned; 95% |
|---|---|---|---|
| C0 | 5/99; [0.017, 0.114] | 5/100; [0.016, 0.113] | 1/100; [0.000, 0.054] |
| C1 | 2/99; [0.002, 0.071] | 2/100; [0.002, 0.070] | 1/100; [0.000, 0.054] |
| C3 | 23/96; [0.158, 0.337] | 23/100; [0.152, 0.325] | 4/100; [0.011, 0.099] |
| C5 | 4/96; [0.011, 0.103] | 4/100; [0.011, 0.099] | 4/100; [0.011, 0.099] |

Intervals as declared: Clopper–Pearson, exact two-sided 95%, each marginal; a case's runs taken as
independent and identically distributed given the frozen `t`, the count binomial; conditional on
`t` and, over evaluable runs, on their number; no adjustment across the twelve; sampling error in
these designs, not a guarantee.

**Separation without threshold**, conditional on evaluable runs, ties counted ½ — the share of
pairs where the unbalanced run's `I` exceeds the neutral one's: C3 against C0, 0.7209 (96 × 99
pairs); C3 against C1, 0.7563; C5 against C0, 0.4737; C5 against C1, 0.5001.

The false alarms' values: C0 1.375 (r71), 1.053 (r17), 1.048 (r10), 0.954 (r73), 0.870 (r1); C1
0.984 (r95), 0.801 (r24). Every run kept the one-class fit; every shared candidate but the two not
converged was set aside as "BIC not lower". The generating DTF spans 0.526–0.662 in C3 and
0.130–0.158 in C5. The not-evaluable runs are reported apart, as neither passes nor flags.

**Readings, proportionate.**

- *Neutral cases.* 5 false alarms in 99 evaluable C0 runs and 2 in 99 C1 runs, with intervals
  [0.017, 0.114] and [0.002, 0.071]. α = 0.05 was a nominal target of the rule (§13.21), not a
  guarantee; these are frequencies in these designs at this `t`.
- *C3.* 23 detections in 96 evaluable runs, [0.158, 0.337]: at this threshold most C3 runs, 73,
  are not flagged. Without a threshold a C3 run's `I` exceeds a neutral run's in about 72–76% of
  pairs. §13.19's paired ordering, 14 of 16, compared each C3 batch with the C0 batch on its seed;
  on a single test, unpaired, the separation is this one.
- *C5.* 4 detections in 96 evaluable runs, [0.011, 0.103], against C0's 5 false alarms in 99; the
  separation, 0.47 and 0.50, does not distinguish C5 from the neutral cases in these runs. This
  concerns one indicator, one size and one estimator; it does not show the information absent in
  general.
- *What does not follow.* No threshold is chosen anew and no case is pooled; no success rule is
  applied. The frequencies hold for these four synthetic designs at `N = 3,000`, inside the model's
  family; they are not §11.2's guarantee or §12's coverage, and A2's obligations (§13.6) and the
  whole-test objective remain.

**Reading approved on `ccb8f05`** (record before §1). With the frozen threshold C3 has 23
detections and 73 misses among 96 evaluable runs. The observed ordering does not make this check
sufficient for the whole-test objective. C5 shows no useful separation in these data, with no
deduction about an absence of information in general. The separation of 0.72–0.76 is not
classification accuracy.

**Limits.** One calibration and one validation; 100 runs a case; one design and its item laws; the
default `LatentParams`, the chosen start's solution kept; the indicator chosen after the
development data (§13.20). No further experiment is started.

### 13.24 The single-test trade-off: an exploratory reading (post hoc, on `ccb8f05`)

**Status:** an analysis by Claude Code on `ccb8f05` at the owner's direction, after the validation
and its review; not reviewed. Post hoc and exploratory: no fit and no run; `sim/a2_tradeoff.py`
and its tests, its output in
[`verification/reports/a2-tradeoff-ccb8f05/`](../verification/reports/a2-tradeoff-ccb8f05/README.md),
apart from the original artefacts, which are unchanged. The validation's results, threshold and
conclusions (§13.23) stand as they are. No threshold is chosen or adopted. A2 stays open and
blocking; D38 stays apart.

**What the validation now is.** This reading uses the validation's 400 records alone, with the
frozen `t` to locate its point. Reading them over every threshold turns them into development
material: any rule, threshold or indicator this reading suggests would need verifying on new data,
under a new declaration, with new seeds (§13.10, item 4; §13.20).

**Method.** For each comparison apart — C3/C0, C3/C1, C5/C0, C5/C1 — the evaluable values of the
unbalanced case and of the neutral one; the rule `I > t`, `t` descending through every distinct
value of either, then below every value, where all are flagged; at each `t`, the detections among
the unbalanced case's evaluable runs and the false alarms among the neutral case's. Ties move both
counts in one step; a zero is flagged only below every value. Not-evaluable runs stay outside the
curves and are counted per case, as in §13.23: C0 99 evaluable of 100, C1 99, C3 96, C5 96. Each
curve's trapezoidal area equals §13.21's separation, ties ½, in all four comparisons. The area
rests on 96 and 99 runs, not on the 9,504 pairs, which are not independent observations; no
interval is drawn on a curve or an area.

**Curves.** For each count of false alarms, the most detections the curve reaches with at most
that many, and the fewest false alarms reaching them (`tradeoff.json` keeps every point):

| False alarms at most, of 99 | C3/C0 | C3/C1 | C5/C0 | C5/C1 |
|---|---|---|---|---|
| 0 | 1 at 0 | 14 at 0 | 0 at 0 | 1 at 0 |
| 1 | 13 at 1 | 23 at 1 | 1 at 1 | 4 at 1 |
| 2 | 13 at 1 | 34 at 2 | 1 at 1 | 7 at 2 |
| 5 | 32 at 5 | 43 at 5 | 5 at 5 | 13 at 5 |
| 10 | 39 at 10 | 57 at 9 | 12 at 10 | 18 at 10 |
| 20 | 62 at 20 | 69 at 19 | 22 at 20 | 26 at 19 |
| 50 | 74 at 47 | 75 at 46 | 41 at 50 | 45 at 50 |
| Frozen `t` | 23 at 5 | 23 at 2 | 4 at 5 | 4 at 2 |
| `t = 0`, zeros unflagged | 78 at 69 | 78 at 78 | 66 at 69 | 66 at 78 |
| Area | 0.7209 | 0.7563 | 0.4737 | 0.5001 |

Detections are of 96 evaluable runs. Each row but the frozen one picks its threshold on these same
runs: it is in-sample and optimistic, not a rate a rule fixed in advance would reach. On C3/C0, for
instance, 32 detections at 5 false alarms against the frozen `t`'s 23, a threshold set on other
runs and for both neutral cases.

**What the trade-off suggests**, for the owner's choice; no campaign is started or declared.

- *C3.* Flagging most C3 runs costs many false alarms on these runs: 62 and 69 of 96 with 20 and
  19 of 99 neutral runs flagged, about a fifth; 74 and 75, three quarters, only with 47 and 46,
  about half. The frozen point sits at the low-false-alarm end. No threshold
  of this indicator separates C3 from the neutral cases on a single test at `N = 3,000` here: the
  trade-off is gradual, and a threshold chosen on these data would not change that.
- *Zeros.* A single counted class sets `I` to 0 in 18–30% of every case's runs (C0 30, C1 21, C3
  18, C5 30): no threshold flags a zero without flagging every zero of both cases, the last step of
  each curve.
- *C5.* The curves lie near the diagonal, areas 0.47 and 0.50: here C5 runs are flagged about as
  often as neutral ones at every threshold.
- *Paired and single.* §13.19's paired ordering, 14 of 16, and this unpaired separation, 0.72 and
  0.76, measure different things; a single test faces the second.
- *For a next step.* Used alone on one test at this size, this indicator cannot serve the
  whole-test objective: any threshold either misses most C3-like forms or flags many neutral ones.
  Directions a new declared study could take, each with its own development and validation data:
  how the separation changes with the sample size; whether another statistic of the same fits
  separates better, chosen on development data only; or whether A2's effort belongs on its other
  questions — composition and the target (§13.13) — rather than on detection. None is adopted.
