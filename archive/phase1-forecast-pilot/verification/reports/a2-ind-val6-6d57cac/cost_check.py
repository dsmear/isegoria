"""The validation's measured cost from its ledger and records (`docs/19` §13.31): CPU per run by
case, each run's peak resident set, the sessions' totals, and with the calibration's."""
import json
import sys
from pathlib import Path


def ledger(d):
    ends, sessions = {}, []
    for line in (d / "ledger.tsv").read_text(encoding="utf-8").splitlines()[1:]:
        f = line.split("\t")
        if f[0] == "end":
            ends[f[1]] = (f[2], int(f[3]), int(f[4]))
        elif f[0] == "session":
            sessions.append(f)
    return ends, sessions


d, cal = Path(sys.argv[1]), Path(sys.argv[2])
ends, sessions = ledger(d)
recs = [json.loads(line) for line in (d / "records.jsonl").read_text(encoding="utf-8").splitlines() if line.strip()]
for case in ("C0", "C1", "C3", "C5"):
    cpu = [ends[r["key"]][1] / 1000 for r in recs if r["case"] == case]
    peak = [r["cost"]["hwm_kib"] / 1024 for r in recs if r["case"] == case]
    print(f"{case}: {len(cpu)} runs, CPU {sum(cpu):.1f} s, a run {sum(cpu) / len(cpu):.1f} s mean "
          f"({min(cpu):.1f}–{max(cpu):.1f}); each run's peak {min(peak):.1f}–{max(peak):.1f} MiB")
runs, own = sum(v[1] for v in ends.values()), sum(int(s[2]) for s in sessions)
wall = sum(int(s[1]) for s in sessions)
print(f"runs' CPU {runs / 1000:.2f} s; supervisor's own {own / 1000:.2f} s; total {(runs + own) / 1000:.2f} s")
print(f"sessions: wall {[int(s[1]) / 1000 for s in sessions]} s; largest sampled resident sum "
      f"{[int(s[3]) for s in sessions]} KiB; largest gap between checks {[int(s[4]) for s in sessions]} ms; "
      f"stop {[s[5] for s in sessions]}")
c_ends, c_sessions = ledger(cal)
c_cpu = sum(v[1] for v in c_ends.values()) + sum(int(s[2]) for s in c_sessions)
c_wall = sum(int(s[1]) for s in c_sessions)
print(f"with the calibration: CPU {(runs + own + c_cpu) / 1000:.2f} s, wall {(wall + c_wall) / 1000:.1f} s")
