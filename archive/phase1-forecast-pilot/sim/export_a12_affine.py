"""Transfer complete exact A12 epochs to frozen binary64 tables; docs/16 §7.2."""

import hashlib
import json
import os
import re
import sys
import tempfile
from fractions import Fraction
from pathlib import Path

from a12_offline import NUMERICAL_PROFILE, InvalidInput, canonical_json, evaluate_json, load_document


NYM = re.compile(r"[0-9a-f]{64}\Z")
MAX_U64 = 2**64 - 1


def _bits(weight):
    import struct
    converted = float(weight)
    if abs(Fraction.from_float(converted) - weight) > Fraction(1, 2**53):
        raise InvalidInput("binary64 final weight exceeds absolute 2^-53 error")
    return f"{struct.unpack('>Q', struct.pack('>d', converted))[0]:016x}", Fraction.from_float(converted)


def export_epochs(document):
    try:
        result = evaluate_json(canonical_json(document))
    except (TypeError, ValueError, OverflowError, UnicodeError, RecursionError) as error:
        return {"status": "Unavailable", "reasons": [f"export input: {error}"]}
    if result["status"] != "Complete":
        return {"status": "Unavailable", "reasons": result["reasons"]}
    manifest = document["manifest"]
    reviewers = {row["id"]: row for row in manifest["reviewers"]}
    if any(NYM.fullmatch(nym) is None for nym in reviewers):
        return {"status": "Unavailable", "reasons": ["export: reviewer ID must be canonical lowercase 64-digit Nym hex"]}
    for epoch in result["epochs"]:
        if epoch["cut"] > MAX_U64 or len(epoch["reviewers"]) > MAX_U64:
            return {"status": "Unavailable", "reasons": ["export: wire integer exceeds u64"]}
        for row in epoch["reviewers"].values():
            if row["n"] > MAX_U64 or row["opportunities"] > MAX_U64:
                return {"status": "Unavailable", "reasons": ["export: wire integer exceeds u64"]}
    try:
        eta, k0 = manifest["parameters"]["eta"], manifest["parameters"]["k0"]
        epochs = []
        cumulative_opportunities = {nym: 0 for nym in reviewers}
        rounded_cumulative = {nym: Fraction(0) for nym in reviewers}
        for epoch in result["epochs"]:
            rows = epoch["reviewers"]
            lines = ["affine-frozen-v1\t" + "\t".join((result["manifest_id"], result["snapshot_id"],
                     epoch["id"], str(epoch["cut"]), eta, k0, epoch["cap"] or "null",
                     str(len(rows))))]
            for nym in sorted(rows):
                row = rows[nym]
                weight = Fraction(row["weight"])
                cumulative_opportunities[nym] += row["opportunities"]
                opportunity_bound = Fraction(row["opportunities"], 2**53)
                cumulative_bound = Fraction(cumulative_opportunities[nym], 2**53)
                deviation_bound = Fraction(cumulative_opportunities[nym], 2**52)
                bits, rounded_weight = _bits(weight)
                rounded_epoch = row["opportunities"] * rounded_weight
                rounded_cumulative[nym] += rounded_epoch
                lines.append("row\t" + "\t".join((nym, str(int(reviewers[nym]["founder"])),
                    str(row["n"]), str(row["opportunities"]), row["score_sum"],
                    row["raw_weight"], row["weight"], bits,
                    row["contribution"], row["cumulative_utility"],
                    str(rounded_epoch), str(rounded_cumulative[nym]),
                    str(opportunity_bound), str(cumulative_bound), str(deviation_bound))))
            artifact = ("\n".join(lines) + "\n").encode("ascii")
            epochs.append({"id": epoch["id"], "cut": epoch["cut"], "artifact": artifact,
                           "sha256": hashlib.sha256(artifact).hexdigest()})
        return {"status": "Complete", "epochs": epochs, "numerical_profile": NUMERICAL_PROFILE}
    except (OverflowError, ValueError, InvalidInput) as error:
        return {"status": "Unavailable", "reasons": [f"export: {error}"]}


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if len(argv) != 2:
        print("usage: python3 sim/export_a12_affine.py INPUT.json OUTPUT_DIR", file=sys.stderr)
        return 2
    try:
        document = load_document(Path(argv[0]).read_text(encoding="utf-8"))
        result = export_epochs(document)
    except (InvalidInput, OSError, UnicodeError) as error:
        result = {"status": "Unavailable", "reasons": [str(error)]}
    if result["status"] != "Complete":
        sys.stdout.write(canonical_json(result))
        return 2
    destination = Path(argv[1])
    if os.path.lexists(destination):
        sys.stdout.write(canonical_json({"status": "Unavailable",
                                         "reasons": ["output destination already exists"]}))
        return 2
    index = []
    try:
        with tempfile.TemporaryDirectory(prefix=".a12-stage-", dir=destination.parent) as staging:
            staged = Path(staging)
            for ordinal, epoch in enumerate(result["epochs"]):
                filename = f"epoch-{ordinal}.affine.tsv"
                (staged / filename).write_bytes(epoch["artifact"])
                index.append({"file": filename, "id": epoch["id"], "cut": epoch["cut"],
                              "sha256": epoch["sha256"]})
            (staged / "index.json").write_text(canonical_json({"epochs": index}), encoding="ascii")
            if os.path.lexists(destination):
                raise FileExistsError("output destination already exists")
            staged.rename(destination)
    except (OSError, UnicodeError, ValueError) as error:
        sys.stdout.write(canonical_json({"status": "Unavailable", "reasons": [f"output: {error}"]}))
        return 2
    sys.stdout.write(canonical_json({"status": "Complete", "epochs": index}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
