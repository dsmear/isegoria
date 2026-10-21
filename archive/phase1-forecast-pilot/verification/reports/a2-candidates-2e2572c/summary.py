"""Per case and two-class variant, from the candidates records alone (`docs/19` §13.15): counts,
ranges, and the signs of the gain paired with C0 where both fits converged. `python3 summary.py DIR SIM`."""
import json
import sys

sys.path.insert(0, sys.argv[2])
import a2_probe as P  # noqa: E402

recs = {(r["case"], r["replicate"]): r for r in P.read_records(sys.argv[1])[0] if r["phase"] == "candidates"}
rng = lambda v: [round(min(v), 4), round(max(v), 4)] if v else None
out = {}
for case in ("C0", "C3", "C5"):
    runs = [recs[(case, k)] for k in range(8)]
    case_out = {"runs": len(runs), "baseline_converged": sum(r["candidates"][0]["status"] == "converged" for r in runs),
                "generating_dtf": rng([r["truth"]["dtf"] for r in runs])}
    for slopes in (False, True):
        cands = [next(c for c in r["candidates"] if c["classes"] == 2 and c["non_uniform"] == slopes) for r in runs]
        comp = [c for c in cands if P.comparable(c)]
        two = [c for c in comp if c["counted"] >= 2]
        v = {"fitted": len(cands), "converged": sum(c["status"] == "converged" for c in cands), "comparable": len(comp),
             "verdicts": sorted({c["verdict"] for c in cands}),
             "gain": rng([c["gain"] for c in comp]), "penalty": rng([c["penalty"] for c in comp]),
             "delta_bic": rng([c["delta_bic"] for c in comp]),
             "two_counted": len(two), "dtf_two_counted": rng([c["dtf"] for c in two if c["dtf"] is not None]),
             "dtf_unavailable": sum(c["dtf"] is None for c in cands)}
        if case != "C0":
            diffs = []
            for k in range(8):
                c = next(x for x in recs[(case, k)]["candidates"] if x["classes"] == 2 and x["non_uniform"] == slopes)
                c0 = next(x for x in recs[("C0", k)]["candidates"] if x["classes"] == 2 and x["non_uniform"] == slopes)
                if P.comparable(c) and P.comparable(c0):
                    diffs.append(round(c["gain"] - c0["gain"], 2))
            v["paired_gain_minus_c0"] = diffs
            v["paired_positive"] = sum(d > 0 for d in diffs)
        case_out["per-class" if slopes else "shared"] = v
    out[case] = case_out
print(json.dumps(out, indent=1))
