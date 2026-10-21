"""Recomputes the sample-size reading from the records alone, apart from `sim/` (`docs/19`
§13.27): per replicate `I` at both sizes, separations, moves and placements, against `size.json`."""
import json
import sys

recs = {r["key"]: r for r in (json.loads(line) for line in open(sys.argv[1], encoding="utf-8") if line.strip())}
got = json.load(open(sys.argv[2], encoding="utf-8"))
SIZES, CASES = (3000, 6000), ("C0", "C3")


def indicator(rec):
    shared = [c for c in rec["candidates"] if c["classes"] == 2 and not c["non_uniform"]]
    ok = (rec["outcome"] == "read" and rec["candidates"][0]["status"] == "converged" and shared
          and shared[0]["status"] == "converged" and shared[0]["dtf"] is not None)
    return shared[0]["dtf"] if ok else None


def share(hi, lo):
    return sum((a > b) + 0.5 * (a == b) for a in hi for b in lo) / (len(hi) * len(lo)) if hi and lo else None


I = {(n, c, r): indicator(recs[f"size-n{n}-{c}-{r}"]) for n in SIZES for c in CASES for r in range(16)}
fmt = lambda v: "—" if v is None else f"{v:.4f}"
for c in CASES:
    print(c, "replicate: I at 3000 / I at 6000")
    print("  " + "; ".join(f"r{r} {fmt(I[(3000, c, r)])} / {fmt(I[(6000, c, r)])}" for r in range(16)))
same = True
common = {c: [r for r in range(16) if all(I[(n, c, r)] is not None for n in SIZES)] for c in CASES}
for name, reps in (("all", {c: range(16) for c in CASES}), ("common", common)):
    for n in SIZES:
        hi = [I[(n, "C3", r)] for r in reps["C3"] if I[(n, "C3", r)] is not None]
        lo = [I[(n, "C0", r)] for r in reps["C0"] if I[(n, "C0", r)] is not None]
        mine = share(hi, lo)
        same &= mine == got[name][str(n)]["separation"]
        print(f"{name} N {n}: {mine!r} over {len(hi)} x {len(lo)}")
for c in CASES:
    moves = {"greater": 0, "smaller": 0, "equal": 0}
    for r in common[c]:
        a, b = I[(3000, c, r)], I[(6000, c, r)]
        moves["greater" if b > a else "smaller" if b < a else "equal"] += 1
    same &= moves == got["replicates"][c]["I_6000_against_3000"]
    print(c, "moves", moves)
for name, refs in (("every evaluable C0", range(16)), ("the common C0", common["C0"])):
    rows = [(r, [share([I[(n, "C3", r)]], [I[(n, "C0", q)] for q in refs if I[(n, "C0", q)] is not None]) for n in SIZES])
            for r in common["C3"]]
    same &= [p for _, p in rows] == [[x["3000"], x["6000"]] for x in got["placements"][name]["rows"]]
    print(name, "placements", "; ".join(f"r{r} {p[0]:.3f}->{p[1]:.3f}" for r, p in rows))
print("equal to size.json:", same)
