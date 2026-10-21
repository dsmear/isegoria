"""Post-hoc exploratory re-reading (`docs/19` §13.16), from the preserved records alone: per two-class
variant and replicate, C3's and C5's candidate whole-form DTF beside C0's, with baselines, counted
classes and zeros. `python3 paired_dtf.py RECORDS.jsonl > paired-dtf.json`."""
import json
import sys
from pathlib import Path

recs = {}
for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines():
    if line.strip():
        r = json.loads(line)
        recs[(r["case"], r["replicate"])] = r


def cell(case, rep, slopes):
    r = recs[(case, rep)]
    base = r["candidates"][0]
    c = next(x for x in r["candidates"] if x["classes"] == 2 and x["non_uniform"] == slopes)
    return {"baseline": base["status"], "candidate": c["status"], "counted": c["counted"], "dtf": c["dtf"],
            "dtf_basis": c["dtf_basis"], "generating": r["truth"]["dtf"]}


def ok(x):
    return x["baseline"] == "converged" and x["candidate"] == "converged"


out = {"note": "post-hoc, exploratory; both variants refit the same batches, so they are not independent replicates"}
for slopes in (False, True):
    name = "per-class" if slopes else "shared"
    rows, tally = [], {}
    for case in ("C3", "C5"):
        greater = smaller = equal = comparable = 0
        for rep in range(8):
            a, b = cell(case, rep, slopes), cell("C0", rep, slopes)
            pair = ok(a) and ok(b)
            order = None
            if pair and a["dtf"] is not None and b["dtf"] is not None:
                comparable += 1
                order = "greater" if a["dtf"] > b["dtf"] else ("smaller" if a["dtf"] < b["dtf"] else "equal")
                greater += order == "greater"
                smaller += order == "smaller"
                equal += order == "equal"
            rows.append({"case": case, "replicate": rep, "comparable": pair, "order_vs_c0": order,
                         "case_cell": a, "c0_cell": b})
        tally[case] = {"comparable": comparable, "greater": greater, "smaller": smaller, "equal": equal}
    out[name] = {"pairs": rows, "tally": tally}
print(json.dumps(out, indent=1))
