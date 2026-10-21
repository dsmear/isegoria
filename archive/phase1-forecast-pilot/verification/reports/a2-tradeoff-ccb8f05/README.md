# A2's single-test trade-off, post hoc and exploratory, on `ccb8f05`

An exploratory reading, after the validation and its review, of the frozen indicator's empirical
detection and false-alarm curves over its threshold, stated in
[`docs/19` §13.24](../../../docs/19-a2-dtf-composition-design.md#1324-the-single-test-trade-off-an-exploratory-reading-post-hoc-on-ccb8f05).
It reads the validation's preserved records, [`../a2-ind-val-185ec93/`](../a2-ind-val-185ec93/README.md),
and locates the frozen threshold of [`../a2-ind-cal-7bbfddb/`](../a2-ind-cal-7bbfddb/README.md);
both are unchanged. No fit, no run, and no threshold chosen. Reading the validation over every
threshold makes it development material: a rule suggested here would need new data.

**Commands and exit codes** (from the repository root; `R` =
`verification/reports/a2-ind-val-185ec93/records.jsonl`, `T` =
`verification/reports/a2-ind-cal-7bbfddb/threshold.tsv`):

| Command | Exit |
|---|---|
| `python3 sim/a2_tradeoff.py R T > tradeoff.md` | 0 |
| `python3 sim/a2_tradeoff.py R T --json > tradeoff.json` | 0 |
| `python3 -m unittest discover -s sim -p test_a2_tradeoff.py` | 0 (5 tests) |

Both outputs reproduce byte for byte on a second run.

| File | Contents |
|---|---|
| `tradeoff.md` | per case the denominators; per comparison the frozen point, the point at `t = 0`, the area and the most detections within a false-alarm count |
| `tradeoff.json` | the same, with every point of the four curves |
| `tradeoff.exit`, `tradeoff-json.exit` | the exit codes |
