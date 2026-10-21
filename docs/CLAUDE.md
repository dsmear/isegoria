# Instructions for Claude Code

This repository contains the specification of a distributed network for producing
and validating quiz questions. You are here to **implement it**. Read
`00-overview.md` first, then `02-scoring-engine.md` (the central piece), then the
rest as needed.

## Invariants never to violate

These constraints are non-negotiable: breaking one destroys the property that makes
the system useful. Before every implementation choice, check you are not violating
them. Only a recorded owner decision changes them: `01` D48 rectified #2. Invariants 2 and 8
also name contracts of the existing mechanism; those bind the existing code in its scope, and
D52 states they are not conditions of the answer review (below).

1. **Anonymity is the base.** No personal, demographic, or affiliation data ever
   enters the system. Nodes are pseudonymous IDs and nothing else. Do not add
   "optional" fields that collect people's attributes, not even to improve DIF: the
   bias analysis runs on latent axes, not on declared labels (see `02`, §DIF).

2. **Consensus is an operational criterion, never a guarantee of truth.** Until D47 this
   invariant banned deciding quality by voting in any mechanism; `01` D48 rectifies that ban.
   In the answer review, the first implementation of D47's mechanism, the consensus of the
   drawn reviewers' answers, each with equal weight, is the review's operational criterion,
   with the quorum, thresholds and outcomes of `01` D49; do not change them in code. Never
   present a consensus as showing that the key or an answer is true, or that the reviewers are
   reliable, and never reward agreement with the majority (`01` D6, D48). *Existing
   mechanism:* acceptance goes through bridging (opinion filter) + empirical validation
   (evidence filter), with no vote count; keep that contract in the existing code. Neither
   filter is a condition of acceptance in the answer review (`01` D52).

3. **No money as stake.** No token, no economic stake, no monetary bond to propose
   or vote. The cost of proposing is reputation and rate limits. A money-based
   mechanism reintroduces wealth-based access, which is exactly the problem to avoid.

4. **Two separate scores, never combined.** The author score (`C`) and the evaluator
   score (`E`) live on **different, unlinkable pseudonyms** of the same person. `E`
   weights the review vote; `C` governs the rate limit on proposals. Do not merge
   them into a single weight and do not have them share the same ID. In the answer review
   every reviewer weighs the same and ordinary reviews do not update `E` (`01` D48, D49);
   `C` follows D50 and governs D51's quotas there; the separation stands.

5. **One role, one deterministic pseudonym, not rotatable.** Role pseudonyms derive
   deterministically from the credential (`H(secret, context)`). A person can derive
   exactly one author pseudonym, one evaluator pseudonym, one respondent pseudonym.
   Do not allow generating fresh pseudonyms for the same role: it would wipe out
   negative reputation (whitewashing).

6. **The state authenticates, it does not issue.** Whoever verifies identity
   (CIE/SPID) is distinct from whoever issues the anonymous credential (the
   threshold committee). Do not collapse the two roles: the separation is what stops
   the state from linking person and activity.

7. **Score computation is deterministic and reproducible.** The scoring engine,
   given the same input (ratings + answers), produces the same output bit-for-bit.
   No unseeded sources of non-determinism (dictionary iteration order, uncontrolled
   floating point, the platform's libm — transcendental functions go through
   `scoring::fmath` — timestamps in the computation). Reproducibility is the defense
   that unmasks a dishonest signer.

8. **The existing evidence filter validates in batches, never a single question.**
   Level B, as implemented, validates groups of questions with shared anchors; keep it
   so. Historical simulations show weak detection of an isolated biased item; they do
   not prove universal non-identifiability (`15-phase1-review.md`, A8). The answer review
   uses no empirical validation as a condition of acceptance (`01` D52): this contract is
   not imposed on it.

## Current priorities

The six crates exist. Follow `10-roadmap.md`: **mathematics**, then **P2P network**,
then **the rest**. Task status belongs there; calibration procedures belong in
`14-parameter-register.md`; measurement provenance belongs in `13-characterization.md`.

For Phase 1 corrections, read `15-phase1-review.md` before implementing. It records
the A–E review's open guarantees, code-contract findings and completion criteria.
Verify each finding; do not silently change the specification to approve a code defect.
T81 remains complete for the ten files listed in its run 22. Pre-D43 DIF measurements
remain historical evidence, not current-model validation.

Inside a phase, fix defects before adding features. A code-defect correction starts
with a test that fails on the current code. Documentation-only changes are checked
against their sources and links; they do not require an artificial failing Rust test.
Long characterization runs belong on the owner's machine and require explicit
authorization; use targeted tests and the smoke grid during development. The design
and implementation handoff is in `15-phase1-review.md`.

## Phase 1 direction: D47, D48 and the answer review (D49–D52, 2026-10-21)

The owner changed Phase 1's direction before its implementation (`01-decisions.md` D47): the
author states the key, reviewers answer the item with any resource, and forecasting the pilot
outcome is no longer the reviewers' main contribution. D48 then settled its first implementation:
reviewers are drawn by lot from the reviewer pool, with no new selection by specialist
competence; every reviewer weighs the same; ordinary reviews do not update the reviewers'
reputation, and the evaluation of their reliability over time that D47 asked for is deferred;
the consensus of the answers is the review's operational criterion; a reviewer may answer that
none of the alternatives is correct, which stays distinct from choosing another alternative; a
question is never corrected retroactively (its author submits a new one), and its key is never
replaced automatically. D45 and D46 stay the references for conditions and objectives.

D49–D52 are the owner's approved contract for the *answer review*, the offline path that
implements D48: 12 distinct reviewers drawn at random from the pool; an answer is an
alternative, "none is correct" or "don't know"; at least 9 usable answers (alternatives and
"none"), abstentions and absences outside the count; an explicit closure; accepted when
`4C ≥ 3V`, discarded when `4(V − C) ≥ 3V`, divided answers otherwise; only accepted questions
usable (D49). The author's reputation keeps `02` §C.1's index and aging with adapted inputs —
accepted 1 at weight 1, discarded 0 at weight 1, divided 0 at weight 1/4, insufficient
participation nothing (D50) — and sets a quota of 1, 2 or 3 questions per epoch (D51). Bridging,
the pilot, IRT, DIF and the source check are not conditions of acceptance there; A2's
whole-test guarantee is not required for it, and A2 is not resolved (D52).

- Keep three things apart: the approved contract (D49–D52), what is implemented (`10` T87:
  `protocol::answer*`, `scoring::reputation::author_score_weighted`, specified in `05` §The
  answer review), and what has been verified (the targeted tests in `tests/answer_review.rs` and
  the modules' unit tests; Astra's review pending). The existing code still implements the
  previous design; the answer review adds a path beside it and changes none of it.
- The parameters 12, 9, 3/4, 1/4 and the quota bands are provisional and uncalibrated (D52,
  `14`): keep them explicit in the configuration and in the results; do not tune them.
- Do not add what the contract excludes: reviewer weights, a reputation update or penalty for
  reviewers (an abstention or absence included), a selection by competence, a replacement in the
  round, an automatic closure, deadline or retry, a new round, an automatic correction of the
  key, a source requirement or check, or any previous filter as a condition of acceptance. Where
  the contract is silent on a product choice, stop and report it instead of choosing a policy.
- No answer to a request may depend on another participant's hidden data: deposits are
  deduplicated per author, never globally, since a global check revealed keys (`05`, AT-ANS-14).
  The offline state and log hold keys in clear, and the draw assumes its caller does not select
  the beacon value: do not claim confidentiality or grind-resistance the offline path lacks.
- Consensus is not truth, and agreement with the author's key or with the majority is not
  reliability; divided answers are not demonstrated ambiguity, and insufficient participation
  is not a discard. The exclusion of a Schelling point below stays. D48 records the risks it
  accepts (shared errors, different competences under equal weights, superficial reviews,
  coordination); the draw eliminates none of them, so do not describe the design as removing
  them. Accepted questions establish no fairness of the whole quiz (D52).
- The author's reputation stays separate (#4). The existing path's author score and its C5–C6
  closures are unchanged and are not extended to D50's inputs.
- Reviewers may use the Internet, documents and AI; they do not state or record the sources they
  consult, and the system does not detect, infer or assess their use of outside tools. The
  answer review asks the author for no source; the existing deposit's mandatory source and the
  contested-fact path's source check (`02` §B.5) stay as they are in the existing path.
- `21-d47-inventory.md` lists what the previous design's code is and what stays in place (§2,
  §4).
- `archive/phase1-forecast-pilot/` is historical: do not extend it or cite it as the active
  specification. Its closures keep their perimeters; A2 is not resolved by its archiving.
  Code comments citing `docs/16`, `docs/17`, `docs/19` or `docs/20` resolve through redirect
  pages; the archive's index has the path map. The working handoff is `phase1-handoff.md`.
- Phase 1 is not complete before Astra's final review of the implemented path (D52).

## Original build order (done)

The order the reference implementation was built in — each phase verifiable before the
next:

1. **`scoring/`** — the deterministic engine. Input: a ratings file (node ×
   question) and an answers file (respondent × question). Output: bridging scores,
   IRT parameters, DIF verdicts, reputations. It must run offline, with no network
   or identity. The simulations in `sim/` supplied the original fixtures and scoped
   differential oracles. Their retired BSS and proxy-DIF paths are historical;
   current behavior is specified in `02` and checked by the corresponding Rust tests.
2. **Tests against the corner cases** documented in `06` and in `sim/`: coordinated
   cartel, elite consensus, true-but-divisive question, survival rate. These are the
   engine's acceptance tests.
3. **`identity/`** — enrollment adapters, threshold credential, role nullifiers.
   See `03`.
4. **`network/`** — signed append-only logs, gossip, DHT, erasure coding, anchoring.
   See `04`.
5. **`protocol/`** — lifecycle orchestration: random reviewer assignment,
   commit-reveal, bridging gate, two-stage pilot, appeal channel, honeypot. See `05`.

## Suggested stack (not binding)

- **Scoring engine**: Rust is implemented; Python with NumPy/SciPy supplies research
  prototypes and scoped numerical oracles. Keep their contracts explicit.
- **Identity cryptography**: prefer mature, audited libraries — Semaphore for
  nullifiers, BBS+ schemes for credentials, a threshold OPRF for anchoring. Rolling
  our own is permitted when it genuinely serves the design — no suitable library
  exists, or we need a variant a library does not provide (e.g. a threshold scheme
  built from a vetted single-party one). When we do, keep it small, build it on
  audited primitives (not from first principles), pin it down with known-answer and
  property tests, and mark it clearly as bespoke. The bar is high, not absolute.
- **P2P layer**: consider libp2p (gossipsub + Kademlia DHT). CRDTs with Automerge or
  Yjs where converging state is needed.
- **Anchoring**: OpenTimestamps or equivalent to publish the Merkle root on a public
  chain.

## What NOT to do

- Do not introduce a permissionless blockchain with global consensus. Heavy
  consensus is not needed (writes almost never conflict) and reintroduces cost,
  latency, and public data. See `01` and `04`.
- Do not put voting patterns or questions under review on a public register in the
  clear: they are a vector for statistical deanonymization.
- Do not implement a Kleros-style Schelling point (rewarding those who vote with the
  majority). It is the opposite of what is needed: see `01` D6 and D48.
- Do not treat an LLM's output as a source of truth in the scoring engine. The
  engine is deterministic and mathematical.

## Code style and docs

- **Comments are budgeted, and the budget is enforced.** The design docs are the source of
  truth for the logic (`ARCHITECTURE.md` principle #4); code comments never restate them.
  `scripts/comment_budget.py` fails a file that exceeds the budget, and it runs in CI and,
  through the hook in `.claude/settings.json`, after every edit Claude Code makes:
  - a `//!` module header of at most 3 lines: what the module is, plus the `docs/` pointer;
  - `///` item docs of at most 3 lines: only the contract the signature does not express
    (shapes, units, preconditions, what an error means), plus the `docs/` pointer;
  - inline `//` runs of at most 2 lines, only for a mechanic the code cannot express;
  - no comment line over 100 characters (a longer line is not a shorter comment);
  - at most 10% comment lines per file (files with 20 comment lines or fewer are exempt);
  - one line per test: the property checked and its AT id.
  Never in a comment: rationale, history (task or decision narratives), measurements,
  rejected alternatives, or a narration of the next statement. A fact worth keeping that is
  not in `docs/` goes into `docs/` (`02`, `08`), not into a comment.
- **Keep the docs in step with the code.** When a change implements or alters behaviour
  the docs describe, update the relevant section in the same change (e.g. the `docs/08`
  §15 status matrix, the `docs/10` roadmap) rather than letting them drift.
