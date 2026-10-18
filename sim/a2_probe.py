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


LEDGER_HEADER = "ledger\ta2-probe/v2"
SCHEMA = "a2-probe/v1"
DTF_MAX = 0.10
OBSERVED = ("read", "refused")
FIELDS = {"start": 3, "end": 6, "session": 7, "stop": 2}


def read_records(d):
    """Every line of `records.jsonl` as a record, and the numbers of the lines that are not one."""
    recs, invalid = [], []
    path = Path(d) / "records.jsonl"
    text = path.read_text(encoding="utf-8") if path.exists() else ""
    for n, line in enumerate(text.splitlines(), 1):
        try:
            rec = json.loads(line)
            if not isinstance(rec, dict) or rec.get("schema") != SCHEMA:
                raise ValueError
            recs.append(rec)
        except ValueError:
            invalid.append(n)
    return recs, invalid


def read_ledger(d):
    """`ledger.tsv`, strictly: a malformed line raises `ValueError`, it never reads as zero."""
    path = Path(d) / "ledger.tsv"
    if not path.exists():
        return {"started": {}, "ended": {}, "sessions": [], "stop": None}
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != LEDGER_HEADER:
        raise ValueError("ledger line 1: not an a2-probe/v2 ledger")
    out = {"started": {}, "ended": {}, "sessions": [], "stop": None}
    for n, line in enumerate(lines[1:], 2):
        f = line.split("\t")
        if FIELDS.get(f[0]) != len(f):
            raise ValueError(f"ledger line {n}: unknown or truncated line")
        num = lambda i: int(f[i]) if f[i].isdigit() else _bad(n, i)
        if f[0] == "start":
            if f[1] in out["started"]:
                raise ValueError(f"ledger line {n}: a run started twice")
            out["started"][f[1]] = num(2)
        elif f[0] == "end":
            if f[1] not in out["started"] or f[1] in out["ended"]:
                raise ValueError(f"ledger line {n}: an end without its start, or a second end")
            out["ended"][f[1]] = (f[2], num(3), num(4), f[5])
        elif f[0] == "session":
            out["sessions"].append({"wall_ms": num(1), "cpu_ms": num(2), "peak_kib": num(3),
                                    "max_gap_ms": num(4), "stop": f[5], "provenance": f[6]})
        else:
            out["stop"] = f[1]
    return out


def _bad(n, i):
    raise ValueError(f"ledger line {n}: field {i} is not a count")


def read_plans(d):
    plans = {}
    for path in sorted(Path(d).glob("plan-*.tsv")):
        rows = [line.split("\t") for line in path.read_text(encoding="utf-8").splitlines()]
        phase = next((r for r in rows if r[0] == "phase"), None)
        if phase is None:
            raise ValueError(f"{path.name}: no phase line")
        plans[phase[1]] = {"replicates": int(phase[2]),
                           "tasks": {r[1]: int(r[2]) for r in rows if r[0] == "task"}}
    return plans


def audit(d, phase):
    """Plan, ledger and records of `phase` compared: duplicates, records missing, inconsistent or
    unplanned, runs started and never ended, never started. Status complete, partial or invalid."""
    plans, led = read_plans(d), read_ledger(d)
    recs, invalid = read_records(d)
    plan = plans.get(phase, {"replicates": None, "tasks": {}})
    planned = plan["tasks"]
    mine = [r for r in recs if r.get("phase") == phase]
    keys = [r["key"] for r in mine]
    started = [k for k in led["started"] if k.startswith(phase + "-")]
    ended = {k: v for k, v in led["ended"].items() if k.startswith(phase + "-")}
    a = {"phase": phase, "planned": len(planned), "replicates": plan["replicates"], "records": len(mine),
         "invalid_lines": invalid,
         "duplicates": sorted({k for k in keys if keys.count(k) > 1}),
         "unplanned": sorted(set(keys) - set(planned)),
         "inconsistent": sorted(r["key"] for r in mine if r["key"] in planned and (
             r.get("seed") != planned[r["key"]]
             or (r["key"] in ended and ended[r["key"]][0] != r.get("outcome")))),
         "ended_without_record": sorted(set(ended) - set(keys)),
         "record_without_end": sorted(set(keys) - set(ended)),
         "started_not_ended": sorted(set(started) - set(ended)),
         "never_started": sorted(set(planned) - set(started)),
         "not_observed": sorted(k for k, v in ended.items() if v[0] not in OBSERVED),
         "stop": led["stop"]}
    if not planned and not mine:
        a["status"] = "empty"
    elif any(a[k] for k in ("invalid_lines", "duplicates", "unplanned", "inconsistent", "ended_without_record")):
        a["status"] = "invalid"
    elif any(a[k] for k in ("record_without_end", "started_not_ended", "never_started", "not_observed", "stop")):
        a["status"] = "partial"
    else:
        a["status"] = "complete"
    return a


def forms_of(rec):
    if rec.get("outcome") != "read":
        return []
    out = [("fixed", rec["fixed"])]
    for r in rec["readings"]:
        out += [(f"{r['source']}.{name}", r[name]) for name in ("active", "composed", "selection") if r[name]]
    return out


def precision(recs):
    """Recomputes every generating DTF and raw gap by method B and every grid value by the
    41-node replica; returns the largest differences and the failures."""
    worst, fails, n = {"dtf": 0.0, "raw_gap": 0.0, "grid": 0.0}, [], 0
    for rec in recs:
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
            diff = abs(got - want) if isinstance(got, (int, float)) else math.inf
            worst[kind] = max(worst[kind], diff)
            if not diff <= tol:
                fails.append({"key": rec["key"], "value": name, "harness": got, "oracle": want})
    return {"values": n, "max_diff": worst, "failures": fails, "tol": TOL, "grid_tol": GRID_TOL}


def defects(d, recs):
    """The blocking defect of §13.11: a returned selection whose cost exceeds `DTF_MAX`, in a
    record or reported by the supervisor. A generating DTF above the line is not one."""
    out = [r["key"] for r in recs if "defect" in r or str(r.get("reason", "")).startswith("defect")]
    for r in recs:
        for x in r.get("readings", []):
            if x["draw"]["outcome"] in ("selected", "reduced") and x["cost"] is not None and x["cost"] > DTF_MAX:
                out.append(f"{r['key']}.{x['source']}")
    out += [f"rejected/{p.name}" for p in sorted((Path(d) / "rejected").glob("*.json"))]
    return out


def check(d):
    """The precision check over every record of `d`, with each phase's audit; it fails when no
    value is verified, a value departs from the oracle, or an audit is invalid."""
    recs, _ = read_records(d)
    phases = sorted({r["phase"] for r in recs} | set(read_plans(d)))
    audits = [audit(d, p) for p in phases]
    result = precision(recs)
    result["audits"] = audits
    result["passed"] = (result["values"] > 0 and not result["failures"]
                        and all(a["status"] != "invalid" for a in audits))
    return result


def ledger(d):
    led = read_ledger(d)
    ended = {k: (v[0], v[1]) for k, v in led["ended"].items()}
    spent = sum(v[1] for v in led["ended"].values()) + sum(s["cpu_ms"] for s in led["sessions"])
    return ended, spent, led["stop"]


def budget(d, cap_ms=CAP_CPU_MS):
    """The confirmation's replicates by cost alone: `min(8, ⌊(cap − spent) / (1.5 · Σ c)⌋)`, `c`
    each case's explore run's CPU; refused below 2, after a stop, or with an explore run missing."""
    try:
        ended, spent, stop = ledger(d)
    except ValueError as e:
        return {"replicates": None, "reason": f"malformed ledger: {e}"}
    if stop:
        return {"replicates": None, "reason": f"stop recorded: {stop}"}
    costs = {}
    for case in CASES:
        outcome, cpu = ended.get(f"explore-{case}-0", (None, None))
        if outcome not in OBSERVED:
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


def report(d, phase="confirm"):
    """Markdown per case, each metric with its denominator and values per run; exit code 0 when
    the phase is complete, 2 when partial (declared), 1 with nothing to analyse or a stop."""
    try:
        a = audit(d, phase)
    except ValueError as e:
        return f"# A2 probe, phase {phase}\n\nStopped: {e}.\n", 1
    head = [f"# A2 probe, phase {phase}", "", f"Status: {a['status'].upper()}.",
            f"Audit: {json.dumps({k: v for k, v in a.items() if k != 'phase'})}", ""]
    recs = [r for r in read_records(d)[0] if r.get("phase") == phase]
    if a["status"] in ("empty", "invalid") or not recs:
        return "\n".join(head + ["Stopped: nothing verifiable to analyse."]) + "\n", 1
    checked = precision(recs)
    head.append(f"Precision check: {checked['values']} values, largest differences {checked['max_diff']}, "
                f"{len(checked['failures'])} failures (tolerance {TOL}, grid {GRID_TOL}).")
    if checked["failures"]:
        return "\n".join(head + ["", "Stopped: the precision check failed; no quantitative analysis.",
                                 json.dumps(checked["failures"], indent=1)]) + "\n", 1
    found = defects(d, recs)
    if found:
        return "\n".join(head + ["", f"Stopped: blocking defect, a returned selection over DTF_MAX: {found}."]) + "\n", 1
    lines = head + [""]
    if a["status"] == "partial":
        lines += [f"PARTIAL: {a['records']} of {a['planned']} planned runs recorded; never started "
                  f"{len(a['never_started'])}, started and never ended {len(a['started_not_ended'])}, "
                  f"not observed (killed or failed) {len(a['not_observed'])}, stop {a['stop'] or 'none'}.", ""]
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
                sides = {s: count(fs, lambda f, s=s: side(f["dtf"]) == s) for s in ("below", "near", "above")}
                lines.append(f"- {source} {name}: {len(fs)} forms of {len(rs)} runs ({len(rs) - len(read)} without a "
                             f"reading); {comp}; sizes {[f['size'] for f in fs]}; against {LINE}: {sides}")
            drawn = [x for x in rd if x["contested"]]
            if drawn:
                outs = {}
                for x in drawn:
                    outs[x["draw"]["outcome"]] = outs.get(x["draw"]["outcome"], 0) + 1
                sel = [x for x in drawn if x["selection"] is not None and x["cost"] is not None]
                lines.append(f"- {source} draws: {len(drawn)} runs with contested items of {len(read)}; {outs}; "
                             f"cost within DTF_MAX: {count(sel, lambda x: x['cost'] <= DTF_MAX)} of {len(sel)}; "
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
    ended, spent, stop = ledger(d)
    sessions = read_ledger(d)["sessions"]
    lines.append(f"Ledger: {len(ended)} runs ended, cumulative CPU {spent} ms, stop {stop or 'none'}; "
                 f"largest gap between checks per session {[s['max_gap_ms'] for s in sessions]} ms.")
    return "\n".join(lines) + "\n", (2 if a["status"] == "partial" else 0)


def main(argv):
    if not argv:
        sys.exit(__doc__)
    cmd, rest = argv[0], argv[1:]
    if cmd == "reference":
        print(json.dumps(reference(), indent=1))
    elif cmd == "check":
        try:
            result = check(rest[0])
        except ValueError as e:
            sys.exit(f"check stopped: {e}")
        print(json.dumps(result, indent=1))
        sys.exit(0 if result["passed"] else 1)
    elif cmd == "budget":
        result = budget(rest[0])
        print(json.dumps(result, indent=1))
        sys.exit(0 if result["replicates"] else 1)
    elif cmd == "report":
        phase = rest[rest.index("--phase") + 1] if "--phase" in rest else "confirm"
        text, code = report(rest[0], phase)
        print(text, end="")
        sys.exit(code)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
