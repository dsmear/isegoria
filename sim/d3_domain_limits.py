"""D3's resource domain (`docs/15` D3): the limits `protocol::contested` enforces, the byte model
behind them, the documented configurations they admit or exclude, and the measured shapes.
Standard library only: `python3 sim/d3_domain_limits.py`."""
from math import comb

VISITS, MEMBER_ENTRIES, POINT_TERMS = 1 << 20, 1 << 22, 1 << 29
TABLE_CELLS, TABLE_SCANS = 1 << 20, 1 << 26
FITS, CLASSES, POINTS, ITEMS, POOL_BYTES = 1 << 12, 16, 1 << 12, 1 << 12, 1 << 27
EXTRA_BUDGET = 256 << 20  # the owner's target for a draw's memory beyond the pool
MMAP_THRESHOLD = 128 << 10
NODES = 41


def chunk(req):
    """Bytes an allocation of `req` holds under glibc: at most 16 more, or 1/31 more if mapped."""
    if req == 0:
        return 0
    return req + (16 if req < MMAP_THRESHOLD else -(-req // 31))


def cap(n):
    """Capacity of a `Vec` grown by pushes to `n` elements of at most 1 KiB."""
    return 0 if n == 0 else max(4, 1 << (n - 1).bit_length())


def stack_peak(m, n):
    """Most subsets on the enumeration's stack: `1 + Σ_{d < n'} (M − 1 − d)`."""
    return 1 + sum(m - 1 - d for d in range(min(m, n)))


def counts(fits, n):
    """`(V, W, P, T, R)` of a draw of `n` from `fits`, each `(M, G, Q)`; the check's order."""
    v = sum(comb(m, r) for m, _, _ in fits for r in range(min(m, n) + 1))
    w = sum(r * comb(m, r) for m, _, _ in fits for r in range(min(m, n) + 1))
    p = sum(comb(g, 2) * q * r * comb(m, r) for m, g, q in fits for r in range(min(m, n) + 1))
    return v, w, p, (len(fits) + 1) * (n + 1), (n + 1) * v


def refusal(fits, n):
    """The first limit `ContestedPool::admission` reports, or `None` when admitted."""
    if (len(fits) + 1) * (n + 1) > TABLE_CELLS:
        return "table cells"
    v = w = p = 0
    for m, g, q in fits:
        for r in range(min(m, n) + 1):
            c = comb(m, r)
            v, w, p = v + c, w + r * c, p + comb(g, 2) * q * r * c
            for name, value, limit in (("visits", v, VISITS), ("table scans", (n + 1) * v,
                                       TABLE_SCANS), ("member entries", w, MEMBER_ENTRIES),
                                      ("point terms", p, POINT_TERMS)):
                if value > limit:
                    return name
    return None


def draw_bytes(fits, n):
    """The model's peak of a draw's own allocations, every subset kept, allocator overhead in."""
    m = len(fits)
    total = sum(f[0] for f in fits)
    per_fit = [(sum(comb(f[0], r) for r in range(min(f[0], n) + 1)), f[0], min(f[0], n))
               for f in fits]
    base = chunk(8 * m) + chunk(24 * m) + sum(chunk(32 * cap(v)) for v, _, _ in per_fit)
    base += sum(comb(mm, r) * chunk(8 * max(2 * r - 2, 4))
                for _, mm, nn in per_fit for r in range(1, nn + 1))
    enum = max((chunk(32 * cap(v) // 2) + chunk(24 * cap(stack_peak(mm, nn)))
                + chunk(24 * cap(stack_peak(mm, nn)) // 2)
                + stack_peak(mm, nn) * chunk(8 * max(2 * nn - 2, 4)) + 2 * chunk(8 * nn)
                for v, mm, nn in per_fit), default=0)
    table = chunk(24 * (m + 1)) + (m + 1) * chunk(16 * (n + 1))
    vmax = max((v for v, _, _ in per_fit), default=0)
    opened = chunk(8 * cap(vmax)) + chunk(8 * cap(vmax) // 2)
    success = chunk(32 * n) + opened if n <= total else 0
    feasible = chunk(8 * cap(min(n, total) + 1)) + chunk(8 * cap(min(n, total) + 1) // 2)
    return base + max(enum, table + max(success, feasible))


def bytes_at_limits():
    """An upper bound on `draw_bytes` over every admitted request, from the inequalities."""
    def big(req, count=1):
        return -(-req * 32 // 31) + 16 * count
    base = big(8 * FITS) + big(24 * FITS) + big(32 * (2 * VISITS + 2 * FITS), FITS)
    base += 16 * MEMBER_ENTRIES + 32 * VISITS
    stack = 0
    for m in range(1, ITEMS + 1):
        for n in range(1, m + 1):
            if sum(comb(m, r) for r in range(n + 1)) > VISITS:
                break
            sk = stack_peak(m, n)
            stack = max(stack, 3 * big(24 * (2 * sk + 2)) // 2 + sk * (16 * n + 32) + 16 * n + 32)
    enum = big(16 * (2 * VISITS + 2)) + stack
    table = big(24 * (FITS + 1)) + big(16 * TABLE_CELLS, FITS + 1)
    passing = max(big(32 * (TABLE_CELLS // 2)) + 3 * big(8 * (2 * VISITS + 2)) // 2,
                  3 * big(8 * (2 * TABLE_CELLS + 2)) // 2)
    return base + max(enum, table + passing), base, enum, table, passing, stack


def fit_bytes(k, g, q, m):
    """The pool's estimate of one fit: its curves, weights and members, allocator overhead in."""
    return 8 * k * g * q + 40 * k * g + 40 * k + 8 * q + 42 * m + 48


POOL_FIXED = 240 * FITS  # the pool's buffer of fits, with a reallocation's old copy


def default(m, g, n):
    return [(m, g, g * NODES)], n


CONFIGS = [
    ("`contested_facts.rs` `pool()`", [(5, 2, 41), (3, 2, 41), (3, 3, 41), (1, 2, 41)], range(8)),
    ("AT-PRO-08 on fitted batches, four classes", [(3, 4, 164), (3, 4, 164)], [2]),
    ("`a2_dtf_composition.rs`", [(1, 2, 82), (1, 2, 82)], [2]),
    ("one batch of 8, all contested (`02` §B.6)", [(8, 4, 164)], [8]),
    ("one batch of 16 (SC-2's largest `K`), G = 2", [(16, 2, 82)], [8, 16]),
    ("one batch of 16 (SC-2's largest `K`), G = 4", [(16, 4, 164)], [8, 16]),
    ("re-validation fit of 20 (`19` §9.3), G = 2", [(20, 2, 82)], [5, 8, 10, 20]),
    ("re-validation fit of 20 (`19` §9.3), G = 4", [(20, 4, 164)], [5, 8, 10, 20]),
    ("re-validation fit of 40 (`19` §9.3), G = 2", [(40, 2, 82)], [4, 5, 10]),
    ("re-validation fit of 40 (`19` §9.3), G = 4", [(40, 4, 164)], [4, 5, 10]),
    ("one class, 20 members", [(20, 1, 41)], [10, 20]),
    ("4,096 fits of one member (the fit limit)", [(1, 2, 82)] * FITS, [1, 255, 256]),
]

MEASURED = [
    ("kept: one class, M = 70", [(70, 1, 41)], 4),
    ("member entries: one class, 8 fits of 16", [(16, 1, 41)] * 8, 16),
    ("point terms: four classes, M = 16", [(16, 4, 164)], 16),
    ("table cells: one fit of 2", [(2, 1, 41)], (1 << 19) - 1),
    ("table cells: 1,023 fits of 1", [(1, 1, 41)] * 1023, 1023),
    ("table scans: one class, M = 16", [(16, 1, 41)], 1023),
    ("combined: 8 fits of 16, two classes", [(16, 2, 82)] * 8, 127),
]


def main():
    total, base, enum, table, passing, stack = bytes_at_limits()
    print(f"draw bytes at the limits <= {total:,} ({total / (1 << 20):.1f} MiB) of "
          f"{EXTRA_BUDGET >> 20} MiB: base {base:,}, enumeration {enum:,} (stack {stack:,}), "
          f"table {table:,}, result and open list {passing:,}")
    one = fit_bytes(16, 4, 4 * NODES, 16)
    fits = (POOL_BYTES - POOL_FIXED) // one
    print(f"pool: a default fit of 16 (G = 4, Q = 164) {one:,} bytes; {fits:,} such fits "
          f"({16 * fits:,} facts) within {POOL_BYTES >> 20} MiB; a fit of 40: "
          f"{fit_bytes(40, 4, 164, 40):,} bytes")
    print("\n| Configuration | n | V | W | P | T | R | model MiB | admitted |")
    print("|---|---|---:|---:|---:|---:|---:|---:|---|")
    rows = CONFIGS + [(f"pool at its limit: {fits:,} default fits", [(16, 4, 164)] * fits,
                       [1, 2, 3])]
    for label, shape, ns in rows:
        for n in ns:
            v, w, p, t, r = counts(shape, n)
            why = refusal(shape, n)
            mib = f"{draw_bytes(shape, n) / (1 << 20):.1f}" if why is None else "—"
            verdict = "yes" if why is None else f"no: {why}"
            print(f"| {label} | {n} | {v:,} | {w:,} | {p:,} | {t:,} | {r:,} | {mib} | {verdict} |")
    print("\n| Measured shape | n | V | W | P | T | R | model MiB | admitted |")
    print("|---|---|---:|---:|---:|---:|---:|---:|---|")
    for label, shape, n in MEASURED:
        v, w, p, t, r = counts(shape, n)
        why = refusal(shape, n) or "yes"
        print(f"| {label} | {n} | {v:,} | {w:,} | {p:,} | {t:,} | {r:,} | "
              f"{draw_bytes(shape, n) / (1 << 20):.1f} | {why} |")


if __name__ == "__main__":
    main()
