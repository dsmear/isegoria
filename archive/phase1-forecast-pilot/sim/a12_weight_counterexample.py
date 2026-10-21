#!/usr/bin/env python3
"""A12's mathematical reproducer; see docs/16 §7.1 for the declared item-level bridge.
Checked transcription of track/cap formulas, not a Rust or bitwise fmath test.
"""

from dataclasses import dataclass
from decimal import Decimal, localcontext
from fractions import Fraction as F
from math import factorial


GAMMA, K0, PROBATION = 35, 100, 30
ALLOWANCE, THRESHOLD = F(3, 100), F(3, 2)
Q, BASELINE = F(4, 5), F(1, 2)
LAMBDA = F(GAMMA, 31 + K0)


def decimal(value):
    return Decimal(value.numerator) / Decimal(value.denominator)


def difference_score(report, baseline, outcome):
    return (baseline - outcome) ** 2 - (report - outcome) ** 2


@dataclass
class Track:
    total: F = F(0)
    plain: F = F(0)
    reviewed: int = 0
    scored: int = 0
    cusum: F = F(0)
    alarms: int = 0

    def record(self, score):
        reference = self.plain / self.scored if self.scored else F(0)
        if self.scored >= PROBATION:
            self.cusum = max(F(0), self.cusum + reference - score - ALLOWANCE)
            if self.cusum > THRESHOLD:
                self.total = self.plain = self.cusum = F(0)
                self.reviewed = self.scored = 0
                self.alarms += 1
                return True
        self.total += score
        self.plain += score
        self.reviewed += 1
        self.scored += 1
        return False

    def raw_weight(self):
        assert self.scored >= PROBATION
        skill = self.total / self.reviewed
        return decimal(GAMMA * skill * self.scored / (self.scored + K0)).exp()


def initial_roster():
    roster = [Track() for _ in range(19)]
    for track in roster:
        for outcome in [F(0), F(1)] * 15:
            assert not track.record(difference_score(BASELINE, BASELINE, outcome))
        assert track == Track(reviewed=30, scored=30)
    assert [t.raw_weight() for t in roster] == [Decimal(1)] * 19
    return roster


def branch(report, outcome):
    roster = initial_roster()
    frozen = [t.raw_weight() for t in roster]
    assert 3 * sorted(frozen)[9] == 3
    reports = [report] + [BASELINE] * 8
    baselines = [sum((reports[v] for v in range(9) if v != u), F(0)) / 8
                 for u in range(9)]
    scores = [difference_score(p, b, outcome) for p, b in zip(reports, baselines)]
    for track, score in zip(roster, scores):
        assert not track.record(score)
    expected = {
        (Q, 0): (F(-39, 100), F(249, 6400), F(9, 25), F(0)),
        (Q, 1): (F(21, 100), F(-231, 6400), F(0), F(39, 6400)),
        (F(41, 50), 0): (F(-264, 625), F(26, 625), F(981, 2500), F(0)),
        (F(41, 50), 1): (F(136, 625), F(-24, 625), F(0), F(21, 2500)),
    }
    du, dv, cu, cv = expected[(report, outcome)]
    assert baselines[0] == BASELINE and baselines[1:] == [(report + 7 * BASELINE) / 8] * 8
    assert scores == [du] + [dv] * 8
    assert roster[0].cusum == cu and all(t.cusum == cv for t in roster[1:9])
    assert all(t.alarms == 0 for t in roster)
    assert all(t.reviewed == t.scored == 31 for t in roster[:9])
    assert all(t == Track(reviewed=30, scored=30) for t in roster[9:])
    weights = [t.raw_weight() for t in roster]
    median = sorted(weights)[9]
    cap = 3 * median
    assert median == 1 and cap == 3 and max(weights) < cap
    assert weights[0] == decimal(LAMBDA * du).exp()
    print(f"p={report} Y={outcome}: score_u={du}, CUSUM_u={cu}, "
          f"peer_baseline={baselines[1]}, peer_score={dv}, peer_CUSUM={cv}")
    print(f"  next_weight_u={weights[0]}; roster_median={median}; cap={cap}; alarms=0")
    return du, min(weights[0], cap)


def exp8(value):
    return sum((value ** k / factorial(k) for k in range(9)), F(0))


def expected8(report):
    result = F(0)
    for outcome, probability in ((F(0), 1 - Q), (F(1), Q)):
        exponent = LAMBDA * difference_score(report, BASELINE, outcome)
        assert abs(exponent) < F(1, 8)
        result += probability * exp8(exponent)
    return result


def main():
    with localcontext() as context:
        context.prec = 65
        expectations = []
        for report, expected_score in ((Q, F(9, 100)), (F(41, 50), F(56, 625))):
            mean_score, mean_weight = F(0), Decimal(0)
            for outcome, probability in ((F(0), 1 - Q), (F(1), Q)):
                score, weight = branch(report, outcome)
                mean_score += probability * score
                mean_weight += decimal(probability) * weight
            assert mean_score == expected_score
            expectations.append(mean_weight)
            print(f"p={report}: E_score={mean_score}; E_next_weight={mean_weight}; "
                  f"U_two_opportunities={1 + mean_weight}")
        assert F(9, 100) - F(56, 625) == (F(41, 50) - Q) ** 2 == F(1, 2500)
        remainder = F(2, 8 ** 9 * factorial(9))
        center = expected8(F(41, 50)) - expected8(Q)
        lower, upper = center - 2 * remainder, center + 2 * remainder
        assert F(1, 6000) < lower < upper < F(1, 5900)
        gain = expectations[1] - expectations[0]
        assert decimal(lower) < gain < decimal(upper)
        print(f"utility_gain={gain}")
        print(f"rational_gain_interval=[{decimal(lower)}, {decimal(upper)}]")
        print("Exact certificate: 1/6000 < lower <= gain <= upper < 1/5900")
        print("Mathematical transcription only; no Rust/fmath or end-to-end check.")


if __name__ == "__main__":
    main()
