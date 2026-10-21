"""The single-test trade-off reading (`docs/19` §13.24) on constructed values and records."""
import json
import tempfile
import unittest
from pathlib import Path

import a2_probe as P
import a2_tradeoff as T
from test_a2_probe import ind_record, unread

POS, NEG = [0.0, 0.5, 0.5, 0.9], [0.0, 0.0, 0.5, 0.7]


class Curve(unittest.TestCase):
    def test_thresholds_descend_through_every_value_and_flag_strictly_above(self):
        self.assertEqual(T.curve(POS, NEG), [(0.9, 0, 0), (0.7, 1, 0), (0.5, 1, 1), (0.0, 3, 2), (None, 4, 4)])

    def test_ties_move_both_counts_at_once_and_zeros_only_below_every_value(self):
        points = T.curve(POS, NEG)
        self.assertEqual([p[1:] for p in points if p[0] in (0.5, 0.0, None)], [(1, 1), (3, 2), (4, 4)])
        self.assertEqual(T.curve([0.2, 0.2], [0.2]), [(0.2, 0, 0), (None, 2, 1)])

    def test_the_area_is_the_separation_with_ties_counted_half(self):
        self.assertEqual(T.area(T.curve(POS, NEG), 4, 4), 0.625)
        self.assertEqual(T.area(T.curve(POS, NEG), 4, 4), P.separation(POS, NEG))

    def test_the_envelope_takes_the_most_detections_within_a_false_alarm_count(self):
        points = T.curve(POS, NEG)
        self.assertEqual([T.at_most(points, k) for k in (0, 1, 2, 4)], [(1, 0), (1, 0), (3, 2), (4, 4)])


class Records(unittest.TestCase):
    def test_not_evaluable_runs_leave_the_curves_and_are_counted_by_case(self):
        recs = [ind_record("ind-val", c, r, v) for c, vals in (("C0", NEG), ("C1", NEG), ("C3", POS), ("C5", POS))
                for r, v in enumerate(vals)]
        unread(recs[0], "refused")
        recs[9]["candidates"][1]["status"] = "max_iters"
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "records.jsonl").write_text("".join(json.dumps(r) + "\n" for r in recs))
            (Path(d) / "threshold.tsv").write_text(f"threshold\t{P.bits(0.5)}\t0.5\n")
            out = T.tradeoff(Path(d) / "records.jsonl", Path(d) / "threshold.tsv")
        self.assertEqual(out["cases"]["C0"], {"planned": 4, "evaluable": 3, "not_evaluable": {"gate refused": 1},
                                              "zeros": 1})
        self.assertEqual(out["cases"]["C3"]["not_evaluable"], {"candidate not converged": 1})
        c3c0 = out["comparisons"]["C3/C0"]
        self.assertEqual((c3c0["detections_of"], c3c0["false_alarms_of"], c3c0["frozen"]),
                         (3, 3, {"detections": 1, "false_alarms": 1}))
        self.assertEqual(c3c0["area"], P.separation([0.0, 0.5, 0.9], [0.0, 0.5, 0.7]))


if __name__ == "__main__":
    unittest.main()
