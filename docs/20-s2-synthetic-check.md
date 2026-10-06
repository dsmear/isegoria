# 20 — S2: the synthetic-event technical check

| | |
|---|---|
| **Status** | **Proposal awaiting Astra's review**, written by Claude Code on `5af22b9`. The approvals of [`19`](19-a2-dtf-composition-design.md) §10.6 and of [`17`](17-a1-pilot-batch-design.md) §§8–13 do not cover it. It specifies a check to be built later; it implements, runs and authorizes nothing — no implementation, run, data collection, adoption of T1 or T2, or start of S2. |
| **Baseline** | `docs/phase1-review-alignment` at `5af22b9`; code named by symbol. |
| **Scope** | `19` §10.6's next deliverable: what a technical check on synthetic events would execute, what its fixtures supply, what it must report and when it fails. References, within their recorded limits: design A, B-b and the reporting of cases 5–7 with 6c and 7a (`17` §§8–9), α (`17` §10), T1 the analytic reference and T2 the comparison (`17` §§11–13); B-b's study scores outside the reputation and the gate's decisions recorded with no effect on operational pools (Astra, decided on `cbcbc67`, confirmed on `5af22b9`). |
| **Evidence** | **L** targeted reads of the code named; **C** every expected value recalculated with exact fractions in a scratch script, not kept; no Rust, fit or run. |

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
| study records | — | statuses, outcomes, references to evidence, declared costs | group, attempt, terminal (`A`, `R`, or `I` with `17` §9.2's content), shortfall and refused-batch records; an encoding both `NodeEvent::decode` and `MemberObject::decode` refuse; their reader | a lifecycle state or `NodeEvent` variant for them: `Ledger::apply` refuses each as `NotAnEvent`, the state unchanged |
| freezes, starts | the applied order of `CutReport` | — | `Φ_j` from the applied steps — `Score` on `Pass` or `Reject`; `Resolve` unless appealable; `Appeal` or `AppealExpires` after an appealable decision (`16` §5) — and `Φ_G`; T1's start check; T2's hold | T58, or any rule letting a freeze pass |
| B-b | `panel_scores::first_panel_baselines` on frozen weights; `difference_score` | frozen weights per nym | the contribution per outcome (`17` §8.1), `N`, `O`, `V`, the cohort rule, consolidation and the bound (§8.3) | `SkillTrack`, `exploration::record_outcome`, `ResultRecord`, probation, shrinkage, cap, CUSUM; any per-nym score on the log |
| order evidence | `CutReport::{applied, refused}`; feeds' chains; cuts' marks; `MemberObject::CutSignature` on members' feeds | — | the classifier at (R) and (E), with `17` §13.2's four outcomes; `x_O` and the relevant commitments (§13.3) | (I), declared only; any dependence finding; a results root and inclusion proofs (`08` PRIV-004.1) |
| snapshot | — | `Ω` | the report of §7 | its publication |

`nullifier::prove` draws from the operating system's randomness, so entries' hashes and CIDs
change between builds: every expected result names positions, states and values, never a hash.

## 3. Conventions of the check

Synthetic, limited to this check, recorded in the fixtures; none is a parameter of the protocol
or of a future human study.

- **K1, `Ω`.** The prefix through cut 4 of the main scenario; cut 5 holds the late records.
  Declared in a study record `S0` in cut 0.
- **K2, cohorts.** For `x ∈ {u, v, w}`, every assignment of `x` in epoch 0: a rule on the
  assignment record alone (`17` §8.3); `|K| = N_x`.
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
- **K6, records.** One object per record: one attempt record per group and attempt, one terminal
  record per item, one record per shortfall and per refused batch.
- **K7, constructions.** T1 groups start after their `Φ_G`, their objects opened by being logged
  (`17` §13.3). The T2 group produces at the position its record fixes and holds every record
  until `Φ_G`.
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
| 2 | stage-1 attempt records of G1, G2, G3, G5 |
| 3 | stage-2 first-attempt records of G1, G2, G3; G4's shortfall record; G5's refused-batch record; terminal records j1, j2, j4, j9 |
| 4 | G2's second stage-2 attempt record; j3's terminal record. **`Ω` closes here** |
| 5 | G3's second stage-2 attempt record; terminal records j5, j6 |
| held | G6 (T2), produced after cut 1 and never logged while `Φ_G6` is unreached: both survive stage 1; stage 2 first attempt j11 non-conclusion, j12 `A`; second attempt j11 non-conclusion; terminal records j11 `I`, j12 `A` |

| Item, group | Panel (extra) | Reveals | Gate inputs → `bridging_gate` | Lifecycle at `Ω` | Study outcome at `Ω` (after cut 5) |
|---|---|---|---|---|---|
| j1, G1 (T1; term 4) | u v w r4–r7 | u ¾, v ¼, w ½; others ½ | 0.90, 0.05, 3 → `Pass` | `Pilot1` | `R`, first stage-2 attempt |
| j2, G1 | v r4–r9 | v ¾, r4 ¼, r9 1; others ½ | 0.50, 0.10, 2 → `Reject` | `Rejected(Defect)` | `A`, first stage-2 attempt |
| j3, G2 (T1; term 4) | u v r4–r8 | u ¼, v ¾; others ½ | as j1 → `Pass` | `Pilot1` | `I`: non-conclusions at both stage-2 attempts |
| j4, G2 | r4–r10 (r11–r14) | ½ | 0.80, 0.05, 2 → `SupplementaryReview`; `Resolve { Reject }` supplied | `Rejected(Borderline)` | `R`, first attempt; its second reading unused |
| j5, G3 (T1; term 6) | u w r4–r8 | u ¾, w ¼; others ½ | `Pass` | `Pilot1` | pending, one non-conclusion (`A`) |
| j6, G3 | r4–r10 | ½ | 0.50, 0.40, 2 → `AppealEligible`; `AppealExpires` | `Rejected(Polarized)` | pending, one non-conclusion (`I`, exhausted) |
| j7, j8, G4 (T1; term 3) | r4–r10 | ½ | `Pass` | `Pilot1` | pending: stage-1 shortfall, 120 admitted of 300 |
| j9, G5 (T1; term 3) | r4–r10 | ½ | `Pass` | `Pilot1` | `R` at stage 1 |
| j10, G5 | r4–r10 | ½ | `Pass` | `Pilot1` | pending: stage-2 batch {j10} refused |
| j11, G6 (T2; term 3) | u r4–r9 | `u` committed, no reveal; others ½ | `Score { Pass }` refused, `PartialEpoch` | `Revealing` | held `I` |
| j12, G6 | r4–r10 | ½ | `Pass` | `Pilot1` | held `A` |
| j13, G7 (T1; term 6) | r4–r10 | ½ | 0.50, 0.40, 2 → `AppealEligible`; no appeal by `Ω` | `AppealEligible` | pending: `Φ_G7` unreached, not started |
| j14, G7 | r4–r10 | ½ | `Pass` | `Pilot1` | pending: not started |

Gate-only rows, no lifecycle: (0.95, 0.00, coverage 0) → `SupplementaryReview` (below
`MIN_COVERAGE`); the four decisions above, each recomputed on its inputs.

**Fixtures.** Each row's expected results hold at `Ω` unless marked; the scenario's common
expectations follow the table.

| Fixture | Purpose; units | Events, prefix | Rule | Expected | Fails if |
|---|---|---|---|---|---|
| F1 | conclusive outcomes from completed reports, the gate apart; j1, j2 and their 14 assignments | cuts 0–3 | K4 | j1 `R`, j2 `A`, terminal records in cut 3; assignments cases 4 and 3; contributions in §5 | a gate decision read as the outcome; a pilot step logged; any item in `ActivePool`, `Contested`, `Explored` or `Measured` |
| F2 | verifiable `I`; j3 and its 7 assignments, j4 | G2's records, cuts 2–4 | K4, term 4 | j3 `I`, its record carrying `17` §9.2's six items (the rule by reference, the group, both attempts with statuses and evidence references, no earlier conclusion, the reason, its position); its assignments add 0, enter `O`, not `V`; j4 `R` from its first attempt | `I` after one non-conclusion, or without the attempts' evidence; a term depending on `p` or `b`; j3 in `V`; j4's second reading replacing its conclusion |
| F3 | an indeterminate attempt without exhaustion; j5, j6 and 14 assignments | cuts 2–3; second attempt in cut 5 | K4, term 6 | at `Ω` both pending (one executed attempt of two, term not reached), assignments case 2, no term; after cut 5, j5 `A`, j6 `I` | a non-conclusion read as `I`; a pending item read as 0 or dropped from `N` |
| F4 | a shortfall and a refused batch, apart from `I`; G4, G5 | `pilot::screen` on 120 synthetic ids returns `NotEnoughRespondents { have: 120, need: 300 }`, shortfall record in cut 3; G5's stage 1 in cut 2 (j9 `R`, j10 survives), then `admit_dif_batch(1, 3000, N_LATENT_MIN)` returns `BatchTooSmall { items: 1 }`, refused-batch record in cut 3 | K4: no attempt executed or consumed | j7, j8, j10 pending past their terms, two distinct records, neither `I`; j9 `R`; no search for either refusal | either refusal made `I`, counted as an attempt, merged with the other, or simulated rather than returned by the gate; any computation past a gate |
| F5 | a missing report, its item holding an outcome (T2); j11 (u's assignment and 6 others), j12 | `u`'s commitment in cut 1, no reveal by K5's close; `Score` refused; G6's records held | K4, K5, K7 | `u`'s assignment case 6, with its commitment and the held `I` recorded beside it: no term although `g(I) = 0` needs no `p`, no `O`; j11's other assignments case 7; j12's case 2 (record held); no G6 record logged | `u`'s assignment given 0, a term or an `O` from the item's `I`; the other panelists' baselines formed over the six reveals present (`17` §9.4, 7c); a held record logged or counted available |
| F6 | freeze not reached; j11's other panelists, G7 | j13's `Score { AppealEligible }`, no `Appeal` or `AppealExpires` by `Ω` | K7 | `Φ_j13`, `Φ_G7`, `Φ_G6` unreached; no attempt record of G7; j13's 7 assignments case 7, j14's 7 case 2 | a G7 attempt recorded; `Φ_j13` placed at `Score`; any of these assignments valued |
| F7 | a record after `Ω`; G3, cohorts | cut 5 | — | the snapshot at `Ω` unchanged; the state after cut 5 updated (§5) | the snapshot rewritten; a cut-5 record deleted, substituted or valued at `Ω` |
| F8 | cohorts fully evaluable or not; `K_u`, `K_v`, `K_w` | as above | K2 | §5 | a value while a member is in case 2, 6 or 7; an interval for `K_u`; D1 attributed to the consolidated cohorts alone |
| F9 | the gate on supplied inputs | the gate rows | K8 | the decisions above | a decision differing; a recorded `Score` differing from it; `Resolve`'s outcome reported as `supplementary_review`'s result |

**Common to the scenario.** `Ledger::apply` refuses every study object as `NotAnEvent`, and its
only other refusal is j11's `Score` (`PartialEpoch`); no item enters `ActivePool`, `Contested`,
`Explored` or `Measured`; every attempt, shortfall or refused-batch record of a T1 group follows
its `Φ_G`; nothing of
`SkillTrack`, a results event or a per-nym score reaches the log.

## 5. Scores, counts and baselines

Weights K3. A baseline is the weighted mean of the other first panelists (`16` §4.6); `g` as in
`17` §8.1; `π = 1` under A. The values are worked by hand from these definitions and checked with
fractions, not taken from the functions the check exercises.

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
| participations, stage 1 | 1,620: five admitted batches of 300, G4's 120 | 1,620 |
| participations, stage 2 | 21,000: six batches of 3,000 and G5's refused batch of 3,000 | 24,000 |
| pseudonyms counted by the gates | per batch, the same numbers; never summed into distinct pseudonyms | — |
| distinct persons | not reported: no count shows them | — |
| answers, items and 2 anchors per participation | 87,480, refused batches included | 99,480 |
| searches run, as declared | 5 stage-1 fits; 6 latent searches, 94 optimizer runs (17, 9, 25, 17, 17, 9) | 7 searches, 111 runs |
| refused before the fit | 2, no search | 2 |

The check verifies these sums and that each declared search's runs lie in {9, 17, 25} (`19`
§9.3); it measures none of them. A replay or record-handling time, if reported, is named as such
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
