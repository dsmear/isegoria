"""D3's exact counts (`docs/15` D3): what `ContestedPool::draw` enumerates per fit, for the
configurations the repository motivates. Standard library only:
`python3 sim/d3_selection_counts.py`."""
from math import comb

NODES = 41  # `LatentParams::default().nodes`
MAX_CLASSES = 4  # `LatentParams::default().max_classes`


def subsets(m, n):
    """Subsets of at most `n` of `m` members, the empty one included: `S(m, n)`."""
    return sum(comb(m, r) for r in range(min(m, n) + 1))


def evaluations(m, n):
    """Member terms over those subsets, `Σ r C(m, r)`: the work `ClassCurves::dtf` scales with."""
    return sum(r * comb(m, r) for r in range(min(m, n) + 1))


def dfs_visits(m, n):
    """A transcription of `ContestedPool::candidates`' loop, counting the subsets it evaluates."""
    visits, stack = 0, [[]]
    while stack:
        members = stack.pop()
        if len(members) < n:
            nxt = members[-1] + 1 if members else 0
            stack.extend(members + [k] for k in reversed(range(nxt, m)))
        visits += 1
    return visits


def point_terms(classes):
    """Class pairs times ability points, per member term: `C(G, 2) · G · NODES`."""
    return comb(classes, 2) * classes * NODES


POOLS = [
    ("`contested_facts.rs` `pool()`, n = 2", [5, 3, 3, 1], 2),
    ("`contested_facts.rs` `pool()`, n = 6", [5, 3, 3, 1], 6),
    ("AT-PRO-08 on fitted batches (`calibration`), n = 2", [3, 3], 2),
    ("`a2_dtf_composition.rs`, n = 2", [1, 1], 2),
]
FITS = [
    ("one batch of 8 trial items, all contested (`02` §B.6)", 8, [2, 4, 8]),
    ("one batch of 16, SC-2's largest `K` (`08` §16.1)", 16, [2, 4, 8, 16]),
    ("a re-validation fit of 20 contested facts (`19` §9.3)", 20, [5, 10, 20]),
    ("a re-validation fit of 40 contested facts (`19` §9.3)", 40, [5, 10, 20]),
]


def main():
    for m in range(15):
        for n in range(16):
            assert dfs_visits(m, n) == subsets(m, n), (m, n)
    print("transcription check: the loop visits S(M, n) subsets for M <= 14, n <= 15")
    print(f"point terms per member term: G=2 {point_terms(2)}, G={MAX_CLASSES} "
          f"{point_terms(MAX_CLASSES)}")
    print("\n| Pool | fits' M | n | subsets | member terms |\n|---|---|---|---:|---:|")
    for label, ms, n in POOLS:
        s = sum(subsets(m, n) for m in ms)
        e = sum(evaluations(m, n) for m in ms)
        print(f"| {label} | {', '.join(map(str, ms))} | {n} | {s:,} | {e:,} |")
    print("\n| One fit | M | n | subsets | member terms |\n|---|---|---|---:|---:|")
    for label, m, ns in FITS:
        for n in ns:
            print(f"| {label} | {m} | {n} | {subsets(m, n):,} | {evaluations(m, n):,} |")


if __name__ == "__main__":
    main()
