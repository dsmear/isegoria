"""Tests of `a2_probe.py` (`docs/19` §13.11) on constructed records and ledgers, not on fits.
Run: `python3 -m unittest discover -s sim -p test_a2_probe.py -v`."""
import json
import math
import tempfile
import unittest
from pathlib import Path
from statistics import NormalDist

import a2_probe as P

REF_B = [0.6 * NormalDist().inv_cdf((j + 0.5) / P.K) for j in range(P.K)]


def items(case, a=None, b=None):
    layout, delta, impact = P.CASES[case]
    return P.Items(a or [1.25] * P.K, b or REF_B, P.signs(layout), delta, impact)


def form(it, members, reference=P.K, completeness=None):
    size = len(members)
    comp = completeness or ("empty" if not size else "complete" if size == reference else "incomplete")
    return {"members": members, "size": size, "reference": reference, "completeness": comp,
            "removed": [], "dtf": P.dtf(it, members), "grid": P.grid_dtf(it, members), "fitted": 0.0}


def record(case, r, outcome="read", status="converged", counted=2, rechecks="NNNNNNNN", cpu=1000):
    it = items(case)
    rec = {"schema": "a2-probe/v1", "key": f"confirm-{case}-{r}", "phase": "confirm", "case": case,
           "replicate": r, "design": {"n": 3000, "anchors": 60, "k": 8, "layout": P.CASES[case][0],
                                      "delta": it.delta, "impact": it.impact, "pi": 0.5},
           "items": {"a": it.a, "b": it.b, "signs": it.s,
                     "roles": "".join("+" if s > 0 else "-" if s < 0 else "c" for s in it.s)},
           "truth": {"dtf": P.dtf(it, range(8)), "grid": P.grid_dtf(it, range(8)),
                     "raw_gap": P.raw_gap(it, range(8))},
           "outcome": outcome, "cost": {"wall_ms": cpu, "cpu_ms": cpu, "hwm_kib": 1}}
    if outcome != "read":
        return rec
    active = [j for j in range(8) if rechecks[j] == "N"]
    reading = lambda src: {"source": src, "routes": "", "contested": [], "draw": {"outcome": "not_needed"},
                           "cost": None, "active": form(it, active), "composed": form(it, active),
                           "selection": None}
    rec.update(fit={"status": status, "classes": counted, "counted": counted}, rechecks=rechecks,
               fixed=form(it, list(range(8))), readings=[reading("S0"), reading("S1")])
    return rec


def write(dirpath, recs, ledger_rows=()):
    (dirpath / "records.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
    (dirpath / "ledger.tsv").write_text("".join("\t".join(map(str, row)) + "\n" for row in ledger_rows))


class Oracle(unittest.TestCase):
    def test_no_dif_gives_an_exact_zero_and_impact_a_raw_gap(self):
        for case in ("C0", "C1"):
            self.assertEqual(P.dtf(items(case), range(8)), 0.0)
        self.assertAlmostEqual(P.raw_gap(items("C1"), range(8)), 1.7072066242, places=9)

    def test_methods_agree_on_smooth_and_kinked_gaps(self):
        for case in ("C2", "C3", "C5"):
            it = items(case)
            self.assertLess(abs(P.simpson_dtf(it, range(8)) - P.dtf(it, range(8))), 1e-12)
        it = items("C4")
        self.assertLess(abs(P.simpson_dtf(it, range(8)) - P.dtf(it, range(8))), P.TOL)
        self.assertLess(abs(P.dtf(it, range(8), P.PANEL / 2) - P.dtf(it, range(8))), 1e-12)

    def test_the_grid_is_an_approximation_not_the_truth(self):
        it = items("C4")
        self.assertGreater(abs(P.grid_dtf(it, range(8)) - P.dtf(it, range(8))), 1e-5)

    def test_values_within_the_band_are_near_the_line(self):
        self.assertEqual([P.side(v) for v in (0.0985, 0.0995, 0.1005, 0.1015)],
                         ["below", "near", "near", "above"])

    def test_the_kl_bound_vanishes_without_dif_and_bounds_tv(self):
        self.assertAlmostEqual(P.kl_bound(items("C0")), 0.0, places=15)
        kl = P.kl_bound(items("C5"))
        self.assertTrue(0 < kl and P.tv_bound(kl, 3000) < 0.1)


class Records(unittest.TestCase):
    def test_check_accepts_exact_values_and_flags_a_perturbed_one(self):
        with tempfile.TemporaryDirectory() as d:
            recs = [record("C4", 0), record("C2", 0, outcome="refused")]
            write(Path(d), recs)
            self.assertEqual(P.check(Path(d) / "records.jsonl")["failures"], [])
            recs[0]["fixed"]["dtf"] += 2 * P.TOL
            write(Path(d), recs)
            fails = P.check(Path(d) / "records.jsonl")["failures"]
            self.assertEqual([f["value"] for f in fails], ["fixed"])

    def test_the_report_counts_every_outcome_and_keeps_its_denominators(self):
        recs = [record("C2", 0, rechecks="DDNNNNNN"), record("C2", 1, status="max_iters", rechecks="IIIIIIII"),
                record("C2", 2, counted=1), record("C2", 3, outcome="refused"),
                record("C2", 4, outcome="killed"), record("C2", 5, rechecks="DDDDDDDD")]
        with tempfile.TemporaryDirectory() as d:
            write(Path(d), recs)
            text = P.report(Path(d) / "records.jsonl")
        self.assertIn("- runs 6: read 4, refused 1, killed 1, failed 0", text)
        self.assertIn("- read 4: converged 3, not converged 1; one counted class among converged 1", text)
        self.assertIn("- leaners flagged 4 of 6 (converged runs; 12 over all runs)", text)
        self.assertIn("- clean items flagged 6 of 18; runs with one 1 of 3", text)
        self.assertIn("- S0 active: 4 forms of 6 runs (2 without a reading); "
                      "{'empty': 2, 'incomplete': 1, 'complete': 1}", text)

    def test_the_report_does_not_classify_after_a_failed_check(self):
        recs = [record("C3", 0)]
        recs[0]["truth"]["dtf"] += 1.0
        with tempfile.TemporaryDirectory() as d:
            write(Path(d), recs)
            self.assertIn("not classified: precision check failed", P.report(Path(d) / "records.jsonl"))


class Budget(unittest.TestCase):
    def ledger(self, rows):
        with tempfile.TemporaryDirectory() as d:
            write(Path(d), [], rows)
            return P.budget(Path(d) / "ledger.tsv")

    def ends(self, cpu):
        return [("end", f"explore-{c}-0", "read", cpu, cpu) for c in P.CASES]

    def test_the_replicates_follow_the_cost_rule_alone(self):
        out = self.ledger(self.ends(100_000) + [("session", 120_000, 2_000, 1, "-")])
        left, per = P.CAP_CPU_MS - 602_000, 1.5 * 600_000
        self.assertEqual(out["replicates"], min(8, math.floor(left / per)))
        self.assertEqual(self.ledger(self.ends(10_000))["replicates"], 8)

    def test_a_missing_or_killed_explore_run_a_stop_or_too_few_replicates_refuse(self):
        rows = self.ends(1000)
        rows[2] = ("end", "explore-C2-0", "killed", 1000, 1000)
        self.assertIsNone(self.ledger(rows)["replicates"])
        self.assertIsNone(self.ledger(self.ends(1000)[1:])["replicates"])
        self.assertIsNone(self.ledger(self.ends(1000) + [("stop", "cpu cap")])["replicates"])
        self.assertIsNone(self.ledger(self.ends(700_000))["replicates"])


if __name__ == "__main__":
    unittest.main()
