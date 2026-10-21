"""A2 probe (`docs/19` §§13.11–13.29): truths, checks, cost rule, reports; standard library only.
`python3 sim/a2_probe.py reference | check D | budget D | report D [--phase P] | compare-first D F |
candidates D F | dtf-check D [--json] | threshold D | ind-val D CAL [--json]`, both [--phase S]."""
import hashlib
import json
import math
import random
import struct
import sys
from fractions import Fraction
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
    """`ledger.tsv`, strictly: a malformed line raises `ValueError`, it never reads as zero. A
    session's stop binds without the `stop` line that follows it."""
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
            if f[5] != "-" and out["stop"] is None:
                out["stop"] = f[5]
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
    """The confirmation's replicates by cost alone, `min(8, ⌊(cap − spent) / (1.5 · Σ c)⌋)`, `c`
    each case's explore CPU, once explore is complete, precise and free of the blocking defect;
    refused otherwise, below 2 replicates, or after a stop."""
    try:
        ended, spent, stop = ledger(d)
        explore = audit(d, "explore")
    except ValueError as e:
        return {"replicates": None, "reason": f"malformed ledger or plan: {e}"}
    if stop:
        return {"replicates": None, "reason": f"stop recorded: {stop}"}
    if explore["status"] != "complete":
        return {"replicates": None, "reason": f"explore is {explore['status']}", "audit": explore}
    recs = [r for r in read_records(d)[0] if r.get("phase") == "explore"]
    checked = precision(recs)
    if checked["failures"] or not checked["values"]:
        return {"replicates": None, "reason": "explore's precision check failed", "check": checked}
    found = defects(d, recs)
    if found:
        return {"replicates": None, "reason": f"blocking defect: {found}"}
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


COMPARED = ("schema", "seed", "engine_seed", "draw_seed", "design", "items", "truth", "kr20",
            "outcome", "reason", "fit", "rechecks", "fixed", "readings")


def records_at(path):
    return [json.loads(line) for line in Path(path).read_text(encoding="utf-8").splitlines() if line.strip()]


def compare_first(d, first):
    """Each `candidates` record's fields in `COMPARED` against the first probe's confirm record of
    its case and replicate, value for value; keys, phase, cost and candidates are left out."""
    recs = [r for r in read_records(d)[0] if r.get("phase") == "candidates"]
    twins = {r["key"]: r for r in records_at(first)}
    out = {"compared": 0, "missing": [], "mismatches": []}
    for r in recs:
        twin = twins.get(r["key"].replace("candidates-", "confirm-", 1))
        if twin is None:
            out["missing"].append(r["key"])
            continue
        out["compared"] += 1
        fields = [f for f in COMPARED if r.get(f) != twin.get(f)]
        if fields:
            out["mismatches"].append({"key": r["key"], "fields": fields})
    out["passed"] = bool(recs) and not out["missing"] and not out["mismatches"]
    return out


def comparable(c):
    return c["status"] == "converged" and c["one_converged"]


def candidates_report(d, first):
    """Per case and run, every candidate: verdict, gain and penalty against the one-class fit, its
    classes and whole-form `DTF_F`; two-class variants paired with C0 only where comparable."""
    head = ["# A2 probe, candidates (§13.14)", ""]
    try:
        a = audit(d, "candidates")
    except ValueError as e:
        return "\n".join(head + [f"Stopped: {e}."]) + "\n", 1
    recs = [r for r in read_records(d)[0] if r.get("phase") == "candidates"]
    if a["status"] in ("empty", "invalid") or not recs:
        return "\n".join(head + [f"Stopped: phase {a['status']}."]) + "\n", 1
    same = compare_first(d, first)
    checked = precision(recs)
    head += [f"Status: {a['status'].upper()}; {a['records']} of {a['planned']} planned runs recorded.",
             f"Reproduction of the first probe: {same['compared']} compared, {len(same['missing'])} missing, "
             f"{len(same['mismatches'])} with a declared field differing.",
             f"Precision check: {checked['values']} values, {len(checked['failures'])} failures.", ""]
    if not same["passed"] or checked["failures"]:
        return "\n".join(head + ["Stopped: no candidate is read.", json.dumps(same["mismatches"])]) + "\n", 1
    by = {}
    for r in recs:
        by.setdefault(r["case"], {})[r["replicate"]] = r
    lines = head
    for case in sorted(by):
        rs = [by[case][k] for k in sorted(by[case])]
        read = [r for r in rs if r["outcome"] == "read"]
        cands = [c for r in read for c in r["candidates"]]
        lines += [f"## {case}", "",
                  f"- runs {len(rs)}, read {len(read)}; candidates {len(cands)}, converged "
                  f"{count(cands, lambda c: c['status'] == 'converged')}; one-class fit converged in "
                  f"{count(read, lambda r: r['candidates'][0]['status'] == 'converged')} of {len(read)}"]
        for r in read:
            lines.append(f"- r{r['replicate']} generating DTF {round(r['truth']['dtf'], 4)}:")
            for c in r["candidates"]:
                tag = "comparable" if comparable(c) else "not comparable: baseline or candidate not converged"
                dtf = "unavailable" if c["dtf"] is None else round(c["dtf"], 4)
                lines.append(f"  - {c['classes']} class{'es' if c['classes'] > 1 else ''}, "
                             f"{'per-class' if c['non_uniform'] else 'shared'} slopes: {c['status']}, {c['verdict']}; "
                             f"gain {round(c['gain'], 2)}, penalty {round(c['penalty'], 2)}, delta BIC "
                             f"{round(c['delta_bic'], 2)}; counted {c['counted']}, pi {[round(x, 3) for x in c['pi']]}; "
                             f"DTF_F {dtf} ({c['dtf_basis']}); {tag if c['classes'] > 1 else 'baseline'}")
        lines.append("")
    if "C0" in by:
        lines += ["## Two-class variants paired with C0 by replicate (gain minus C0's gain)", ""]
        for case in sorted(c for c in by if c != "C0"):
            for rep in sorted(by[case]):
                r, b = by[case][rep], by["C0"].get(rep)
                if b is None or r["outcome"] != "read" or b["outcome"] != "read":
                    lines.append(f"- {case} r{rep}: no pair (a run without a reading)")
                    continue
                for slopes in (False, True):
                    pick = lambda x: next((c for c in x["candidates"] if c["classes"] == 2 and c["non_uniform"] == slopes), None)
                    c, c0 = pick(r), pick(b)
                    name = "per-class" if slopes else "shared"
                    if c is None or c0 is None:
                        lines.append(f"- {case} r{rep}, {name}: not fitted in both")
                    elif comparable(c) and comparable(c0):
                        lines.append(f"- {case} r{rep}, {name}: {round(c['gain'] - c0['gain'], 2)}")
                    else:
                        lines.append(f"- {case} r{rep}, {name}: not comparable (baseline or candidate not converged)")
        lines.append("")
    return "\n".join(lines) + "\n", (2 if a["status"] == "partial" else 0)


VARIANTS = ((False, "shared"), (True, "per-class"))
DTF_CHECK_PAIRS = 16


def pair_side(rec, slopes):
    """One side of a pair: its two-class candidate of the given slopes, and why the pair cannot be
    ordered on its account (`None` when it can)."""
    if rec is None:
        return None, "run missing"
    if rec.get("outcome") != "read":
        return None, f"run {rec.get('outcome')}"
    c = next((x for x in rec["candidates"] if x["classes"] == 2 and x["non_uniform"] == slopes), None)
    if rec["candidates"][0]["status"] != "converged":
        return c, "baseline not converged"
    if c is None:
        return None, "candidate missing"
    if c["status"] != "converged":
        return c, "candidate not converged"
    if c["dtf"] is None:
        return c, "DTF unavailable"
    return c, None


def order(a, b):
    return "greater" if a > b else ("smaller" if a < b else "equal")


def dtf_check_plan(d):
    """Why `d` does not hold `docs/19` §13.18's phase: its declared replicates, its 32 tasks,
    records matching their keys, paired seeds, a complete audit. Empty when it does."""
    phase, pairs = "dtf-check", DTF_CHECK_PAIRS
    a = audit(d, phase)
    plan = read_plans(d).get(phase, {"replicates": None, "tasks": {}})
    tasks, want = plan["tasks"], {f"{phase}-{c}-{r}" for c in ("C0", "C3") for r in range(pairs)}
    why = []
    if plan["replicates"] != pairs:
        why.append(f"the plan declares {plan['replicates']} replicates, not {pairs}")
    if set(tasks) != want:
        why.append(f"the plan's tasks are not the {len(want)} declared: missing {sorted(want - set(tasks))}, "
                   f"unexpected {sorted(set(tasks) - want)}")
    off = sorted(str(r.get("key")) for r in read_records(d)[0]
                 if (r.get("phase") == phase or str(r.get("key")).startswith(phase + "-"))
                 and (r.get("phase") != phase or r.get("key") not in want
                      or r.get("key") != f"{phase}-{r.get('case')}-{r.get('replicate')}"))
    if off:
        why.append(f"records not matching their key's phase, case and replicate: {off}")
    seeds = [(tasks.get(f"{phase}-C0-{r}"), tasks.get(f"{phase}-C3-{r}")) for r in range(pairs)]
    unpaired = [r for r, (s0, s3) in enumerate(seeds) if None not in (s0, s3) and s0 != s3]
    if unpaired:
        why.append(f"C0's and C3's seeds differ at replicates {unpaired}")
    if a["status"] != "complete":
        why.append(f"the audit is {a['status']}, {a['records']} records of {a['planned']} planned")
    return why


def dtf_check(d):
    """`docs/19` §13.18's command: `paired_dtf` on the declared phase alone, else a stop."""
    try:
        why = dtf_check_plan(d)
    except ValueError as e:
        why = [str(e)]
    if why:
        return "# A2 probe, dtf-check (§13.18)\n\nStopped, not the declared phase: " + "; ".join(why) + ".\n", None, 1
    return paired_dtf(d, "dtf-check", DTF_CHECK_PAIRS)


def paired_dtf(d, phase, pairs):
    """`docs/19` §13.18's analysis: per variant, C3's two-class candidate's whole-form DTF against
    C0's on the same replicate, unrounded; zeros counted with their origin; the pairs that cannot be
    ordered listed with why; then the pairs without a zero, as a conditional reading."""
    try:
        a = audit(d, phase)
    except ValueError as e:
        return f"# A2 probe, {phase}\n\nStopped: {e}.\n", None, 1
    recs = {(r["case"], r["replicate"]): r for r in read_records(d)[0] if r.get("phase") == phase}
    checked = precision(list(recs.values()))
    head = [f"# A2 probe, {phase} (§13.18)", "", f"Audit: {a['status']}, {a['records']} of {a['planned']} runs.",
            f"Precision check: {checked['values']} values, {len(checked['failures'])} failures.", ""]
    if a["status"] != "complete" or checked["failures"] or not checked["values"]:
        return "\n".join(head + ["Stopped: the analysis needs a complete audit and a passing check."]) + "\n", None, 1
    summary, lines = {}, head
    for slopes, name in VARIANTS:
        rows, tally, lost = [], {"greater": 0, "smaller": 0, "equal": 0}, {}
        nonzero = {"greater": 0, "smaller": 0, "equal": 0}
        for rep in range(pairs):
            r0, r3 = recs.get(("C0", rep)), recs.get(("C3", rep))
            (c0, why0), (c3, why3) = pair_side(r0, slopes), pair_side(r3, slopes)
            row = {"replicate": rep, "c0": c0, "c3": c3,
                   "generating_c3": r3["truth"]["dtf"] if r3 else None}
            if why0 or why3:
                row["not_comparable"] = {"C0": why0, "C3": why3}
                for why in filter(None, (why0 and f"C0: {why0}", why3 and f"C3: {why3}")):
                    lost[why] = lost.get(why, 0) + 1
            else:
                row["order"] = order(c3["dtf"], c0["dtf"])
                row["gain_minus_c0"] = c3["gain"] - c0["gain"]
                tally[row["order"]] += 1
                if c3["dtf"] != 0 and c0["dtf"] != 0:
                    nonzero[row["order"]] += 1
            rows.append(row)
        comparable = sum(tally.values())
        summary[name] = {"planned": pairs, "comparable": comparable, "order": tally, "not_comparable": lost,
                         "without_zeros": dict(nonzero, pairs=sum(nonzero.values())), "rows": rows}
        lines += [f"## {name} slopes", "",
                  f"- C3 against C0, of {pairs} planned pairs: {comparable} comparable — greater "
                  f"{tally['greater']}, smaller {tally['smaller']}, equal {tally['equal']}; not comparable "
                  f"{pairs - comparable} {lost or ''}",
                  f"- conditional on the {sum(nonzero.values())} comparable pairs without a zero: greater "
                  f"{nonzero['greater']}, smaller {nonzero['smaller']}, equal {nonzero['equal']}", ""]
        for row in rows:
            cell = lambda c: "none" if c is None else (
                f"DTF {c['dtf']!r} ({c['dtf_basis']}), counted {c['counted']}, pi "
                f"{[round(x, 3) for x in c['pi']]}, gain {round(c['gain'], 2)}, {c['status']}")
            tail = (f"{row['order']}, gain minus C0's {round(row['gain_minus_c0'], 2)}" if "order" in row
                    else f"not comparable: {row['not_comparable']}")
            lines.append(f"- r{row['replicate']} (C3 generating DTF {row['generating_c3']}): C0 {cell(row['c0'])}; "
                         f"C3 {cell(row['c3'])} — {tail}")
        lines.append("")
    lines.append("The variants refit the same batches: they are not independent and are not summed.")
    return "\n".join(lines) + "\n", summary, 0


IND_REPLICATES = 100
IND_ALPHA = Fraction(1, 20)
IND_CASES = {"ind-cal": ("C0", "C1"), "ind-val": ("C0", "C1", "C3", "C5"),
             "ind-cal6": ("C0", "C1"), "ind-val6": ("C0", "C1", "C3", "C5")}
IND_N = {"ind-cal": 3000, "ind-val": 3000, "ind-cal6": 6000, "ind-val6": 6000}
CALIBRATION = {"ind-val": "ind-cal", "ind-val6": "ind-cal6"}
TITLES = {"ind-val": "validation (§13.21)", "ind-val6": "validation at N = 6,000 (§13.29)"}
NEUTRAL = ("C0", "C1")


def cal_files(phase):
    return ("records.jsonl", f"plan-{phase}.tsv", "ledger.tsv")


CAL_FILES = cal_files("ind-cal")


def header(phase):
    """The lines of `phase`'s `threshold.tsv` before its cases: `ind-cal`'s schema v1 and rule, or
    schema v2 naming the stage and its `N`, then the rule."""
    stage = ["schema\ta2-ind/v1"] if phase == "ind-cal" else ["schema\ta2-ind/v2", f"stage\t{phase}\tn\t{IND_N[phase]}"]
    return stage + ["rule\torder-statistic\talpha\t1/20\tstrict"]


def declared(path):
    """The stage and `N` a `threshold.tsv` names: `ind-cal` at 3,000 by schema v1, else v2's."""
    lines = Path(path).read_text(encoding="utf-8").splitlines() + ["", ""]
    stage = lines[1].split("\t")
    if lines[0] == "schema\ta2-ind/v1":
        return "ind-cal", 3000
    if lines[0] == "schema\ta2-ind/v2" and len(stage) == 4 and stage[::2] == ["stage", "n"] and stage[3].isdigit():
        return stage[1], int(stage[3])
    return None


def probe_seed(phase, r):
    """The harness's `probe_seed` (`docs/19` §13.11); `case_seed` adds the case (§13.21)."""
    return seed_of(phase, r)


def case_seed(phase, case, r):
    return seed_of(phase, r, case)


def seed_of(phase, r, case=None):
    h = hashlib.sha256(b"isegoria/a2-probe/v1")
    for name in (phase, case) if case else (phase,):
        h.update(struct.pack("<Q", len(name)) + name.encode())
    h.update(struct.pack("<I", r))
    return struct.unpack("<Q", h.digest()[:8])[0]


def bits(x):
    return f"{struct.unpack('<Q', struct.pack('<d', x))[0]:016x}"


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def indicator(rec):
    """`docs/19` §13.21's `I` for one run: the shared two-class candidate's `DTF_F`, or the
    observation that leaves the run not evaluable."""
    if rec.get("outcome") != "read":
        return None, "gate refused" if rec.get("outcome") == "refused" else f"run {rec.get('outcome')}"
    if rec["candidates"][0]["status"] != "converged":
        return None, "baseline not converged"
    c = next((x for x in rec["candidates"] if x["classes"] == 2 and not x["non_uniform"]), None)
    if c is None:
        return None, "candidate missing"
    if c["status"] != "converged":
        return None, "candidate not converged"
    return (c["dtf"], None) if c["dtf"] is not None else (None, "DTF unavailable")


def rank(n):
    """The order statistic's rank `⌈(1 − α)(n + 1)⌉` among `n` values, or `None` above `n`."""
    k = math.ceil((1 - IND_ALPHA) * (n + 1))
    return k if k <= n else None


def case_threshold(values):
    k = rank(len(values))
    return (k, sorted(values)[k - 1]) if k else (None, None)


def listed(keys):
    keys = sorted(keys)
    return f"{len(keys)} {keys[:4]}{' …' if len(keys) > 4 else ''}"


def designs(d, phase):
    """Each case's `N` in the configuration of `phase`'s plan in `d`."""
    path = Path(d) / f"plan-{phase}.tsv"
    lines = config_of(path) if path.is_file() else []
    return {f[1]: f[2] for f in (l.split("\t") for l in lines) if f[0] == "design" and len(f) > 2}


def stage_reasons(d, phase):
    """Why `d` does not hold §13.21's or §13.29's `phase` exactly: its plan alone, its replicates,
    tasks and seeds, every case's design at the stage's `N`, records matching their keys and
    size, a complete audit. Empty when it does."""
    a = audit(d, phase)
    plans = read_plans(d)
    plan = plans.get(phase, {"replicates": None, "tasks": {}})
    tasks, n = plan["tasks"], IND_N[phase]
    want = {f"{phase}-{c}-{r}": case_seed(phase, c, r) for c in IND_CASES[phase] for r in range(IND_REPLICATES)}
    why = []
    if sorted(plans) != [phase]:
        why.append(f"the directory holds the plans of {sorted(plans)}, not of {phase} alone")
    sizes = designs(d, phase)
    if sizes != {c: str(n) for c in CASES}:
        why.append(f"the plan's configuration is not every case's design at N = {n}: {sorted(sizes.items())}")
    if plan["replicates"] != IND_REPLICATES:
        why.append(f"the plan declares {plan['replicates']} replicates, not {IND_REPLICATES}")
    if set(tasks) != set(want):
        why.append(f"the plan's tasks are not the {len(want)} declared: missing {listed(set(want) - set(tasks))}, "
                   f"unexpected {listed(set(tasks) - set(want))}")
    wrong = [k for k in set(tasks) & set(want) if tasks[k] != want[k]]
    if wrong:
        why.append(f"seeds not the declared ones: {listed(wrong)}")
    off = [str(r.get("key")) for r in read_records(d)[0]
           if (r.get("phase") == phase or str(r.get("key")).startswith(phase + "-"))
           and (r.get("phase") != phase or r.get("key") not in want
                or r.get("key") != f"{phase}-{r.get('case')}-{r.get('replicate')}"
                or r.get("design", {}).get("n") != n)]
    if off:
        why.append(f"records not matching their key's phase, case, replicate and N = {n}: {listed(off)}")
    if a["status"] != "complete":
        why.append(f"the audit is {a['status']}, {a['records']} records of {a['planned']} planned")
    return why


def read_stage(d, phase):
    """The stage's records by case, after its contract and the numerical check; else the reasons."""
    try:
        why = stage_reasons(d, phase)
    except ValueError as e:
        why = [str(e)]
    if why:
        return None, why
    recs = [r for r in read_records(d)[0] if r.get("phase") == phase]
    checked = precision(recs)
    if checked["failures"] or not checked["values"]:
        return None, [f"the numerical check fails on {phase}: {len(checked['failures'])} failures"]
    return {c: sorted((r for r in recs if r["case"] == c), key=lambda r: r["replicate"]) for c in IND_CASES[phase]}, []


def calibrate(d, phase="ind-cal"):
    """`docs/19` §13.21's calibration reading, or §13.29's at 6,000 for `ind-cal6`: the text of
    `threshold.tsv`, or why none is set."""
    by_case, why = read_stage(d, phase)
    if why:
        return None, why
    lines, lost, ts = header(phase), [], []
    for case in NEUTRAL:
        seen = [indicator(r) for r in by_case[case]]
        values = [v for v, _ in seen if v is not None]
        k, t = case_threshold(values)
        if k is None:
            return None, [f"{case}: {len(values)} evaluable runs give no finite threshold (19 needed)"]
        lines.append(f"case\t{case}\tplanned\t{IND_REPLICATES}\tevaluable\t{len(values)}\tk\t{k}\tt\t{bits(t)}\t{t!r}")
        for reason in sorted({w for _, w in seen if w}):
            lost.append(f"not-evaluable\t{case}\t{reason}\t{sum(w == reason for _, w in seen)}")
        ts.append(t)
    t = max(ts)
    lines += lost + [f"threshold\t{bits(t)}\t{t!r}"] + [f"file\t{f}\t{sha256(Path(d) / f)}" for f in cal_files(phase)]
    return "\n".join(lines) + "\n", []


def clopper_pearson(k, n, level=0.95):
    """The exact two-sided interval for `k` events in `n` binomial trials; `None` when `n = 0`."""
    if n == 0:
        return None
    tail = (1 - level) / 2

    def solve(j, target):
        lo, hi = 0.0, 1.0
        for _ in range(100):
            mid = (lo + hi) / 2
            above = sum(math.comb(n, i) * mid ** i * (1 - mid) ** (n - i) for i in range(j, n + 1))
            lo, hi = (mid, hi) if above < target else (lo, mid)
        return (lo + hi) / 2
    return 0.0 if k == 0 else solve(k, tail), 1.0 if k == n else solve(k + 1, 1 - tail)


def separation(high, low):
    """The share of pairs where a value of `high` exceeds one of `low`, ties counted ½."""
    if not high or not low:
        return None
    return sum((a > b) + 0.5 * (a == b) for a in high for b in low) / (len(high) * len(low))


def ci(c):
    return "none" if c is None else f"[{c[0]:.3f}, {c[1]:.3f}]"


def config_of(path):
    text = Path(path).read_text(encoding="utf-8")
    lines = text.splitlines()
    end = next((i for i, l in enumerate(lines) if l.startswith("phase\t")), len(lines))
    return lines[:end]


def ind_val(d, cal, phase="ind-val"):
    """`docs/19` §13.21's validation reading, or §13.29's at 6,000 for `ind-val6`, against its own
    frozen calibration `cal`, per case."""
    title = f"# A2 single-test study, {TITLES[phase]}"

    def stop(why):
        return f"{title}\n\nStopped, not read: " + "; ".join(why) + ".\n", None, 1
    by_case, why = read_stage(d, phase)
    if why:
        return stop(why)
    frozen, stage = Path(cal) / "threshold.tsv", CALIBRATION[phase]
    if not frozen.is_file():
        return stop(["no threshold.tsv in the calibration"])
    try:
        plans, named = sorted(read_plans(cal)), declared(frozen)
    except ValueError as e:
        return stop([f"the calibration: {e}"])
    if plans != [stage] or named != (stage, IND_N[stage]):
        return stop([f"the calibration is not {stage}'s at N = {IND_N[stage]}: its plans are {plans}, "
                     f"its threshold.tsv names {named}"])
    tsv, why = calibrate(cal, stage)
    if why:
        return stop(["the calibration: " + "; ".join(why)])
    if tsv.encode() != frozen.read_bytes():
        return stop(["threshold.tsv is not the threshold the calibration gives"])
    t_bits = next(l for l in tsv.splitlines() if l.startswith("threshold\t")).split("\t")[1]
    t = struct.unpack("<d", struct.pack("<Q", int(t_bits, 16)))[0]
    own, theirs = config_of(Path(d) / f"plan-{phase}.tsv"), config_of(Path(cal) / f"plan-{stage}.tsv")
    if f"calibration\t{sha256(frozen)}\t{t_bits}" not in own:
        return stop(["the validation plan is not bound to this threshold"])
    if [l for l in own if not l.startswith(("caps\t", "calibration\t"))] != [l for l in theirs if not l.startswith("caps\t")]:
        return stop(["the two stages ran different configurations"])
    summary, values = {"threshold": {"bits": t_bits, "value": t}, "cases": {}, "separation": {}}, {}
    lines = [title, "", f"Threshold `t` = {t!r} ({t_bits}), frozen "
             f"in {frozen}; flagged when `I > t`.", ""]
    for case in IND_CASES[phase]:
        seen = [indicator(r) for r in by_case[case]]
        vals = values[case] = [v for v, _ in seen if v is not None]
        lost = {w: sum(x == w for _, x in seen) for w in sorted({w for _, w in seen if w})}
        n, k = len(vals), sum(v > t for v in vals)
        neutral = case in NEUTRAL
        summary["cases"][case] = {
            "role": "neutral" if neutral else "unbalanced", "planned": IND_REPLICATES, "evaluable": n,
            "not_evaluable": lost, "flagged": k, "not_flagged": n - k, "zeros": sum(v == 0 for v in vals),
            "flagged_over_evaluable": {"k": k, "n": n, "ci95": clopper_pearson(k, n)},
            "flagged_over_planned": {"k": k, "n": IND_REPLICATES, "ci95": clopper_pearson(k, IND_REPLICATES)},
            "not_evaluable_over_planned": {"k": IND_REPLICATES - n, "n": IND_REPLICATES,
                                           "ci95": clopper_pearson(IND_REPLICATES - n, IND_REPLICATES)},
            "values": [v for v, _ in seen]}
        flag, keep = ("false alarms", "correct non-flags") if neutral else ("detections", "misses")
        lines += [f"## {case} ({summary['cases'][case]['role']})", "",
                  f"- planned {IND_REPLICATES}; evaluable {n}; not evaluable {IND_REPLICATES - n} {lost or ''}, "
                  f"95% {ci(clopper_pearson(IND_REPLICATES - n, IND_REPLICATES))}",
                  f"- {flag}: {k} of {n} evaluable, conditional, 95% {ci(clopper_pearson(k, n))}; "
                  f"{k} of {IND_REPLICATES} planned, 95% {ci(clopper_pearson(k, IND_REPLICATES))}",
                  f"- {keep}: {n - k}; zeros of `I`: {summary['cases'][case]['zeros']}", ""]
    lines += ["## Separation without threshold, conditional on evaluable runs (ties ½)", ""]
    for hi in ("C3", "C5"):
        for lo in NEUTRAL:
            share = summary["separation"][f"{hi}>{lo}"] = separation(values[hi], values[lo])
            lines.append(f"- {hi} against {lo}: {share!r} over {len(values[hi])} × {len(values[lo])} pairs")
    lines += ["", "Intervals: Clopper–Pearson, exact two-sided 95%, each marginal; a case's runs taken as "
              "independent and identically distributed given the frozen `t`; no adjustment across intervals. "
              "Cases are not pooled; nothing is set on validation."]
    return "\n".join(lines) + "\n", summary, 0


def option(rest, flag, default):
    """The value after `flag`, `default` without the flag, `None` with the flag and no value."""
    return rest[rest.index(flag) + 1] if flag in rest[:-1] else None if flag in rest else default


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
    elif cmd == "compare-first":
        result = compare_first(rest[0], rest[1])
        print(json.dumps(result, indent=1))
        sys.exit(0 if result["passed"] else 1)
    elif cmd == "candidates":
        text, code = candidates_report(rest[0], rest[1])
        print(text, end="")
        sys.exit(code)
    elif cmd == "dtf-check":
        text, summary, code = dtf_check(rest[0])
        print(json.dumps(summary, indent=1) + "\n" if summary and "--json" in rest else text, end="")
        sys.exit(code)
    elif cmd == "threshold":
        phase = option(rest, "--phase", "ind-cal")
        tsv, why = calibrate(rest[0], phase) if phase in CALIBRATION.values() else (None, [f"no stage {phase}"])
        print(tsv or "Stopped, no threshold: " + "; ".join(why) + ".", end="" if tsv else "\n",
              file=sys.stdout if tsv else sys.stderr)
        sys.exit(0 if tsv else 1)
    elif cmd == "ind-val":
        phase = option(rest, "--phase", "ind-val")
        text, summary, code = (ind_val(rest[0], rest[1], phase) if phase in CALIBRATION
                               else (f"Stopped, not read: no stage {phase}.\n", None, 1))
        print(json.dumps(summary, indent=1) + "\n" if summary and "--json" in rest else text, end="")
        sys.exit(code)
    elif cmd == "report":
        phase = rest[rest.index("--phase") + 1] if "--phase" in rest else "confirm"
        text, code = report(rest[0], phase)
        print(text, end="")
        sys.exit(code)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])
