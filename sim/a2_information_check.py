"""A2 two-point information check (`docs/19` §§13.5, 13.8): bounds certified by outward-rounded
fixed-point intervals on exact integers, and floating-point consistency checks.
Standard library only, deterministic; `python3 sim/a2_information_check.py > result.json`."""
import json
import math
import platform
import statistics
import sys
import time
from fractions import Fraction as F

P = 128
ONE = 1 << P
EPS = F(1, 1 << (P + 4))
FORMS = (4, 8, 16)
FLOORS = (F(0), F(1, 4), F(1, 2))
DELTAS = (F(1, 10), F(2, 10), F(4, 10))
SIZES = (300, 1500, 3000, 6000)
SLOPE = F(5, 4)
SPREAD = F(3, 5)
NODES, THETA_MAX = 41, 5
TAU_BITS, TAU_BUMPS = 30, 8
BRACKET, BISECTIONS = F(1, 1 << 40), 64
ANCHORS = ((F(5, 4), F(-3, 5)), (F(5, 4), F(3, 5)))
CHI2_REL_TOL = 1e-8
ORDER_BAND = (0.1, 10.0)
IDEAL = [(m, p, d) for m in FORMS for p in (F(1, 4), F(1, 2)) for d in (F(1, 64), F(1, 16))]


class Stop(Exception):
    pass


def _fl(x):
    return (x.numerator << P) // x.denominator


def _cl(x):
    return -((-x.numerator << P) // x.denominator)


class Iv:
    """A closed interval `[lo, hi] / 2^P`; every operation rounds its endpoints outward."""
    __slots__ = ("lo", "hi")

    def __init__(self, lo, hi):
        if lo > hi:
            raise Stop("inverted interval")
        self.lo, self.hi = lo, hi

    @staticmethod
    def of(x):
        x = F(x)
        return Iv(_fl(x), _cl(x))

    def __add__(self, o):
        o = iv(o)
        return Iv(self.lo + o.lo, self.hi + o.hi)

    __radd__ = __add__

    def __neg__(self):
        return Iv(-self.hi, -self.lo)

    def __sub__(self, o):
        o = iv(o)
        return Iv(self.lo - o.hi, self.hi - o.lo)

    def __rsub__(self, o):
        return iv(o) - self

    def __mul__(self, o):
        o = iv(o)
        ps = (self.lo * o.lo, self.lo * o.hi, self.hi * o.lo, self.hi * o.hi)
        return Iv(min(ps) >> P, -(-max(ps) >> P))

    __rmul__ = __mul__

    def __truediv__(self, o):
        o = iv(o)
        if o.lo <= 0 <= o.hi:
            raise Stop("interval divisor contains 0")
        qs = [(a << P, b) for a in (self.lo, self.hi) for b in (o.lo, o.hi)]
        return Iv(min(a // b for a, b in qs), max(-(-a // b) for a, b in qs))

    def __rtruediv__(self, o):
        return iv(o) / self

    def sqr(self):
        a, b = sorted((abs(self.lo), abs(self.hi)))
        low = 0 if self.lo <= 0 <= self.hi else a * a >> P
        return Iv(low, -(-(b * b) >> P))

    def mag(self):
        return F(max(abs(self.lo), abs(self.hi)), ONE)

    def mid(self):
        return float(F(self.lo + self.hi, 2 * ONE))


def iv(x):
    if isinstance(x, Iv):
        return x
    if isinstance(x, int):
        return Iv(x << P, x << P)
    return Iv.of(x)


def sqrt_iv(x):
    if x.lo < 0:
        raise Stop("square root of a negative interval")
    n = x.hi << P
    hi = math.isqrt(n)
    return Iv(math.isqrt(x.lo << P), hi + (hi * hi < n))


def _exp_terms():
    n = 1
    while 2 * F(1, 8) ** n / math.factorial(n) > EPS:
        n += 1
    return n, _cl(2 * F(1, 8) ** n / math.factorial(n))


EXP_TERMS, EXP_REM = _exp_terms()


def exp_point(xs):
    """Encloses `exp(xs / 2^P)`: Taylor at `|y| ≤ 1/8` with its remainder, then squarings."""
    s = (abs(xs) >> P).bit_length() + 3
    y = Iv(xs >> s, -(-xs >> s))
    t = iv(1)
    for k in range(EXP_TERMS - 1, 0, -1):
        t = 1 + y * t / k
    t = Iv(t.lo - EXP_REM, t.hi + EXP_REM)
    for _ in range(s):
        t = t.sqr()
    return t


def exp_iv(x):
    return Iv(exp_point(x.lo).lo, exp_point(x.hi).hi)


def ln_iv(y):
    if y.lo <= 0:
        raise Stop("logarithm of a non-positive interval")
    t = (y - 1) / (y + 1)
    tm = t.mag()
    if tm >= 1:
        raise Stop("logarithm series out of range")
    n = 1
    while tm ** (2 * n + 1) / ((2 * n + 1) * (1 - tm * tm)) > EPS:
        n += 1
    rem = _cl(tm ** (2 * n + 1) / ((2 * n + 1) * (1 - tm * tm)))
    t2, s = t.sqr(), Iv.of(F(1, 2 * n - 1))
    for k in range(n - 2, -1, -1):
        s = Iv.of(F(1, 2 * k + 1)) + t2 * s
    s = t * s
    return 2 * Iv(s.lo - rem, s.hi + rem)


def sigmoid_iv(z):
    return 1 / (1 + exp_iv(-z))


def pi_iv():
    def atan_inv(k, n):
        s = sum(F((-1) ** i, (2 * i + 1) * k ** (2 * i + 1)) for i in range(n))
        return s, F(1, (2 * n + 1) * k ** (2 * n + 1))
    a, ea = atan_inv(5, 60)
    b, eb = atan_inv(239, 20)
    v, e = 16 * a - 4 * b, 16 * ea + 4 * eb
    return Iv(_fl(v - e), _cl(v + e))


def phi_iv(x, c):
    """Encloses the normal CDF on `x`, `c` enclosing `1/√(2π)`: alternating series with a
    geometric bound on its tail once `x² < 2(n + 1)`."""
    xm = x.mag()
    n = 1
    while True:
        r = xm * xm / (2 * (n + 1))
        if r < 1:
            tail = xm ** (2 * n + 1) / (2 ** n * math.factorial(n) * (2 * n + 1)) / (1 - r)
            if tail <= EPS:
                break
        n += 1
    z = -x.sqr() / 2
    s = Iv.of(F(1, math.factorial(n - 1) * (2 * n - 1)))
    for k in range(n - 2, -1, -1):
        s = Iv.of(F(1, math.factorial(k) * (2 * k + 1))) + z * s
    s = x * s
    rem = _cl(tail)
    return F(1, 2) + c * Iv(s.lo - rem, s.hi + rem)


def quantile_iv(p, c):
    """Encloses `Φ⁻¹(p)` between dyadic points certified below and above `p`; the float
    guess only seeds the bracket."""
    g = F(statistics.NormalDist().inv_cdf(float(p)))
    lo, hi = g - BRACKET, g + BRACKET
    if not (F(phi_iv(Iv.of(lo), c).hi, ONE) < p < F(phi_iv(Iv.of(hi), c).lo, ONE)):
        raise Stop(f"quantile bracket not certified at p = {p}")
    for _ in range(BISECTIONS):
        mid = (lo + hi) / 2
        v = phi_iv(Iv.of(mid), c)
        if F(v.hi, ONE) < p:
            lo = mid
        elif F(v.lo, ONE) > p:
            hi = mid
        else:
            break
    return Iv(_fl(lo), _cl(hi))


def make_grid():
    theta = [F(-THETA_MAX) + F(2 * THETA_MAX * i, NODES - 1) for i in range(NODES)]
    e = [exp_iv(Iv.of(-t * t / 2)) for t in theta]
    z = sum(e[1:], e[0])
    w = [x / z for x in e]
    mean = sum((wi * t for wi, t in zip(w[1:], theta[1:])), w[0] * theta[0])
    var = sum((wi * (t - mean).sqr() for wi, t in zip(w[1:], theta[1:])),
              w[0] * (theta[0] - mean).sqr())
    sd = sqrt_iv(var)
    return w, [(t - mean) / sd for t in theta], mean, sd


def chi2_terms(p, pp, pm):
    """`1 + χ²(½ I⁺ + ½ I⁻ ‖ I)` by the product form, from lists of item probabilities."""
    def s(x, y, b):
        return x * y / b + (1 - x) * (1 - y) / (1 - b)
    spp = spm = smm = iv(1)
    for b, x, y in zip(p, pp, pm):
        spp, spm, smm = spp * s(x, x, b), spm * s(x, y, b), smm * s(y, y, b)
    return spp / 4 + spm / 2 + smm / 4


def float_contrast(tau, c, w, u, b):
    a = float(SLOPE)
    sig = lambda z: 1 / (1 + math.exp(-z)) if z >= 0 else math.exp(z) / (1 + math.exp(z))
    return (1 - c) * math.fsum(wq * (sig(a * (uq - bj + tau)) - sig(a * (uq - bj - tau)))
                               for wq, uq in zip(w, u) for bj in b)


def float_root(delta, c, w, u, b):
    hi = 1.0
    while float_contrast(hi, c, w, u, b) <= delta:
        hi *= 2
    lo = 0.0
    while True:
        mid = (lo + hi) / 2
        if mid in (lo, hi):
            return hi
        lo, hi = (mid, hi) if float_contrast(mid, c, w, u, b) <= delta else (lo, mid)


def certify(c, tau, w, u, b, base):
    """The certified `D(τ)`, `K` and per-node `1 + χ²` and curves, at an exact dyadic `τ`."""
    t = Iv.of(tau)
    d, k, nodes = iv(0), iv(0), []
    for wq, uq, bq in zip(w, u, base):
        p = [c + (1 - c) * s for s in bq]
        pp = [c + (1 - c) * sigmoid_iv(SLOPE * (uq - bj + t)) for bj in b]
        pm = [c + (1 - c) * sigmoid_iv(SLOPE * (uq - bj - t)) for bj in b]
        d = d + wq * sum((x - y for x, y in zip(pp[1:], pm[1:])), pp[0] - pm[0])
        one = chi2_terms(p, pp, pm)
        k = k + wq * ln_iv(one)
        nodes.append((one, p, pp, pm))
    return d, k, nodes


def frontier(kbar, n):
    x = n * kbar
    low = exp_point(-x).lo
    bh = sqrt_iv(iv(1) - Iv(low, low)).hi
    pin = sqrt_iv(Iv(-(-x >> 1), -(-x >> 1))).hi
    delta = min(ONE, bh, pin)
    return delta, ONE - delta


def kl_term(r):
    if abs(r) < 1e-2:
        return r * r * (0.5 - r * (1 / 6 - r * (1 / 12 - r * (1 / 20 - r * (1 / 30 - r * (
            1 / 42 - r / 56))))))
    return (1 + r) * math.log1p(r) - r


def enumerate_node(p, pp, pm):
    """Per pattern, by exhaustive enumeration: `I(z)` and `r(z) = M(z)/I(z) − 1`."""
    law, rp, rm = [1.0], [1.0], [1.0]
    for b, x, y in zip(p, pp, pm):
        law = [v * (1 - b) for v in law] + [v * b for v in law]
        rp = [v * ((1 - x) / (1 - b)) for v in rp] + [v * (x / b) for v in rp]
        rm = [v * ((1 - y) / (1 - b)) for v in rm] + [v * (y / b) for v in rm]
    return law, [0.5 * (s + t) - 1.0 for s, t in zip(rp, rm)]


def checks(m, nodes, w, u, kmid, kbar):
    """Floating-point consistency checks of one configuration (not certification)."""
    wf = [x.mid() for x in w]
    combos = [(z1, z2) for z1 in (0, 1) for z2 in (0, 1)]
    size = 1 << m
    p0 = {za: [0.0] * size for za in combos}
    num = {za: [0.0] * size for za in combos}
    worst, ideal = 0.0, 0.0
    for wq, uq, (one, p, pp, pm) in zip(wf, u, nodes):
        pf, ppf, pmf = ([x.mid() for x in v] for v in (p, pp, pm))
        law, r = enumerate_node(pf, ppf, pmf)
        chi2 = math.fsum(i * x * x for i, x in zip(law, r))
        ref = F(one.lo + one.hi, 2 * ONE) - 1
        worst = max(worst, abs(chi2 - float(ref)) / float(ref))
        lr = [i * x for i, x in zip(law, r)]
        ua = uq.mid()
        anchor = [1 / (1 + math.exp(-float(a) * (ua - float(b)))) for a, b in ANCHORS]
        for za in combos:
            mass = wq * math.prod(anchor[i] if z else 1 - anchor[i] for i, z in enumerate(za))
            p0[za] = [s + mass * x for s, x in zip(p0[za], law)]
            num[za] = [s + mass * x for s, x in zip(num[za], lr)]
        dq = math.fsum(x - y for x, y in zip(ppf, pmf))
        pbar = math.fsum(pf) / m
        ideal += wq * (m - 1) * dq ** 4 / (32 * m ** 3 * pbar ** 2 * (1 - pbar) ** 2)
    with_anchors = math.fsum(s * kl_term(t / s) for za in combos for s, t in zip(p0[za], num[za]))
    q0 = [math.fsum(v) for v in zip(*(p0[za] for za in combos))]
    qn = [math.fsum(v) for v in zip(*(num[za] for za in combos))]
    without = math.fsum(s * kl_term(t / s) for s, t in zip(q0, qn))
    kbar_f = float(F(kbar, ONE))
    return {
        "chi2_product_vs_enumeration_max_rel_dev": f"{worst:.3e}",
        "kl_no_anchor_over_Kbar": f"{without / kbar_f:.6f}",
        "kl_two_anchors_over_Kbar": f"{with_anchors / kbar_f:.6f}",
        "K_over_idealized_leading_term": f"{kmid / ideal:.6f}",
        "passed": bool(worst <= CHI2_REL_TOL and 0 <= without <= kbar_f
                       and 0 <= with_anchors <= kbar_f
                       and ORDER_BAND[0] <= kmid / ideal <= ORDER_BAND[1]),
    }


def ideal_exact():
    """§13.4's idealized identity: the certified product form encloses the exact rational
    `½[(1 + e)^m + (1 − e)^m]`."""
    for m, p, d in IDEAL:
        one = chi2_terms([Iv.of(p)] * m, [Iv.of(p + d)] * m, [Iv.of(p - d)] * m)
        e = d * d / (p * (1 - p))
        exact = ((1 + e) ** m + (1 - e) ** m) / 2
        if not F(one.lo, ONE) <= exact <= F(one.hi, ONE):
            return False
    return True


def sci(x, up, digits=10):
    if x == 0:
        return "0"
    a, e = abs(x), len(str(abs(x.numerator))) - len(str(x.denominator))
    while a >= F(10) ** (e + 1):
        e += 1
    while a < F(10) ** e:
        e -= 1
    v = x / F(10) ** (e - digits + 1)
    k = -((-v.numerator) // v.denominator) if up else v.numerator // v.denominator
    s = str(abs(k))
    sign = "-" if k < 0 else ""
    return f"{sign}{s[0]}.{s[1:]}e{e - digits + 1 + len(s) - 1:+03d}"


def span(x):
    return [sci(F(x.lo, ONE), False), sci(F(x.hi, ONE), True)]


def run():
    clock = time.perf_counter()
    pi = pi_iv()
    c = 1 / sqrt_iv(2 * pi)
    w, u, mean, sd = make_grid()
    quant = {m: [quantile_iv(F(2 * j - 1, 2 * m), c) for j in range(1, m + 1)] for m in FORMS}
    inputs_s = time.perf_counter() - clock
    inputs = {
        "pi": span(pi), "grid_mean": span(mean), "grid_sd": span(sd),
        "max_weight_width": sci(max(F(x.hi - x.lo, ONE) for x in w), True, 3),
        "max_node_width": sci(max(F(x.hi - x.lo, ONE) for x in u), True, 3),
        "max_quantile_width": sci(max(F(x.hi - x.lo, ONE) for v in quant.values() for x in v),
                                  True, 3),
    }
    wf, uf = [x.mid() for x in w], [x.mid() for x in u]
    configs, cert_s, check_s, passed, uniform = [], 0.0, 0.0, True, {}
    for m in FORMS:
        b = [Iv.of(SPREAD) * x for x in quant[m]]
        bf = [x.mid() for x in b]
        base = [[sigmoid_iv(SLOPE * (uq - bj)) for bj in b] for uq in u]
        for floor in FLOORS:
            for delta in DELTAS:
                t0 = time.perf_counter()
                root = float_root(float(delta), float(floor), wf, uf, bf)
                step = math.ceil(math.ldexp(root, TAU_BITS)) + 1
                for _ in range(TAU_BUMPS):
                    tau = F(step, 1 << TAU_BITS)
                    d, k, nodes = certify(floor, tau, w, u, b, base)
                    if F(d.lo, ONE) > delta:
                        break
                    step += 1
                else:
                    raise Stop(f"no certified contrast above {delta} at m={m}, floor={floor}")
                if k.lo <= 0:
                    raise Stop(f"K not certified positive at m={m}, floor={floor}")
                sizes = []
                for n in SIZES:
                    dn, fr = frontier(k.hi, n)
                    uniform[(delta, n)] = min(fr, uniform.get((delta, n), ONE))
                    sizes.append({"n": n, "Delta_bar_up": sci(F(dn, ONE), True),
                                  "frontier_1_minus_Delta_bar_down": sci(F(fr, ONE), False)})
                t1 = time.perf_counter()
                cert_s += t1 - t0
                chk = checks(m, nodes, w, u, (k.lo + k.hi) / (2 * ONE), k.hi)
                check_s += time.perf_counter() - t1
                passed = passed and chk["passed"]
                configs.append({
                    "m": m, "floor": str(floor), "delta": str(delta),
                    "tau_delta": f"{step}/2^{TAU_BITS}", "tau_delta_decimal": sci(tau, False),
                    "D_tau": span(d), "K": span(k),
                    "size_coefficient_1_over_Kbar_down": sci(F(ONE * ONE // k.hi, ONE), False),
                    "sizes": sizes, "checks": chk,
                })
                if not chk["passed"]:
                    raise Stop(f"consistency check failed at m={m}, floor={floor}, delta={delta}")
    t2 = time.perf_counter()
    ideal = ideal_exact()
    passed = passed and ideal
    if not ideal:
        raise Stop("idealized identity not enclosed")
    total = time.perf_counter() - clock
    return {
        "contract": "docs/19 §13.5; result in §13.8",
        "precision_bits": P,
        "declared": {
            "forms_m": list(FORMS), "floors": [str(x) for x in FLOORS],
            "deltas": [str(x) for x in DELTAS], "sizes": list(SIZES), "slope": str(SLOPE),
            "difficulty": "0.6 Phi^-1((j - 1/2)/m)", "grid": f"{NODES} nodes on [-5, 5]",
            "tau_grid": f"2^-{TAU_BITS}", "anchors_check": [[str(a), str(b)] for a, b in ANCHORS],
            "chi2_rel_tol": CHI2_REL_TOL, "order_band": list(ORDER_BAND),
        },
        "inputs": inputs,
        "label": "every value certified under the numerical contract",
        "reading": {
            "excluded": "alpha + beta < frontier, at that configuration and size only",
            "void": "alpha + beta >= 1 constrains no n",
            "K_zero_branch": "not reached: K certified > 0 in every configuration",
            "sizes": "n >= max{2 s^2, ln(1/(1 - s^2))} * coefficient, s = 1 - alpha - beta in "
                     "(0, 1]; s = 1 needs infinite n; levels stay symbolic",
        },
        "uniform_frontier_down": [
            {"delta": str(dl), "n": n, "min_over_forms_and_floors": sci(F(uniform[(dl, n)], ONE),
                                                                        False)}
            for dl in DELTAS for n in SIZES],
        "configurations": configs,
        "idealized_identity_exact_cases": len(IDEAL),
        "idealized_identity_enclosed": ideal,
        "all_checks_passed": passed,
        "run": {
            "python": platform.python_version(), "platform": platform.platform(),
            "cpu": cpu(), "seconds_inputs": round(inputs_s, 2),
            "seconds_certification": round(cert_s, 2), "seconds_checks": round(check_s, 2),
            "seconds_idealized": round(time.perf_counter() - t2, 2), "seconds_total": round(total, 2),
        },
    }


def cpu():
    try:
        with open("/proc/cpuinfo", encoding="utf-8") as f:
            for line in f:
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or "unknown"


def main():
    try:
        result = run()
    except Stop as e:
        print(json.dumps({"stopped": str(e)}), file=sys.stderr)
        return 1
    print(json.dumps(result, indent=1, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
