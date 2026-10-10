# 20 — S2: the synthetic-event technical check

| | |
|---|---|
| **Status** | **Approved by Astra on `c76c035` as the specification of the synthetic technical check**, with four precisions, made in place (below). **Its implementation in test code (`e5e0898`), partly reviewed on `e5e0898`, and its rectifications were approved by Astra on `3559396`, within the check's perimeter** (§9), its two precisions approved on `bef891b`: no general validation of arbitrary flows, no H-c, properness, reputational incentive, confidentiality, true DTF or neutrality shown. The owner's assignments authorized the implementation, the rectifications and their targeted tests, not those reviews. The approvals of [`19`](19-a2-dtf-composition-design.md) §10.6 and of [`17`](17-a1-pilot-batch-design.md) §§8–13 do not cover it. No data collection, adoption of T1 or T2, or start of S2. **§10's correction of the absent baseline and §11's adaptive verification of `17` §14.6, in test code on the owner's assignment, were approved by Astra on `d7a4449`**, within their perimeter: a verification of the computation and of finite constructions, no H-c in the protocol, availability, confidentiality, reputational incentive or neutrality shown. **§12, the check's scoring path through B-b's scorer `protocol::cohort_scores` ([`17`](17-a1-pilot-batch-design.md) §14.9), on the owner's assignment after Astra's review of `23283f2`, was partly reviewed on `7d1b19e` and `0ca81e9`, the scorer corrected after each; the scorer and the check's passage through it were approved by Astra on `6b9f42a`.** The contract of the scorer's future caller (`17` §14.10), partly reviewed on `3e34f6f`, is rectified in place and awaits review. |
| **Baseline** | specification written on `5af22b9` and approved on `c76c035`; implementation in `e5e0898`, rectified in `3559396` and approved there; §§10–11 written on `d2be7da` and approved on `d7a4449`; §12 written on `23283f2`, partly reviewed on `7d1b19e` and `0ca81e9`, its scorer corrected after each, approved on `6b9f42a`; code named by symbol. |
| **Scope** | `19` §10.6's next deliverable: what a technical check on synthetic events would execute, what its fixtures supply, what it must report and when it fails. References, within their recorded limits: design A, B-b and the reporting of cases 5–7 with 6c and 7a (`17` §§8–9), α (`17` §10), T1 the analytic reference and T2 the comparison (`17` §§11–13); B-b's study scores outside the reputation and the gate's decisions recorded with no effect on operational pools (Astra, decided on `cbcbc67`, confirmed on `5af22b9`). |
| **Evidence** | Specification (Claude Code, before `c76c035`): **L** targeted reads of the code named; **C** every expected value recalculated with exact fractions in a scratch script, not kept. Implementation and rectifications (Claude Code): **E** the check's targeted tests, §9. §§10–11 (Claude Code): **C** the expected values with fractions, before the tests; **E** the regressions before the correction and the tests after it, §11.3. §12 (Claude Code): **C** the new expected values with fractions, before the tests; **E** §12's runs; for the numeric correction, **E** the regressions before and after it and the bit-for-bit comparison of the outputs, **D** the finiteness of the means (`17` §14.9); for the correction of `known`'s scale, **E** the regression before and after it and the comparison of the outputs, **D** the bounds and the narrowed identity, **C** their scratch check (`17` §14.9). Astra's reviews: below. |

## Design review (Astra)

Review of `c76c035` (the commit, its parent and the published branch checked; the
specification, its documentary joins and the relevant code read; the baselines, contributions,
cohorts, counts and declared costs recalculated independently with exact fractions; the order
fixtures checked by reasoning, not run; no Rust, fit or synthetic check run by Astra; the working
tree and the local checks stay Claude Code's evidence): this document **approved as the
specification of the synthetic technical check, with four precisions; no blocking error in the
expected results verified**. The precisions, made in place: (A) K2's cohorts are finite sets of
assignments, fixed independently of the outcomes and complete for the check, which no later
assignment enlarges (§3); (B) the fixture's view, held records included, the log's view and the
results derived from each kept apart, the fixture's knowledge never raising `O` or `V`, giving a
contribution or consolidating a cohort (§2, §§5, 7); (C) T1's start a synthetic event the
fixture supplies, its order checked, no execution nor H-c inferred from a record's position (§3,
§4); (D) K4 keeps both of G6's items in its second stage-2 batch, j12's second reading supplied
and unused (§4). The review approves no implementation, none existing then: the implementation
and its targeted tests (§9) rest on the owner's later assignment. R1 stays approved; A1, A2,
B1–B3 and Phase 1 stay open.

Review of `e5e0898` (the commit, its parent and the published branch checked; the seven new Rust
files, the relevant code and the documentary joins read; minimal Python reproductions of the
branches concerned, no Rust run; the 23 passing tests and Clippy stay Claude Code's evidence):
**partial**. Accepted: the record of the review of `c76c035`; the four precisions made; prefixed
cohorts and the report of a later assignment; the log, the held records and the declared costs
kept apart; T1's starts explicitly synthetic; j12's second reading kept and unused; the check
isolated in test code; expected values independent of the code checked. Not approved: the
implementation as a whole, for two defects — K4 counted two registrations of one attempt as two
executions and read a stage's attempts in their numbers' order, not the order recorded, so
`validate` could accept an `I` resting on one non-conclusion logged twice, or on a conclusion
recorded first but numbered after two non-conclusions; and the classifier at (R) turned a
documented contrary order into `Indeterminate` once another entry of `x_O` lacked evidence. A
precision: at (E), the unchecked member's signature inside a `CutSignature` is not by itself
blocking; the contract is to state what is verified and what that evidence concludes. Both
defects are rectified, and the precision made, in §9. The specification stays approved on
`c76c035`; R1 stays approved; A1, A2, B1–B3 and Phase 1 stay open.

Review of `3559396` (the commit, its parent and the published branch checked; the diff, the
regressions and the documentary joins read; small Python transcriptions checking the attempt
sequences and the aggregation of evidence; no Rust run; the 28 passing tests, Clippy and the
local checks stay Claude Code's evidence, with the failed assertions of the run before the
rectifications kept apart from those it did not reach): **approved** — the rectifications of
the two defects found on `e5e0898`, both findings resolved (repeated or incoherent attempts and
the order of the registrations; contrary evidence or an absent precondition lost when other
entries lack coverage); the implementation of the check within this document's perimeter; the
original fixtures' results kept; the `CutSignature` precision, its authenticated reference kept
apart from a check of its inner signature. Scope: the specification approved on `c76c035`, the
implementation and its rectifications on `3559396`, within the synthetic check's perimeter; no
general validation of arbitrary flows; no proof of H-c, properness, reputational incentives,
confidentiality, the true DTF or neutrality; S2 not started. Two non-blocking precisions, made in
§9: a terminal validated on its own prefix can stay valid when an incoherent registration comes
later; duplicated shortfall and refused-batch identities are not checked. R1 stays approved; A1,
A2, B1–B3 and Phase 1 stay open.

Review of `bef891b` (the commit, its parent and the published branch checked; the diff read; no
Rust, fit or campaign run): the record of the approval of `3559396` and §9's two precisions
**approved**. The rest of that review concerns the handoff's closure synthesis (`15`, A1). R1
stays approved; A1, A2, B1–B3 and Phase 1 stay open.

Review of `d2be7da` (the commit, its parent and the published branch checked; the diffs of
three documents, the constructions they cite and the relevant code read; no Rust, fit or campaign
run; the local checks and the working tree stay Claude Code's evidence), as far as this document
goes: confirmed by reading the code, a defect of the check's aggregation — `contribution` gives
`None` for a verdict without a baseline, and `cohort` sums through `filter_map` without making
that case blocking, so it can give `Final(0)`. The approved fixtures do not exercise it; the
check's approval stays valid within its perimeter and covers neither this behaviour nor an
extension. Requiring a defined baseline makes a precondition of the formula explicit, not a new
design of the score, and authorizes no exclusion, fallback or change of the denominator. The rest
of that review concerns `17` §14 (`15`, A1). The correction and the adaptive verification it
pointed to are §§10–11, approved on `d7a4449` (below).

Review of `d7a4449` (the commit, its parent and the published branch checked; the diff, the
regressions, the code and the documentary joins read; independent enumerations in Python summing
the contributions directly with exact fractions, which confirmed the truthful values, the
strategy counts, the minimum and maximum losses, the constant strategies and P1's substitution of
the target; no Rust run; the 39 passing tests, Clippy, the regressions failing first, the probes
and the local checks stay Claude Code's evidence): **approved**, within §§10–11's perimeter — the
correction of the aggregation for an absent baseline; assignments, the denominator, the counts and
the defined contributions kept; no final value and no interval when a needed contribution is
undefined; `g(I) = 0` kept; the adaptive verification on the two finite constructions; the five
contrary channels reproduced within their limits. Accepted, the representation: where cases 6–7
and undefined contributions coexist the value stays `Unresolved`; `Cohort::undefined` keeps the
members whose term is undefined; this precedence is no reputational policy. Scope: a verification
of the computation and of the finite constructions, no proof of H-c in the protocol, availability,
confidentiality, reputational incentives or neutrality. The approval is of `d7a4449`, not of a
later commit. R1 and the check's earlier approvals stay within their perimeters; A1, A2, B1–B3
and Phase 1 stay open; S2 not started.

Review of `23283f2` (the commit, its parent and the published branch checked; the diff, the
documents and the relevant code read; no Rust, fit or campaign run; the local checks stay Claude
Code's evidence), as far as this document goes: the record of `d7a4449`'s approval and the
editorial corrections **approved**; B-b's scorer with no production caller accepted as the next
intervention, independent of O1–O3. The rest concerns `17` §14.8 (`15`, A1). §12 came after that
review and is not covered by it.

Review of `7d1b19e` (the commit, its parent and the published branch checked; the code, the diff,
the tests and the documents read; the numeric defects reproduced with Python transcriptions of the
floating-point operations and rational references; an independent exact enumeration of the 72
states on the 27 report vectors, confirming −41/1728 and −91/5184; no Rust run; the 10, 39 and 7
passing tests, Clippy and the local checks stay Claude Code's evidence), as far as this document
goes: **partial**. The record of the review of `23283f2` approved. The scorer through which §12
runs not approved as a whole, for a numeric defect on its declared domain (`17` §14); three of its
contract's choices accepted there. The numeric correction (§12) was partly reviewed on `0ca81e9`;
the approval covers neither it nor the scorer as a whole.

Review of `0ca81e9` (the code, the diff, the tests and the documents read; the counterexample
reproduced with a Python transcription of the operations and a rational reference; no Rust run;
the regressions, the 19, 9 and 39 tests, the comparison of 988 364 calls, Clippy and the local
checks stay Claude Code's evidence), as far as this document goes: **partial**. The record of the
review of `7d1b19e` and the corrections of that review's cases A–E approved (`17` §14). The scorer
through which §12 runs not approved as a whole: its scale common to the defined terms and the
pending radii could cancel a term from `known`. Its correction (§12) was reviewed on `6b9f42a`; the
approval covers neither it nor every numeric claim of `17` §14.9.

Review of `6b9f42a` (the diff, the code, the regression and the documents read; the numeric cases
transcribed in Python and 1 200 sums checked against exact rationals, 210 of them where the
narrowed identity applied; no Rust run; those checks do not replace the general derivation; the
20, 9 and 39 tests, Clippy and the comparative traces stay Claude Code's evidence), as far as this
document goes: **approved, with no further blocking finding** — the correction of `known`'s scale,
the record of the review of `0ca81e9`, the scorer as an isolated component within `17` §14.9's
contract and numeric limits, and §12's passage of the check through it. Acquired within that
perimeter: no further enumeration or comparison is needed for this approval; a later change needs
checks pertinent to it. Not shown: A1's closure, an adoption of A, C or T1, the hypotheses in the
protocol, availability, confidentiality, reputational incentives or neutrality.

Review of `3e34f6f` (the contract of `17` §14.10 read and compared with the cuts, the ledger,
`NodeState`, `NodeEvent`, the lifecycle and §9's `study::freeze`; no Rust run), as far as this
document goes: the record of the scorer's approval on `6b9f42a` **approved**; the scorer and §12's
check through it stay approved. The freeze derived from today's lifecycle, the rule of
`study::freeze`, accepted as the direction of `17` §14.10's isolated component only. The rest
concerns `17` §14.10, not approved as a whole and rectified there.

## 1. What the check is

The first kind of evidence of `19` §10.6 (A): events and outcomes supplied as fixtures. It checks
that given events yield the records, states, classifications, contributions and counts of `17`
§§8–9 and §13, and that the costs its fixtures declare are accounted as declared. It runs no
fitter, latent search, bootstrap or benchmark and involves no human participant; its data are
synthetic.

Passing it shows that an implementation handles these fixtures as specified. It does not show
properness, H-c or reputational incentives (an arithmetic match proves none of them), the
anonymity or confidentiality of a future field study, the absence of disclosures off the log, the
true DTF, neutrality, nor any cost of a fit, of human availability or of production.

## 2. Boundary of the execution

| Component | Executed (real code) | Supplied by the fixture | Missing: check code to write | Excluded |
|---|---|---|---|---|
| feeds, replica, cuts | `FeedWriter::sign` over `TransparencyLog` entries; `Replica::insert`, through `Replica::check`; `Replica::{feed, chain}`; `Cut::next`; `cut::{added, sign_cut}`; `Member::sign` | each feed's entries in order; which cut counts what; test keys from `FeedWriter::from_seed` and `Member::from_seed`, three members with `t = 2`, as `crates/protocol/tests/ledger.rs` builds them | the fixture builder | `p2p`, libp2p, sync; durable stores (an in-memory `Replica` suffices) |
| ledger, lifecycle | `Ledger::apply`, its signature check included; `NodeState::apply`; `lifecycle::step`, the `Reveal` check against `review::commit` included | drafts; synthetic panel nyms `Nym([b; 32])`; probabilities and nonces; `seed_from_beacon` and the appeal's booleans as the lifecycle's inputs, no beacon run | — | `Pilot1Batch`, `Pilot2Batch`, `Explore`, `Administer`, `Revalidate` (they move items toward the pool); `NodeEvent::Results`; `AdmitReviewer`, `AdmitRespondent`; `orchestrator::{run_item, review_round}` |
| identity | `deposit_with_identity`, through `NodeState::apply`: every deposit carries a valid proof | test credentials as in `ledger.rs`: `Issuer::new`, `Credential::from_secret`, issuance, `nullifier::prove` with `Role::Propose` and `deposit_context`; one author per item, quota 1 | — | enrollment, T20, recruitment; reviewers' and respondents' proofs: panelists and respondents are synthetic ids, declared as such |
| gate | `gate::bridging_gate` on the fixture's inputs, with `TAU`, `EPS`, `APPEAL_GAP` | score, gap, coverage; the outcome of a supplementary review, as `Resolve`'s input | — | `bridge_scores`, `fit`, the bootstrap; `gate::supplementary_review`, which runs a fit: its outcome is supplied, never reported as a call's result |
| pilot gates | `pilot::screen` on a `NullifierSet` of synthetic ids with empty matrices: it refuses at `N1_MIN`, and no input reaches `stage1_screen`; `pilot::admit_dif_batch`, `latent_batch`'s first gate, at `K_MIN` | declared counts of admitted pseudonyms; per executed attempt, each item's status, a verdict or a named non-conclusion | — | `stage1_screen`, `stage1_fit`, `latent_batch` past its first gate, `latent_dif`; respondent admission (`submit_response`) |
| study records | — | statuses, outcomes, references to evidence, declared costs; T1's synthetic starts; G6's records, held | `S0`, group, start, attempt, terminal (`A`, `R`, or `I` with `17` §9.2's content), shortfall and refused-batch records; an encoding both `NodeEvent::decode` and `MemberObject::decode` refuse; their reader | a lifecycle state or `NodeEvent` variant for them: `Ledger::apply` refuses each as `NotAnEvent`, the state unchanged |
| freezes, starts | the applied order of `CutReport` | — | `Φ_j` from the applied steps — `Score` on `Pass` or `Reject`; `Resolve` unless appealable; `Appeal` or `AppealExpires` after an appealable decision (`16` §5) — and `Φ_G`; T1's start check against the supplied start; T2's hold | T58, or any rule letting a freeze pass |
| B-b | `panel_scores::first_panel_baselines` on frozen weights; `difference_score`; since §12 both through `cohort_scores::score`, which also forms the contributions, the counts and the cohorts | frozen weights per nym | the contribution per outcome (`17` §8.1), `N`, `O`, `V`, the cohort rule, consolidation and the bound (§8.3) | `SkillTrack`, `exploration::record_outcome`, `ResultRecord`, probation, shrinkage, cap, CUSUM; any per-nym score on the log |
| order evidence | `CutReport::{applied, refused}`; feeds' chains; cuts' marks; `MemberObject::CutSignature` on members' feeds | — | the classifier at (R) and (E), with `17` §13.2's four outcomes; `x_O` and the relevant commitments (§13.3) | (I), declared only; any dependence finding; a results root and inclusion proofs (`08` PRIV-004.1) |
| snapshot | — | `Ω` | the report of §7 | its publication |

`nullifier::prove` draws from the operating system's randomness, so entries' hashes and CIDs
change between builds: every expected result names positions, states and values, never a hash.

**Three sources, kept apart.** (1) *Supplied by the fixture*: the entries it logs, the cuts that
count them, the gate's inputs, `Resolve`'s outcome, the declared counts — and G6's records,
produced under T2 and held, with their costs; the check receives these from the fixture, never
from the log. (2) *Available in the log's prefix*: the entries the cuts through it count, applied
or refused. (3) *Derived*: from (2) alone, every study state, case, contribution, `N`, `O`, `V`,
cohort value and pair; from G6's held records, their outcomes, reported apart as the fixture's;
the costs as declared, each total split between the records in the prefix and the held ones. No
held record raises `O` or `V`, gives a contribution or consolidates a cohort.

## 3. Conventions of the check

Synthetic, limited to this check, recorded in the fixtures; none is a parameter of the protocol
or of a future human study.

- **K1, `Ω`.** The prefix through cut 4 of the main scenario; cut 5 holds the late records.
  Declared in a study record `S0` in cut 0.
- **K2, cohorts.** For `x ∈ {u, v, w}`, a finite set of assignments listed in `S0`, in cut 0,
  before any assignment or outcome: `K_u` = {j1, j3, j5, j11}, `K_v` = {j1, j2, j3}, `K_w` =
  {j1, j5} — every assignment of `x` in the check, so a rule on the assignment record alone
  (`17` §8.3); `|K| = N_x`. The check verifies that `S0` precedes every assignment and that each
  set equals its nym's accepted assignments in all the log it holds; a later assignment is
  reported, never added (F8). No rule closing a protocol epoch is introduced.
- **K3, frozen weights.** `r4` weighs 2, `r9` 0, every other nym 1; recorded in `S0`, before any
  assignment.
- **K4, attempt rule** (each group record). At most two executed attempts per stage by the group's
  term, a cut number, inclusive. An executed attempt gives each item of its batch a verdict or a
  named non-conclusion; an item's first conclusion stands, later readings recorded and unused; a
  stage-2 batch is the stage-1 survivors at every attempt. An item is exhausted, `I`, once two
  executed attempts of one stage gave it named non-conclusions by the term, none concluding. A
  refused administration or batch executes no attempt and consumes none: it is recorded apart,
  never as `I`. Past the term without conclusion or exhaustion the item stays pending.
- **K5, reveal close.** Per item, the position of its first panel's `Score` in cut 1, declared
  in its group record; a reveal absent by then is missing, and the ledger refuses that `Score`
  with `PartialEpoch`.
- **K6, records.** One object per record: `S0`; one group record per group; one start record per
  T1 group started; one attempt record per group and attempt; one terminal record per item; one
  record per shortfall and per refused batch.
- **K7, constructions.** A T1 group starts after its `Φ_G`. Its start is a synthetic event the
  fixture supplies, a start record (G1–G5, cut 2); the check verifies `Φ_G` < start < the
  group's first attempt, shortfall or refused-batch record. A record's position shows where the
  record lies, not when a procedure began; no execution nor H-c is inferred from it. Its objects
  are opened by being logged (`17` §13.3). The T2 group produces at the position its record fixes
  and holds every record until `Φ_G`.
- **K8, numbers.** Gate inputs lie away from the thresholds (`TAU + EPS` evaluates to
  0.8200000000000001), so no rounding decides a case. Expected values are exact rationals; a
  computed `f64` matches within 1e−12 (the scratch's `f64` recomputation differs by at most
  1.2e−16).

## 4. The main scenario

One feed, member 0's (M0), carries every step and record; cuts 0–5, each signed by members 0 and
1. Reviewers `u`, `v`, `w` are the scored ones; `r4`–`r14` complete the panels.

| Cut | Entries, in feed order |
|---|---|
| 0 | `S0`; group records G1–G7 (members, construction, K4's budget and term, K5's reveal close; G6's production position, after cut 1); per item, `Deposit` and `Admit` |
| 1 | per item: `AssignReviewers`, seven `Commit` (`u`'s on j11 included), `CloseCommits`, the reveals (`u`'s on j11 missing), `Score`; j4's extra round (`AssignExtraReviewers`, four commits, `CloseCommits`, four reveals) and `Resolve`; j6's `AppealExpires` |
| 2 | the synthetic start records of G1–G5; stage-1 attempt records of G1, G2, G3, G5 |
| 3 | stage-2 first-attempt records of G1, G2, G3; G4's shortfall record; G5's refused-batch record; terminal records j1, j2, j4, j9 |
| 4 | G2's second stage-2 attempt record; j3's terminal record. **`Ω` closes here** |
| 5 | G3's second stage-2 attempt record; terminal records j5, j6 |
| held | G6 (T2), produced after cut 1 and never logged while `Φ_G6` is unreached: its start; both survive stage 1; stage 2 first attempt j11 non-conclusion, j12 `A`; second attempt, on K4's batch {j11, j12}, j11 non-conclusion and j12 `R`, a second reading recorded and unused, j12's `A` standing; terminal records j11 `I`, j12 `A` |

| Item, group | Panel (extra) | Reveals | Gate inputs → `bridging_gate` | Lifecycle at `Ω` | Study outcome at `Ω` (after cut 5) |
|---|---|---|---|---|---|
| j1, G1 (T1; term 4) | u v w r4–r7 | u ¾, v ¼, w ½; others ½ | 0.90, 0.05, 3 → `Pass` | `Pilot1` | `R`, first stage-2 attempt |
| j2, G1 | v r4–r9 | v ¾, r4 ¼, r9 1; others ½ | 0.50, 0.10, 2 → `Reject` | `Rejected(Defect)` | `A`, first stage-2 attempt |
| j3, G2 (T1; term 4) | u v r4–r8 | u ¼, v ¾; others ½ | as j1 → `Pass` | `Pilot1` | `I`: non-conclusions at both stage-2 attempts |
| j4, G2 | r4–r10 (r11–r14) | ½ | 0.80, 0.05, 2 → `SupplementaryReview`; `Resolve { Reject }` supplied | `Rejected(Borderline)` | `R`, first attempt; its second reading (`A`) unused |
| j5, G3 (T1; term 6) | u w r4–r8 | u ¾, w ¼; others ½ | `Pass` | `Pilot1` | pending, one non-conclusion (`A`) |
| j6, G3 | r4–r10 | ½ | 0.50, 0.40, 2 → `AppealEligible`; `AppealExpires` | `Rejected(Polarized)` | pending, one non-conclusion (`I`, exhausted) |
| j7, j8, G4 (T1; term 3) | r4–r10 | ½ | `Pass` | `Pilot1` | pending: stage-1 shortfall, 120 admitted of 300 |
| j9, G5 (T1; term 3) | r4–r10 | ½ | `Pass` | `Pilot1` | `R` at stage 1 |
| j10, G5 | r4–r10 | ½ | `Pass` | `Pilot1` | pending: stage-2 batch {j10} refused |
| j11, G6 (T2; term 3) | u r4–r9 | `u` committed, no reveal; others ½ | `Score { Pass }` refused, `PartialEpoch` | `Revealing` | held `I` |
| j12, G6 | r4–r10 | ½ | `Pass` | `Pilot1` | held `A` |
| j13, G7 (T1; term 6) | r4–r10 | ½ | 0.50, 0.40, 2 → `AppealEligible`; no appeal by `Ω` | `AppealEligible` | pending: `Φ_G7` unreached, not started |
| j14, G7 | r4–r10 | ½ | `Pass` | `Pilot1` | pending: not started |

A row reading `Pass` alone takes j1's inputs (0.90, 0.05, 3). Gate-only rows, no lifecycle:
(0.95, 0.00, coverage 0) → `SupplementaryReview` (below `MIN_COVERAGE`); the four decisions
above, each recomputed on its inputs.

**Fixtures.** Each row's expected results hold at `Ω` unless marked; the scenario's common
expectations follow the table.

| Fixture | Purpose; units | Events, prefix | Rule | Expected | Fails if |
|---|---|---|---|---|---|
| F1 | conclusive outcomes from completed reports, the gate apart; j1, j2 and their 14 assignments | cuts 0–3 | K4 | j1 `R`, j2 `A`, terminal records in cut 3; assignments cases 4 and 3; contributions in §5 | a gate decision read as the outcome; a pilot step logged; any item in `ActivePool`, `Contested`, `Explored` or `Measured` |
| F2 | verifiable `I`; j3 and its 7 assignments, j4 | G2's records, cuts 2–4 | K4, term 4 | j3 `I`, its record carrying `17` §9.2's six items (the rule by reference, the group, both attempts with statuses and evidence references, no earlier conclusion, the reason, its position); its assignments add 0, enter `O`, not `V`; j4 `R` from its first attempt | `I` after one non-conclusion, or without the attempts' evidence; a term depending on `p` or `b`; j3 in `V`; j4's second reading replacing its conclusion |
| F3 | an indeterminate attempt without exhaustion; j5, j6 and 14 assignments | cuts 2–3; second attempt in cut 5 | K4, term 6 | at `Ω` both pending (one executed attempt of two, term not reached), assignments case 2, no term; after cut 5, j5 `A`, j6 `I` | a non-conclusion read as `I`; a pending item read as 0 or dropped from `N` |
| F4 | a shortfall and a refused batch, apart from `I`; G4, G5 | `pilot::screen` on 120 synthetic ids returns `NotEnoughRespondents { have: 120, need: 300 }`, shortfall record in cut 3; G5's stage 1 in cut 2 (j9 `R`, j10 survives), then `admit_dif_batch(1, 3000, N_LATENT_MIN)` returns `BatchTooSmall { items: 1 }`, refused-batch record in cut 3 | K4: no attempt executed or consumed | j7, j8, j10 pending past their terms, two distinct records, neither `I`; j9 `R`; no search for either refusal | either refusal made `I`, counted as an attempt, merged with the other, or simulated rather than returned by the gate; any computation past a gate |
| F5 | a missing report, its item holding an outcome (T2); j11 (u's assignment and 6 others), j12 | `u`'s commitment in cut 1, no reveal by K5's close; `Score` refused; G6's records held | K4, K5, K7 | `u`'s assignment case 6, with its commitment and the held `I` recorded beside it, as the fixture's: no term although `g(I) = 0` needs no `p`, no `O`; j11's other assignments case 7; j12's case 2 (record held), its held second reading unused; no G6 record logged | `u`'s assignment given 0, a term or an `O` from the item's `I`; the other panelists' baselines formed over the six reveals present (`17` §9.4, 7c); a held record logged or counted available; j12's second reading used |
| F6 | freeze not reached; j11's other panelists, G7 | j13's `Score { AppealEligible }`, no `Appeal` or `AppealExpires` by `Ω` | K7 | `Φ_j13`, `Φ_G7`, `Φ_G6` unreached; no attempt record of G7; j13's 7 assignments case 7, j14's 7 case 2 | a G7 attempt recorded; `Φ_j13` placed at `Score`; any of these assignments valued |
| F7 | a record after `Ω`; G3, cohorts | cut 5 | — | the snapshot at `Ω` unchanged; the state after cut 5 updated (§5) | the snapshot rewritten; a cut-5 record deleted, substituted or valued at `Ω` |
| F8 | cohorts fully evaluable or not; `K_u`, `K_v`, `K_w` | as above | K2 | §5 | a value while a member is in case 2, 6 or 7; an interval for `K_u`; D1 attributed to the consolidated cohorts alone |
| F9 | the gate on supplied inputs | the gate rows | K8 | the decisions above | a decision differing; a recorded `Score` differing from it; `Resolve`'s outcome reported as `supplementary_review`'s result |

**Common to the scenario.** `Ledger::apply` refuses every study object as `NotAnEvent`, and its
only other refusal is j11's `Score` (`PartialEpoch`); no item enters `ActivePool`, `Contested`,
`Explored` or `Measured`; each started T1 group's synthetic start follows its `Φ_G` and precedes
its first attempt, shortfall or refused-batch record (K7); nothing of `SkillTrack`, a results
event or a per-nym score reaches the log.

## 5. Scores, counts and baselines

Weights K3. A baseline is the weighted mean of the other first panelists (`16` §4.6); `g` as in
`17` §8.1; `π = 1` under A. The values are worked by hand from these definitions and checked with
fractions, not taken from the functions the check exercises. Every value and count below comes
from the log's prefix; G6's held outcomes (j11 `I`, j12 `A`) enter none of them (§2).

| Assignment | `p` | Baseline | Outcome | `g` |
|---|---|---|---|---|
| (u, j1) | ¾ | 13/28: v ¼, w ½, r4 ½ at weight 2, r5–r7 ½ | `R` | −17/49 |
| (v, j1) | ¼ | 15/28 | `R` | 11/49 |
| (w, j1) | ½ | ½ | `R` | 0: a verdict with `p = b` (`17` §8.1, D4), counted in `V` |
| (v, j2) | ¾ | 5/12: r4 ¼ at weight 2, r5–r8 ½, r9 1 at weight 0 (an unweighted mean, 13/24, would give 85/576) | `A` | 5/18 |
| (u, j3), (v, j3) | ¼, ¾ | not needed | `I` | 0 whatever `p`: in `O`, not in `V` |
| (u, j5), (w, j5) | ¾, ¼ | 13/28, 15/28 | `A`, cut 5 | 11/49, −17/49 |
| (u, j11) | none | — | held `I` | none: case 6 |

| Cohort | `N = \|K\|`, `O`, `V` at `Ω` | At `Ω` | After cut 5 |
|---|---|---|---|
| `K_v` = {j1, j2, j3} | 3, 3, 2 | consolidated: `Ŝ = (11/49 + 5/18 + 0)/3 = 443/2646` | unchanged |
| `K_u` = {j1, j3, j5, j11} | 4, 2, 1 | known sum −17/49; no value (j5 case 2, j11 case 6) and no interval (case 6) | `O` 3, `V` 2, known sum −6/49; still no value (j11) |
| `K_w` = {j1, j5} | 2, 1, 1 | known sum 0, one pending member: the sum within [−1, 1], `Ŝ` within [−½, ½] (`17` §8.3, D3) | consolidated: `Ŝ = −17/98` |

**Assignments by case**, all 102 (98 first-panel, 4 extra): at `Ω`, case 2: 49; 3: 7; 4: 25;
5: 7; 6: 1; 7: 13. After cut 5, case 2: 35; 3: 14; 5: 14; the others unchanged. No missing
report gets 0; no unresolved freeze gets a value; an item's `I` completes no assignment without a
report.

## 6. Order evidence

Base for O1–O6: members M0–M2 (`t = 2`), relay W; on M0, item `k` deposited, admitted and
assigned to seven synthetic nyms, then its relevant commitments `C`, seven `Commit` and
`CloseCommits` (`17` §13.3); `O` a terminal record of an item of another group, with one
disclosure entry `x`, a study object. Cuts signed by members 0 and 1; prefixes complete unless
stated.

| Fixture | Feeds and cuts | (R) | (E) | Fails if |
|---|---|---|---|---|
| O1, one feed | M0: base, `C`, `x`; cut 0 counts it | verified | verified, by one chain | `x`'s refusal (`NotAnEvent`) changes the outcome; the case reported as a cross-feed one |
| O2, two feeds | M0: base, `C`; cut 0 counts M0; M1: member 1's signature of cut 0, then `x`; cut 1 counts M1 | verified | verified: `x` follows, on M1, a signature of a cut counting `C` | (E) not verified, or verified without that reference |
| O3, a relay | M0: base, `C`; W: `x`, signed by the builder before `C`; cut 0 counts M0, cut 1 counts W | verified | indeterminate: W carries no reference to `C` | (E) verified; (R) read as excluding early information (`17` §13.1) |
| O4, contrary | M0: base, `C` without `CloseCommits`; M1: `x`; cut 0 counts both; M0 then member 0's signature of cut 0 and `CloseCommits`, counted by cut 1 | contrary order documented | contrary order documented: `CloseCommits` follows, on M0, a signature of a cut counting `x` | read as a violation of H-c, or as the channel used |
| O5, absent | O4 through cut 0 | precondition absent in the prefix | precondition absent in the prefix | reported contrary or indeterminate; read as showing `CloseCommits` exists nowhere (O4 brings it later) |
| O6, incomplete | (a) O2 with the third `Commit` missing from the verifier's replica; (b) O3 through cut 0, `x` held and counted by no cut | indeterminate: (a) `Ledger::apply` returns `CutError::Missing`; (b) `x` uncounted | indeterminate: (a) `C` unidentifiable; (b) no reference | any other outcome |

**In the main scenario** (one feed). The (ii) pairs at `Ω`: the 5 terminal records logged by
`Ω` with their groups' attempt records, against the 14 items reviewed — 70 pairs, verified at (R)
and (E), every `CloseCommits` in cut 1 and every such record in cuts 2–4. O7, the (iv) pairs:
those 5 records against the 6 groups administered (G1–G6) — 30 pairs, precondition absent in the
prefix at both levels, since the check logs no reference binding answers; filling it with a
results root or an inclusion proof fails the check. Cut 5's records add pairs to the state, not
to the snapshot.

## 7. Snapshot at `Ω` and costs

**Minimal content.** Per item: the study state — terminal, with its record's position, or pending
with its kind: not started, attempt indeterminate, past term with a shortfall, refused batch,
held (T2), freeze unreached — and, apart, the lifecycle state from the ledger. Per assignment: its
case and why; for the declared cohorts, each contribution or why none. Per group: its
construction, `Φ_G` reached or not, the first attempt's position against `Φ_G` (T1), attempts
executed and refused per stage, budget used, term, records held. Per cohort: `N`, `O`, `V`, `|K|`,
consolidated or why not, known sum, final value or bound. Per pair: outcome, level, prefix,
evidence. Costs as declared, apart from any measure of the run.

| Declared count | At `Ω` | After cut 5 |
|---|---|---|
| participations, stage 1 | 1,620: five admitted batches of 300, G4's 120 — log 1,320, held 300 | 1,620 |
| participations, stage 2 | 21,000: six batches of 3,000 and G5's refused batch of 3,000 — log 15,000, held 6,000 | 24,000 (log 18,000) |
| pseudonyms counted by the gates | per batch, the same numbers; never summed into distinct pseudonyms | — |
| distinct persons | not reported: no count shows them | — |
| answers, items and 2 anchors per participation | 87,480, refused batches included — log 62,280, held 25,200 | 99,480 (log 74,280) |
| searches run, as declared | 5 stage-1 fits (log 4, held 1); 6 latent searches, 94 optimizer runs (17, 9, 25, 17, 17, 9: G1, G2, G3's first stage-2 attempts, G2's second; G6's two, held) | 7 searches, 111 runs (G3's second, 17) |
| refused before the fit | 2, no search (log) | 2 |

The approved totals stand; each splits between the records the prefix counts and G6's held
records, whose side is the fixture's declaration, not evidence from the log (§2). The check
verifies these sums and that each declared search's runs lie in {9, 17, 25} (`19` §9.3); it
measures none of them. A replay or record-handling time, if reported, is named as such
and extrapolated to no fit, human availability or production. The study's scores and decisions
reach neither the reputation nor an operational pool.

## 8. For the implementer

- **Inputs**: §§3–4's conventions, log and item table; §6's feeds; §7's declared costs.
- **Results to compare**: §4's rows and common expectations; §5's values and counts; §6's
  outcomes; §7's sums — at `Ω` and after cut 5.
- **Capabilities to develop**: the "Missing" column of §2.
- **APIs left out**: the "Excluded" column of §2. Calling one, running any fit, or reporting a
  supplied outcome as a call's result fails the check, as does any "Fails if" of §§4–6 or any
  difference from an expected result.
- **Conclusions not drawn**: §1's.

**Left open, recognizable as conventions** — none blocks writing the check; each needs a decision
before a field study:

1. the reveal close has no representation in code; K5 declares one (T58's questions apart);
2. whether a shortfall or a refused batch consumes an attempt or counts as `I` (`17` §§9.2,
   9.6); K4 says neither;
3. a stage-2 retry's batch once a member has concluded, given `K_MIN`; K4 keeps the survivors
   (`17` §7.3's closure at association);
4. where study records live and who reads them; K6–K7 log them under T1, and the access
   decisions of `19` §10.6 remain;
5. under 7a a missing first-panel report keeps the item's freeze unreached, so an outcome
   coexists with it only as a record held under T2 (F5); a disclosed one would need a rule letting
   the freeze pass (T58 or a term), not approved.

## 9. The implementation (approved on `3559396` within the check's perimeter)

Written by Claude Code on `c76c035` and committed in `e5e0898`, on the owner's assignment; the
review of `c76c035` approved the specification above, not this implementation. Astra's review of
`e5e0898` (above) was partial; its two defects are rectified below, on a further assignment, in
`3559396`, which Astra approved within the check's perimeter (above).

**Where.** Test code only: `crates/protocol/tests/s2_synthetic_check.rs`, one test per fixture,
common expectation or regression, and its support `crates/protocol/tests/s2/` — `fixture.rs`
(the builder of §§4 and 6), `records.rs` (the study records and their reader), `replay.rs` (the
log view), `study.rs` (freezes, K4 and K6, cases, B-b, cohorts, T1 and T2 checks, the snapshot,
the costs), `order.rs` (the classifier at (R) and (E)). No production code, API, `NodeEvent`
variant, dependency, threshold, golden output or policy changes.

**Executed**, as §2's column: `FeedWriter::sign` over `TransparencyLog` entries;
`Replica::insert` (its `check`); `Replica::{feed, chain}`; `Cut::next`; `cut::{added, sign_cut,
member_objects}`; `Member::sign`; `Ledger::apply`, and through it `NodeState::apply`,
`deposit_with_identity` and `lifecycle::step` with the `Reveal` check against `review::commit`;
`Issuer::new`, `Credential::from_secret`, issuance and `nullifier::prove`; `bridging_gate`;
`pilot::screen` on 120 synthetic ids with empty matrices (it returns `NotEnoughRespondents`, not
the `RowCountMismatch` its next check would give, so no input passes its first gate);
`admit_dif_batch(1, 3000, N_LATENT_MIN)`; `first_panel_baselines`; `difference_score`. The
shortfall and refused-batch records carry what those two calls returned. **Supplied**, as §2's
column, held records included. **Not called**: the "Excluded" column — no fit, latent search or
bootstrap; `supplementary_review` (j4's `Resolve { Reject }` is supplied and reported as such);
no pilot step, admission or results event; nothing of the reputation, `SkillTrack` or the pools;
no inclusion proof; no `p2p`.

**Choices within the specification**, none changing an expected value: a log position is an
entry's index in the replay's examination order across cuts (in the main scenario, M0's
sequence); `Pass`-only rows take j1's gate inputs (§4); the runs per declared search are mapped
as in §7; j4's second reading is `A`; evidence references are synthetic labels, checked present,
not recomputed; `Φ_j` reads applied steps only; an extra reviewer's report counts completed on
its accepted reveal, K5 declaring first panels' closes only; contributions are formed for the
declared cohorts only (§7), baselines only past `Φ_j`; (iv)'s reference is an accepted results
event, none logged.

**Attempt records** (rectification of the first defect). A group's attempt records are read in
the order recorded — log order, or the held list's — and never reordered. `study::sequence`
checks K6 and K4 on them first and repairs nothing: one attempt logged twice (`Repeated`), two
different records claiming one (group, stage, number) (`Conflicting`), or a stage's attempt
recorded out of its number's order or numbered past the budget (`Misnumbered`) refuses the
derivation. `k4` then reads the records in that order, the first conclusion standing; `validate`
matches each attempt it used to the terminal's references by that attempt's own CID. A
derivation resting on an incoherent sequence is refused: a terminal record whose earlier
attempts include it gets `TerminalError::Sequence`. A terminal is validated on the attempts
recorded before it, so one validated on its own prefix stays valid when an incoherent
registration comes later; the group's report still names the incoherence, read on every attempt
record of the prefix, and only its items with no valid terminal read `Pending::Incoherent`. In
`executed` and in the costs each distinct attempt counts once, however often logged, so the same
records never mean two executions to one function and one to another. Not checked, a limit of
the perimeter verified and no guarantee: duplicated identities of shortfall and refused-batch
records.

**Order evidence** (rectification of the second defect). At each level, each entry of `x_O` is
classified alone — not examined (R) or not held (E); held with no reference either way (E);
shown before a relevant entry; shown after every relevant entry — then aggregated as `17` §13.2
says: a contrary order for one entry is reported, with its witness (that entry and the relevant
entry it precedes), whatever the others; so is an absent precondition, with an entry examined
(R) or held (E) while the relevant set is not closed; `Verified` needs every entry shown after
every relevant entry; the entries lacking evidence are listed apart (`order::Evidence`: outcome,
witness, uncovered). An incomplete prefix still leaves the relevant set unidentifiable and every
entry indeterminate.

**The `CutSignature` reference at (E)** (Astra's precision). It is used when (i) the object is an
entry of its member's own feed — `member_objects` matches the feed's writer to the member's key —
whose writer signature `Replica::check` verified on insertion; (ii) its cut equals a cut the
replay applied in the prefix, one `Ledger::apply` accepted with the threshold's signatures; (iii)
the later entry follows it on that feed, and the cut's mark of the earlier entry's writer covers
that entry on the chain `Replica::chain` rebuilds. Not verified: the member's signature inside
the object, over the cut's checkpoint (`Consortium::signed_by` is crate-private and stays so). It
concludes `17` §13.1's existence precedence: the feed's writer logged an object carrying that
cut, whose marks commit to the earlier entry's chain, before the later entry — not that the
member signed the cut by that object, which the applied cut's threshold signatures show apart.

**Negative cases**: a terminal `I` after one non-conclusion, missing an attempt's reference, or
resting on attempts without evidence; T1 starts before `Φ_G`, records before a start or without
one, a T2 record logged while held; cut 6 after the main scenario — j15 assigned to `u`, G7's
start while `Φ_G7` is unreached — which K2's check reports (`K_u` stays four) and T1's refuses.
Since the rectifications: one attempt recorded twice, or claimed by two records; a conclusion
numbered 3 recorded before attempts 1 and 2; G3's first stage-2 attempt logged again on M1 in a
cut 4' (`executed` [1, 1], stage-2 participations in the log 12,000, j5 and j6 incoherent); O4's
contrary order beside an entry no cut counts (R) or the verifier lacks (E); O5's absent
precondition, and O1's order not verified, beside such entries.

**Evidence** (E, Claude Code, 2026-10-08):

- on `e5e0898`'s implementation: `cargo test -p protocol --test s2_synthetic_check`, 23 passed,
  0 failed (0.91 s, a time of these tests only); Clippy, rustfmt, the comment budget and
  `git diff --check` clean; two temporary probes, reverted (K3's weight of r4 set to 1 fails F8;
  G6's stage-1 attempt logged in cut 2 fails F5 and the costs);
- the five regressions written before the rectifications, run on `e5e0898`'s check code: 23
  passed, 5 failed — the repeated attempt and the misnumbered sequence each accepted j3's `I`;
  G3's `executed` read [1, 2]; O4 with an uncounted or lacking entry read (`Indeterminate`,
  `Indeterminate`) for (`Contrary`, `Contrary`); O5 likewise for (`Absent`, `Absent`). The
  conflicting-record case, the costs check and O1's iteration sat after a failing assertion and
  were not reached in that run;
- after the rectifications: `cargo test -p protocol --test s2_synthetic_check`, 28 passed, 0
  failed, every expected value of §§4–7 unchanged (scores, cohorts, the 102 assignments, the
  declared costs); `cargo clippy -p protocol --test s2_synthetic_check -- -D warnings`, no
  warning; `rustfmt --edition 2021 --check crates/protocol/tests/s2_synthetic_check.rs` (its
  modules included), `python3 scripts/comment_budget.py`, `git diff --check`: clean.

Not run: the workspace suite or other targets, any fit, latent search, bootstrap, smoke,
characterization, calibration, benchmark, mutation or Phase 2 work. Passing shows what §1 says,
no more: no properness, H-c, reputational incentive, confidentiality, anonymity, true DTF or
neutrality, and S2 is not started.

## 10. The absent baseline: regression and correction (approved on `d7a4449`)

On the owner's assignment after Astra's review of `d2be7da`, in test code only; **approved by
Astra on `d7a4449`** (design review). The approval on `3559396` stays valid within the check's
perimeter; it does not cover this behaviour, which §§4–5's fixtures never reach (K3 leaves every
scored verdict another first panelist of positive weight).

**The defect** (L, recorded in `17` §14.4 on `d2be7da`; confirmed by Astra by reading the code;
E, reproduced below). `study::contribution` returned `None` for a verdict whose baseline
`first_panel_baselines` left `None`, and `study::cohort` summed the members' terms through
`filter_map` without treating that `None` as blocking: a cohort otherwise in cases 3–5 got a
final value, one with members in case 2 got D3's bound, the undefined term read as 0.

**The regression**, written before the correction and run on `d2be7da`'s check code: the main
scenario with `S0` weighing `w` 1 and every other nym 0 (`Main::build_with`; nothing else
changes). (w, j1), (v, j2) and (w, j5) get no baseline; (u, j1) and (v, j1) get ½, (u, j5) ¼,
`w`'s reports. Expected values worked by hand, checked with fractions.

| Cohort | Members | `N`, `O`, `V` | Known sum | Before | After |
|---|---|---|---|---|---|
| `K_w`, after cut 5 | j1 `R`, j5 `A`, both without a baseline | 2, 2, 2 | 0 | `Final(0)` | `Undefined(NoBaseline)`; j1, j5 listed |
| `K_v`, at `Ω` and after | j1 `R`: 3/16; j2 `A` without a baseline; j3 `I`: 0 | 3, 3, 2 | 3/16 | `Final(1/16)`, j2 dropped from the sum | `Undefined(NoBaseline)`; j2 listed |
| `K_w`, at `Ω` | j1 `R` without a baseline; j5 pending | 2, 1, 1 | 0 | `Bound`, sum in [−1, 1]: j1 absorbed into j5's interval | `Undefined(NoBaseline)`; j1 listed |
| `K_u`, at `Ω`; after cut 5 | j1 `R`: −5/16; j3 `I`: 0; j5 pending, then `A`: ½; j11 case 6 | 4, 2, 1; 4, 3, 2 | −5/16; 3/16 | `Unresolved(6)` | unchanged |

**The correction.** `contribution` keeps `g(I) = 0` without a baseline and no term for cases 2,
6 and 7, and names a verdict without a baseline `Undefined::NoBaseline`; `study::member` builds a
member's term (§11 calls it too) and records that cause in `Member::undefined`. `study::cohort`
keeps `Unresolved` for a member in case 6 or 7 (6c, 7a); otherwise a member with an undefined term
gives `Value::Undefined`, never a final value nor a bound; `Cohort::undefined` lists those members
whatever else blocks the value; `known` stays the sum of the defined terms. Unchanged:
assignments, cohort membership, `|K|` as the denominator, the cases, `N`, `O` and `V` by their
definitions (a verdict without a baseline is an observed, conclusive outcome), the defined terms.
No imputation, fallback, penalty, removal or reputational rule. §§4–7's expected values are all
unchanged.

## 11. A1's adaptive verification (`17` §14.6, approved on `d7a4449`)

On the owner's assignment, in test code only (`crates/protocol/tests/s2/adaptive.rs`, tests in
`s2_synthetic_check.rs`); described here before its execution; **approved by Astra on
`d7a4449`** (design review). Abstract constructions, not models of the protocol; no fit, no
behavioural model, no attack on the protocol, no frequency.

**The path.** Per state: `u`'s forecast and each other first panelist's report with its frozen
weight → `first_panel_baselines` → `study::member`, through `difference_score` → `study::cohort`,
the prefixed cohort's final value: the functions §5's snapshot uses (§10). A construction's
expected value is `Σ P(state) × value`, in `f64`. Since §12 that path is `cohort_scores::score`,
which composes the same functions.

**Expected values**, derived apart from that path. Per strategy, D1 (`17` §8.1) with exact
rationals in the test:
`E[Ŝ_K] = (1/|K|) Σ_cells P(cell) Σ_j (t_j − c_j[(p − q_c)² + q_c(1 − q_c)])`, the term 0 where
`c_j = 0`, with `c`, `q_c` and `t = E[1{Y ≠ I}(b − o)² | cell]` read from the
construction's states, `b` the weighted mean of the other first panelists' reports (`16` §4.6). The
headline values below were enumerated beforehand with fractions in a scratch script. Tolerance
`1e−12` (K8): the path sums at most 36 states' terms in `[−1, 1]`, with probabilities in halves,
quarters and thirds, so its rounding stays near `1e−15`; the smallest strict loss below, 1/768,
exceeds the tolerance by nine orders of magnitude, so no comparison is decided by rounding.

### 11.1 Within the theorem's domain: (A)

Each construction: `u`'s prefixed cohort holds all its assignments, `|K| = N_u` fixed; design A
(`π = 1`, every item piloted); `F_u` one cell per value of `u`'s signals, observed before it
commits and the same for every strategy compared; the other first panelists' reports formed from
their own information, never from `u`'s report; frozen weights 1. H-a: the signals precede the
freeze (declared). H-b: `π = 1`, the observed outcome the construction's `Y` (declared). H-c: the
states' law, outcomes and other reports included, reads no report of `u`'s, by construction; C5 is
checked pathwise: each state's baseline is identical for every grid report of `u`. H-d: every
strategy reports on every assignment, every baseline has positive weight; checked: every state's
cohort value is `Final`.

**Strategies enumerated**: every map from (cell, item) to the grid — joint over `u`'s
assignments, adaptive to every signal, the constant maps (fixed forecasts) among them. Truthful:
`q_c` where `c > 0`, ½ where `c = 0` (`q_c` undefined). *Strict*: differing from the truthful
strategy at some (cell, item) with `P(cell) > 0` and `c > 0`; *indifferent*: differing only where
`c = 0`. Checked for every strategy: the path's value equals D1's within the tolerance; none
exceeds the truthful value; each strict one falls below it, each indifferent one equals it.

**P1 — two groups, a private and a public signal.** 24 states.

| Element | Law |
|---|---|
| latent | `Z` Bernoulli(½), a quality shared by j1 ∈ G1 and j2 ∈ G2 |
| `u`'s signals | private `s`, `P(s = Z \| Z) = ¾`; public `d` Bernoulli(½), independent of `Z`, `s`, `t` |
| conclusion | G1 always on `d = 0`, with probability ⅔ on `d = 1`; G2 always on `d = 0`, never on `d = 1`; independent of `Z`, `s`, `t` given `d` |
| verdicts | on conclusion, `A` if `Z = 1`, `R` if `Z = 0`, for both items |
| first panels | j1: `u`; `r`, ¾ if its private `t` is 1, ¼ if 0, `P(t = Z \| Z) = ¾`, independent of `s` given `Z`; `r′`, ½. j2: `u`; `r″`, ½ |
| `F_u` | `(s, d)`: four cells of probability ¼ |

| `(s, d)` | j1: `c`, `q_c`, `t` | j2: `c`, `q_c`, `t` |
|---|---|---|
| (0, 0) | 1, ¼, 13/64 | 1, ¼, ¼ |
| (0, 1) | ⅔, ¼, 13/96 | 0, —, 0 |
| (1, 0) | 1, ¾, 13/64 | 1, ¾, ¼ |
| (1, 1) | ⅔, ¾, 13/96 | 0, —, 0 |

Grid {¼, ½, ¾}: `3^8` = 6,561 strategies. Expected: truthful `E[Ŝ_K]` = 17/768; 6,552 strict
strategies below it, by at least 1/192 and at most 1/6; 8 indifferent, equal to it (only j2's
reports at `d = 1` changed). Named: ½ everywhere, −5/256; j1 reporting `P(A | F_u) = ½` rather
than `q_c = ¾` in cell (1, 1), 13/768 — a loss `¼ × ⅔ × (¼)² / 2 = 1/192`, `17` §10.2's
`c(p − q_c)²` over the cohort.

**P2 — one group, three items, another item's gate decision.** 36 states.

| Element | Law |
|---|---|
| latent | `Z` Bernoulli(½), G's quality |
| `u`'s signal | `D`, the gate decision on an item of G that `u` does not review, logged before `u` commits: a pass with probability ¾ if `Z = 1`, ¼ if `Z = 0` |
| conclusion | one fit: the three items all conclusive with probability ½, all `I` otherwise; independent of `Z` and `D` |
| verdicts | on conclusion, independent given `Z`: `A` with probability ¾ if `Z = 1`, ¼ if `Z = 0` |
| first panels | each item: `u`; `r1`, ⅝ on a pass and ⅜ on a rejection; `r2`, ½ |
| `F_u` | `D`: two cells of probability ½ |

Every item and cell: `c` = ½, `t` = 61/512; `q_c` = ⅝ on a pass, ⅜ on a rejection
(`P(A | F_u)` = 5/16, 3/16). Grid {⅜, ½, ⅝}: `3^6` = 729 strategies. Expected: truthful
1/512; 728 strict strategies below it, by at least 1/768 and at most 1/32; none indifferent. ½
everywhere: −3/512.

### 11.2 The channels outside the hypotheses: (B)

Each test reproduces the effect `17` §14.6 (B) names, under that construction's hypotheses, and
nothing stronger: one assignment's expected term, through the path above unless stated; no net
over a cohort.

| Construction (`17`) | Hypothesis violated | Supplied as synthetic input | Computed by the code | Reproduced | Not shown | Under A |
|---|---|---|---|---|---|---|
| §7.3, read for B-b (§7.5) | H-c, through load | `u`'s report on `k` decides `k`'s entry, the entry whether `k`'s group is piloted in `j`'s term, and then `j`'s group misses its floor: `Y_j = I`; otherwise `Y_j` has law (3/5, 2/5, 0); `j`'s other first panelist reports the conclusive outcome `j` would have (1 or 0); `u` reports `q_c` = 3/5 on `j` | the baseline, `j`'s term | `j`'s expected term: −6/25 truthful, 0 deviating | `k`'s term and any net over the cohort (§7.3's 19/25 against `δ²` is the ternary score's); a model of the protocol | outside design A: an entry deciding a group's piloting is the entry channel A removes (§8.5); the test concerns that construction alone. Load through the work actually done stays subordinate to H-c, not reproduced |
| §9.4 | C5: the baseline moves with `u`'s report | the effective baseline `b(p) = min(1, max(0, 2p − ½))`, as a replacement under 7b responding so; the outcome Bernoulli(½), conclusive, independent of `p` | the term, from the supplied `b(p)`; `first_panel_baselines` not called for it until §12, which supplies `b(p)` as the only other first report at weight 1, returned unchanged | 0 at `p = ½`, 3/16 at `p = ¾` | that a replacement responds so; any effect under `16` §4.6's composition, whose baseline over the same reports does not move with `p` (checked beside) | only through a replacement rule, none decided (T58 open); the contract keeps C5 under any |
| §11.3, timing | H-c, through the outcome's law | the simplified gate (`p ≤ ½` sends `j` to the band); T1's start after the moved `Φ_G` meeting a sample giving `A` with probability 2/5 instead of 3/5; always conclusive; one other first panelist reporting 3/5 | the baseline, the term | 0 at the truthful 3/5; 1/25 at 2/5, the report's own cost included | effects on other assignments; that `bridging_gate` or a T1 start behaves so | stays under A with T1 unless samples and resources are invariant in time (§11.2), a declared hypothesis |
| §11.3, disclosure | C5 and H-c | `Y = Z`, Bernoulli(½), on both paths; `p = ½`; the other first panelist reports ½ without the signal, `Z` with it; whether the signal arrives, decided by `u`'s deviation on another assignment, is not modelled | the baseline, the term | 0 without the signal, −¼ with it | a gain over the cohort: the deviation's other changed terms are not counted | stays under A, T1 or T2; §12.4's (ii) excludes the channel to the reports' values where their commitments close before the opening |
| §12.4, across groups | C5 and H-c | `Y_k` equiprobable; a signal, ± equiprobable, setting `P(Y_k = A)` to `½ ± δ`; the other first panelist's report equal to that posterior, the construction's hypothesis; `p = ½` | the baseline, the term | 0 without the signal; `−δ²` with it: −1/16 at `δ = ¼`, −¼ at `δ = ½` | as above; that reports equal the posterior | stays under A; (i)–(iv) each exclude one channel under their own hypotheses |

### 11.3 Evidence and limits

**Evidence** (E, Claude Code, 2026-10-08; the expected values C, with fractions, beforehand):

- the three regressions of §10, written first and run on `d2be7da`'s check code, only
  `Main::build_with` added to the fixture, with `cargo test -p protocol --test
  s2_synthetic_check`: 28 passed, 3 failed, each at its value
  assertion after its counts, members and known sum had passed — `K_w` after cut 5 read
  `Final(-0.0)`, `K_v` at `Ω` `Final(0.0625)` (1/16), `K_w` at `Ω` `Bound` with sum
  [−1, 1] and mean [−½, ½];
- after the correction, the same command: 32 passed (the 28 original tests with every expected
  value of §§4–7 unchanged, the three regressions, one test of the named cause); with §11's seven
  tests, 39 passed, 0 failed (1.04 s, a time of these tests only);
- two temporary probes, reverted: the path reading `A` as 0 fails every test of §11; D1's loss
  without its factor `c` fails P1 and P2;
- `cargo clippy -p protocol --test s2_synthetic_check -- -D warnings`, no warning; `rustfmt
  --edition 2021 --check crates/protocol/tests/s2_synthetic_check.rs` (its modules included),
  `python3 scripts/comment_budget.py`, `git diff --check`: clean.

Not run: the workspace suite or other targets, any fit, latent search, bootstrap, smoke,
characterization, calibration, benchmark, mutation or Phase 2 work.

**Limits.** A finite grid checks only the strategies enumerated, on two abstract constructions;
the theorem rests on its proof (`16` §4.3, `17` §§7.5 and 8.1), not on these tests. H-a and H-b
are declared, H-c holds by construction and is shown for no protocol, H-d holds on the
constructions; nothing here shows availability, confidentiality, anonymity, reputational
incentives, the gate's or the bridging's behaviour, or neutrality. (B) reproduces one effect per
construction, a term, never a gain over a cohort, and no frequency; §9.4's dependence is a
supplied input, not a property of `panel_scores`. The values are `f64` within `1e−12` of exact
rationals. §10's correction covers the check's aggregation in test code only; the operational
policy for an absent baseline stays open (`17` §14.4).

## 12. The check through B-b's scorer (approved on `6b9f42a`)

On the owner's assignment after Astra's review of `23283f2`, which accepted B-b's scorer as the next
intervention; Claude Code's. Partly reviewed by Astra on `7d1b19e` and `0ca81e9`, neither approving
the scorer as a whole — first for a numeric defect, then for a scale common to terms and pending
radii that could cancel a term from `known`; the corrections are at the end of this section.
**Approved on `6b9f42a`**: the scorer as an isolated component and the check through it. The scorer
is `protocol::cohort_scores` (`17` §14.9), a library component with no production caller; the check
now computes its cases, terms, counts and cohorts through it. No record, fixture or expected value
of §§4–11 changes.

**Adapters**, in test code. `study.rs`: `Snapshot::of` builds one scorer `Item` per item from the
log's prefix — the first panel's reveals completed before K5's close as `Revealed`, any other as
`Missing` (every snapshot follows the closes, so `Open` never arises), `S0`'s weights, the extra
round's accepted reveals, `frozen` at `Φ_j`, `Selected` at `π = 1` with the validated terminal
outcome or `None` — and `S0`'s cohorts by item index, then reads each assignment's case and each
cohort from `score`. `study::cohort` maps a scored cohort to §10's approved representation:
`Unresolved` with the first member in case 6 or 7, otherwise `Undefined(NoBaseline)`, and
`undefined` listing the members without a baseline; case 1 and an undrawn item do not arise under
A and would fail the check. `study.rs`'s own contribution and aggregation are removed: it calls
neither `first_panel_baselines` nor `difference_score`. `adaptive.rs`: each state's seats become
items, `u` first at weight 1, scored as `u`'s prefixed cohort over all of them; a channel's term is
that cohort's single member; `baseline` reads the scorer's; §9.4's `b(p)` is supplied as the only
other first report. §10's named-cause test builds its four members as scorer items — a verdict and
an `I` with no weight left, a missing report before the freeze, a pending member — rather than
through the removed `study::member`; its expected values are unchanged.

**Kept.** Every expected value of §§4–7, §10's regressions and §11's constructions; their oracles,
the hand values and D1 in exact rationals, computed apart from the scorer.

**Added**, in `crates/protocol/tests/cohort_scores.rs`, the scorer's checks listed in `17` §14.9:
`π < 1` with positive and negative terms; a non-selection apart from `I` and pending; `N_u`
against `|K|`; the interval weighted by `Σ 1/π_j`; refused numbers and combinations; the supported
compositions; four coexisting causes; a finite enumeration with a draw shared by two items, its
effective probabilities those the construction declares, against D1's reference on 27 report
vectors. It checks arithmetic under the construction's hypotheses, not C in the protocol, and
reopens nothing of §11.

**Evidence** (E, Claude Code, 2026-10-08; the new expected values C, with fractions, beforehand):

- the new tests could not compile before the scorer existed: the component's absence, not a
  regression; no defect of the existing code was found, so none was regressed;
- `cargo test -p protocol --test cohort_scores`: 10 passed, 0 failed. As first written, two
  expectations were wrong (a panelist's term when `u` alone carries weight, and the extra
  reviewer's baseline, which that weight defines) and one construction put a missing first report
  on a frozen item, which the contract refuses; each was recomputed by hand and with fractions,
  the code unchanged;
- `cargo test -p protocol --test s2_synthetic_check`: 39 passed, 0 failed, through the scorer,
  every expected value unchanged; `cargo test -p protocol --test panel_scores`: 7 passed;
- three temporary probes, reverted: the pending radius counted rather than weighted fails two of
  the scorer's tests; a term not divided by `π` fails four, the enumeration among them; an absent
  baseline read as the panelist's own report fails two of the scorer's tests and, through it, the
  check's four tests of §10;
- `cargo clippy -p protocol --lib --test cohort_scores --test s2_synthetic_check --test
  panel_scores -- -D warnings`, no warning; `rustfmt --edition 2021 --check` on the new and
  changed files, `python3 scripts/comment_budget.py`, `git diff --check`: clean.

Not run: the workspace suite or other targets, any fit, latent search, bootstrap, smoke,
characterization, calibration, benchmark, mutation campaign or Phase 2 work.

**Limits.** Passing shows that the scorer reproduces the check's approved values and the
constructions' expectations, and computes the weighting and the causes as `17` §14.9 states. It
shows no H-b, H-c or H-e in the protocol, no availability, confidentiality, reputational incentive
or neutrality; the scorer's inputs are supplied and typed, with no record format or authentication
behind them, and it has no production caller.

**The numeric correction** (after Astra's review of `7d1b19e`; partly reviewed on `0ca81e9`, its
corrections of cases A–E approved; the contract in `17` §14.9, *Numerics*). `panel_scores` scales
the weights it averages by the largest one's power of two; the scorer names a term or a pending
member's `1/π_j` beyond `f64`'s range `OutOfRange`, as a cause of no value beside the others, and
computes its sums in power-of-two scales — on `0ca81e9` one common to terms and radii, since the
correction below `known`'s own — `known` and the bound's `sum` `OutOfRange` where they leave the
range, the means never.

- *Adapters*: `study::cohort` maps `NoTerm::Undefined` to the check's `Undefined` and reads
  `known` and the bound's `sum` with `expect`. Under A, `π_j = 1`, every term lies in `[−1, 1]`
  and every sum within `|K|`, so no `OutOfRange` arises; one would fail the check, as a case 1 or
  an undrawn item would. `adaptive.rs` is unchanged.
- *Kept*: every expected value of §§4–12 and their oracles. Beyond passing, every output of
  `score`, `first_panel_baselines` and `extra_round_baseline` over the check's 39 tests and the
  scorer's and `panel_scores`' earlier tests — 988 364 calls, recorded by a temporary
  instrumentation, removed — is identical bit for bit before and after the correction, the new
  types' `Ok` normalized. That comparison is evidence on these fixtures only; the general identity
  `0ca81e9` stated beside it is withdrawn (`17` §14.9, narrowed there).
- *Evidence* (E, Claude Code, 2026-10-08): nine regressions failing on `7d1b19e`'s code and
  passing after the correction, two checks added after it, listed in `17` §14.9;
  `cargo test -p protocol --test cohort_scores`: 19 passed; `--test panel_scores`: 9 passed;
  `--test s2_synthetic_check`: 39 passed, P1, P2, §10's regressions and §11's constructions
  among them, every expected value unchanged; `cargo clippy -p protocol --lib --test
  cohort_scores --test s2_synthetic_check --test panel_scores -- -D warnings`, no warning;
  `rustfmt --edition 2021 --check` on the changed files, `python3 scripts/comment_budget.py`,
  `git diff --check`: clean. `panel_scores` has no caller but the scorer and its own tests; the
  lifecycle test of `panel_scores.rs` that feeds it real bridging weights passes.
- *Not run*: as above; nor any other target, the workspace's suite included.
- *Limits*: the bounds are properties of the computation, not of the protocol; `OutOfRange` is a
  limit of `f64`, not a rule; an exact value may exist where it is named (`17` §14.9). Nothing
  above shows H-b, H-c or H-e, availability, confidentiality, reputational incentives or
  neutrality.

**The correction of `known`'s scale** (after Astra's review of `0ca81e9`; **approved on `6b9f42a`**;
`17` §14.9, *Numerics*). The scorer sums the defined terms in their own power-of-two scale and the
pending members' `1/π_j` in theirs; `known` and the final value come from the terms' sum alone, the
bound moves both sums to the coarser scale. No threshold, clipping, imputation or domain
restriction; formula, denominator, counts, members' order, causes and `OutOfRange` unchanged.

- *Adapters*: unchanged. Under A every `π_j` and radius is 1 and every term lies in `[−1, 1]`:
  terms are scaled up or not at all, radii not at all, so none is rounded by its scaling.
- *Kept*: every expected value of §§4–12 and of the scorer's and `panel_scores`' tests. Beyond
  passing, a temporary trace, removed, found the same output for each of the 186 509 `score`
  calls of the scorer's 19 earlier tests and the check's 39 before and after (`17` §14.9).
- *Evidence* (E, Claude Code, 2026-10-09): the regression of `17` §14.9 failing on `0ca81e9`'s code,
  `known = Ok(-0.0)` against `Ok(−2^−100)`, and passing after; `cargo test -p protocol --test
  cohort_scores`: 20 passed; `--test panel_scores`: 9 passed; `--test s2_synthetic_check`: 39
  passed, every expected value unchanged; `cargo clippy -p protocol --lib --test cohort_scores
  --test s2_synthetic_check --test panel_scores -- -D warnings`, no warning; `rustfmt --edition
  2021 --check` on the changed files, `python3 scripts/comment_budget.py`, `git diff --check`:
  clean.
- *Not run*: as above.
- *Limits*: as above; `known` is exact for one term, otherwise within `17` §14.9's absolute bound,
  cancellation included; the interval's centre is `known` only within its bounds.
