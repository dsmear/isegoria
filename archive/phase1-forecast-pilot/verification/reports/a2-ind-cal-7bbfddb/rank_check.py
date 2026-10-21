"""Recomputes the calibration's threshold from the records alone, apart from `sim/a2_probe.py`
(`docs/19` §13.22): per case the evaluable values, rank, threshold, neighbours and ties."""
import json
import sys

recs = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
ts = {}
for case in ("C0", "C1"):
    rows = sorted((r for r in recs if r["case"] == case), key=lambda r: r["replicate"])
    values, lost = [], {}
    for r in rows:
        shared = [c for c in r["candidates"] if c["classes"] == 2 and not c["non_uniform"]]
        why = (f"outcome {r['outcome']}" if r["outcome"] != "read"
               else f"baseline {r['candidates'][0]['status']}" if r["candidates"][0]["status"] != "converged"
               else "candidate missing" if not shared
               else f"candidate {shared[0]['status']}" if shared[0]["status"] != "converged"
               else "DTF unavailable" if shared[0]["dtf"] is None else None)
        if why:
            lost.setdefault(why, []).append(r["replicate"])
        else:
            values.append((shared[0]["dtf"], r["replicate"]))
    n, s = len(values), sorted(values)
    k = -(-19 * (n + 1) // 20)
    t = ts[case] = s[k - 1][0]
    print(f"{case}: planned 100, observed {len(rows)}, evaluable {n}, not evaluable {lost}")
    print(f"  zeros {sum(v == 0 for v, _ in values)}; k = {k}; t_{case} = {t!r} (r{s[k - 1][1]})")
    for i in range(max(0, k - 4), min(n, k + 3)):
        print(f"  rank {i + 1}: {s[i][0]!r} (r{s[i][1]})")
    print(f"  equal to t_{case}: {sum(v == t for v, _ in values)}; above: {sum(v > t for v, _ in values)}")
print(f"t = {max(ts.values())!r}")
