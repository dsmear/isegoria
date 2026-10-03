# Phase 1 correction work — handoff note

**Working note, not normative.** It hands work over between Claude Code sessions and is meant to
be enough to resume in a new chat. The specification is in `docs/01`–`docs/14`; the review
register is [`15-phase1-review.md`](15-phase1-review.md). Where this note and those disagree,
they win.

## 1. Where things stand

- Repository `dsmear/isegoria`, owner's working directory `…/rust/isegoria`, branch
  `docs/phase1-review-alignment`. Baseline requested for R1: `0519626`. HEAD
  `8fa07dd66698ecf1c23141b942bf8ed7ad57242f`, its direct child, publishes R1.
- The commit that carries this note, on top of `8fa07dd`, holds the documentation follow-up of
  Astra's review (§7), integrated by Claude Code in the owner's checkout; not pushed by the
  session.
- Owner's modifications, never to restore, commit or clean: `.gitignore` (it ignores
  `/characterization*/`, `/smoke*/` and `.gpt/`, where Astra's patch sits) and the ignored
  directories. The follow-up touches no `.gitignore`.
- R1 does not close A2 or Phase 1.

## 2. Roles and working agreement

- **Claude Code** implements and produces evidence (tests, calculations, runs). **Astra** decides
  the design and reviews each commit, reading diffs and recalculating; it does not re-run Rust.
  **The owner** wants to intervene as little as possible: commits and pushes when asked (the
  Claude Code session has no GitHub credentials), decides only genuine product trade-offs.
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

- **A2 (high): diagnosis approved, R1 implemented, Astra's final review pending, A2 open.**
  Approved on `0519626`: the contested facts' cost does not certify the whole test's DTF; the
  conditional proposition of `19` §3 (partition of the items, common measure, representation of
  the groups; per-pair maximum for mixtures constant in ability, pointwise-maximum envelope in
  general; no label matching needed). Not approved: R3 (a test-level fit) as a solution, a batch
  per form, any new calibration, selection or group policy. R1 rectifies the claims only.
- **A1 (critical for the incentive claim): open.** Acquired: the diagnosis (the exploration draw is
  known before reports); the beacon-manipulability and count analysis (`16` §§6–7); the band
  baselines, implemented (`protocol::panel_scores`, `e8fdbe7`, no production caller yet); the
  properness proof as a result conditional on C6 and its hypotheses (`16` §4.3); `17`'s argument and
  comparison of batch designs as a conditional analysis (`37addca`). Open decisions: the batch
  contract (`17`: universal pilot, group audit, prefixed groups activated by entry or draw; no
  batching policy approved); missing outcomes (C2); the denominator and no-show rule (C4, T58); the
  randomness guarantee (a behavioral model of the beacon's members, or a source change); deferred
  draw against audit; whether A1 needs the incentives of the reputation actually used (`k_u`,
  shrinkage, cap, CUSUM).
- **B1–B3:** identification of the floors and of the histogram through the responses, weak
  information, the moment penalty's choice and effects, BIC under misspecification and on
  penalized mixture fits, selection after penalized fitting.
- **Costs and calibration:** sample floors are not power guarantees (B4–B5, C3); respondent burden
  and runtime need budgets (D2, D4); the contested selection's enumeration cost (D3); T83's
  operating point (B7, D1, D6, E). Also open: A8, A9, A10, the register's C5 and C6, A4 (b)–(d),
  C2, C4, the full runtime.

## 5. Next segment (not started)

A short decision synthesis that links: (1) what outcome a reviewer forecasts and how it is
produced — A1; (2) the population and contrasts the DTF must protect — A2; (3) the dependence on
the model and on selection — B1–B3; (4) respondents, answers including anchors, fits and time. It
must compare a reusable common calibration with a calibration per form without presuming either
valid or inevitable. Constraints: anonymity (no personal or group attributes enter), the
recovery of contested facts; the test's neutrality is not to be declared certified. No full new
roadmap yet.

## 6. Essential reading to resume

`CLAUDE.md` and `docs/CLAUDE.md`; `docs/15` (rows A1, A2, B1–B3, D2–D4 and the correction
records); `docs/16` (A1's theorem, conditions C1–C6, beacon model) and `docs/17` (A1's batches and
missing outcomes); `docs/18` (A3); `docs/19` (A2); `docs/02` §B.3, §B.7; `docs/01` D33–D38, D43;
code: `crates/scoring/src/{latent.rs,dtf.rs}`,
`crates/protocol/src/{contested.rs,lifecycle.rs,orchestrator.rs,exploration.rs}`.

## 7. This intervention

- **R1:** `D(T)` described as the contested facts' admission cost, not a certified bound on the
  test's DTF — `01` D38 (dated clarification), `08` DIF-011, its status row, AT-PRO-08 and the
  `Contested` row, `02` §B.7 (cross-fit paragraph, "cost"), `10` T55 (dated clarification),
  `ARCHITECTURE.md`, comments in `protocol/src/contested.rs` and `scoring/src/dtf.rs` (`DTF_MAX`),
  `paper/sections/065-revisions.tex` (the tolerance an aim, not guaranteed). No formula, API,
  threshold, serialization, selection, golden or historical result changed.
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
- **Documentation follow-up against `8fa07dd` (Astra's patch, integrated and committed):** `01`
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
- Never run in this work: full characterization, smoke, mutation or calibration campaigns.

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
  seed campaign or mutation runs. Astra's final review of R1 remains pending.
- **Claude Code, integration of the follow-up:** HEAD `8fa07dd` and a working tree with only
  `.gitignore` modified checked first; `git apply --check` and `git apply --whitespace=error-all`
  of the patch on the owner's checkout (clean); the handoff's environment lines adapted to this
  checkout; documentation checks: `python3 scripts/comment_budget.py`, relative links of the
  edited docs, a search of current contracts for remaining whole-test claims. No Rust, fit, script
  or campaign re-run (documentation only).
