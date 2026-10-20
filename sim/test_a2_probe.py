"""Tests of `a2_probe.py` (`docs/19` §13.11) on constructed records, plans and ledgers, not fits.
Run: `python3 -m unittest discover -s sim -p test_a2_probe.py -v`."""
import contextlib
import functools
import io
import json
import math
import tempfile
import unittest
from unittest import mock
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


def write(d, recs, planned=None, ledger=None, stop=None, phase="confirm", session_stop=None, replicates=8,
          config="schema\tstand-in\n"):
    """`records.jsonl`, the phase's plan (the records' runs unless `planned`; `replicates` declared)
    and a ledger starting and ending each record's run with its outcome, unless `ledger` given."""
    d = Path(d)
    (d / "records.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
    plan = planned if planned is not None else [(r["key"], r["seed"]) for r in recs]
    (d / f"plan-{phase}.tsv").write_text(f"{config}phase\t{phase}\t{replicates}\n"
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


class DtfCheck(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def records(self, reps=range(16)):
        """C0 and C3 at `reps`, seeds paired, both two-class candidates converged."""
        recs = []
        for rep in reps:
            for case, dtf in (("C0", 0.2), ("C3", 0.5)):
                rec = record(case, rep, phase="dtf-check")
                rec["candidates"] = [candidate(1, False, 0.0), candidate(2, False, 20.0, dtf=dtf),
                                     candidate(2, True, 30.0, dtf=dtf)]
                recs.append(rec)
        return recs

    def build(self, edit=None):
        recs = self.records()
        if edit:
            edit({(r["case"], r["replicate"]): r for r in recs})
        write(self.d, recs, phase="dtf-check", replicates=16)
        return P.dtf_check(self.d)

    def stops(self, why):
        text, summary, code = P.dtf_check(self.d)
        self.assertEqual((code, summary), (1, None))
        self.assertIn(why, text)
        out = io.StringIO()
        with contextlib.redirect_stdout(out), self.assertRaises(SystemExit) as stop:
            P.main(["dtf-check", str(self.d), "--json"])
        self.assertEqual((stop.exception.code, out.getvalue()), (1, text))
        return text

    def test_a_plan_of_two_runs_stops_the_analysis_whatever_replicates_it_declares(self):
        for replicates in (16, 8):
            write(self.d, self.records(range(1)), phase="dtf-check", replicates=replicates)
            text = self.stops("the plan's tasks are not the 32 declared")
            self.assertEqual("declares 8 replicates" in text, replicates == 8)

    def test_a_substituted_key_or_eight_declared_replicates_stop_a_plan_of_32(self):
        for case, rep in (("C3", 16), ("C5", 15)):
            recs = self.records()
            recs[-1].update(key=f"dtf-check-{case}-{rep}", case=case, replicate=rep)
            write(self.d, recs, phase="dtf-check", replicates=16)
            self.stops(f"missing ['dtf-check-C3-15'], unexpected ['dtf-check-{case}-{rep}']")
        write(self.d, self.records(), phase="dtf-check", replicates=8)
        self.stops("the plan declares 8 replicates, not 16")

    def test_records_not_matching_their_key_stop_the_analysis(self):
        for field, value in (("replicate", 14), ("case", "C0")):
            recs = self.records()
            recs[-1][field] = value
            write(self.d, recs, phase="dtf-check", replicates=16)
            self.stops("records not matching their key's phase, case and replicate: ['dtf-check-C3-15']")

    def test_unpaired_seeds_stop_the_analysis(self):
        recs = self.records()
        recs[15]["seed"] = 999
        write(self.d, recs, phase="dtf-check", replicates=16)
        self.stops("C0's and C3's seeds differ at replicates [7]")

    def test_the_earlier_candidates_are_reread_apart_and_never_pass_as_the_check(self):
        old = Path(__file__).resolve().parents[1] / "verification/reports/a2-candidates-2e2572c"
        ref = json.loads((old / "paired-dtf.json").read_text())
        _, summary, code = P.paired_dtf(old, "candidates", 8)
        self.assertEqual(code, 0)
        for name in ("shared", "per-class"):
            tally = ref[name]["tally"]["C3"]
            self.assertEqual(summary[name]["order"], {k: tally[k] for k in ("greater", "smaller", "equal")})
            orders = [x["order_vs_c0"] for x in ref[name]["pairs"] if x["case"] == "C3"]
            self.assertEqual([row.get("order") for row in summary[name]["rows"]], orders)
        self.assertIn("not the declared phase", P.dtf_check(old)[0])

    def test_orders_on_the_unrounded_value_with_zeros_and_their_origin(self):
        def edit(by):
            by[("C3", 0)]["candidates"][1]["dtf"] = 0.2 + 1e-12
            for case in ("C0", "C3"):
                c = by[(case, 1)]["candidates"][1]
                c.update(dtf=0.0, dtf_basis="one counted class", counted=1)
            by[("C3", 2)]["candidates"][1].update(dtf=0.0, dtf_basis="one counted class", counted=1)
        _, summary, code = self.build(edit)
        shared = summary["shared"]
        self.assertEqual(code, 0)
        self.assertEqual((shared["comparable"], shared["order"]), (16, {"greater": 14, "smaller": 1, "equal": 1}))
        self.assertEqual(shared["rows"][0]["order"], "greater")
        self.assertEqual(shared["rows"][2]["c3"]["dtf_basis"], "one counted class")
        self.assertEqual(shared["without_zeros"], {"greater": 14, "smaller": 0, "equal": 0, "pairs": 14})
        self.assertEqual(summary["per-class"]["order"], {"greater": 16, "smaller": 0, "equal": 0})

    def test_unorderable_pairs_are_listed_with_why_and_leave_the_denominator(self):
        def edit(by):
            by[("C3", 1)]["candidates"][0]["status"] = "max_iters"
            by[("C0", 2)]["candidates"][1]["status"] = "max_iters"
            by[("C3", 3)]["candidates"][1].update(dtf=None, dtf_basis="unavailable")
            del by[("C0", 4)]["candidates"][2]
            by[("C3", 5)].update(outcome="refused")
            for k in ("fit", "rechecks", "fixed", "readings", "candidates"):
                by[("C3", 5)].pop(k)
        text, summary, code = self.build(edit)
        shared, per = summary["shared"], summary["per-class"]
        self.assertEqual((shared["planned"], shared["comparable"]), (16, 12))
        self.assertEqual(shared["not_comparable"], {"C3: baseline not converged": 1, "C0: candidate not converged": 1,
                                                    "C3: DTF unavailable": 1, "C3: run refused": 1})
        self.assertEqual(per["not_comparable"], {"C3: baseline not converged": 1, "C0: candidate missing": 1,
                                                 "C3: run refused": 1})
        self.assertEqual(per["comparable"], 13)
        self.assertIn("not comparable", text)
        self.assertIn("not independent and are not summed", text)

    def test_the_analysis_stops_without_a_complete_audit_or_a_passing_check(self):
        def drop(by):
            by[("C3", 15)]["truth"]["dtf"] += 1.0
        self.assertEqual(self.build(drop)[2], 1)
        recs = P.read_records(self.d)[0]
        write(self.d, recs[:-1], planned=[(r["key"], r["seed"]) for r in recs], phase="dtf-check", replicates=16)
        self.stops("the audit is partial, 31 records of 32 planned")


BASES = {}


def ind_record(phase, case, rep, dtf):
    """A run of `docs/19` §13.21's or §13.29's stages at their `N`, its shared candidate's `DTF_F`
    at `dtf`."""
    if (phase, case) not in BASES:
        BASES[(phase, case)] = record(case, 0, phase=phase)
    rec = json.loads(json.dumps(BASES[(phase, case)]))
    rec.update(key=f"{phase}-{case}-{rep}", replicate=rep, seed=P.case_seed(phase, case, rep))
    rec["design"]["n"] = P.IND_N[phase]
    rec["candidates"] = [candidate(1, False, 0.0), candidate(2, False, 20.0, dtf=dtf),
                         candidate(2, True, 30.0, dtf=dtf)]
    return rec


def unread(rec, outcome):
    rec["outcome"] = outcome
    for k in ("fit", "rechecks", "fixed", "readings", "candidates"):
        rec.pop(k)


def memo(f):
    @functools.wraps(f)
    def run(items, members, *rest):
        key = (tuple(items.a), tuple(items.b), tuple(items.s), items.delta, items.impact, items.pi,
               tuple(members), rest)
        if key not in cache:
            cache[key] = f(items, members, *rest)
        return cache[key]
    cache = {}
    return run


def design_lines(n):
    """The plan's design lines the harness writes, every case at `n` respondents."""
    return "".join(f"design\t{c}\t{n}\t60\t8\t{lay}\t{d!r}\t{i!r}\t0.5\n" for c, (lay, d, i) in P.CASES.items())


class Stages:
    """The indicator's stages at 22 replicates a case, the quadratures memoized."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.cal, self.val = Path(self.tmp.name) / "cal", Path(self.tmp.name) / "val"
        self.cal6, self.val6 = Path(self.tmp.name) / "cal6", Path(self.tmp.name) / "val6"
        for d in (self.cal, self.val, self.cal6, self.val6):
            d.mkdir()
        for patch in (mock.patch.object(P, "IND_REPLICATES", 22), mock.patch.object(P, "dtf", memo(P.dtf)),
                      mock.patch.object(P, "grid_dtf", memo(P.grid_dtf)),
                      mock.patch.object(P, "raw_gap", memo(P.raw_gap))):
            patch.start()
            self.addCleanup(patch.stop)

    def tearDown(self):
        self.tmp.cleanup()

    def stage(self, d, phase, value, edit=None, config="schema\tstand-in\n", n=None):
        """`phase`'s plan, records and ledger in `d`, the configuration's designs at `n` (the
        stage's `N` by default) before `config`."""
        recs = [ind_record(phase, c, r, value(c, r)) for r in range(22) for c in P.IND_CASES[phase]]
        planned = [(x["key"], x["seed"]) for x in recs]
        if edit:
            edit({(x["case"], x["replicate"]): x for x in recs}, recs)
        write(d, [x for x in recs if x], phase=phase, replicates=22, config=design_lines(n or P.IND_N[phase]) + config,
              planned=[(k, x["seed"] if x else s) for (k, s), x in zip(planned, recs)])
        return recs


class SingleTest(Stages, unittest.TestCase):
    """§13.21's stages at 22 replicates a case, the quadratures memoized: same values, less time."""

    def calibrated(self):
        """C0 at r/100, C1 at r/50 with r3 refused by the gate and r4's candidate not converged."""
        def edit(by, _):
            unread(by[("C1", 3)], "refused")
            by[("C1", 4)]["candidates"][1]["status"] = "max_iters"
        self.stage(self.cal, "ind-cal", lambda c, r: r / 100 if c == "C0" else r / 50, edit)
        tsv, why = P.calibrate(self.cal)
        (self.cal / "threshold.tsv").write_text(tsv)
        return tsv, why

    def validated(self, edit=None, config=None):
        """C0 at r/100 with r5 equal to `t` and r6 above; C1 at r/100, r2 refused; C3 at 0.5, r1 not
        converged, r2 equal to `t`; C5 at r/50, r7 at 0.9."""
        def values(c, r):
            return {"C0": {5: 0.42, 6: 0.43}.get(r, r / 100), "C1": r / 100,
                    "C3": 0.42 if r == 2 else 0.5, "C5": 0.9 if r == 7 else r / 50}[c]

        def edits(by, recs):
            unread(by[("C1", 2)], "refused")
            by[("C3", 1)]["candidates"][1]["status"] = "max_iters"
            if edit:
                edit(by, recs)
        line = f"calibration\t{P.sha256(self.cal / 'threshold.tsv')}\t{P.bits(0.42)}\n"
        self.stage(self.val, "ind-val", values, edits, config or line + "schema\tstand-in\n")
        return P.ind_val(self.val, self.cal)

    def test_seeds_are_the_harness_s_new_and_never_shared(self):
        self.assertEqual((P.case_seed("ind-cal", "C0", 0), P.case_seed("ind-val", "C5", 99),
                          P.probe_seed("dtf-check", 0)),
                         (14817136108943293105, 12356750456368884976, 11841848735597837198))
        ours = [P.case_seed(p, c, r) for p in ("ind-cal", "ind-val") for c in P.IND_CASES[p] for r in range(100)]
        earlier = {P.probe_seed(p, r) for p in ("smoke", "explore", "confirm") for r in range(64)}
        earlier |= {P.probe_seed("dtf-check", r) for r in range(16)}
        self.assertEqual(len(set(ours)), 600)
        self.assertFalse(set(ours) & earlier)

    def test_the_order_rule_takes_the_declared_rank_ties_and_zeros_included(self):
        self.assertEqual([P.rank(n) for n in (100, 98, 40, 19, 18, 0)], [96, 95, 39, 19, None, None])
        self.assertEqual(P.case_threshold([r / 100 for r in range(100, 0, -1)]), (96, 0.96))
        self.assertEqual(P.case_threshold([0.0] * 97 + [0.3, 0.4, 0.5]), (96, 0.0))
        self.assertEqual(P.case_threshold([0.1] * 100), (96, 0.1))
        self.assertEqual(P.case_threshold([0.5] * 18), (None, None))

    def test_exact_intervals_match_their_closed_forms_and_reviewed_values(self):
        for n in (50, 100, 200):
            self.assertAlmostEqual(P.clopper_pearson(0, n)[1], 1 - 0.025 ** (1 / n), places=12)
            self.assertAlmostEqual(P.clopper_pearson(n, n)[0], 0.025 ** (1 / n), places=12)
            lo, hi = P.clopper_pearson(7, n)
            self.assertAlmostEqual(lo, 1 - P.clopper_pearson(n - 7, n)[1], places=12)
        for (k, n), (lo, hi) in {(2, 50): (0.005, 0.137), (5, 100): (0.016, 0.113), (10, 200): (0.024, 0.090)}.items():
            self.assertEqual([round(x, 3) for x in P.clopper_pearson(k, n)], [lo, hi])
        self.assertIsNone(P.clopper_pearson(0, 0))
        self.assertEqual(P.separation([1.0, 0.0], [0.0, 0.0]), 0.75)
        self.assertIsNone(P.separation([], [0.0]))

    def test_calibration_sets_the_larger_case_threshold_from_evaluable_runs_and_names_its_data(self):
        tsv, why = self.calibrated()
        self.assertEqual(why, [])
        lines = tsv.splitlines()
        self.assertEqual(lines[:2], ["schema\ta2-ind/v1", "rule\torder-statistic\talpha\t1/20\tstrict"])
        self.assertEqual(lines[2], f"case\tC0\tplanned\t22\tevaluable\t22\tk\t22\tt\t{P.bits(0.21)}\t0.21")
        self.assertEqual(lines[3], f"case\tC1\tplanned\t22\tevaluable\t20\tk\t20\tt\t{P.bits(0.42)}\t0.42")
        self.assertEqual(lines[4:7], ["not-evaluable\tC1\tcandidate not converged\t1",
                                      "not-evaluable\tC1\tgate refused\t1", f"threshold\t{P.bits(0.42)}\t0.42"])
        self.assertEqual(lines[7:], [f"file\t{f}\t{P.sha256(self.cal / f)}" for f in P.CAL_FILES])

    def test_calibration_stops_on_a_technical_failure_or_too_few_evaluable_runs(self):
        def killed(by, _):
            unread(by[("C0", 9)], "killed")
        self.stage(self.cal, "ind-cal", lambda c, r: 0.1, killed)
        self.assertIn("the audit is partial", P.calibrate(self.cal)[1][0])

        def refused(by, _):
            for r in range(4):
                unread(by[("C1", r)], "refused")
        self.stage(self.cal, "ind-cal", lambda c, r: 0.1, refused)
        self.assertEqual(P.calibrate(self.cal), (None, ["C1: 18 evaluable runs give no finite threshold (19 needed)"]))

        def reseeded(by, _):
            by[("C1", 0)]["seed"] = 1
        self.stage(self.cal, "ind-cal", lambda c, r: 0.1, reseeded)
        self.assertIn("seeds not the declared ones: 1 ['ind-cal-C1-0']", P.calibrate(self.cal)[1])
        err, out = io.StringIO(), io.StringIO()
        with contextlib.redirect_stderr(err), contextlib.redirect_stdout(out), self.assertRaises(SystemExit) as stop:
            P.main(["threshold", str(self.cal)])
        self.assertEqual((stop.exception.code, out.getvalue()), (1, ""))
        self.assertIn("Stopped, no threshold", err.getvalue())

    def test_validation_reads_each_case_apart_against_the_frozen_threshold(self):
        self.calibrated()
        text, summary, code = self.validated()
        self.assertEqual(code, 0)
        got = {c: (x["evaluable"], x["flagged"], x["zeros"], x["not_evaluable"]) for c, x in summary["cases"].items()}
        self.assertEqual(got, {"C0": (22, 1, 1, {}), "C1": (21, 0, 1, {"gate refused": 1}),
                               "C3": (21, 20, 0, {"candidate not converged": 1}), "C5": (22, 1, 1, {})})
        self.assertEqual(summary["cases"]["C3"]["flagged_over_evaluable"],
                         {"k": 20, "n": 21, "ci95": P.clopper_pearson(20, 21)})
        self.assertAlmostEqual(summary["separation"]["C3>C0"], 460.5 / 462)
        self.assertIn("- false alarms: 1 of 22 evaluable, conditional", text)
        self.assertIn("- detections: 20 of 21 evaluable, conditional", text)

    def test_validation_stops_when_incomplete_or_not_bound_to_the_frozen_threshold(self):
        self.calibrated()

        def killed(by, _):
            unread(by[("C5", 21)], "killed")
        text, summary, code = self.validated(killed)
        self.assertEqual((code, summary), (1, None))
        self.assertIn("the audit is partial", text)
        out = io.StringIO()
        with contextlib.redirect_stdout(out), self.assertRaises(SystemExit) as stop:
            P.main(["ind-val", str(self.val), str(self.cal), "--json"])
        self.assertEqual((stop.exception.code, out.getvalue()), (1, text))

        def missing(by, recs):
            recs[recs.index(by[("C3", 4)])] = None
        self.assertIn("the audit is partial, 87 records of 88 planned", self.validated(missing)[0])
        self.assertIn("not bound to this threshold", self.validated(config="schema\tstand-in\n")[0])
        line = f"calibration\t{P.sha256(self.cal / 'threshold.tsv')}\t{P.bits(0.42)}\n"
        self.assertIn("different configurations", self.validated(config=line + "schema\tother\n")[0])
        frozen = (self.cal / "threshold.tsv").read_text()
        (self.cal / "threshold.tsv").write_text(frozen.replace(f"threshold\t{P.bits(0.42)}\t0.42",
                                                                f"threshold\t{P.bits(0.5)}\t0.5"))
        self.assertIn("not the threshold the calibration gives", self.validated()[0])


def run_main(argv):
    """`P.main(argv)`'s exit code, standard output and standard error."""
    out, err = io.StringIO(), io.StringIO()
    try:
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            P.main(argv)
    except SystemExit as e:
        return e.code, out.getvalue(), err.getvalue()
    raise AssertionError("main returned without exiting")


class AtSixThousand(Stages, unittest.TestCase):
    """§13.29's stages at `N = 6,000`, 22 replicates a case: identity and size, binding to their own
    calibration, the order rule's edges, and stops."""

    def calibrated(self, d, phase="ind-cal6", edit=None):
        """C0 at r/100, zeros at r0–r3; C1 at r/50, r3 refused by the gate, r4's baseline not
        converged: `t` = 0.42, C1's."""
        def edits(by, recs):
            unread(by[("C1", 3)], "refused")
            by[("C1", 4)]["candidates"][0]["status"] = "max_iters"
            if edit:
                edit(by, recs)
        self.stage(d, phase, lambda c, r: (0.0 if r < 4 else r / 100) if c == "C0" else r / 50, edits)
        tsv, why = P.calibrate(d, phase)
        if tsv:
            (d / "threshold.tsv").write_text(tsv)
        return tsv, why

    def validated(self, d, phase, cal, edit=None, config=None):
        """C0 at r/100, zeros r0–r2, r5 at `t`, r6 above; C1 at r/100, r2 refused, r8 without DTF;
        C3 at 0.5, r1 and r9 not converged, r2 at `t`, r10 zero; C5 at r/50, r21 at `t`, r7 0.9."""
        def values(c, r):
            return {"C0": {5: 0.42, 6: 0.43}.get(r, 0.0 if r < 3 else r / 100), "C1": r / 100,
                    "C3": {2: 0.42, 10: 0.0}.get(r, 0.5), "C5": 0.9 if r == 7 else r / 50}[c]

        def edits(by, recs):
            unread(by[("C1", 2)], "refused")
            by[("C1", 8)]["candidates"][1].update(dtf=None, dtf_basis="unavailable")
            by[("C3", 1)]["candidates"][1]["status"] = "max_iters"
            by[("C3", 9)]["candidates"][0]["status"] = "max_iters"
            if edit:
                edit(by, recs)
        line = f"calibration\t{P.sha256(cal / 'threshold.tsv')}\t{P.bits(0.42)}\n"
        self.stage(d, phase, values, edits, config or line + "schema\tstand-in\n")
        return P.ind_val(d, cal, phase)

    def test_seeds_at_6000_are_the_harness_s_and_apart_from_every_earlier_run(self):
        self.assertEqual((P.case_seed("ind-cal6", "C0", 0), P.case_seed("ind-val6", "C5", 99)),
                         (4800293454221026440, 4099950399348182921))
        ours = [P.case_seed(p, c, r) for p in ("ind-cal6", "ind-val6") for c in P.IND_CASES[p] for r in range(100)]
        earlier = {P.probe_seed(p, r) for p in ("smoke", "explore", "confirm") for r in range(64)}
        earlier |= {P.probe_seed("dtf-check", r) for r in range(16)}
        earlier |= {P.case_seed(p, c, r) for p in ("ind-cal", "ind-val") for c in P.IND_CASES[p] for r in range(100)}
        earlier |= {P.case_seed("size", c, r) for c in ("C0", "C3") for r in range(16)}
        self.assertEqual(len(set(ours)), 600)
        self.assertFalse(set(ours) & earlier)

    def test_calibration_at_6000_names_its_stage_size_and_data(self):
        tsv, why = self.calibrated(self.cal6)
        self.assertEqual(why, [])
        lines = tsv.splitlines()
        self.assertEqual(lines[:3], ["schema\ta2-ind/v2", "stage\tind-cal6\tn\t6000",
                                     "rule\torder-statistic\talpha\t1/20\tstrict"])
        self.assertEqual(lines[3:8], [f"case\tC0\tplanned\t22\tevaluable\t22\tk\t22\tt\t{P.bits(0.21)}\t0.21",
                                      f"case\tC1\tplanned\t22\tevaluable\t20\tk\t20\tt\t{P.bits(0.42)}\t0.42",
                                      "not-evaluable\tC1\tbaseline not converged\t1",
                                      "not-evaluable\tC1\tgate refused\t1", f"threshold\t{P.bits(0.42)}\t0.42"])
        self.assertEqual(lines[8:], [f"file\t{f}\t{P.sha256(self.cal6 / f)}"
                                     for f in ("records.jsonl", "plan-ind-cal6.tsv", "ledger.tsv")])
        self.assertEqual(P.declared(self.cal6 / "threshold.tsv"), ("ind-cal6", 6000))
        self.assertEqual(run_main(["threshold", str(self.cal6), "--phase", "ind-cal6"]), (0, tsv, ""))
        code, out, err = run_main(["threshold", str(self.cal6)])
        self.assertEqual((code, out), (1, ""))
        self.assertIn("the directory holds the plans of ['ind-cal6'], not of ind-cal alone", err)
        self.assertIn("no stage ind-val6", run_main(["threshold", str(self.cal6), "--phase", "ind-val6"])[2])

    def test_validation_at_6000_reads_each_case_against_its_own_frozen_threshold(self):
        self.calibrated(self.cal6)
        text, summary, code = self.validated(self.val6, "ind-val6", self.cal6)
        self.assertEqual(code, 0)
        self.assertTrue(text.startswith("# A2 single-test study, validation at N = 6,000 (§13.29)\n"))
        self.assertEqual(summary["threshold"], {"bits": P.bits(0.42), "value": 0.42})
        got = {c: (x["evaluable"], x["flagged"], x["zeros"], x["not_evaluable"]) for c, x in summary["cases"].items()}
        self.assertEqual(got, {"C0": (22, 1, 3, {}), "C1": (20, 0, 1, {"DTF unavailable": 1, "gate refused": 1}),
                               "C3": (20, 18, 1, {"baseline not converged": 1, "candidate not converged": 1}),
                               "C5": (22, 1, 1, {})})
        self.assertEqual(summary["cases"]["C3"]["flagged_over_planned"], {"k": 18, "n": 22, "ci95": P.clopper_pearson(18, 22)})
        self.assertEqual(summary["separation"]["C3>C0"], 418 / 440)
        self.assertIn("- false alarms: 1 of 22 evaluable, conditional", text)
        code, out, _ = run_main(["ind-val", str(self.val6), str(self.cal6), "--phase", "ind-val6", "--json"])
        self.assertEqual((code, json.loads(out)), (0, json.loads(json.dumps(summary))))

    def test_a_calibration_of_the_other_size_is_refused_both_ways(self):
        self.calibrated(self.cal, "ind-cal")
        self.calibrated(self.cal6)
        self.assertIn("the calibration is not ind-cal6's at N = 6000: its plans are ['ind-cal'], its threshold.tsv "
                      "names ('ind-cal', 3000)", self.validated(self.val6, "ind-val6", self.cal)[0])
        self.assertIn("the calibration is not ind-cal's at N = 3000: its plans are ['ind-cal6'], its threshold.tsv "
                      "names ('ind-cal6', 6000)", self.validated(self.val, "ind-val", self.cal6)[0])
        self.assertIn("the directory holds the plans of ['ind-val6'], not of ind-val alone", P.ind_val(self.val6, self.cal6)[0])
        frozen = (self.cal6 / "threshold.tsv").read_text()
        (self.cal6 / "threshold.tsv").write_text(frozen.replace("schema\ta2-ind/v2\nstage\tind-cal6\tn\t6000\n",
                                                                "schema\ta2-ind/v1\n"))
        self.assertIn("its threshold.tsv names ('ind-cal', 3000)", self.validated(self.val6, "ind-val6", self.cal6)[0])
        code, out, _ = run_main(["ind-val", str(self.val6), str(self.cal6), "--phase", "ind-val6", "--json"])
        self.assertEqual(code, 1)
        self.assertIn("Stopped, not read: the calibration is not ind-cal6's", out)

    def test_a_validation_at_6000_reads_nothing_unless_bound_to_its_unaltered_calibration(self):
        self.calibrated(self.cal6)
        self.assertEqual(self.validated(self.val6, "ind-val6", self.cal6)[2], 0)
        recs = self.cal6 / "records.jsonl"
        kept = recs.read_text()
        recs.write_text("".join(reversed(kept.splitlines(keepends=True))))
        self.assertIn("not the threshold the calibration gives", self.validated(self.val6, "ind-val6", self.cal6)[0])
        recs.write_text(kept)
        frozen = (self.cal6 / "threshold.tsv").read_text()
        (self.cal6 / "threshold.tsv").write_text(frozen.replace(f"threshold\t{P.bits(0.42)}\t0.42",
                                                                f"threshold\t{P.bits(0.5)}\t0.5"))
        self.assertIn("not the threshold the calibration gives", self.validated(self.val6, "ind-val6", self.cal6)[0])
        (self.cal6 / "threshold.tsv").write_text(frozen)
        self.assertIn("not bound to this threshold", self.validated(self.val6, "ind-val6", self.cal6,
                                                                    config="schema\tstand-in\n")[0])
        other = f"calibration\t{'0' * 64}\t{P.bits(0.42)}\n"
        self.assertIn("not bound to this threshold", self.validated(self.val6, "ind-val6", self.cal6,
                                                                    config=other + "schema\tstand-in\n")[0])
        line = f"calibration\t{P.sha256(self.cal6 / 'threshold.tsv')}\t{P.bits(0.42)}\n"
        self.assertIn("different configurations", self.validated(self.val6, "ind-val6", self.cal6,
                                                                 config=line + "schema\tother\n")[0])
        self.assertEqual(self.validated(self.val6, "ind-val6", self.cal6)[2], 0)

    def test_a_stage_at_6000_with_another_size_phase_task_or_seed_stops(self):
        def stops(why, edit=None, n=None):
            self.stage(self.cal6, "ind-cal6", lambda c, r: 0.1, edit, n=n)
            got = P.calibrate(self.cal6, "ind-cal6")
            self.assertIsNone(got[0])
            self.assertTrue(any(why in x for x in got[1]), (why, got[1]))

        def at3000(by, _):
            by[("C0", 3)]["design"]["n"] = 3000
        stops("records not matching their key's phase, case, replicate and N = 6000: 1 ['ind-cal6-C0-3']", at3000)
        stops("the plan's configuration is not every case's design at N = 6000: [('C0', '3000'), ", n=3000)

        def phased(by, _):
            by[("C1", 5)]["phase"] = "ind-cal"
        stops("records not matching their key's phase, case, replicate and N = 6000: 1 ['ind-cal6-C1-5']", phased)

        def reseeded(by, _):
            by[("C1", 0)]["seed"] = 1
        stops("seeds not the declared ones: 1 ['ind-cal6-C1-0']", reseeded)
        plan = self.cal6 / "plan-ind-cal6.tsv"
        for old, new, why in (("phase\tind-cal6\t22\n", "phase\tind-cal6\t21\n", "the plan declares 21 replicates, not 22"),
                              ("task\tind-cal6-C1-21\t", "task\tind-cal6-C3-0\t", "the plan's tasks are not the 44 "
                               "declared: missing 1 ['ind-cal6-C1-21'], unexpected 1 ['ind-cal6-C3-0']")):
            self.stage(self.cal6, "ind-cal6", lambda c, r: 0.1)
            plan.write_text(plan.read_text().replace(old, new))
            self.assertIn(why, P.calibrate(self.cal6, "ind-cal6")[1])
        self.stage(self.cal6, "ind-cal", lambda c, r: 0.1)
        self.assertIn("the directory holds the plans of ['ind-cal', 'ind-cal6'], not of ind-cal6 alone",
                      P.calibrate(self.cal6, "ind-cal6")[1])

    def test_fewer_than_19_evaluable_runs_set_nothing_and_zeros_and_ties_at_t_are_not_flagged(self):
        def refused(by, _):
            for r in range(4):
                unread(by[("C0", r)], "refused")
        self.stage(self.cal6, "ind-cal6", lambda c, r: 0.1, refused)
        self.assertEqual(P.calibrate(self.cal6, "ind-cal6"),
                         (None, ["C0: 18 evaluable runs give no finite threshold (19 needed)"]))
        self.assertEqual(run_main(["threshold", str(self.cal6), "--phase", "ind-cal6"])[:2], (1, ""))
        self.stage(self.cal6, "ind-cal6", lambda c, r: 0.0)
        tsv, _ = P.calibrate(self.cal6, "ind-cal6")
        (self.cal6 / "threshold.tsv").write_text(tsv)
        self.assertIn(f"threshold\t{P.bits(0.0)}\t0.0\n", tsv)
        line = f"calibration\t{P.sha256(self.cal6 / 'threshold.tsv')}\t{P.bits(0.0)}\n"
        self.stage(self.val6, "ind-val6", lambda c, r: 0.5 if c == "C3" else r / 50 if c == "C5" else 0.0,
                   config=line + "schema\tstand-in\n")
        _, summary, code = P.ind_val(self.val6, self.cal6, "ind-val6")
        self.assertEqual(code, 0)
        self.assertEqual({c: (x["flagged"], x["zeros"]) for c, x in summary["cases"].items()},
                         {"C0": (0, 22), "C1": (0, 22), "C3": (22, 0), "C5": (21, 1)})

    def test_a_technical_failure_leaves_a_stage_at_6000_unread(self):
        def killed(by, _):
            unread(by[("C0", 9)], "killed")
        self.assertIn("the audit is partial, 44 records of 44 planned", self.calibrated(self.cal6, edit=killed)[1][0])
        self.assertFalse((self.cal6 / "threshold.tsv").exists())
        self.calibrated(self.cal6)

        def failed(by, _):
            unread(by[("C5", 21)], "failed")
        text, summary, code = self.validated(self.val6, "ind-val6", self.cal6, failed)
        self.assertEqual((code, summary), (1, None))
        self.assertIn("the audit is partial, 88 records of 88 planned", text)

        def missing(by, recs):
            recs[recs.index(by[("C3", 4)])] = None
        self.assertIn("the audit is partial, 87 records of 88 planned", self.validated(self.val6, "ind-val6", self.cal6, missing)[0])
        self.assertIn("no stage ind-val7", run_main(["ind-val", str(self.val6), str(self.cal6), "--phase", "ind-val7"])[1])
        self.assertEqual(run_main(["threshold", str(self.cal6), "--phase"])[:2], (1, ""))


if __name__ == "__main__":
    unittest.main()
