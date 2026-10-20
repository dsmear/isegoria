"""Recomputes the calibration at N = 6,000 from its records alone, apart from `sim/a2_probe.py`
(`docs/19` §13.30): per case the evaluable values, rank, threshold, neighbours and ties; then t."""
import json
import struct
import sys

recs = [json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip()]
keys = {f"ind-cal6-{c}-{i}" for c in ("C0", "C1") for i in range(100)}
assert len(recs) == 200 and {r["key"] for r in recs} == keys, "not the 200 declared runs"
assert all(r["phase"] == "ind-cal6" and r["design"]["n"] == 6000 and r["design"]["anchors"] == 60
           and r["key"] == f"ind-cal6-{r['case']}-{r['replicate']}" for r in recs), "not ind-cal6 at 6,000"
ts = {}
for case in ("C0", "C1"):
    rows = sorted((r for r in recs if r["case"] == case), key=lambda r: r["replicate"])
    values, lost = [], {}
    for r in rows:
        shared = [c for c in r.get("candidates", []) if c["classes"] == 2 and not c["non_uniform"]]
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
    assert k <= n, f"{case}: {n} evaluable runs give no finite threshold"
    t = ts[case] = s[k - 1][0]
    print(f"{case}: planned 100, observed {len(rows)}, evaluable {n}, not evaluable {lost}")
    print(f"  zeros {sum(v == 0 for v, _ in values)}; k = {k}; t_{case} = {t!r} (r{s[k - 1][1]})")
    for i in range(max(0, k - 4), min(n, k + 3)):
        print(f"  rank {i + 1}: {s[i][0]!r} (r{s[i][1]})")
    print(f"  equal to t_{case}: {sum(v == t for v, _ in values)}; above: {sum(v > t for v, _ in values)}")
t = max(ts.values())
print(f"t = {t!r}, bits {struct.unpack('<Q', struct.pack('<d', t))[0]:016x}")
