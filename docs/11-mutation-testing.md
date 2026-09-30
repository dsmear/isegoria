# Isegoria — Mutation testing report (T41)

| | |
|---|---|
| **Purpose** | Measure how much of the code the tests actually *verify*, not just execute, and record every mutant that survives with the reason it is acceptable. |
| **Tool** | `cargo-mutants` 26.0.0 (the newest release that builds on the pinned rustc 1.86). |
| **Date** | 2026-09-24, branch `test/t41-mutation-survivors`. |
| **Status** | Every surviving mutant is either killed or justified below (runs 1–3 for T41, run 4 for the T48 optimizer, run 5 for the T40 detector, run 6 for the T55 contested-facts pool, run 7 for its follow-up, run 8 for the T63 consortium, run 9 for the T37 beacon, run 10 for the T72 candidate order, run 11 for the T13 store, runs 12–14 for T73's three steps, run 15 for T18, runs 16–17 for T74's two steps, run 18 for T25's first step, run 19 for its third step's harness, run 20 for T82). Not covered yet: the decision logic Phase 1 changed after run 5 — T49–T62, T71 and T39 — apart from T55's (runs 6–7): `docs/10` T81. |

## Why this was needed

Line coverage was already ~97% (`cargo llvm-cov`), yet the second review (`docs/10`
P1.5) found defects in files covered at 96–100%. Coverage says a line *ran*; a mutant
survives when the line can be changed — a `<` into `<=`, a `+` into `-`, a function
body into `return 0` — and no test notices. The survivors are a map of the checks the
suite does not make.

## How to run

```sh
cargo install cargo-mutants --version 26.0.0 --locked
# whole workspace
cargo mutants --workspace -j 4
# one file, or selected mutants by name
cargo mutants -p scoring -f crates/scoring/src/optim.rs
cargo mutants -p scoring --re 'optim.rs:97:'
# only the lines a branch changed
git diff master > /tmp/branch.diff && cargo mutants --workspace --in-diff /tmp/branch.diff -j 4
```

`.cargo/mutants.toml` is read automatically. It builds every mutant under the optimized
`mutants` profile (`Cargo.toml`), which runs the `scoring` suite in ~10 s instead of ~47 s
with identical bits (the golden test passes under it); it enables the `calibration`
feature; and it sets a 60 s minimum test timeout, because the automatic timeout is scaled
from a baseline that runs alone and parallel jobs otherwise produced false timeouts.
Each mutant runs the tests of the crate that contains it. It is too slow for every push:
run it after changing decision logic (`--in-diff` for a branch), and before a release.
Output goes to `mutants.out/` (ignored by git).

A diff whose changed `src/` lines are all in `network` or `identity` fails the baseline under
the config (`the package 'network' does not contain this feature: calibration`): run it with
`--no-config --profile mutants --timeout-multiplier 3 --minimum-test-timeout 60` instead.

## Results

| Run | Scope | Mutants | Caught | Timeout¹ | Unviable² | Missed |
|---|---|---|---|---|---|---|
| 1 | whole workspace | 1482 | 876 | 5 | 381 | **220** |
| 2 | the 17 files that had survivors, after the new tests | 1088 | 1008 | 3 | 37 | **40** |
| 3 | the 9 real survivors of run 2, after their tests | 9 | 9 | — | — | **0** |

¹ The mutant made a loop never terminate (union-find `find`, the mixture optimizer):
counted as caught. ² The mutant does not compile.

By crate, run 1: `network` and `identity` had **no** survivors; `protocol` 30;
`scoring` 188 (bridging 60, DIF 62, optimizer 23, reputation 19, …). After run 3 the
32 survivors left are all **equivalent** mutants (next section).

### Defects the survivors exposed

The work was not only adding assertions. Two survivors pointed at real problems:

1. **The bridging oracle tolerance hid fit errors.** With the gradient's data or
   regularization term corrupted, the fit still reported `Converged` and moved `b_j` by
   up to 0.009 — inside the 0.03 tolerance against SciPy, but wider than the gate's
   uncertainty band (`ε = 0.008`), so it could flip an item across `τ`. The gradient is
   now pinned against central differences, and the oracle tolerance is 0.004 (Rust and
   SciPy agree to ~0.002).
2. **A failed line search was reported as `Converged`** (`optim::lbfgs`). Two paths:
   the step halved 60 times barely moved `f` and the relative-progress stall test ran
   before the failure check; and once `x + step·d` rounded back to `x`, Armijo passed
   with `f_new == f` and no movement. Both are now `LineSearchFailed`, and a failed
   trial point that raised the cost is not taken (`docs/08` OPT-001).

Also recorded (not fixed here): the Armijo-only line search needs ~670 gradients on
Rosenbrock where SciPy needs ~46 (T45); Mantel–Haenszel with more strata than
respondents makes α infinite and classes the item C (calibration-only).

### What was added

| Area | Tests |
|---|---|
| Bridging | gradient vs central differences; objective by hand; oracle tolerance 0.004 and `Converged`; bootstrap must lower scores; empty input |
| All fits | `golden.rs` + `fixtures/golden_bits.txt`: every output of `fit`, `bridge_scores`, `mixture_dif` pinned bit-for-bit (kills initialization, RNG-order and optimizer-path mutants). Regenerate on an intended change with `ISEGORIA_UPDATE_GOLDEN=1 cargo test -p scoring --test golden` |
| Optimizer | condition-10⁴ quadratic within a SciPy-like budget; Rosenbrock to 1e-6; immediate stop at a stationary point; exact 1-D convergence; Armijo rejects a non-decreasing step; uphill gradient → `LineSearchFailed`; a failed search keeps the better point |
| Level B | hand-computed Mantel–Haenszel (one stratum, group swap, two strata, unequal strata, empty strata, no discordant cells); purification iterations and θ; 2PL `b`; point-biserial by hand; `sigmoid`/`softplus` at ±800 |
| Level C, anti-collusion | `hand_computed.rs`: author decay, weighted crowd baseline, zero weight, even/odd median, Pearson by hand, constant rows, zero-weight cluster |
| Protocol | `exact_outcomes.rs`: seats per stratum, sortition deficit fill, largest-remainder apportionment, zero shares, coverage deviation, exactly-enough items, honeypots inserted, lottery epoch mixing, `K_MIN` batch, `ExposureLimit`, accessors, one reviewer per stratum |

## Surviving mutants — all equivalent

A mutant is *equivalent* when no valid input can tell it from the original. Line
numbers are those of this branch.

| Mutant(s) | Why it is equivalent |
|---|---|
| `dif.rs:264` and `glm.rs:37` `softplus`: `>` → `>=` | At `z = 0` both branches return `ln 2` exactly. |
| `dif.rs:171`, `dif.rs:238` `d[j] * z` → `d[j] / z` | `z ∈ {−1, +1}`, so `d·z = d/z`. |
| `dif.rs:102`, `dif.rs:103` (Mantel–Haenszel) `item > 0.5`, `group > 0.0` → `>=` | Items are 0/1 and groups ±1: neither threshold value occurs. |
| `dif.rs:111` `ns > 0.0` → `>=` | An empty stratum only exists when there are more strata than respondents; then every stratum holds at most one person, which carries no discordant pair, and α is ∞ either way. |
| `dif.rs:119`, `dif.rs:121` ETS class boundaries `<` → `<=` | `Δ = −2.35 ln α` with α a ratio of counts never equals 1.0 or 1.5 exactly. |
| `validation.rs:36`, `revalidation.rs:47` `|β₂| > BETA2_MAX` → `>=`; `glm.rs:76` `max_z > SEPARATION_LOGIT` → `>=` | Equality of a fitted continuous statistic with the threshold has probability zero. |
| `bridging.rs:314` `rng < keep_frac` → `<=` (was 272) | A uniform `f64` draw equal to 0.85 exactly has probability ~2⁻⁵³ per draw. |
| `bridging.rs:328` `bj < b` → `<=` (was 286); `bridging.rs:156` `obj < best` → `<=` in the multi-start | Replacing a minimum by an equal value changes nothing. |
| `optim.rs:110` `sy > 1e-12` → `>=` | Floating equality at a continuous boundary. |
| `bridging.rs:172`, `bridging.rs:174` (`canonical_sign`) `>` → `>=`, `<` → `<=` | A tie between two `|f_j|` or a leading `f_j` of exactly 0 (then every `f` is 0 and negating gives `−0.0`, equal as a number). |
| `optim.rs:193` `i > 0` → `>=` (expansion) | At `i = 0` a value not below the start already fails sufficient decrease. |
| `optim.rs:218`, `optim.rs:219` the bracket-width stop (`hi − lo` → `hi + lo`; `1e-16 ·` → `/`) | The width stop is reached only by a failing search, where `lo` is still the start (`a = 0`): then `hi + lo = hi − lo` and `max(1, 0) = 1`. |
| `optim.rs:230` `dg · (hi − lo)` → `dg / (hi − lo)` | Product and quotient have the same sign. |
| `optim.rs:247` `\|\|` → `&&`, `optim.rs:252` `disc < 0` → `<=`, `==` | Any non-finite bracket end or negative discriminant makes the cubic minimizer NaN, which the final `is_finite` check sends to bisection anyway (a zero discriminant is a measure-zero double root). |
| `collusion.rs:32` `skip(i + 1)` → `skip(i * 1)` | Adds the diagonal pair `(i, i)`; `union(i, i)` is a no-op. |
| `collusion.rs:64` `s > 0.0` → `>=` | At `s = 0` the product is `0 · min(NaN, 1) = 0 · 1 = 0` (`f64::min` ignores NaN): same result. |
| `collusion.rs:81` (×2) `(a[i] − ma)` or `(b[i] − mb)` → `+` in the covariance term | Centring one factor is enough: `Σ(a + ma)(b − mb) = Σ(a − ma)(b − mb) + 2ma·Σ(b − mb)` and `Σ(b − mb) = 0` (same for the other factor). The variance terms, which are not equivalent, are pinned by `hand_computed.rs`. |
| `governance.rs:74` `.max(lo + 1)` → `.max(lo * 1)`; `review.rs:56` same | The guard only matters for an empty stratum, and `strata ≤ seats ≤ n` (resp. `k ≤ n`) rules that out. |
| `governance.rs:85` (`:76` since T72) `count < seats` → `<=` | At equality the fill loop breaks before changing anything. |
| `review.rs:44` `n == 0 \|\| k == 0` → `&&` | Either zero makes `k = min(k, n) = 0`, and the loop draws nothing. |
| `optim.rs:60` `yy > 0` → `>=` (was line 61) | `yy = 0` means `y = 0`, so `sᵀy = 0` and the pair was never stored (`sᵀy > 1e-12`). |
| `optim.rs:85` `gd >= 0` → `<` (second check, after the steepest-descent fallback; was line 86) | The fallback direction is `−g`, whose slope `−‖g‖²` is negative unless `g = 0`, which the gradient test has already stopped on. The fallback itself is unreachable while stored pairs keep the Hessian estimate positive definite. |

## Run 4 — the T48 optimizer and multi-start

T48 replaced the Armijo line search with a strong-Wolfe one and made the bridging fit a
multi-start, so `optim.rs` and `bridging.rs` were re-run: 464 mutants, 28 survivors.
Three were real and are killed: a fit with every weight 0 produced NaN through a 0/0
start value; the sufficient-decrease sign (a flat point that *raises* the cost must be
rejected); and the bisection fallback (a cost that is infinite past a wall). The
bracketing branches of `zoom` were not exercised at all by the reference problems (with
`c₂ = 0.9` the first interpolated point is almost always accepted), so
`optim::tests::trajectories_on_reference_problems_are_pinned` pins the exact number of
evaluations and the final point, bit for bit, on problems built to reach them — a
cliff past the minimum, a steep wall, a failing search — together with Rosenbrock and an
ill-conditioned quadratic. The survivors that remain are in the table above, plus four
**path-only** mutants:

| Mutant(s) | Why it is accepted |
|---|---|
| `optim.rs:193` `i > 0` → `<`, `==` | Drops the "value rose during expansion" test (Nocedal & Wright 3.5). The search then brackets on the slope sign or accepts a Wolfe point further out: a different trajectory to a valid step, and no constructed problem made it differ. |
| `optim.rs:230` `hi − lo` → `hi + lo` (×2: `+`, `/`) | Differs only once the bracket is reversed (`hi < lo`) *and* an interior point is too steep — not reached by any reference problem. |
| `optim.rs:258`, `optim.rs:259` (×2) the interpolation margins | Matter only when the cubic minimizer falls outside the bracket, which cannot happen while the slope changes sign inside it. |

## Run 5 — the T40 latent-class detector

With the faster configuration, `cargo mutants --workspace --in-diff` on the T40 branch:
208 mutants in 10 minutes, 8 survivors. Four were real and are killed: the free-parameter
count behind the BIC (`dif::tests::free_parameters_are_counted_as_specified`); the seed,
which the seed-robustness test could not see being dropped precisely because the verdict
does not depend on it (`latent_classes.rs::the_seed_changes_the_starts_not_the_verdict`);
the staged-search rule (`::larger_mixtures_are_tried_only_while_the_bic_improves`); and
the gap over three or more classes, which needed data where the BIC really selects three
(`::three_well_separated_classes_are_selected_as_three`). The other four are equivalent:

| Mutant(s) | Why it is equivalent |
|---|---|
| `dif.rs:283` `lo > 0` → `>=` | At `lo = 0` both branches give `softplus = ln 2`, `σ = ½`. |
| `dif.rs:410` `fit.0 < f` → `<=`; `dif.rs:420` `b < best` → `<=` | Exact ties of two fitted likelihoods or BICs. |

## Run 6 — the T55 contested-facts pool

`cargo mutants --in-diff` on the T55 diff, file by file with the suites that exercise
each (the `mutants` profile, `calibration` on, `--cargo-test-arg=--test=…`):
`scoring/src/dtf.rs` with `dtf.rs` and `golden.rs`; `protocol/src/revalidation.rs` with
`latent_revalidation.rs`, `anchor_reliability.rs`, `inv8_batch_min.rs` and
`proto013_respondent_gate.rs`; every other changed line of `protocol` with
`contested_facts.rs` (the fitted scenario skipped), the lifecycle and orchestrator suites
and their reference models, `appeal_stake.rs`, `exploration.rs` and `exact_outcomes.rs`.
115 mutants: 101 caught, 9 unviable, 5 missed. Two were real and are killed: `is_empty`
and `contains` of `ContestedPool` were only ever asserted true (the malformed-records test
now checks both answers). One was equivalent and is gone: `dtf.rs` iterated the pairs of
classes as `h in g + 1..`, and `g * 1` only added the pair `(g, g)`, whose DTF is 0; the
loop is now `h in 0..g` — the same unordered pairs, the same bits (the golden rows did not
move) — and the re-run of `dtf.rs` has no survivor (30 mutants, 29 caught, 1 unviable).
The other two are equivalent:

| Mutant(s) | Why it is equivalent |
|---|---|
| `contested.rs:207` `members.len() < n` → `<=` in `candidates` | Also enumerates subsets of `n + 1` facts, which neither the table nor the draw ever reads: both take only subsets of at most `left ≤ n` members. |
| `contested.rs:208` `k + 1` → `k * 1` in `candidates` | Also enumerates sequences that repeat a member; `ClassCurves::dtf` refuses a repeated index, so they are dropped, and the valid subsets come out in the same order. |

## Run 7 — the T55 follow-up: the pool's canonical order

`cargo mutants --in-diff` on the follow-up's diff of `protocol/src/contested.rs` (the
`mutants` profile, `calibration` on, `--cargo-test-arg=--test=contested_facts`, the fitted
scenario skipped): 4 mutants — `record` to `Ok(())`, `remove` to `true` and to `false`,
`canonicalize` to `()` — 4 caught, none missed, none unviable. The last is the one the
follow-up adds, killed by the history test of `contested_facts.rs` (`docs/08` AT-PRO-08):
without the canonical order the same fits recorded last to first draw another test on 46
of 50 seeds (another set on 43).

## Run 8 — T63: the consortium's configuration and member-set check

`cargo mutants --in-diff` on the T63 diff (`network/src/consortium.rs`; `--no-config` with
the config's profile and timeouts, since `network` has no `calibration` feature): 5 mutants
— `member_set_hash` to `[0; 32]` and to `[1; 32]`, `verify` to `true` and to `false`, and
the new member-set comparison `!=` → `==` — 5 caught, none missed, none unviable. The two
checks of `Consortium::new` sit inside `assert!`, whose arguments cargo-mutants does not
mutate; `consortium_config.rs` pins them instead: every bound (`t = 0`, `t = n + 1`, no
members) and a duplicate apart from its twin panic, and every `t` in `1..=n` is accepted.

## Run 9 — T37: the commit-reveal beacon and the lottery's canonical order

`cargo mutants --in-diff` on the T37 diff, in two halves. `network` (`beacon.rs`, the
`consortium.rs` accessors and `verify_excluding`; `--no-config` as in run 8): 53 mutants —
42 caught, 8 unviable (a `Default` for a type that has none: `BeaconCommit`, `Signature`,
`RoundId`, `Cid`, `BeaconRound`, `BeaconOutcome`), 3 missed. The three were real and are killed:
`indices` — the encoding of who revealed and who withheld in the outcome's record — could
return anything, because every test that compared two records also compared two different
beacon values; `beacon_round.rs::at_net_10_the_record_binds_who_revealed_and_who_withheld`
compares five outcomes without a beacon that differ only in those lists (re-run: 3 caught).
`protocol` (`lottery.rs`, `randomness.rs`, and the renamed `lifecycle`/`orchestrator` lines;
the config's profile and `calibration`, `--cargo-test-arg=--test=…` with `inv10_beacon_seed`,
`exploration`, `lifecycle`, `properties`, `exact_outcomes`, `orchestrator`,
`lifecycle_model`, `orchestrator_model` and `panel_diversification`): 13 mutants — 9 caught,
4 unviable, none missed. cargo-mutants deletes no statement, so the `sort_unstable` and
`dedup` that make the lottery a function of the set are not mutated; the lottery test of
`inv10_beacon_seed.rs` pins them (every order and a repeat draw the same vector).

## Run 10 — T72: the draws' canonical candidate order

`cargo mutants --in-diff` on the T72 diff (`review.rs`, `governance.rs`; the config's
profile and `calibration`, `--cargo-test-arg=--test=…` with `draw_order`, `exact_outcomes`,
`properties`, `lifecycle`, `panel_diversification`, `inv10_beacon_seed` and
`reviewer_floor`): 12 mutants — 7 caught, 4 unviable, 1 missed. The survivor is
`governance.rs:76` `count < seats` → `<=`, the equivalent mutant of the table above
(`:85` before T72): at equality the fill loop stops before changing anything.

## Run 11 — T13: the durable log and object store

`cargo mutants --in-diff` on the T13 diff (`network/src/store.rs`, `log.rs`; `--no-config`
as in run 8): 92 mutants — 75 caught, 7 unviable, 10 missed, all taken up. Four were real
test gaps and are killed: `put` at exactly `MAX_OBJECT` bytes (`>` → `>=`), the offset of an
object read right after its `put` without a reopen (`self.end + 8`, two mutants), and
`is_empty` after a `put`. Five went with the code: the torn-tail cut of both files was
guarded by `torn > 0` (four mutants, one of them real: with `==` an object file kept its torn
tail on disk, which the test now checks), and the directory sync by a `created` flag; the
cut now always runs (a no-op without a tear) and the directory is synced whenever a header
is written. The tenth is the one left below. Re-run: 85 mutants, 77 caught, 7 unviable, 1 missed — `sync_dir` → `()`, whose
effect shows only on power loss, which no test can produce.

## Run 12 — T73, first step: the encoding, the proof's wire format, the node's replay

`cargo mutants --in-diff` on the step's diff, in two halves. `network` and `identity`
(`codec.rs`, `log.rs`'s derives, `nullifier.rs`'s `encode`/`decode`; `--no-config`):
33 mutants — 31 caught, 1 unviable, 1 missed. The survivor was real: `&&` → `||` in
`NullifierProof::decode` accepted a non-canonical encoding again, and only the `protocol`
suite held the case fuzzing had found; `identity/tests/proof_encoding.rs` now holds it too
(re-run: caught). `protocol` (`events.rs`, `node.rs`, the derives in `admission.rs`; the
config's profile and `calibration`, `--cargo-test-arg=--test=…` with `node_replay`,
`proto007_deposit_replay`, `proto013_respondent_gate`, `inv9_nym_proof` and
`id008_proposal_quota`): 25 mutants — 19 caught, 6 unviable, none missed. The node's
poisoned state after a failed write has no test: it needs an I/O error after a successful
open, which the tests cannot provoke.

## Run 13 — T73, second step: item lifecycles as events

`cargo mutants --in-diff` on the step's diff (`protocol/src/events.rs`, `node.rs`; the
config's profile and `calibration`, `--cargo-test-arg=--test=…` with `lifecycle_replay` and
`node_replay`): 52 mutants — 44 caught, 8 unviable, none missed. The walks use every
lifecycle event and the round-trip test pins each event's number and the refusal of every
out-of-range byte, so the encoder's and decoder's arms leave no survivor.

## Run 14 — T73, third step: epoch results as events

`cargo mutants --in-diff` on the step's diff (`protocol/src/results.rs`, `events.rs`,
`node.rs`, `exposure.rs`, `appeal.rs`, `contested.rs`; the config's profile and
`calibration`, `--cargo-test-arg=--test=…` with `results_replay`, `lifecycle_replay`,
`node_replay`, `contested_facts`, `appeal_stake`, `exploration` and `lifecycle`): 94
mutants — 85 caught, 8 unviable, 1 missed. The survivor deleted the decoder's arm for a
failed appeal's settlement (`promoted` 0): the round trip encoded only a promotion, and the
failed settlement was checked only for its refused flag. Killed: that settlement now also
decodes and re-encodes to the same bytes.

## Run 15 — T18: replication and the libp2p transport

`cargo mutants --no-config` (the `mutants` profile, `--timeout-multiplier 3
--minimum-test-timeout 60`, `-j 2`) on T18's diff, one run per crate. A first run with
three jobs filled the disk after 15 mutants; its three survivors — `WriterSet::contains`
and `network_id`, used only by `p2p`'s tests, which a mutant in `network` does not run —
were killed by checks in `replication.rs`.

- `network/src/replica.rs` (`--cargo-test-arg=--test=replication`): 122 mutants — 98
  caught, 22 unviable, 2 missed: `Replica::is_empty` → `true`/`false`. Killed; the re-run
  gives 100 caught, 22 unviable, none missed.
- `p2p/src/lib.rs`: 57 mutants — 9 caught, 23 timeouts, 15 unviable, 10 missed. A timeout
  is a mutant that stops sync: the tests wait for convergence until their deadline. The
  survivors: the sync message limit, never reached (`read_limited` now takes the limit and
  is tested at 5 bytes); the topic name (`topic()`, pinned to `docs/04`); `peers`; and
  three paths the periodic pull masked — the pull on connect, a relay's announcement of
  entries it pulled, and the handle's `Drop`. With the deadline cut to 15 s and tests that
  run with no periodic pull, the re-run gives 36 caught, 4 timeouts, 15 unviable, 2
  missed. `Drop` was dead code — dropping the handle closes the command channel, which
  already ends the node's task — and is removed; the announcement of pulled entries is
  killed by `at_net_14_pulled_entries_are_announced_on` (checked by hand: without the
  announcement the third node never converges).

## Run 16 — T74, first step: cuts, the ledger, the replica on disk

`cargo mutants --no-config` (the `mutants` profile, `--timeout-multiplier 3
--minimum-test-timeout 60`, `-j 2`) on the step's diff, one run per crate:

- `network/src` (`--cargo-test-arg=--test=cuts`, `--test=replication`): 59 mutants — 43
  caught, 12 unviable, 4 missed: `Consortium::is_member` (→ `true`, → `false`, `==` → `!=`),
  checked only by `protocol`'s tests, which a mutant in `network` does not run; and
  `Cut::of`'s `> 0` → `>= 0`, since no test had a writer whose feed is empty (its first
  entry missing). Both killed in `cuts.rs`; the re-run gives 47 caught, 12 unviable, none
  missed.
- `protocol/src/ledger.rs` (`--test=ledger`): 10 mutants — 8 caught, 2 unviable.
- `p2p/src` (the replica on disk): 14 mutants — 8 caught, 6 unviable.

## Run 17 — T74, second step: members' cuts and the beacon between nodes

Same settings as run 16, on the step's diff:

- `network/src` (`--test=cuts`, `--test=cut_signing`): 69 mutants — 58 caught, 8 unviable,
  3 missed: `Cut::next` accepting a longer feed on another branch as extending the last mark
  (`&&` → `||`), `should_sign` counting a commit that sits right at the mark's end (`>` →
  `>=`), and requiring a commit in a cut that does not close (`&&` → `||`). Killed in
  `cut_signing.rs`; the re-run gives 61 caught, 8 unviable.
- `protocol/src/ledger.rs` (`--test=ledger`, `--test=beacon_on_cuts`): 10 mutants — 8
  caught, 2 unviable.
- `p2p/src`: 38 mutants — 28 caught, 4 unviable, 6 missed, all in `MemberRole::duties`: when
  the proposer closes, its waits for commits and for reveals, and its patience — masked in
  `beacon_between_nodes.rs`, where the patience eventually proposes anyway. Killed by
  `member_duties.rs`, which calls the duties directly; the re-run gives 34 caught, 4
  unviable.

## Run 18 — T25, first step: the guessing floor in the latent re-check

`cargo mutants --in-diff` on the step's diff, with `--no-config`, the `mutants` profile, a 3×
timeout multiplier and the 60 s floor — without `calibration`, whose paper-scale suites
would take hours per mutant:

- `scoring/src` (`latent.rs`, `dtf.rs`; the unit tests and `--test=dtf`, `--test=golden`,
  `--test=latent_guessing`, `--test=reproducibility`): 372 mutants in 36 minutes — 347
  caught, 19 unviable, 6 missed. One was real: `>` → `==` in `softplus` takes
  `ln(1 + e^z)` directly, which overflows past `z ≈ 709`, and no fit reaches such a logit;
  `the_floor_helpers_are_stable_at_extreme_arguments` pins the helpers at ±800 and
  `log_odds` at its analytic limits. One is gone: a guard around the floors' penalty only
  made `count > 0` → `>= 0` possible, and with no floor the penalty is +0.0, so the guard
  was dropped and the golden rows did not move. The re-run of `softplus` and `fit_from` gives
  18 caught and the two below. The other four are equivalent:

  | Mutant | Why it is equivalent |
  |---|---|
  | `latent.rs` `softplus` `z > 0.0` → `>=` | At `z = 0` both branches give `ln 2`, bit for bit. |
  | `latent.rs` `log_odds` `wrong < 0.5` → `<=` | Differs only when `P(x = 0)` is exactly 0.5, where both branches compute `ln 0.5`. |
  | `latent.rs` `evaluate` `floors.count > 0` → `>=` | Runs the floors' per-respondent work on a batch that has none: every term it adds is 0 and every table empty, so only the cost changes. |
  | `latent.rs` `fit_from` `floors.count > 0` → `>=` in the NLL returned | Re-evaluates the likelihood instead of unscaling the optimizer's cached objective: the same value up to an ulp, and bit for bit on every dataset the suites pin — applied by hand, the golden rows, every candidate's BIC included, did not move. The guard keeps a batch of open answers on the arithmetic T24's records ran. |

- `protocol/src` (`revalidation.rs`, `results.rs`; `--test=results_replay`,
  `--test=anchor_reliability`, `--test=inv8_batch_min`, `--test=proto013_respondent_gate`,
  `--test=contested_facts`): 12 mutants — 10 caught, 2 unviable. The gate's explicit format
  check (`||` → `&&`) is killed by a batch whose anchors are unreliable and whose formats
  are missing: it must read `BadFormats`, not `UnreliableAnchors`.
- `characterization/src/run.rs` (`--test=production`, `--test=harness`): 8 mutants — 5
  caught, 3 unviable.

## Run 19 — T25, third step: the supplement's harness

`cargo mutants --no-config` on the step's diff of `crates/characterization/src` — the
`mutants` profile, a 3× timeout multiplier and the 60 s floor, `-j 4`; the crate has no
`calibration` — one run per file with the tests that read it:

- `summary.rs` (`--test=summary`): 78 mutants in 21 minutes — 58 caught, 3 unviable, 17
  missed.
- `record.rs` (`--test=harness`, `--test=summary`): 16 mutants — 12 caught, 4 unviable.
- `grid.rs` (`--test=grid`, `--test=harness`): 66 mutants — 43 caught, 5 unviable, 18
  missed.
- `generate.rs` (`--test=generate`, `--test=harness`): 111 mutants — 98 caught, 3
  unviable, 10 missed.
- `run.rs` (`--test=harness`, `--test=production`, `--test=summary`): 41 mutants — 29
  caught, 7 unviable, 5 missed.
- `main.rs` (`--test=grid`): the diff's one mutant, `parse` returning `Ok(Default::default())`,
  does not build — `Args` has no default. The file's other lines, T24's printout of `plan`
  and the dispatch of `main`, are outside the diff and reached by no test: the binary's
  glue, run by hand (`docs/13` §2).

None of the 50 survivors was a wrong result; each was a check the suite did not make, and
the tests that make them kill them on the re-run:

- *The draws of the new options.* A test of a distribution cannot see a change that keeps
  it — `+` → `−` on a symmetric normal in `ability`, the template's effect subtracted or
  scaled — so `floor-misspec`'s smoke record, which draws skew, templates and spread, is
  pinned, and so are the bits of a drawn batch (`a_batch_s_draws_are_pinned`): the same bits
  T24's generator draws at `5931ff1`, checked on a worktree of it. The batch also reads the
  second axis's class `z₂`, which no test read since T24.
- *The coverage guard* of the gate's record code and of the extra round's reach: they are
  now `run::gate_char` and `run::redecides`, tested at `MIN_COVERAGE`
  (`the_gate_s_code_sends_an_uncovered_item_to_review`); the sweep shares the first.
- *The supplement's cells*: a field or a sign dropped in `full` or `smoke` leaves the cell
  counts unchanged, so the keys of both grids are pinned against `docs/13` §8.3
  (`the_supplement_s_cells_are_the_specified_ones`), and every study must cite what it
  measures (`every_study_names_what_it_measures`).
- *The summary's floor tables*: the floor test now carries a second format, a null with 20
  anchors and a power cell off each condition of the cut table's selection, and reads each
  floor table's header and the cell the cut table names; the threshold tables are checked
  with one study's records alone.

The re-run on the survivors' lines: `grid.rs` 19 caught; `run.rs`, `gate_char` and
`redecides`, 13 caught; `generate.rs` 25, then 9 on the lines of `z₁` and `z₂` once the
batch was pinned — all caught, one as a timeout (a template's effect divided by a normal
has tails that keep the fit from converging in time), but for the two equivalents below;
`summary.rs` 32, then 11 on the cut table's selection once the test named the cell it
reads — all caught but for the equivalent below. Three are equivalent:

  | Mutant | Why it is equivalent |
  |---|---|
  | `generate.rs` `dif_batch` `z1 > 0.0` → `>=` | `z₁` is +1 or −1, never 0. |
  | `generate.rs` `dif_batch` `< 0.5` → `<=` in the draw of `z₂` | Differs only on a uniform draw of exactly 0.5, one chance in 2⁵³ per respondent, and no pinned batch meets one. |
  | `summary.rs` `extra_pass` `TAU + eps` → `TAU − eps` | Reached only for a probe outside the band `[τ − ε, τ + ε)`, where a score at or above `τ + ε` and one at or above `τ − ε` are the same probes. |

## Run 20 — T82: the ability's shape estimated, one template per batch

`cargo mutants --no-config --in-diff` on T82's diff (`70d7c9b`) — the `mutants` profile, a
3× timeout multiplier and the 60 s floor, `-j 4`, without `calibration` — one run per crate
with the tests that read its files:

- `scoring/src` (`latent.rs`, `dtf.rs`; the unit tests and `--test=dtf`, `--test=golden`,
  `--test=latent_guessing`, `--test=latent_misspecification`, `--test=reproducibility`):
  357 mutants in 30 minutes — 339 caught, 18 unviable, none missed. The gradient check,
  the histogram's weights and moving nodes included, kills every change to the
  likelihood's arithmetic; the golden rows and the fits, the rest.
- `protocol/src` (`pilot.rs`, `revalidation.rs`, `results.rs`, `exposure.rs`;
  `--test=shared_templates`, `--test=results_replay`, `--test=anchor_reliability`,
  `--test=inv8_batch_min`, `--test=proto013_respondent_gate`, `--test=contested_facts`,
  `--test=lifecycle`): 16 mutants in 14 minutes — 12 caught, 4 unviable.
- `characterization/src/run.rs` (`--test=misspecification`, `--test=production`,
  `--test=harness`): 10 mutants in 24 minutes — 9 caught, 1 unviable.

No survivor, and so no equivalent to justify.

## Keeping it this way

- New decision logic gets a hand-computed or exact-outcome test, not only a range or
  an ordering check: those are what the survivors were.
- Re-run on the files you touched; a new survivor is either killed or added to the
  table above with its reason.
- The golden file changes only on purpose. Its diff is part of the review of any
  change to the engine.
