# A2's sample-size study, on `b8c9dce`

The raw records, plan, ledger, checks and readings of the study declared in
[`docs/19` §13.25](../../../docs/19-a2-dtf-composition-design.md#1325-next-study-the-sample-size-and-single-test-separation-proposal-not-implemented)
and
[§13.26](../../../docs/19-a2-dtf-composition-design.md#1326-the-sample-size-study-implementation-and-declaration-not-run);
its results are stated in
[§13.27](../../../docs/19-a2-dtf-composition-design.md#1327-the-sample-size-study-result-run-on-b8c9dce).
C0 and C3 at `N` = 3,000 and 6,000, 16 replicates each, one seed per case and replicate shared by
the two sizes.

| | |
|---|---|
| Candidate | `b8c9dce` (= `origin/docs/phase1-review-alignment`); `HEAD:crates` `cddcb19`, `HEAD:Cargo.lock` `e661541`, `crates/scoring/src/latent.rs` `fe101c8`, clean; `crates/scoring` and `crates/protocol` identical to `2e2572c`; `a2-probe` SHA-256 `4692f732…4d842756` (`environment.txt`, `ledger.tsv`) |
| Date | 2026-10-19, from 19:13 (`environment.txt`) |
| Machine | the owner's: AMD Ryzen 7 5800X, 8 cores, 16 threads, 32 GB, Linux 7.1.5; rustc 1.86.0, Python 3.12.3 |
| Runs | 64 of 64 `read`; no stop, refusal, killed or failed run |
| Caps | CPU 2 h, wall 30 min, 10 min a run, 2 GiB aggregate resident, 8 workers (`plan-size.tsv`) |
| Cost | 1,938.74 s CPU (32.3 min), 262.4 s wall (4.4 min); largest sampled resident sum 76.2 MiB (78,028 KiB); largest gap between checks 145 ms. At 3,000: 680.4 s CPU for 32 runs, C0 21.4 s mean (10.9–34.8), C3 21.1 (12.5–27.8), each run's peak 6.3–6.5 MiB. At 6,000: 1,257.5 s for 32 runs, C0 40.0 s (22.5–56.3), C3 38.6 (16.9–58.0), each run's peak 9.7–10.0 MiB; per replicate, 6,000 against 3,000, a median 1.89 times the CPU (0.61–3.79) |
| Run directory | `characterization-results/a2-size-b8c9dce/` and its `-logs/`, kept by the owner, untracked; the files below are copies |

**Commands and exit codes** (from the repository root; `D` the run directory):

| Command | Exit |
|---|---|
| `cargo run --release -p characterization --bin a2-probe -- run --phase size --jobs 8 --max-cpu-s 7200 --max-wall-s 1800 --max-run-s 600 --max-rss-mib 2048 --out D` | 0 |
| `python3 sim/a2_probe.py check D` | 0 (832 values, largest `\|A − B\|` 1.1·10⁻¹⁴; the audit complete, 64 of 64) |
| `python3 sim/a2_size.py D` | 0 |
| `python3 sim/a2_size.py D --json` | 0 |
| `python3 size_check.py records.jsonl size.json > size-check.txt` (here) | 0 |

| File | Contents |
|---|---|
| `records.jsonl`, `plan-size.tsv`, `ledger.tsv` | the 64 records, the plan (configuration, runs with `n` and seeds), the supervisor's ledger |
| `environment.txt`, `size.cmd`, `.stderr`, `.exit`, `.time` | commit, machine, command, the supervisor's summary, exit code, `/usr/bin/time -v` |
| `check.json`, `check.exit` | the precision check with the audit |
| `size-report.md`, `size.json` and their `.exit` | the declared reading, as text and as JSON |
| `size_check.py`, `size-check.txt` | the reading recomputed from the records apart from `sim/`, with `I` per replicate at both sizes |
