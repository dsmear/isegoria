# A2's candidates' diagnostics, on `2e2572c`

The raw records, plan, ledger, checks and reports of the exploratory look declared in
[`docs/19` §13.14](../../../docs/19-a2-dtf-composition-design.md#1314-candidates-diagnostics-implementation-and-declaration-not-run);
its results are stated and read in
[`docs/19` §13.15](../../../docs/19-a2-dtf-composition-design.md#1315-candidates-diagnostics-result-run-on-2e2572c).
The 24 batches are the first probe's confirm batches of C0, C3 and C5, refitted on their seeds
(`../a2-probe-9511ce1/`, which this run did not write to).

| | |
|---|---|
| Candidate | `2e2572c` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `fa97c40`, `HEAD:Cargo.lock` `e661541`, `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/protocol` and the rest of `crates/scoring` identical to `6274270`; `a2-probe` SHA-256 `88b6387e…9ebf088` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-19, from 09:36 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 24 of 24 `read`; no stop, refusal, killed or failed run, no rejected part |
| Caps | CPU 30 min, wall 15 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-candidates.tsv`) |
| Cost | 599.44 s CPU (10.0 min), 90.2 s wall; largest sampled resident sum 53.2 MiB (54,480 KiB), each run's peak 6.2–6.5 MiB; largest gap between checks 150 ms. CPU a run: C0 24.8 s mean (13.6–35.6), C3 25.9 (18.4–32.5), C5 24.2 (14.7–30.5) |
| Run directory | `characterization-results/a2-candidates-2e2572c/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory, `F` =
`verification/reports/a2-probe-9511ce1/records.jsonl`):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase candidates --jobs 8 --max-cpu-s 1800 --max-wall-s 900 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (312 values, largest `\|A − B\|` 1.9·10⁻¹⁵) |
| audit of phase `candidates` (`audit.json`) | complete: 24 of 24, no duplicate, missing, failed or killed run |
| `python3 sim/a2_probe.py compare-first D F` | 0 (24 compared, no missing twin, no field differing) |
| `python3 sim/a2_probe.py candidates D F` | 0 (complete) |
| `python3 summary.py D sim` | 0 |

| File | Contents |
|---|---|
| `records.jsonl` | the 24 records: the first probe's fields, then `candidates` |
| `ledger.tsv`, `plan-candidates.tsv` | the supervisor's ledger and the plan (configuration and runs with seeds) |
| `environment.txt`, `candidates.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `audit.json`, `compare-first.json` | the precision check, the audit, the reproduction of the first probe |
| `candidates-report.md` | every candidate per run, and the two-class variants paired with C0 where comparable |
| `summary.json`, `summary.py` | per case and variant: counts, ranges and paired signs, from the records alone |
