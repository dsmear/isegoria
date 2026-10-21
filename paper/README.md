# Working paper: the mathematics of Isegoria

**Opinion as Filter, Evidence as Verdict: The Mathematics of Isegoria, an Authority-Free
Mechanism for Validating Test Items** (working paper, version 0.2, September 2026).

- **PDF:** [`main.pdf`](main.pdf), the existing September snapshot; not rebuilt for the
  source amendments of 2026-10-11 and 2026-10-14.
- **Describes:** the code at commit `2ee5e79` and the design decisions D32–D41 of
  [`docs/01`](../docs/01-decisions.md) (commit `7507eab`); D32 is implemented since T49
  (2026-09-24), the rest are planned (roadmap `docs/10` Phase 1.1). The paper is a dated
  snapshot: what the repository changed after it is listed below
  ([Since version 0.2](#since-version-02)). The living specification is still
  [`docs/02-scoring-engine.md`](../docs/02-scoring-engine.md) and
  [`docs/08-formal-specification.md`](../docs/08-formal-specification.md). A new version of the
  paper names the commit it describes.
- **Versions:** 0.1, the analysis; 0.2, adds Section 7, *Adopted revisions*.
- **Status under D47 (2026-10-21):** a historical snapshot. Its Level C analysis (finding 4
  below, Section 7's D33–D36 row, `prop:ipw`) concerns the reviewers' forecast score, which
  [`docs/01` D47](../docs/01-decisions.md) sets aside as their main contribution; its Level A and
  B analyses concern code still implemented. It stays in place: `scripts/common.py` is imported by
  `sim/oracle_bridging.py` and `sim/oracle_mixture.py`, which CI runs as differential oracles, and
  its scripts write the data its LaTeX reads ([`docs/21`](../docs/21-d47-inventory.md) §4).

## What it contains

The paper states the scoring mechanism formally: bridging (Level A), IRT with latent-class DIF
(Level B), reputation (Level C) and the anti-collusion discount. The identity and network layers
are abstracted as explicit assumptions. Each result is labelled *proved*, *empirical*, *design*
or *open*. The analysis finds five properties that the design documents do not anticipate. Each
comes with a proof or a reproducible experiment and a candidate correction:

1. **The bridging gate is relative to its batch.** With an unpenalized global mean, item
   intercepts sum to zero at every stationary point. The six consensus items of the reference
   simulation pass (5 of 6) when fitted with divisive items and all fail when fitted alone.
2. **The bridge score is partly majoritarian.** The origin of the latent axis is a gauge fixed only
   by the regularization. The corrected two-camp result describes fitted camp means,
   with a residual bound against raw ratings; the historical raw-lean approximation uses
   `1/(1+ρ)`, `ρ ≈ (λ_b/λ_f)·sqrt(S/n)`. At the default penalties the historical measured
   leak is 0.53–0.87 for 50–3,200 reviewers.
3. **Paired distortions supply conditional covariance; ability error confounds DIF.** A single biased
   item carries 250–300 times less information than a biased pair. An imperfect ability proxy
   creates spurious latent classes. On null batches the production detector flags clean items
   when θ comes from 10 or 20 anchors, and at 30 anchors it passes by a margin of 0.01–0.06.
4. **The evaluator's skill score is not proper.** Against a crowd baseline `b`, the optimal report
   on one scored item satisfies `logit p* = logit q + 2 logit b`, which pushes dissenters toward
   the crowd. A leave-one-out difference score is strictly proper under stated conditions (fixed
   scored items, a defined baseline, a joint law of baseline and outcome that the report does not
   move), and still gives exactly zero to a reviewer who copies the crowd. The weight cap `3 × median` never binds for `E_u ∈ (0,1)`.
5. **The anti-collusion discount is only as strong as its detector.** At design scale two
   reviewers share under one item per epoch, so random assignment carries the per-epoch defence.

## Adopted revisions (version 0.2, Section 7)

| Finding | Revision | Decision / task |
|---|---|---|
| 1, 2 | side-balanced bridge score: sides by 2-means on `f_u`, predicted approval averaged per side, each side counts once; absolute threshold ≈ 0.80. Leak falls to between −0.08 and 0.00; the score is stable to ±0.01 with or without other items and decoys | D32 / T49 (done 2026-09-24) |
| 4 | leave-one-out difference score; odds weights `exp(γ·S·k/(k+100))`, `γ ≈ 35`; CUSUM change detector instead of the asymmetric update; outcomes of live items plus 5% randomized exploration of rejections (original claim qualified by the O1 amendment to `prop:ipw`); probation of 30 | D33–D36 / T50–T52 |
| 3 | anchor KR-20 ≥ 0.90 before a latent re-check (about 40 anchors); the differential gap only as a diagnostic (it inverts in a campaign); θ inside the likelihood as the target model | D37 / T53, T54 |
| — | contested facts (DIF on knowledge, key backed by a primary source) in a balanced pool | D38 / T55 |
| 5 | coordination detected on model residuals over long histories (honest pairs flagged 50.6% → 0%); clusters limit panel co-assignment instead of losing weight | D39, D40 / T56, T57 |
| — | beacon: commit-reveal now, a threshold signature after the DKG | D41 / T37 |

## Since version 0.2

The September results and the stored PDF remain a dated snapshot. The LaTeX sources now have
an explicit **O1 target amendment (owner's decision, 2026-10-11; approved on `e735a8e`)**:
`prop:ipw` states the conditional mean for `q_c`, defined only where `c > 0`; at `c = 0` it
is undefined and the score is flat. The binary analysis is labelled historical / conclusive.
Motivation, limits and the affected promises are recorded once in
[`docs/17`](../archive/phase1-forecast-pilot/docs/17-a1-pilot-batch-design.md) §14.12.
No data, figure, PDF, code or expected value is regenerated. Current formulas,
tasks and evidence live in `docs/02`, `docs/10`, `docs/13` and `docs/14`.

| Snapshot topic | Subsequent implementation and current qualification |
|---|---|
| Bridging and appeal (D32) | Side-balanced score and side-gap appeal implemented; D42 adds the exact cut, side floor, clipping and coverage. Current limits and open participation contracts: `docs/02` §A; `docs/15` A7/B6. |
| Evaluator score, dynamics and exploration (D33–D36) | Difference score, odds weights, CUSUM and exploration implemented. O1 changes A1's decided target; the binary runtime, observation design and reputational guarantees remain separate (`docs/17` §14.12); A1 stays open. |
| DIF (D37, D25, D43) | Anchors enter the likelihood; formats determine guessing floors; an estimated common ability histogram replaces the fixed normal in the re-check. Histogram interpretation and current calibration remain open: `docs/15` A3/B1–B5; `docs/13` §8.8. |
| Contested facts (D38) | Per-fit DTF selection implemented; a common-population whole-test bound is not established by that implementation: `docs/15` A2/D3. |
| Coordination and beacon (D39–D41) | Residual-history detection, panel diversification and the epoch beacon implemented. Their scope is documented in `docs/02` and `docs/04`; implementation does not by itself establish the exploration information assumptions. |
| Protocol composition and reproducibility | Band/appeal paths, validation and Rust golden checks have subsequent fixes. New contract findings remain open (`docs/15` A4–A7); Python comparisons are scoped numerical checks, not bitwise equality with Rust. |

The bounded source correction of 2026-10-14 separates single-item distinguishability,
parameter identifiability and finite-sample power (A8), and states the leak on fitted camp means
with a raw-score residual bound and a compatible stationary witness (A9). Astra accepted the
bounded correction and closed these two findings within that scope; the disposition is recorded in
[`docs/15` A8–A9](../docs/15-phase1-review.md). The stored PDF remains the September snapshot,
without these amendments. Historical numbers, figures and tables were not regenerated or
re-labelled as current-system measurements.

## Layout

| Path | Contents |
|---|---|
| `main.tex`, `preamble.tex`, `sections/` | LaTeX sources |
| `references.bib` | bibliography |
| `data/` | CSV outputs of the scripts; the figures and several tables read them directly |
| `scripts/` | one script per table/figure (see Appendix C of the paper) |
| `scripts/dif-harness/` | runs the production detector (`crates/scoring`) on generated batches |

## Building

```sh
cd paper
latexmk -pdf main.tex        # TeX Live with pgfplots, cleveref, natbib, booktabs
```

## Reproducing the numbers

```sh
cd paper/scripts
pip install -r requirements.txt   # numpy, scipy
python levelA_relativity.py       # Lemma 1 identities, Table 2
python levelA_leak.py             # Table 3, Figure 2 (~8 min)
python levelB_information.py      # Table 4
python levelB_proxy.py            # Table 5 (~2 min)
python levelB_detector.py         # Table 6 and production-detector counts (needs cargo; ~8 min)
python levelB_differential.py     # differential gap (Section 4.5)
python levelC_bss.py              # Figure 3, Table 7
python assignment.py              # Table 8
python revisions_bridging.py      # Tables 10-11 (~8 min)
python revisions_evaluator.py     # Tables 12-13, scenario rates (~5 min)
python revisions_dif.py           # Tables 14-15
python revisions_collusion.py     # Table 16, panel and beacon figures
```

All randomness is seeded. `levelB_detector.py` builds `scripts/dif-harness`, a standalone Cargo
package (its own workspace) that depends on `crates/scoring` by path.

## Disclosure

The analysis, experiments and text were prepared with the assistance of an AI system.
A separate AI-assisted A–E review is tracked in
[`docs/15`](../docs/15-phase1-review.md); its open findings are not an external
certification under T26. Reviews, corrections and objections remain welcome.
