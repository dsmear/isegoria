# Simulations

Research prototypes, fixture generators and differential oracles. Their role is
specific to each script: they are **not a complete executable specification of the
current engine**. The current mechanism is in [`docs/02`](../docs/02-scoring-engine.md);
open correctness questions are in [`docs/15`](../docs/15-phase1-review.md).

The bridging prototype includes D32/D42, but the BSS evaluator score and proxy-ability
DIF paths remain historical. Rust retains those functions for scoped regression and
oracle comparisons; the production path uses the difference score and
`scoring::latent::latent_dif` with formats and the protocol's admission checks.
Reproducing an old fixture does not validate the newer model.

## Requirements

```
pip install numpy scipy
```

## Files

### `a12_offline.py`

An executable **exact offline reference path** for the owner-approved A12 affine
candidate ([`docs/16` §7.2](../docs/16-a1-incentive-design.md#72-a12-restricted-affine-guarantee--owners-decision-2026-10-12)).
It consumes a prefixed manifest and a snapshot of trusted local evidence, checks their
structural/timing composition, scores reports, counts complete cumulative exposures,
applies probation/founder and affine weights, computes the homogeneous-roster median
cap, freezes epoch weights and sums assigned-opportunity utility. It does not call or
replace the Rust difference/odds/reset consumers, bridging or empirical validation.
A12 remains open for those pertinent integrations and the final candidate review.

Python standard library only, supported numerical profile below. From the repository root:

```
python3 sim/a12_offline.py sim/fixtures/a12_complete.json
python3 sim/a12_offline.py sim/fixtures/a12_incomplete.json
python3 -m unittest discover -s sim -p test_a12_offline.py -v
python3 scripts/comment_budget.py sim/a12_offline.py sim/test_a12_offline.py
```

The second command deliberately exits 2 with `Unavailable`; a complete result exits 0.
Use `-` as input path to read stdin. Output is one canonical JSON line. The callable
interfaces are `load_document(text)`, `evaluate(document)` and `evaluate_json(text)`;
`evaluate` returns a result object, whereas `load_document` raises `InvalidInput` for
invalid JSON. `canonical_json` and `content_hash` expose the provenance serialization.
The small committed fixtures use a bootstrap founder with no initial history; tests
construct complete 29/30-item histories in memory rather than storing bulky examples.
The fixture's explicit parameters are test examples, **not operating defaults**.

#### Closed version-1 JSON input schema

Every listed object has **exactly** the listed fields; no optional/unknown fields,
duplicate JSON keys or implicit defaults. `ID` is an opaque ASCII string matching
`[A-Za-z0-9][A-Za-z0-9_.:-]{0,63}`. Only evaluator pseudonyms are represented; there
are no author/respondent links or personal attributes. ID syntax does not establish
anonymous credential provenance. `N` is a nonnegative JSON integer, never a boolean.
Positions are ordered experiment positions, not wall-clock measurements. `Q` is a
canonical reduced rational **string**: `0`, an integer without leading zeroes, or
`numerator/positive-denominator`; fractions with denominator 1, redundant signs,
`-0`, decimals and unreduced fractions are rejected. Floating JSON numbers and
NaN/Infinity are rejected before interpretation. The supported integer conversion/resource
limits can reject extremely large inputs; no precision truncation is used.

| Object | Exact fields and types |
|---|---|
| Root | `manifest: Manifest`, `snapshot: Snapshot` |
| Manifest | `version: 1`, `frozen_at: N`, `assumptions: Assumptions`, `parameters: Parameters`, `reviewers: Reviewer[]`, `groups: Group[]`, `assignments: Assignment[]`, `epochs: Epoch[]`, `initial_history_hash: lowercase SHA-256`, `weight_policy: "affine_complete_v1"`, `diagnostic_policy: "none"` |
| Assumptions | `ex_ante_fixed_calendar`, `common_initial_history`, `policy_invariant_outcomes`, `policy_invariant_joint_information`, `policy_invariant_completion`, `no_report_controlled_resources`: each exactly `true` |
| Parameters | `eta: Q`, `k0: Q`; mandatory, `0 < eta <= 4/5`, `k0 >= 0` |
| Reviewer | `id: ID`, `founder: boolean`; nonempty unique roster |
| Group | `id: ID`, `rule: ID`, `reference: ID`, `verifier: ID`, `items: ID[]` |
| Assignment | `id: ID`, `reviewer: ID`, `item: ID`, `epoch: ID` or reserved `"initial"` |
| Epoch | `id: ID`, `cut: N`, `opens_at: N`, `closes_at: N`, `prefixes: {reviewer-ID: assignment-ID[]}`, `opportunities: {reviewer-ID: N}` |
| Snapshot | `observed_at: N`, `evidence_mode: "trusted_local_assertion"`, `reports: Report[]`, `terminals: Terminal[]` |
| Report | `assignment: ID`, `p: Q` in `[0,1]`, `frozen_at: N`, `available_at: N` |
| Terminal | `item: ID`, `group: ID`, `rule: ID`, `reference: ID`, `verifier: ID`, `outcome: "A"`/`"R"`/`"I"`, `terminal_at: N`, `disclosed_at: N`, `available_at: N`, `verified: true`, `evidence_id: ID` |

The input validator additionally enforces these relationships:

- IDs are unique within each record type. Every item belongs to one declared group,
  with at least two distinct items. Each assignment references its reviewer, item and
  epoch; a reviewer/item pair contributes once. Assignments for one item are its
  **entire declared cohort** and share one epoch and one terminal record. Duplicate
  terminal records cannot encode different outcomes for different reviewers. Reference,
  rule, group and verifier fields must match the manifest. Evidence IDs are unique.
  A group may contain items without reviewer assignments; no cohort is silently added.
- Assignment array order fixes each reviewer's ordered history. It starts with that
  reviewer's common initial history, then follows epoch order. For every epoch and
  every reviewer, `prefixes` must equal **all assignments in earlier epochs**, including
  all initial records, in that order. This v1 update calendar consumes complete prior
  epochs; it does not implement arbitrary subepoch prefix calendars. Both `prefixes`
  and `opportunities` must contain exactly the whole roster, including probationers.
- `opportunities[u]` equals the number of u's assignments in that epoch. It is never an
  independent multiplier or count derived from arrivals/outcomes. The epoch weight
  is calculated at `cut`, before `opens_at`, and reused for all these opportunities.
  Current reports never enter their own epoch weight. The nonempty epoch array is the
  finite horizon; `initial` cannot name a measured epoch.
- `manifest.frozen_at <= first cut`; for later epochs `previous closes_at <= cut`,
  and always `cut < opens_at <= closes_at <= snapshot.observed_at`. Every compared
  report freezes in its assigned epoch, strictly after the declared manifest freeze.
  Report freeze precedes report availability, which cannot exceed snapshot observation.
- A terminal satisfies `terminal_at <= disclosed_at <= available_at <= observed_at`.
  Every supplied report on that item must freeze **strictly before disclosure**;
  the terminal may already have formed while still hidden. For every required prefix,
  both report and terminal evidence must be available at its **original cut**. Later
  arrival or snapshot observation does not move that cut or retroactively repair it.
- All assigned reports are required so the compared policy was actually recorded.
  Terminals are required for the union of prefixes read by the horizon, including
  every initial assignment. A final assignment never read by a later prefix can lack
  its terminal: it appears in `unscored_assignments`, never as an imputed zero/I.
  Optional supplied terminal records are still validated. An unconsumed final report
  has zero coefficient in this horizon's utility, independent of whether it is scored.
- Common initial reports and terminal evidence must already be available at the
  manifest freeze. `initial_history_hash` pins their complete record objects: SHA-256
  of `{"reports": [...], "terminals": [...]}` using `canonical_json`, sorting initial
  reports by assignment and distinct initial terminals by item. Use
  `initial_history_hash(manifest, snapshot)` to construct it before compared reports.
  The manifest itself is not secretly rewritten when later evidence arrives.

`verified: true`, the reference/verifier labels and all invariance flags are **trusted
local assertions**. The program checks their presence, associations and consistency;
it does not authenticate signatures, prove actual ex ante commitment, check off-log
knowledge, infer a common filtration or run a batch fit. A declared group with two
items does not establish empirical validation quality. The declared terminal meaning
must be intrinsic A/R/I under that group's procedure, not missing work/resources.
Changing a boolean or timestamp in a fabricated input is not evidence that the theorem's
assumptions hold. The harness cannot infer policy-dependent disclosure from one transcript.

#### Numerical and refusal contract

The fixed score is `1/4 - (p-y)^2` on A/R and `0` on intrinsic terminal I. At 30
complete observed assignments, initial history included, the weight becomes
`1 + eta * score_sum / (k0+n)`. Before 30 a founder weighs 1 and another reviewer 0;
no denominator is evaluated. CUSUM is **not implemented**, and no reset or detector
side effect is adopted. Its diagnostic-only statistic/reference/threshold contract
and calibration remain to be specified ([`docs/14`](../docs/14-parameter-register.md)).

The supported numerical profile is `cpython-3.11+-int4300-v1`: CPython version 3.11
or newer, with `sys.get_int_max_str_digits() == 4300`. The executable checks this
before input decoding/evaluation; another implementation/version or configured limit
returns `Unavailable` with an **unsupported environment** reason. It does not change
process-global settings or silently disable integer limits. The profile is named in
complete outputs. This fixes `PYTHONINTMAXSTRDIGITS` variability for supported replays.
An input within that profile can still produce an exact intermediate/result exceeding
the integer-to-decimal encoding limit (for example a squared rational denominator).
That returns `Unavailable` with an **exact computation or encoding limit** reason,
atomically without partial numeric certificates. Memory/time resource exhaustion is
not an arithmetic accuracy guarantee; no large-input performance claim is made.

All computation uses `fractions.Fraction`, including an even roster's median (mean
of the two middle positive weights), the `3*median` cap and opportunity utility.
Zero-weight probationers are excluded from that median. An empty carrying roster has
`cap: null` and zero weights. Positive weights follow the same candidate/founder rule,
so the cap is slack within the declared parameter range. No existing odds weight is
accepted in the roster. Exact canonical rational strings are used for scores, sums,
weights, caps, contributions and utilities; counts remain integers. There is no
float conversion, tolerance, rounding or transcendental operation.

This is an exact executable contract on **rational inputs**, a subset of the real
mathematical domain. If the truthful `q_c` is irrational or not represented, the harness
does not certify a rounded report as maximizing utility. The finite incentive tests
use rational conditional beliefs and a prefixed common outcome/information law. They
are composed regression evidence, not a replacement for the conditional proof or a
claim about every possible policy. At `c=0` no truthful target is imputed; final
unrewarded reports demonstrate the separate `beta=0` flat case. A Rust adapter will
need its own numerical contract; these results do not certify binary floating behavior.

A complete output contains `manifest_id`, `snapshot_id`, exact per-assignment scores,
`unscored_assignments`, epoch IDs/cuts, roster counts/sums/raw and capped weights,
assigned opportunity counts, epoch contributions and cumulative utilities. Completion
certifies this calculation **conditional on the supplied trusted assertions**, not
empirical properness or a general anti-betrayal property.

Any malformed schema, unsupported evidence/policy, missing required prefix member or
invalid timing returns `Unavailable` with deterministic `reasons`, **without any
numeric weight, score, partial epoch result or cumulative-utility certificate**. In
particular it never returns a certified hold-old weight, fabricated zero or intrinsic I.
The first structural error may stop validation; completeness errors are collected
where structure permits. Validly encodable manifest/snapshot objects retain their
provenance hashes in a refusal; unparsable JSON need not have them.

Canonical serialization is ASCII JSON, sorted object keys, compact separators, plus
one trailing newline, hashed with SHA-256. Array order is retained and meaningful;
whitespace/object-key order changes do not change hashes. Output maps and reasons are
deterministically ordered. These hashes identify declared inputs, **not their author
or authenticity**; a new snapshot has a new ID even if its calculated weights coincide.
The process does not keep a mutable history that could overwrite an old certificate.

### `a12_weight_counterexample.py`

A12's accepted finite counterexample ([`docs/16` §7.1](../docs/16-a1-incentive-design.md#71-a12-finite-effective-weight-counterexample)):
exact Fraction scores, track/CUSUM transitions, full-roster cap and a rational Taylor8
certificate, with Decimal expectations. A mathematical transcription of the declared
item-level bridge, not a Rust test, bitwise `fmath` check or end-to-end protocol run.
Deterministic; Python standard library only, with no additional dependencies. From the
repository root:

```
python3 sim/a12_weight_counterexample.py
```

### `bridging_irt_dif.py`

End-to-end simulation over 10 civic questions, 200 reviewers (a 60/40 unbalanced
network), 1500 respondents with 30 anchor items. Covers:

- **Level A** — bridging with asymmetric regularization; the side-balanced score `S_j`
  (two sides by the exact 2-means of `f_u`, each at least 5% of the reviewers; the
  model's predicted approval, clipped to [0, 1], averaged per side, the two averaged
  again; `docs/02` §A.3, D32, D42) with its bootstrap-min and side gap; latent axis
  recovery
- **Level B (historical)** — IRT, point-biserial, DIF via logistic regression with
  generated group labels and anchor-total ability; not the current anonymous re-check
- **Combined verdict** of the two filters
- **Evaluators (historical)** — retired Brier Skill Score for various profiles
  (follows-the-crowd, expert, partisan…); not the current difference score
- **Corner case 1** — elite consensus (an item neutral on the political axis,
  distorted on education)
- **Corner case 2** — cost of bipartisan corruption
- **Corner case 3** — threshold sweep (the false-positive / partisan-item trade-off)

Parameters editable at the top of the file: the majority camp's share (`share_B`), the
threshold (`TAU`), the item leans.

Historical prototype results (indicative, seed-dependent; not current-system rates):
- latent axis recovered with correlation ~0.99
- the consensus items score `S_j` ≈ 0.83–0.86 and pass at τ = 0.80; the partisan items
  (0.53–0.56) and the mildly partisan one (0.70) drop; the intercept `b_j` is printed
  alongside for comparison
- the MES item passes bridging but is stopped by DIF (β₂ ≈ 0.7)
- "follows the peer average" → negative BSS; "psychometric expert" → BSS ~0.95
- elite consensus: political DIF ≈ 0, education DIF ≈ 0.66
- bipartisan corruption: ~55/80 nodes of the opposing camp are needed to pass the item
  (`S_j` 0.75 with 40, 0.83 with 55)

### `latent_dif_and_capacity.py`

Two experiments:

1. **Detecting bias without knowing which axis to look on** — historical proxy-ability IRT mixture,
   the distorting axis never observed. Shows that with ≥2 distorted items in a batch the
   model estimates delta ~1.0 on the defective ones and ~0.1 on the clean ones, and
   reconstructs the hidden axis (correlation 0.5–0.8). A single distorted item is not
   detected in this experiment; that is a finite-sample result, not a general
   non-identifiability proof. The protocol's batch requirement remains.
2. **Sustainable proposal quota** — shows the bottleneck is pilot respondents, not
   reviewers under its original sample/cost assumptions. It motivated the lottery
   (`docs/01` D10); its quota calculation is not a capacity estimate for the current
   guessing-aware model. See `docs/02` §B.6 for the cost accounting still needed.

### `export_fixtures.py`

Regenerates the historical oracle fixtures consumed by the Rust acceptance tests
(`crates/scoring/tests/fixtures/`). It mirrors the two simulations above on the same
seeds and dumps the datasets and the full-fit results as CSV. Run it as:

```
python export_fixtures.py <output_dir>
```

### `oracle_bridging.py`, `oracle_mixture.py`

Differential oracles on random datasets (`docs/08` REPRO-003, T45): the paper's
NumPy/SciPy implementations of the bridging fit and of the two-class mixture
(`paper/scripts/common.py`) on a dataset the Rust tests write, seeded multi-starts, the
lowest objective kept. `crates/scoring/tests/differential_oracle.rs` runs them and
compares; it self-skips without the pinned environment (`requirements.txt`).

```
python oracle_bridging.py <dir>    # reads R.csv, mask.csv, weights.csv; writes oracle.csv
python oracle_mixture.py <dir>     # reads theta.csv, X.csv; writes oracle.csv
```

### `latent_shape_dimension.py`

A3's calculations (`docs/18`): the latent ability histogram's map from logits to a
standardized distribution, as `scoring::latent` computes it — the review's 3-node example,
the map's rank (`Q − 1`), and how much the grid's mean and variance move the response
patterns of eight 2PL items at a smooth and at a rough histogram. numpy only; deterministic.

```
python latent_shape_dimension.py
```

### `dtf_composition.py`

A2's calculations (`docs/19`): the DTF figures of the dossier by continuous quadrature beside
the 41-node grid `scoring::dtf` uses, and the three-class example where a group's gap exceeds
the per-pair maximum, in exact rationals. numpy and scipy; deterministic.

```
python dtf_composition.py
```

## What these prototypes demonstrate

Motivating scenarios documented in `docs/06-threat-model.md`, within these prototypes:
- the true-but-divisive false negative (no threshold saves a polarizing fact) —
  motivates the appeal channel
- the elite-consensus blind spot — motivates multi-axis DIF
- the fixture's ~30% survival rate — an example, not an expected production yield
- the threshold as a blade — motivates the uncertainty band
