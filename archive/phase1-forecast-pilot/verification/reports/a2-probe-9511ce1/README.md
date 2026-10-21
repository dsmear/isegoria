# A2's first bounded probe, on `9511ce1`

The raw records, plans, ledger, checks and reports of the probe declared in
[`docs/19` §13.11](../../../docs/19-a2-dtf-composition-design.md#1311-first-bounded-probe-declaration-and-harness-not-run);
its results are stated and read in
[`docs/19` §13.12](../../../docs/19-a2-dtf-composition-design.md#1312-first-bounded-probe-result-run-on-9511ce1).
Explore measured cost only; its values are not results and are not pooled with confirm's.

| | |
|---|---|
| Candidate | `9511ce1` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `d0cdf4e`, `HEAD:Cargo.lock` `e661541`, clean; `crates/scoring` and `crates/protocol` identical to `6274270`; `a2-probe` SHA-256 `d8f128da…3065b00` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-19, explore from 08:18, confirm right after (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | explore 6 of 6 `read`; confirm `R = 8` from the cost rule, 48 of 48 `read`; no stop, no refusal, no killed or failed run, no rejected part |
| Cost | explore 172.87 s CPU, 44.6 s wall; confirm 1,551.08 s CPU, 220.0 s wall; cumulative 1,723.95 s CPU (28.7 min) of the 2 h cap, 264.6 s wall of the 45 min cap. Largest sampled resident sum 41.6 MiB (explore), 55.0 MiB (confirm), 42,568 and 56,356 KiB; each run's peak 6.2–6.6 MiB. Largest gap between checks 134 ms (explore), 147 ms (confirm). 8 workers |
| Run directory | `characterization-results/a2-probe-9511ce1/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase explore --jobs 8 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (82 values, largest `\|A − B\|` 4.7·10⁻⁸) |
| `python3 sim/a2_probe.py report D --phase explore` | 0 (complete) |
| `python3 sim/a2_probe.py budget D` | 0 (`R = 8`) |
| `cargo run --release -p characterization --bin a2-probe -- run --phase confirm --jobs 8 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (736 values, largest `\|A − B\|` 4.8·10⁻⁷) |
| `python3 sim/a2_probe.py report D --phase confirm` | 0 (complete) |

**`R`.** Explore's CPU per case: C0 21.66 s, C1 21.54, C2 44.41, C3 25.51, C4 37.63, C5 22.00;
`Σ c = 172.75 s`, spent 172.87 s. `R = min(8, ⌊(7,200 − 172.87)/(1.5 × 172.75)⌋) = min(8, 27) = 8`;
`a2-probe plan --phase confirm` gave the same 48 runs.

| File | Contents |
|---|---|
| `records.jsonl` | the 54 records, one JSON line per run (schema `a2-probe/v1`) |
| `ledger.tsv`, `plan-explore.tsv`, `plan-confirm.tsv` | the supervisor's ledger and the two plans (configuration and runs with seeds) |
| `environment.txt`, `*.cmd`, `*.stderr`, `*.exit`, `*.time`, `confirm.load` | commit, machine, commands, the supervisor's summaries, exit codes, `/usr/bin/time -v` |
| `explore-check.json`, `confirm-check.json` | the precision check and the audits, after each phase |
| `explore-report.md`, `confirm-report.md`, `budget.json` | the reports and the cost rule's inputs |
| `confirm-details.json`, `details.py` | per run: fit, readings, draw, cost and forms; `python3 details.py D sim` |
| `confirm-factual.json`, `factual.py` | C2 and C4 under S1: the draws and the generating DTF with every sourced fact kept, method B; `python3 factual.py D sim confirm` |
