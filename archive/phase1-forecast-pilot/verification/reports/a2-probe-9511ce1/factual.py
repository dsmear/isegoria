"""Factual-knowledge reading of docs/19 §13.11 for C2 and C4 under S1, from the records only:
per run, the contested facts, what the balancing drew, and the generating DTF of the form
with every sourced fact kept (A ∪ C) against the forms the selector produced. Method B."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[0]))
sys.path.insert(0, sys.argv[2])
import a2_probe as P  # noqa: E402

d, phase = sys.argv[1], sys.argv[3]
recs, _ = P.read_records(d)
out = []
for r in sorted((r for r in recs if r["phase"] == phase and r["case"] in ("C2", "C4")),
                key=lambda r: (r["case"], r["replicate"])):
    row = {"key": r["key"], "outcome": r["outcome"]}
    if r["outcome"] == "read":
        it = P.of_record(r)
        s1 = next(x for x in r["readings"] if x["source"] == "S1")
        kept = sorted(s1["active"]["members"] + s1["contested"])
        row.update(roles=r["items"]["roles"], rechecks=r["rechecks"], contested=s1["contested"],
                   draw=s1["draw"], cost=s1["cost"],
                   selection=s1["selection"]["members"] if s1["selection"] else None,
                   dtf_active=s1["active"]["dtf"], dtf_composed=s1["composed"]["dtf"],
                   size_composed=s1["composed"]["size"], members_kept=kept,
                   dtf_all_sourced_kept=P.dtf(it, kept), dtf_fixed=r["truth"]["dtf"])
    out.append(row)
print(json.dumps(out, indent=1))
