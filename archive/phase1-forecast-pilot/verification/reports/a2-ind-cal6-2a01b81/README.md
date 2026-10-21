# A2's study at `N = 6,000`: calibration, on `2a01b81`

The raw records, plan, ledger, threshold and checks of the calibration stage `ind-cal6` declared in
[`docs/19` §13.28](../../../docs/19-a2-dtf-composition-design.md#1328-next-study-calibration-and-validation-at-n--6000-proposal-not-implemented)
and implemented in
[§13.29](../../../docs/19-a2-dtf-composition-design.md#1329-the-study-at-n--6000-implementation-and-declaration-not-run);
its result is stated in
[§13.30](../../../docs/19-a2-dtf-composition-design.md#1330-the-study-at-n--6000-calibration-run-on-2a01b81).
This directory is the frozen calibration a later `ind-val6` run names with `--calibration`: it
holds `plan-ind-cal6.tsv` alone, and `threshold.tsv` (schema v2, `stage ind-cal6 n 6000`) names
`records.jsonl`, `plan-ind-cal6.tsv` and `ledger.tsv` by SHA-256; they are byte copies of the run
directory's. Validation has not run.

| | |
|---|---|
| Candidate | `2a01b81` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `dfd81c1`, `HEAD:Cargo.lock` `e661541`, `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `eff363fb…da3445eb` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-20, from 11:21 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 200 of 200 `read` (C0 and C1, 100 each, `N = 6,000`, 60 anchors); no stop, refusal, killed or failed run |
| Caps | CPU 4 h, wall 45 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-ind-cal6.tsv`) |
| Cost | 8,639.99 s CPU (144.0 min), 1,107.0 s wall (18.4 min); largest sampled resident sum 81.8 MiB (83,780 KiB), each run's peak 9.6–10.3 MiB; largest gap between checks 146 ms. CPU a run: C0 41.7 s mean (15.3–69.2), C1 44.7 (18.2–71.2) (`cost-check.txt`) |
| Run directory | `characterization-results/a2-ind-cal6-2a01b81/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory, `F` this one):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase ind-cal6 --jobs 8 --max-cpu-s 14400 --max-wall-s 2700 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (2,600 values, largest `\|A − B\|` 4.8·10⁻¹⁴; the audit complete, 200 of 200) |
| `python3 sim/a2_probe.py threshold D --phase ind-cal6 > D-logs/threshold.tsv.tmp`, renamed to `D/threshold.tsv` after exit 0 | 0, nothing on standard error |
| `python3 sim/a2_probe.py threshold F --phase ind-cal6`, compared with `threshold.tsv` | 0, identical |
| `python3 rank_check.py records.jsonl > rank-check.txt` (here) | 0 |
| `python3 cost_check.py . > cost-check.txt` (here) | 0 |

| File | Contents |
|---|---|
| `records.jsonl`, `plan-ind-cal6.tsv`, `ledger.tsv` | the 200 records, the plan (configuration, runs and seeds), the supervisor's ledger |
| `threshold.tsv` | the threshold and the rule's terms, as §§13.21 and 13.29 declare; the file the validation is bound to |
| `environment.txt`, `ind-cal6.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `check.exit`, `threshold.exit` | the precision check with the audit; the threshold reading's exit code |
| `rank_check.py`, `rank-check.txt` | the threshold recomputed from the records apart from `sim/a2_probe.py`: the stage and size, per case the values around the rank and the ties, `t` and its bits |
| `cost_check.py`, `cost-check.txt` | the measured cost from the ledger and records, per case |
