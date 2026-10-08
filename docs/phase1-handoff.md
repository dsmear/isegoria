# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions and is meant to
be enough to resume in a new chat. The specification is in `docs/01`–`docs/14`; the review
register is [`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree,
they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, owner's working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`.
- **R1 implemented and approved by Astra on `d13bf09`; A2 open.** R1's baseline: `0519626` (the
  approved diagnosis). Implemented in `8fa07dd66698ecf1c23141b942bf8ed7ad57242f`; completed by
  `d13bf0931fffe3615bd61ec1d890e7f8cff755f0`, the documentation follow-up of Astra's patch. The
  approved commit is `d13bf09`, with no blocking finding (§7).
- `81cd467`, on top of `d13bf09`, only records that approval: no design or behaviour change; not
  pushed by the session.
- **Astra's three-category candidate** (a reference outcome admissible, rejected or inconclusive
  within its term; a forecast over the three; a quadratic difference score): checked in `17` §7
  (`654dbff`). Astra's review of `654dbff` approved, **as conditional results**, the ternary
  derivation, its binary reduction and the pointwise bound `[−1, 1]`; B-b's derivation (fixed
  denominator, joint invariance, positive probability of conclusion); the abstract constructions'
  calculations within their stated limits. **Not approved: §7 as a whole, any implementation.**
  §7 rectified in `825cee3` (the vector IPW's path conditions, B-b's conditional target and the
  case `c = 0`, the calibrations, constructions sufficient rather than unique). **Astra approved
  `825cee3`: `17` §7 accepted as a conditional analysis, not as a protocol approved for
  implementation** (the diff read; the IPW identity checked with rational arithmetic on a
  construction with dependent draw and observed outcome meeting the corrected conditions; no Rust
  or fit run). B-b is the main candidate, the ternary score an alternative.
- **B-b's candidate contract: `17` §8 (`44f0dbd`, rectified in `fa11791`), approved by Astra on
  `fa11791` as a conditional analysis; as a protocol not approved, not implemented.** It defines
  the contribution per outcome, the cases, consolidation, the counts and the joins with the
  reputation. Astra's review of `44f0dbd` approved, **as conditional results**: B-b's identity
  for a prefixed cohort; the algebraic sufficiency of the identities on the `A` and `R`
  coordinates for the mean; the pointwise bound and the zero on conclusive verdicts; the
  distinctions between pending, terminal inconclusiveness and non-selection, and between `N_u`,
  `O_u` and `V_u` (the diff, the dossier and the relevant code read; exact rational calculations
  on the counterexamples and the costs; no Rust or fit run). **Not approved by that review: §8 as
  a whole, the protocol, the implementation, the adoption of A.** Direction: A the main analytic
  reference for deepening B-b, C kept as the comparison even if A proves sustainable; no adoption
  or expenditure approved.
- **Rectifications of §8 after `44f0dbd`, made in `fa11791`** (documentation only):
  consolidation (the final estimator of a prefixed cohort apart from availability, from
  conditioning on consolidation and from the cohorts consolidated by a time; Astra's
  counterexample, −5/16 against −1/16); a normalizing weight `ω` apart from the inclusion
  probability, `S` as a selection indicator, the observed and reference outcomes apart, C1's role;
  A's guarantees narrowed; the costs with their denominator (A is not the largest increment).
- **Astra's review of `fa11791`** (it verified `fa11791` published and HEAD): `17` §8 **approved as
  a conditional analysis, with no blocking finding**. Accepted: prefixed cohorts, availability and
  the selection of the consolidated cohorts; algebraic weights apart from inclusion
  probabilities; the delimitation of A's guarantees; the costs and their correct denominators.
  **Not approved: the protocol, the implementation, the adoption of A.** A stays the main analytic
  reference, C the comparison even if A proves sustainable. Evidence: the diff and the documents
  read; exact rational calculations on the counterexamples and the costs; no Rust or fit run. The
  approval of §8 as an analysis is this review's, not that of `44f0dbd`. Recorded in `17` (status,
  §8, §8.5's direction), the A1 row of `15` (which now lists the reviews of `654dbff`, `825cee3`,
  `44f0dbd` and `fa11791`) and this note; `17` §8.4's "without H-e it is not that either" now
  reads "without H-e that unbiasedness is not guaranteed".
- **Decision synthesis: `19` §6.1** (documentation only, written on `fa11791`, committed in
  `cebcb2e`). It links A1 (B-b's forecast and how the outcome is produced), A2 (the DTF target),
  B1–B3 and resources; keeps observation (A against C) apart from calibration (reusable common
  against per form); gives the parametric accounting with its assumptions and the decisions still
  missing. It rests on readings and targeted static checks, no new run. **Astra approved
  `cebcb2e` as a documentation intervention, with no blocking finding** (the diff read and
  compared with the documents it cites; the parametric formulas checked algebraically; no Rust,
  fit or campaign); no protocol, calibration, adoption or expenditure approved. Its two
  precisions are made in `19` §6.1: D1 rests on H-a–H-d, completed reports and C5 included, and
  H-e is a distinct requirement of the contract, used for `17` §8.4's expected counts; the
  decisions on formats concern the items ("item formats and the anchors' substantive reference").
  A1, C2, A2 and Phase 1 stay open; R1 stays approved within its limits.
- **Common against per-form calibration: `19` §9** (documentation only, written on `cebcb2e`,
  rectified in `63b13fa`; **approved by Astra on `63b13fa` as a conditional comparison**, below).
  Complete administrations only, on §2's
  target kept symbolic: what makes each design pertinent (H1–H3 with one fit, model, selection);
  the resource counts and break-even inequalities, simple and with attempts, refusals, updates
  and uncertainty; partial administrations as extensions only; the link with D3; a conditional
  recommendation with no winner today and the minimum information to cross the boundary. No
  design, calibration or expenditure chosen.
- **Verification of `a9700fd`** (Claude Code, before Astra's review; documentation only, not
  pushed by the session). §9's counts, gates, model search, templates, nullifiers and the D3
  numbers agree with the code and the cited sources. Three statements rectified in §9: with
  confirmations of equal size in both designs the break-even is unchanged, so confirmations
  favour the per-form design only when a form drawn without estimates is checked on its own
  sample (`n''_T = 0`); the common design then saves answers only through the refused per-form
  administrations its screen avoids. Merging fits lowers the enumeration's total by one empty
  set for `n ≤ 1`. The load statement needs forms in `B` and `A ≥ a_T`. Committed in `63b13fa`.
- **Astra's review of `63b13fa`** (the diff, the dossier and the relevant code passages read;
  exact rational calculations on the combinatorial counts, the confirmations and the selection;
  no Rust, fit or campaign run): the three rectifications and §9 **approved as a conditional
  comparison**; no calibration, protocol, implementation, expenditure or realized guarantee
  approved; A2 stays open. Decided for the comparison: a form's own sample can be its check
  sample, `n''_T = 0` under conditions. Withdrawn: reading a stricter control as confirmations in
  both designs. Direction, not a policy: the probability of at least one false acceptance over a
  declared horizon, at a symbolic `α_H`, as the analytic reference.
- **Deepening after that review, `6681b03`** (documentation only): `19` §9.3 states the check
  sample's conditions, the comparison at one guarantee and precision, `v_T` (candidates verified
  per kept form) and `r_T` (attempts per candidate) with the hypotheses of their product and the
  general sum; §9.4's selection rows follow; §9.5 separates the error per candidate, over the
  search and among accepted forms, and gives the obligations of proof of two constructions under
  one guarantee: per-form checks under an error budget (A) and a common calibration with a
  simultaneous bound (B). No estimator, level, budget, sample size or winner proposed.
- **Astra's review of `6681b03`: partial** (the diff and the updated passages read; exact
  calculations on the counterexamples, the expected-share inequality and a capped search's cost;
  no Rust, fit or campaign run). Approved as conditional results: A's proof under conditional
  coverage and a budget along every path; B's result for a single calibration with simultaneous
  coverage; checks kept apart from the optimizer's internal operations; the cost identity and
  its product form under the stated hypotheses; `E[V/max(R, 1)] ≤ P(V ≥ 1)`, distinct from a
  guarantee conditional on one form's acceptance. Not approved: the deepening as a whole. The
  approval of `63b13fa` is unchanged.
- **Rectifications of `6681b03`, `b59fd03`** (documentation only), in `19` §9.5: B's updates
  under choices that depend on the history (conditional coverage given `𝓖_{u−1}` and a budget
  along every path as a sufficient construction; marginal
  coverages for a sequence fixed in advance; a joint guarantee as another route; the abstract
  `Z` counterexample); two checks of one form, `A₁ ∩ A₂` (no further budget by itself) against
  `A₁ ∪ A₂` (a new opportunity to cover), 1/400 against 39/400 under independence; `DTF_μ(T)` as
  the target, `E_F(T)` a conservative majorant under H1–H3 whose excess does not imply the
  target's.
- **Astra's review of `b59fd03`** (the commit, its parent and the published HEAD checked; the
  diff and the relevant passages read; exact calculations with fractions — 1/400 and 39/400, the
  inclusion and the union over 1,771 joint laws, the `Z` counterexample, the contrasts ¾, ½, ¾ and
  the envelope 1, a construction with adaptive levels and a shared dependence; no Rust, fit or
  campaign run; Claude Code's documentation checks not repeated): `19` §9.3–§9.5 **approved as a
  conditional analysis, with no blocking finding** — the check sample's conditions, the cost links
  at one guarantee and precision, the adaptive updates (conditional coverage and a budget along
  every path as a sufficient construction; the alternatives already stated), the mandatory
  confirmation apart from a new opportunity of acceptance, the true DTF apart from the envelope.
  No calibration, protocol, implementation, expenditure or realized guarantee approved. Its
  non-blocking precision is made in §9.3: the check needs coverage conditional on the history,
  independence being one way to obtain it.
- **One fixed form: `19` §9.6, `907c245`** (documentation only), on that review's direction.
  Under a reference simplification (one form fixed before the data, complete administrations,
  symbolic `μ`, `𝒢` and `α`, no search, no drift), it separates representation (R-a–R-f),
  identification and inference, and maps the procedure's elements to them.
- **Astra's review of `907c245`: partial** (the commit, its parent and the published HEAD
  checked; the diff, the relevant code and the theoretical references read; exact calculations on
  duplicated components, the DTF, the population KR-20 and the affine change; no Rust, fit or
  campaign run). Approved as conditional results: the separation of representation,
  identification and inference; the envelope as the supremum over the abstract family of admitted
  mixtures at fixed curves and measure; the uninformative-anchor construction within its limits;
  the rare-class argument under uniform coverage and an addable component; the scheme of a region
  with coverage and a conservative supremum. Not approved: §9.6 as a whole, a share floor in the
  target, a restriction to minimal representations, a policy on coincident components, any
  calibration or implementation. The approval of `b59fd03` is unchanged.
- **Rectifications of `907c245`, `00f8e2c`** (documentation only), in `19` §9.6: the 5% filter
  applies per component, so Astra's construction (one law, a component at 8% against two exact
  copies at 4%, population KR-20 ≈ 0.9983783235) gives a
  filtered DTF of ≈ 0.2439024373 and 0 — recorded for A2 and B1, not as a selector bug, with no
  merging, minimal representation, separation or threshold chosen; point identification kept
  apart from compatible values, valid bounds and their use, the claim that no calibration can
  bound an unidentified target withdrawn; the rare-class result with its hypotheses, its bound
  `U ≥ ½ Σ_{j∈T} (1 − c_j)` and its relation to `DTF_MAX`, the deduction that small groups must be
  left out withdrawn; the coordinate change without a double shift; the BIC consequence tied to
  the acceptance rule; regularity without universal claims; the envelope's sharpness restricted
  to the declared family; `ClassCurves::of`'s contract an observation, not a production defect.
- **Astra's review of `00f8e2c`** (the commit, its parent and the published HEAD checked; the diff
  and the updated text read; exact calculations with fractions on the standardization, the
  population KR-20 ≈ 0.9983783235, the filtered DTF ≈ 0.2439024373 against 0 and the rare-class
  consequence; no Rust, fit or campaign run; Claude Code's documentation checks not repeated):
  the rectifications and `19` §9.6 **approved as a conditional analysis**; no new target,
  threshold, component policy, calibration, implementation or realized guarantee. The approval
  belongs to this review, not to the partial one of `907c245`. Its three non-blocking precisions
  are made in §9.6: the final paragraph separates target and representation, statistical and
  computational validity, and precision, usefulness and cost; the BIC consequence is limited to a
  procedure taking the supremum over a region restricted to the selected model and accepting on
  that condition alone; a gap vanishing at an atom can give a kink, not necessarily.
- **Toward an experimental protocol: `19` §10, `26593e7`** (documentation only). On Astra's
  direction, it sorts
  A1, A2, B1–B3 and resources into decisions needed before a run, hypotheses a circumscribed study
  can measure (with what would refute them and what they would not authorize) and promises
  excluded until proved; it maps executable paths against missing ones by targeted reads. My
  recommendation, not a decision: two separable strands — S1, generative scenarios with known
  truth; S2, a field pilot on a few items fixed before the data, arm A, scores kept out of any
  reputation — and, as the next intervention, a documentation-level specification of B-b's
  records and rules for cases 5–7 under arm A, as options for Astra.
- **Astra's review of `26593e7`** (the commit, its parent and the published HEAD checked; the
  diff, the documents and the relevant passages of `run_item`, `lifecycle`, `ResultRecord`,
  `SkillTrack` and `ClassCurves::dtf` read; no new calculation needed; no Rust, fit or campaign
  run; the local working tree as Claude Code reported it, not checked through GitHub): approved
  the record of `00f8e2c`'s review, §9.6's three precisions and `19` §10 as a preparatory
  synthesis, with four precisions now made in §10 — blocks by strand (availability and human
  load do not block S1; S2 needs an executable path, possibly a study harness; delays can be
  studied without an availability guarantee; the study's end does not make a pending item a
  terminal `I`), the bridging input's meaning under `q_c` among the decisions, what
  `ClassCurves::dtf` already computes, measures refuting hypotheses only against criteria declared
  in advance. Not approved: the S2 protocol, any implementation, the start of an experiment,
  penalties or new reputational rules.
- **Cases 5–7 for a study under design A: `17` §9, `d31e9fa`** (documentation only). It keeps
  apart the procedure's term, the terminal record (existence and availability) and the study's
  observation close `Ω`; gives the logical content of a terminal-inconclusiveness record and
  separates an indeterminate attempt, exhaustion by rule and a record that never arrived, with
  intrinsic and resource non-conclusions reported apart; compares three options for a missing
  report (reported apart, a candidate penalty from `16` §4.4, kept unresolved) and three for an
  unreached freeze (pending, T58's replacement and quorum, a candidate term), each by what it
  changes; states what the study can report at `Ω` and keeps the state apart from the study's
  snapshot for late records.
- **Astra's review of `d31e9fa`: partial** (the commit, its parent and the published HEAD
  checked; the diff, the documents, T58 and the relevant lifecycle arms read; a small rational
  calculation on the baseline's dependence channel; no Rust, fit or campaign run). Approved: the
  record of `26593e7`'s review; `19` §10's four precisions; in `17` §9, the term, the terminal
  record and the study's snapshot kept apart, no silence turned into an outcome, the state's
  update apart from the historical snapshot. Not approved: `17` §9 as a whole. **Decided for the
  study's reporting specification only**: 6c and 7a as its basis, unresolved cases left
  unresolved with no imputed value — not a general answer to withholding or availability, and not
  making the consolidated cohorts alone representative. Not approved: S2's start, an
  implementation, penalties or reputational policies. T58's deepening is no prerequisite of that
  perimeter; T58 stays open for a procedure that must proceed despite missing reports.
- **Rectifications of `d31e9fa`, `b2a1c2f`** (documentation only), in `17` §9 corrected in
  place: passing the freeze, the item's progress, the
  procedure's conclusion, the terminal record's availability and every contribution's
  availability kept apart — T58 or a term guarantee none of the last three, and no count of
  rounds bounds the delay; the item, a reviewer's assignment and the cohort as distinct units —
  an item's `I` does not resolve an assigned reviewer's missing report, which gets no 0 through
  case 5; time intervals with named origins; the replacement's access to revealed reports kept
  apart from a dependence and from a violation of C5, with Astra's abstract channel (expected
  contribution 0 at `p = ½`, 3/16 at `p = ¾`, checked exactly); exhaustion shown by the rule and
  every required attempt's evidence, not by an indeterminate last attempt; a resource shortfall
  recorded positively, its classification as `I` a candidate rule, a declared invariance no
  proof; the decision's scope (no imputed values; selection, censoring and outcome-dependent
  times not removed; no D1 on consolidated cohorts alone).
- **Astra's review of `b2a1c2f`** (the commit, its parent and the published HEAD checked; the
  diff and the relevant documents read; the baseline example recalculated with rationals, 0 at
  `p = ½` and 3/16 at `p = ¾`; no Rust, fit or campaign run; the local checks and the working
  tree are Claude Code's evidence, not Astra's): the rectifications and `17` §9 **approved as the
  study's reporting specification and a conditional analysis, with no blocking finding** —
  freeze, progress, conclusion, record availability and consolidation apart; item, assignment
  and cohort apart; access to reveals, a baseline's dependence and a demonstrated violation of C5
  apart; verifiable exhaustion apart from a resource shortfall and silence; 6c and 7a imputing no
  value, the consolidated cohorts alone not representative. Not approved: an implementation,
  S2's start, penalties or new reputational policies; T58 stays open, no prerequisite of this
  specification. Its non-blocking precision is made in `17` §§9.2–9.3: `g(I) = 0` does not depend
  on `p`; a missing report's assignment stays unscored because no approved rule extends B-b to
  case 6.
- **The bridging input under B-b: `17` §10, `ac06d3c`** (documentation only): the path from a
  report to the gate; what a B-b report carries; α (`q_c` as the input) against β (an
  unconditional probability); whether the gate's decisions can move `Y`.
- **Astra's review of `ac06d3c`: partial** (the commit, its parent and the published HEAD
  checked; the diff, the contracts and the relevant code read, the harness's bootstrap path
  included; small exact calculations with fractions; no Rust, fit or campaign run; Claude Code's
  local checks not repeated, the working tree not checked through GitHub). **Approved results**,
  under the stated hypotheses: `P(Y = A | F_u) = c q_c` for `c > 0`; `q_c` alone does not fix it;
  at `c = 0` the score is indifferent among reports; the expected loss of contribution
  `c(p − q_c)²`, a property of the score, not a demonstrated measure of the reputational
  incentive or of the reviewer's overall utility. Also approved: the record of `b2a1c2f`'s
  approval and §9's precision. Not approved: §10 as a whole. **Direction**: α the main
  reference, β a comparison with its information obligations; no threshold, pool entry,
  implementation or start of the study approved.
- **Rectifications of `ac06d3c`, `111e595`** (documentation only), in `17` §10 corrected in
  place: the path split into report, ratings matrix, fit and
  side-balanced score, robust score (the bootstrap minimum, composed only by the harness), gap
  and coverage (from the full fit), first decision (`bridging_gate`'s API apart from what feeds
  it) and supplementary review (a fresh fit against the plain `τ`); the 9/10 against 9/20 example
  a difference of probabilities, its comparison with `τ ± ε` a matter of scale only; empirical
  success, conclusion and pool entry apart, so `P(Y = A | F_u)` is not an entry probability; α
  keeps the scalar format and can keep the commitment but changes the forecast's semantic
  contract; thresholds to revalidate, no change of value shown necessary; a public conclusion
  frequency need not equal `c_uj` (Astra's example, checked: 9/50 and 18/25 against 9/20), so
  `ĉ_j q_c` is a candidate indicator needing declared hypotheses; under A the prescribed set is
  fixed, not the work, samples, timing or outcome law; production, publication and reviewers'
  information apart in time; shared parameters moved only by participants of the collective fit
  (A7); a term in attempts and procedures not waiting for freezes exclude no channel by
  themselves; the claim that the waiting rule decides the channel withdrawn.
- **Astra's review of `111e595`** (the commit, its parent and the published HEAD checked; the
  diff of the four documents read; the relevant code and harness passages checked again; the
  example checked with fractions, 9/50 and 18/25 against the public product's 9/20; no Rust, fit
  or campaign run; Claude Code's local checks not repeated, the working tree not checked): the
  rectifications and `17` §10 **approved as a conditional analysis of the join between B-b and the
  bridging, with no blocking finding**. Not approved: a protocol, thresholds, pool entry, an
  implementation or S2's start. Direction: specify timing under A, α the main reference and β the
  comparison.
- **Timing under design A: `17` §11, `82bdb9a`** (documentation only): units and events, T1
  (start after the group's freezes) against T2 (produce at a report-independent position,
  disclose later), joins with the score's conditions, pending cases and anonymity.
- **Astra's review of `82bdb9a`: partial** (the commit, its parent and the published HEAD
  checked; the diff of the four documents, the relevant contracts and the replay and results
  code read; exact calculations with fractions on the timing counterexample and on a disclosure
  example; no Rust, fit or campaign run; Claude Code's local checks and working tree not
  checked). **Approved**: the record of `111e595`'s approval; the setting of the comparison
  between T1 and T2; the timing counterexample as an abstract construction (truthful 0, deviation
  1/25); the pending cases kept, and production apart from disclosure as problems to specify.
  **Not approved**: §11 as a whole; a preference for T2 as the construction adopted; a protocol,
  an implementation, thresholds, pool entry or S2's start. A and α stay analytic references
  within their limits; T1 and T2 stay alternatives.
- **Rectifications of `82bdb9a`, `cf5575a`** (documentation only), in `17` §11 corrected in
  place: a partial order, not one sequence for both
  constructions, with `Ω` able to precede production, disclosure or consolidation and a freeze
  unreached at `Ω` not a freeze that never comes; per object, production, availability (internal,
  then the replicated log), the condition authorizing disclosure and the disclosure itself — the
  identities `Δ_G = Φ_G` and `Δ_j = Φ_j` withdrawn, a record opening once available and its
  condition met, a collective opening policy kept apart from respecting the group's freeze;
  Astra's disclosure–baseline construction (0 against −¼, checked), so a moved disclosure can
  reach baselines through later reports and is not only a selection; T1 preventing the planned
  execution, not every source of information; T2 needing an information control, a fixed log
  position not fixing resources, samples or the outcome's law; D1's scope tied to the declared
  information and §8's hypotheses, the absence of a draw under A showing neither H-c nor C5; the
  respondent channel split into four questions, §7.3's statement on exclusion qualified; the
  records' evidentiary limits; the reveals' publication kept as `08` PRIV-004's known discrepancy;
  the preference for T2 withdrawn. Next: the review of §11; the per-object disclosure requirement
  is not developed before it.
- **Astra's review of `cf5575a`** (the commit, its parent and the published HEAD checked; the diff
  of the four documents read; `Node::submit`'s path checked again; the disclosure–baseline
  example checked with fractions, 0 without the signal and −¼ with it; no Rust, fit or campaign
  run; Claude Code's local documentation checks and working tree not checked): the
  rectifications and `17` §11 **approved as a conditional comparison between T1 and T2, with no
  blocking finding**. Accepted, kept apart: a partial order and one total sequence; production,
  availability, the condition of opening and the actual opening; a disclosure changing later
  reports' information and a mere selection of the observations available; the baselines'
  composition and the invariance of the reports feeding them; the theorem's perimeter and
  actions in other roles; what the records attest and what they do not show. **Not approved**:
  the adoption of T1 or T2, a protocol, an implementation, thresholds, pool entry or S2's start.
  Its non-blocking precision is made in §11.4: "check, apply, then write" is `Node::submit`'s,
  which calls `NodeState::apply`, itself writing nothing. R1 stays approved; A1, A2, B1–B3 and
  Phase 1 stay open.
- **Access and disclosure per object: `17` §12, `8af257d`** (documentation only; not covered by
  §11's approval; T1 and T2 alternatives, neither assumed adopted). Functional roles kept apart
  from subjects; six properties apart (authenticity and integrity, availability,
  confidentiality, correctness, no early disclosure, H-c's invariance) with what a record, an
  access rule or a holder's behaviour gives each; per
  object — answers and fit data, attempt statuses and reasons, intermediate results, terminal
  records, gate and pool decisions — production, readers, recipients and conditions, the log and
  what stays off it, what a record attests, the code's capabilities and gaps, T1 against T2;
  inferences from steps, reasons, positions, resource events, pool entry and other items' gate
  decisions, each with the hypothesis it may involve; per opening condition (`Φ_j`, `Φ_G`, a
  collective opening) what must be complete, what follows an object not available or a freeze
  not reached, and the boundary between groups (a partly informative signal: 0 against `−δ²`),
  with four sufficient conditions, none shown necessary, a common freeze not proposed; §9's
  decisions kept. Observations recorded, not fixed: today an object entering the replicated set
  is disclosed to every peer; `results::inputs_root` hashes leaves without salt, so it binds
  without hiding enumerable answers; a refused pilot step leaves no record, so a shortfall is
  unrecorded; pilot steps are refused before `Score`. My recommendation, not a decision: the
  common requirements first; T1 as the reference for information handling and T2 as the
  comparison, T1's timing channel through samples and time remaining. Next: Astra's review of
  §12.
- **Astra's review of `8af257d`: partial** (the commit, its parent and the published HEAD
  checked; the diff and the relevant code read; the result `−δ²` checked with fractions; the
  code's encoding and hashes reproduced in Python on a synthetic two-leaf Merkle construction; no
  Rust, fit or campaign run; the local working tree and Claude Code's documentation checks not
  checked). **Approved**: the record of `cf5575a`'s approval; roles as functions, kept apart from
  persons; the setting of the matrix per object; the partly informative signal's result under
  its stated hypotheses (`−δ²`: −1/16 at `δ = ¼`, −¼ at `δ = ½`). **Not approved**: §12 as a
  whole; an implementation, a cryptographic solution, thresholds, pool entry or S2's start.
  **Direction**: T1 the reference for the next comparison of informational requirements, T2 the
  comparison; T1 not adopted as a protocol, its timing channel open; (ii) and (iv) made concrete,
  their scope bounded; (i) may stay a declared hypothesis, no demonstration of H-c. R1 stays
  approved; A1, A2, B1–B3 and Phase 1 stay open.
- **Rectifications of `8af257d`, `7d8db08`** (documentation only), in `17` §12 corrected in
  place: the general sufficiency of "(i) or (iii); or (ii)
  with (iv)" withdrawn; each construction with the channel it excludes, its further hypotheses,
  the channels left and its kind — (i) a statistical and a behavioural hypothesis, the
  conditional forecast's invariance apart from the actual report's; (ii) the values of the
  reports forming a baseline, with `16` §4.6's composition, frozen weights and rule fixed, value,
  reveal, availability and consolidation apart, T58 not introduced; (iii) the opening's timing
  only, content, presence, recipients and metadata left; (iv) those answers only, selection and
  inclusion, procedure and parameters, the source check, attempts and resources left — with (ii)
  and (iv) as per-pair predicates on recorded positions, no schedule and no common freeze
  adopted; §12.1's primitives (a hash, commitment or root authenticates no producer and checks
  integrity only against a reference of established provenance; a signature shows neither truth
  nor correctness; replication and availability rules constructions, not universal
  requirements); the matrix with the code, what the requirement asks and candidate policies
  apart (a verifier after the group's condition, no intermediate before the terminal record:
  options, not consequences of D1); `Node::submit`'s writes to the local store and log, the
  replication later and distinct, also in §11.4; the shortfall's positive descriptive record apart
  from the pilot's transition, the refused step still refused and never by itself `I`; an entry
  implying `Y = A` a disclosure under the record's condition, an absence of entry identifying
  neither `R` nor `I`; §12.5.
- **Sub-finding `08` PRIV-004.1, separate from A1** (documentation only, in the same commit):
  what an inclusion proof lets its holder infer of other inputs. Observed by Claude Code in
  `8af257d`; confirmed by Astra by reading the code and a synthetic Python reproduction;
  re-checked by Claude Code with a synthetic script (below). With two answer leaves, the proof's
  one sibling is the other leaf's hash, and knowing that leaf's respondent id, batch and index and
  that the answer is binary, exactly one candidate matches. Limits: it needs known or enumerable
  fields; not every proof exposes every leaf directly; it shows no recovery of every input from
  any root; no break of integrity or collision resistance; NET-003 not reopened. `04` §Events and
  replay and `08` PRIV-004's "seeing nobody else's" qualified: no other input in the clear, no
  general guarantee against inference. Open: no correction chosen, no salt prescribed as
  sufficient; hashes, leaf format, serialization and tests unchanged. Not the reveals'
  publication (T77), not an A1 finding.
- **Astra's review of `7d8db08`** (the commit, its parent and the published HEAD checked; the
  diff and the documents read; a small calculation with fractions on the conditional target; no
  Rust, fit, campaign, Merkle reproduction or local documentation check; the working tree and the
  local checks stay Claude Code's evidence): the substantive rectifications of `17` §12
  **approved as a conditional analysis of the informational requirements, with no new blocking
  finding on its setting** — the channels (i)–(iv) exclude, their further hypotheses and residual
  channels; cryptographic properties, requirements and candidate policies apart; the local write
  apart from replication; the shortfall's descriptive record apart from the refused transition
  and from `I`; an entry implying `A` apart from an absence of entry; `08` PRIV-004.1's
  counterexample, limits and provenance; the rectified promises of `04` and `08`. **Not
  approved**: the adoption of T1 or T2, a protocol, an implementation, a cryptographic solution,
  thresholds, pool entry or S2's start. Its two precisions, made by Claude Code: §12.4's (i)
  states the invariance of the whole outcome's law on `{A, R, I}` — `P(A)` alone does not fix
  `q_c` (two laws with `P(A) = 1/4`: `c = 1`, `q_c = 1/4` against `c = ½`, `q_c = ½`) — a
  sufficient hypothesis, not shown necessary, the behavioural one kept apart; `08` PRIV-004.1
  reads "general non-inference claim refuted under the stated enumerable-field assumptions;
  remediation open", in the sub-finding and §15's matrix, with a reference among §18's residual
  risks; every limit kept. R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- **Observable evidence under T1: `17` §13, `f38f8a4`** (documentation only; not covered by
  §12's approval; T1 the analytic reference, T2
  the comparison, T1 not adopted and its timing channel open). Three orders apart: local (one
  hash-chained log, relative to a trusted head), application (the signed cuts), effective
  disclosure (replicated entries readable before any cut, nothing recorded off the log); the
  existence order across feeds shown only through a cut signature on the later entry's feed,
  otherwise indeterminate. Each pair classified as verified, contrary order documented or
  indeterminate, in a stated order kind; a contrary order no violation of H-c, a verified one no
  proof against disclosures off the log. (ii): `k`'s first-panel `Commit` and `CloseCommits`
  against every entry bearing on `O`, with the approved composition, frozen weights and weighted
  mean, values, reveals and consolidation apart. (iv): a reference binding `G'`'s answers before
  those entries, limited to the answers it binds. Pairs not limited to reviewer overlaps (shared
  parameters, §10.4); no dependence-finding algorithm. Its reading of application order as
  information, its outcomes and its statements on (iv) are rectified below.
- **Astra's review of `f38f8a4`: partial** (the commit, its parent and the published HEAD
  checked; the diff, the documentary links and the relevant code of `network::{cut, log,
  replica}`, `protocol::{ledger, lifecycle}` and `p2p::member` read; no Rust, fit, campaign, new
  Merkle reproduction or local documentation check; the working tree and the local checks stay
  Claude Code's evidence). **Approved**: the record of `7d8db08`'s review; §12.4 (i)'s precision,
  the whole outcome's law apart from the behavioural hypothesis; `08` PRIV-004.1's state and its
  reference among the residual risks, so its record is approved within its stated limits; in
  §13, the local order, the application order and disclosure apart, the cryptographic references
  showing some precedences across feeds, pairs not limited to reviewer overlaps. **Not
  approved**: §13 as a whole; a protocol, an implementation or S2's start. T1 stays the analytic
  reference and T2 the alternative, neither adopted. R1 stays approved; A1, A2, B1–B3 and Phase 1
  stay open.
- **Rectifications of `f38f8a4`, `176dd8c`** (documentation only), in `17` §13 corrected in
  place: an abstract construction, checked against the cut
  rules (`Cut::next` marks feeds as the proposer holds them; `should_sign` needs no cut to count
  every replicated entry), where an entry carrying `O` is readable before the commitments that an
  earlier cut counts, so the replay puts them first while the information came first; three
  levels apart — the replay check (R), documented existence precedence (E), and the exclusion of
  early information (I), which also needs the coverage of disclosures and the declared
  hypotheses; `Replica` (accepts by signature) apart from `Ledger::apply` (examines, applies or
  refuses), the relevant commitments as accepted `Commit` steps of the panel in `InReview` and an
  accepted `CloseCommits`, a refused entry named `Commit` fixing no report; disclosure entries
  applied, refused or never counted; four outcomes with the prefix and evidence each needs —
  verified, contrary order documented (both events present), precondition absent in a complete
  prefix (an absence, not a later event), indeterminate; a results root logged before the
  disclosure can bind answers from its own existence, without dating their collection, the
  absolute "every (iv) pair indeterminate" withdrawn; the minimal result a replay check, not a
  verification of (ii)'s or (iv)'s informational predicate.
- **Astra's review of `176dd8c`** (the commit, its parent and the published HEAD checked; the
  diff of the four documents read; the relevant passages of replication, cuts and the ledger
  checked again; no Rust, fit, campaign or Merkle reproduction, and no local check of the working
  tree or of the documentation checks): the rectifications and `17` §13 **approved as a
  conditional analysis of the observable evidence**, within its stated limits and with three
  precisions. Resolved, the substantive findings of the review of `f38f8a4`: replay kept apart
  from the availability of information, in a construction compatible with the rules read of
  `Cut::next`, `added` and `should_sign`; relevant commitments apart from mere entries named
  `Commit`; disclosures that can include refused or uncounted entries; an absence in the prefix
  apart from a contrary order and from incomplete evidence; a results root able to bind answers
  before a later disclosure without dating their collection; the minimal result limited to the
  replay check. **Not approved**: the adoption of T1 or T2, a protocol, an implementation or S2's
  start. Its three precisions, made by Claude Code in `17` §13: (R) and (E) distinct checks, (E)
  needing the relevant commitments' existence and identification but not every disclosure entry
  examined by the replay (§13.2, aligned with §13.3; the outcome table's absent-precondition and
  indeterminate rows now state each level's evidence); (E)'s conclusions limited to the entries
  of `x_O` its evidence covers, every disclosure only under (I)'s coverage (§13.4, §13.5);
  `Replica::check` accepting subject to its authenticity and integrity checks — writer,
  signature, object size, CID — with no check of protocol validity, entries later refused by the
  ledger staying readable (§13.1, §13.4). R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.
- **S2's minimal perimeter: `19` §10.6** (documentation only, not pushed by the session; **a new
  proposal awaiting review**, not covered by §13's approval; no implementation authorized). On
  `17` §§9–13 without restating them: a question (does an executable path under A produce, at a
  declared `Ω`, the records `17` §§9.5 and 13.5 need, at which times and resources?); what only
  participants observe, against what a synthetic-event harness only verifies (a technical check,
  not a field pilot); no conclusion on properness, the reputation's incentives, H-c, the true DTF
  or neutrality, and none on A2. Recommended for Astra: study scores kept out of any reputation,
  gate decisions recorded and none applied to the pool, with the questions this leaves aside. The
  capabilities strictly needed (what exists, what is missing, what a harness can supply, the
  evidence that it works), among them enrollment bound to an identity (T20, open), without which
  a study's credentials leave uniqueness per person declared, not shown; the path issues no
  inclusion proof, so it does not meet `08` PRIV-004.1 (its scope stated below). Five blocking
  decisions (scores and pool, the forecast's meaning, each group's procedure, a shortfall or a
  refused batch, pseudonyms before T20), the deferrable ones apart; resources from documented
  floors (at least 3,300 participations for an attempt reaching stage 2, qualified below); the
  cost data to acquire before a pilot. Next deliverable recommended: a documentary specification
  of the synthetic-event technical check.
- **Astra's review of `cbcbc67`** (the commit, its parent and the published HEAD checked; the diff
  and the relevant passages of the pilot, revalidation, enrollment, latent, harness and contracts
  read; no Rust, fit, benchmark or campaign run; the local working tree not checked).
  **Approved**: the record of `176dd8c`'s review; `17` §13's three precisions; the further changes
  to `17` §13.2's classification at (R) and (E) — an absence concerns acceptance in the declared
  prefix, without showing that the commitment or reference did not exist elsewhere. **`19` §10.6:
  partial** — the direction accepted, the section to rectify. **Decided, for preparing the
  specification only**: B-b's study scores outside the reputation; the gate's decisions recorded,
  with no effect on operational pools; α the semantic reference to specify; the synthetic-event
  technical check the next deliverable, after the rectification. **Not authorized**: an
  implementation, recruitment, S2's start, the adoption of T1 or T2, new protocol policies. The
  length beyond an indicative 800 words is no blocking finding. R1 stays approved; A1, A2,
  B1–B3 and Phase 1 stay open.
- **Rectifications of `cbcbc67`, in `19` §10.6 corrected in place** (documentation only, not
  pushed by the session; **awaiting review**, no approval recorded): three kinds of evidence —
  events and outcomes as fixtures (records, states, classifications and the accounting of costs
  the fixtures supply, no cost of a fit not run), synthetic responses through the real fitters (a
  distinct statistical execution, its own costs and authorization, outside the next perimeter),
  human participants (behaviour, availability, delays, load); the check's promise of fit and
  search times removed, a replay or record-handling time named for what it measures. Anonymity
  and D17 as requirements of the path, not as holding: the access decisions left for reports,
  answers and results, the publication T77 contests not to be extended, no cryptography designed;
  synthetic data keep real data out of the check without validating a human study's
  confidentiality; issuing no inclusion proof keeps `08` PRIV-004.1's vector out of the path, no
  general guarantee, its remediation open and apart from A1. α the specification's reference, the
  harmlessness claim withdrawn: its meaning can move reports, gate decisions, extra rounds,
  appeals and times; `τ`, `ε` and the gap provisional, no value change approved. `N1_MIN = 300` and
  `N_LATENT_MIN = 3,000` kept; 3,300 the sum for two distinct administrations, one passing each
  gate, not the least cost of every attempt reaching stage 2 (a batch can be refused before the
  fit, for `K_MIN` among others); participations, gate-counted pseudonyms, persons, answers,
  searches actually run and attempts refused before the fit apart; no saving from reuse, no power
  or feasibility from the floors; `pilot::screen` the size gate, `stage1_screen` its computation;
  9, 17 or 25 runs per latent search actually run. Decisions for the synthetic-event
  specification (the fixture's procedure with a declared synthetic budget and term, neither a
  parameter of the human study or of the protocol; a shortfall and a refused batch distinct
  records, neither turned into `I` automatically; synthetic pseudonyms, T20, recruitment and
  availability no prerequisite, no enrollment chosen for the field) apart from those before a
  human study (planning hypotheses and resource limits before authorization, the rates possibly
  its results, a preliminary collection a separate activity). Nothing asked of the owner.
- **Astra's review of `5af22b9`** (the commit, its parent and the published HEAD checked; the diff
  of the four documents read; the joins checked; `bridging_gate`, `supplementary_review`,
  `screen` and `stage1_screen` read again; no Rust, fit, benchmark or campaign run; the local
  checks and the working tree stay Claude Code's evidence): the rectifications and `19` §10.6
  **approved as a preparatory synthesis of the experimental perimeter, with no blocking
  finding**. Resolved: synthetic events, synthetic responses through the fitters and human
  participants apart; anonymity and D17 as requirements; α's meaning and the provisional
  thresholds' limits; the 3,300 participations qualified; the check's prerequisites apart from
  the human study's. Approved: the gate check as `bridging_gate`'s decision on the fixture's
  inputs, with no claim on how those inputs are produced. **Confirmed for the next
  specification**: B-b's study scores outside the reputation; the gate recorded with no effect on
  operational pools; α the semantic reference; fixtures with explicit synthetic budgets and terms,
  small values exercising the cases, never parameters of the protocol or of a future human study.
  **Not authorized**: an implementation, a run, the adoption of T1 or T2, data collection or S2's
  start. Its two editorial joins, made by Claude Code: `19` §10.4 names `pilot::screen` as
  `stage1_screen`'s gated entry; `17` §13.2's absent-precondition row stands on its own (an
  absence of acceptance in the declared prefix, no proof that the commitment or reference does
  not exist elsewhere), the classification unchanged.
- **S2's synthetic-event technical check: `20`** (written on `5af22b9`, committed in `c76c035`;
  **approved by Astra on `c76c035` as the check's specification**, below; not covered by the
  approval above). The boundary of a future execution per component — what real code would run
  (feeds, cuts, `Ledger::apply`, the lifecycle, deposits with test credentials, `bridging_gate`, the
  first gates of `pilot::screen` and `latent_batch`, `panel_scores`), what the fixtures supply,
  the check code missing and the APIs left out (fitters, `supplementary_review`, pilot steps,
  `SkillTrack`, results events, inclusion proofs). Eight conventions, synthetic and limited to the
  check (`Ω`, cohorts, frozen weights, an attempt rule with budget 2 and terms in cuts, the reveal
  close, records, T1 and T2 groups, numbers). A main scenario of 14 items in 7 groups on one feed
  with fixtures F1–F9 (conclusive outcomes, a verifiable `I`, an indeterminate attempt, a shortfall
  and a refused batch apart from `I`, a missing report beside a held `I` under T2, unreached
  freezes, a late record, three cohorts, the gate); exact B-b values (cohort `K_v` = 443/2646;
  `K_w` −17/98 once consolidated; `K_u` without a value); order fixtures O1–O7 for `17` §13.2's
  outcomes at (R) and (E), on one feed and across feeds; the snapshot's content and the declared
  costs. Five conventions flagged as open for a field study, none blocking the check.
- **Astra's review of `c76c035`** (the commit, its parent and the published branch checked; the
  specification, its documentary joins and the relevant code read; baselines, contributions,
  cohorts, counts and declared costs recalculated independently with exact fractions; the order
  fixtures checked by reasoning, not run; no Rust, fit or synthetic check run by Astra; the
  working tree and the local checks stay Claude Code's evidence): `20` **approved as the
  specification of the synthetic technical check, with four precisions; no blocking error in the
  expected results verified**. The precisions: (A) K2's cohorts finite sets of assignments, fixed
  apart from the outcomes and complete for the check, no later assignment enlarging them, with no
  new epoch-closing rule; (B) the fixture's data, held records included, the log prefix's data and
  the results derived from each kept apart — the fixture's knowledge raising no `O` or `V`, giving
  no contribution and consolidating no cohort, the approved totals kept with their provenance;
  (C) T1's start a synthetic event explicitly supplied, its order checked, no real execution nor
  H-c inferred from a record's position; (D) K4 keeping both of G6's items, j12's second reading
  supplied and checked unused. The review approves no implementation, none existing then.
- **The precisions and the implementation, `e5e0898`** (on the owner's assignment, which
  authorized the circumscribed implementation, its targeted tests and the commit; not on that
  review; **partly reviewed by Astra on `e5e0898`**, below). The precisions made in place in
  `20` (§§2–5, 7: the three sources, K2's sets in `S0`, K6–K7's start records, G6's second
  reading, the costs split between log and held). The check in test code only:
  `crates/protocol/tests/s2_synthetic_check.rs` (23 tests: F1–F9, O1–O7, K5, the 102
  assignments by case, the common expectations, T1's starts, the costs) and
  `crates/protocol/tests/s2/` (fixture builder, study records and reader, log view, derivations,
  order classifier). The real APIs of `20` §2's "Executed" column run;
  fixtures supply the rest; nothing of the "Excluded" column is called. No production code, API,
  event variant, dependency, threshold or golden output changes. Choices within the specification
  and one limit (the member's signature inside a `CutSignature` not re-checked, `signed_by` being
  crate-private) are recorded in `20` §9.
- **Astra's review of `e5e0898`: partial** (the commit, its parent and the published branch
  checked; the seven new Rust files, the relevant code and the documentary joins read; minimal
  Python reproductions of the branches concerned, no Rust run; the 23 passing tests and Clippy
  stay Claude Code's evidence). Accepted: the record of the review of `c76c035`; the four
  precisions made; prefixed cohorts and the report of a later assignment; log, held records and
  declared costs apart; T1's starts explicitly synthetic; j12's second reading kept and unused;
  the check isolated in test code; expected values independent of the code checked. Not
  approved: the implementation as a whole, for two defects. (1) `k4` counted two registrations
  of one attempt as two executions, and sorted a stage's attempts by number rather than reading
  the order recorded: `validate` could accept an `I` from one surviving stage-1 attempt and one
  stage-2 non-conclusion logged twice (the terminal can only cite that one CID, which the lookup
  by stage and number found twice), or hide a conclusion recorded first but numbered after two
  non-conclusions. (2) `replay_level` returned `Indeterminate` for a pair once an entry of `x_O`
  went uncounted, though another entry's contrary order stayed documented. Precision: the
  unchecked member's signature inside a `CutSignature` is not by itself blocking for (E); the
  contract states which authenticated reference is used, what is verified, what is not, and what
  it concludes. The specification stays approved on `c76c035`; R1 approved; A1, A2, B1–B3 and
  Phase 1 open.
- **Rectifications of `e5e0898`, `3559396`** (test code and docs only, on the owner's assignment,
  which authorized the circumscribed corrections, targeted regressions, checks and the commit;
  **approved by Astra on `3559396`**, below). Five regressions written first, failing on `e5e0898`'s
  check code (`20` §9). `study::sequence` checks K6 and K4 on a group's attempt records in the
  order recorded, never reordering or repairing: repeated, conflicting or misnumbered records
  refuse the derivation (`TerminalError::Sequence`, `Pending::Incoherent`, the error in the
  group's report); `k4` reads that order; `validate` matches each attempt used by its own CID;
  `executed` and the costs count each distinct attempt once. `order` classifies each entry of
  `x_O` alone and aggregates as `17` §13.2 says, a contrary order or an absent precondition
  standing with its witness whatever the others, `Verified` needing every entry, the entries
  without evidence listed apart; an incomplete prefix still leaves every entry indeterminate.
  The `CutSignature` contract is stated in `order::References::new` and `20` §9. 28 tests pass;
  every approved expected value is unchanged.
- **Astra's review of `3559396`: approved** (the commit, its parent and the published branch
  checked; the diff, the regressions and the documentary joins read; small Python
  transcriptions checking the attempt sequences and the aggregation of evidence; no Rust run;
  the 28 passing tests, Clippy and the local checks stay Claude Code's evidence, the failed
  assertions of the run before the rectifications kept apart from those it did not reach).
  Approved: the rectifications of both defects of `e5e0898`, the two findings resolved; the
  implementation of the synthetic check within `20`'s perimeter; the original fixtures' results
  kept; the `CutSignature` precision, its authenticated reference apart from a check of its inner
  signature. Scope: the specification on `c76c035`, the implementation and rectifications on
  `3559396`, within the synthetic check's perimeter; no general validation of arbitrary flows; no
  proof of H-c, properness, reputational incentives, confidentiality, the true DTF or neutrality;
  S2 not started. Two non-blocking precisions, made in `20` §9 (documentation only, after that
  review): a terminal validated on its own prefix stays valid when an incoherent registration
  comes later, the group's report still naming the incoherence (read in `study.rs`, behaviour
  unchanged); duplicated shortfall and refused-batch identities are not checked, a limit of the
  perimeter. R1 approved; A1, A2, B1–B3 and Phase 1 open.
- **Astra's review of `bef891b`: partial** (the commit, its parent and the published branch
  checked; the diff read and compared with the criteria of `15`, `10` and `08` §16.1; no Rust,
  fit or campaign run). Approved: the record of `3559396`'s approval; `20` §9's two precisions.
  Not approved as a whole: §5.1's closure synthesis; rectifications asked on T82 and B1–B3, the
  later tasks' dependencies, A4's and A5's residues, the meaning of closure and T26. Direction:
  A1 the next block — B-b the reference score, A the analytic reference, C in the comparison;
  no adoption, expenditure or sample size. R1 approved; A1, A2, B1–B3 and Phase 1 open.
- **§5.1 rectified in place and A1's candidate contract proposed in `17` §14** (documentation
  only, on the owner's assignment; **awaiting Astra's review**, neither covered by the review of
  `bef891b`).
- Owner's modifications, never to restore, commit or clean: `.gitignore` (it ignores
  `/characterization*/`, `/smoke*/` and `.gpt/`, where Astra's patch sits) and the ignored
  directories. No commit of this work touches `.gitignore`.
- R1 closes neither A2 nor Phase 1.

## 2. Roles and working agreement

- **Claude Code** implements, verifies (tests, calculations, runs) and commits when the owner
  asks; it does not push (the session has no GitHub credentials). **The owner** orchestrates the
  sessions and publishes (pushes); wants to intervene as little as possible and decides only
  genuine product trade-offs. **Astra** decides the design and reviews each commit, reading diffs
  and recalculating; it does not re-run Rust.
- A finding closes only on Astra's recorded review (`15`), never on green tests. One finding per
  intervention; a code defect starts with a test that fails on the baseline.
- Evidence labels: L read, D proved, C calculated, E executed; say whether a fit is real, whether a
  composition is the API or a runtime (no epoch driver exists), and never present a counterexample
  as a frequency.
- Repository rules (`docs/CLAUDE.md`): comment budget enforced by `scripts/comment_budget.py`
  (hook after every edit); docs updated with the code; English only in the repository. Build
  scratch worktrees with a separate `CARGO_TARGET_DIR` (a shared target left stale artifacts).

## 3. Closed findings (`15`, each within its record's scope)

| Finding | Commit | Scope and limits |
|---|---|---|
| A5 | `afc84d0` | an extra reviewer without a row enters the re-decision on its standing |
| A4 | `fb8a3d4` | an unconverged latent re-check is indeterminate; residues (b)–(d) open |
| A6 | `f5ce98b7` | an alarm ends a founder's seed weight |
| A7 | `cabdae4` | participation = axis and positive weight, everywhere in bridging |
| A11 | `dd842d6` | an unconverged stage-1 screen is indeterminate: no rejection, no observed outcome |
| A4 (a), A11 (f) | `af17eb3` | stage 2 carries `Recheck`; an appeal settles only on the pilot's conclusive outcomes (a caller settling later must keep that outcome) |
| A3 | `16e862c` | the histogram's moment penalty is an `n`-scaled regularizer, the BIC counts `Q − 1` nominally; identifiability, weak information, the penalty's choice and effects and the BIC on penalized mixtures moved to B1–B3 |

## 4. Open work

- **A2 (high): R1 implemented and approved by Astra on `d13bf09`; A2 open.** Approved on
  `0519626`: the contested facts' cost does not certify the whole test's DTF; the conditional
  proposition of `19` §3 (partition of the items, common measure, representation of the groups;
  per-pair maximum for mixtures constant in ability, pointwise-maximum envelope in general; no
  label matching needed). Approved on `d13bf09`: R1 (`8fa07dd`, completed by `d13bf09`) as a
  rectification of the declared guarantees, not as a realization of the whole-test DTF guarantee.
  Not approved by either review: R3 (a test-level fit) as a solution, a mandatory batch per form,
  any new calibration, selection or group policy. A reusable common calibration and a calibration
  per form are compared in `19` §9 under explicit assumptions, approved by Astra on `63b13fa` as
  a conditional comparison; §9.3–§9.5, deepened in `6681b03` and rectified in `b59fd03`, approved
  on `b59fd03` as a conditional analysis; §9.6 (one fixed form) approved on `00f8e2c` as a
  conditional analysis; §10, toward an experimental protocol, approved on `26593e7` as a
  preparatory synthesis; `17` §9 (cases 5–7 for a study) approved on `b2a1c2f` as the study's
  reporting specification; `17` §10 (the bridging input) approved on `111e595` as a conditional
  analysis; `17` §11 (timing under A) approved on `cf5575a` as a conditional comparison; `17`
  §12 (access and disclosure per object) approved on `7d8db08` as a conditional analysis; `17`
  §13 (observable evidence under T1) approved on `176dd8c` as a conditional analysis; `19` §10.6
  (S2's minimal perimeter) approved on `5af22b9` as a preparatory synthesis; `20` (its
  synthetic-event check) approved on `c76c035` as the check's specification, its implementation
  in test code approved on `3559396` within the check's perimeter. No calibration
  is chosen, S2 is not claimed to settle A2, and these approvals do not close A2.
- **A1 (critical for the incentive claim): open.** Acquired: the diagnosis (the exploration draw is
  known before reports); the beacon-manipulability and count analysis (`16` §§6–7); the band
  baselines, implemented (`protocol::panel_scores`, `e8fdbe7`, no production caller yet); the
  properness proof as a result conditional on C6 and its hypotheses (`16` §4.3); `17`'s argument and
  comparison of batch designs as a conditional analysis (`37addca`); `17` §7 (`825cee3`) and B-b's
  candidate contract, `17` §8 (`fa11791`), as conditional analyses. Open decisions: the batch
  contract (`17`: universal pilot, group audit, prefixed groups activated by entry or draw; no
  batching policy approved); missing outcomes (C2); the denominator and no-show rule (C4, T58); the
  randomness guarantee (a behavioral model of the beacon's members, or a source change); deferred
  draw against audit; whether A1 needs the incentives of the reputation actually used (`k_u`,
  shrinkage, cap, CUSUM). For a study's reporting, `17` §9's 6c and 7a are decided (`d31e9fa`;
  §9 approved on `b2a1c2f`); the join with the bridging is analysed (`17` §10, approved on
  `111e595`, α the reference, β the comparison); timing under A is analysed (`17` §11, approved
  on `cf5575a` as a conditional comparison, T1 and T2 alternatives, neither adopted); access and
  disclosure per object are specified (`17` §12, approved on `7d8db08` as a conditional
  analysis, T1 the reference and T2 the comparison by Astra's direction); the evidence for its
  predicates (ii) and (iv) under T1 is specified (`17` §13, approved on `176dd8c` as a
  conditional analysis); S2's minimal perimeter is approved as a preparatory synthesis (`19`
  §10.6, `5af22b9`; for its specification only: study scores outside the reputation, gate
  decisions recorded with no pool effect, α the semantic reference) and its synthetic-event check
  specified (`20`, approved on `c76c035`; its implementation approved on `3559396` within the
  check's perimeter); T58 stays open
  for a procedure that must proceed despite missing reports. The contract of A1's candidate
  guarantee is proposed in `17` §14, awaiting review.
- **`08` PRIV-004.1 (separate from A1): remediation open.** The general claim that an inclusion
  proof lets its holder infer no other input is refuted under the stated enumerable-field
  assumptions; the confidentiality goal and a construction meeting it stay to be stated; no
  correction chosen (`08` §18). Its record approved by Astra on `f38f8a4` within its limits.
- **B1–B3:** identification of the floors and of the histogram through the responses, weak
  information, the moment penalty's choice and effects, BIC under misspecification and on
  penalized mixture fits, selection after penalized fitting.
- **Costs and calibration:** sample floors are not power guarantees (B4–B5, C3); respondent burden
  and runtime need budgets (D2, D4); the contested selection's enumeration cost (D3); T83's
  operating point (B7, D1, D6, E). Also open: A8, A9, A10, the register's C5 and C6, A4 (b)–(d),
  C2, C4, the full runtime.

## 5. Decision synthesis and the calibration comparison

The synthesis is in `19` §6.1, approved by Astra on `cebcb2e` as a documentation intervention.
The comparison of a reusable common calibration with a calibration per form is in `19` §9,
rectified in `63b13fa` and approved there by Astra as a conditional comparison. Its deepening
(`6681b03`: §9.3's check conditions and cost hypotheses, §9.5's error control over a search) was
partly approved by Astra, rectified in `b59fd03` and approved there as a conditional analysis.
§9.6 analyses, on Astra's direction, whether data can bound the true DTF of one fixed form; it
was partly approved on `907c245`, rectified and approved on `00f8e2c` as a conditional
analysis. The shared aim is a circumscribed, coherent, executable and evaluable experimental
protocol; that direction approves no protocol and closes no finding. `19` §10, approved on
`26593e7` as a preparatory synthesis, sorts the decisions, measurable hypotheses and excluded
promises toward it; `17` §9, approved on `b2a1c2f`, is the study's reporting specification for
cases 5–7 under design A, with 6c and 7a as its basis. `17` §10, approved on `111e595`, analyses
the bridging input's meaning when the report forecasts `q_c`, α the main reference and β the
comparison. `17` §11, approved on `cf5575a` as a conditional comparison, analyses timing under
A, T1 and T2 staying alternatives; `17` §12, approved on `7d8db08` as a conditional analysis,
specifies what each needs to handle information per object, T1 the reference and T2 the
comparison by Astra's direction; `17` §13, approved on `176dd8c` as a conditional analysis,
specifies the evidence a study under T1 would record for §12.4's (ii) and (iv) pair by pair, its
minimal result a replay check. `19` §10.6, S2's minimal perimeter, was partly approved by Astra
on `cbcbc67`, rectified and approved on `5af22b9` as a preparatory synthesis. The synthetic-event
technical check is specified in `20`, approved by Astra on `c76c035` with four precisions, and
implemented in test code on the owner's assignment (`e5e0898`), partly reviewed there,
rectified and approved on `3559396` within its perimeter. S2 is not declared ready; no further
run, recruitment, data collection or start is authorized. The next step is Astra's review of
§5.1's rectifications and of A1's candidate contract (`17` §14).
No roadmap or campaign is added, and no choice or parameter is asked of the owner.
§9 finds no winner today: with complete administrations the common design is excluded where the
bank and anchors exceed the tolerable load per participation, and saves answers only under its
break-even inequalities; the minimum information to cross the boundary is the tolerable load,
whether one population serves every form, the construction under §9.5's reference and its
obligations of proof, the horizon's numbers and, only if those leave it open, the ratio of the
samples at equal precision. `α_H` stays symbolic; no level is put to the owner.
The synthesis's scope, as set at the owner's request: a short decision synthesis that links:
(1) what outcome a reviewer forecasts and how it is produced — A1; (2) the population and
contrasts the DTF must protect — A2; (3) the dependence on the model and on selection — B1–B3;
(4) respondents, answers including anchors, fits and time. The comparison sets a reusable common
calibration against a calibration per form without presuming either valid or inevitable.
Constraints: anonymity (no personal or
group attributes enter), the recovery of contested facts; the test's neutrality is not to be
declared certified. No full new roadmap yet. Astra's candidate for linking A1, C2, A2 and the
costs is checked in `17` §7, accepted by Astra on `825cee3` as a conditional analysis; B-b, the
main candidate, is specified as a candidate contract in `17` §8, partly reviewed on `44f0dbd`,
rectified in `fa11791` and approved there as a conditional analysis; A is the main analytic
reference, C the comparison. The synthesis adopts and funds nothing; the decisions it lists as
missing (population and contrasts to protect, item formats and the anchors' substantive
reference, tolerable errors and inconclusiveness, latency and resources) are not put to the owner
before a sufficient comparison of the designs.

### 5.1 Phase 1 closure check (partly reviewed on `bef891b`; rectified, awaiting review)

Claude Code's proposal, committed in `bef891b`. **Astra's review of `bef891b`: partial** (the
commit, its parent and the published branch checked; the diff read and compared with the criteria of
`15`, `10` and `08` §16.1; no Rust, fit or campaign run). Approved: the record of `3559396`'s
approval; `20` §9's two precisions. Not approved as a whole: this synthesis. Rectified in place
below by this intervention, awaiting review: the A1, B1–B3 and A4–A5 rows; T82 and S1; the later
tasks' dependencies; T26 and SC-8; the next block. Neither the rectified text nor A1's candidate
contract (`17` §14) is covered by that review.

**Astra's precisions on the meaning of closure.** A narrowed guarantee is not a realized one.
`10` §1.5 allows a completion claim on an explicitly narrowed domain, not the automatic closure of
the original technical problem: R1 corrected a promise without resolving A2. Severity alone does
not decide whether a finding blocks; its effect on the guarantees kept does. Each exclusion needs
its own justification. No substantive narrowing of any guarantee is approved.

**Criteria used**, none added: each finding's criterion in `15`; `10` §1.5's completion
qualification, read with the precisions above (provisional parameters with defined pilots
allowed; field validation T27; no documentation-only change closes a code or proof finding);
`15`'s working agreement 6 (proportionate evidence for every critical claim, `08` §16.1; T26
reviews the final candidate). Missing items: **Dd** design decision, **Pr** proof, **Im**
implementation, **Em** empirical evidence; for residues, **B** shown blocking, **O** an open
decision, **L** a documented limit.

| Findings, state | Closure criterion (ref) | Acquired; limits | Missing | Depends on | Minimal next deliverable |
|---|---|---|---|---|---|
| A1, critical, open | `15` A1: the information at reporting defined, IPW properness proved under it, an adaptive strategy tested | theorem conditional on C1–C6 (`16` §4.3); band baselines in code; B-b, arms A/C, cases 5–7, α, T1/T2 and their evidence as conditional analyses (`17` §§7–13); `20`'s check. Limits: C1 rests on a member model (`16` §6); H-c only declared; no arm or contract adopted | Dd: the guarantee's perimeter — target, arm, information, missing reports — each narrowing justified (`17` §14.7); Im: `17` §14.5's missing capabilities; Pr: each hypothesis realized, or declared with its residual channels; Em: `17` §14.6's adaptive test | A's capacity (+316.825 slots against the declared 333.5, `17` §8.5); the records' availability (`17` §8.3); T58 for C4 and cases 6–7 | the review of `17` §14's contract |
| A2, high, open; R1 approved | `15` A2: target measure, linking, whole-test contribution specified; bound proved; estimation error apart; a margin alone does not close it | conditional proposition (`19` §3); R1, `D(T)` an admission cost; calibration comparison, no winner (`19` §9); one fixed form (`19` §9.6). Limits: no `μ`, `𝒢`; the filtered functional is no function of the law; no bound with coverage | Dd: `μ`, `𝒢`, contrasts, forms (`19` §6.1, §10.2); Pr: the bound and its coverage; Im | B1–B3; the calibration arm, common reusable or per form, neither valid nor inevitable; D2, D4 | a contract of A1's kind, after A1's |
| B1–B3, open | `15`: decision-level sensitivity, search stability, simpler alternatives, misspecified populations | A3 closed on its nominal count; `19` §9.6's identification limits; the pre-D43 supplement, historical (`13` §8.7) | Em: S1's scenarios and decision criteria declared, then one campaign on an identified candidate, its design declared before execution; Pr where identification or coverage is claimed, which simulations do not replace | A2's target; the owner's authorization of the campaign | S1's declaration, no run |
| A4 (b)–(d); A5's residues | the closures stand within their records; the residues are their "Left open" (`15`) | indeterminate readings typed on the decision paths (A4, A11, A4 (a)) | A4 (b), `emerging_dif` an untyped boolean whose `false` means no retirement: L, its typing O; (c), the retired proxy path, fixtures only: L, its listing requiring no change; (d), a one-class fit reading `Evaluated` while mixture candidates failed: O, within B1–B3's search. A5: whether the extra draw weighs reviewers, O; an explicit `w_max` contract and a binding-cap test, non-blocking (Astra, `afc84d0`). None shown B | B1–B3 for (d) | none required; a decision where a kept guarantee is shown to rest on one |
| A8, A9, medium | `15`: distinguishability, identifiability and power apart (A8); hypotheses or a controlled approximation (A9) | the paper a dated snapshot | Pr and the paper's text | — | a rectification of the two claims |
| A10, low; C5, C6, medium | `15`: properness scoped or the fallback decided (A10); index or inferential model (C5); decay policy (C6) | behaviour documented | Dd, Im if changed | A1 for A10 | decisions, Astra's or the owner's |
| B4–B5, C3; B6, D5; B7, D1, D6, E; D2, D4; D3 | `15`'s statistical table; B7: provisional values allowed with T27 procedures | T24; T25 steps 1–3; `14` Level A | Em on a frozen candidate; D3: Dd on a sustainable domain | T83 (owner); T25 step 4, reading T82's measures | D3's domain; the rest after the candidate |

**Phase 1 outside the register** (`10` §1.5): T25 step 4 for Level B, which reads T82's DIF
studies on the D43 model and T83's operating point (owner). T82 is no prerequisite for writing
S1: its measures and B1–B3's can belong to one experimental design declared before execution, on
an identified candidate, with criteria fixed beforehand and the owner's authorization of the
campaign. Simulations replace no proof of identification or coverage where those guarantees are
claimed. No campaign is designed or started here.

**Kept apart.** *S2*, a human study: no recorded Phase 1 criterion requires it; it would need
respondents, a tolerable load, an executable path and `17` §12's access decisions, and would
measure rates and costs, not H-c. *Later tasks*: a dependence is assessed against each guarantee
kept in the closure perimeter — incentives, privacy, security, empirical validity — and no later
task moves into Phase 1 by default. Links on the record: T58 to C4 and cases 6–7 (`16` §4.4,
`17` §9.4); T79, with T76 which it needs (`10`), to the records' availability (`17` §8.3) and to
`17` §13.4's evidence; T19 to C1 under arm C and to the beacon's other roles (`16` §6); T77 to
D17 and PV-5 wherever a guarantee on reveals is kept (`17` §11.4); T27 to the provisional
parameters' field values (B7). T17, T75 and T23 stay in their perimeter unless a kept guarantee
is shown to rest on them. Costs stay counted apart — participations, persons, answers with
anchors, fits, times (`19` §10.4) — and D2, D4 unmeasured.

**T26 and SC-8.** `15`'s agreement 6 has T26 review the final candidate; `10`'s dependency notes
say nothing in Phases 1–2 needs the external gates. The contrast stays visible until resolved
explicitly, and until then T26 is not declared superfluous. Astra's internal review does not by
itself meet SC-8 (`08` §16.1), an external psychometric review by a qualified reviewer, recorded
in `reports/`. Neither requirement is rewritten here, and no external reviewer is contacted.

**For Astra.** The first version's ambiguities (1) and (2) are answered by the precisions above.
Left: the T26 contrast; which guarantees the closure perimeter keeps, against which the
dependencies above are assessed.

**Next block (Astra's choice): A1.** B-b the reference score, A the analytic reference, C kept in
the comparison; no adoption of A or C in production; no expenditure or sample size. The contract
of the candidate guarantee is proposed in `17` §14, awaiting review: it closes nothing, and its
narrowings of the declared promise (`17` §14.7) are a proposal, not a decision.

## 6. Essential reading to resume

`CLAUDE.md` and `docs/CLAUDE.md`; `docs/15` (rows A1, A2, B1–B3, D2–D4 and the correction records);
`docs/16` (A1's theorem, conditions C1–C6, beacon model) and `docs/17` (A1's batches and missing
outcomes; §14, the candidate contract); `docs/18` (A3); `docs/19` (A2); `docs/20` (S2's synthetic
check: specification and implementation approved); `docs/02` §B.3, §B.7; `docs/01` D33–D38, D43;
code: `crates/scoring/src/{latent.rs,dtf.rs}`,
`crates/protocol/src/{contested.rs,lifecycle.rs,orchestrator.rs,exploration.rs}`,
`crates/protocol/tests/{s2_synthetic_check.rs,s2/}`.

## 7. R1 and its approval

- **Approval, recorded by `81cd467`:** Astra approved `d13bf09` with no
  blocking finding. It covers R1 as implemented in `8fa07dd` and completed by `d13bf09`: a
  rectification of the declared guarantees, not a realization of the whole-test DTF guarantee.
  It is distinct from the approval of the diagnosis and conditional proposition on `0519626`.
  Recorded in `15` (A2 row, closing paragraph), `19` (status, design review, §8),
  `docs/README.md` and this note.
- **R1 (`8fa07dd`):** `D(T)` described as the contested facts' admission cost, not a certified
  bound on the test's DTF — `01` D38 (dated clarification), `08` DIF-011, its status row,
  AT-PRO-08 and the `Contested` row, `02` §B.7 (cross-fit paragraph, "cost"), `10` T55 (dated
  clarification), `ARCHITECTURE.md`, comments in `protocol/src/contested.rs` and
  `scoring/src/dtf.rs` (`DTF_MAX`), `paper/sections/065-revisions.tex` (the tolerance an aim, not
  guaranteed). No formula, API, threshold, serialization, selection, golden or historical result
  changed.
- **`19`:** Astra's review of `0519626`; disjoint sets for subadditivity; the ¾-against-1 example an
  abstract construction; R3 a candidate (estimated measure, confidence bound to be built, groups
  absent from the model, misspecification, coverage under selection); the owner trade-off withdrawn
  (common versus per-form calibration still to compare); the draw's law corrected (§1); the real fit
  converged, on a fixture under the admission gate; §8 lists R1.
- **Draw's law:** `ContestedPool::draw` picks uniformly among a fit's completable options, not among
  whole selections: one fact from fits `[a]` and `[b, c]` at zero cost gives 5/12, 7/24, 7/24
  (Astra's figures, C-verified by an exact enumeration added to `sim/dtf_composition.py`). No
  current contract promised a uniform law (`02` §B.7 and `10` T55 describe the local rule), so only
  the dossier was corrected; no separate discrepancy.
- **Test:** `scoring/tests/dtf_composition.rs::a_one_class_fit_reads_zero_where_two_items_lean` now
  asserts `Convergence::Converged`; it passes.
- **Registers:** `15` (A2 row, closing paragraph), `docs/README.md`, this note.
- **Documentation follow-up against `8fa07dd` (`d13bf09`; Astra's patch, integrated):** `01`
  D38's choice and rationale describe an aim, `06`'s countermeasure and limitation L1 describe the
  admission cost, `10` T55's acceptance cell no longer asserts whole-test DTF control, `08`'s
  DIF-011 open-work cell names A2, and `19` distinguishes the old paper claim from R1. Added by
  Claude Code in the same spirit: "cost" for "bound" in the rest of DIF-011's status row and in
  `10`'s list of open parameters. These are A2 claim residues, not a new finding. `05` already
  states the guarantee is open; the flow diagrams of `00` and `05` still say "balanced sets", the
  code's name for the selections within the cost, left as they are.

## 8. Evidence

- **Claude Code, original R1 intervention (reported in `8fa07dd`):**
  that test (passed: converged, one class, fitted DTF 0, true 0.78196);
  `python3 sim/dtf_composition.py` (draw law 5/12, 7/24, 7/24); `cargo test -p protocol
  --test a2_dtf_composition --test contested_facts` and clippy for the edited crates (comments
  only); `cargo fmt --all -- --check`; `python3 scripts/comment_budget.py`. Full suites not re-run
  (no behaviour change).
- **Claude Code, earlier:** see each record in `15`; A2's diagnostic tests and script on
  `0519626`.
- **Astra:** reads diffs, code, tests and docs, checks proofs, runs the Python scripts and exact
  enumerations; it has not re-run Rust or the real fits.
- **Astra, documentation follow-up on 2026-10-03** (its patch, prepared in a Codex environment
  without the owner's checkout; as it records): read the published diff and relevant code
  and contracts; re-ran `python3 sim/dtf_composition.py` successfully (including exact
  probabilities 5/12, 7/24, 7/24). The comment-budget check passed on the three Rust files and
  Python script changed by `8fa07dd`. No explicit global-uniformity promise was found in the
  checked current contracts (`01`–`14`, architecture and contested module); no separate issue
  was opened. The follow-up patch passed `git apply --check --whitespace=error-all` against
  the downloaded `8fa07dd` files. No Rust test, fit, clippy or cargo-format check was re-run:
  cargo is unavailable here. The converged-one-class result and true DTF 0.78196 remain
  Claude's reported execution evidence. No full suites, smoke, characterization, calibration,
  seed campaign or mutation runs. Astra's final review of R1 was then pending.
- **Claude Code, integration of the follow-up:** HEAD `8fa07dd` and a working tree with only
  `.gitignore` modified checked first; `git apply --check` and `git apply --whitespace=error-all`
  of the patch on the owner's checkout (clean); the handoff's environment lines adapted to this
  checkout; documentation checks: `python3 scripts/comment_budget.py`, relative links of the
  edited docs, a search of current contracts for remaining whole-test claims. No Rust, fit, script
  or campaign re-run (documentation only).
- **Astra, final review of R1 on `d13bf09`** (as it records): read the diff of `d13bf09` and the
  updated documents; approved R1 with no blocking finding. No new Rust or fit run.
- **Claude Code, record of the approval (`81cd467`):** HEAD `d13bf09` and
  a working tree with only `.gitignore` modified checked first; documentation checks only: the
  diff, whitespace (`git diff --check`), references and relative links of the edited docs, and
  the consistency of A2's and R1's state across `15`, `19`, `docs/README.md` and this note. No
  Rust test, fit, smoke, characterization, calibration or mutation run.
- **Astra, review of `fa11791`** (as it records): verified `fa11791` published and HEAD; read the
  diff and the documents; exact rational calculations on the counterexamples and the costs. No
  Rust or fit re-run. Its decision synthesis rests on readings and targeted static checks, with
  no new run.
- **Claude Code, record of that review and the synthesis of `19` §6.1:** branch, HEAD `fa11791`
  (parent `44f0dbd`) and a working tree with only `.gitignore` modified checked first; the
  synthesis's references checked statically against `17` §§4–8, `19` §§2–5, `15` (A1, A3, B1–B5,
  D2–D4), `16` §§5–7, `02` §B.3, `05` [7b], `01` D38 and the code symbols named
  (`LatentParams::n_starts`, the up-to-seven candidates and the complete matrices of
  `scoring::latent`, `PilotError`); documentation checks only: the diff, whitespace
  (`git diff --check`), references and relative links of the edited docs, the consistency of the
  states of A1, A2, R1 and `17` §8 across `15`, `17`, `19` and this note. No Rust test, fit,
  script, smoke, characterization, calibration or mutation run.
- **Astra, review of `cebcb2e`** (as it records): the diff read and compared with the documents it
  cites; the parametric formulas checked algebraically. No Rust, fit or campaign run.
- **Claude Code, record of that review and the comparison of `19` §9:** branch, HEAD `cebcb2e`
  (parent `fa11791`) and a working tree with only `.gitignore` modified checked first. Read (L):
  `latent_dif_with` and `LatentParams` (one start for the one-class model, `n_starts` per mixture
  candidate, the stop rule, the parameter count), `latent_batch` and its gates (`admit_dif_batch`,
  `admit_templates`, `admit_anchors`, rows of equal length), `ClassCurves::{of, dtf}`,
  `ContestedPool::{record, candidates}`, `blueprint::assemble_test`, `NullifierSet`. Exact
  calculations (C), in a scratch script not kept, formulas given in `19` §9: the subset counts
  `S(M, n)` and their superadditivity for `M₁, M₂ ≤ 30`, `n ≤ 15`; the break-even equivalences on
  20,000 random rational cases; the share of kept forms over the tolerance (1/11, 9/19); the
  marginal consistency of a three-item, two-class rational model under item deletion. Documentation
  checks: the diff, whitespace (`git diff --check`), references and relative links of the edited
  docs, the consistency of the states across `15`, `19` and this note. No Rust test, fit, smoke,
  characterization, calibration, benchmark or mutation run.
- **Claude Code, verification of `a9700fd`:** branch, HEAD `a9700fd` (parent `cebcb2e`), no later
  local commit and a working tree with only `.gitignore` modified checked first. The diff
  `cebcb2e..a9700fd` read against the sections it cites (`17` §§4.1, 7.6; `15` D3; `05` [8]–[9];
  `01` D43). Read (L): `latent_batch`, `admit_templates`, `admit_dif_batch`, `admit_anchors`,
  `NullifierSet` and `NullifierProof::id` (one respondent id per person, independent of the
  context), `latent_dif_with` and `LatentParams` (9, 17 or 25 optimizer runs per search),
  `ContestedPool::{record, candidates}` (the empty set evaluated per fit),
  `blueprint::assemble_test`, `ClassCurves::{of, dtf}`. Exact calculations (C), in a scratch
  script not kept: `S(10, 5) = 638`, `S(20, 5) = 21,700`, `S(40, 10) = 1,221,246,132`; the merged
  total below the separate totals only for `n ≤ 1` (`M₁, M₂ ≤ 30`, `n ≤ 15`); the kept-form
  shares 1/11 and 9/19; the confirmation cases on a small rational example and the new
  equivalence on 20,000 random rational cases. Documentation checks: the diff, whitespace
  (`git diff --check`), references and relative links of the edited docs, the states across
  `15`, `19` and this note. No Rust test, fit, smoke, characterization, calibration, benchmark or
  mutation run.
- **Astra, review of `63b13fa`** (as it records): the diff, the dossier and the relevant code
  passages read; exact rational calculations on the combinatorial counts, the confirmations and
  the selection. No Rust, fit or campaign run.
- **Claude Code, record of that review and the deepening of `19` §9:** branch, HEAD `63b13fa`
  (parent `a9700fd`), no later local commit and a working tree with only `.gitignore` modified
  checked first. Exact calculations (C), in a scratch script not kept: `1 − (19/20)^20`
  (≈ 0.641514); the union bound on an exact case with a hidden factor shared by every check and a
  budget spent adaptively; `E[V/max(R, 1)] ≤ P(V ≥ 1)` on 5,000 random rational laws; the expected
  cost's sum and its product form, with a probability of keeping a form below 1 under a cap, on a
  small exact case. Derivations (D) in `19` §9.5. Documentation checks: the diff, whitespace
  (`git diff --check`), line widths, references and relative links of the edited docs, the states
  across `15`, `19` and this note. No Rust test, fit, smoke, characterization, calibration,
  benchmark or mutation run.
- **Astra, review of `6681b03`** (as it records): the diff and the updated passages read; exact
  calculations on the counterexamples, the expected-share inequality and a capped search's cost.
  No Rust, fit or campaign run.
- **Claude Code, record of that review and the rectifications of `19` §9.5:** branch, HEAD
  `6681b03` (parent `63b13fa`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Exact calculations (C), in a scratch script not kept: 1/400 and 39/400
  for two checks with independent errors of 1/20; the inclusion and the union bound on 5,000
  random joint laws of two checks; the `Z` counterexample (1/20 for each prefixed procedure, 1 for
  `j = Z`); §3's construction (target ¾ with its classes as the groups, envelope 1). Documentation
  checks: the diff, whitespace (`git diff --check`), line widths, references and relative links
  of the edited docs, the states across `15`, `19` and this note. No Rust test, fit, smoke,
  characterization, calibration, benchmark or mutation run.
- **Astra, review of `b59fd03`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the relevant passages read; exact calculations with fractions (1/400 and
  39/400; the inclusion and the union over 1,771 joint laws; the `Z` counterexample; the
  contrasts ¾, ½, ¾ and the envelope 1; adaptive levels with a shared dependence). No Rust, fit or
  campaign run; Claude Code's documentation checks not repeated.
- **Claude Code, record of that review and `19` §9.6:** branch, HEAD `b59fd03` (parent `6681b03`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L), at `b59fd03`: `scoring::latent` (`Model`, `free_params`, `Grid::shape`, `penalty`,
  `moment_penalty`, `fit_from`, `latent_dif_with`, `LatentDif::flags`), `scoring::dtf`
  (`ClassCurves::{with_ability, at, of, dtf}`), `MIN_CLASS_SHARE`, the callers of
  `ClassCurves::of`; `18` §§2–5; `02` §B.3 and §B.7. Exact calculations (C), in a scratch script
  not kept: the total variation of an added class of share `ε` and of `n` respondents' laws
  (at most `nε`) on random rational laws; the identity of the two integrals summing to
  `Σ_j (1 − c_j)`; the population KR-20 of anchors independent of ability, 0. Documentation
  checks: the diff, whitespace (`git diff --check`), line widths, `scripts/comment_budget.py`,
  references and relative links of the edited docs, the states across `15`, `19` and this note.
  No Rust test, fit, smoke, characterization, calibration, benchmark or mutation run.
- **Astra, review of `907c245`** (as it records): the commit, its parent and the published HEAD
  checked; the diff, the relevant code and the theoretical references read; exact calculations on
  duplicated components, the DTF, the population KR-20 and the affine change. No Rust, fit or
  campaign run.
- **Claude Code, record of that review and the rectifications of `19` §9.6:** branch, HEAD
  `907c245` (parent `b59fd03`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Exact calculations (C), with fractions in a scratch script not kept:
  the standardization (`Var q = 140 = (2√35)²`; the code's grid with uniform logits, variance
  35/4, gives `u_q = q/(2√35)`); the anchors' mean ½; the population KR-20 ≈ 0.9983783235 from its
  formula and from its definition; `p_q > r_q` at every node; the filtered DTF ≈ 0.2439024373 at
  92%/8% and 0 at 92%/4%/4%, the unfiltered one equal in both; three complete 62-answer patterns
  equal in probability under both mixtures (an arithmetic check, not the proof, which is the
  mixture identity); `∫_{−1}^{1} |x − t| dx = 1 + t²` at three values; the affine change keeping
  `a(θ − b)`, a double shift not keeping it. Documentation checks: the diff, whitespace
  (`git diff --check`), line widths, `scripts/comment_budget.py`, references and relative links of
  the edited docs, the states across `15`, `19` and this note. No Rust test, fit, smoke,
  characterization, calibration, benchmark or mutation run.
- **Astra, review of `00f8e2c`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the updated text read; exact calculations with fractions on the
  standardization, the population KR-20, the filtered DTF and the rare-class consequence. No
  Rust, fit or campaign run; Claude Code's documentation checks not repeated.
- **Claude Code, record of that review and `19` §10:** branch, HEAD `00f8e2c` (parent `907c245`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L), at `00f8e2c`: `17` §8 and the open rows of `15`; `pilot::{submit_response, stage1_screen}`,
  `revalidation::latent_batch`, `orchestrator::{run_item, review_round}`, `NodeState::apply`,
  `lifecycle::State` and its `Indeterminate` transitions, `ResultRecord`'s variants,
  `SkillTrack`'s counters, the callers of `ContestedPool`, the characterization harness's studies
  and `DifDesign`. No calculation was needed for a new claim. Documentation checks: the diff,
  whitespace (`git diff --check`), line widths, `scripts/comment_budget.py`, references and
  relative links of the edited docs, the states across `15`, `19` and this note. No Rust test,
  fit, smoke, characterization, calibration, benchmark or mutation run.
- **Astra, review of `26593e7`** (as it records): the commit, its parent and the published HEAD
  checked; the diff, the documents and the relevant passages of `run_item`, `lifecycle`,
  `ResultRecord`, `SkillTrack` and `ClassCurves::dtf` read; no new calculation needed. No Rust,
  fit or campaign run; the local working tree as Claude Code reported it.
- **Claude Code, record of that review and `17` §9:** branch, HEAD `26593e7` (parent `00f8e2c`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L): `17` §§5, 7.2–7.5, 8; `16` §§4.2, 4.4, 4.6, 5; `10` T58; `lifecycle.rs`'s `Score` arms
  and `PartialEpoch`. No calculation was needed for a new claim. Documentation checks: the diff,
  whitespace (`git diff --check`), line widths, `scripts/comment_budget.py`, references and
  relative links of the edited docs, the states across `15`, `17`, `19` and this note. No Rust
  test, fit, smoke, characterization, calibration, benchmark or mutation run.
- **Astra, review of `d31e9fa`** (as it records): the commit, its parent and the published HEAD
  checked; the diff, the documents, T58 and the relevant lifecycle arms read; a small rational
  calculation on the baseline's dependence channel. No Rust, fit or campaign run.
- **Claude Code, record of that review and the rectifications of `17` §9:** branch, HEAD
  `d31e9fa` (parent `26593e7`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Exact calculation (C), with fractions in a scratch script not kept:
  Astra's channel, `b(p) = min(1, max(0, 2p − ½))` with a Bernoulli(½) outcome — expected
  contribution 0 at `p = ½`, 3/16 at `p = ¾`, the largest value on a grid of step 1/400.
  Documentation checks: the diff, whitespace (`git diff --check`), line widths,
  `scripts/comment_budget.py`, references and relative links of the edited docs, the states
  across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization,
  calibration, benchmark or mutation run.
- **Astra, review of `b2a1c2f`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the relevant documents read; the baseline example recalculated with
  rationals. No Rust, fit or campaign run; the local checks and the working tree are Claude Code's
  evidence.
- **Claude Code, record of that review and `17` §10:** branch, HEAD `b2a1c2f` (parent `d31e9fa`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L), at `b2a1c2f`: `02` §§A.1–A.3 and §C.2; `17` §§4, 7.4–7.5, 8–9; `16` §§4.5, 5;
  `review::{commit, reveal}`, the lifecycle's reveal-range checks, `gate::{bridging_gate,
  supplementary_review}` and its constants, `orchestrator::{weighted_ratings, expanded_ratings,
  run_item}`, `exploration::outcome_of`, the callers of the gate path (the characterization
  harness only). Exact calculations (C), with fractions in a scratch script not kept: `P(A | F_u)`
  at `c = 1` and `c = ½` against `τ ± ε`; `p_A/(p_A + p_R) = q_c`; the price `c(p − q_c)²` of a
  shift of 1/10 at `c = 1, ½, 0`. Documentation checks: the diff, whitespace (`git diff --check`),
  line widths, `scripts/comment_budget.py`, references and relative links of the edited docs, the
  states across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization,
  calibration, benchmark or mutation run.
- **Astra, review of `ac06d3c`** (as it records): the commit, its parent and the published HEAD
  checked; the diff, the contracts and the relevant code read, the harness's bootstrap path
  included; small exact calculations with fractions. No Rust, fit or campaign run; Claude Code's
  local checks not repeated; the working tree not checked through GitHub.
- **Claude Code, record of that review and the rectifications of `17` §10:** branch, HEAD
  `ac06d3c` (parent `b2a1c2f`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Read (L), at `ac06d3c`: `scoring::bridging::{fit, participates,
  Core::{of, expand}, side_balanced, bridge_scores, coverage}`; `characterization::run::
  {gate_char, sweep}` with `BOOTSTRAPS` and `KEEP`; `gate::{bridging_gate,
  supplementary_review}`; `02` §A.4; `15` A7; `17` §4 (A). Exact calculations (C), with fractions
  in a scratch script not kept: Astra's private-signal example (public frequency ½; individual
  probabilities of `A` 9/50 and 18/25; the public product 9/20 in both, equal to their mean only).
  Documentation checks: the diff, whitespace (`git diff --check`), line widths,
  `scripts/comment_budget.py`, references and relative links of the edited docs, the states
  across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization,
  calibration, benchmark or mutation run.
- **Astra, review of `111e595`** (as it records): the commit, its parent and the published HEAD
  checked; the diff of the four documents read; the relevant code and harness passages checked
  again; the example checked with fractions. No Rust, fit or campaign run; Claude Code's local
  checks not repeated; the working tree not checked.
- **Claude Code, record of that review and `17` §11:** branch, HEAD `111e595` (parent `ac06d3c`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L), at `111e595`: `16` §§3, 4.1–4.2, 5; `17` §§4, 4.1, 7.2–7.3, 8–10; `19` §10; the
  lifecycle's events (`AssignReviewers` to `Pilot2Batch`) and `PartialEpoch`; `NodeEvent`
  (`AdmitRespondent`, `Step`); `results::{rating_leaf, answer_leaf, inputs_root}`; `08`
  PRIV-004. Exact calculation (C), with fractions in a scratch script not kept: §11.3's timing
  construction, 0 for the truthful report and 1/25 for the deviation, the largest value on a grid
  of step 1/100. Documentation checks: the diff, whitespace (`git diff --check`), line widths,
  `scripts/comment_budget.py`, references and relative links of the edited docs, the states
  across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization,
  calibration, benchmark or mutation run.
- **Astra, review of `82bdb9a`** (as it records): the commit, its parent and the published HEAD
  checked; the diff of the four documents, the relevant contracts and the replay and results code
  read; exact calculations with fractions on the timing counterexample and on a disclosure
  example. No Rust, fit or campaign run; Claude Code's local checks and working tree not checked.
- **Claude Code, record of that review and the rectifications of `17` §11:** branch, HEAD
  `82bdb9a` (parent `111e595`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Read (L), at `82bdb9a`: `NodeState::apply` and its `AdmitRespondent`
  arm, `NodeEvent`, `pilot::submit_response`, `results::{inputs_root, inclusion_proof}`, `08`'s
  PRIV-004 and its T73 note; `16` §§4.1–4.2, 5; `17` §§4.1, 7.3, 8. Exact calculation (C), with
  fractions in a scratch script not kept: Astra's disclosure–baseline construction, expected
  contribution 0 without the signal and −¼ with it. Documentation checks: the diff, whitespace
  (`git diff --check`), line widths, `scripts/comment_budget.py`, references and relative links
  of the edited docs, the states across `15`, `17`, `19` and this note. No Rust test, fit, smoke,
  characterization, calibration, benchmark or mutation run.
- **Astra, review of `cf5575a`** (as it records): the commit, its parent and the published HEAD
  checked; the diff of the four documents read; `Node::submit`'s path checked again; the
  disclosure–baseline example checked with fractions (0 without the signal, −¼ with it). No Rust,
  fit or campaign run; Claude Code's local documentation checks and working tree not checked.
- **Claude Code, record of that review and `17` §12:** branch, HEAD `cf5575a` (parent `82bdb9a`),
  no later local commit and a working tree with only `.gitignore` modified checked first. Read
  (L), at `cf5575a`: `Node::{submit, open}` and `NodeState::apply` (a refused event is not
  written); `NodeEvent`; `lifecycle::Event` and the pilot transitions (refused outside `Pilot1`,
  `Pilot2` and `Explored`; an indeterminate reading leaves the state); `pilot::{submit_response,
  batch_id, PilotError}` and `batch_id`'s callers (tests only); `results::{answer_leaf,
  inputs_root, inclusion_proof}`, `network::merkle::leaf_hash` and `hash::tagged` (no salt);
  `review::commit`; `04` §§A node's own disk, Events and replay, Replication between nodes (Who
  reads); `08` PRIV-004, PRIV-006 and Q-1; `16` §§4.6, 5; `17` §§4, 7.2–7.3, 8–11. Exact
  calculation (C), with fractions in a scratch script not kept: a partly informative signal,
  expected contribution 0 without it and `−δ²` with it (−1/16 at `δ = ¼`, −¼ at `δ = ½`), on a
  grid of step 1/40. Documentation checks: the diff, whitespace (`git diff --check`), line widths,
  `scripts/comment_budget.py`, references and relative links of the edited docs, the states
  across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization, calibration,
  benchmark or mutation run.
- **Astra, review of `8af257d`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the relevant code read; the result `−δ²` checked with fractions; the
  code's encoding and hashes reproduced in Python on a synthetic two-leaf Merkle construction. No
  Rust, fit or campaign run; the local working tree and Claude Code's documentation checks not
  checked.
- **Claude Code, record of that review, the rectifications of `17` §12 and `08` PRIV-004.1:**
  branch, HEAD `8af257d` (parent `cf5575a`), no later local commit and a working tree with only
  `.gitignore` modified checked first. Read (L), at `8af257d`: `network::{codec::Writer,
  hash::tagged, merkle::{leaf_hash, merkle_root, merkle_proof, verify_proof}}`,
  `results::{answer_leaf, inputs_root, inclusion_proof}` and `answer_leaf`'s callers (tests),
  `Node::submit` (the object store, then the log); `04` §§Events and replay, Replication between
  nodes; `08` PRIV-004, NET-002, NET-003 and the §15 matrix rows; `16` §4.6. Synthetic
  reproduction (C), in a scratch Python script not kept, with the code's encoding and domain tags
  and synthetic ids: two answer leaves, for either hidden binary answer the proof verifies, its
  one sibling is the other leaf's hash and only the true candidate matches; four leaves, one of
  the two siblings a leaf hash. No other calculation was needed. Documentation checks: the diff,
  whitespace (`git diff --check`), line widths, table columns, `scripts/comment_budget.py`,
  references and relative links of the edited docs, the states across `08`, `15`, `17`, `19` and
  this note. No Rust test, fit, smoke, characterization, calibration, benchmark or mutation run.
- **Astra, review of `7d8db08`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the documents read; a small calculation with fractions on the conditional
  target. No Rust, fit, campaign, Merkle reproduction or local documentation check; the working
  tree and the local checks are Claude Code's evidence.
- **Claude Code, record of that review, its two precisions and `17` §13:** branch, HEAD
  `7d8db08` (parent `8af257d`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Read (L), at `7d8db08`: `04` §§Replication between nodes (Who reads,
  the replica, a writer's own feed), Cuts, Members' objects; `network::cut::{Cut, Mark,
  MemberObject}` (a cut signature carries the cut's encoding and its marks); `network::log`
  (`entry_hash`, `prev`); `protocol::ledger::Ledger::apply` (orchestration only from members,
  each cut's applied and refused entries); the lifecycle's `Commit`, `CloseCommits` and `Reveal`
  arms; `08` NET-004, §§15, 17, 18; `10` T78, T79. Exact calculation (C), with fractions: Astra's
  two laws, `c = 1`, `q_c = 1/4` and `c = ½`, `q_c = ½`. Documentation checks: the diff,
  whitespace (`git diff --check`), line widths, table columns, `scripts/comment_budget.py`,
  references and relative links of the edited docs, the states across `08`, `15`, `17`, `19` and
  this note. No Rust test, fit, smoke, characterization, calibration, benchmark, mutation or
  Merkle reproduction run.
- **Astra, review of `f38f8a4`** (as it records): the commit, its parent and the published HEAD
  checked; the diff, the documentary links and the relevant code of `network::{cut, log,
  replica}`, `protocol::{ledger, lifecycle}` and `p2p::member` read. No Rust, fit, campaign, new
  Merkle reproduction or local documentation check; the working tree and the local checks are
  Claude Code's evidence.
- **Claude Code, record of that review and the rectifications of `17` §13:** branch, HEAD
  `f38f8a4` (parent `7d8db08`), no later local commit and a working tree with only `.gitignore`
  modified checked first. Read (L), at `f38f8a4`: `Replica::{insert, feed}` (acceptance by
  signature, a feed stopping at a fork), `Cut::next`, `added`, `should_sign`, `Ledger::apply`
  and `apply_one` (orchestration only from members; applied and refused entries), the
  lifecycle's `AssignReviewers`, `Commit` and `CloseCommits` arms, `p2p::member`'s proposal and
  co-signing. The ordering construction checked logically against those rules, no calculation
  and no run. Documentation checks: the diff, whitespace (`git diff --check`), line widths, table
  columns, `scripts/comment_budget.py`, references and relative links of the edited docs, the
  states across `15`, `17`, `19` and this note. No Rust test, fit, smoke, characterization,
  calibration, benchmark, mutation or Merkle reproduction run.
- **Astra, review of `176dd8c`** (as it records): the commit, its parent and the published HEAD
  checked; the diff of the four documents read; the relevant passages of replication, cuts and
  the ledger checked again. No Rust, fit, campaign or Merkle reproduction; the working tree and
  the local documentation checks not checked, they stay Claude Code's evidence.
- **Claude Code, record of that review, its three precisions and `19` §10.6:** branch, HEAD
  `176dd8c` (parent `f38f8a4`), no later local commit (the remote-tracking branch at the same
  commit) and a working tree with only `.gitignore` modified checked first. Read (L), at
  `176dd8c`: `Replica::check` and `WriterSet::check` (the writer set, the signature, `MAX_OBJECT`,
  the object's CID against the payload; then new, duplicate or equivocation),
  `SignedEntry::read` (the hash recomputed); `admit_dif_batch` and `latent_batch` (`K_MIN = 2`
  items, `N_LATENT_MIN`), `N1_MIN`, the `AssignReviewers` guard (odd `k` in [7, 11]),
  `review::K_EXTRA`; the module headers of `identity::{enrollment, credential}` and `10` T19–T20
  (enrollment bound to an identity, open); `01` D17; `02` §B.6; `05` [6]–[7]; `08` PRIV-004.1;
  `17` §§8–13; `19` §§6.1, 10. Calculation (C): the sum of the two floors, 300 + 3,000. No other
  calculation was needed. Documentation checks: the diff, whitespace (`git diff --check`), line
  widths of the new lines, table columns, `scripts/comment_budget.py`, references and relative
  links of the edited docs, the states across `15`, `17`, `19` and this note. No Rust test, fit,
  smoke, characterization, calibration, benchmark, mutation or Merkle reproduction run.
- **Astra, review of `cbcbc67`** (as it records): the commit, its parent and the published HEAD
  checked; the diff and the relevant passages of the pilot, revalidation, enrollment, latent,
  harness and contracts read. No Rust, fit, benchmark or campaign run; the local working tree not
  checked.
- **Claude Code, record of that review and the rectifications of `19` §10.6:** branch, HEAD
  `cbcbc67` (parent `176dd8c`), no later local commit (the remote-tracking branch at the same
  commit) and a working tree with only `.gitignore` modified checked first. Read (L), at
  `cbcbc67`: `pilot::{screen, stage1_screen, stage1_fit, admit_dif_batch, submit_response}`
  (`screen` holds the `N1_MIN` gate, then calls `stage1_screen`), `revalidation::latent_batch` and
  its gates before the search, `latent_dif_with` and `LatentParams`' defaults, the lifecycle's
  `Pilot2Batch` guard (`K_MIN`), `gate::bridging_gate` (thresholds on supplied inputs);
  `08` PRIV-004 and PRIV-004.1; `17` §§12.2, 12.5, 13; `19` §§9.3, 10. No calculation was
  needed. Documentation checks: the diff, whitespace (`git diff --check`), line widths of the new
  lines, table columns, `scripts/comment_budget.py`, references and relative links of the edited
  docs, the withdrawn wordings absent from the current passages, the states across `15`, `17`,
  `19` and this note. No Rust test, fit, smoke, characterization, calibration, benchmark,
  mutation, Merkle reproduction or Phase 2 work run.
- **Astra, review of `5af22b9`** (as it records): the commit, its parent and the published HEAD
  checked; the diff of the four documents read; the joins checked; `bridging_gate`,
  `supplementary_review`, `screen` and `stage1_screen` read again. No Rust, fit, benchmark or
  campaign run; the local checks and the working tree are Claude Code's evidence.
- **Claude Code, record of that review, the two joins and `20`:** branch, HEAD `5af22b9` (parent
  `cbcbc67`), no later local commit (the remote-tracking branch at the same commit) and a working
  tree with only `.gitignore` modified checked first. Read (L), at `5af22b9`: `lifecycle::{State,
  Event, step}`; `NodeState::apply`, `NodeEvent`; `Ledger::{apply, apply_one}` (`NotAnEvent`,
  orchestration only from members); `network::cut::{Cut::next, added, MemberObject, sign_cut,
  collect, should_sign}`; `Replica`; `review::{commit, reveal}`; `panel_scores`;
  `reputation::difference_score`; `gate::{bridging_gate, supplementary_review}` and constants;
  `pilot::{screen, admit_dif_batch, N1_MIN}`, `NullifierSet`; `nullifier::prove` (randomized);
  the test support of `crates/protocol/tests/ledger.rs`; `16` §§4–5; `17` §§8–13. Exact
  calculations (C), with fractions in a scratch script not kept: §5's baselines, contributions,
  cohort values and bounds, the unweighted counterexample 85/576, the `f64` recomputation within
  1.2e−16; the assignments by case (102); the gate decisions on the fixture's inputs, with
  `TAU + EPS` as evaluated; §7's declared sums. The order fixtures checked by reasoning against
  `added`, `Ledger::apply` and `17` §13.2, not run. Documentation checks: the diff, whitespace
  (`git diff --check`), line widths of the new lines, table columns, `scripts/comment_budget.py`,
  references and relative links of the edited docs, the states across `15`, `17`, `19`, `20`,
  `docs/README.md` and this note. No Rust test, fit, smoke, characterization, calibration,
  benchmark, mutation, Merkle reproduction or Phase 2 work run; the check neither written nor run.
- **Astra, review of `c76c035`** (as it records): the commit, its parent and the published branch
  checked; the specification, its documentary joins and the relevant code read; baselines,
  contributions, cohorts, counts and declared costs recalculated independently with exact
  fractions; the order fixtures checked by reasoning, not run. No Rust, fit or synthetic check
  run; the working tree and the local checks are Claude Code's evidence.
- **Claude Code, the precisions and the implementation of `20`:** branch
  `docs/phase1-review-alignment`, HEAD `c76c035` (parent `5af22b9`), the remote-tracking branch at
  the same commit, and a working tree with only the owner's `.gitignore` modified checked first;
  `.gitignore` and the ignored directories, `.gpt/` included, left untouched and out of the
  commit. Read (L): `docs/20`, `16` §§4.6–5, `17` §§8–9 and 13; `network::{cut, replica, log,
  consortium, codec}`, `protocol::{ledger, events, node, lifecycle, review, gate, pilot,
  admission, panel_scores}`, `scoring::reputation::difference_score`, the test support of
  `crates/protocol/tests/ledger.rs`. Executed (E), 2026-10-08: `cargo test -p protocol --test
  s2_synthetic_check`, 23 passed, 0 failed, the test binary in 0.91 s (a time of these tests
  only); `cargo clippy -p protocol --test s2_synthetic_check -- -D warnings`, no warning;
  `rustfmt --edition 2021 --check` on the test file and its modules; `scripts/comment_budget.py`;
  `git diff --check` on the staged change; two temporary probes, reverted (r4's weight set to 1
  fails F8; G6's stage-1 attempt logged in cut 2 fails F5 and the costs). Documentation checks:
  references and relative links of the edited docs, the states across `15`, `17`, `19`, `20`,
  `docs/README.md` and this note. Not run: the workspace suite or other targets, any fit, smoke,
  characterization, calibration, benchmark, mutation or Phase 2 work. The fixtures' costs are
  declared counts, not measured times or resources.
- **Astra, review of `e5e0898`** (as it records): the commit, its parent and the published branch
  checked; the seven new Rust files, the relevant code and the documentary joins read; minimal
  Python reproductions of the branches concerned. No Rust run; the 23 passing tests and Clippy are
  Claude Code's evidence.
- **Claude Code, the rectifications of `e5e0898`:** branch `docs/phase1-review-alignment`, HEAD
  `e5e0898` (parent `c76c035`), the remote-tracking branch at the same commit, and a working tree
  with only the owner's `.gitignore` modified checked first; `.gitignore` and the ignored
  directories, `.gpt/` included, left untouched and out of the commit. Read (L): `20`, `17`
  §13, `crates/protocol/tests/s2/{study,order}.rs`. Executed (E), 2026-10-08: the five
  regressions first, on `e5e0898`'s check code — `cargo test -p protocol --test
  s2_synthetic_check`, 23 passed, 5 failed (j3's `I` accepted from a repeated attempt and from a
  misnumbered sequence; G3's `executed` [1, 2]; O4 and O5 with an entry lacking evidence read
  `Indeterminate` at both levels; the conflicting-record case, the costs check and O1's
  iteration not reached behind a failing assertion); after the rectifications the same command,
  28 passed, 0 failed, the approved expected values unchanged; `cargo clippy -p protocol --test
  s2_synthetic_check -- -D warnings`, no warning; `rustfmt --edition 2021 --check` on the test
  file and its modules; `scripts/comment_budget.py`; `git diff --check` on the staged change.
  Documentation checks: references and relative links of the edited docs, the states across `15`,
  `17`, `19`, `20`, `docs/README.md` and this note. Not run: the workspace suite or other
  targets, any fit, latent search, bootstrap, smoke, characterization, calibration, benchmark,
  mutation or Phase 2 work.
- **Astra, review of `3559396`** (as it records): the commit, its parent and the published branch
  checked; the diff, the regressions and the documentary joins read; small Python transcriptions
  of the attempt sequences and of the evidence's aggregation. No Rust run; the 28 passing tests,
  Clippy and the local checks are Claude Code's evidence.
- **Claude Code, record of that review, its two precisions and §5.1:** branch
  `docs/phase1-review-alignment`, HEAD `3559396` (parent `e5e0898`), the remote-tracking branch
  at the same commit, and a working tree with only the owner's `.gitignore` modified checked
  first; `.gitignore` and the ignored directories, `.gpt/` included, left untouched and out of
  the commit. Read (L): `15` in full; the relevant parts of `08` §16, `10` §§1.4–1.5, 3.4 and
  the dependency notes, `16` §9, `17` §8.5, `19` §§6.1, 10; `study.rs`'s terminal validation,
  item states and group report (the precision on later incoherent registrations). Documentation
  checks only: `git diff --check`, `scripts/comment_budget.py`, the widths of the new lines,
  table columns, relative links and references, the states across `15`, `17`, `19`, `20`,
  `docs/README.md` and this note. No Rust test, fit, search, bootstrap, smoke, characterization,
  calibration, benchmark, mutation or Phase 2 work; no code change.
- **Astra, review of `bef891b`** (as it records): the commit, its parent and the published
  branch checked; the diff read and compared with the criteria of `15`, `10` and `08` §16.1. No
  Rust, fit or campaign run.
- **Claude Code, record of that review, §5.1's rectifications and `17` §14:** branch
  `docs/phase1-review-alignment`, HEAD `bef891b` (parent `3559396`), the remote-tracking branch
  at the same commit, and a working tree with only the owner's `.gitignore` modified checked
  first; `.gitignore` and the ignored directories, `.gpt/` included, left untouched and out of
  the commit. Read (L): `15`'s criteria, records and working agreement; `16` §§1–9; `17` §§4,
  7–13; `19` §§6.1, 10.1, 10.5; `20`'s status and §§1–2, 9; `10` §§1.5, 3.4 and the dependency
  notes; `08` §16; the code naming what exists (`panel_scores`, `difference_score`,
  `exploration::{outcome_of, record_outcome}`, `SkillTrack`, the check's `study.rs`).
  Documentation checks only: `git diff --check`, `scripts/comment_budget.py`, the widths of the
  new lines, table columns, relative links and references, the states across `15`, `17`, `20`
  and this note. No Rust test, fit, search, bootstrap, smoke, characterization, calibration,
  benchmark, mutation or Phase 2 work; no code change.
- Never run in this work: full characterization, smoke, mutation or calibration campaigns.
