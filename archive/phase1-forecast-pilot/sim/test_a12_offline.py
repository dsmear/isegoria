"""Composed exact offline acceptance/refusal tests; docs/16 §7.2, AT-REP-01."""

import copy
import itertools
import json
import os
import subprocess
import sys
import unittest
from fractions import Fraction
from pathlib import Path

from a12_offline import canonical_json, content_hash, evaluate, evaluate_json, initial_history_hash


HERE = Path(__file__).resolve().parent


def experiment(initial_counts=None, founders=(), epoch_counts=None):
    initial_counts = {"u": 30} if initial_counts is None else initial_counts
    epoch_counts = [{u: 1 for u in initial_counts}, {u: 1 for u in initial_counts}] \
        if epoch_counts is None else epoch_counts
    manifest = {
        "version": 1, "frozen_at": 50,
        "assumptions": {key: True for key in (
            "ex_ante_fixed_calendar", "common_initial_history", "policy_invariant_outcomes",
            "policy_invariant_joint_information", "policy_invariant_completion",
            "no_report_controlled_resources")},
        "parameters": {"eta": "4/5", "k0": "0"},
        "reviewers": [{"id": u, "founder": u in founders} for u in initial_counts],
        "groups": [], "assignments": [], "epochs": [], "initial_history_hash": "",
        "weight_policy": "affine_complete_v1", "diagnostic_policy": "none",
    }
    snapshot = {"observed_at": 115 + 20 * (len(epoch_counts) - 1),
                "evidence_mode": "trusted_local_assertion", "reports": [], "terminals": []}
    histories = {u: [] for u in initial_counts}
    stages = [("initial", initial_counts)] + [(f"e{i}", counts) for i, counts in enumerate(epoch_counts)]
    for stage, counts in stages:
        if stage == "initial":
            report_time, terminal_time = 10, 20
        else:
            i = int(stage[1:])
            cut = 100 + 20 * i
            manifest["epochs"].append({"id": stage, "cut": cut, "opens_at": cut + 2,
                                       "closes_at": cut + 10,
                                       "prefixes": copy.deepcopy(histories),
                                       "opportunities": counts.copy()})
            report_time, terminal_time = cut + 3, cut + 11
        group = {"id": f"g-{stage}", "rule": "intrinsic-ARI-v1", "reference": f"ref-{stage}",
                 "verifier": "local-verifier", "items": []}
        for u, count in counts.items():
            for index in range(count):
                item, assignment = f"i-{stage}-{u}-{index}", f"a-{stage}-{u}-{index}"
                group["items"].append(item)
                manifest["assignments"].append({"id": assignment, "reviewer": u, "item": item,
                                                "epoch": stage})
                histories[u].append(assignment)
                snapshot["reports"].append({"assignment": assignment, "p": "1/2",
                                             "frozen_at": report_time, "available_at": report_time + 1})
                snapshot["terminals"].append({"item": item, "group": group["id"],
                    "rule": group["rule"], "reference": group["reference"],
                    "verifier": group["verifier"], "outcome": "A", "terminal_at": terminal_time,
                    "disclosed_at": terminal_time + 1, "available_at": terminal_time + 2,
                    "verified": True, "evidence_id": f"ev-{item}"})
        while len(group["items"]) < 2:
            group["items"].append(f"unassigned-{stage}-{len(group['items'])}")
        manifest["groups"].append(group)
    manifest["initial_history_hash"] = initial_history_hash(manifest, snapshot)
    return {"manifest": manifest, "snapshot": snapshot}


def set_report(document, assignment, p):
    next(r for r in document["snapshot"]["reports"] if r["assignment"] == assignment)["p"] = p


def set_outcome(document, assignment, outcome):
    item = next(a["item"] for a in document["manifest"]["assignments"] if a["id"] == assignment)
    next(t for t in document["snapshot"]["terminals"] if t["item"] == item)["outcome"] = outcome


def reanchor(document):
    document["manifest"]["initial_history_hash"] = initial_history_hash(document["manifest"], document["snapshot"])


class OfflineA12Tests(unittest.TestCase):
    def complete(self, document):
        result = evaluate(document)
        self.assertEqual(result["status"], "Complete", result)
        return result

    def unavailable(self, document, reason=None):
        result = evaluate(document)
        self.assertEqual(result["status"], "Unavailable", result)
        self.assertTrue(result["reasons"])
        self.assertTrue({"epochs", "scores", "cumulative_utility"}.isdisjoint(result))
        if reason:
            self.assertIn(reason, " ".join(result["reasons"]))
        return result

    def test_fixed_score_endpoints_and_intrinsic_I(self):
        # AT-REP-01/05: fixed benchmark endpoints and intrinsic I use the executable score path.
        doc = experiment({"u": 0}, epoch_counts=[{"u": 7}])
        cases = [("0", "A", "-3/4"), ("1", "A", "1/4"), ("1/2", "A", "0"),
                 ("1", "R", "-3/4"), ("0", "R", "1/4"), ("1/2", "R", "0"),
                 ("3/7", "I", "0")]
        for i, (p, outcome, expected) in enumerate(cases):
            assignment = f"a-e0-u-{i}"
            set_report(doc, assignment, p)
            set_outcome(doc, assignment, outcome)
        result = self.complete(doc)
        for i, (_, _, expected) in enumerate(cases):
            self.assertEqual(result["scores"][f"a-e0-u-{i}"], expected)

    def test_probation_29_to_30_counts_verified_I(self):
        # AT-REP-01: terminal I advances complete exposure and does not impute a conclusive score.
        doc = experiment({"u": 29})
        set_outcome(doc, "a-e0-u-0", "I")
        result = self.complete(doc)
        first, second = [e["reviewers"]["u"] for e in result["epochs"]]
        self.assertEqual((first["n"], first["weight"], first["score_sum"]), (29, "0", "0"))
        self.assertEqual((second["n"], second["weight"], second["score_sum"]), (30, "1", "0"))
        self.assertEqual(result["cumulative_utility"]["u"], "1")

    def test_founder_bootstrap_transitions_to_affine(self):
        # AT-REP-01: founder unit bootstrap ends exactly at the thirtieth observed terminal.
        doc = experiment({"u": 29}, founders=("u",))
        set_report(doc, "a-e0-u-0", "1")
        result = self.complete(doc)
        self.assertEqual(result["epochs"][0]["reviewers"]["u"]["weight"], "1")
        self.assertEqual(result["epochs"][1]["reviewers"]["u"]["weight"], "151/150")

    def test_empty_history_zero_k0_has_no_probation_division(self):
        # AT-REP-01: zero k0 and no carrier give defined probation outputs.
        result = self.complete(experiment({"u": 0}))
        self.assertEqual(result["epochs"][0]["cap"], None)
        self.assertEqual(result["cumulative_utility"], {"u": "0"})

    def test_mixed_roster_affine_bounds_and_slack_cap(self):
        # AT-REP-01/04: homogeneous roster composes bounds, founders and probation.
        doc = experiment({"low": 30, "high": 30, "founder": 0, "new": 0}, founders=("founder",))
        for report in doc["snapshot"]["reports"]:
            if report["assignment"].startswith("a-initial-low-"):
                report["p"] = "0"
            elif report["assignment"].startswith("a-initial-high-"):
                report["p"] = "1"
        reanchor(doc)
        result = self.complete(doc)
        first = result["epochs"][0]
        self.assertEqual(first["cap"], "3")
        expected = {"low": "2/5", "high": "6/5", "founder": "1", "new": "0"}
        for u, weight in expected.items():
            self.assertEqual(first["reviewers"][u]["weight"], weight)
            self.assertEqual(first["reviewers"][u]["raw_weight"], weight)
        for epoch in result["epochs"]:
            for row in epoch["reviewers"].values():
                self.assertEqual(row["raw_weight"], row["weight"])

    def test_even_roster_median_and_weighted_opportunities(self):
        # AT-REP-01: opportunities multiply the weight frozen before current reports.
        doc = experiment({"u": 30, "v": 30}, epoch_counts=[{"u": 2, "v": 1}, {"u": 3, "v": 0}])
        set_report(doc, "a-e0-u-0", "1")
        set_report(doc, "a-e0-u-1", "1")
        result = self.complete(doc)
        self.assertEqual(result["epochs"][0]["reviewers"]["u"]["contribution"], "2")
        self.assertEqual(result["epochs"][1]["reviewers"]["u"]["weight"], "81/80")
        self.assertEqual(result["epochs"][1]["cap"], "483/160")
        self.assertEqual(result["cumulative_utility"]["u"], "403/80")
        self.assertEqual(result["cumulative_utility"]["v"], "1")

    def test_exact_finite_adaptive_policy_enumeration(self):
        # AT-REP-01/05: enumerate common information/outcomes through the executable path.
        template = experiment({"u": 30}, epoch_counts=[{"u": 1}] * 3)
        law = [(("A", "A"), Fraction(3, 8)), (("A", "R"), Fraction(1, 8)),
               (("R", "A"), Fraction(1, 8)), (("R", "R"), Fraction(3, 8))]
        policies = [(first, after_a, after_r)
                    for first in ("0", "1/2", "1")
                    for after_a in ("0", "3/4", "1")
                    for after_r in ("0", "1/4", "1")]
        def expected(policy):
            utility = Fraction(0)
            for (one, two), probability in law:
                doc = copy.deepcopy(template)
                set_outcome(doc, "a-e0-u-0", one)
                set_outcome(doc, "a-e1-u-0", two)
                set_outcome(doc, "a-e2-u-0", "I")
                set_report(doc, "a-e0-u-0", policy[0])
                set_report(doc, "a-e1-u-0", policy[1] if one == "A" else policy[2])
                utility += probability * Fraction(self.complete(doc)["cumulative_utility"]["u"])
            return utility
        truthful = expected(("1/2", "3/4", "1/4"))
        self.assertEqual(truthful, Fraction(1921, 640))
        for policy in policies:
            with self.subTest(policy=policy):
                self.assertLessEqual(expected(policy), truthful)
        self.assertLess(expected(("1", "1", "1")), truthful)

    def test_c_zero_and_beta_zero_are_flat_without_imputed_forecast(self):
        # AT-REP-01/05: intrinsic I and a final report have flat utility for every tested forecast.
        template = experiment({"u": 30}, epoch_counts=[{"u": 1}] * 3)
        set_outcome(template, "a-e0-u-0", "I")
        utilities = set()
        for first, last in itertools.product(("0", "2/7", "1"), repeat=2):
            doc = copy.deepcopy(template)
            set_report(doc, "a-e0-u-0", first)
            set_report(doc, "a-e2-u-0", last)
            utilities.add(self.complete(doc)["cumulative_utility"]["u"])
        self.assertEqual(utilities, {"3"})

    def test_same_item_cohort_shares_one_terminal(self):
        # AT-REP-01: reviewers share one item outcome, never per-reviewer labels.
        doc = experiment({"u": 30, "v": 30})
        old_item = next(a["item"] for a in doc["manifest"]["assignments"] if a["id"] == "a-e0-v-0")
        next(a for a in doc["manifest"]["assignments"] if a["id"] == "a-e0-v-0")["item"] = "i-e0-u-0"
        doc["snapshot"]["terminals"] = [t for t in doc["snapshot"]["terminals"] if t["item"] != old_item]
        set_report(doc, "a-e0-u-0", "1")
        set_report(doc, "a-e0-v-0", "0")
        result = self.complete(doc)
        self.assertEqual(result["scores"]["a-e0-u-0"], "1/4")
        self.assertEqual(result["scores"]["a-e0-v-0"], "-3/4")
        conflicting = copy.deepcopy(next(t for t in doc["snapshot"]["terminals"] if t["item"] == "i-e0-u-0"))
        conflicting["outcome"] = "R"
        doc["snapshot"]["terminals"].append(conflicting)
        self.unavailable(doc, "duplicate item")

    def test_missing_report_or_terminal_refuses_entire_horizon(self):
        # AT-REP-01: missing any contributor cannot certify a partial roster or hold an old weight.
        for field in ("reports", "terminals"):
            doc = experiment({"u": 30, "v": 30})
            doc["snapshot"][field].pop(60)
            self.unavailable(doc, "missing")

    def test_pending_unverified_and_unsupported_evidence(self):
        # AT-REP-01: pending, unverified and unsupported observations never become terminal I.
        for field, value, reason in (("outcome", "Pending", "intrinsic"),
                                     ("verified", False, "positive trusted"),
                                     ("verified", 1, "positive trusted")):
            doc = experiment()
            doc["snapshot"]["terminals"][-1][field] = value
            self.unavailable(doc, reason)
        doc = experiment()
        doc["snapshot"]["evidence_mode"] = "verified_signature"
        self.unavailable(doc, "unsupported evidence")

    def test_later_evidence_cannot_retroactively_certify_original_cut(self):
        # AT-REP-01: new snapshots keep the prefixed cut; late arrival never repairs a prior epoch.
        doc = experiment()
        terminal = next(t for t in doc["snapshot"]["terminals"] if t["item"] == "i-e0-u-0")
        terminal["available_at"] = 130
        self.unavailable(doc, "original cut")
        old_hash = content_hash(doc["snapshot"])
        doc["snapshot"]["observed_at"] = 200
        self.assertNotEqual(old_hash, content_hash(doc["snapshot"]))
        self.unavailable(doc, "original cut")

    def test_late_report_and_terminal_positions(self):
        # AT-REP-01: report availability, terminal positions and snapshot bounds differ.
        doc = experiment()
        next(r for r in doc["snapshot"]["reports"] if r["assignment"] == "a-e0-u-0")["available_at"] = 121
        self.unavailable(doc, "report unavailable at original cut")
        for changes in ({"terminal_at": 140}, {"disclosed_at": 110}):
            doc = experiment()
            next(t for t in doc["snapshot"]["terminals"] if t["item"] == "i-e0-u-0").update(changes)
            self.unavailable(doc)

    def test_group_rule_reference_verifier_associations(self):
        # AT-REP-01: terminal assertions must match every prefixed group/procedure association.
        for field in ("group", "rule", "reference", "verifier"):
            doc = experiment()
            doc["snapshot"]["terminals"][-1][field] = "other"
            self.unavailable(doc, "does not match")

    def test_prefix_cannot_skip_reorder_or_include_current_assignment(self):
        # AT-REP-01: assigned exposures fix the full prefix, independent of completion or reports.
        for mutation in (lambda p: p.pop(0), lambda p: p.reverse(), lambda p: p.append("a-e1-u-0")):
            doc = experiment()
            mutation(doc["manifest"]["epochs"][1]["prefixes"]["u"])
            self.unavailable(doc, "every prior assignment")

    def test_roster_and_opportunities_are_coupled_to_assignments(self):
        # AT-REP-01: neither the cap roster nor opportunity weights can omit or invent contributors.
        doc = experiment({"u": 30, "v": 30})
        del doc["manifest"]["epochs"][0]["prefixes"]["v"]
        self.unavailable(doc, "whole roster")
        doc = experiment()
        doc["manifest"]["epochs"][0]["opportunities"]["u"] = 99
        self.unavailable(doc, "equal assigned")

    def test_duplicate_records_and_cohort_contributors(self):
        # AT-REP-01: duplicate IDs and reviewer/item pairs never count twice.
        for field, container in (("reports", "snapshot"), ("terminals", "snapshot"),
                                 ("assignments", "manifest"), ("reviewers", "manifest")):
            doc = experiment()
            doc[container][field].append(copy.deepcopy(doc[container][field][0]))
            self.unavailable(doc, "duplicate")
        doc = experiment()
        doc["manifest"]["assignments"][-1]["item"] = "i-e0-u-0"
        self.unavailable(doc, "duplicate reviewer/item")

    def test_initial_history_is_fixed_before_compared_reports(self):
        # AT-REP-01: initial reports and outcomes are pinned before compared policies.
        doc = experiment()
        set_report(doc, "a-initial-u-0", "1")
        self.unavailable(doc, "initial records do not match")
        reanchor(doc)
        self.complete(doc)
        doc["snapshot"]["reports"][0]["available_at"] = 51
        reanchor(doc)
        self.unavailable(doc, "initial report unavailable")

    def test_manifest_and_epoch_ordering_are_enforced(self):
        # AT-REP-01: policy inputs freeze before current reports and opportunities.
        doc = experiment()
        doc["manifest"]["epochs"][0]["cut"] = 103
        self.unavailable(doc, "cut < opens")
        doc = experiment()
        doc["manifest"]["epochs"][1]["cut"] = 105
        self.unavailable(doc, "previous close")
        doc = experiment()
        next(r for r in doc["snapshot"]["reports"] if r["assignment"] == "a-e0-u-0")["frozen_at"] = 101
        self.unavailable(doc, "assigned epoch")

    def test_unknown_fields_bool_counts_and_unaccepted_policies(self):
        # AT-REP-01: strict schema refuses unknown data and alternative weight policies.
        for mutate in (
            lambda d: d["manifest"].update(extra="data"),
            lambda d: d["snapshot"]["reports"][0].update(demographic="forbidden"),
            lambda d: d["manifest"]["epochs"][0]["opportunities"].update(u=True),
            lambda d: d["manifest"].update(weight_policy="odds_reset"),
            lambda d: d["manifest"].update(diagnostic_policy="punitive_CUSUM"),
            lambda d: d["manifest"]["assumptions"].update(policy_invariant_joint_information=False),
        ):
            doc = experiment()
            mutate(doc)
            self.unavailable(doc)

    def test_rational_grammar_and_parameter_domain(self):
        # AT-REP-01: probabilities and parameters are canonical fractions in their domains.
        for value in (0.5, True, "0.5", "2/4", "-0", "1/0", "2", "-1/2", "NaN"):
            doc = experiment()
            doc["snapshot"]["reports"][-1]["p"] = value
            self.unavailable(doc)
        for eta, k0 in (("0", "0"), ("1", "0"), ("4/5", "-1")):
            doc = experiment()
            doc["manifest"]["parameters"] = {"eta": eta, "k0": k0}
            self.unavailable(doc)
        doc = experiment()
        del doc["manifest"]["parameters"]["eta"]
        self.unavailable(doc)

    def test_duplicate_json_keys_float_and_nonfinite_rejected(self):
        # AT-REP-01: the JSON parser refuses lossy numbers and duplicate keys before interpretation.
        for source in ('{"manifest":{},"manifest":{}}', '{"x":0.5}', '{"x":NaN}', '{"x":Infinity}', '{'):
            result = evaluate_json(source)
            self.assertEqual(result["status"], "Unavailable")
            self.assertNotIn("cumulative_utility", result)

    def test_cli_replays_canonical_bytes_and_order_independent_objects(self):
        # AT-REP-01: library and CLI replay the same complete snapshot byte-for-byte.
        doc = experiment()
        expected = canonical_json(self.complete(doc)).encode()
        for text in (json.dumps(doc), json.dumps(doc, sort_keys=True, indent=2)):
            result = subprocess.run([sys.executable, str(HERE / "a12_offline.py"), "-"],
                                    input=text.encode(), capture_output=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout, expected)
        doc["snapshot"]["terminals"].pop(30)
        result = subprocess.run([sys.executable, str(HERE / "a12_offline.py"), "-"],
                                input=json.dumps(doc).encode(), capture_output=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(result.stdout)["status"], "Unavailable")

    def test_unconsumed_final_terminal_is_not_required_or_imputed(self):
        # AT-REP-01: final reports with beta=0 need no terminal for horizon utility.
        doc = experiment()
        before = self.complete(doc)
        doc["snapshot"]["terminals"].pop()
        after = self.complete(doc)
        self.assertEqual(after["cumulative_utility"], before["cumulative_utility"])
        self.assertEqual(after["unscored_assignments"], ["a-e1-u-0"])
        self.assertNotIn("a-e1-u-0", after["scores"])
        doc["snapshot"]["reports"].pop()
        self.unavailable(doc, "missing frozen report")

    def test_undisclosed_terminal_may_predate_report(self):
        # AT-REP-01: the declared disclosure time, not hidden terminal formation, bounds the freeze.
        doc = experiment()
        terminal = next(t for t in doc["snapshot"]["terminals"] if t["item"] == "i-e0-u-0")
        terminal["terminal_at"] = 101
        self.complete(doc)
        for disclosure in (102, 103):
            terminal["disclosed_at"] = disclosure
            self.unavailable(doc, "precede terminal disclosure")

    def test_oversized_json_integer_returns_unavailable(self):
        # AT-REP-01: parser integer limits cause structured refusal rather than a CLI traceback.
        result = evaluate_json('{"value":' + '9' * 10000 + '}')
        self.assertEqual(result["status"], "Unavailable")

    def test_large_exact_result_refuses_atomically_at_encoding_limit(self):
        # AT-REP-01: serializing a squared denominator beyond the profile limit never leaks weights.
        doc = experiment({"u": 0}, founders=("u",), epoch_counts=[{"u": 1}])
        set_report(doc, "a-e0-u-0", "1/" + "1" + "0" * 2199 + "7")
        self.unavailable(doc, "encoding limit")

    def test_nonstandard_integer_limit_is_unsupported_environment(self):
        # AT-REP-01: integer-limit settings cannot silently change replay availability.
        env = dict(os.environ, PYTHONINTMAXSTRDIGITS="0")
        result = subprocess.run([sys.executable, str(HERE / "a12_offline.py"), "-"],
                                input=json.dumps(experiment()).encode(), env=env,
                                capture_output=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertIn("unsupported numerical profile", json.loads(result.stdout)["reasons"][0])

    def test_fixture_files_complete_and_unavailable(self):
        # AT-REP-01: committed small examples exercise callable input decoding and refusal outputs.
        for name, status in (("a12_complete.json", "Complete"), ("a12_incomplete.json", "Unavailable")):
            result = evaluate_json((HERE / "fixtures" / name).read_text())
            self.assertEqual(result["status"], status, result)


if __name__ == "__main__":
    unittest.main()
