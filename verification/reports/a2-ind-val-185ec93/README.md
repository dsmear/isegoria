# A2's single-test study: validation, on `185ec93`

The raw records, plan, ledger, checks and readings of the validation stage declared in
[`docs/19` §13.21](../../../docs/19-a2-dtf-composition-design.md#1321-the-single-test-study-declaration-and-implementation-not-run);
its result is stated in
[§13.23](../../../docs/19-a2-dtf-composition-design.md#1323-the-single-test-study-validation-run-on-185ec93).
It ran against the calibration frozen in
[`../a2-ind-cal-7bbfddb/`](../a2-ind-cal-7bbfddb/README.md), which it did not write to:
`t = 0.7984473698688216`, bits `3fe98ce17fa53de3`.

| | |
|---|---|
| Candidate | `185ec93` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `b618aaa` and `HEAD:Cargo.lock` `e661541`, the calibration's; `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `61f048fb…66248e7d`, the calibration's binary (`environment.txt`, `ledger.tsv`) |
| Calibration | `verification/reports/a2-ind-cal-7bbfddb/`, committed and unmodified before and after; `threshold.tsv` SHA-256 `34fb80c3…cd840`, the plan's `calibration` line |
| Date | 2026-10-19, from 13:53 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 400 of 400 `read` (C0, C1, C3 and C5, 100 each); no stop, refusal, killed or failed run |
| Caps | CPU 4 h, wall 60 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-ind-val.tsv`) |
| Cost | 8,553.94 s CPU (142.6 min), 1,083.4 s wall (18.1 min); largest sampled resident sum 53.9 MiB (55,236 KiB), each run's peak 6.2–6.7 MiB; largest gap between checks 145 ms. CPU a run: C0 21.3 s mean (7.5–35.8), C1 21.5 (6.5–34.8), C3 21.3 (9.4–38.9), C5 21.4 (9.7–35.0) |
| Run directory | `characterization-results/a2-ind-val-185ec93/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory, `F` the calibration):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase ind-val --calibration F --jobs 8 --max-cpu-s 14400 --max-wall-s 3600 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (5,200 values, largest `\|A − B\|` 4.0·10⁻¹⁴; the audit complete, 400 of 400) |
| `python3 sim/a2_probe.py ind-val D F` | 0 |
| `python3 sim/a2_probe.py ind-val D F --json` | 0 |
| `python3 val_check.py records.jsonl F/threshold.tsv ind-val.json > val-check.txt` (here) | 0 |

| File | Contents |
|---|---|
| `records.jsonl`, `plan-ind-val.tsv`, `ledger.tsv` | the 400 records, the plan (configuration with the `calibration` line, runs and seeds), the supervisor's ledger |
| `environment.txt`, `ind-val.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, calibration, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `check.exit` | the precision check with the audit |
| `ind-val-report.md`, `ind-val.json` and their `.exit` | the declared reading, as text and as JSON |
| `val_check.py`, `val-check.txt` | the reading recomputed from the records and the frozen threshold apart from `sim/a2_probe.py`, intervals by SciPy 1.18.1 |
