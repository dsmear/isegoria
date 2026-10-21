"""Frozen A12 transport and numerical contract; docs/16 §7.2."""

import copy
import contextlib
import io
import json
import subprocess
import sys
import tempfile
from unittest.mock import patch
import hashlib
import struct
import unittest
from fractions import Fraction
from pathlib import Path

from a12_offline import evaluate, initial_history_hash
from export_a12_affine import export_epochs, main
from test_a12_offline import experiment, set_outcome, set_report

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixtures" / "a12_affine_nyms.json"
GOLDEN = HERE.parents[2] / "crates" / "protocol" / "tests" / "fixtures" / "a12_affine"


def table(result, epoch, nym):
    return next(line.split("\t") for line in result["epochs"][epoch]["artifact"].decode().splitlines()
                if line.startswith("row\t" + nym + "\t"))


class AffineExportTests(unittest.TestCase):
    def test_golden_two_epoch_regeneration_and_drift(self):
        # AT-REP-01: canonical full-Nym producer regenerates the Rust-consumed two-epoch bytes.
        import json
        doc = json.loads(FIXTURE.read_text())
        result = export_epochs(doc)
        self.assertEqual(result["status"], "Complete", result)
        for i, epoch in enumerate(result["epochs"]):
            data = (GOLDEN / f"epoch-{i}.affine.tsv").read_bytes()
            self.assertEqual(epoch["artifact"], data)
            self.assertEqual(epoch["sha256"], hashlib.sha256(data).hexdigest())
            self.assertIn(f"\t{epoch['id']}\t{epoch['cut']}\t".encode(), data.splitlines()[0])
            self.assertEqual(len([line for line in data.splitlines() if line.startswith(b"row\t")]), 4)
        self.assertNotEqual(result["epochs"][0]["sha256"], result["epochs"][1]["sha256"])
        altered = copy.deepcopy(doc)
        set_report(altered, "a-e0-u-0", "1")
        changed = export_epochs(altered)
        self.assertEqual(changed["status"], "Complete", changed)
        self.assertNotEqual(changed["epochs"][1]["sha256"], result["epochs"][1]["sha256"])

    def test_bits_bounds_founder_probation_and_coefficients(self):
        # AT-REP-01: one nearest conversion has the absolute bound and utility bounds are rational.
        import json
        doc = json.loads(FIXTURE.read_text())
        result = export_epochs(doc)
        for i, epoch in enumerate(result["epochs"]):
            exact = evaluate(doc)["epochs"][i]
            for line in epoch["artifact"].decode().splitlines()[1:5]:
                parts = line.split("\t")
                nym = parts[1]
                bits = int(parts[8], 16)
                weight = struct.unpack(">d", bits.to_bytes(8, "big"))[0]
                self.assertEqual(Fraction(parts[7]), Fraction(exact["reviewers"][nym]["weight"]))
                self.assertLessEqual(abs(Fraction.from_float(weight) - Fraction(parts[7])), Fraction(1, 2**53))
                self.assertEqual(Fraction(parts[11]), int(parts[4]) * Fraction.from_float(weight))
                self.assertLessEqual(abs(Fraction(parts[11]) - Fraction(parts[9])), Fraction(parts[13]))
                self.assertLessEqual(abs(Fraction(parts[12]) - Fraction(parts[10])), Fraction(parts[14]))
                self.assertEqual(Fraction(parts[13]), Fraction(int(parts[4]), 2**53))
                self.assertEqual(Fraction(parts[15]), 2 * Fraction(parts[14]))
                if int(parts[3]) < 30:
                    self.assertEqual(weight, float(int(parts[2])))
        nym = f"{2:064x}"
        self.assertEqual(table(result, 0, nym)[8], "0000000000000000")
        self.assertEqual(table(result, 1, nym)[3], "30")

    def test_unavailable_and_alias_refusal(self):
        # AT-REP-01: missing required evidence and alias IDs yield no numerical table.
        doc = experiment({"u": 30})
        self.assertEqual(export_epochs(doc)["status"], "Unavailable")
        import json
        complete = json.loads(FIXTURE.read_text())
        del complete["snapshot"]["terminals"][0]
        refused = export_epochs(complete)
        self.assertEqual(refused["status"], "Unavailable")
        self.assertNotIn("epochs", refused)

    def test_cli_publishes_once_and_refuses_partial_exports(self):
        # AT-REP-01: a new directory becomes visible only with all epoch files and its index.
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / "published"
            command = [sys.executable, str(HERE / "export_a12_affine.py"), str(FIXTURE), str(target)]
            first = subprocess.run(command, capture_output=True, text=True, check=False)
            self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
            original = {p.name: p.read_bytes() for p in target.iterdir()}
            self.assertEqual(set(original), {"epoch-0.affine.tsv", "epoch-1.affine.tsv", "index.json"})
            second = subprocess.run(command, capture_output=True, text=True, check=False)
            self.assertEqual(second.returncode, 2)
            self.assertEqual({p.name: p.read_bytes() for p in target.iterdir()}, original)
            incomplete = json.loads(FIXTURE.read_text())
            incomplete["snapshot"]["terminals"].pop(0)
            bad_input = root / "incomplete.json"
            bad_input.write_text(json.dumps(incomplete))
            refused = subprocess.run([sys.executable, str(HERE / "export_a12_affine.py"),
                                     str(bad_input), str(root / "incomplete-output")],
                                     capture_output=True, text=True, check=False)
            self.assertEqual(refused.returncode, 2)
            self.assertFalse((root / "incomplete-output").exists())
            missing_parent = root / "missing-parent" / "out"
            failed = subprocess.run([sys.executable, str(HERE / "export_a12_affine.py"),
                                     str(FIXTURE), str(missing_parent)],
                                     capture_output=True, text=True, check=False)
            self.assertEqual(failed.returncode, 2)
            self.assertFalse(missing_parent.exists())
            staged_target = root / "write-failure"
            with patch.object(Path, "write_bytes", side_effect=OSError("synthetic write failure")):
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    self.assertEqual(main([str(FIXTURE), str(staged_target)]), 2)
            self.assertIn("Unavailable", output.getvalue())
            self.assertFalse(staged_target.exists())
            self.assertFalse(any(p.name.startswith(".a12-stage-") for p in root.iterdir()))

    def test_rounding_counterexample_and_uniform_advantage_bound(self):
        # AT-REP-01: one rounded path reverses the tiny exact gain; 2^-52 per opportunity covers it.
        q = Fraction(3, 4)
        deviation = Fraction(6755399441055711, 9007199254740992)
        nym = f"{1:064x}"
        def utility(p, outcome):
            doc = experiment({"u": 30}, epoch_counts=[{"u": 1}, {"u": 1}])
            doc["manifest"]["parameters"]["eta"] = "1/2"
            doc["manifest"]["reviewers"][0]["id"] = nym
            for assignment in doc["manifest"]["assignments"]:
                assignment["reviewer"] = nym
            for epoch in doc["manifest"]["epochs"]:
                for field in ("prefixes", "opportunities"):
                    epoch[field][nym] = epoch[field].pop("u")
            doc["manifest"]["initial_history_hash"] = initial_history_hash(doc["manifest"], doc["snapshot"])
            set_report(doc, "a-e0-u-0", str(p))
            set_outcome(doc, "a-e0-u-0", outcome)
            output = export_epochs(doc)
            self.assertEqual(output["status"], "Complete", output)
            return Fraction(evaluate(doc)["cumulative_utility"][nym]), table(output, 1, nym)[8]
        ta, tab = utility(q, "A")
        tr, trb = utility(q, "R")
        da, dab = utility(deviation, "A")
        dr, drb = utility(deviation, "R")
        self.assertEqual(tab, dab)
        self.assertEqual(tab, "3ff00c6318c6318c")
        self.assertEqual(trb, "3fefd6b5ad6b5ad7")
        self.assertEqual(drb, "3fefd6b5ad6b5ad8")
        truth = q * ta + (1 - q) * tr
        dev = q * da + (1 - q) * dr
        self.assertGreater(truth - dev, 0)
        def f(bits): return Fraction.from_float(struct.unpack(">d", int(bits, 16).to_bytes(8, "big"))[0])
        rounded_truth = q * f(tab) + (1 - q) * f(trb)
        rounded_dev = q * f(dab) + (1 - q) * f(drb)
        self.assertEqual(rounded_dev - rounded_truth, Fraction(1, 2**55))
        self.assertLessEqual(rounded_dev - rounded_truth, Fraction(2, 2**52))


if __name__ == "__main__":
    unittest.main()
