# A2's single-test study: calibration, on `7bbfddb`

The raw records, plan, ledger, threshold and checks of the calibration stage declared in
[`docs/19` §13.21](../../../docs/19-a2-dtf-composition-design.md#1321-the-single-test-study-declaration-and-implementation-not-run);
its result is stated in
[§13.22](../../../docs/19-a2-dtf-composition-design.md#1322-the-single-test-study-calibration-run-on-7bbfddb).
This directory is the frozen calibration a later `ind-val` run names with `--calibration`:
`threshold.tsv` names `records.jsonl`, `plan-ind-cal.tsv` and `ledger.tsv` by SHA-256, and they are
byte copies of the run directory's. Validation has not run.

| | |
|---|---|
| Candidate | `7bbfddb` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `b618aaa`, `HEAD:Cargo.lock` `e661541`, `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `61f048fb…66248e7d` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-19, from 13:35 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 200 of 200 `read` (C0 and C1, 100 each); no stop, refusal, killed or failed run |
| Caps | CPU 2 h, wall 30 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-ind-cal.tsv`) |
| Cost | 4,790.56 s CPU (79.8 min), 609.1 s wall (10.2 min); largest sampled resident sum 54.3 MiB (55,564 KiB), each run's peak 6.2–6.7 MiB; largest gap between checks 150 ms. CPU a run: C0 23.2 s mean (10.0–38.4), C1 24.6 (10.9–39.2) |
| Run directory | `characterization-results/a2-ind-cal-7bbfddb/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase ind-cal --jobs 8 --max-cpu-s 7200 --max-wall-s 1800 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (2,600 values, largest `\|A − B\|` 4.3·10⁻¹⁴; the audit complete, 200 of 200) |
| `python3 sim/a2_probe.py threshold D > D-logs/threshold.tsv.tmp`, renamed to `D/threshold.tsv` after exit 0 | 0, nothing on standard error |
| `python3 rank_check.py records.jsonl > rank-check.txt` (here) | 0 |
| `python3 sim/a2_probe.py threshold` on this directory, compared with `threshold.tsv` | 0, identical |

| File | Contents |
|---|---|
| `records.jsonl`, `plan-ind-cal.tsv`, `ledger.tsv` | the 200 records, the plan (configuration, runs and seeds), the supervisor's ledger |
| `threshold.tsv` | the threshold and the rule's terms, as §13.21 declares; the file the validation is bound to |
| `environment.txt`, `ind-cal.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `check.exit`, `threshold.exit` | the precision check with the audit; the two readings' exit codes |
| `rank_check.py`, `rank-check.txt` | the threshold recomputed from the records apart from `sim/a2_probe.py`: per case the values around the rank and the ties |
