# Phase 1 — review status and correction handoff

| | |
|---|---|
| **Baseline** | `master` at [`64a4c530781e65709b834ec6e0b3dc4f74cbcfd6`](https://github.com/dsmear/isegoria/tree/64a4c530781e65709b834ec6e0b3dc4f74cbcfd6), reviewed on 2026-10-01/02. |
| **Purpose** | Track the independent A–E review's open claims and acceptance criteria without treating either the code or the review as infallible. Stable review IDs are preserved below. |
| **Status** | Documentation alignment only. No scoring formula, threshold, protocol transition, fixture or measurement changed. This register does not mark an implementation finding fixed. |
| **Evidence** | L: source/report read; D: independent algebra or static deduction; C: Python calculation performed during the review; P: compilation and tests reported passing by the owner. Rust tests were not run by this reviewer: `cargo` was unavailable. P is not a claim that the post-D43 characterization has run. |

The five review answers concern correctness (A), statistics (B), consistency (C),
direction (D), and completion prospects (E). This is their repository handoff, not
a replacement for the dated audits or an external certification under T26.
Line references below are to the **baseline commit**, since documentation edits move
lines. Code symbols identify the implementation to re-check on a newer branch.

## How to read the documentation

- [`CLAUDE.md`](CLAUDE.md) holds the invariants; [`01`](01-decisions.md) records decisions.
  A decision's implementation status does not establish every mathematical claim made
  in its rationale. Design changes need an explicit decision amendment.
- [`02`](02-scoring-engine.md) states the current mechanism and identifies disputed
  interpretations. A mismatch with code remains a finding until resolved; it is not
  made correct by copying the code into the specification.
- [`08`](08-formal-specification.md) contains a dated audit plus subsequent remediation
  entries. Read those entries with this register; a historical RESOLVED label is scoped
  to the repair and evidence recorded there.
- [`10`](10-roadmap.md) owns task status; [`14`](14-parameter-register.md) owns calibration
  procedures. This register adds review findings, not new task IDs or deadlines.
- [`13`](13-characterization.md) and the report READMEs own measurement provenance.
  Pre-D43 DIF/DTF/floor results do not characterize D43. Bridging results have a
  separate provenance. A code sample floor is not a power guarantee.
- [`sim/`](../sim/README.md) contains research prototypes and scoped test oracles.
  The paper is a dated snapshot. Neither is an executable specification of every
  current production-path component.

## Open correctness findings

All entries below remain **open for implementation/design review**. The evidence
column says what was checked, not that a proposed correction has been validated.

| ID / severity | Finding and baseline source | Evidence and completion criterion |
|---|---|---|
| **A1 — critical for the incentive claim** | The epoch beacon determines assignment and exploration before reports. Domain separation does not hide the exploration outcome. `protocol/src/{randomness.rs:1–37,review.rs:28–41,exploration.rs:12–20}`; paper §7, `065-revisions.tex:207–224`. | L/D/C. Define the information available at reporting and prove IPW properness under that information. Test an adaptive reporting strategy, not only a forecast fixed before the draw. This is a counterexample to the simplified incentive argument, not a demonstrated attack on the full bridging fit. |
| **A2 — high** | Per-fit DTF uses each fit's own distribution/classes; summing it is not by itself a bound for a common target population. Unflagged active items need not contribute zero. `02:560–598`; `scoring/src/dtf.rs:104–154,194–224`; `protocol/src/contested.rs:126–142`. | L/D/C. Specify the target measure, linking assumptions and whole-test contribution, then prove the bound and separate estimation error. A sampling-error margin alone does not close this. |
| **A3 — high** | The histogram's claimed gauge can affect standardized shape. Its penalty and `Q−3` parameter count require justification. `02:318–331`; `scoring/src/latent.rs:205–207,241–262,818–859`. | L/D/C. Specify genuine constraints or regularization, the objective, model dimension and selection criterion consistently. The same two-parameter counting offset across histogram candidates does not alone alter their BIC ordering. |
| **A4 — high** | An unconverged latent fit produces false flags; `revalidate_batch_latent` discards fit status, while a false `emerging_dif` can restore ActivePool. `scoring/src/latent.rs:145–152`; `protocol/src/revalidation.rs:74–90`; `protocol/src/lifecycle.rs:580–590`. | L/D, static path, no observed failure frequency. Preserve an indeterminate outcome through the decision boundary; verify it cannot certify absence of DIF. |
| **A5 — high** | `expanded_ratings` adds a newcomer's row and weight without extending `axis`; the next fit can return `AxisCount`. `protocol/src/orchestrator.rs:205–227`; `scoring/src/bridging.rs:72–86,255–257`. | L/D, static. Verify newcomer → expanded ratings → fit → supplementary verdict, including the eligibility of the new row. The existing newcomer test at `protocol/tests/supplementary_redecision.rs:335–344` stops before the fit. |
| **A6 — high** | After a CUSUM reset, `status(true, 0)` restores founder weight 1, whereas §C.4 promises weight 0 until new outcomes. `protocol/src/probation.rs:20–36,94–104,152–166`; `02:800–803`. | L/D, static. Decide whether initial founder privilege survives a disciplinary reset; implement and test that explicit policy. Keep the mismatch visible until then. |
| **A7 — medium** | Participation differs between fitting, side averaging and bootstrap: zero-weight axis rows can affect the score; bootstrap can reuse positive weights of off-axis rows. `protocol/src/orchestrator.rs:49–62`; `scoring/src/bridging.rs:261–294,515–568,650–675`. | L/D/C. Define consistent participation semantics for fitting, partition, means, coverage and bootstrap. The off-axis/positive-weight case is an API-contract counterexample, not a demonstrated production mapping. |
| **A8 — medium** | Low power for one biased item is presented as structural non-identifiability. `paper/sections/04-level-b.tex:62–85,138–142`; `08:522–526`. | L/D. Separate distinguishability from the null, parameter identifiability and finite-sample power. Preserve observed simulation failures without turning them into a general impossibility theorem. |
| **A9 — medium** | Exact camp-mean reproduction and positive regularization need a non-degenerate stationary example for the majoritarian-leak proposition. `paper/sections/03-level-a.tex:154–203`. | L/D in the complete, uniformly weighted case. Exhibit compatible hypotheses or state a controlled approximation; no impossibility claim was established for every missingness/weight pattern. |
| **A10 — low** | With no other positive weight, the leave-one-out fallback uses the reviewer's own report, giving zero for every forecast. `scoring/src/reputation.rs:56–85`. | L/D. Scope strict properness to a report-independent baseline; treat the fallback as uninformative or change it through an explicit decision. |
| **C5 — medium** | The implemented author average is not the posterior mean of the Beta–Beta hierarchy formerly stated in §C.1. `02:648–662`; `scoring/src/reputation.rs:14–33`; `paper/sections/05-level-c.tex:8–16`. | L/D/C. Decide between the existing regularized index and a specified inferential model; also define the continuous quality input. The documentation now describes the implemented formula without claiming that posterior derivation. |
| **C6 — medium** | `exp(−age/18)` uses an exponential decay time, not an 18-month half-life. `02:656–662,902`; `scoring/src/reputation.rs:14–29`. | L/D/C. Current behavior is documented as a decay time, with half-life `T ln 2`. A policy change to an 18-month half-life remains a separate decision and code change. |

Paths beginning `scoring/` or `protocol/` in the table are relative to `crates/`;
numbered document shorthand is relative to `docs/`. Paper paths are repository-relative.

### Small reproductions to preserve during correction

- **A1:** use the test's simplified gate, `q=0.4`, baseline `0.7`, gate `0.5`,
  exploration `0.05`. Expected difference score is `(b−q)²−(p−q)²`.
  Truthful reporting gives `0.09`; reporting `0.4` on explored slots and `0.5`
  otherwise gives `0.05×(0.09/0.05)+0.95×0.08=0.166`.
  Source: `crates/scoring/tests/exploration_weights.rs:8–30,37–75` (L/D/C).
- **A2:** equal classes with normal unit ability, `a=1.25`, `c=0.2`,
  difficulties `−0.45,+0.45`: their gap `0.9` is below the flag cut `1.0`,
  while numerical integration gives unsigned DTF `0.1698619472`, above `0.10`.
  This refutes the zero-contribution claim even within one fit (D/C).
- **A3:** symmetric weights `(0.25,0.5,0.25)` and `(0.125,0.75,0.125)` on
  nodes `−1,0,1` both standardize to mean 0 and variance 1, but have fourth
  moments 2 and 4. Standardization does not remove every shape change (D/C).
- **C5:** for `q=0.8`, age 0, prior `Beta(2,3)`, and hierarchy parameter
  `κ=10`, integrating the stated hierarchy gives posterior mean `0.6682708888`;
  the implemented average is `(2+0.8)/6=0.4666666667`. `κ=10` is a
  counterexample value, not an implemented project parameter (D/C).

## Statistical and operational work still open

| Review IDs | Question and source | Completion evidence needed |
|---|---|---|
| **B1–B3** | Useful identification of `c`, sensitivity to its prior, anchor invariance, BIC under misspecification, and selection after penalized fitting. `02` §B.1/§B.3; `scoring/src/latent.rs:13–15,806–859,955–1019`. | Decision-level sensitivity, candidate/search stability and comparison with simpler alternatives and relevant misspecified populations. Do not equate fitted classes with real social groups. |
| **B4–B5, C3** | Current sample floors are not power guarantees; screening and KR-20 can reject informative batches or retain guessable items. `13` §8.7.1/§8.7.5/§8.8; `protocol/src/pilot.rs:284–318`. | Post-change results by format, gate admission, convergence, conditional and joint error rates, and indeterminate outcomes. The stage-1 filter has a stated purpose consistent with its information at that sample size. |
| **B6, D5** | Exact side splitting is not validation of the social meaning of the sides or confidence coverage; panel size, uncertainty band and extra round interact. `14` entries `d`, `k`; `13` §8.7.4. | Gate error/cost measurements for the declared domain; a second axis requires a score definition, not only a different dimension setting. |
| **B7, D1, D6, E** | T83 has no chosen operating point; model changes can repeatedly invalidate calibration. `10` T83; `14` “How to read an entry”; `13` §8.8. | Error and resource criteria fixed before final threshold selection; a frozen candidate and confirmation beyond the cases used to choose it. Phase 1 may retain provisional values with defined T27 procedures. |
| **D2, D4** | Respondent burden and current runtime need their own budgets. `02` §B.6; `characterization/src/generate.rs:82–89,116–143`; `runner.rs:117–119`. | Count anchors and unsuccessful batches; record latency and hardware for the actual candidate. `fit_seconds` is mean harness-run elapsed time, not an isolated optimizer benchmark. |
| **D3** | Contested selection enumerates subsets within each fit. `protocol/src/contested.rs:75–104,148–169,212–234`. | Define a sustainable maximum domain or revise selection while preserving its specified distribution. With `M` members and at most `n` slots the current traversal visits `Σ(r=0..min(M,n)) binom(M,r)` subsets (D/C). |

## Documentation disposition

| Review IDs | What this documentation pass does | What it does not close |
|---|---|---|
| **C1** | Identifies current specifications, historical simulations and scoped oracles. | It does not port retired simulations to the current model. |
| **C2** | Labels report versions and removes promises that every later model reproduces old DIF records. | It does not replace historical measurements or claim a post-D43 full run. |
| **C3** | Replaces conflicting sample/throughput promises with current code floors and measurement provenance. | It does not choose new sample sizes or approve the present gates. |
| **C4** | Points disputed guarantee closures to A1–A7 above. | It does not repair implementation or erase the audit's historical evidence. |
| **C5–C6** | Describes the author formula and actual time-decay convention accurately. | It does not choose a new model, continuous quality definition or decay policy. |
| **C7** | Names the current and retired entry points in the architecture and handoff. | Stale Rust doc comments in `revalidation.rs:22–25` and `pilot.rs:356–358` remain for the code pass; no `.rs` files are changed here. |
| **C8** | States T81's ten-file scope explicitly. | T81 stays complete in that scope; it is not a proof of every composed guarantee. |

## Working agreement for implementation and design review

1. The implementation agent checks each finding against its branch and records
   **confirmed / partly confirmed / rejected / unresolved**, with evidence. Do not
   close a finding merely because an existing test passes, or implement a review
   suggestion before checking its premises.
2. Work in small coherent changes. Reproduce a code defect with a failing test,
   repair it, and test the full affected decision path. For a mathematical claim,
   supply a derivation or counterexample as well as any numerical regression.
3. The design reviewer checks the property, assumptions, patch and documentation.
   Routine fixes proceed under the owner's correction mandate. Escalate only choices
   that change an invariant, a substantive guarantee, accepted errors or resource
   commitments; present a concrete recommendation and consequences together.
4. Keep implemented behavior, intended behavior and open mismatch distinguishable.
   Update the relevant sections and this register with each accepted correction.
   Preserve task IDs, historical run records and the scope of completed T81 work.
5. Settle changes to the estimator before final calibration. Use targeted checks and
   the smoke grid during development. Full characterization remains an explicit
   owner-run/owner-authorized campaign, with its candidate commit recorded.
6. A Phase 1 completion claim requires proportionate evidence for every critical
   claim (`08` §16.1), plus the stated procedures for provisional parameters.
   T26 must review the final candidate; T27 remains the empirical pilot dependency.

**First correction handoff:** verify A1–A3's guarantee premises and A4–A7's code paths.
The local input-contract defect A5 is a suitable first bounded implementation once its
reproduction and newcomer eligibility policy are explicit. T83 is needed before final
calibration, not before investigating these defects.
