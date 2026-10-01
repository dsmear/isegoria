# T25, step 3: the guessing supplement's full run

The tables of the supplement's grid (`docs/13` §8.3), as the harness summarized them. The
results are stated and read in `docs/13` §8.7; this directory is the evidence they rest on.

| | |
|---|---|
| Runs | 19,700 — every cell of `docs/13` §8.3 at its full replicate count, every run recorded, no `errors.log` |
| Commit | the harness of `a9cdafa` (pushed as `5f45d16` before the branch was rebased onto `master`), as the owner pulled it; every later commit keeps what the runs compute — a record of each kind is pinned (`tests/harness.rs`) |
| Date | the first pass (`--replicates 20`) on 2026-09-29, the rest by 2026-09-30 |
| Machine | the owner's, as for T24: 8 cores, 16 threads, Linux |
| Wall time | about 10 hours for the full run |
| Records | kept by the owner (`characterization-t25/<study>/records.csv`); each reproduces from its seed on any machine (`docs/13` §2) |

| File | Contents |
|---|---|
| `summary.md` | every study's table and the DIF cut with a guessing floor, per floor and format |
| `thresholds-dif-cut-floor.csv` | per floor, format and cut on `DIF_j` (0.5–1.5): the clean-item rate on the null batches with 40 or 60 anchors and the power of each two-of-eight cell with 60 anchors, `π = 0.5` |
| `bridging-lambda-tau.csv` | `bridging-lambda/tau.csv`: per `τ` (0.70–0.90) and reviewer count, over the eight `(λ_b, λ_f)` pairs, the robust score's false passes and false failures, no band |
| `<study>-summary.csv` | each study's `summary.csv`: the rows of `summary.md` with every column — the share the gates admit, the interval bounds, the fitted floors' spread; for `bridging-extra`, per cell, `ε` and `k_extra`, the intervals of the false passes and failures and the extra reviews per probe, which `summary.md` gives as point estimates |

## The screen's study

`floor-screen` (`docs/13` §8.1–8.4), the stage-1 screen of T25's second step, stated in
`docs/13` §8.7.5.

| | |
|---|---|
| Runs | 3,600 — its 18 cells at 200 replicates, every run recorded, no `errors.log` |
| Commit | the harness of `ec34fa4` |
| Date | 2026-09-30 |
| Machine | the cloud development container: 4 cores, Linux |
| Wall time | 3 min 26 s, four workers |
| Records | not kept; each reproduces from its seed on any machine (`docs/13` §2) |

| File | Contents |
|---|---|
| `floor-screen-summary.md` | the study's tables: per cell, the share kept of each kind and of each good item; the thresholds, as each moves |
| `floor-screen-summary.csv` | per cell and kind, the share kept with its interval |
| `floor-screen-items.csv` | per cell and item, the share of runs it entered the fit, the share kept, the mean and spread of its fitted `a`, `b`, `c` and its mean point-biserial |
| `thresholds-screen.csv` | per threshold (`A_MIN`, `B_ABS_MAX`, `C_EXCESS_MAX`) and value, stage-1 size and format, over anchors: each kind's share kept with its interval |
