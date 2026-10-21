# Archive — Phase 1, forecast-pilot design

| | |
|---|---|
| **What** | Material that belongs exclusively to the previous Phase 1 design, in which reviewers forecast whether an item passes the pilot and are scored on that forecast (A1, A10's baseline, A12, S2), and to the work on A2's whole-test requirement under that design. |
| **Why** | [`docs/01` D47](../../docs/01-decisions.md) (owner, 2026-10-21): the author states the key and reviewers answer the item; the pilot forecast is no longer their main contribution. |
| **Baseline** | Branch `docs/phase1-review-alignment` at `38780af`. Moved in `c158670`, renames only; in the commit that added this index, link targets were adjusted for the new location and one test path was updated (below). |
| **Status** | Historical record, not the active specification. Approvals, closures and results keep their perimeters. A1's and A12's conditional closures concern the forecast score and its weights and certify nothing of D47's mechanism. **A2 is not resolved by this archiving**: its disposition is in [`docs/15`](../../docs/15-phase1-review.md) A2 and in D47. |
| **Inventory** | [`docs/21`](../../docs/21-d47-inventory.md) classifies what D47 affects: kept, to adapt, archived (with motivation and dependencies), still needed as dependencies. |

## Layout

The archive mirrors the repository's paths: a file once at `P` is now at
`archive/phase1-forecast-pilot/P`. Relative links and imports between archived items therefore
resolve unchanged. Links from archived Markdown to material that stayed outside were retargeted;
no other text changed. Code comments that point to `docs/16`, `docs/17`, `docs/19` or `docs/20`
resolve through short redirect pages left at those paths.

## Index

### Dossiers — [`docs/`](docs/)

| File | Content | State when archived |
|---|---|---|
| [`16-a1-incentive-design.md`](docs/16-a1-incentive-design.md) | A1: the exploration draw's information sequence, the adaptive counterexample, the IPW lemma, the beacon; the reputation actually used; A12's counterexample (§7.1), restricted affine guarantee (§7.2) and closure review (§7.3) | A1 closed conditionally on `1bd865c` (via `17` §14.15); A12 closed conditionally on `15b846c` |
| [`17-a1-pilot-batch-design.md`](docs/17-a1-pilot-batch-design.md) | A1: pilot batches (C3), missing outcomes (C2), the ternary candidate, B-b's contract and scorer, cases 5–7, timing, access and disclosure, observable evidence, A1's candidate guarantee (§14), O1–O3 and the conditional closure (§14.15) | Approvals as recorded in its header; A11, found in §2, has its record in `docs/15` |
| [`19-a2-dtf-composition-design.md`](docs/19-a2-dtf-composition-design.md) | A2: the diagnosis and R1, the sufficient proposition, common against per-form calibration, the candidate contract (§11), the coverage proof (§12), the review and the studies (§13.1–§13.31), the balance (§13.32), the conditional target and majorant (§13.33) | A2 open; certifier and experiments suspended |
| [`20-s2-synthetic-check.md`](docs/20-s2-synthetic-check.md) | S2: the synthetic-event check's specification, its implementation, the absent baseline's correction, A1's adaptive verification, the check through B-b's scorer | Approved within the check's perimeter; its test code stays in `crates/protocol/tests/` |
| [`phase1-handoff.md`](docs/phase1-handoff.md) | The working note that handed Phase 1 work between sessions up to `38780af` | Not normative; replaced by a new `docs/phase1-handoff.md` |

### Simulations — [`sim/`](sim/README.md)

The A12 offline reference (`a12_offline.py`, `export_a12_affine.py`, `a12_weight_counterexample.py`,
their tests and `fixtures/a12_*.json`) and A2's tools (`dtf_composition.py`,
`a2_information_check.py` with its committed result, `a2_probe.py` with `a2_probe_reference.json`,
`a2_tradeoff.py`, `a2_size.py`, their tests). Their documentation moved verbatim from
`sim/README.md` into [`sim/README.md`](sim/README.md).

### Results — [`verification/reports/`](verification/reports/)

| Directory | Study (`19`) | Run on |
|---|---|---|
| [`a2-probe-9511ce1`](verification/reports/a2-probe-9511ce1/README.md) | first bounded probe, §§13.11–13.12 | `9511ce1` |
| [`a2-candidates-2e2572c`](verification/reports/a2-candidates-2e2572c/README.md) | candidates' diagnostics, §§13.14–13.15 | `2e2572c` |
| [`a2-dtf-check-821f52d`](verification/reports/a2-dtf-check-821f52d/README.md) | DTF check on independent seeds, §§13.18–13.19 | `821f52d` |
| [`a2-ind-cal-7bbfddb`](verification/reports/a2-ind-cal-7bbfddb/README.md) | single-test study, calibration, §13.22 | `7bbfddb` |
| [`a2-ind-val-185ec93`](verification/reports/a2-ind-val-185ec93/README.md) | single-test study, validation, §13.23 | `185ec93` |
| [`a2-tradeoff-ccb8f05`](verification/reports/a2-tradeoff-ccb8f05/README.md) | trade-off reading, post hoc, §13.24 | `ccb8f05` |
| [`a2-size-b8c9dce`](verification/reports/a2-size-b8c9dce/README.md) | sample-size study, §13.27 | `b8c9dce` |
| [`a2-ind-cal6-2a01b81`](verification/reports/a2-ind-cal6-2a01b81/README.md) | study at `N = 6,000`, calibration, §13.30 | `2a01b81` |
| [`a2-ind-val6-6d57cac`](verification/reports/a2-ind-val6-6d57cac/README.md) | study at `N = 6,000`, validation, §13.31 | `6d57cac` |

The information check's result (§13.8, run on `2aa136d`) is
[`sim/a2_information_check.json`](sim/a2_information_check.json).

## Path map

| Original path (at `38780af`) | Archived path |
|---|---|
| `docs/16-a1-incentive-design.md` | `archive/phase1-forecast-pilot/docs/16-a1-incentive-design.md` |
| `docs/17-a1-pilot-batch-design.md` | `archive/phase1-forecast-pilot/docs/17-a1-pilot-batch-design.md` |
| `docs/19-a2-dtf-composition-design.md` | `archive/phase1-forecast-pilot/docs/19-a2-dtf-composition-design.md` |
| `docs/20-s2-synthetic-check.md` | `archive/phase1-forecast-pilot/docs/20-s2-synthetic-check.md` |
| `docs/phase1-handoff.md` | `archive/phase1-forecast-pilot/docs/phase1-handoff.md` |
| `sim/a12_offline.py` | `archive/phase1-forecast-pilot/sim/a12_offline.py` |
| `sim/a12_weight_counterexample.py` | `archive/phase1-forecast-pilot/sim/a12_weight_counterexample.py` |
| `sim/a2_information_check.json` | `archive/phase1-forecast-pilot/sim/a2_information_check.json` |
| `sim/a2_information_check.py` | `archive/phase1-forecast-pilot/sim/a2_information_check.py` |
| `sim/a2_probe.py` | `archive/phase1-forecast-pilot/sim/a2_probe.py` |
| `sim/a2_probe_reference.json` | `archive/phase1-forecast-pilot/sim/a2_probe_reference.json` |
| `sim/a2_size.py` | `archive/phase1-forecast-pilot/sim/a2_size.py` |
| `sim/a2_tradeoff.py` | `archive/phase1-forecast-pilot/sim/a2_tradeoff.py` |
| `sim/dtf_composition.py` | `archive/phase1-forecast-pilot/sim/dtf_composition.py` |
| `sim/export_a12_affine.py` | `archive/phase1-forecast-pilot/sim/export_a12_affine.py` |
| `sim/fixtures/a12_affine_nyms.json` | `archive/phase1-forecast-pilot/sim/fixtures/a12_affine_nyms.json` |
| `sim/fixtures/a12_complete.json` | `archive/phase1-forecast-pilot/sim/fixtures/a12_complete.json` |
| `sim/fixtures/a12_incomplete.json` | `archive/phase1-forecast-pilot/sim/fixtures/a12_incomplete.json` |
| `sim/test_a12_offline.py` | `archive/phase1-forecast-pilot/sim/test_a12_offline.py` |
| `sim/test_a2_information_check.py` | `archive/phase1-forecast-pilot/sim/test_a2_information_check.py` |
| `sim/test_a2_probe.py` | `archive/phase1-forecast-pilot/sim/test_a2_probe.py` |
| `sim/test_a2_size.py` | `archive/phase1-forecast-pilot/sim/test_a2_size.py` |
| `sim/test_a2_tradeoff.py` | `archive/phase1-forecast-pilot/sim/test_a2_tradeoff.py` |
| `sim/test_export_a12_affine.py` | `archive/phase1-forecast-pilot/sim/test_export_a12_affine.py` |
| `verification/reports/a2-candidates-2e2572c/` | `archive/phase1-forecast-pilot/verification/reports/a2-candidates-2e2572c/` |
| `verification/reports/a2-dtf-check-821f52d/` | `archive/phase1-forecast-pilot/verification/reports/a2-dtf-check-821f52d/` |
| `verification/reports/a2-ind-cal6-2a01b81/` | `archive/phase1-forecast-pilot/verification/reports/a2-ind-cal6-2a01b81/` |
| `verification/reports/a2-ind-cal-7bbfddb/` | `archive/phase1-forecast-pilot/verification/reports/a2-ind-cal-7bbfddb/` |
| `verification/reports/a2-ind-val-185ec93/` | `archive/phase1-forecast-pilot/verification/reports/a2-ind-val-185ec93/` |
| `verification/reports/a2-ind-val6-6d57cac/` | `archive/phase1-forecast-pilot/verification/reports/a2-ind-val6-6d57cac/` |
| `verification/reports/a2-probe-9511ce1/` | `archive/phase1-forecast-pilot/verification/reports/a2-probe-9511ce1/` |
| `verification/reports/a2-size-b8c9dce/` | `archive/phase1-forecast-pilot/verification/reports/a2-size-b8c9dce/` |
| `verification/reports/a2-tradeoff-ccb8f05/` | `archive/phase1-forecast-pilot/verification/reports/a2-tradeoff-ccb8f05/` |

`sim/README.md` stayed in place; its archived sections are the new
[`sim/README.md`](sim/README.md) here.

## Reproducing the historical results

- Each report names its candidate commit, machine, commands and exit codes. Its commands are
  written from the repository root with the original paths and run as written on a checkout of
  that commit. No result was regenerated for this move, and no data file was altered: digests,
  plans and environment records still name the original paths.
- On the current tree, translate a path with the map above. The run directories the reports copy
  from (`characterization-results/…`) are the owner's, untracked and outside the archive.
- The archived scripts' tests: `python3 -m unittest discover -s archive/phase1-forecast-pilot/sim
  -p 'test_*.py'`. The exporter's test reads the protocol crate's golden tables in
  `crates/protocol/tests/fixtures/a12_affine/`, which stay with the Rust test that consumes them;
  its path to them is the one line of code changed by the move
  (`archive/phase1-forecast-pilot/sim/test_export_a12_affine.py`, `GOLDEN`).

## Kept outside the archive

Material tied to the previous design stays in place where current code, tests or CI need it, or
where a document is mixed; each case is listed with its reason in
[`docs/21`](../../docs/21-d47-inventory.md) §§2 and 4. In short: the Rust code of the forecast path
and its tests (current behaviour, to adapt); the S2 and A1 test code, which exercises current
code; `crates/protocol/tests/fixtures/a12_affine/`; `scoring::latent::latent_dif_candidates`; the
working paper, whose `scripts/common.py` feeds CI's differential oracles; the redirect pages
`docs/16`, `docs/17`, `docs/19` and `docs/20`.

**The A2 harness stays in `crates/characterization`** (`a2-probe`, `probe*.rs` and their tests), a
residual separation: it compiles apart, but `a2-probe run` records its candidate from
`HEAD:crates` and refuses uncommitted changes under `crates/`, so outside `crates/` its provenance
guard would no longer cover its own code. Its commands are unchanged
(`cargo run --release -p characterization --bin a2-probe -- …`); only its oracle and readings are
here.
