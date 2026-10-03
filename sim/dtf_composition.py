"""A2 (`docs/19`): the DTF figures of the dossier recomputed apart from `scoring::dtf` — the
continuous integrals beside the 41-node grid, and the three-class envelope example in exact
rationals. numpy and scipy; deterministic."""
from fractions import Fraction as F

import numpy as np
from scipy.integrate import quad


def gap(t, a, b, lean, c):
    low = c + (1 - c) / (1 + np.exp(-a * (t - (b - lean))))
    high = c + (1 - c) / (1 + np.exp(-a * (t - (b + lean))))
    return low - high


def continuous(items):
    density = lambda t: np.exp(-t * t / 2) / np.sqrt(2 * np.pi)
    value, _ = quad(lambda t: abs(sum(gap(t, *it) for it in items)) * density(t), -12, 12,
                    limit=400)
    return value


def grid(items):
    nodes = np.linspace(-5.0, 5.0, 41)
    w = np.exp(-nodes ** 2 / 2)
    w /= w.sum()
    return sum(wq * abs(sum(gap(t, *it) for it in items)) for t, wq in zip(nodes, w))


def main():
    for name, items in (("one item, gap 0.9, floor 0.2", [(1.25, 0.0, 0.45, 0.2)]),
                        ("one item, gap 1.8, difficulty 3", [(1.25, 3.0, 0.9, 0.0)]),
                        ("one item, gap 1.8, difficulty 1", [(1.25, 1.0, 0.9, 0.0)]),
                        ("one item, gap 1.8, difficulty 4", [(1.25, 4.0, 0.9, 0.0)]),
                        ("two items, gap 1.8, difficulty 2", [(1.25, 2.0, 0.9, 0.0)] * 2)):
        print(f"{name}: continuous {continuous(items):.10f}, 41-node grid {grid(items):.10f}")

    curves = {-1: (F(1), F(0), F(1, 2)), 1: (F(1, 2), F(0), F(1))}
    pairs = [(0, 1), (0, 2), (1, 2)]
    per_pair = {p: sum(F(1, 2) * abs(curves[t][p[0]] - curves[t][p[1]]) for t in curves)
                for p in pairs}
    envelope = sum(F(1, 2) * max(abs(curves[t][g] - curves[t][h]) for g, h in pairs)
                   for t in curves)
    groups = sum(F(1, 2) * abs(curves[t][0 if t < 0 else 2] - curves[t][1]) for t in curves)
    print(f"three classes, two points: per-pair {[str(v) for v in per_pair.values()]}, "
          f"max {max(per_pair.values())}, envelope {envelope}, the two groups' gap {groups}")


if __name__ == "__main__":
    main()
