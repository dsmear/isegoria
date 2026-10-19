"""A2's sample-size study (`docs/19` §13.26): C3's single-test separation from C0 at N = 3,000 and
6,000 on nested samples, from a run directory. `python3 sim/a2_size.py DIR [--json]`; standard
library only, reusing `a2_probe.py`."""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import a2_probe as P  # noqa: E402

PHASE, CASES, SIZES, REPLICATES = "size", ("C0", "C3"), (3000, 6000), 16


def key(n, case, r):
    return f"{PHASE}-n{n}-{case}-{r}"


def alike(a, b):
    """Whether two runs of one case and replicate share their seed, items and generating DTF."""
    return (a["seed"], a["items"], a["truth"]["dtf"]) == (b["seed"], b["items"], b["truth"]["dtf"])


def reasons(d):
    """Why `d` does not hold §13.26's phase exactly: its replicates, its 64 tasks and seeds,
    records matching their keys and sizes, runs alike across sizes, a complete audit."""
    a = P.audit(d, PHASE)
    plan = P.read_plans(d).get(PHASE, {"replicates": None, "tasks": {}})
    tasks = plan["tasks"]
    want = {key(n, c, r): P.case_seed(PHASE, c, r) for n in SIZES for c in CASES for r in range(REPLICATES)}
    why = []
    if plan["replicates"] != REPLICATES:
        why.append(f"the plan declares {plan['replicates']} replicates, not {REPLICATES}")
    if set(tasks) != set(want):
        why.append(f"the plan's tasks are not the {len(want)} declared: missing {P.listed(set(want) - set(tasks))}, "
                   f"unexpected {P.listed(set(tasks) - set(want))}")
    wrong = [k for k in set(tasks) & set(want) if tasks[k] != want[k]]
    if wrong:
        why.append(f"seeds not the declared ones: {P.listed(wrong)}")
    recs = [r for r in P.read_records(d)[0] if r.get("phase") == PHASE or str(r.get("key")).startswith(PHASE + "-")]
    off = [str(r.get("key")) for r in recs if r.get("phase") != PHASE or r.get("key") not in want
           or r.get("key") != key(r.get("design", {}).get("n"), r.get("case"), r.get("replicate"))]
    if off:
        why.append(f"records not matching their key's phase, size, case and replicate: {P.listed(off)}")
    by = {r["key"]: r for r in recs if r.get("key") in want}
    unlike = [f"{c}-{r}" for c in CASES for r in range(REPLICATES)
              if all(key(n, c, r) in by for n in SIZES) and not alike(*(by[key(n, c, r)] for n in SIZES))]
    if unlike:
        why.append(f"runs not alike across the sizes (seed, items, generating DTF): {P.listed(unlike)}")
    if a["status"] != "complete":
        why.append(f"the audit is {a['status']}, {a['records']} records of {a['planned']} planned")
    return why


def counts(values):
    return {k: sum(v == k for v in values) for k in sorted(set(values))}


def ties(high, low):
    return sum(a == b for a in high for b in low)


def placement(x, low):
    return P.separation([x], low)


def size_reading(d):
    def stop(why):
        return "# A2 sample-size study (§13.26)\n\nStopped, not read: " + "; ".join(why) + ".\n", None, 1
    try:
        why = reasons(d)
    except ValueError as e:
        why = [str(e)]
    if why:
        return stop(why)
    recs = {r["key"]: r for r in P.read_records(d)[0] if r.get("phase") == PHASE}
    checked = P.precision(list(recs.values()))
    if checked["failures"] or not checked["values"]:
        return stop([f"the numerical check fails: {len(checked['failures'])} failures"])
    seen = {(n, c, r): P.indicator(recs[key(n, c, r)]) for n in SIZES for c in CASES for r in range(REPLICATES)}
    I = {k: v for k, (v, _) in seen.items()}
    out = {"sizes": {}, "all": {}, "common": {}, "replicates": {}, "placements": {}}
    lines = ["# A2 sample-size study (§13.26): C3 against C0 on a single test", ""]
    for n in SIZES:
        for c in CASES:
            rs = [recs[key(n, c, r)] for r in range(REPLICATES)]
            vals = [I[(n, c, r)] for r in range(REPLICATES) if I[(n, c, r)] is not None]
            zero = [next(x for x in rec["candidates"] if x["classes"] == 2 and not x["non_uniform"])["dtf_basis"]
                    for rec, v in zip(rs, (I[(n, c, r)] for r in range(REPLICATES))) if v == 0]
            out["sizes"][f"{n}/{c}"] = {
                "planned": REPLICATES, "observed": len(rs), "evaluable": len(vals),
                "not_evaluable": counts([seen[(n, c, r)][1] for r in range(REPLICATES) if seen[(n, c, r)][1]]),
                "zeros": len(zero), "zero_bases": sorted(set(zero)),
                "selected": counts([f"{x['fit']['classes']} {x['fit']['status']}" for x in rs if "fit" in x]),
                "shared": counts([f"{s['status']} {s['verdict']}" for x in rs if "candidates" in x
                                  for s in x["candidates"] if s["classes"] == 2 and not s["non_uniform"]])}
            x = out["sizes"][f"{n}/{c}"]
            lines.append(f"- N {n}, {c}: planned {REPLICATES}, observed {x['observed']}, evaluable {x['evaluable']}, "
                         f"not evaluable {x['not_evaluable'] or 0}; zeros {x['zeros']} {x['zero_bases']}; selected "
                         f"{x['selected']}; shared candidate {x['shared']}")
    common = {c: [r for r in range(REPLICATES) if all(I[(n, c, r)] is not None for n in SIZES)] for c in CASES}
    for name, reps in (("all", None), ("common", common)):
        lines += ["", f"## Separation, {'every evaluable run' if reps is None else 'replicates evaluable at both sizes'}", ""]
        for n in SIZES:
            hi = [I[(n, "C3", r)] for r in (reps["C3"] if reps else range(REPLICATES)) if I[(n, "C3", r)] is not None]
            lo = [I[(n, "C0", r)] for r in (reps["C0"] if reps else range(REPLICATES)) if I[(n, "C0", r)] is not None]
            share = P.separation(hi, lo)
            out[name][n] = {"separation": share, "pairs": len(hi) * len(lo), "tied_pairs": ties(hi, lo),
                            "C3": len(hi), "C0": len(lo)}
            lines.append(f"- N {n}: {share!r} over {len(hi)} × {len(lo)} pairs, {ties(hi, lo)} tied")
        both = [out[name][n]["separation"] for n in SIZES]
        out[name]["difference"] = None if None in both else both[1] - both[0]
        lines.append(f"- 6,000 minus 3,000: {out[name]['difference']!r}"
                     + (", conditional on the common replicates" if reps else ""))
    lines += ["", "## Per replicate, evaluable at both sizes", ""]
    for c in CASES:
        moves = {"greater": 0, "smaller": 0, "equal": 0}
        zeros = {}
        for r in common[c]:
            a, b = I[(3000, c, r)], I[(6000, c, r)]
            moves[P.order(b, a)] += 1
            t = f"{'zero' if a == 0 else 'positive'}→{'zero' if b == 0 else 'positive'}"
            zeros[t] = zeros.get(t, 0) + 1
        one = {r: [n for n in SIZES if I[(n, c, r)] is not None] for r in range(REPLICATES) if r not in common[c]}
        out["replicates"][c] = {"common": common[c], "I_6000_against_3000": moves, "zeros": zeros,
                                "evaluable_at_one_size": {r: ns for r, ns in one.items() if ns}}
        lines.append(f"- {c}: {len(common[c])} replicates; I at 6,000 against 3,000 {moves}; zeros {zeros}; "
                     f"evaluable at one size only {out['replicates'][c]['evaluable_at_one_size'] or 'none'}")
    for name, against in (("every evaluable C0", None), ("the common C0", common["C0"])):
        moved, rows = {"rise": 0, "fall": 0, "same": 0}, []
        for r in common["C3"]:
            p = [placement(I[(n, "C3", r)], [I[(n, "C0", q)] for q in (against if against is not None else range(REPLICATES))
                                               if I[(n, "C0", q)] is not None]) for n in SIZES]
            moved["rise" if p[1] > p[0] else "fall" if p[1] < p[0] else "same"] += 1
            rows.append({"replicate": r, "3000": p[0], "6000": p[1]})
        out["placements"][name] = {"moves": moved, "rows": rows}
        lines.append(f"- C3 placements against {name} runs of the same size: {moved}")
    lines += ["", "No interval: the pairs are not independent, the sizes share their samples. No threshold, "
              "no success rule; the frozen t is not read here."]
    return "\n".join(lines) + "\n", out, 0


def main(argv):
    text, summary, code = size_reading(argv[0])
    print(json.dumps(summary, indent=1) + "\n" if summary and "--json" in argv[1:] else text, end="")
    sys.exit(code)


if __name__ == "__main__":
    main(sys.argv[1:])
