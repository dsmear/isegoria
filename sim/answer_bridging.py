"""Exploratory, not a validation: D49's count against the side-balanced score of `docs/02`
§A.3 (D32, D42) read on simulated answer-review rounds. Standard library only, seeded.
`python3 sim/answer_bridging.py [--quick] [--jobs N] [--out FILE.csv]`."""
import argparse
import csv
import math
import random
import sys
from multiprocessing import Pool

GROUP, QUORUM, NUM, DEN = 12, 9, 3, 4
LAM_B, LAM_F = 0.15, 0.03
TAUS = (0.75, 0.80)
MIN_COVERAGE = 1
MISSING = 0.10
LEAN_GAIN = 0.45
STARTS = 4
TOL = 1e-9
MAX_SWEEPS = 3000

KINDS = {
    "consensus": (0.90, 0.0),
    "hard": (0.65, 0.0),
    "wrong_key": (0.10, 0.0),
    "mild": (0.72, 0.30),
    "partisan_majority": (0.55, 0.80),
    "partisan_minority": (0.55, -0.80),
}
REST = (("consensus", 0.70), ("hard", 0.10), ("wrong_key", 0.10), ("mild", 0.10))

FULL = {"n": (50, 200), "share": (0.6, 0.7, 0.8), "history": (6, 12, 24, 48),
        "divisive": (0.05, 0.15, 0.30), "target": 200}
QUICK = {"n": (50,), "share": (0.7,), "history": (12, 24), "divisive": (0.15,), "target": 40}


def verdict(usable, coinciding):
    """D49's outcome, as `protocol::answer_rule::ConsensusRule::verdict`."""
    if usable < QUORUM:
        return "insufficient"
    if DEN * coinciding >= NUM * usable:
        return "accepted"
    if DEN * (usable - coinciding) >= NUM * usable:
        return "discarded"
    return "divided"


def kinds_of(questions, divisive):
    """The kinds of a run's questions: the divisive share split between the two camps."""
    n_div = max(2, round(questions * divisive))
    kinds = ["partisan_majority"] * ((n_div + 1) // 2) + ["partisan_minority"] * (n_div // 2)
    rest = questions - len(kinds)
    for name, share in REST[1:]:
        kinds += [name] * round(rest * share)
    return kinds + [REST[0][0]] * (questions - len(kinds))


def generate(n, share, history, divisive, rng):
    """One run: camps, drawn groups and answers. `obs` holds (reviewer, question, 0 or 1)
    for usable answers, 1 when the answer coincides with the key."""
    major = round(n * share)
    camp = [1] * major + [0] * (n - major)
    pos = [rng.gauss(1.0 if c else -1.0, 0.25) for c in camp]
    sev = [rng.gauss(0.0, 0.06) for _ in range(n)]
    kinds = kinds_of(n * history // GROUP, divisive)
    obs = []
    for j, kind in enumerate(kinds):
        q, lean = KINDS[kind]
        for u in rng.sample(range(n), GROUP):
            if rng.random() < MISSING:
                continue
            p = min(max(q + LEAN_GAIN * pos[u] * lean + sev[u], 0.0), 1.0)
            obs.append((u, j, 1.0 if rng.random() < p else 0.0))
    return camp, pos, kinds, obs


def index(n, m, obs):
    by_u = [[] for _ in range(n)]
    by_j = [[] for _ in range(m)]
    for u, j, r in obs:
        by_u[u].append((j, r))
        by_j[j].append((u, r))
    return by_u, by_j


def objective(by_u, params):
    mu, bu, bj, fu, fj = params
    cost = 0.0
    for u, row in enumerate(by_u):
        b, f = mu + bu[u], fu[u]
        for j, r in row:
            e = b + bj[j] + f * fj[j] - r
            cost += e * e
    sq = lambda v: sum(x * x for x in v)
    return cost + LAM_B * (sq(bu) + sq(bj)) + LAM_F * (sq(fu) + sq(fj))


def spectral_start(by_u, by_j, rng):
    """A start from the data: the intercepts alone, then the leading singular pair of their
    residuals by power iteration, split evenly between f_u and f_j."""
    n, m = len(by_u), len(by_j)
    total = sum(len(row) for row in by_u)
    mu = sum(r for row in by_u for _, r in row) / total
    bu, bj = [0.0] * n, [0.0] * m
    for _ in range(20):
        bu = [sum(r - mu - bj[j] for j, r in row) / (len(row) + LAM_B) for row in by_u]
        bj = [sum(r - mu - bu[u] for u, r in col) / (len(col) + LAM_B) for col in by_j]
    v = [rng.gauss(0.0, 1.0) for _ in range(m)]
    scale = 0.0
    for _ in range(60):
        left = [sum((r - mu - bu[u] - bj[j]) * v[j] for j, r in row)
                for u, row in enumerate(by_u)]
        norm = math.sqrt(sum(x * x for x in left)) or 1.0
        left = [x / norm for x in left]
        v = [sum((r - mu - bu[u] - bj[j]) * left[u] for u, r in col)
             for j, col in enumerate(by_j)]
        scale = math.sqrt(sum(x * x for x in v)) or 1.0
        v = [x / scale for x in v]
    root = math.sqrt(scale)
    return bu, bj, [root * x for x in left], [root * x for x in v]


def fit(by_u, by_j, seed, tol=TOL, spectral=False):
    """The bridging objective of `docs/02` §A (d = 1, equal weights, mu unpenalized) by exact
    block updates from a seeded start. Returns (objective, (mu, b_u, b_j, f_u, f_j))."""
    rng = random.Random(seed)
    n, m = len(by_u), len(by_j)
    total = sum(len(row) for row in by_u)
    mu = sum(r for row in by_u for _, r in row) / total
    if spectral:
        bu, bj, fu, fj = spectral_start(by_u, by_j, rng)
    else:
        bu = [rng.gauss(0.0, 0.1) for _ in range(n)]
        bj = [rng.gauss(0.0, 0.1) for _ in range(m)]
        fu = [rng.gauss(0.0, 0.3) for _ in range(n)]
        fj = [rng.gauss(0.0, 0.3) for _ in range(m)]
    prev = math.inf
    for sweep in range(MAX_SWEEPS):
        s = 0.0
        for u, row in enumerate(by_u):
            b, f = bu[u], fu[u]
            for j, r in row:
                s += r - b - bj[j] - f * fj[j]
        mu = s / total
        for u, row in enumerate(by_u):
            f = fu[u]
            bu[u] = sum(r - mu - bj[j] - f * fj[j] for j, r in row) / (len(row) + LAM_B)
        for j, col in enumerate(by_j):
            f = fj[j]
            bj[j] = sum(r - mu - bu[u] - fu[u] * f for u, r in col) / (len(col) + LAM_B)
        for u, row in enumerate(by_u):
            b, num, den = mu + bu[u], 0.0, LAM_F
            for j, r in row:
                g = fj[j]
                num += g * (r - b - bj[j])
                den += g * g
            fu[u] = num / den
        for j, col in enumerate(by_j):
            b, num, den = mu + bj[j], 0.0, LAM_F
            for u, r in col:
                g = fu[u]
                num += g * (r - b - bu[u])
                den += g * g
            fj[j] = num / den
        if sweep % 5 == 4:
            cost = objective(by_u, (mu, bu, bj, fu, fj))
            if prev - cost <= tol * max(cost, 1.0):
                break
            prev = cost
    params = (mu, bu, bj, fu, fj)
    return objective(by_u, params), params


def gradient_max(by_u, by_j, params):
    """The largest absolute partial derivative of the objective at `params`."""
    mu, bu, bj, fu, fj = params
    g_mu = 0.0
    g_bu = [2 * LAM_B * x for x in bu]
    g_bj = [2 * LAM_B * x for x in bj]
    g_fu = [2 * LAM_F * x for x in fu]
    g_fj = [2 * LAM_F * x for x in fj]
    for u, row in enumerate(by_u):
        for j, r in row:
            e = 2 * (mu + bu[u] + bj[j] + fu[u] * fj[j] - r)
            g_mu += e
            g_bu[u] += e
            g_bj[j] += e
            g_fu[u] += e * fj[j]
            g_fj[j] += e * fu[u]
    return max(abs(x) for x in [g_mu] + g_bu + g_bj + g_fu + g_fj)


def side_floor(n):
    """The fewest of n reviewers a side holds (D42): 5% rounded up, at least 1, at most n/2."""
    return min(max(-(-n * 50 // 1000), 1), n // 2)


def two_means(f):
    """D42's exact 1-D 2-means, as `sim/bridging_irt_dif.py`: labels 0 below the cut, 1 above;
    all 1 when no cut is admissible."""
    n = len(f)
    order = sorted(range(n), key=lambda i: (f[i], i))
    s = [f[i] for i in order]
    low = [0.0]
    for x in s:
        low.append(low[-1] + x)
    floor = max(side_floor(n), 1)
    best = None
    for k in range(floor, n - floor + 1):
        if not s[k - 1] < s[k]:
            continue
        d = (n - k) * low[k] - k * (low[n] - low[k])
        between = d * d / (k * (n - k))
        if best is None or between > best[0] or (
                between == best[0] and abs(2 * k - n) < abs(2 * best[1] - n)):
            best = (between, k)
    labels = [1] * n
    for i in order[:0 if best is None else best[1]]:
        labels[i] = 0
    return labels


def side_scores(params, by_u, by_j):
    """Per question: the side-balanced score S_j over the reviewers with a usable answer in
    the run, and the coverage (usable answers of its less-answering side). Also the labels."""
    mu, bu, bj, fu, fj = params
    seated = [u for u, row in enumerate(by_u) if row]
    cut = two_means([fu[u] for u in seated])
    label = {u: side for u, side in zip(seated, cut)}
    sides = [[u for u in seated if label[u] == k] for k in (0, 1)]
    if not sides[0]:
        sides[0] = sides[1]
    scores, coverage = [], []
    for j, col in enumerate(by_j):
        means = []
        for side in sides:
            base, g = mu + bj[j], fj[j]
            means.append(sum(min(max(base + bu[u] + fu[u] * g, 0.0), 1.0) for u in side)
                         / len(side))
        scores.append((means[0] + means[1]) / 2)
        counts = [sum(1 for u, _ in col if label[u] == k) for k in (0, 1)]
        coverage.append(min(counts))
    return scores, coverage, label


def correlation(x, y):
    n = len(x)
    mx, my = sum(x) / n, sum(y) / n
    sxy = sum((a - mx) * (b - my) for a, b in zip(x, y))
    sxx = sum((a - mx) ** 2 for a in x)
    syy = sum((b - my) ** 2 for b in y)
    return sxy / math.sqrt(sxx * syy) if sxx > 0 and syy > 0 else 0.0


def run(task):
    """One seeded run of a cell: per kind, the questions and how each rule decides them."""
    cell, rep, n, share, history, divisive = task
    seed = 1_000_003 * cell + rep
    camp, pos, kinds, obs = generate(n, share, history, divisive, random.Random(seed))
    by_u, by_j = index(n, len(kinds), obs)
    _, params = min((fit(by_u, by_j, 31 * seed + s, spectral=s == 0) for s in range(STARTS)),
                    key=lambda result: result[0])
    scores, coverage, label = side_scores(params, by_u, by_j)
    counts = {kind: dict.fromkeys(COLUMNS, 0) for kind in KINDS}
    for j, kind in enumerate(kinds):
        col = by_j[j]
        usable, coinciding = len(col), sum(int(r) for _, r in col)
        row = counts[kind]
        row["questions"] += 1
        outcome = verdict(usable, coinciding)
        row["insufficient"] += outcome == "insufficient"
        row["d49"] += outcome == "accepted"
        if usable < QUORUM:
            continue
        row["uncovered"] += coverage[j] < MIN_COVERAGE
        for tau in TAUS:
            row[f"sb{round(100 * tau)}"] += coverage[j] >= MIN_COVERAGE and scores[j] >= tau
        true_sides = [[r for u, r in col if camp[u] == k] for k in (0, 1)]
        if all(true_sides):
            balanced = sum(sum(side) / len(side) for side in true_sides) / 2
            row["known75"] += balanced >= TAUS[0]
    seated = sorted(label)
    agree = sum(label[u] == camp[u] for u in seated) / len(seated)
    axis = abs(correlation([params[3][u] for u in seated], [pos[u] for u in seated]))
    return cell, counts, axis, max(agree, 1 - agree), len(kinds)


COLUMNS = ("questions", "insufficient", "uncovered", "d49", "sb75", "sb80", "known75")


def check():
    """The count's boundaries (`answer_rule.rs`, AT-ANS-01), the side floor, the cut, and a
    stationary fit: the block updates end where the objective's gradient vanishes."""
    for usable, accept_from, discard_to in ((9, 7, 2), (10, 8, 2), (11, 9, 2), (12, 9, 3)):
        for c in range(usable + 1):
            want = "accepted" if c >= accept_from else "discarded" if c <= discard_to else "divided"
            assert verdict(usable, c) == want, (usable, c)
    assert verdict(8, 8) == "insufficient"
    assert [side_floor(n) for n in (1, 2, 20, 21, 200, 800)] == [0, 1, 1, 2, 10, 40]
    assert two_means([-1.0, -0.9, 1.0, 1.1, 0.95]) == [0, 0, 1, 1, 1]
    assert two_means([0.5] * 4) == [1] * 4
    _, _, kinds, obs = generate(40, 0.7, 24, 0.3, random.Random(5))
    by_u, by_j = index(40, len(kinds), obs)
    cost, params = fit(by_u, by_j, 0, tol=1e-14)
    assert gradient_max(by_u, by_j, params) < 1e-4 * cost, gradient_max(by_u, by_j, params)


def cells(grid):
    out = []
    for n in grid["n"]:
        for share in grid["share"]:
            for history in grid["history"]:
                for divisive in grid["divisive"]:
                    kinds = kinds_of(n * history // GROUP, divisive)
                    reps = -(-grid["target"] // kinds.count("partisan_majority"))
                    out.append((n, share, history, divisive, reps))
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--jobs", type=int, default=None)
    parser.add_argument("--out")
    args = parser.parse_args()
    check()
    grid = cells(QUICK if args.quick else FULL)
    tasks = [(c, rep, n, share, history, divisive)
             for c, (n, share, history, divisive, reps) in enumerate(grid)
             for rep in range(reps)]
    totals = [{"counts": {k: dict.fromkeys(COLUMNS, 0) for k in KINDS},
               "axis": 0.0, "sides": 0.0, "runs": 0, "per_run": 0} for _ in grid]
    with Pool(args.jobs) as pool:
        for cell, counts, axis, sides, questions in pool.imap(run, tasks, chunksize=4):
            t = totals[cell]
            for kind, row in counts.items():
                for column, value in row.items():
                    t["counts"][kind][column] += value
            t["axis"] += axis
            t["sides"] += sides
            t["runs"] += 1
            t["per_run"] = questions
    header = ["reviewers", "majority_share", "answers_per_reviewer", "divisive_share",
              "runs", "questions_per_run", "axis_corr", "side_accuracy", "kind", *COLUMNS]
    rows = []
    for (n, share, history, divisive, _), t in zip(grid, totals):
        for kind in KINDS:
            rows.append([n, share, history, divisive, t["runs"], t["per_run"],
                         f"{t['axis'] / t['runs']:.3f}", f"{t['sides'] / t['runs']:.3f}",
                         kind, *(t["counts"][kind][c] for c in COLUMNS)])
    if args.out:
        with open(args.out, "w", newline="") as f:
            csv.writer(f).writerows([header, *rows])
    csv.writer(sys.stdout).writerows([header, *rows])


if __name__ == "__main__":
    main()
