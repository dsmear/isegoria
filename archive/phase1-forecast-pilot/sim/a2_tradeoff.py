"""A2's single-test trade-off, post hoc and exploratory (`docs/19` §13.24): empirical detection and
false-alarm curves of the frozen indicator over its threshold, per comparison, from preserved
records. `python3 sim/a2_tradeoff.py RECORDS THRESHOLD [--json]`; standard library only."""
import json
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import a2_probe as P  # noqa: E402

COMPARISONS = (("C3", "C0"), ("C3", "C1"), ("C5", "C0"), ("C5", "C1"))
FALSE_ALARMS = (0, 1, 2, 5, 10, 20, 50)


def curve(pos, neg):
    """The points `(t, detections, false alarms)` of the rule `I > t`, `t` from the largest value
    down through every distinct value, then `None`: below every value, all flagged."""
    points = [(t, sum(v > t for v in pos), sum(v > t for v in neg)) for t in sorted(set(pos) | set(neg), reverse=True)]
    return points + [(None, len(pos), len(neg))]


def area(points, n_pos, n_neg):
    """The trapezoidal area under the curve, ties as diagonal steps."""
    out, (d0, f0) = 0.0, (0, 0)
    for _, d, f in points:
        out += (f - f0) * (d + d0) / 2
        d0, f0 = d, f
    return out / (n_pos * n_neg)


def at_most(points, k):
    """The largest detection count among the points with at most `k` false alarms, and the
    fewest false alarms reaching it."""
    best = max((d for _, d, f in points if f <= k), default=0)
    fa = min(f for _, d, f in points if f <= k and d == best)
    return best, fa


def evaluable(recs, case):
    seen = [P.indicator(r) for r in sorted((r for r in recs if r["case"] == case), key=lambda r: r["replicate"])]
    lost = {w: sum(x == w for _, x in seen) for w in sorted({w for _, w in seen if w})}
    return [v for v, _ in seen if v is not None], lost, len(seen)


def tradeoff(records, threshold):
    recs = [json.loads(line) for line in Path(records).read_text(encoding="utf-8").splitlines() if line.strip()]
    bits = next(l.split("\t")[1] for l in Path(threshold).read_text(encoding="utf-8").splitlines()
                if l.startswith("threshold\t"))
    t = struct.unpack("<d", struct.pack("<Q", int(bits, 16)))[0]
    cases = {c: evaluable(recs, c) for c in ("C0", "C1", "C3", "C5")}
    out = {"threshold": {"bits": bits, "value": t}, "cases": {}, "comparisons": {}}
    for c, (vals, lost, planned) in cases.items():
        out["cases"][c] = {"planned": planned, "evaluable": len(vals), "not_evaluable": lost,
                           "zeros": sum(v == 0 for v in vals)}
    for hi, lo in COMPARISONS:
        pos, neg = cases[hi][0], cases[lo][0]
        points = curve(pos, neg)
        out["comparisons"][f"{hi}/{lo}"] = {
            "detections_of": len(pos), "false_alarms_of": len(neg),
            "frozen": {"detections": sum(v > t for v in pos), "false_alarms": sum(v > t for v in neg)},
            "area": area(points, len(pos), len(neg)), "separation": P.separation(pos, neg),
            "at_most": {k: at_most(points, k) for k in FALSE_ALARMS},
            "points": [{"t": p[0], "detections": p[1], "false_alarms": p[2]} for p in points]}
    return out


def report(out):
    t = out["threshold"]
    lines = ["# A2 single-test trade-off (§13.24), post hoc and exploratory", "",
             f"Frozen `t` = {t['value']!r} ({t['bits']}); flagged when `I > t`. Validation runs used as "
             "development material: no threshold is chosen here.", "", "| Case | Planned | Evaluable | "
             "Not evaluable | Zeros of `I` |", "|---|---|---|---|---|"]
    for c, x in out["cases"].items():
        lines.append(f"| {c} | {x['planned']} | {x['evaluable']} | {x['not_evaluable'] or 0} | {x['zeros']} |")
    for name, x in out["comparisons"].items():
        n_pos, n_neg = x["detections_of"], x["false_alarms_of"]
        lines += ["", f"## {name}: detections of {n_pos} evaluable, false alarms of {n_neg}", "",
                  f"- frozen `t`: {x['frozen']['detections']}/{n_pos} detections, {x['frozen']['false_alarms']}/{n_neg} "
                  f"false alarms; area {x['area']!r} (the separation, {x['separation']!r})",
                  f"- {len(x['points'])} points; at `t = 0`, zeros unflagged: "
                  + next(f"{p['detections']}/{n_pos}, {p['false_alarms']}/{n_neg}" for p in x["points"] if p["t"] == 0),
                  "", "| False alarms at most | Largest detections | At false alarms |", "|---|---|---|"]
        lines += [f"| {k}/{n_neg} | {d}/{n_pos} | {f}/{n_neg} |" for k, (d, f) in x["at_most"].items()]
    runs = ", ".join(f"{c} {x['evaluable']}" for c, x in out["cases"].items())
    lines += ["", f"The pairs behind each area are not independent observations: the runs are, {runs} evaluable."]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    result = tradeoff(sys.argv[1], sys.argv[2])
    print(json.dumps(result, indent=1) if "--json" in sys.argv[3:] else report(result), end="\n" if "--json" in sys.argv[3:] else "")
