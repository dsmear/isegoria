# T24 characterization: the full run

The tables of the full grid of `docs/13` §4, as the harness summarized them. The results
are stated and read in `docs/13` §7; this directory is the evidence they rest on.

| | |
|---|---|
| Runs | 54,412 — every cell of `docs/13` §4 at its full replicate count, every run recorded |
| Commits | the DIF and DTF studies ran on `e8dcc7d`; the two bridging studies were re-run after T71, and the sixteen cells added after the first pass ran, on `9478533`, which also summarized every record. The DIF and DTF records reproduce bit for bit on the later commits (`docs/13` §6) |
| Date | 2026-09-26 to 2026-09-28 |
| Machine | the owner's: AMD Ryzen 7 5800X (8 cores, 16 threads), Linux |
| Wall time | about 30 hours in all |
| Records | kept by the owner (`characterization-results/<study>/records.csv`); each reproduces from its seed on any machine (`docs/13` §2) |

| File | Contents |
|---|---|
| `summary.md` | every study's table, the τ table, the capture curve every 20 boosters, and the two threshold tables |
| `thresholds-dif-cut.csv` | per cut on `DIF_j` (0.5–1.5): the clean-item rate on the admitted null batches and the power of each two-of-eight cell with 60 anchors and `N ≥ 3000` |
| `thresholds-a-gap.csv` | the same per cut on `a_gap` (0.2–1.0), on the pure non-uniform cells |
| `bridging-sweep-tau.csv` | `bridging-sweep/tau.csv`: per `τ` (0.70–0.90) and reviewer count, the robust score's false passes and false failures, no band |
| `bridging-capture-curve.csv` | `bridging-capture/curve.csv`: per cell and opposing count, the robust, full and plain scores and the share of runs passing |

The per-study `summary.csv` files hold the same rows as `summary.md` with more columns
(means beside medians, interval bounds); they stay with the records.
