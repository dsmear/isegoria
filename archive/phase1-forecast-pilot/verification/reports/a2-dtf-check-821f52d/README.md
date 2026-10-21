# A2's DTF check on independent seeds, on `821f52d`

The raw records, plan, ledger, checks and reports of the descriptive check proposed in
[`docs/19` §13.17](../../../docs/19-a2-dtf-composition-design.md#1317-next-check-the-candidates-dtf-on-independent-seeds-proposal-not-run)
and declared in
[§13.18](../../../docs/19-a2-dtf-composition-design.md#1318-the-check-on-independent-seeds-implementation-and-declaration-not-run);
its results are stated in
[§13.19](../../../docs/19-a2-dtf-composition-design.md#1319-the-check-on-independent-seeds-result-run-on-821f52d).
C0 and C3, 16 replicates each, paired by seed under the tag `dtf-check`; no earlier evidence was
written to.

| | |
|---|---|
| Candidate | `821f52d` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `0d0729d`, `HEAD:Cargo.lock` `e661541`, `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `0b48ad5f…b7671433` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-19, from 12:56 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 32 of 32 `read`; no stop, refusal, killed or failed run |
| Caps | CPU 30 min, wall 15 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-dtf-check.tsv`) |
| Cost | 645.31 s CPU (10.8 min), 91.1 s wall; largest sampled resident sum 53.3 MiB (54,544 KiB), each run's peak 6.2–6.5 MiB; largest gap between checks 155 ms. CPU a run: C0 20.5 s mean (10.7–28.3), C3 19.9 (14.8–29.2) |
| Run directory | `characterization-results/a2-dtf-check-821f52d/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase dtf-check --jobs 8 --max-cpu-s 1800 --max-wall-s 900 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (416 values, largest `\|A − B\|` 1.7·10⁻¹⁵) |
| audit of phase `dtf-check` (`audit.json`) and the declared phase's contract | complete: 32 of 32, no duplicate, missing, failed or killed run; no contract reason |
| `python3 sim/a2_probe.py dtf-check D` | 0 |
| `python3 sim/a2_probe.py dtf-check D --json` | 0 |

| File | Contents |
|---|---|
| `records.jsonl` | the 32 records: the first probe's fields, then `candidates` |
| `ledger.tsv`, `plan-dtf-check.tsv` | the supervisor's ledger and the plan (configuration and runs with seeds) |
| `environment.txt`, `dtf-check.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `check.exit`, `audit.json` | the precision check with its audit, and the audit alone |
| `dtf-check-report.md`, `dtf-check.json` and their `.exit` | the declared analysis, as text and as JSON |
