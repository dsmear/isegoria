# A2's study at `N = 6,000`: validation, on `6d57cac`

The raw records, plan, ledger, checks and readings of the validation stage `ind-val6` declared in
[`docs/19` §13.28](../../../docs/19-a2-dtf-composition-design.md#1328-next-study-calibration-and-validation-at-n--6000-proposal-not-implemented)
and implemented in
[§13.29](../../../docs/19-a2-dtf-composition-design.md#1329-the-study-at-n--6000-implementation-and-declaration-not-run);
its result is stated in
[§13.31](../../../docs/19-a2-dtf-composition-design.md#1331-the-study-at-n--6000-validation-run-on-6d57cac).
It ran against the calibration frozen in
[`../a2-ind-cal6-2a01b81/`](../a2-ind-cal6-2a01b81/README.md), which it did not write to:
`t = 0.641178985817746`, bits `3fe48489cadf0783`.

| | |
|---|---|
| Candidate | `6d57cac` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `dfd81c1` and `HEAD:Cargo.lock` `e661541`, the calibration's; `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `eff363fb…da3445eb`, the calibration's binary (`environment.txt`, `ledger.tsv`) |
| Calibration | `verification/reports/a2-ind-cal6-2a01b81/`, committed; its 17 files' SHA-256 the same before and after the run (`calibration-before.sha256`, `calibration-after.sha256`); `threshold.tsv` SHA-256 `ca93135a…e7b9`, the plan's `calibration` line |
| Date | 2026-10-20, from 19:17 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 400 of 400 `read` (C0, C1, C3 and C5, 100 each, `N = 6,000`, 60 anchors); no stop, refusal, killed or failed run |
| Caps | CPU 7.5 h, wall 90 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-ind-val6.tsv`) |
| Cost | 16,708.19 s CPU (278.5 min), 2,113.4 s wall (35.2 min); largest sampled resident sum 81.7 MiB (83,708 KiB), each run's peak 9.6–10.4 MiB; largest gap between checks 145 ms. CPU a run: C0 40.5 s mean (17.6–64.2), C1 44.1 (16.5–70.0), C3 38.9 (14.7–64.7), C5 43.5 (21.3–72.0); with the calibration 25,348.18 s CPU and 53.7 min wall (`cost-check.txt`) |
| Run directory | `characterization-results/a2-ind-val6-6d57cac/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory, `F` the calibration,
`V` this one):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase ind-val6 --calibration F --jobs 8 --max-cpu-s 27000 --max-wall-s 5400 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (5,200 values, largest `\|A − B\|` 4.6·10⁻¹⁴; the audit complete, 400 of 400) |
| `python3 sim/a2_probe.py ind-val D F --phase ind-val6` | 0, nothing on standard error |
| `python3 sim/a2_probe.py ind-val D F --phase ind-val6 --json` | 0, nothing on standard error |
| the three above on `V`, compared with `check.json`, `ind-val-report.md` and `ind-val.json` | 0, identical |
| `python3 val_check.py records.jsonl plan-ind-val6.tsv F ind-val.json > val-check.txt` (here) | 0 |
| `python3 cost_check.py . F > cost-check.txt` (here) | 0 |

| File | Contents |
|---|---|
| `records.jsonl`, `plan-ind-val6.tsv`, `ledger.tsv` | the 400 records, the plan (configuration with the `calibration` line, runs and seeds), the supervisor's ledger |
| `environment.txt`, `ind-val6.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, calibration, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `calibration-before.sha256`, `calibration-after.sha256` | the calibration's files' digests before and after the run |
| `check.json`, `check.exit` | the precision check with the audit |
| `ind-val-report.md`, `ind-val.json` and their `.exit` | the declared reading, as text and as JSON |
| `val_check.py`, `val-check.txt` | apart from `sim/a2_probe.py`: `t` recomputed from the calibration's records, the binding, plan and records checked, the reading recomputed, intervals by SciPy 1.18.1 |
| `cost_check.py`, `cost-check.txt` | the measured cost from the ledger and records, per case, and with the calibration's |
