"""Tests of `a2_probe.py` (`docs/19` §13.11) on constructed records, plans and ledgers, not fits.
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


def form(it, members, reference=P.K):
    size = len(members)
    comp = "empty" if not size else "complete" if size == reference else "incomplete"
    return {"members": members, "size": size, "reference": reference, "completeness": comp,
            "removed": [], "dtf": P.dtf(it, members), "grid": P.grid_dtf(it, members), "fitted": 0.0}


def record(case, r, outcome="read", status="converged", counted=2, rechecks="NNNNNNNN", cpu=1000,
           phase="confirm"):
    it = items(case)
    rec = {"schema": P.SCHEMA, "key": f"{phase}-{case}-{r}", "phase": phase, "case": case,
           "replicate": r, "seed": 100 + r,
           "design": {"n": 3000, "anchors": 60, "k": 8, "layout": P.CASES[case][0],
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


def write(d, recs, planned=None, ledger=None, stop=None, phase="confirm", session_stop=None):
    """`records.jsonl`, the phase's plan (the records' runs unless `planned`) and a ledger that
    starts and ends each record's run with its outcome (unless `ledger` rows are given)."""
    d = Path(d)
    (d / "records.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
    plan = planned if planned is not None else [(r["key"], r["seed"]) for r in recs]
    (d / f"plan-{phase}.tsv").write_text(f"schema\tstand-in\nphase\t{phase}\t8\n"
                                         + "".join(f"task\t{k}\t{s}\n" for k, s in plan))
    if ledger is None:
        ledger = [r for rec in recs for r in (("start", rec["key"], 1),
                                                ("end", rec["key"], rec["outcome"], rec["cost"]["cpu_ms"], 1, "self"))]
    rows = [P.LEDGER_HEADER] + ["\t".join(map(str, r)) for r in ledger]
    if session_stop:
        rows.append(f"session\t10\t0\t0\t100\t{session_stop}\tx")
    rows += [f"stop\t{stop}"] if stop else []
    (d / "ledger.tsv").write_text("\n".join(rows) + "\n")


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


class Analysis(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = self.tmp.name

    def tearDown(self):
        self.tmp.cleanup()

    def test_check_fails_with_nothing_to_verify(self):
        self.assertFalse(P.check(self.d)["passed"])
        write(self.d, [])
        self.assertFalse(P.check(self.d)["passed"])
        self.assertEqual(P.report(self.d)[1], 1)

    def test_check_accepts_exact_values_and_flags_a_perturbed_one(self):
        recs = [record("C4", 0), record("C2", 1, outcome="refused")]
        write(self.d, recs)
        self.assertTrue(P.check(self.d)["passed"])
        recs[0]["fixed"]["dtf"] += 2 * P.TOL
        write(self.d, recs)
        out = P.check(self.d)
        self.assertFalse(out["passed"])
        self.assertEqual([f["value"] for f in out["failures"]], ["fixed"])

    def test_a_complete_phase_reports_complete(self):
        write(self.d, [record("C3", 0), record("C3", 1)])
        text, code = P.report(self.d)
        self.assertEqual(code, 0)
        self.assertIn("Status: COMPLETE.", text)

    def test_the_report_counts_every_outcome_and_keeps_its_denominators(self):
        recs = [record("C2", 0, rechecks="DDNNNNNN"), record("C2", 1, status="max_iters", rechecks="IIIIIIII"),
                record("C2", 2, counted=1), record("C2", 3, outcome="refused"),
                record("C2", 4, outcome="killed"), record("C2", 5, rechecks="DDDDDDDD")]
        write(self.d, recs)
        text, code = P.report(self.d)
        self.assertEqual(code, 2)
        self.assertIn("PARTIAL: 6 of 6 planned runs recorded; never started 0, started and never ended 0, "
                      "not observed (killed or failed) 1", text)
        self.assertIn("- runs 6: read 4, refused 1, killed 1, failed 0", text)
        self.assertIn("- read 4: converged 3, not converged 1; one counted class among converged 1", text)
        self.assertIn("- leaners flagged 4 of 6 (converged runs; 12 over all runs)", text)
        self.assertIn("- clean items flagged 6 of 18; runs with one 1 of 3", text)
        self.assertIn("- S0 active: 4 forms of 6 runs (2 without a reading); "
                      "{'empty': 2, 'incomplete': 1, 'complete': 1}", text)

    def test_a_partial_phase_declares_itself_with_its_counts(self):
        recs = [record("C0", 0)]
        planned = [("confirm-C0-0", 100), ("confirm-C1-0", 100), ("confirm-C2-0", 100)]
        rows = [("start", "confirm-C0-0", 1), ("end", "confirm-C0-0", "read", 1000, 1, "self"),
                ("start", "confirm-C1-0", 2)]
        write(self.d, recs, planned=planned, ledger=rows, stop="cpu cap")
        text, code = P.report(self.d)
        self.assertEqual(code, 2)
        self.assertIn("PARTIAL: 1 of 3 planned runs recorded; never started 1, started and never ended 1", text)

    def test_the_audit_finds_duplicates_and_inconsistent_unplanned_or_missing_records(self):
        cases = {"duplicates": [record("C0", 0), record("C0", 0)],
                 "inconsistent": [dict(record("C0", 0), seed=7)],
                 "unplanned": [record("C0", 0)]}
        once = [("start", "confirm-C0-0", 1), ("end", "confirm-C0-0", "read", 1000, 1, "self")]
        for name, recs in cases.items():
            planned = [("confirm-C1-0", 100)] if name == "unplanned" else [("confirm-C0-0", 100)]
            write(self.d, recs, planned=planned, ledger=once)
            a = P.audit(self.d, "confirm")
            self.assertEqual((a["status"], bool(a[name])), ("invalid", True), name)
            self.assertEqual(P.report(self.d)[1], 1, name)
        write(self.d, [], planned=[("confirm-C0-0", 100)],
              ledger=[("start", "confirm-C0-0", 1), ("end", "confirm-C0-0", "read", 1, 1, "self")])
        self.assertEqual(P.audit(self.d, "confirm")["ended_without_record"], ["confirm-C0-0"])

    def test_a_failed_precision_check_stops_the_analysis_with_its_diagnostics(self):
        recs = [record("C3", 0)]
        recs[0]["truth"]["dtf"] += 1.0
        write(self.d, recs)
        text, code = P.report(self.d)
        self.assertEqual(code, 1)
        self.assertIn("Stopped: the precision check failed", text)
        self.assertIn('"value": "truth.dtf"', text)
        self.assertNotIn("## C3", text)

    def test_a_selection_over_dtf_max_is_a_blocking_defect(self):
        rec = record("C2", 0, rechecks="DDNNNNNN")
        rec["readings"][1].update(contested=[0, 1], cost=0.2, draw={"outcome": "selected", "members": [0, 1]},
                                  selection=form(items("C2"), [0, 1], reference=2))
        write(self.d, [rec])
        text, code = P.report(self.d)
        self.assertEqual(code, 1)
        self.assertIn("blocking defect", text)

    def test_a_generating_dtf_above_the_line_is_a_result_not_a_defect(self):
        rec = record("C2", 0)
        self.assertGreater(rec["truth"]["dtf"], 0.5)
        write(self.d, [rec])
        text, code = P.report(self.d)
        self.assertEqual(code, 0)
        self.assertIn("'above': 1", text)

    def test_a_malformed_ledger_stops_check_report_and_budget(self):
        for bad in ("end\tconfirm-C0-0\tread\t5\t5\tself", "start\tconfirm-C0-0\t1\nend\tconfirm-C0-0\tread\tNaN\t5\tself",
                    "bogus"):
            write(self.d, [record("C0", 0)])
            (Path(self.d) / "ledger.tsv").write_text(P.LEDGER_HEADER + "\n" + bad + "\n")
            with self.assertRaises(ValueError):
                P.check(self.d)
            self.assertEqual(P.report(self.d)[1], 1)
            self.assertIn("malformed", P.budget(self.d)["reason"])


class Budget(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = self.tmp.name

    def tearDown(self):
        self.tmp.cleanup()

    def explore(self, cpu=100_000, outcome="read", cases=tuple(P.CASES), **kw):
        recs = [record(c, 0, outcome=outcome, cpu=cpu, phase="explore") for c in cases]
        planned = [(f"explore-{c}-0", 100) for c in P.CASES]
        write(self.d, recs, planned=planned, phase="explore", **kw)
        return recs

    def test_the_replicates_follow_the_cost_rule_alone(self):
        self.explore()
        left, per = P.CAP_CPU_MS - 600_000, 1.5 * 600_000
        self.assertEqual(P.budget(self.d)["replicates"], min(8, math.floor(left / per)))
        self.explore(cpu=10_000)
        self.assertEqual(P.budget(self.d)["replicates"], 8)

    def test_an_incomplete_imprecise_or_defective_explore_proposes_no_replicates(self):
        self.explore(outcome="killed")
        self.assertIsNone(P.budget(self.d)["replicates"])
        self.explore(cases=tuple(P.CASES)[1:])
        self.assertIsNone(P.budget(self.d)["replicates"])
        self.explore(cpu=700_000)
        self.assertIsNone(P.budget(self.d)["replicates"])
        recs = self.explore()
        recs[2]["truth"]["dtf"] += 1.0
        write(self.d, recs, planned=[(r["key"], r["seed"]) for r in recs], phase="explore")
        self.assertIn("precision", P.budget(self.d)["reason"])
        self.explore()
        (Path(self.d) / "rejected").mkdir()
        (Path(self.d) / "rejected" / "explore-C2-0.json").write_text("{}")
        self.assertIn("blocking defect", P.budget(self.d)["reason"])

    def test_a_stop_in_the_session_line_alone_binds_budget_audit_and_report(self):
        self.explore(session_stop="cpu cap")
        self.assertEqual(P.read_ledger(self.d)["stop"], "cpu cap")
        self.assertIsNone(P.budget(self.d)["replicates"])
        self.assertEqual(P.audit(self.d, "explore")["status"], "partial")
        text, code = P.report(self.d, "explore")
        self.assertEqual(code, 2)
        self.assertNotIn("COMPLETE", text)
        self.explore(stop="cpu cap")
        self.assertIsNone(P.budget(self.d)["replicates"])


def candidate(classes, slopes, gain, status="converged", one_converged=True, dtf=0.0):
    return {"classes": classes, "non_uniform": slopes, "status": status, "verdict": "bic_not_lower",
            "log_lik": -1000.0 + gain / 2, "params": 76 + 10 * (classes - 1), "bic": 0.0, "gain": gain,
            "penalty": 80.0 * (classes - 1), "delta_bic": 80.0 * (classes - 1) - gain,
            "one_converged": one_converged, "pi": [1.0 / classes] * classes, "counted": classes,
            "dtf": dtf, "dtf_basis": "counted classes" if classes > 1 else "one counted class"}


class Candidates(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name) / "run"
        self.d.mkdir()
        self.first = Path(self.tmp.name) / "first.jsonl"

    def tearDown(self):
        self.tmp.cleanup()

    def make(self, gains=(10.0, 30.0), one_converged=True):
        firsts, news = [], []
        for case, gain in zip(("C0", "C3"), gains):
            rec = record(case, 0)
            firsts.append(rec)
            new = json.loads(json.dumps(rec))
            new.update(key=f"candidates-{case}-0", phase="candidates", cost={"wall_ms": 7, "cpu_ms": 7, "hwm_kib": 2})
            new["candidates"] = [candidate(1, False, 0.0, one_converged=one_converged),
                                 candidate(2, False, gain, one_converged=one_converged, dtf=0.3),
                                 candidate(2, True, gain + 5, one_converged=one_converged)]
            news.append(new)
        self.first.write_text("".join(json.dumps(r) + "\n" for r in firsts))
        write(self.d, news, phase="candidates")
        return firsts, news

    def test_the_reproduction_check_compares_the_declared_fields_alone(self):
        self.make()
        self.assertTrue(P.compare_first(self.d, self.first)["passed"])
        firsts, news = self.make()
        news[1]["fit"]["status"] = "max_iters"
        write(self.d, news, phase="candidates")
        out = P.compare_first(self.d, self.first)
        self.assertEqual((out["passed"], out["mismatches"]), (False, [{"key": "candidates-C3-0", "fields": ["fit"]}]))
        self.first.write_text(json.dumps(firsts[0]) + "\n")
        self.make()
        self.first.write_text(json.dumps(firsts[0]) + "\n")
        self.assertEqual(P.compare_first(self.d, self.first)["missing"], ["candidates-C3-0"])

    def test_variants_are_paired_with_c0_only_where_comparable(self):
        self.make()
        text, code = P.candidates_report(self.d, self.first)
        self.assertEqual(code, 0)
        self.assertIn("- C3 r0, shared: 20.0", text)
        self.assertIn("- C3 r0, per-class: 20.0", text)
        self.assertIn("gain 30.0, penalty 80.0, delta BIC 50.0", text)
        self.make(one_converged=False)
        text, code = P.candidates_report(self.d, self.first)
        self.assertIn("- C3 r0, shared: not comparable (baseline or candidate not converged)", text)
        self.assertNotIn("- C3 r0, shared: 20.0", text)

    def test_the_candidates_report_stops_when_the_reproduction_fails(self):
        _, news = self.make()
        news[0]["rechecks"] = "DNNNNNNN"
        write(self.d, news, phase="candidates")
        text, code = P.candidates_report(self.d, self.first)
        self.assertEqual(code, 1)
        self.assertIn("Stopped: no candidate is read.", text)


if __name__ == "__main__":
    unittest.main()
