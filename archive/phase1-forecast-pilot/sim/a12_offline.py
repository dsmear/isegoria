"""Exact offline A12 candidate; contract: docs/16 §7.2 and sim/README.md.
Trusted local assertions and declared invariance are not authenticated evidence.
"""

import hashlib
import json
import re
import sys
from fractions import Fraction
from pathlib import Path


NUMERICAL_PROFILE = "cpython-3.11+-int4300-v1"


class InvalidInput(ValueError):
    pass


def _require_profile():
    if (sys.implementation.name != "cpython" or sys.version_info < (3, 11)
            or sys.get_int_max_str_digits() != 4300):
        _fail("environment", f"unsupported numerical profile; requires {NUMERICAL_PROFILE}")


def canonical_json(value):
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n"


def content_hash(value):
    return hashlib.sha256(canonical_json(value).encode("ascii")).hexdigest()


def _fail(path, reason):
    raise InvalidInput(f"{path}: {reason}")


def _keys(value, fields, path):
    if type(value) is not dict or set(value) != set(fields.split()):
        _fail(path, f"expected exactly fields [{fields}]")


def _array(value, path):
    if type(value) is not list:
        _fail(path, "expected array")
    return value


def _integer(value, path):
    if type(value) is not int or value < 0:
        _fail(path, "expected nonnegative integer (not bool)")
    return value


def _id(value, path):
    if type(value) is not str or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.:-]{0,63}", value):
        _fail(path, "expected opaque ASCII ID of 1..64 characters")
    return value


def _rational(value, path):
    if type(value) is not str or not re.fullmatch(r"-?(0|[1-9][0-9]*)(/[1-9][0-9]*)?", value):
        _fail(path, "expected canonical rational string")
    try:
        result = Fraction(value)
    except (ValueError, ZeroDivisionError):
        _fail(path, "invalid rational")
    if str(result) != value:
        _fail(path, "rational must be reduced, with no redundant sign or denominator")
    return result


def _indexed(rows, fields, key, path):
    result = {}
    for i, row in enumerate(_array(rows, path)):
        loc = f"{path}[{i}]"
        _keys(row, fields, loc)
        identity = _id(row[key], f"{loc}.{key}")
        if identity in result:
            _fail(loc, f"duplicate {key} {identity}")
        result[identity] = row
    return result


def _unique_ids(values, path):
    values = _array(values, path)
    for value in values:
        _id(value, path)
    if len(set(values)) != len(values):
        _fail(path, "duplicate ID")
    return values


def _no_nonfinite(value):
    _fail("JSON", f"unsupported numeric literal {value}")


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            _fail("JSON", f"duplicate object key {key}")
        result[key] = value
    return result


def load_document(text):
    """Decode JSON without duplicate keys, floats or nonfinite numbers."""
    _require_profile()
    try:
        return json.loads(text, object_pairs_hook=_pairs,
                          parse_float=_no_nonfinite, parse_constant=_no_nonfinite)
    except (ValueError, UnicodeError, RecursionError) as error:
        raise InvalidInput(f"JSON: {error}") from error


def initial_history_hash(manifest, snapshot):
    """Hash declared initial records; provenance only, never authentication."""
    initial = {a["id"] for a in manifest["assignments"] if a["epoch"] == "initial"}
    items = {a["item"] for a in manifest["assignments"] if a["id"] in initial}
    return content_hash({
        "reports": sorted((r for r in snapshot["reports"] if r["assignment"] in initial),
                          key=lambda r: r["assignment"]),
        "terminals": sorted((t for t in snapshot["terminals"] if t["item"] in items),
                            key=lambda t: t["item"]),
    })


def _validate(document):
    _keys(document, "manifest snapshot", "document")
    manifest, snapshot = document["manifest"], document["snapshot"]
    _keys(manifest, "version frozen_at assumptions parameters reviewers groups assignments epochs "
          "initial_history_hash weight_policy diagnostic_policy", "manifest")
    _keys(snapshot, "observed_at evidence_mode reports terminals", "snapshot")
    if type(manifest["version"]) is not int or manifest["version"] != 1:
        _fail("manifest.version", "supported version is integer 1")
    if manifest["weight_policy"] != "affine_complete_v1":
        _fail("manifest.weight_policy", "unsupported policy")
    if manifest["diagnostic_policy"] != "none":
        _fail("manifest.diagnostic_policy", "CUSUM contract is not implemented")
    if snapshot["evidence_mode"] != "trusted_local_assertion":
        _fail("snapshot.evidence_mode", "unsupported evidence")
    frozen = _integer(manifest["frozen_at"], "manifest.frozen_at")
    observed = _integer(snapshot["observed_at"], "snapshot.observed_at")
    if observed < frozen:
        _fail("snapshot.observed_at", "precedes manifest freeze")
    assumptions = "ex_ante_fixed_calendar common_initial_history policy_invariant_outcomes " \
                  "policy_invariant_joint_information policy_invariant_completion " \
                  "no_report_controlled_resources"
    _keys(manifest["assumptions"], assumptions, "manifest.assumptions")
    if any(v is not True for v in manifest["assumptions"].values()):
        _fail("manifest.assumptions", "all domain assertions must be true")
    _keys(manifest["parameters"], "eta k0", "manifest.parameters")
    eta = _rational(manifest["parameters"]["eta"], "parameters.eta")
    k0 = _rational(manifest["parameters"]["k0"], "parameters.k0")
    if not 0 < eta <= Fraction(4, 5) or k0 < 0:
        _fail("manifest.parameters", "requires 0 < eta <= 4/5 and k0 >= 0")
    reviewers = _indexed(manifest["reviewers"], "id founder", "id", "reviewers")
    if not reviewers:
        _fail("reviewers", "empty roster")
    for key, reviewer in reviewers.items():
        if type(reviewer["founder"]) is not bool:
            _fail(f"reviewers.{key}.founder", "expected boolean")
    groups = _indexed(manifest["groups"], "id rule reference verifier items", "id", "groups")
    item_groups = {}
    for key, group in groups.items():
        for field in ("rule", "reference", "verifier"):
            _id(group[field], f"groups.{key}.{field}")
        items = _unique_ids(group["items"], f"groups.{key}.items")
        if len(items) < 2:
            _fail(f"groups.{key}", "empirical group must contain at least two items")
        for item in items:
            if item in item_groups:
                _fail(f"groups.{key}", f"item {item} belongs to multiple groups")
            item_groups[item] = group
    epochs = _indexed(manifest["epochs"], "id cut opens_at closes_at prefixes opportunities",
                      "id", "epochs")
    if not epochs or "initial" in epochs:
        _fail("epochs", "nonempty horizon required; initial is reserved")
    previous_close = frozen
    epoch_index = {"initial": -1}
    for index, (key, epoch) in enumerate(epochs.items()):
        epoch_index[key] = index
        cut = _integer(epoch["cut"], f"epochs.{key}.cut")
        opens = _integer(epoch["opens_at"], f"epochs.{key}.opens_at")
        closes = _integer(epoch["closes_at"], f"epochs.{key}.closes_at")
        if not previous_close <= cut < opens <= closes:
            _fail(f"epochs.{key}", "requires previous close <= cut < opens <= closes")
        if closes > observed:
            _fail(f"epochs.{key}", "horizon extends past snapshot observation")
        previous_close = closes
        for field in ("prefixes", "opportunities"):
            if type(epoch[field]) is not dict or set(epoch[field]) != set(reviewers):
                _fail(f"epochs.{key}.{field}", "must contain exactly the whole roster")
        for reviewer in reviewers:
            _unique_ids(epoch["prefixes"][reviewer], f"epochs.{key}.prefixes.{reviewer}")
            _integer(epoch["opportunities"][reviewer], f"epochs.{key}.opportunities.{reviewer}")
    assignments = _indexed(manifest["assignments"], "id reviewer item epoch", "id", "assignments")
    histories = {reviewer: [] for reviewer in reviewers}
    item_epoch, contributor = {}, set()
    for key, assignment in assignments.items():
        reviewer, item, epoch = (assignment[field] for field in ("reviewer", "item", "epoch"))
        for field in ("reviewer", "item", "epoch"):
            _id(assignment[field], f"assignments.{key}.{field}")
        if reviewer not in reviewers or item not in item_groups or epoch not in epoch_index:
            _fail(f"assignments.{key}", "unknown reviewer, item or epoch")
        if (reviewer, item) in contributor:
            _fail(f"assignments.{key}", "duplicate reviewer/item contribution")
        contributor.add((reviewer, item))
        if item in item_epoch and item_epoch[item] != epoch:
            _fail(f"assignments.{key}", "item cohort spans different reporting epochs")
        item_epoch[item] = epoch
        history = histories[reviewer]
        if history and epoch_index[assignments[history[-1]]["epoch"]] > epoch_index[epoch]:
            _fail(f"assignments.{key}", "reviewer assignment order is not chronological")
        history.append(key)
    previous_prefixes = {reviewer: 0 for reviewer in reviewers}
    for key, epoch in epochs.items():
        for reviewer, history in histories.items():
            prefix = epoch["prefixes"][reviewer]
            expected = [a for a in history if epoch_index[assignments[a]["epoch"]] < epoch_index[key]]
            if prefix != expected or len(prefix) < previous_prefixes[reviewer]:
                _fail(f"epochs.{key}.prefixes.{reviewer}", "must include every prior assignment in order")
            previous_prefixes[reviewer] = len(prefix)
            count = sum(assignments[a]["epoch"] == key for a in history)
            if epoch["opportunities"][reviewer] != count:
                _fail(f"epochs.{key}.opportunities.{reviewer}", "must equal assigned current-epoch reports")
    reports = _indexed(snapshot["reports"], "assignment p frozen_at available_at",
                       "assignment", "reports")
    terminals = _indexed(snapshot["terminals"], "item group rule reference verifier outcome "
                         "terminal_at disclosed_at available_at verified evidence_id", "item", "terminals")
    for key, report in reports.items():
        if key not in assignments:
            _fail(f"reports.{key}", "unknown assignment")
        probability = _rational(report["p"], f"reports.{key}.p")
        if not 0 <= probability <= 1:
            _fail(f"reports.{key}.p", "must lie in [0,1]")
        freeze = _integer(report["frozen_at"], f"reports.{key}.frozen_at")
        available = _integer(report["available_at"], f"reports.{key}.available_at")
        if not freeze <= available <= observed:
            _fail(f"reports.{key}", "requires report freeze <= availability <= snapshot")
        epoch = assignments[key]["epoch"]
        if epoch == "initial":
            if available > frozen:
                _fail(f"reports.{key}", "initial report unavailable at manifest freeze")
        elif not frozen < epochs[epoch]["opens_at"] <= freeze <= epochs[epoch]["closes_at"]:
            _fail(f"reports.{key}", "report not frozen in its assigned epoch after manifest")
    evidence_ids = set()
    for item, terminal in terminals.items():
        if item not in item_groups:
            _fail(f"terminals.{item}", "unknown item")
        group = item_groups[item]
        for field in ("group", "rule", "reference", "verifier"):
            _id(terminal[field], f"terminals.{item}.{field}")
            expected = group["id"] if field == "group" else group[field]
            if terminal[field] != expected:
                _fail(f"terminals.{item}.{field}", "does not match declared group/reference procedure")
        evidence = _id(terminal["evidence_id"], f"terminals.{item}.evidence_id")
        if evidence in evidence_ids:
            _fail(f"terminals.{item}", "duplicate terminal evidence ID")
        evidence_ids.add(evidence)
        if terminal["outcome"] not in ("A", "R", "I"):
            _fail(f"terminals.{item}.outcome", "only intrinsic terminal A/R/I is supported")
        if terminal["verified"] is not True:
            _fail(f"terminals.{item}.verified", "requires positive trusted local assertion")
        positions = [_integer(terminal[field], f"terminals.{item}.{field}")
                     for field in ("terminal_at", "disclosed_at", "available_at")]
        if not positions[0] <= positions[1] <= positions[2] <= observed:
            _fail(f"terminals.{item}", "requires terminal <= disclosure <= availability <= snapshot")
        if item_epoch.get(item) == "initial" and positions[2] > frozen:
            _fail(f"terminals.{item}", "initial terminal unavailable at manifest freeze")
    digest = manifest["initial_history_hash"]
    if type(digest) is not str or not re.fullmatch(r"[0-9a-f]{64}", digest):
        _fail("manifest.initial_history_hash", "expected lowercase SHA-256")
    if digest != initial_history_hash(manifest, snapshot):
        _fail("manifest.initial_history_hash", "initial records do not match prefixed history hash")
    reasons = []
    required = {a for epoch in epochs.values() for prefix in epoch["prefixes"].values() for a in prefix}
    for key, assignment in assignments.items():
        report = reports.get(key)
        terminal = terminals.get(assignment["item"])
        if report is None:
            reasons.append(f"assignment {key}: missing frozen report")
        if terminal is None and key in required:
            reasons.append(f"assignment {key}: missing terminal evidence for a required prefix")
        if report is not None and terminal is not None:
            if report["frozen_at"] >= terminal["disclosed_at"]:
                reasons.append(f"assignment {key}: report must precede terminal disclosure")
    for key, epoch in epochs.items():
        for reviewer, prefix in epoch["prefixes"].items():
            for assignment in prefix:
                report = reports.get(assignment)
                terminal = terminals.get(assignments[assignment]["item"])
                if report is not None and report["available_at"] > epoch["cut"]:
                    reasons.append(f"epoch {key} assignment {assignment}: report unavailable at original cut")
                if terminal is not None and terminal["available_at"] > epoch["cut"]:
                    reasons.append(f"epoch {key} assignment {assignment}: terminal unavailable at original cut")
    if reasons:
        raise InvalidInput("; ".join(sorted(set(reasons))))
    return manifest, snapshot, reviewers, assignments, reports, terminals, eta, k0


def _unavailable(error, document=None):
    result = {"status": "Unavailable", "reasons": str(error).split("; ")}
    if type(document) is dict:
        for name in ("manifest", "snapshot"):
            if name in document:
                try:
                    result[f"{name}_id"] = content_hash(document[name])
                except (TypeError, ValueError, UnicodeError, RecursionError):
                    pass
    return result


def _evaluate(document):
    try:
        _require_profile()
        manifest, snapshot, reviewers, assignments, reports, terminals, eta, k0 = _validate(document)
    except InvalidInput as error:
        return _unavailable(error, document)
    try:
        manifest_id, snapshot_id = content_hash(manifest), content_hash(snapshot)
    except (ValueError, UnicodeError, RecursionError) as error:
        return _unavailable(f"canonical encoding: {error}")
    scores = {}
    for key, assignment in assignments.items():
        if assignment["item"] not in terminals:
            continue
        outcome = terminals[assignment["item"]]["outcome"]
        p = Fraction(reports[key]["p"])
        scores[key] = Fraction(0) if outcome == "I" else Fraction(1, 4) - (p - int(outcome == "A")) ** 2
    cumulative = {reviewer: Fraction(0) for reviewer in reviewers}
    results = []
    for epoch in manifest["epochs"]:
        raw, sums, counts = {}, {}, {}
        for reviewer in sorted(reviewers):
            prefix = epoch["prefixes"][reviewer]
            counts[reviewer] = len(prefix)
            sums[reviewer] = sum((scores[a] for a in prefix), Fraction(0))
            raw[reviewer] = (Fraction(int(reviewers[reviewer]["founder"])) if len(prefix) < 30
                             else 1 + eta * sums[reviewer] / (k0 + len(prefix)))
        carrying = sorted(weight for weight in raw.values() if weight > 0)
        if carrying:
            middle = len(carrying) // 2
            median = (carrying[middle] if len(carrying) % 2
                      else (carrying[middle - 1] + carrying[middle]) / 2)
            cap = 3 * median
        else:
            cap = None
        weights = {reviewer: min(weight, cap) if cap is not None else Fraction(0)
                   for reviewer, weight in raw.items()}
        entries = {}
        for reviewer in sorted(reviewers):
            contribution = epoch["opportunities"][reviewer] * weights[reviewer]
            cumulative[reviewer] += contribution
            entries[reviewer] = {"n": counts[reviewer], "score_sum": str(sums[reviewer]),
                                 "raw_weight": str(raw[reviewer]), "weight": str(weights[reviewer]),
                                 "opportunities": epoch["opportunities"][reviewer],
                                 "contribution": str(contribution),
                                 "cumulative_utility": str(cumulative[reviewer])}
        results.append({"id": epoch["id"], "cut": epoch["cut"],
                        "cap": str(cap) if cap is not None else None, "reviewers": entries})
    return {"status": "Complete", "scope": "trusted_local_complete_offline_A12_v1",
            "numerical_profile": NUMERICAL_PROFILE,
            "manifest_id": manifest_id, "snapshot_id": snapshot_id,
            "unscored_assignments": sorted(set(assignments) - set(scores)),
            "scores": {key: str(scores[key]) for key in sorted(scores)}, "epochs": results,
            "cumulative_utility": {key: str(cumulative[key]) for key in sorted(cumulative)}}


def evaluate(document):
    """Return complete exact horizon results or Unavailable without numeric weights."""
    try:
        return _evaluate(document)
    except (ValueError, OverflowError) as error:
        return _unavailable(f"exact computation or encoding limit: {error}", document)


def evaluate_json(text):
    try:
        return evaluate(load_document(text))
    except InvalidInput as error:
        return {"status": "Unavailable", "reasons": [str(error)]}


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if len(argv) != 1:
        print("usage: python3 sim/a12_offline.py INPUT.json (or - for stdin)", file=sys.stderr)
        return 2
    try:
        text = sys.stdin.read() if argv[0] == "-" else Path(argv[0]).read_text(encoding="utf-8")
        result = evaluate_json(text)
    except (OSError, UnicodeError) as error:
        result = {"status": "Unavailable", "reasons": [f"input: {error}"]}
    sys.stdout.write(canonical_json(result))
    return 0 if result["status"] == "Complete" else 2


if __name__ == "__main__":
    sys.exit(main())
