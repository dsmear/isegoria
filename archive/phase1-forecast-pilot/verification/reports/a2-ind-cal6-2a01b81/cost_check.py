"""The calibration's measured cost from its ledger and records (`docs/19` §13.30): CPU per run by
case, each run's peak resident set, and the sessions' totals."""
import json
import sys
from pathlib import Path

d = Path(sys.argv[1])
ends, sessions = {}, []
for line in (d / "ledger.tsv").read_text(encoding="utf-8").splitlines()[1:]:
    f = line.split("\t")
    if f[0] == "end":
        ends[f[1]] = (f[2], int(f[3]), int(f[4]))
    elif f[0] == "session":
        sessions.append(f)
recs = [json.loads(line) for line in (d / "records.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
for case in ("C0", "C1"):
    cpu = [ends[r["key"]][1] / 1000 for r in recs if r["case"] == case]
    peak = [r["cost"]["hwm_kib"] / 1024 for r in recs if r["case"] == case]
    print(f"{case}: {len(cpu)} runs, CPU {sum(cpu):.1f} s, a run {sum(cpu) / len(cpu):.1f} s mean "
          f"({min(cpu):.1f}–{max(cpu):.1f}); each run's peak {min(peak):.1f}–{max(peak):.1f} MiB")
runs, own = sum(v[1] for v in ends.values()), sum(int(s[2]) for s in sessions)
print(f"runs' CPU {runs / 1000:.2f} s; supervisor's own {own / 1000:.2f} s; total {(runs + own) / 1000:.2f} s")
print(f"sessions: wall {[int(s[1]) / 1000 for s in sessions]} s; largest sampled resident sum "
      f"{[int(s[3]) for s in sessions]} KiB; largest gap between checks {[int(s[4]) for s in sessions]} ms; "
      f"stop {[s[5] for s in sessions]}")
