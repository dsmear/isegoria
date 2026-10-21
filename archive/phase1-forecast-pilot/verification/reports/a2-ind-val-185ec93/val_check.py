"""Recomputes the validation reading from the records and the frozen threshold, apart from
`sim/a2_probe.py`, intervals by SciPy (`docs/19` §13.23); compares with `ind-val.json`."""
import json
import struct
import sys

from scipy.stats import beta

records, threshold, reading = sys.argv[1:4]
bits = next(l.split("\t")[1] for l in open(threshold, encoding="utf-8") if l.startswith("threshold\t"))
t = struct.unpack("<d", bytes.fromhex(bits)[::-1])[0]
recs = [json.loads(line) for line in open(records, encoding="utf-8") if line.strip()]
got = json.load(open(reading, encoding="utf-8"))


def cp(k, n):
    return (0.0 if k == 0 else beta.ppf(0.025, k, n - k + 1), 1.0 if k == n else beta.ppf(0.975, k + 1, n - k))


values, worst = {}, 0.0
print(f"t = {t!r} ({bits})")
for case in ("C0", "C1", "C3", "C5"):
    vals, lost = [], {}
    for r in sorted((r for r in recs if r["case"] == case), key=lambda r: r["replicate"]):
        shared = [c for c in r["candidates"] if c["classes"] == 2 and not c["non_uniform"]]
        why = (f"outcome {r['outcome']}" if r["outcome"] != "read"
               else "baseline not converged" if r["candidates"][0]["status"] != "converged"
               else "candidate missing" if not shared
               else "candidate not converged" if shared[0]["status"] != "converged"
               else "DTF unavailable" if shared[0]["dtf"] is None else None)
        if why:
            lost[why] = lost.get(why, 0) + 1
        else:
            vals.append(shared[0]["dtf"])
    values[case], n, k = vals, len(vals), sum(v > t for v in vals)
    mine = {"evaluable": n, "flagged": k, "zeros": sum(v == 0 for v in vals), "not_evaluable": lost}
    theirs = {x: got["cases"][case][x] for x in mine}
    for ours, name, m in ((cp(k, n), "flagged_over_evaluable", n), (cp(k, 100), "flagged_over_planned", 100),
                          (cp(100 - n, 100), "not_evaluable_over_planned", 100)):
        worst = max(worst, *(abs(a - b) for a, b in zip(ours, got["cases"][case][name]["ci95"])))
    print(f"{case}: {mine}; equal to the reading: {mine == theirs}; equal to t: {sum(v == t for v in vals)}")
for hi in ("C3", "C5"):
    for lo in ("C0", "C1"):
        share = sum((a > b) + 0.5 * (a == b) for a in values[hi] for b in values[lo]) / (len(values[hi]) * len(values[lo]))
        print(f"{hi} against {lo}: {share!r}; equal to the reading: {share == got['separation'][hi + '>' + lo]}")
print(f"largest |interval − SciPy|: {worst:.1e}")
