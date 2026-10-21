"""Tests of the interval primitives of `a2_information_check.py` (`docs/19` §13.8).
Run: `python3 -m unittest discover -s sim -p test_a2_information_check.py -v`."""
import itertools
import math
import random
import statistics
import unittest
from decimal import Decimal, localcontext
from fractions import Fraction as F

import a2_information_check as A

PI_60 = Decimal("3.14159265358979323846264338327950288419716939937510582097494")


def encloses(x, v):
    with localcontext() as ctx:
        ctx.prec = 80
        return Decimal(x.lo) / Decimal(A.ONE) <= v <= Decimal(x.hi) / Decimal(A.ONE)


def dec(f):
    with localcontext() as ctx:
        ctx.prec = 80
        return Decimal(f.numerator) / Decimal(f.denominator)


class Primitives(unittest.TestCase):
    def test_exp_encloses_decimal_reference(self):
        rng = random.Random(7)
        for _ in range(200):
            v = F(rng.uniform(-14, 14)).limit_denominator(1 << 40)
            with localcontext() as ctx:
                ctx.prec = 80
                self.assertTrue(encloses(A.exp_iv(A.Iv.of(v)), dec(v).exp()))

    def test_ln_encloses_decimal_reference(self):
        rng = random.Random(11)
        for _ in range(200):
            v = F(rng.uniform(0.5, 4)).limit_denominator(1 << 40)
            with localcontext() as ctx:
                ctx.prec = 80
                self.assertTrue(encloses(A.ln_iv(A.Iv.of(v)), dec(v).ln()))

    def test_pi_and_sqrt_enclose_references(self):
        self.assertTrue(encloses(A.pi_iv(), PI_60))
        with localcontext() as ctx:
            ctx.prec = 80
            self.assertTrue(encloses(A.sqrt_iv(A.Iv.of(F(2))), Decimal(2).sqrt()))

    def test_quantiles_bracket_the_normal_quantile(self):
        c = 1 / A.sqrt_iv(2 * A.pi_iv())
        for p in (F(1, 32), F(3, 16), F(7, 8), F(31, 32)):
            q = A.quantile_iv(p, c)
            self.assertLess(F(A.phi_iv(A.Iv(q.lo, q.lo), c).hi, A.ONE), p)
            self.assertGreater(F(A.phi_iv(A.Iv(q.hi, q.hi), c).lo, A.ONE), p)
            self.assertAlmostEqual(q.mid(), statistics.NormalDist().inv_cdf(float(p)), places=14)

    def test_product_form_encloses_exact_enumeration(self):
        p, pp, pm = [F(1, 3), F(1, 2), F(3, 5)], [F(2, 5), F(3, 5), F(2, 3)], [F(1, 4), F(2, 5), F(1, 2)]
        law = lambda q, z: math.prod(x if b else 1 - x for x, b in zip(q, z))
        exact = sum(((law(pp, z) + law(pm, z)) / 2) ** 2 / law(p, z)
                    for z in itertools.product((0, 1), repeat=3))
        one = A.chi2_terms(*([A.Iv.of(x) for x in v] for v in (p, pp, pm)))
        self.assertTrue(F(one.lo, A.ONE) <= exact <= F(one.hi, A.ONE))

    def test_idealized_identity_is_enclosed(self):
        self.assertTrue(A.ideal_exact())

    def test_grid_weights_sum_to_one_and_mean_is_zero(self):
        w, _, mean, _ = A.make_grid()
        total = sum(w[1:], w[0])
        self.assertTrue(total.lo <= A.ONE <= total.hi)
        self.assertTrue(mean.lo <= 0 <= mean.hi)


if __name__ == "__main__":
    unittest.main()
