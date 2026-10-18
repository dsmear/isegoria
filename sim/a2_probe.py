"""A2 probe (`docs/19` §13.11): reference truths, an independent check of the harness's records,
the cost rule for the confirmation's replicates, and the report. Standard library only:
`python3 sim/a2_probe.py reference | check DIR | budget DIR | report DIR [--phase P]`."""
import json
import math
import random
import sys
from pathlib import Path
from statistics import NormalDist, median

GL5 = ((0.0, 0.5688888888888889), (-0.5384693101056831, 0.4786286704993665),
       (0.5384693101056831, 0.4786286704993665), (-0.9061798459386640, 0.2369268850561891),
       (0.9061798459386640, 0.2369268850561891))
SPAN, PANEL, SCAN = 12.0, 0.25, 0.05
TOL, GRID_TOL = 1e-5, 1e-9
LINE, BAND = 0.10, 0.001
MARGIN, R_MAX, R_MIN = 1.5, 8, 2
CAP_CPU_MS = 7_200_000
K, N_REF = 8, 3000
SEED, DRAWS = 20261018, 500
CASES = {"C0": ("c0", 0.0, 0.0), "C1": ("c0", 0.0, 1.0), "C2": ("c2", 0.9, 0.0),
         "C3": ("c8", 0.17, 0.0), "C4": ("m", 0.9, 0.0), "C5": ("c8", 0.04, 0.0)}


def signs(layout, k=K):
    if layout == "m":
        return [1.0, -1.0, 1.0, -1.0] + [0.0] * (k - 4)
    n = int(layout[1:])
    return [1.0] * n + [0.0] * (k - n)


def sig(z):
    return 1 / (1 + math.exp(-z)) if z >= 0 else math.exp(z) / (1 + math.exp(z))


class Items:
    def __init__(self, a, b, s, delta, impact, pi=0.5):
        self.a, self.b, self.s, self.delta, self.impact, self.pi = a, b, s, delta, impact, pi

    def p(self, j, z, t):
        return sig(self.a[j] * (t - self.b[j] - self.delta * self.s[j] * z))

    def gap(self, members, t):
        return sum(self.p(j, 1, t) - self.p(j, -1, t) for j in members)


def of_record(rec):
    d, it = rec["design"], rec["items"]
    return Items(it["a"], it["b"], it["signs"], d["delta"], d["impact"], d["pi"])


def roots(f, lo, hi):
    out, n = [], int(round((hi - lo) / SCAN))
    x0, f0 = lo, f(lo)
    for i in range(1, n + 1):
        x1 = lo + i * SCAN
        f1 = f(x1)
        if f0 * f1 < 0:
            a, b, fa = x0, x1, f0
            for _ in range(60):
                m = 0.5 * (a + b)
                fm = f(m)
                if fa * fm <= 0:
                    b = m
                else:
                    a, fa = m, fm
            out.append(0.5 * (a + b))
        x0, f0 = x1, f1
    return out


def expect(f, mean, breaks=(), panel=PANEL):
    """`E f(θ)`, `θ ~ N(mean, 1)`, by composite 5-point Gauss–Legendre over `mean ± SPAN`, the
    pieces split at `breaks` (method B)."""
    lo, hi = mean - SPAN, mean + SPAN
    cuts = [lo] + sorted(x for x in breaks if lo < x < hi) + [hi]
    total = 0.0
    for a, b in zip(cuts, cuts[1:]):
        n = max(1, math.ceil((b - a) / panel))
        h = (b - a) / n
        for i in range(n):
            c = a + (i + 0.5) * h
            for x, w in GL5:
                t = c + 0.5 * h * x
                total += 0.5 * h * w * f(t) * math.exp(-0.5 * (t - mean) ** 2)
    return total / math.sqrt(2 * math.pi)


def dtf(items, members, panel=PANEL):
    """The generating law's unsigned DTF of `members` between `z = ±1` over the population
    mixture, split at the zeros of the gap (method B)."""
    g = lambda t: items.gap(members, t)
    total = 0.0
    for w, mean in ((1 - items.pi, 0.0), (items.pi, items.impact)):
        breaks = roots(g, mean - SPAN, mean + SPAN) if members else []
        total += w * expect(lambda t: abs(g(t)), mean, breaks, panel)
    return total


def raw_gap(items, members):
    up = expect(lambda t: sum(items.p(j, 1, t) for j in members), items.impact)
    down = expect(lambda t: sum(items.p(j, -1, t) for j in members), 0.0)
    return up - down


def simpson_dtf(items, members, panels=4000, span=10.0):
    """Method A, the harness's composite Simpson without splitting, here for the self-check."""
    total = 0.0
    for w, mean in ((1 - items.pi, 0.0), (items.pi, items.impact)):
        h = 2 * span / panels
        acc = 0.0
        for i in range(panels + 1):
            z = -span + h * i
            c = 1 if i in (0, panels) else (4 if i % 2 else 2)
            acc += c * abs(items.gap(members, mean + z)) * math.exp(-0.5 * z * z)
        total += w * acc * h / 3 / math.sqrt(2 * math.pi)
    return total


def grid_dtf(items, members):
    """`ClassCurves::with_floors`'s 41-node approximation, as `run::true_curves` builds it."""
    nodes = [-5 + 10 * q / 40 for q in range(41)]
    dens = [(1 - items.pi) * math.exp(-0.5 * t * t) + items.pi * math.exp(-0.5 * (t - items.impact) ** 2)
            for t in nodes]
    s = sum(dens)
    return sum(d / s * abs(items.gap(members, t)) for t, d in zip(nodes, dens))


def kl_bound(items):
    """`E ln(1 + χ²(θ))`, `θ ~ N(0, 1)`: §13.4's per-ability bound on `KL` between the two-class law
    at `b ∓ δs` (shares ½) and the one-class law at `b`; floating point, indicative only."""
    def term(t):
        prod = {"++": 1.0, "+-": 1.0, "--": 1.0}
        for j in range(len(items.a)):
            p0 = sig(items.a[j] * (t - items.b[j]))
            pp, pm = items.p(j, -1, t), items.p(j, 1, t)
            f = lambda x, y: x * y / p0 + (1 - x) * (1 - y) / (1 - p0)
            prod["++"] *= f(pp, pp)
            prod["+-"] *= f(pp, pm)
            prod["--"] *= f(pm, pm)
        return math.log(0.25 * prod["++"] + 0.5 * prod["+-"] + 0.25 * prod["--"])
    return expect(term, 0.0)


def tv_bound(kl, n):
    return min(1.0, math.sqrt(1 - math.exp(-n * kl)), math.sqrt(n * kl / 2))


def quantiles(v):
    v = sorted(v)
    q = lambda x: v[int(x * (len(v) - 1))]
    return {"median": median(v), "p5": q(0.05), "p95": q(0.95), "min": v[0], "max": v[-1]}


def side(value):
    return "near" if abs(value - LINE) <= BAND else ("above" if value > LINE else "below")


def reference(draws=DRAWS):
    ref_b = [0.6 * NormalDist().inv_cdf((j + 0.5) / K) for j in range(K)]
    out = {"constants": {"span": SPAN, "panel": PANEL, "scan": SCAN, "tol": TOL, "line": LINE,
                         "band": BAND, "seed": SEED, "draws": draws, "n": N_REF},
           "reference_items": {"a": 1.25, "b": ref_b}, "cases": {}}
    rng = random.Random(SEED)
    item_draws = [([rng.uniform(1.0, 1.5) for _ in range(K)], [0.6 * rng.gauss(0, 1) for _ in range(K)])
                  for _ in range(draws)]
    for name, (layout, delta, impact) in CASES.items():
        s = signs(layout)
        lean = [j for j in range(K) if s[j]]
        ref = Items([1.25] * K, ref_b, s, delta, impact)
        b_ref = dtf(ref, range(K))
        row = {"layout": layout, "delta": delta, "impact": impact,
               "reference": {"dtf": b_ref, "dtf_half_panel": dtf(ref, range(K), PANEL / 2),
                             "simpson": simpson_dtf(ref, range(K)), "grid": grid_dtf(ref, range(K)),
                             "raw_gap": raw_gap(ref, range(K)),
                             "per_leaner": [dtf(ref, [j]) for j in lean]}}
        forms, per, raws, kls = [], [], [], []
        for a, b in item_draws:
            it = Items(a, b, s, delta, impact)
            forms.append(dtf(it, range(K)) if delta else 0.0)
            raws.append(raw_gap(it, range(K)))
            per.extend(dtf(it, [j]) for j in lean)
            if name == "C5":
                kls.append(kl_bound(it))
        row["draws"] = {"form": quantiles(forms), "raw_gap": quantiles(raws),
                        "per_leaner": quantiles(per) if per else None,
                        "sides": {k: sum(side(f) == k for f in forms) for k in ("below", "near", "above")}}
        if name == "C5":
            kl = kl_bound(ref)
            row["kl_bound"] = {"reference": kl, "reference_tv_n": tv_bound(kl, N_REF),
                               "draws": quantiles(kls),
                               "draws_tv_n": quantiles([tv_bound(x, N_REF) for x in kls])}
        out["cases"][name] = row
    return out


def records(path, phase=None):
    out = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            if line.strip():
                rec = json.loads(line)
                if phase is None or rec["phase"] == phase:
                    out.append(rec)
    return out


def forms_of(rec):
    if rec.get("outcome") != "read":
        return []
    out = [("fixed", rec["fixed"])]
    for r in rec["readings"]:
        out += [(f"{r['source']}.{name}", r[name]) for name in ("active", "composed", "selection") if r[name]]
    return out


def check(path):
    """Recomputes every generating DTF and raw gap of the records by method B, and every grid
    value by the 41-node replica; returns the largest differences and the failures."""
    worst, fails, n = {"dtf": 0.0, "raw_gap": 0.0, "grid": 0.0}, [], 0
    for rec in records(path):
        it = of_record(rec)
        full = list(range(rec["design"]["k"]))
        pairs = [("truth.dtf", rec["truth"]["dtf"], dtf(it, full), TOL, "dtf"),
                 ("truth.raw_gap", rec["truth"]["raw_gap"], raw_gap(it, full), TOL, "raw_gap")]
        if rec["truth"]["grid"] is not None:
            pairs.append(("truth.grid", rec["truth"]["grid"], grid_dtf(it, full), GRID_TOL, "grid"))
        for name, form in forms_of(rec):
            pairs.append((name, form["dtf"], dtf(it, form["members"]), TOL, "dtf"))
            if form["grid"] is not None:
                pairs.append((name + ".grid", form["grid"], grid_dtf(it, form["members"]), GRID_TOL, "grid"))
        for name, got, want, tol, kind in pairs:
            n += 1
            diff = abs(got - want)
            worst[kind] = max(worst[kind], diff)
            if diff > tol:
                fails.append({"key": rec["key"], "value": name, "harness": got, "oracle": want})
    return {"values": n, "max_diff": worst, "failures": fails, "tol": TOL, "grid_tol": GRID_TOL}


def ledger(path):
    text = Path(path).read_text(encoding="utf-8")
    rows = [line.split("\t") for line in text.splitlines() if line.strip()]
    ended = {r[1]: (r[2], int(r[3])) for r in rows if r[0] == "end"}
    spent = sum(c for _, c in ended.values()) + sum(int(r[2]) for r in rows if r[0] == "session")
    stop = next((r[1] for r in rows if r[0] == "stop"), None)
    return ended, spent, stop


def budget(path, cap_ms=CAP_CPU_MS):
    """The confirmation's replicates by cost alone: `min(8, ⌊(cap − spent) / (1.5 · Σ c)⌋)`, `c`
    each case's explore run's CPU; refused below 2, after a stop, or with an explore run missing."""
    ended, spent, stop = ledger(path)
    if stop:
        return {"replicates": None, "reason": f"stop recorded: {stop}"}
    costs = {}
    for case in CASES:
        outcome, cpu = ended.get(f"explore-{case}-0", (None, 0))
        if outcome not in ("read", "refused"):
            return {"replicates": None, "reason": f"explore-{case}-0: {outcome or 'missing'}"}
        costs[case] = cpu
    per_replicate = MARGIN * sum(costs.values())
    left = cap_ms - spent
    r = R_MAX if per_replicate == 0 else min(R_MAX, int(left // per_replicate))
    out = {"costs_ms": costs, "spent_ms": spent, "cap_ms": cap_ms, "per_replicate_ms": per_replicate}
    if r < R_MIN:
        return dict(out, replicates=None, reason=f"{r} replicates fit the cap, fewer than {R_MIN}")
    return dict(out, replicates=r, reason="cost rule")


def count(xs, pred=bool):
    return sum(1 for x in xs if pred(x))


def report(path, phase="confirm"):
    """Markdown per case: every outcome counted, each metric with its denominator, values per
    run; no pooled rate, interval or score. Classification against the line needs the check."""
    recs = records(path, phase)
    checked = check(path)
    lines = [f"# A2 probe, phase {phase}", "",
             f"Precision check: {checked['values']} values, largest differences {checked['max_diff']}, "
             f"{len(checked['failures'])} failures (tolerance {TOL}, grid {GRID_TOL}).", ""]
    ok = not checked["failures"]
    by = {}
    for rec in recs:
        by.setdefault(rec["case"], []).append(rec)
    for case in sorted(by):
        rs = sorted(by[case], key=lambda r: r["replicate"])
        read = [r for r in rs if r["outcome"] == "read"]
        ev = [r for r in read if r["fit"]["status"] == "converged"]
        one = [r for r in ev if (r["fit"]["counted"] or 0) < 2]
        roles = rs[0]["items"]["roles"]
        lean = [j for j, c in enumerate(roles) if c in "+-2"]
        clean = [j for j, c in enumerate(roles) if c == "c"]
        flag = lambda r, j: r["rechecks"][j] == "D"
        lines += [f"## {case}", "",
                  f"- runs {len(rs)}: " + ", ".join(f"{o} {count(rs, lambda r, o=o: r['outcome'] == o)}"
                                                    for o in ("read", "refused", "killed", "failed")),
                  f"- read {len(read)}: converged {len(ev)}, not converged {len(read) - len(ev)}; "
                  f"one counted class among converged {len(one)}",
                  f"- leaners flagged {sum(flag(r, j) for r in ev for j in lean)} of {len(lean) * len(ev)} "
                  f"(converged runs; {len(lean) * len(rs)} over all runs)",
                  f"- clean items flagged {sum(flag(r, j) for r in ev for j in clean)} of {len(clean) * len(ev)}; "
                  f"runs with one {count(ev, lambda r: any(flag(r, j) for j in clean))} of {len(ev)}",
                  f"- fixed form, generating DTF per run: {[round(r['truth']['dtf'], 4) for r in rs]}",
                  f"- raw gap per run: {[round(r['truth']['raw_gap'], 4) for r in rs]}",
                  f"- fitted minus generating DTF, fixed form, converged runs: "
                  f"{[round(r['fixed']['fitted'] - r['truth']['dtf'], 4) for r in ev if r['fixed']['fitted'] is not None]}"]
        for source in ("S0", "S1"):
            rd = [next(x for x in r["readings"] if x["source"] == source) for r in read]
            for name in ("active", "composed"):
                fs = [x[name] for x in rd]
                comp = {c: count(fs, lambda f, c=c: f["completeness"] == c) for c in ("empty", "incomplete", "complete")}
                sides = ({s: count(fs, lambda f, s=s: side(f["dtf"]) == s) for s in ("below", "near", "above")}
                         if ok else "not classified: precision check failed")
                lines.append(f"- {source} {name}: {len(fs)} forms of {len(rs)} runs ({len(rs) - len(read)} without a "
                             f"reading); {comp}; sizes {[f['size'] for f in fs]}; against {LINE}: {sides}")
            drawn = [x for x in rd if x["contested"]]
            if drawn:
                outs = {}
                for x in drawn:
                    outs[x["draw"]["outcome"]] = outs.get(x["draw"]["outcome"], 0) + 1
                sel = [x for x in drawn if x["selection"] is not None and x["cost"] is not None]
                lines.append(f"- {source} draws: {len(drawn)} runs with contested items of {len(read)}; {outs}; "
                             f"cost within {LINE}: {count(sel, lambda x: x['cost'] <= LINE)} of {len(sel)}; "
                             f"selection generating DTF minus cost: "
                             f"{[round(x['selection']['dtf'] - x['cost'], 4) for x in sel]}")
        cpu = [r["cost"]["cpu_ms"] if r.get("cost") else None for r in rs]
        lines += [f"- CPU ms per run: {cpu}", ""]
    if "C0" in by:
        base = {r["replicate"]: r for r in by["C0"] if r["outcome"] == "read"}
        lines += ["## Paired with C0 (same replicate): classes, flags, fitted DTF of the fixed form", ""]
        for case in ("C1", "C5"):
            for r in sorted(by.get(case, []), key=lambda r: r["replicate"]):
                b = base.get(r["replicate"])
                if b and r["outcome"] == "read":
                    pick = lambda x: (x["fit"]["classes"], x["rechecks"].count("D"), x["fixed"]["fitted"])
                    lines.append(f"- {case} r{r['replicate']}: {pick(r)} against C0 {pick(b)}")
        lines.append("")
    led = Path(path).parent / "ledger.tsv"
    if led.exists():
        ended, spent, stop = ledger(led)
        lines.append(f"Ledger: {len(ended)} runs ended, cumulative CPU {spent} ms, stop {stop or 'none'}.")
    return "\n".join(lines) + "\n"


def main(argv):
    if not argv:
        sys.exit(__doc__)
    cmd, rest = argv[0], argv[1:]
    if cmd == "reference":
        print(json.dumps(reference(), indent=1))
    elif cmd == "check":
        result = check(Path(rest[0]) / "records.jsonl")
        print(json.dumps(result, indent=1))
        sys.exit(1 if result["failures"] else 0)
    elif cmd == "budget":
        print(json.dumps(budget(Path(rest[0]) / "ledger.tsv"), indent=1))
    elif cmd == "report":
        phase = rest[rest.index("--phase") + 1] if "--phase" in rest else "confirm"
        print(report(Path(rest[0]) / "records.jsonl", phase), end="")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
