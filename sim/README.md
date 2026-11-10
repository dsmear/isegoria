# Simulations

Historical research prototypes, fixture generators and scoped differential oracles
for the earlier forecast-and-pilot mechanism. They do not specify or validate the
current [answer review](../docs/05-question-lifecycle.md#the-answer-review-01-d49d52).

Some scripts still serve retained Rust code; retired BSS and proxy-DIF paths remain
historical. Reproducing their fixtures validates only the numerical contracts named
by the corresponding tests. Current engine contracts are in
[02](../docs/02-scoring-engine.md), evidence and finding states in
[15](../docs/15-phase1-review.md), and archived paths are listed below.

## Requirements

```
pip install numpy scipy
```

## Files

### `answer_bridging.py`

Exploratory, on synthetic rounds of the answer review; not a validation, and no rule it
compares is adopted ([`01` D52](../docs/01-decisions.md) keeps bridging out of the answer
review's acceptance conditions; `00` §Future direction). Standard library only, seeded.

Each run draws two camps of reviewers (majority share 0.6–0.8, positions as
`bridging_irt_dif.py`), then questions of six kinds, each answered by 12 reviewers drawn
uniformly, a tenth of the answers missing. An answer coincides with the key with probability
`q + 0.45 · f_u · lean`, clipped. Three readings of the same answers are counted per kind:

- `d49`: the count of `01` D49 (quorum 9, three quarters);
- `sb75`, `sb80`: the quorum, a usable answer from each estimated side, and the side-balanced
  score `S_j ≥ τ` (`02` §A.3, D32, D42) from the bridging fit of all the run's answers,
  0 or 1, with equal weights. The full fit's score, not the bootstrap minimum;
- `known75`: the mean of the two generated camps' observed agreement at 0.75, sides known —
  what 12 answers allow when the axis is not to be estimated.

The fit minimizes `02` §A's objective by exact block updates from four seeded starts, the
lowest objective kept: one from the leading singular pair of the intercepts' residuals, three
random. Random starts alone stopped at worse optima on these sparse 0/1 data and understated
the axis' recovery. It is not the engine's optimizer and no fixture depends on it. `check()`
runs first: D49's
boundaries as AT-ANS-01, the side floor, the cut, and the gradient at the fit's end.

```
python3 sim/answer_bridging.py --quick           # two cells, seconds
python3 sim/answer_bridging.py --out cells.csv   # 72 cells, about 85 CPU-minutes
```

Indicative results of the full grid (2026-11-11, pooled over 50 and 200 reviewers; 450 to
1,600 majority-partisan questions per row, not confidence bounds), by `d`, the answers to
divisive questions a reviewer has given — answers per reviewer times the divisive share.
Accepted, in percent: the majority camp's partisan key, then a consensus question (`q` = 0.90).

| majority | `d` | partisan: `d49` | `sb75` | `known75` | consensus: `d49` | `sb75` | `known75` |
|---|---|---|---|---|---|---|---|
| 60% | ≤ 1.2 | 18 | 15 | 7 | 91 | 70 | 90 |
| 60% | 3.6 | 18 | 9 | 7 | 91 | 83 | 90 |
| 60% | 7.2 | 18 | 5 | 7 | 91 | 87 | 90 |
| 60% | 14.4 | 18 | 2 | 5 | 91 | 87 | 89 |
| 70% | ≤ 1.2 | 35 | 21 | 9 | 91 | 69 | 88 |
| 70% | 3.6 | 34 | 17 | 8 | 92 | 82 | 89 |
| 70% | 7.2 | 33 | 10 | 10 | 90 | 83 | 88 |
| 70% | 14.4 | 32 | 6 | 8 | 92 | 87 | 89 |
| 80% | ≤ 1.2 | 53 | 35 | 13 | 91 | 67 | 81 |
| 80% | 3.6 | 55 | 24 | 11 | 91 | 73 | 82 |
| 80% | 7.2 | 58 | 19 | 15 | 91 | 79 | 82 |
| 80% | 14.4 | 55 | 10 | 10 | 91 | 80 | 82 |

Below about `d` = 2.4 the fitted axis does not separate the camps (correlation with the
generated positions 0.07–0.37) and the score mostly refuses more of every kind. Not
represented: reviewers who hide their side, shared tools or sources (`01` D45), more than two
camps, a draw stratified on the axis, sides estimated only from earlier epochs.

### `bridging_irt_dif.py`

End-to-end simulation over 10 civic questions, 200 reviewers (a 60/40 unbalanced
network), 1500 respondents with 30 anchor items. Covers:

- **Level A** — bridging with asymmetric regularization; the side-balanced score `S_j`
  (two sides by the exact 2-means of `f_u`, each at least 5% of the reviewers; the
  model's predicted approval, clipped to [0, 1], averaged per side, the two averaged
  again; `docs/02` §A.3, D32, D42) with its bootstrap-min and side gap; latent axis
  recovery
- **Level B (historical)** — IRT, point-biserial, DIF via logistic regression with
  generated group labels and anchor-total ability; not the current anonymous re-check
- **Combined verdict** of the two filters
- **Evaluators (historical)** — retired Brier Skill Score for various profiles
  (follows-the-crowd, expert, partisan…); not the current difference score
- **Corner case 1** — elite consensus (an item neutral on the political axis,
  distorted on education)
- **Corner case 2** — cost of bipartisan corruption
- **Corner case 3** — threshold sweep (the false-positive / partisan-item trade-off)

Parameters editable at the top of the file: the majority camp's share (`share_B`), the
threshold (`TAU`), the item leans.

Historical prototype results (indicative, seed-dependent; not current-system rates):
- latent axis recovered with correlation ~0.99
- the consensus items score `S_j` ≈ 0.83–0.86 and pass at τ = 0.80; the partisan items
  (0.53–0.56) and the mildly partisan one (0.70) drop; the intercept `b_j` is printed
  alongside for comparison
- the MES item passes bridging but is stopped by DIF (β₂ ≈ 0.7)
- "follows the peer average" → negative BSS; "psychometric expert" → BSS ~0.95
- elite consensus: political DIF ≈ 0, education DIF ≈ 0.66
- bipartisan corruption: ~55/80 nodes of the opposing camp are needed to pass the item
  (`S_j` 0.75 with 40, 0.83 with 55)

### `latent_dif_and_capacity.py`

Two experiments:

1. **Detecting bias without knowing which axis to look on** — historical proxy-ability IRT mixture,
   the distorting axis never observed. Shows that with ≥2 distorted items in a batch the
   model estimates delta ~1.0 on the defective ones and ~0.1 on the clean ones, and
   reconstructs the hidden axis (correlation 0.5–0.8). A single distorted item is not
   detected in this experiment; that is a finite-sample result, not a general
   non-identifiability proof. The protocol's batch requirement remains.
2. **Sustainable proposal quota** — shows the bottleneck is pilot respondents, not
   reviewers under its original sample/cost assumptions. It motivated the lottery
   (`docs/01` D10); its quota calculation is not a capacity estimate for the current
   guessing-aware model. See `docs/02` §B.6 for the cost accounting still needed.

### `export_fixtures.py`

Regenerates the historical oracle fixtures consumed by the Rust acceptance tests
(`crates/scoring/tests/fixtures/`). It mirrors the two simulations above on the same
seeds and dumps the datasets and the full-fit results as CSV. Run it as:

```
python export_fixtures.py <output_dir>
```

### `oracle_bridging.py`, `oracle_mixture.py`

Differential oracles on random datasets (`docs/08` REPRO-003, T45): NumPy/SciPy
implementations of the bridging fit and of the two-class mixture, each in its script (taken
unchanged from the archived paper's `scripts/common.py`), on a dataset the Rust tests write,
seeded multi-starts, the lowest objective kept. `crates/scoring/tests/differential_oracle.rs` runs them and
compares; it self-skips without the pinned environment (`requirements.txt`).

```
python oracle_bridging.py <dir>    # reads R.csv, mask.csv, weights.csv; writes oracle.csv
python oracle_mixture.py <dir>     # reads theta.csv, X.csv; writes oracle.csv
```

### `latent_shape_dimension.py`

A3's calculations (`docs/18`): the latent ability histogram's map from logits to a
standardized distribution, as `scoring::latent` computes it — the review's 3-node example,
the map's rank (`Q − 1`), and how much the grid's mean and variance move the response
patterns of eight 2PL items at a smooth and at a rough histogram. numpy only; deterministic.

```
python latent_shape_dimension.py
```

### `d3_selection_counts.py`

D3's exact counts (`docs/15`, D3 section): the subsets `ContestedPool::draw` evaluates per fit,
`S(M, n) = Σ_{r ≤ min(M, n)} C(M, r)`, and their member terms, for the pools of the tests and the
fit sizes the documents motivate; it first checks a transcription of the enumeration loop against
the formula. Standard library only, under a second. The isolated timing is the Rust example
`crates/protocol/examples/d3_draw_cost.rs`. From the repository root:

```
python3 sim/d3_selection_counts.py
cargo build --release -p protocol --example d3_draw_cost
target/release/examples/d3_draw_cost 16 8 2
```

### `d3_domain_limits.py`

D3's resource domain (`docs/15`, D3 section): the limits `protocol::contested` enforces
(`DRAW_LIMITS`, `POOL_LIMITS`), the byte model behind them — allocations as the draw makes them,
capacities, a reallocation's old copy, the stack, glibc's chunk overhead, every subset kept — its
bound with every limit reached at once, the pool's estimate, and which documented configurations
and measured shapes are admitted or refused, with the limit that refuses them. Standard library
only, under a second. The measurements are the Rust example
`crates/protocol/examples/d3_domain_cost.rs`, whose counting allocator reports the bytes a draw
requests; the regression record of the draw's outcomes is rewritten by `D3_RECORD`:

```
python3 sim/d3_domain_limits.py
cargo build --release -p protocol --example d3_domain_cost
/usr/bin/time -f "%e s %M KB" target/release/examples/d3_domain_cost kept
D3_RECORD=$PWD/crates/protocol/tests/fixtures/d3_draws.txt cargo test -p protocol --test d3_draw_regression
```

The shapes are `kept`, `entries`, `terms`, `terms-all`, `cells-one`, `cells-many`, `scans`,
`combined`, `combined-all`, `pool` and `input`.

## Archived scripts (D47)

The A12 offline reference (`a12_offline.py`, `a12_weight_counterexample.py`,
`export_a12_affine.py`, their tests and `fixtures/a12_*.json`) and A2's tools
(`dtf_composition.py`, `a2_information_check.py`, `a2_probe.py`, `a2_tradeoff.py`, `a2_size.py`,
their tests and reference data) moved on 2026-10-21 to
[`archive/phase1-forecast-pilot/sim/`](../archive/phase1-forecast-pilot/sim/README.md), with their
documentation, under [`docs/01` D47](../docs/01-decisions.md). They belong to the previous Phase 1
design; the path map is in the [archive's index](../archive/phase1-forecast-pilot/README.md).

## What these prototypes demonstrate

Motivating scenarios documented in `docs/06-threat-model.md`, within these prototypes:
- the true-but-divisive false negative (no threshold saves a polarizing fact) —
  motivates the appeal channel
- the elite-consensus blind spot — motivates multi-axis DIF
- the fixture's ~30% survival rate — an example, not an expected production yield
- the threshold as a blade — motivates the uncertainty band
