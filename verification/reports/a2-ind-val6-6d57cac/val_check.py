"""Recomputes the validation at N = 6,000 from its records, plan and frozen calibration, apart
from `sim/a2_probe.py`, intervals by SciPy (`docs/19` §13.31); compares with `ind-val.json`."""
import hashlib
import json
import struct
import sys
from pathlib import Path

from scipy.stats import beta

records, plan, cal, reading = sys.argv[1], Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4]
CASES, NEUTRAL = ("C0", "C1", "C3", "C5"), ("C0", "C1")


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path):
    return [json.loads(line) for line in open(path, encoding="utf-8") if line.strip()]


def config(path):
    lines = Path(path).read_text(encoding="utf-8").splitlines()
    return [l for l in lines[:next(i for i, l in enumerate(lines) if l.startswith("phase\t"))]
            if not l.startswith(("caps\t", "calibration\t"))]


def indicator(r):
    """`I`, or the reason with the observed status, as §13.21's evaluability rule reads a run."""
    shared = [c for c in r.get("candidates", []) if c["classes"] == 2 and not c["non_uniform"]]
    why = (("gate refused" if r["outcome"] == "refused" else f"run {r['outcome']}", r["outcome"])
           if r["outcome"] != "read"
           else ("baseline not converged", r["candidates"][0]["status"])
           if r["candidates"][0]["status"] != "converged"
           else ("candidate missing", "-") if not shared
           else ("candidate not converged", shared[0]["status"]) if shared[0]["status"] != "converged"
           else ("DTF unavailable", "-") if shared[0]["dtf"] is None else None)
    return (None, why) if why else (shared[0]["dtf"], None)


lines = (cal / "threshold.tsv").read_text(encoding="utf-8").splitlines()
assert lines[:2] == ["schema\ta2-ind/v2", "stage\tind-cal6\tn\t6000"], "not ind-cal6's threshold"
assert sorted(p.name for p in cal.glob("plan-*.tsv")) == ["plan-ind-cal6.tsv"], "not ind-cal6's plans"
for name, digest in (l.split("\t")[1:] for l in lines if l.startswith("file\t")):
    assert sha256(cal / name) == digest, f"{name} is not the file the threshold names"
bits = next(l.split("\t")[1] for l in lines if l.startswith("threshold\t"))
t = struct.unpack(">d", bytes.fromhex(bits))[0]
ts = []
for case in NEUTRAL:
    vals = sorted(v for v, _ in map(indicator, (r for r in load(cal / "records.jsonl") if r["case"] == case))
                  if v is not None)
    ts.append(vals[-(-19 * (len(vals) + 1) // 20) - 1])
assert struct.pack(">d", max(ts)).hex() == bits, "t is not the calibration's"
print(f"t = {t!r} ({bits}), recomputed from the calibration's records; threshold.tsv {sha256(cal / 'threshold.tsv')}")

text = plan.read_text(encoding="utf-8").splitlines()
assert f"calibration\t{sha256(cal / 'threshold.tsv')}\t{bits}" in text, "the plan is not bound to t"
assert config(plan) == config(cal / "plan-ind-cal6.tsv"), "the two stages' configurations differ"
designs = [l.split("\t") for l in config(plan) if l.startswith("design\t")]
assert len(designs) == 6 and all(d[2] == "6000" and d[3] == "60" for d in designs), "designs not at 6,000"
tasks = {f[1]: int(f[2]) for f in (l.split("\t") for l in text) if f[0] == "task"}
keys = {f"ind-val6-{c}-{i}" for c in CASES for i in range(100)}
recs = load(records)
assert set(tasks) == keys and len(recs) == 400 and {r["key"] for r in recs} == keys, "not the 400 runs"
assert all(r["phase"] == "ind-val6" and r["design"]["n"] == 6000 and r["design"]["anchors"] == 60
           and r["key"] == f"ind-val6-{r['case']}-{r['replicate']}" and r["seed"] == tasks[r["key"]]
           for r in recs), "not ind-val6 at 6,000 with the plan's seeds"
print("plan: bound to t; configuration the calibration's but for caps; six designs at 6,000; "
      "400 tasks, records matching key, phase, N and seed")

got = json.load(open(reading, encoding="utf-8"))
assert got["threshold"] == {"bits": bits, "value": t}


def cp(k, n):
    return (0.0 if k == 0 else beta.ppf(0.025, k, n - k + 1), 1.0 if k == n else beta.ppf(0.975, k + 1, n - k))


values, worst = {}, 0.0
for case in CASES:
    rows = sorted((r for r in recs if r["case"] == case), key=lambda r: r["replicate"])
    seen = [indicator(r) for r in rows]
    vals = values[case] = [v for v, _ in seen if v is not None]
    lost = {}
    for r, (_, why) in zip(rows, seen):
        if why:
            lost.setdefault(why[0], []).append(f"r{r['replicate']} {why[1]}")
    n, k = len(vals), sum(v > t for v in vals)
    mine = {"evaluable": n, "flagged": k, "not_flagged": n - k, "zeros": sum(v == 0 for v in vals),
            "not_evaluable": {w: len(x) for w, x in sorted(lost.items())}}
    theirs = {x: got["cases"][case][x] for x in mine}
    for ours, name in ((cp(k, n), "flagged_over_evaluable"), (cp(k, 100), "flagged_over_planned"),
                       (cp(100 - n, 100), "not_evaluable_over_planned")):
        worst = max(worst, *(abs(a - b) for a, b in zip(ours, got["cases"][case][name]["ci95"])))
        print(f"  {case} {name}: [{ours[0]:.3f}, {ours[1]:.3f}]")
    print(f"{case}: {mine}; equal to the reading: {mine == theirs}; values equal to the reading's: "
          f"{[v for v, _ in seen] == got['cases'][case]['values']}; equal to t: {sum(v == t for v in vals)}")
    print(f"  not evaluable: {lost}")
for hi in ("C3", "C5"):
    for lo in NEUTRAL:
        share = sum((a > b) + 0.5 * (a == b) for a in values[hi] for b in values[lo]) / (len(values[hi]) * len(values[lo]))
        print(f"{hi} against {lo}: {share!r} over {len(values[hi])} × {len(values[lo])} pairs; "
              f"equal to the reading: {share == got['separation'][hi + '>' + lo]}")
print(f"largest |interval − SciPy|: {worst:.1e}")
