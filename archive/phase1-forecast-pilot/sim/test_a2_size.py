"""The sample-size reading (`docs/19` §13.26) on constructed records, quadratures memoized."""
import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import a2_probe as P
import a2_size as S
from test_a2_probe import candidate, memo, record, unread, write

BASES = {}


def size_record(n, case, r, dtf):
    if case not in BASES:
        BASES[case] = record(case, 0, phase="size")
    rec = json.loads(json.dumps(BASES[case]))
    rec.update(key=S.key(n, case, r), replicate=r, seed=P.case_seed("size", case, r))
    rec["design"]["n"] = n
    rec["candidates"] = [candidate(1, False, 0.0), candidate(2, False, 20.0, dtf=dtf), candidate(2, True, 30.0, dtf=dtf)]
    if dtf == 0:
        rec["candidates"][1].update(dtf_basis="one counted class", counted=1)
    return rec


def value(n, case, r):
    """C0 0 at r0–r3, 0.2 else, 0 at r6 at 6,000; C3 0.2, 0 at r2, at 3,000, 0.5 at 6,000."""
    if case == "C0":
        return 0.0 if r < 4 or (n == 6000 and r == 6) else 0.2
    return 0.5 if n == 6000 else (0.0 if r == 2 else 0.2)


class Size(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.d = Path(self.tmp.name)
        for patch in (mock.patch.object(P, "dtf", memo(P.dtf)), mock.patch.object(P, "grid_dtf", memo(P.grid_dtf)),
                      mock.patch.object(P, "raw_gap", memo(P.raw_gap))):
            patch.start()
            self.addCleanup(patch.stop)

    def tearDown(self):
        self.tmp.cleanup()

    def build(self, edit=None):
        """C3 r1's baseline not converged at 3,000 alone, C0 r5's candidate at 6,000 alone."""
        recs = [size_record(n, c, r, value(n, c, r)) for r in range(16) for n in S.SIZES for c in S.CASES]
        by = {(x["design"]["n"], x["case"], x["replicate"]): x for x in recs}
        by[(3000, "C3", 1)]["candidates"][0]["status"] = "max_iters"
        by[(6000, "C0", 5)]["candidates"][1]["status"] = "max_iters"
        planned = [(x["key"], x["seed"]) for x in recs]
        if edit:
            edit(by, recs)
        planned = [(k, x["seed"] if x else s) for (k, s), x in zip(planned, recs)]
        write(self.d, [x for x in recs if x], phase="size", replicates=16, planned=planned)
        return S.size_reading(self.d)

    def test_separations_count_ties_zeros_and_exclusions_that_differ_between_sizes(self):
        text, out, code = self.build()
        self.assertEqual(code, 0)
        self.assertEqual({k: (v["evaluable"], v["zeros"], v["not_evaluable"]) for k, v in out["sizes"].items()},
                         {"3000/C0": (16, 4, {}), "3000/C3": (15, 1, {"baseline not converged": 1}),
                          "6000/C0": (15, 5, {"candidate not converged": 1}), "6000/C3": (16, 0, {})})
        self.assertEqual(out["sizes"]["3000/C3"]["zero_bases"], ["one counted class"])
        self.assertEqual(out["all"][3000], {"separation": 142 / 240, "pairs": 240, "tied_pairs": 172, "C3": 15, "C0": 16,
                                            "why": None})
        self.assertEqual(out["all"][6000]["separation"], 1.0)
        self.assertEqual(out["all"]["difference"], 1.0 - 142 / 240)
        self.assertIn("- N 3000: 0.5916666666666667 over 15 × 16 pairs, 172 tied", text)

    def test_the_common_replicates_give_a_conditional_comparison_and_paired_moves(self):
        _, out, _ = self.build()
        self.assertEqual((out["common"][3000]["separation"], out["common"][6000]["separation"]), (135 / 225, 1.0))
        self.assertEqual(out["common"][3000]["pairs"], 225)
        c0, c3 = out["replicates"]["C0"], out["replicates"]["C3"]
        self.assertEqual((len(c0["common"]), len(c3["common"])), (15, 15))
        self.assertEqual(c0["I_6000_against_3000"], {"greater": 0, "smaller": 1, "equal": 14})
        self.assertEqual(c0["zeros"], {"zero→zero": 4, "positive→zero": 1, "positive→positive": 10})
        self.assertEqual(c3["zeros"], {"positive→positive": 14, "zero→positive": 1})
        self.assertEqual((c0["evaluable_at_one_size"], c3["evaluable_at_one_size"]), ({5: [3000]}, {1: [6000]}))
        common = out["placements"]["the common C0"]
        self.assertEqual(common["moves"], {"rise": 15, "fall": 0, "same": 0, "not available": 0})
        self.assertEqual(common["rows"][0], {"replicate": 0, "3000": 9.5 / 15, "6000": 1.0, "move": "rise", "why": None})
        self.assertEqual(out["placements"]["every evaluable C0"]["rows"][1],
                         {"replicate": 2, "3000": 2 / 16, "6000": 1.0, "move": "rise", "why": None})

    def unconverged(self, cells):
        """The reading with each `(n, case, r)` of `cells` left by a baseline not converged."""
        def edit(by, _):
            for cell in cells:
                by[cell]["candidates"][0]["status"] = "max_iters"
        text, out, code = self.build(edit)
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout), self.assertRaises(SystemExit) as stop:
            S.main([str(self.d), "--json"])
        self.assertEqual((code, stop.exception.code), (0, 0))
        self.assertEqual(json.loads(stdout.getvalue()), json.loads(json.dumps(out)))
        return text, out

    def test_no_c0_evaluable_at_both_sizes_leaves_its_moves_and_placements_unavailable(self):
        text, out = self.unconverged([(3000, "C0", r) for r in range(8)] + [(6000, "C0", r) for r in range(8, 16)])
        self.assertEqual((out["all"][3000]["C0"], out["all"][6000]["C0"]), (8, 7))
        self.assertIsNotNone(out["all"]["difference"])
        self.assertEqual(out["common"][3000], {"separation": None, "pairs": 0, "tied_pairs": 0, "C3": 15, "C0": 0,
                                               "why": "no C0 run evaluable at both sizes"})
        self.assertIsNone(out["common"]["difference"])
        self.assertEqual(out["replicates"]["C0"]["I_6000_against_3000"], None)
        self.assertEqual(out["replicates"]["C0"]["why"], "no replicate evaluable at both sizes")
        common = out["placements"]["the common C0"]
        self.assertEqual(common["moves"], {"rise": 0, "fall": 0, "same": 0, "not available": 15})
        self.assertEqual({r["why"] for r in common["rows"]}, {"no C0 reference at 3000 and 6000"})
        self.assertEqual(out["placements"]["every evaluable C0"]["moves"]["not available"], 0)
        self.assertIn("- C0: no replicate evaluable at both sizes", text)
        self.assertIn("not available 15 (no C0 reference at 3000 and 6000)", text)

    def test_c0_never_evaluable_at_one_size_leaves_that_size_s_separation_unavailable(self):
        text, out = self.unconverged([(6000, "C0", r) for r in range(16)])
        self.assertEqual(out["all"][6000], {"separation": None, "pairs": 0, "tied_pairs": 0, "C3": 16, "C0": 0,
                                            "why": "no evaluable C0 run"})
        self.assertEqual(out["all"][3000]["separation"], 142 / 240)
        self.assertIsNone(out["all"]["difference"])
        every = out["placements"]["every evaluable C0"]
        self.assertEqual((every["references"], every["moves"]["not available"]), ({3000: 16, 6000: 0}, 15))
        self.assertIn("- N 6000: not available, no evaluable C0 run (16 C3, 0 C0)", text)

    def test_no_c3_evaluable_at_both_sizes_leaves_no_placement_to_read(self):
        text, out = self.unconverged([(6000, "C3", r) for r in range(16)])
        self.assertEqual(out["all"][6000]["why"], "no evaluable C3 run")
        self.assertEqual(out["replicates"]["C3"]["why"], "no replicate evaluable at both sizes")
        for name in ("every evaluable C0", "the common C0"):
            placed = out["placements"][name]
            self.assertEqual((placed["C3"], placed["rows"], placed["why"]), (0, [], "no C3 replicate evaluable at both sizes"))
        self.assertIn("- C3 placements, reference every evaluable C0 (same size): not available, no C3 replicate "
                      "evaluable at both sizes", text)

    def test_the_reading_stops_without_the_declared_nested_complete_phase(self):
        def killed(by, _):
            unread(by[(6000, "C3", 15)], "killed")
        text, out, code = self.build(killed)
        self.assertEqual((code, out), (1, None))
        self.assertIn("the audit is partial", text)
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout), self.assertRaises(SystemExit) as stop:
            S.main([str(self.d), "--json"])
        self.assertEqual((stop.exception.code, stdout.getvalue()), (1, text))

        def missing(by, recs):
            recs[recs.index(by[(3000, "C0", 7)])] = None
        self.assertIn("the audit is partial, 63 records of 64 planned", self.build(missing)[0])

        def resized(by, _):
            by[(6000, "C0", 3)]["design"]["n"] = 3000
        self.assertIn("records not matching their key's phase, size, case and replicate: 1 ['size-n6000-C0-3']",
                      self.build(resized)[0])

        def unlike(by, _):
            by[(6000, "C3", 9)]["items"]["a"][0] += 0.1
        self.assertIn("runs not alike across the sizes (seed, items, generating DTF): 1 ['C3-9']", self.build(unlike)[0])

        def reseeded(by, _):
            by[(3000, "C3", 4)]["seed"] = 1
        self.assertIn("seeds not the declared ones", self.build(reseeded)[0])


if __name__ == "__main__":
    unittest.main()
