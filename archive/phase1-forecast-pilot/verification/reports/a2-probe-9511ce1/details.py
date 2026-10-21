"""Per-run details of the A2 probe's confirmation (`docs/19` §13.12) from its records alone:
fit, readings, draw, cost and forms. `python3 details.py RUN_DIR SIM_DIR > confirm-details.json`."""
import json
import sys

sys.path.insert(0, sys.argv[2])
import a2_probe as P  # noqa: E402

recs = [r for r in P.read_records(sys.argv[1])[0] if r["phase"] == "confirm"]
lines = []
for r in sorted(recs, key=lambda r: (r["case"], r["replicate"])):
    f, s0, s1 = r["fit"], r["readings"][0], r["readings"][1]
    sel = s1["selection"]
    lines.append({"key": r["key"], "status": f["status"], "classes": f["classes"], "counted": f["counted"],
                  "bic_gain": round(f["bic_gain"], 2), "rechecks": r["rechecks"], "roles": r["items"]["roles"],
                  "dif": [round(x, 2) for x in f["dif"]],
                  "fixed_dtf": round(r["truth"]["dtf"], 4), "fixed_fitted": r["fixed"]["fitted"],
                  "S0_active": (s0["active"]["size"], s0["active"]["completeness"], round(s0["active"]["dtf"], 4)),
                  "S1_draw": s1["draw"], "S1_cost": s1["cost"],
                  "S1_selection": (sel["members"], round(sel["dtf"], 4)) if sel else None,
                  "S1_composed": (s1["composed"]["size"], s1["composed"]["completeness"],
                                  round(s1["composed"]["dtf"], 4)),
                  "cpu_ms": r["cost"]["cpu_ms"], "hwm_kib": r["cost"]["hwm_kib"], "candidates": f["candidates"]})
print(json.dumps(lines, indent=1))
