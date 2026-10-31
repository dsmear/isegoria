# Instructions for Claude Code

These instructions apply to repository work, regardless of the agent used. Read
`00-overview.md`, then `05-question-lifecycle.md`'s answer-review contract. Phase 1 is
closed within its offline perimeter. This branch develops the Phase 2 runtime under
the owner's assignments; no production readiness is implied.

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
   score (`E`) live on **different, unlinkable pseudonyms** of the same person. `C`
   governs the rate limit on proposals; only the legacy path uses `E` as a review weight. Do not merge
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

Phase 2 targets an independently operated experimental P2P testnet (D55).
Follow `10` §2: T88's contract is approved (`01` D57) and T79 must be specified first;
T79's implementation remains the final runtime integration, followed by the
independent-operator trial. Single-operator runs do not close the phase. Shared event
formats, results, cuts and ledger must change together with their users. Do not restore
legacy pilot gates or add automatic rounds through the integration. D54 is the network
decision previously called D44 here.

Use `phase1-handoff.md` to resume. Task status belongs in `10-roadmap.md`, review
findings and evidence in `15-phase1-review.md`, parameter procedures in
`14-parameter-register.md`, and measurement provenance in `13-characterization.md`.
Do not reopen or extend a closure without a concrete finding. Historical measurements
retain their model and version scope.

Inside a phase, fix defects before adding features. A code-defect correction starts
with a test that fails on the current code. Documentation-only changes are checked
against their sources and links; they do not require an artificial failing Rust test.
Long characterization runs belong on the owner's machine and require explicit
authorization; use targeted tests and the smoke grid during development. The design
and implementation handoff is in `15-phase1-review.md`.

## Phase 1 direction: D47, D48 and the answer review (D49–D52, 2026-10-21)

The current contract is D49–D53 in `01-decisions.md`, specified once in
`05-question-lifecycle.md` (The answer review), with the author index in `02` §C.1.
T87 and Phase 1 are closed only in the offline path's perimeter (`15`, closure record).

- Preserve the approved contract. Do not add reviewer weights or reputation updates,
  selection by competence, replacements, automatic closures, deadlines, retries, key
  correction, source requirements or earlier filters without an owner decision.
- Reviewers may use any resource; do not detect, infer or assess external-tool use,
  or collect their sources. The author is not required to supply a source either.
- Never turn consensus into a claim of truth or reviewer reliability. Divided answers
  are not proved ambiguity; insufficient participation is not rejection. The author's
  index records review outcomes, not measured factual quality or competence.
- Deposits are deduplicated per authenticated author, not globally (AT-ANS-14).
  The key remains in clear in state and log; the caller is trusted not to select a
  beacon for a desired panel. Do not claim stronger confidentiality or randomness.
- Parameters are provisional and uncalibrated; the decoder has not been fuzzed.
  No truth, whole-quiz fairness or D46 assessment objective is certified. A2 remains
  unresolved in its own scope and is not required for this path (D52).
- The legacy code keeps its contracts. Its proofs, tests and closures do not certify
  the answer review. `21-d47-inventory.md` records retained dependencies; the archive
  is historical, not an active specification or work plan.

## Implementation guidance

Rust is the reference implementation; Python prototypes and numerical oracles have
their own scoped contracts. Prefer mature audited cryptographic primitives. Bespoke
composition requires a stated need, small scope, known-answer/property tests and an
explicit label as bespoke. See `ARCHITECTURE.md` for the code map and test strategy.

## What NOT to do

- Do not introduce a permissionless blockchain or economic stake. The existing
  consortium agrees on event order through signed cuts; P2P replication alone does
  not resolve conflicting actions. See `01` and `04`.
- Do not put voting patterns or questions under review on a public register in the
  clear: they are a vector for statistical deanonymization.
- Do not implement a Kleros-style Schelling point (rewarding those who vote with the
  majority). It is the opposite of what is needed: see `01` D6 and D48.
- Do not treat an LLM's output as a source of truth in the scoring engine. The
  engine is deterministic and mathematical.

## Code style and docs

- **Less is more.** Keep one authoritative description per subject and link to it.
  Remove obsolete explanations and duplicated status summaries instead of appending
  another qualification. Preserve unique decision records, proofs and measurements
  with their historical scope. Write repository documentation in English.

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
