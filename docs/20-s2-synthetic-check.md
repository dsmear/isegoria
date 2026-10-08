# 20 — S2: the synthetic-event technical check

| | |
|---|---|
| **Status** | **Approved by Astra on `c76c035` as the specification of the synthetic technical check**, with four precisions, made in place (below). **Its implementation in test code, by Claude Code on `c76c035`, awaits Astra's review** (§9); the owner's assignment authorized it and its targeted tests, not that review. The approvals of [`19`](19-a2-dtf-composition-design.md) §10.6 and of [`17`](17-a1-pilot-batch-design.md) §§8–13 do not cover it. No data collection, adoption of T1 or T2, or start of S2. |
| **Baseline** | specification written on `5af22b9` and approved on `c76c035`; implementation on `c76c035`; code named by symbol. |
| **Scope** | `19` §10.6's next deliverable: what a technical check on synthetic events would execute, what its fixtures supply, what it must report and when it fails. References, within their recorded limits: design A, B-b and the reporting of cases 5–7 with 6c and 7a (`17` §§8–9), α (`17` §10), T1 the analytic reference and T2 the comparison (`17` §§11–13); B-b's study scores outside the reputation and the gate's decisions recorded with no effect on operational pools (Astra, decided on `cbcbc67`, confirmed on `5af22b9`). |
| **Evidence** | Specification (Claude Code, before `c76c035`): **L** targeted reads of the code named; **C** every expected value recalculated with exact fractions in a scratch script, not kept. Implementation (Claude Code): **E** the check's targeted tests, §9. Astra's review: below. |

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
| B-b | `panel_scores::first_panel_baselines` on frozen weights; `difference_score` | frozen weights per nym | the contribution per outcome (`17` §8.1), `N`, `O`, `V`, the cohort rule, consolidation and the bound (§8.3) | `SkillTrack`, `exploration::record_outcome`, `ResultRecord`, probation, shrinkage, cap, CUSUM; any per-nym score on the log |
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

## 9. The implementation (awaiting Astra's review)

Written by Claude Code on `c76c035`, on the owner's assignment; the review of `c76c035` approved
the specification above, not this implementation.

**Where.** Test code only: `crates/protocol/tests/s2_synthetic_check.rs`, one test per fixture
or common expectation, and its support `crates/protocol/tests/s2/` — `fixture.rs` (the builder
of §§4 and 6), `records.rs` (the study records and their reader), `replay.rs` (the log view),
`study.rs` (freezes, K4, cases, B-b, cohorts, T1 and T2 checks, the snapshot, the costs),
`order.rs` (the classifier at (R) and (E)). No production code, API, `NodeEvent` variant,
dependency, threshold, golden output or policy changes.

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
event, none logged; at (E), a `CutSignature` on its member's own feed counts when its cut is one
the replay applied — the feed entry's signature authenticates the object, and the member's
signature inside it is not checked again (`Consortium::signed_by` is crate-private).

**Negative cases**: a terminal `I` after one non-conclusion, missing an attempt's reference, or
resting on attempts without evidence; T1 starts before `Φ_G`, records before a start or without
one, a T2 record logged while held; and cut 6 after the main scenario — j15 assigned to `u`, G7's
start while `Φ_G7` is unreached — which K2's check reports (`K_u` stays four) and T1's refuses.

**Evidence** (E, Claude Code, 2026-10-08):

- `cargo test -p protocol --test s2_synthetic_check`: 23 passed, 0 failed; the test binary ran
  in 0.91 s, a time of these tests only, measuring no fit, availability or production;
- `cargo clippy -p protocol --test s2_synthetic_check -- -D warnings`: no warning;
- `rustfmt --edition 2021 --check crates/protocol/tests/s2_synthetic_check.rs` (its modules
  included), `python3 scripts/comment_budget.py`, `git diff --check`: clean;
- two temporary probes, reverted: K3's weight of r4 set to 1 fails F8; G6's stage-1 attempt
  logged in cut 2 fails F5 and the costs.

Not run: the workspace suite or other targets, any fit, smoke, characterization, calibration,
benchmark, mutation or Phase 2 work. Passing shows what §1 says, no more: no properness, H-c,
reputational incentive, anonymity, true DTF or neutrality, and S2 is not started.
