# Isegoria — Characterization studies (T24, T25 step 3)

| | |
|---|---|
| **Purpose** | The specification of T24: the simulation studies that measure how the production detectors and gates behave, how to run them, and what counts as done. |
| **Derived from** | `docs/10` T24, T25 (step 3) and §1.4; `docs/08` §5.3, §12 (AT-DIF-01..09, AT-DIF-13, AT-BR-02), §16.1 (SC-2, SC-3, SC-5, SC-7); `docs/07` §12–§14; `docs/02` §B.1 (D25); the working paper's designs (`paper/scripts/common.py`). |
| **Status** | T24 done (2026-09-28). Specified and harness built on 2026-09-26 (`crates/characterization`); the full run, 54,412 runs on the owner's machine, ended on 2026-09-28 and §7 states its results. T25 sets the thresholds from §7, the DIF ones once the model accounts for guessing (D25) and a supplement of these studies has measured it (`docs/10` T25, steps 1–3, §7.5). The supplement, T25's step 3, is specified in §8 (2026-09-28); its full run, 19,700 runs on the owner's machine, ended on 2026-09-30 and §8.7 states its results; the screen's study waits on T25's second step. T82 (`docs/01` D43, 2026-09-30) then changed the target model: the DIF results of §7 and §8.7 describe the model before it, and the supplement's DIF studies run again on it (§8.8). |

## 1. What T24 measures, and what it does not

Every threshold of the mechanism is provisional until it is measured (`docs/10` §1.4,
`docs/07` §13). T24 measures the **production** estimators and gates on synthetic
populations with a known truth, over the regimes a deployment will meet, and reports rates
with intervals. It sets no threshold: T25 reads the tables of §7 and sets them.

| Study | Question | `docs/08` |
|---|---|---|
| `dif-null` | How often does the latent re-check flag a clean item, by sample size, batch size and anchor reliability? | AT-DIF-01, DIF-008, the KR-20 floor (T53) |
| `dif-power` | How often does it flag a biased item, by shift, number of biased items, class balance, sample and batch size? | AT-DIF-02, SC-2, STAT-001 |
| `dif-misspec` | Does a real ability gap between the classes (impact) or guessing on multiple-choice items create or hide DIF? | `docs/07` §14, D25 |
| `dif-nonuniform` | Is DIF in the discrimination found, and where would a cut on `a_gap` sit? | AT-DIF-03, T40 |
| `dif-two-axes` | Is bias on two independent hidden axes found? | AT-DIF-04 |
| `dif-poisoning` | What share of coordinated respondents creates DIF on a clean item, or hides it on a biased one? | AT-DIF-07, SC-7 |
| `dif-pool-scale` | What does a batch of 32 or 100 items cost? | AT-DIF-09, DIF-009 |
| `dtf-error` | How far is the fitted DTF of a set of contested facts from its true DTF? | DIF-011 (T55) |
| `bridging-sweep` | How well does the side-balanced score recover the design's quality, and how often does the gate pass what it should not? | SC-3, BRIDGE-002, BRIDGE-003 |
| `bridging-capture` | How many boosters from the camp a partisan item disfavours does it take to pass it? | AT-BR-02, BRIDGE-005 |

**Not in T24.** AT-DIF-05 (one rule on one metric) is a consistency fix: T35 settled the
metric, T25 sets the value. AT-DIF-06 (a separated item) and AT-DIF-08 (oscillating
purification, Variant 1, calibration-only) are constructed cases, deterministic tests
rather than studies. The Level C parameters — the CUSUM `k`/`h`, the weight cap, `γ`,
`N_PROBATION`, the exploration rate — get their calibration procedures in T25. Real
response data exist only in pilots (T27): everything here is inside the simulated regime,
and §7's statements say so.

## 2. The harness

`crates/characterization` is a workspace member but not part of a node: it depends on
`scoring` and `protocol` and nothing depends on it. One binary:

```sh
cargo run --release -p characterization -- plan                 # studies, cells, runs
cargo run --release -p characterization -- run --grid smoke     # one tiny cell per study
cargo run --release -p characterization -- run --replicates 20  # a first pass: 10% of the grid
cargo run --release -p characterization -- run                  # the full grid of §4
cargo run --release -p characterization -- summarize            # summary.md and CSV tables
```

| Option | Meaning | Default |
|---|---|---|
| `--grid full\|smoke` | the grid of §4, or one tiny cell per study | `full` |
| `--study NAME[,NAME…]` | the studies to plan or run: names, `t24` (§4), `t25` (§8) or `all` | all |
| `--replicates R` | the first `R` replicates of each study, at most its own count | the study's |
| `--filter TEXT` | only the cells whose key contains `TEXT` | none |
| `--jobs J` | worker threads | the number of cores |
| `--out DIR` | where the records go | `characterization-results` |
| `--quiet` | no progress lines | off |

**Records.** Each run appends one line to `<out>/<study>/records.csv` as it completes: the
study, the cell's key, the replicate, the seed, the time, and the outcome (§4). Lists
inside a field are `;`-separated; numbers are written in their shortest exact decimal
form, so a record reads back to the same bits. `summarize` writes `<out>/summary.md` and
the CSV tables of §5 beside the records; `run` summarizes when it ends.

**Seeds and reproducibility.** A run's seed is the first eight bytes of SHA-256 over
`isegoria/characterization/v1`, the study's name and the cell's key (each
length-prefixed) and the replicate. The run draws its population from a ChaCha8 stream
on that seed, with the transcendental functions of the pure-Rust `libm` the engine uses
(`scoring::fmath`), and fits it with the engine, whose bits do not depend on the
platform (INV-7, AT-BR-04). The engine's own random starts take a second seed, SHA-256
over `isegoria/characterization/engine/v1` and the run's seed, so they owe nothing to the
population's stream, as in production, where the seed does not depend on the data. So every record reproduces from its study, cell and replicate
on any machine, with any number of workers (`tests/harness.rs`): a surprising cell can be
re-run alone with `--study` and `--filter`.

**Interruptions.** The runs are ordered replicate by replicate, so a partial run covers
every cell evenly. Stopping the process (Ctrl-C, a reboot) loses at most the runs in
flight: at the next `run` a torn last line is cut off, the recorded runs are skipped and
the rest continue. `--replicates 20` followed later by a plain `run` adds replicates 20
to 199 to the first twenty. A run that panics is written to `<out>/errors.log`, which
lists the failures of the latest `run` only, and left unrecorded, so the next `run`
retries it. One process per output directory; a study named twice runs once.

**Fidelity to production.** The DIF studies fit `scoring::latent::latent_dif` with the
production settings and read the verdict with `protocol::revalidation::target_flags`, the
rule of `revalidate_batch_latent`. The gates are recorded, not applied: `admitted` is
`K ≥ K_MIN`, `N ≥ N_LATENT_MIN` and KR-20 ≥ `KR20_MIN`, so the study also measures what
the gates refuse. The rates are over all runs — the engine's behaviour — and every DIF
table gives the share production would admit; `summary.csv` also gives the batch, power
and clean-item rates over the admitted runs alone, which `dif-null` and `dif-misspec`
show (guessing brings the anchors' KR-20 to the floor). `tests/production.rs` checks that an admitted batch gets the flags of
`revalidate_batch_latent` and a refused one is recorded as refused. The bridging studies
compute the gate's robust score as the epoch does (`bridge_scores` with 10 bootstrap
subsamples keeping 85% of the ratings) and read it with `gate::bridging_gate` at the
production `τ`, `ε` and `γ_appeal`.

**Cost.** The DIF studies dominate: a fit is single-threaded and its time grows with the
sample, the batch and the number of classes the BIC tries. Measured in the release
profile on this repository's 4-core development container (one run per cell):

| Cell | Time of one run |
|---|---|
| `dif-null`, N = 1,500, K = 4 | 7 s |
| `dif-null`, N = 3,000, K = 8 | 15 s |
| `dif-null`, N = 6,000, K = 16 | 68 s |
| `dif-power`, N = 3,000, δ = 0.5, two biased items | 20 s |
| `dif-power`, N = 6,000, δ = 0.9, three biased items, π = 0.1 | 81 s |
| `dif-two-axes`, N = 6,000, δ = 0.9 | 148 s |
| `dtf-error`, N = 6,000, δ = 0.9, π = 0.3 | 75 s |
| `bridging-sweep`, 800 reviewers | 2 s |
| `bridging-capture`, item 09 (25 steps) | 5 s |
| `dif-pool-scale`, K = 32 / K = 100, no leaning item | 32 s / 27 s |
| `dif-power`, N = 3,000, 20 or 40 anchors (added after the first pass; mean of 8 cells) | 25 s |
| `dif-nonuniform`, three leaning items (added; mean of 8 cells, 70 s at N = 6,000, δ = 0.9, α = 1.6) | 33 s |

The full grid is 54,412 runs and about 425 CPU-hours — the sixteen cells added after the
first pass are about 25 of them — more than half of it in `dif-power` and most of that in
its cells at N = 6,000: about a day on 16 cores, and about 30 hours on the owner's 8 cores
and 16 threads (§7). A first pass with `--replicates 20` takes a tenth of that and
already gives every cell's point estimates; the intervals of §7 need the full count. The
cloud development container has 4 cores and takes three to four times as long as the
owner's machine: the first pass and the full run go to the owner's machine, the container
runs the smoke grid and single cells.

**On the owner's machine.**

1. `git checkout feat/t24-characterization` (or the commit it was merged at), then
   `cargo run --release -p characterization -- run --grid smoke --out smoke-check`: it
   takes under a minute and checks the build and the machine.
2. `cargo run --release -p characterization -- run --replicates 20`, then look at
   `characterization-results/summary.md`.
3. `cargo run --release -p characterization -- run` for the rest; stop and restart it
   freely.
4. Send back `summary.md`, the `summary.csv` files, `thresholds-*.csv`, `tau.csv` and
   `curve.csv`, with the commit (`git rev-parse HEAD`) and the machine. The records stay
   with the owner: every one of them reproduces from its seed.

**After an engine change.** Records are keyed by study, cell and replicate, not by
commit, so a `run` skips what is recorded even if the code changed. A study whose code
changed is re-run by moving its directory aside — `mv <out>/<study> <out>-before/` keeps
the old records as a baseline — and running it again with `--study`; the summary then
reads the new records with the others. After T71: `bridging-sweep` and
`bridging-capture`.

## 3. The populations

### 3.1 Latent-DIF batches

Every `dif-*` study and `dtf-error` draws a batch of `N` respondents, `A` anchors and `K`
trial items — the paper's `dif_generate` (`paper/scripts/common.py`), extended:

- anchors: `a ~ U(0.9, 1.6)`, `b ~ N(0, 1)`; trial items: `a_j ~ U(1.0, 1.5)`,
  `b_j ~ 0.6 · N(0, 1)`;
- each respondent: a class `z₁ = +1` with probability `π`, else `−1`; an independent
  second class `z₂ = ±1` with probability ½; ability `θ ~ N(0, 1) + impact · [z₁ = +1]`;
- an anchor is answered correctly with probability `c + (1 − c) σ(a(θ − b))`, `c` the
  guessing floor;
- trial item `j` has signs `s₁ⱼ, s₂ⱼ ∈ {−1, 0, +1}` on the two axes; with the lean
  `ℓ = s₁ⱼ z₁ + s₂ⱼ z₂`, it is answered correctly with probability
  `c + (1 − c) σ((a_j + α/2 · ℓ)(θ − b_j − δ ℓ))`: a difficulty gap `2δ` and a
  discrimination gap `α` between the classes of a leaning item.

The layout sets the signs: a *campaign* of `n` items (`s₁ = +1` on the first `n`), a
*mirror* (`+ − + −` on the first four), or *two axes* (`n` items on each). Coordinated
respondents are the last `round(fraction · N)` of the sample: *injecting* ones answer the
last `targets` trial items wrong and the rest honestly; *masking* ones answer the leaning
items as if they leaned on nothing. Each record carries the items' roles: `+` and `-`
lean on the first axis, `2` on the second, `t` is a clean target of an injection, `c` is
clean. With `π = 0.5` and the extensions at zero the population is the paper's; its
random stream is not (the order of draws is fixed by `generate::dif_batch`).

### 3.2 The Level A mirror design

`bridging-sweep` draws the paper's mirror design (`mirror_design`) with the consensus
items spread across the threshold: `n` reviewers in two camps, the majority's share
`share`, positions `±1 + 0.25 · N(0, 1)`, a severity `0.06 · N(0, 1)` each; ten consensus
items of quality `q ~ U(0.70, 0.95)` and no lean, ten partisan items of quality 0.55 in
mirror pairs leaning `±0.8`; each reviewer rates `per_reviewer` items at random, the
rating `clamp(q + 0.45 · position · lean + severity + noise · N(0, 1), 0, 1)`. An item's
*truth* is what the side-balanced score estimates: the mean over the two camps of each
camp's mean expected rating, the clamp included (`E[clamp(X, 0, 1)]` for a normal `X`,
in closed form). It is `q` for a consensus item away from the bounds and a little below
it near the top (0.91 at `q = 0.95` with noise 0.15); a partisan item's is about 0.55.
An item should pass when its truth is above `τ` and fail when it is below.

### 3.3 The capture design

`bridging-capture` uses the reference simulation's dataset (the fixtures of
`crates/scoring/tests/fixtures`, 200 reviewers in camps of 80 and 120, ten items). On a
partisan item (08 favours the majority camp, 09 the minority one), `own` reviewers drawn
from the camp the item favours and then the reviewers of the other camp, in a drawn
order, rate it 1.0; the gate's score is read at every `step` of the opposing count. The
item *passes* at the first opposing count at which its robust score reaches `τ + ε`,
*passes from then on* at the first count after which it never falls back below it, and
*reaches the band* — supplementary review — at the first at which it reaches `τ − ε`.

## 4. The studies

Two hundred replicates per cell, three for `dif-pool-scale`; 54,412 runs. Unless a row
says otherwise a DIF batch has `N = 3,000`, 60 anchors (KR-20 ≈ 0.93), `K = 8`, `π = 0.5`,
no impact, no guessing, no attack.

| Study | Cells | Factors |
|---|---|---|
| `dif-null` | 27 | `N ∈ {1500, 3000, 6000}` × `K ∈ {4, 8, 16}` × anchors `∈ {20, 40, 60}`; no leaning item |
| `dif-power` | 140 | at `K = 8`: `N ∈ {1500, 3000, 6000}` × `δ ∈ {0.3, 0.5, 0.7, 0.9}` × biased items `∈ {1, 2, 3}` × `π ∈ {0.5, 0.3, 0.1}`; at `N = 3000`: `K ∈ {4, 16}` × `δ ∈ {0.5, 0.9}` × biased `∈ {1, 2, 3}` × `π ∈ {0.5, 0.3}`; added after the first pass, at `N = 3000`, `π = 0.5`: anchors `∈ {20, 40}` × `δ ∈ {0.7, 0.9}` × biased `∈ {2, 3}` — the power the KR-20 floor would cost, since no null batch flagged a clean item at 20 anchors |
| `dif-misspec` | 22 | impact `∈ {0, 0.5, 1.0}` × guessing `∈ {0, 0.2}` × biased items `∈ {0, 2}` (`δ = 0.9`) × `π ∈ {0.5, 0.2}`, `π` varied only where it matters |
| `dif-nonuniform` | 16 | `N ∈ {3000, 6000}` × `α ∈ {0.4, 0.8}` × `δ ∈ {0, 0.5}`, two leaning items; added after the first pass, where no cell selected a mixture: three leaning items, `α ∈ {0.8, 1.6}` × `δ ∈ {0, 0.9}` — with `δ = 0.9` the mixture is found and `a_gap` can be read |
| `dif-two-axes` | 4 | `N ∈ {3000, 6000}` × `δ ∈ {0.5, 0.9}`, two items on each axis |
| `dif-poisoning` | 15 | injecting: `fraction ∈ {0, 1, 2, 5, 10%}` × targets `∈ {1, 2}` on a clean batch; masking: `fraction ∈ {0, 1, 2, 5, 10%}` on two items with `δ = 0.9` |
| `dif-pool-scale` | 4 | `K ∈ {32, 100}`, no leaning item or a tenth of the items leaning with `δ = 0.9` |
| `dtf-error` | 8 | `N ∈ {3000, 6000}` × `δ ∈ {0.5, 0.9}` × `π ∈ {0.5, 0.3}`, the mirror layout |
| `bridging-sweep` | 36 | `n ∈ {100, 200, 800}` × `share ∈ {0.5, 0.6, 0.8}` × `per_reviewer ∈ {5, 9}` × `noise ∈ {0.07, 0.15}` |
| `bridging-capture` | 4 | item `∈ {08, 09}` (keys `item=7`, `item=8`, counted from 0) × `own ∈ {0, 40}`, opposing boosters in steps of 5 |

**What a run records.** A DIF run: the anchors' KR-20, `admitted`, the selected number
of classes, uniform or not, convergence, the BIC gain, the class shares and means, per
item `DIF_j`, `a_gap` and the production flag, and the roles. A `dtf-error` run: the fit's
classes, convergence and flags, and for eight item sets of the mirror layout — each
leaner of the first pair, the two mirror pairs, a same-side pair, the four leaners, the
four clean items, a pair with two clean items — the DTF of the fitted curves
(`ClassCurves::of`) and of the true ones on the same 41-node grid. A sweep run: the axis
recovery `|corr(f_u, true position)|`, convergence, and per item `q`, the lean, the
truth, the full and robust scores, the side gap and the gate's outcome — `P`, `S`, `A`,
`R`, or `U` for an item below `MIN_COVERAGE` that the gate sends to review (D42). A capture run: per step the opposing count, the full
and robust scores, and the item's plain mean rating.

## 5. Statistics

- **Over runs** — batches with a clean item flagged, all biased items flagged, items that
  never pass — a proportion with its 95% Wilson interval; runs are independent.
- **Over items** — clean items flagged, power per biased item — the pooled rate with a
  Wilson interval on the effective sample size `p(1 − p) / Var(p̂)`, the variance of the
  pooled rate estimated from the batches (the ratio estimator over batch totals), kept
  between `(Σm)² / Σm²` — every batch counted once, the fewest — and `Σm`, every item
  counted once; the fewest when the rate is 0 or 1 or there is one batch. The items of one
  batch share one fit, so they are not independent trials.
- **Batches with a clean item flagged** counts the items marked `c` only: a leaner or an
  injection's target does not make a batch a false positive.
- **Engine and production.** Every DIF table gives the rates over all runs; the
  admitted column and the admitted-batch rate restrict them to the runs the gates admit.
- **Threshold tables.** The flags at other cuts follow the production rule — a converged
  fit with two or more classes, a gap above the cut: `thresholds-dif-cut.csv` gives, for
  cuts 0.5–1.5 on `DIF_j`, the clean-item rate on admitted null batches and the power of
  every `dif-power` cell with two biased items of eight, `π = 0.5`, 60 anchors, `N ≥ 3000`;
  `thresholds-a-gap.csv` the same for cuts 0.2–1.0 on `a_gap`, with the pure non-uniform
  cells of `dif-nonuniform`, labelled by their number of leaning items; `bridging-sweep/tau.csv`, for `τ` from 0.70 to 0.90, the
  share of the items whose truth is below `τ − 0.05` that the robust score passes and of
  those whose truth is above `τ + 0.05` that it fails, per reviewer count.
- **DTF.** Over the runs whose fit converged with two or more classes — the fits whose
  flags could put facts in the pool — the error is the fitted minus the true DTF. A
  set is falsely admitted when its fitted DTF is within `DTF_MAX` and its true one is
  not: the tables give that share of the fitted runs, and the share of the sets truly
  over the tolerance that the fitted value admits.
- **Quantiles** are type 7 (linear); an item that never passes counts as infinite, and a
  quantile that reaches it reads "never".
- **Estimated gaps.** An item's `DIF_j` or `a_gap` is summarized by its median over the
  runs whose fit selected two or more classes (the mean is kept in `summary.csv`): a
  class-specific difficulty can diverge on a leaning item (quasi-separation — 5 of 32
  estimates above 10 logits in one first-pass cell, the verdict right), and a mean then
  says nothing about the size of the gap.
- **DTF refusals.** The mirror of a false admission: a set whose fitted DTF is over
  `DTF_MAX` while its true one is within, as a share of the fitted runs and of the sets
  truly within.
- **Coverage.** The sweep table gives the share of items the gate sends to review because
  a side never rated them (`U`, D42), apart from the band.

## 6. Done when

1. The full grid ran on one commit of the harness — every one of the 54,412 runs
   recorded, the last `run` with no `errors.log` — and `summarize` ran on the records.
   A study whose code a later commit changes is re-run on that commit and the others
   are shown to reproduce on it: after T71 (D42) the two bridging studies are re-run,
   while the DIF and DTF records of the first pass's commit (`e8dcc7d`) reproduce bit for
   bit on T71's. `tests/harness.rs` pins one record of each kind, so a change to what a
   study measures shows in the tests.
2. §7 holds the summary's tables with the commit, the date, the machine and the wall
   time; the CSV tables are committed under `verification/reports/t24/` (`docs/07` §24);
   the records stay with whoever ran them, reproducible from their seeds.
3. Each study's result is stated in the form `docs/07` §14 asks for — under these
   assumptions, on these populations, this sensitivity and this specificity, and outside
   them nothing established — and `docs/08` is updated: AT-DIF-01, 02, 03, 04, 07, 09 and
   AT-BR-02 in §12; DIF-008, STAT-001, BRIDGE-002, BRIDGE-003, BRIDGE-005 and DIF-011 in
   §15, raised to SCIENTIFICALLY CHARACTERIZED where the tables support it and inside
   their regime only.
4. T25 is then unblocked: it sets the DIF cut, a cut on `a_gap` or none, `KR20_MIN`, the
   sample floors, `DTF_MAX`, `τ` and `ε` from §7, with the calibration procedure
   `docs/07` §13 requires — the DIF ones after the guessing correction and a supplement
   of these studies on the corrected model (§7.5).

## 7. Results

**The run.** 54,412 runs, every run of the grid recorded, on the owner's machine (AMD
Ryzen 7 5800X, 8 cores; about 30 hours in all, 2026-09-26 to 2026-09-28). The DIF and
DTF studies ran on `e8dcc7d`; their records reproduce bit for bit on the later commits —
one cell re-run and compared, one record of each kind pinned (§6). The two bridging
studies, re-run after T71, and the sixteen cells added after the first pass ran on
`9478533`, which summarized every record. The summary and the threshold tables, with
their provenance, are in `verification/reports/t24/`; the records stay with the owner.

**What every statement below assumes.** The populations of §3 and nothing else:
respondents in two latent classes on an axis the model never sees, 2PL items — a
guessing floor only where a statement says so — and one or two lean patterns; reviewers
in two camps with a linear rating model. A rate is over the runs or items of its cell,
with the 95% interval of §5; "never" means 0 of the cell's 200 runs, an upper bound of
1.9%. Outside these populations the behaviour is not established (`docs/07` §14).

### 7.1 Latent DIF

- **Specificity** (`dif-null`, AT-DIF-01, DIF-008). On null batches — `N` 1,500–6,000,
  `K` 4–16, 20–60 anchors (KR-20 0.83–0.94) — the target model never selected a mixture
  and flagged no clean item: 0 of 5,400 batches, 0 of 50,400 items, at most 1.9% in any
  cell. Over the 2,307 batches the gates admit, the clean-item rate is 0.0% [0.0, 0.2] at
  every cut from 0.5 to 1.5. In `dif-power`, with leaning items in the batch, no clean
  item was flagged but for 0.1% in two cells with `N` = 1,500, which the gates refuse.
- **Sensitivity** (`dif-power`, AT-DIF-02, STAT-001): the share of leaning items flagged,
  `K` = 8, 60 anchors, classes of equal size (`π` = 0.5):

  | Leaning items | δ | N = 3,000 | N = 6,000 |
  |---|---|---|---|
  | 1 | 0.3–0.9 | never | never |
  | 2 | 0.3–0.5 | never | never |
  | 2 | 0.7 | 11.2% [7.6, 16.3] | 56.2% [50.0, 62.4] |
  | 2 | 0.9 | 92.8% [88.3, 95.6] | 98.8% [97.1, 99.5] |
  | 3 | 0.3 | never | never |
  | 3 | 0.5 | 3.3% [1.7, 6.4] | 28.8% [23.9, 34.3] |
  | 3 | 0.7 | 95.8% [92.4, 97.7] | 100% [98.1, 100] |
  | 3 | 0.9 | 100% [98.1, 100] | 100% [98.1, 100] |

  One leaning item is never seen, in any of its 44 cells — `N` up to 6,000, δ up to 0.9:
  the model finds a campaign, not a single item (`docs/02` §B.3). A smaller class costs
  power: two items at δ = 0.9 are found in 71.2% / 97.2% of cases (`N` = 3,000 / 6,000)
  at `π` = 0.3 and in 1.0% / 7.5% at `π` = 0.1; three items at δ = 0.9, `π` = 0.1, in
  35.7% / 96.2%. So does a larger batch: two items at δ = 0.9, `N` = 3,000, are found in
  97.2% of cases among 4 items, 92.8% among 8 and 37.5% among 16. At `N` = 1,500, below
  the admission floor, with `π` ≥ 0.3, three items at δ = 0.9 are found in 95.7–99.0% of
  cases and two in 12.0–37.2%.
- **The estimated gap.** Where most fits find the mixture — power above 75% — the median
  fitted gap of the leaning items is close to the true `2δ`: 1.78–1.91 for 1.80 and
  1.41–1.47 for 1.40. Where few do, it is inflated, up to 2.48 for 1.80 (`N` = 1,500,
  `π` = 0.3): the fits that select a mixture there are those that happened to see a
  larger gap.
- **Anchors.** At `N` = 3,000, `π` = 0.5, with two or three leaning items at δ 0.7 or
  0.9, 20 anchors (KR-20 ≈ 0.83, below `KR20_MIN`, so never admitted) and 40 (≈ 0.91,
  admitted in 89.5–92.0% of the batches) flag no clean item and find the leaning items as
  often as 60 do, but in one cell: two items at δ = 0.9, 86.5% with 20 or 40 anchors
  against 92.8% with 60.
- **Two axes** (`dif-two-axes`, AT-DIF-04). With two items leaning on each of two
  independent axes, the items of each axis are found at δ = 0.9 — 85.0% and 80.5% at
  `N` = 3,000, 99.5% and 100% at `N` = 6,000 — and never at δ = 0.5; no clean item is
  flagged.
- **Pool scale** (`dif-pool-scale`, AT-DIF-09, DIF-009). Batches of 32 and 100 items fit
  in 32–58 s each, flag no clean item and find every leaning item at δ = 0.9: three runs
  per cell, a check of feasibility, not a rate.

### 7.2 Robustness

- **Impact** (`dif-misspec`). With the class `z₁ = +1` more able by 0.5 or 1, the target
  model selects no mixture on a clean batch and flags no clean item but in one cell
  (0.1%). At `π` = 0.5 impact costs power — two items at δ = 0.9 are found in 90.5%,
  87.2% and 63.5% of cases at impact 0, 0.5 and 1 — and at `π` = 0.2 it does not
  (32.0–39.0%).
- **Guessing** (`dif-misspec`, D25). With a guessing floor `c` = 0.2 in the population and
  the 2PL model, the target model selects a mixture in 72.5–98.0% of the batches and
  flags 11.9–16.6% of the clean items — 8.6–20.5% over the batches the gates admit —
  with or without impact. The leaning items are found in 32.0–45.0% of cases at `π` = 0.5
  (90.5% without guessing), their gap read at 0.75–0.98 for 1.80, and in 41.5–89.2% at
  `π` = 0.2. Guessing lowers the anchors' KR-20 to 0.89–0.91, so the floor refuses
  between none and 91% of these batches, and it is no protection: the batches it admits
  flag as many clean items. On a population that guesses, the 2PL target model is not
  usable as it stands.
- **Non-uniform DIF** (`dif-nonuniform`, AT-DIF-03). A discrimination gap alone is not
  seen up to α = 0.8, on two or three items, up to `N` = 6,000: no mixture is selected.
  At α = 1.6 on three items a mixture is selected in 6.5% / 45.5% of the batches
  (`N` = 3,000 / 6,000), the non-uniform model in 1.0% / 24.0%, and the production rule,
  which reads the difficulty gap only, flags 4.5% / 18.7% of the leaning items. With a
  difficulty gap as well (three items, δ = 0.9) the mixture is selected in 99.5–100% of
  the batches, and the non-uniform model in 29.5% / 91.0% at α = 1.6 — reading `a_gap`
  at a median 1.60 at `N` = 6,000 — and in 0% / 5.0% at α = 0.8. Two items with δ = 0.5
  and α 0.4–0.8 are found in 0–12.0% of cases, none with α = 0. At most 0.1% of the
  clean items are flagged.
- **Poisoning** (`dif-poisoning`, AT-DIF-07, SC-7). When coordinated respondents answer
  two clean items wrong, 65.0% [58.2, 71.3] of those items are flagged if the attackers
  are 10% of the sample and 8.0% [5.0, 12.6] at 5%; none at 2% or less, and none with one
  target. No other clean item is ever flagged. Masking — coordinated respondents answering
  the leaning items as if they leaned on nothing — lowers the power on two items at
  δ = 0.9 from 93.5% to 88.2%, 84.5%, 88.2% and 75.0% at 1, 2, 5 and 10% of the sample.

### 7.3 The DTF bound

`dtf-error` (DIF-011) measures the fitted DTF on the mirror layout over the fits that
find two or more classes, the only ones whose flags can put facts in the pool (§5): 62%
and 18% of the batches at `N` = 3,000, δ = 0.5 (`π` = 0.5, 0.3), 86–100% elsewhere.

- **Bias.** The fitted DTF is biased upward where the true one is small: the four clean
  items, true DTF 0, are fitted at 0.025–0.065 on average. Over all sets the error's mean
  is −0.001 to +0.065 and its p95 at most +0.17.
- **Far above `DTF_MAX` = 0.10** sets are refused: a single leaner (true DTF 0.22–0.39)
  was admitted in 1 fit of 123 (`N` = 3,000, δ = 0.5, `π` = 0.5), a same-side pair
  (0.44–0.77) never.
- **Below it** the bias refuses sets that are within: the four clean items are refused in
  14.6% and 22.9% of the fits at `N` = 3,000, δ = 0.5 (`π` = 0.5, 0.3), in 1.5–5.2% at
  `N` = 3,000, δ = 0.9 and at `N` = 6,000, δ = 0.5, and in 0–0.5% at `N` = 6,000,
  δ = 0.9.
- **Near it** — the two mirror pairs, the four leaners and a pair with two clean items,
  true DTF 0.06–0.15 on average — the decision is noisy both ways. Of these sets, 4–18% of
  those truly over the tolerance are admitted (in the weakest cell, 1 of 29); of those
  truly within, 5–22% are refused at `N` = 6,000, δ = 0.9 and 30–68% at `N` = 3,000,
  δ = 0.5.

### 7.4 Bridging, after T71

`bridging-sweep` (SC-3, BRIDGE-002, BRIDGE-003), on the mirror design of §3.2: 100–800
reviewers, camps from 50/50 to 80/20, 5 or 9 ratings each out of 20 items, rating noise
0.07 or 0.15; `τ` = 0.80, `ε` = 0.02.

- **The axis** is recovered with `|corr|` 0.886–0.995 on average per cell; the worst run
  reads 0.57 (100 reviewers, 80/20, 5 ratings each, noise 0.15).
- **Specificity.** No partisan item passed: 0 of 72,000; 99.4–100% of them are eligible
  for appeal by their side gap. A consensus item whose truth is at least 0.05 below `τ`
  passed in one cell only, 0.2% [0.0, 1.2] (100 reviewers, 80/20, 9 ratings, noise
  0.15). For any `τ` from 0.70 to 0.90, without the band, the robust score passes at most
  0.03% of the items at least 0.05 below it with 100 reviewers, 0.003% with 200 and none
  with 800.
- **Sensitivity.** The robust score reads a consensus item low, by 0.003–0.051 on average
  per cell. A consensus item at least 0.05 above `τ` passes in 97.6–100% of cases with
  800 reviewers, 82.9–100% with 200 and 64.3–100% with 100, the lowest with 80/20 camps,
  5 ratings each and noise 0.15; the rest go to the band or fail. Without the band, at
  `τ` = 0.80, the robust score fails 3.6% of the items at least 0.05 above it with 100
  reviewers, 0.7% with 200 and 0.01% with 800. The band holds 13.7–20.0% of the consensus items.
- **Coverage and camp size.** The gate sends 0.1–0.4% of the items to review because a
  side never rated them (D42), only with 100 reviewers in 80/20 camps rating 5 items
  each. On the mirror partisan items the full fit's side-balanced score leaks 0.00–0.06 of
  the camp-size effect on average per cell (sd up to 0.17), the leak of `docs/02` §A.3.

`bridging-capture` (AT-BR-02, BRIDGE-005), on the fixture's camps of 80 and 120: the
opposing reviewers it takes to carry a partisan item to `τ + ε`, rating it 1.0 in a drawn
order, over 200 orders.

| Item | Opposing camp | To pass: median [p5, p95] | To the band | Full fit, to pass |
|---|---|---|---|---|
| 08, favoured by the majority | 80 | 60 [60, 65] | 55 [50, 55] | 60 |
| 08, and 40 of its own camp | 80 | 60 [55, 60] | 50 [50, 55] | 55 |
| 09, favoured by the minority | 120 | 50 [45, 55] | 40 [35, 45] | 45 |
| 09, and 40 of its own camp | 120 | 45 [40, 50] | 35 [30, 40] | 40 |

Every order carries the item through, and the first pass and the stable one have the
same median and range: once it passes, it stays passed.

### 7.5 What T25 takes from it

- **Guessing first (D25).** On a population that guesses, 12–17% of the clean items are
  flagged; no DIF threshold should be lowered before the model accounts for it. And
  guessing is the rule, not a corner case: the system's items are true/false or
  multiple-choice with one keyed option (`docs/README.md`), so a real batch has a floor
  near `1/m` for `m` options — and 0.5 for true/false, which no study here tried. The DIF
  values below hold for items with nothing to guess; for the others T25 measures them
  again on the corrected model (`docs/10` T25, steps 1–3). The model carries the floors
  since T25's first step (`docs/02` §B.1, `docs/08` AT-DIF-13); the harness declares every
  column an open answer, so the records above reproduce. The bridging values (`τ`, `ε`,
  `MIN_COVERAGE`) do not depend on it.
- **The DIF cut** (`MIXTURE_DIF_MAX` = 1.0). On 2PL populations no clean item is flagged
  at any cut from 0.5 to 1.5. At 0.6 and below the cut no longer binds — every leaning
  item of a mixture fit is flagged — and power is the share of fits that select a
  mixture. Lowering the cut to 0.6 raises power at `N` = 6,000, δ = 0.7 from 56.2% to
  65.0%, and by at most 1.2 points at δ = 0.9 (two leaning items of eight, `π` = 0.5):

  | Cut on `DIF_j` | Null, admitted | N = 3,000, δ = 0.7 | N = 3,000, δ = 0.9 | N = 6,000, δ = 0.7 | N = 6,000, δ = 0.9 |
  |---|---|---|---|---|---|
  | 0.5–0.6 | 0.0% [0.0, 0.2] | 12.0% | 93.0% | 65.0% | 100% |
  | 0.8 | 0.0% [0.0, 0.2] | 12.0% | 93.0% | 61.0% | 100% |
  | 1.0 (production) | 0.0% [0.0, 0.2] | 11.2% | 92.8% | 56.2% | 98.8% |
  | 1.2 | 0.0% [0.0, 0.2] | 9.2% | 89.2% | 49.2% | 95.0% |
  | 1.5 | 0.0% [0.0, 0.2] | 5.5% | 73.5% | 32.8% | 81.0% |

- **`KR20_MIN` = 0.90.** With 20 anchors (KR-20 ≈ 0.83) no null batch flags a clean item
  and power is that of 60 anchors but in one cell: the floor could come down, or give way
  to a minimum number of anchors.
- **A cut on `a_gap`** adds detection only for a large discrimination gap in a large
  sample — α = 1.6 at `N` = 6,000: 24.0% of the leaning items, against 18.7% by the
  difficulty gap — and flags no clean item at any value from 0.2 to 1.0; if one is
  adopted, its value within 0.2–0.9 changes nothing.
- **`DTF_MAX` = 0.10.** Within about 0.05 of the tolerance the fitted DTF decides noisily
  both ways, and its upward bias refuses up to 23% of the clean sets in weak batches; a
  margin below the tolerance trades refusals for fewer false admissions, a correction of
  the bias the reverse.
- **`N_LATENT_MIN` = 3,000.** At 1,500 respondents at most 0.1% of the clean items are
  flagged, and three leaning items at δ = 0.9 are found in 96–99% of cases (`π` ≥ 0.3),
  two in 12–37%.
- **`τ` = 0.80, `ε` = 0.02.** With the band, no consensus item whose truth is 0.05 or more
  below `τ` passes from 200 reviewers up; without it the false passes stay at or under
  0.03%. The band takes 14–20% of the consensus items, at the cost of the extra round
  (`k_extra`), which this design does not measure. `MIN_COVERAGE` sends at most 0.4% of
  the items to review.

### 7.6 First pass (2026-09-26)

`--replicates 20` — 5,132 runs, a tenth of every cell — ran on the owner's machine; four
of its cells re-run on the development container reproduced its rows exactly. It found a
defect in the side-balanced score, fixed before the full run as T71 (`docs/01` D42,
`docs/08` BRIDGE-010); its bridging tables, which described the engine before the fix,
are superseded by §7.4. Its DIF tables already showed what §7.1–§7.2 confirm: guessing
makes the 2PL target model flag clean items (D25), and the null batches flag none even
with 20 anchors (KR-20 ≈ 0.83), the case the KR-20 floor was set for with the proxy model
(T53). The sixteen cells added after it (§4) answer the two questions its grid could not.

## 8. The guessing supplement (T25, step 3)

T24 drew its latent-DIF populations without a guessing floor, or with one the 2PL target
model could not read (§7.2). T25's first step gave the model a floor per column (D25,
`docs/02` §B.1): this supplement measures the corrected model on populations that guess,
and adds the two Level A measurements T24's design does not make (`docs/10` T25, step 3).
Same harness, seeds, records and statistics as §2–§5; six new studies, four new
population options, and new record columns. Specified on 2026-09-28, the harness
extended and smoke-tested in the container the same day; the runs, on the owner's machine,
ended on 2026-09-30 (§8.7). The screen's study, a seventh, followed T25's second step.

### 8.1 What it measures

| Study | Question | `docs/08` |
|---|---|---|
| `floor-null` | Does the corrected model select a mixture or flag a clean item on a population that guesses — five options (floor 0.2), four (0.25), true/false (0.5), floors that vary around the declared chance — by sample and number of anchors? What is then the anchors' KR-20? | AT-DIF-13, DIF-008, the KR-20 floor |
| `floor-power` | How often are leaning items found under a floor, by format, sample up to 12,000, shift, number of leaners, anchors and class balance? | SC-2, STAT-001, `N_LATENT_MIN`, the DIF cut |
| `floor-misspec` | Does the corrected model create or hide DIF under impact, a skewed ability, items of one template answered alike, or floors off the declared chance? | `docs/07` §14, D25 |
| `floor-dtf` | How far is the fitted DTF from the true one when the items guess? | DIF-011, `DTF_MAX` |
| `bridging-lambda` | How do the gate's verdicts move with the regularization `(λ_b, λ_f)`? | SC-3, BRIDGE-002, BRIDGE-003 |
| `bridging-extra` | What does the band's extra round decide, by `k_extra` and `ε`, on an item rated by a panel of production size? | BRIDGE-006, PROTO-008 |
| `floor-screen` | How often does the pilot's stage 1 keep a good item and drop a bad one — one that barely discriminates, too hard, guessable beyond its format, or keyed backwards — by respondents, anchors and format, and at which thresholds? | IRT-003, PROTO-005, `A_MIN`, `B_ABS_MAX`, `C_EXCESS_MAX` |

`floor-screen` joined the supplement with T25's second step (2026-09-30), which put the
one-class fit of the floor model in stage 1 (`docs/02` §B.2); a fit of a few hundred
respondents takes about a second, so it runs in the container. The `d = 1` fit on
`d = 2` populations that BRIDGE-002 still lists is not part of T25.

### 8.2 The populations

Latent-DIF batches as in §3.1, with four options, each at its default — and absent from
the cell's key — unless the cell sets it:

- `m`, the format the fit is told: 0 declares every column an open answer, as T24 did;
  `m ≥ 2` declares every column a choice among `m` options, its floor estimated under the
  prior of `docs/02` §B.1, centred on `1/m`. The population's floor stays `g`.
- `gs`: each column's floor drawn from `U(g − gs, g + gs)` rather than all at `g`, the
  anchors' then the items', after the items' parameters.
- `sk`: ability drawn from a skew-normal of shape `sk`, standardized to mean 0 and
  variance 1 (skewness ±0.78 at shape ±4), plus the impact; at 0, one normal as in §3.1.
- `tl`, templates: trial items 0–1, 2–3, … are pairs of one template, and each
  respondent gets an effect `tl · N(0, 1)` per template, added to the ability on its two
  items — a testlet, so the two are answered alike beyond what ability explains. Since
  D43 the gate refuses a batch with two items of one template, so `admitted` is false for
  these cells, and their fits measure a template the batch did not declare.

With the four at their defaults a batch draws exactly the random stream of §3.1, so T24's
records reproduce (§6).

**The extra-round design** (`bridging-extra`). The mirror design of §3.2 — `n` reviewers,
the majority's share, 5 ratings each of the 20 items, noise 0.15 — defines the axis, and a
fit of it alone places the reviewers. Ten *probes* of quality `q ~ U(τ − 0.06, τ + 0.06)`
and no lean join it, each rated by a first panel of `panel` reviewers drawn by
`review::assign_reviewers` on the placed positions, as production draws a panel; a rating
is `clamp(q + severity + noise · N(0, 1), 0, 1)`, drawn for every reviewer and probe
whether the reviewer rates it or not. The gate reads the probes as the epoch does
(`bridge_scores`, `gate::bridging_gate` at the production `τ` and `ε`). A probe whose
robust score is within 0.06 of `τ`, or whose coverage is below `MIN_COVERAGE`, is
re-decided once for each `k_extra ∈ {0, 2, 4, 6, 8, 11}`: `k_extra` reviewers outside its
first panel, drawn by the same stratified draw as `review::assign_extra_from_beacon`, add
their ratings of it, and `gate::supplementary_review` decides on the whole; `k_extra = 0`
re-fits the first panel alone, the rule before T60. A probe's truth is §3.2's.

**The screen's design** (`floor-screen`). `n` respondents of normal ability answer `a`
anchors — `a_j ~ U(0.9, 1.6)`, `b_j ~ N(0, 1)`, a floor at the chance `1/m` of `m`
options — and ten trial items of fixed kinds, every column declared with `m` options:
five good ones, `(a, b)` = (0.8, 0), (1.2, −1), (1.2, 1), (1.6, 0.9) and (1.2, 2); two
flat ones, `a` = 0.3 and 0.45 at `b = 0`; one too hard, (1.2, 3); one guessable, (1.2, 0)
with its floor 0.2 over chance; one keyed backwards, (1.2, 0) scored right where it is
wrong. Every floor but the guessable one's is at chance. The screen runs as production runs
it — `pilot::stage1_fit`, then `stage1_verdicts` — and the record keeps every item's
point-biserial and fitted `a`, `b`, `c`, none for an item the point-biserial left out of
the fit, so the summary can re-apply the other thresholds. `R_PBIS_MIN`, which decides
the fit's items, is not re-applied: another value is another run.

### 8.3 The studies

Unless a row says otherwise a batch has `N = 3,000`, 60 anchors, `K = 8`, `π = 0.5`, no
impact, and a floor with its format written *floor/options*: 0.2/5, 0.25/4, 0.5/2.

| Study | Cells | Replicates | Factors |
|---|---|---|---|
| `floor-null` | 12 | 200 | 0.2/5, 0.25/4, 0.5/2 and 0.2 ± 0.1/5 × `N ∈ {3000, 6000}`; 0.2/5 and 0.5/2 with 20 and 40 anchors; no leaning item |
| `floor-power` | 23 | 100 | 0.2/5 and 0.5/2, each: at `N = 3000` two or three items at `δ = 0.9` and three at 0.7; at `N = 6000` two or three at `δ ∈ {0.7, 0.9}`; at `N = 12000` two at `δ ∈ {0.7, 0.9}`. Then 0.25/4, two items at `δ = 0.9`, `N ∈ {6000, 12000}`; 0.2/5 with 40 anchors, two or three items at `δ = 0.9`, `N = 6000`; 0.2/5 with `π = 0.3`, two items at `δ = 0.9`, `N = 12000` |
| `floor-misspec` | 18 | 100 | at 0.2/5, none or three items at `δ = 0.9`, each with one of: impact 1.0; skew −4, −2 or 4; a floor of 0.1 or 0.3 declared as five options; templates `tl = 1.0`. Then templates `tl = 0.5` with no leaning item; floors 0.2 ± 0.1 with three; and on T24's populations — no floor, every column open — skew −4 and templates `tl = 1.0`, with no leaning item |
| `floor-dtf` | 6 | 200 | the mirror layout: 0.2/5 × `N ∈ {3000, 6000}` × `δ ∈ {0.5, 0.9}`; 0.5/2 × `N ∈ {3000, 6000}` at `δ = 0.9` |
| `bridging-lambda` | 48 | 200 | `n ∈ {100, 200, 800}` × `share ∈ {0.6, 0.8}` × `(λ_b, λ_f)`: `λ_b ∈ {0.05, 0.15, 0.45}`, `λ_f ∈ {0.01, 0.03, 0.09}`, `λ_b > λ_f` (eight pairs, the production one among them); 5 ratings each, noise 0.15 |
| `bridging-extra` | 12 | 200 | `n ∈ {100, 200, 800}` × `share ∈ {0.5, 0.8}` × first panel `∈ {7, 11}` |
| `floor-screen` | 18 | 200 | `n ∈ {300, 600, 1500}` × anchors `∈ {30, 60}` × `m ∈ {2, 4, 5}` |

19,700 runs, and `floor-screen`'s 3,600 since T25's second step. `floor-power` and
`floor-misspec` have 100 replicates, not 200: their fits are the dearest of the grid, and an interval of about ±10 points places `N_LATENT_MIN`
and sizes a failure; `floor-null` keeps 200, whose "never" bounds a rate at 1.9%. The
grid was sized on two pre-checks. Two leaning items at `δ = 0.9` under a floor of 0.2
were found in 1 of 8 batches at `N = 3,000` (`docs/08` DIF-008), 4 of 8 at 6,000 and 4 of
4 at 12,000, no clean item flagged. One replicate of every cell then selected a mixture
and flagged clean items on the null batches with a skew of −4 (five items of eight) and
with templates at `tl = 1.0` (four), and not at skew 4 or `tl = 0.5`: the skew of −2 and
the two populations without a floor were added to size it and to tell whether the floor
is its cause. One replicate of each says it is not: without a floor, the skew of −4
flagged one clean item and the templates six of eight, where T24's populations, normal
and locally independent, flagged none.

`floor-screen`'s pre-check, six replicates of every cell in the container, found a defect
of the screen before its run: the fit did not converge, and kept no item, in 31 of the 108
pilots — 27 of the 36 with five options, where the item keyed backwards, fitted with the
rest, ran its slope to 0 and its difficulty past 100. Step 2 then left an item below
`R_PBIS_MIN` out of the fit (`docs/02` §B.2); on the same pilots 106 fits of 108
converged, the other two at 300 respondents and 60 true/false anchors, where an anchor's
slope and floor ran off.

**What a run records.** As §4, and: a floor study's DIF record adds each trial item's
fitted floor and the anchors' mean fitted floor (`floors`, `anchor_floor`); a screen run
records whether the fit converged and per item its fitted `a`, `b`, `c` (NaN if left out
of the fit), point-biserial, verdict and kind; `floor-dtf`'s true curves carry the drawn
floors (`ClassCurves::with_floors`). An extra-round run
records per probe its truth, first-round robust score and gate code, and per `k_extra`
the re-decision — `P`, `A`, `R`, or `-` where the probe was not re-decided.

### 8.4 Statistics

As §5, and:

- **Floors.** The floor tables give the design's floor, the items' mean fitted floor with
  its spread, and the anchors' mean fitted floor.
- **The DIF cut with a floor** (`thresholds-dif-cut-floor.csv`): per floor and format, the
  clean-item rate on the null batches of `floor-null` with 40 or 60 anchors — admitted or
  not, since the KR-20 floor refuses most of them and step 4 revisits it — and the power
  of the `floor-power` cells with two leaning items of eight, `π = 0.5` and 60 anchors.
- **The screen.** Per cell and item kind, the share kept, with the intervals of §5 — over
  runs for a kind of one item, over items grouped by run for the good and the flat ones:
  the good items' is the screen's specificity, each bad kind's its miss rate. Per item
  (`items.csv`), the share of runs it entered the fit, the share kept, the mean and spread
  of its fitted `a`, `b`, `c` and its mean point-biserial. The thresholds
  (`thresholds-screen.csv`): per stage-1 size and format, over anchors, the good items
  dropped and each bad kind kept as one threshold moves — `A_MIN` 0.4–0.8, `C_EXCESS_MAX`
  0.05–0.25, `B_ABS_MAX` 2.0–3.0 — the others at their production values.
- **The extra round.** For `ε ∈ {0.02, 0.04, 0.06}` and each `k_extra`, a probe passes if
  it is covered and its robust score is at or above `τ + ε`, or if it is in the band — or
  uncovered — and its re-decision passes. False passes are over the probes whose truth is
  at least 0.02 below `τ`, false failures over those at least 0.02 above it, both over
  items grouped by run (§5); the table gives the band's share, and the CSV the extra
  reviews it costs per probe, `k_extra` times that share.

### 8.5 Cost and running

Measured in the release profile on the development container, one run per cell, four at
a time:

| Study | Runs | Time of one run: mean, range |
|---|---|---|
| `floor-null` | 2,400 | 50 s, 19–100 s — 24–46 s at N = 3,000, 60–100 s at 6,000 |
| `floor-power` | 2,300 | 119 s, 24–433 s — up to 433 s at N = 12,000, where a mixture is found |
| `floor-misspec` | 1,800 | 87 s, 25–278 s — the spurious mixtures of a skew of −4 are the dearest |
| `floor-dtf` | 1,200 | 91 s, 33–215 s |
| `bridging-lambda` | 9,600 | 1.2 s |
| `bridging-extra` | 2,400 | 13 s |
| `floor-screen` | 3,600 | 0.23 s, 0.02–2.0 s — in all 3.5 minutes in the container (§8.7.5) |

About 195 CPU-hours of the container in all, 76 of them in `floor-power`: at the ratio
T24 measured (§7: 425 of them in about 30 hours), about 14 hours on the owner's machine,
and the first pass (`--replicates 20`) about 2. The container, whose 4 cores take three to
four times as long, runs the smoke grid, single cells and the pre-checks.

**On the owner's machine**, on the branch or commit that carries this section:

1. `cargo run --release -p characterization -- run --grid smoke --study t25 --out smoke-t25`:
   about a minute, a check of the build and the machine.
2. `cargo run --release -p characterization -- run --study t25 --replicates 20 --out characterization-t25`,
   then look at `characterization-t25/summary.md`.
3. `cargo run --release -p characterization -- run --study t25 --out characterization-t25`
   for the rest; stop and restart it freely.
4. Send back `summary.md`, the `summary.csv` files, `thresholds-dif-cut-floor.csv` and
   `bridging-lambda/tau.csv`, with the commit and the machine.

`--study t25` names the seven studies of this section, `--study t24` the ten of §4. A
separate `--out` keeps the supplement's summary to its own studies: `summarize` leaves
out the studies with no records.

### 8.6 Done when

1. Every run of §8.3 recorded on one commit of the harness, the last `run` with no
   `errors.log`, and `summarize` run on the records; T24's pinned records reproduce on
   that commit (`tests/harness.rs`).
2. §8.7 holds the results with the commit, the date, the machine and the wall time; the
   CSV tables are committed under `verification/reports/t25/`.
3. Each result stated in the form of `docs/07` §14, and `docs/08` restated: AT-DIF-13 in
   §12; DIF-008, STAT-001 and DIF-011 with a floor, BRIDGE-002 and BRIDGE-003 for
   `(λ_b, λ_f)`, BRIDGE-006 for `k_extra`, in §15.
4. The screen's study added once step 2 is done, run and stated (§8.1): done on
   2026-09-30 (§8.7.5).
5. T25's step 4 is then unblocked: `N_LATENT_MIN`, the DIF cut, `KR20_MIN` or a minimum
   number of anchors and `DTF_MAX` for items that guess; `k_extra` and `ε`; `λ_b/λ_f`.

### 8.7 Results

**The run.** 19,700 runs, every run of §8.3 recorded, no `errors.log`, on the owner's
machine: the first pass (`--replicates 20`) on 2026-09-29 and the rest by 2026-09-30, about
10 hours, on the harness of `a9cdafa` — every later commit keeps what the runs compute, a
record of each kind pinned (§6). The summary and every study's table, with their provenance,
are in `verification/reports/t25/`; the records stay with the owner.

**What every statement below assumes.** The populations of §3.1 and §8.2 and nothing else:
two latent classes on an axis the model never sees, every item and anchor with the floor the
cell draws, declared with the cell's format, a normal ability in each class unless a
statement says otherwise, trial items answered independently given ability unless a
statement names templates; reviewers in two camps rating by a linear model, probes rated by
panels drawn as production draws them. A rate is over its cell's runs or items with the 95%
interval of §5; "never" is 0 of the cell's runs, an upper bound of 1.9% over 200 runs and
3.7% over 100. Outside these populations the behaviour is not established (`docs/07` §14).

#### 8.7.1 Latent DIF with a guessing floor

- **Specificity** (`floor-null`, AT-DIF-13, DIF-008). On null batches whose every item and
  anchor has a floor — five options (0.2), four (0.25), true/false (0.5), floors varying by
  ±0.1 around the declared 0.2 — N 3,000–6,000, 20–60 anchors, the corrected model never
  selected a mixture and flagged no clean item: 0 of 2,400 batches, 0 of 19,200 clean items,
  at most 1.9% in any cell. The fitted floors are the drawn ones: 0.201–0.204 for 0.2,
  0.251–0.254 for 0.25, 0.500–0.501 for 0.5, with a spread of 0.03–0.06 from item to item.
- **The KR-20 floor.** With 60 anchors the KR-20 is 0.894 with five options, 0.880 with four
  and 0.785–0.787 with true/false; with 40 anchors 0.848 and 0.711, with 20 anchors 0.737 and
  0.546. `KR20_MIN` = 0.90 admits 12.5–16.5% of the null batches of five options with 60
  anchors and none of the others, and 7–17% of the five-option power batches with 60
  anchors — batches that flag no clean item at any of these reliabilities. With 40 anchors
  two leaners at δ = 0.9, N = 6,000, are found in 67.5% [57.8, 75.9] of cases, against
  70.0% [60.4, 78.1] with 60; three in 100% with both.
- **Sensitivity** (`floor-power`, STAT-001): the share of leaning items flagged, `K` = 8,
  60 anchors, `π` = 0.5; T24 found two leaners at δ = 0.9 in 92.8% and 98.8% of the
  batches that do not guess at N = 3,000 and 6,000 (§7.1).

  | Format | Leaning items, δ | N = 3,000 | N = 6,000 | N = 12,000 |
  |---|---|---|---|---|
  | five options (0.2) | 2, 0.9 | 9.0% [4.8, 16.2] | 70.0% [60.4, 78.1] | 97.0% [92.2, 98.9] |
  | | 2, 0.7 | | 2.0% [0.6, 7.0] | 28.0% [20.4, 37.2] |
  | | 3, 0.9 | 96.0% [90.2, 98.4] | 100% [96.3, 100] | |
  | | 3, 0.7 | 22.7% [15.6, 31.8] | 81.0% [72.4, 87.4] | |
  | four options (0.25) | 2, 0.9 | | 39.0% [30.0, 48.8] | 90.0% [82.6, 94.5] |
  | true/false (0.5) | 2, 0.9 | never | never | 5.0% [2.2, 11.2] |
  | | 2, 0.7 | | never | never |
  | | 3, 0.9 | never | 31.0% [22.8, 40.6] | |
  | | 3, 0.7 | never | never | |

  A smaller class costs little at N = 12,000: two leaners at δ = 0.9 with `π` = 0.3 are
  found in 92.0% [85.0, 95.9] of cases. No clean item is flagged in any cell but one, 0.2%
  (three leaners at δ = 0.7, N = 3,000). Where most fits find the mixture the gap is read at
  its scale — 1.78–1.87 for 1.80, 1.45 for 1.40 — and inflated where few do: 2.15 at 9%,
  1.47–1.57 for 1.40 at 2–28%, 2.00–2.09 with true/false.
- **The DIF cut** (`thresholds-dif-cut-floor.csv`). No clean item is flagged at any cut from
  0.5 to 1.5 on the nulls with 40 or 60 anchors (0.0% [0.0, 0.6] with five options and with
  true/false, [0.0, 1.0] with four). Below 1.0 the cut barely binds: at 0.8, five options,
  N = 12,000, two leaners are found in 98.0% of cases at δ = 0.9 and 31.0% at δ = 0.7,
  against 97.0% and 28.0% at 1.0; elsewhere nothing moves. At 1.5 the same cells fall to
  79.0% and 14.5%.

#### 8.7.2 Misspecification

`floor-misspec`, five options, N = 3,000, 60 anchors, `π` = 0.5, none or three leaning
items at δ = 0.9:

- **Absorbed.** Impact 1.0 flags no clean item and costs power (85.0% [76.7, 90.7] of three
  leaners, against 96.0% without it); so do a skew of −2 or +4 (97.0% and 95.0%, one batch
  in 100 with a clean item flagged), templates at `tl` = 0.5, floors of 0.1 or 0.3 declared
  as five options (99.0% and 68.0%) and floors varying by ±0.1 (96.0%). A floor off the
  declared chance is read toward it — 0.1 at 0.14, 0.3 at 0.25–0.26 — and a skew biases the
  fitted floors, 0.26–0.27 at +4 and 0.18–0.20 at −2 and −4, without flags.
- **Read as bias** (T82). A skew of −4 makes the model select classes in 60% of the null
  batches and flag 22.4% [18.1, 27.3] of their clean items, in 56.0% [46.2, 65.3] of the
  batches; with three leaners, 31.0% of the clean items in 77.0% of the batches, the leaners'
  gap read at 3.64 for 1.80. Templates at `tl` = 1.0 do it in 41% of the null batches:
  11.2% [8.7, 14.4] of the clean items, 41.0% [31.9, 50.8] of the batches; with three
  leaners, 7.8% of the clean items in 21.0% of the batches. On T24's populations — no floor,
  every column open — it is not better: a skew of −4 flags 12.5% [9.3, 16.7] of the clean
  items in 52.0% of the batches, templates at 1.0 flag 73.0% [69.8, 76.0] of them in every
  batch.

#### 8.7.3 The DTF bound with a floor

`floor-dtf`, the mirror layout, over the fits that find two or more classes (§5): with five
options in every fit at δ = 0.9 and in 1 of 200 and 53 of 200 at δ = 0.5 (N = 3,000 and
6,000); with true/false at δ = 0.9 in 83 and 189 of 200.

- **Bias.** As in §7.3, the fitted DTF is biased up where the true one is small: the four
  clean items are fitted at 0.029–0.046 on average with five options and 0.036–0.048 with
  true/false, and refused in 0.5–9.4% and 3.7–10.8% of the fits.
- **Far above `DTF_MAX`** a set is admitted almost never: a single leaner (true DTF
  0.18–0.31) in 3 fits of 1,452, a same-side pair (0.36–0.61) never.
- **Near it** — the mirror pairs, the four leaners, a pair with two clean items, true DTF
  0.05–0.12 on average — the decision is noisy both ways. With five options up to 13% of
  the sets truly over the tolerance are admitted; with true/false, at N = 6,000, 17–33%. Of
  the sets truly within, five options refuse 9–31% at N = 6,000, δ = 0.9, 28–59% at
  N = 3,000 and 27–61% at δ = 0.5; true/false refuses 14–45% at N = 6,000 and 41–74% at
  N = 3,000.

#### 8.7.4 Bridging: `(λ_b, λ_f)` and the band's extra round

- **`(λ_b, λ_f)`** (`bridging-lambda`, SC-3, BRIDGE-002, BRIDGE-003), the mirror design of
  §3.2, 100–800 reviewers, camps 60/40 and 80/20, 5 ratings each, noise 0.15, the eight
  pairs within a factor of 3 of the production (0.15, 0.03). No partisan item passes but
  0.05% in two cells (100 reviewers, 80/20); a consensus item 0.05 or more below `τ` passes
  in at most 0.25%; one 0.05 or more above passes in 62–69% of cases with 100 reviewers in
  80/20 camps, 78–84% in 60/40, 79–86% and 91–95% with 200, 97–100% with 800 — within a cell,
  the pair moves it by at most 7 points, the production pair inside the range. `λ_f` sets
  the axis's recovery: `|corr|` 0.80–0.91 on average at 0.01, 0.88–0.93 at 0.03, 0.92–0.95
  at 0.09, and the worst run falls to 0.36 at 0.01 against 0.53 at 0.03 and 0.72 at 0.09.
  `λ_b` moves nothing measurable.
- **The band's extra round** (`bridging-extra`, BRIDGE-006, PROTO-008). On probes within
  0.06 of `τ`, each rated by a first panel of 7 or 11 reviewers, noise 0.15, over 100–800
  reviewers in camps of 50/50 and 80/20: at the production `ε` = 0.02 and `k_extra` = 4,
  6.4–11.7% of the probes 0.02–0.06 below `τ` pass and 39.9–66.5% of those 0.02–0.06 above
  fail. The extra round lowers the false passes — from 8.7–14.1% with no extra reviewer to
  4.5–8.0% with 11 — and leaves the false failures where they are (38.3–64.6% with none,
  40.3–65.9% with 11): they are decided in the first round, below the band. A band of ±0.06
  lowers them by 12–16 points (23.5–52.9% with no extra reviewer, 20.9–51.8% with 11) and
  raises the false passes unless the extra round is large (16.1–22.1% with none, 7.1–12.1%
  with 11), at 3.8–6.1 extra reviews per probe with 11, against 0.5–0.9 at the production
  setting. A first panel of 11 instead of 7 lowers the false failures by 4–12 points; camps
  of 80/20 raise them by 9–18. The number of reviewers in the epoch changes little: a probe's
  score rests on its panel.

#### 8.7.5 The pilot's stage-1 screen

**The run.** `floor-screen`'s 3,600 runs, no `errors.log`, in the container on 2026-09-30,
3 min 26 s with four workers, on the harness of `ec34fa4`: the stage 1 of T25's second
step, the point-biserial first. Its tables are in `verification/reports/t25/`
(`floor-screen-summary.csv`, `floor-screen-items.csv`, `thresholds-screen.csv`).

- **Keyed backwards, too hard.** The item keyed backwards was never kept: 0 of 3,600, at
  most 1.9% in any cell. The item too hard was kept in 10 of 3,600, at most 3.0%
  [1.4, 6.4] in a cell (N = 300, five options). The point-biserial drops both before the
  fit: its mean is −0.22 to −0.37 for the first and 0.03–0.10 for the second.
- **Flat items** were kept in 4.2% of cases: 5.5% with four options, 6.1% with five, 0.9%
  with true/false; 10.5–10.8% at N = 300 with four or five options, 1.4–1.9% at 1,500.
  Most of the kept ones are the `a = 0.45` item, whose fitted slope reads 0.6–0.7 with
  four or five options when the point-biserial lets it into the fit.
- **Good items.** Four of the five were kept in 93.3% of cases with four options and
  95.5% with five, 89.6–97.6% by size. The fifth, `(1.2, 2)`, was kept in 16.6% and
  29.8%. With true/false, the four were kept in 42.0% and the fifth in 1.1%. A larger
  sample does not help: the point-biserial, not the fit, drops them. Its mean on the
  anchors' total is a population value under `R_PBIS_MIN` = 0.20: 0.15–0.19 for
  `(1.2, 2)` with four or five options; with true/false 0.16–0.18 for `(0.8, 0)` and
  `(1.2, 1)`, 0.19–0.21 for `(1.6, 0.9)` and 0.09–0.10 for `(1.2, 2)`. At N = 1,500 the
  true/false pilots keep `(0.8, 0)` and `(1.2, 1)` in 4–21% of cases.
- **The guessable item**, its floor 0.2 over chance, was kept in 79.2% of cases with four
  options, 83.8% with five, and 16.4% with true/false, where the point-biserial drops it
  with the good ones. The fit reads it as an easier item: with four options its floor
  reads 0.27–0.34 against a true 0.45, its difficulty −0.4 to −0.6 against 0. A larger
  sample barely helps: at 1,500 it is still kept in 65.8% and 70.8% of cases.
  `C_EXCESS_MAX` at 0.05 still keeps 40–80% of them, and drops 23–30% of the good items.
- **The thresholds.** `B_ABS_MAX` changes nothing between 2.0 and 3.0: the point-biserial
  has already dropped the items it would drop. Moving `A_MIN` from 0.4 to 0.8 trades flat
  items kept against good items dropped: at N = 300 with five options, from 15.1% and
  18.4% to 3.1% and 27.0%.
- **Convergence.** 98.9% of the fits converged: all of them with five options, 99.8%
  with four, 96.9% with true/false. With true/false at N = 300 and 60 anchors, only 85.5%
  converged: an anchor's slope and floor run off. A pilot whose fit does not converge
  keeps nothing.

**What it means.** `R_PBIS_MIN` decides stage 1, and it is not calibrated for items that
guess. The floor lowers every item's point-biserial, so the screen drops a hard good item
in most pilots whatever the format, and most good true/false items at any size. `C_EXCESS_MAX` cannot
catch a floor 0.2 over chance at stage 1's sizes, because the fit reads the item as easier.
The re-check, at 3,000 respondents and more, is where floors are measured (§8.7.1). T25's
fourth step sets `R_PBIS_MIN` — by format, or lower, leaving the work to the fit's `a` —
and measures the chosen value with a new run of this study. The keyed-backwards item's
point-biserial is negative at every size, so any positive bar still drops it.

#### 8.7.6 What T25 takes from it

- **The model with a floor is specific** on every format tried, down to 20 anchors, and the
  floors it fits are the drawn ones: D25 does what it was for.
- **T82 first.** Templates answered alike and a strongly left-skewed ability are read as
  bias, with a floor or without; the DIF thresholds wait on it (`docs/10` T82).
- **`KR20_MIN` gives way to a minimum number of anchors.** With a floor the KR-20 is
  0.55–0.89 on batches that flag no clean item, and the floor refuses 83–100% of the batches
  of choice items; 40 anchors cost no power measured here.
- **`N_LATENT_MIN` depends on the format.** Two leaners at δ = 0.9 need about 12,000
  respondents with four or five options (90–97%; 6,000 gives 39–70%), where open answers
  need 3,000 (§7.1); three leaners are found from 3,000 with five options. With true/false
  the re-check does not see two leaners up to 12,000, and three only at 6,000 in 31%:
  true/false batches need a policy of their own.
- **The DIF cut** can stay at 1.0: below it nothing gains more than 3 points, above it
  power falls.
- **`DTF_MAX`** as in §7.5: the bias on clean sets is 0.03–0.05 with a floor, and with
  true/false the sets near the tolerance are admitted or refused more often in error.
- **`(λ_b, λ_f)`** can stay; `λ_f` should not go below 0.03.
- **The pilot's stage 1 needs its own `R_PBIS_MIN`** (§8.7.5). At 0.20 it drops most good
  true/false items and most hard good items. `A_MIN` and `B_ABS_MAX` barely matter once
  it has run. `C_EXCESS_MAX` cannot see a guessable item at stage 1's sizes, so the
  floor is judged at the re-check.
- **`τ`, `ε`, `k_extra` and the panel's size go together.** On items rated by a panel of
  production size the robust score is noisy and biased low near `τ`: the extra round only
  lowers the false passes, a wider band the false failures, a larger panel both. T24's
  statements on the gate (§7.4) hold for items rated by 25–360 reviewers, not for these.

#### 8.7.7 First pass (2026-09-29)

`--replicates 20` — 2,380 runs, twenty per cell — ran first, on the same harness, and its
records are the first twenty of every cell of the full run. With intervals of ±15–20 points
it already showed what §8.7.1 states, and found the misspecification of §8.7.2, which T82
records; no cell was added after it.

### 8.8 After T82: the DIF studies again

D43 (T82) gave the target model an estimated ability histogram and the batch gate a
template rule, the day the full run ended. Every DIF, DTF and floor record of §7 and §8.7
comes from the model before it; its bridging records are unchanged. So the four DIF
studies of the supplement run again on the corrected model, same grid and seeds — the
measurements T25's fourth step reads, since every item the system asks guesses:

- **What changes.** Each class's ability is the shared histogram shifted by its mean,
  estimated with the rest (`docs/02` §B.3). The pinned DIF records of `tests/harness.rs`
  change on purpose; the bridging ones reproduce. `admitted` is false for the `tl` cells
  (§8.2). A fit with a floor costs about half as much again as before (T82, `docs/10`).
- **Power, at the margin.** The one-class fit now takes the sample's departures from a
  normal, which a second class used to take as well, so a real mixture can gain less over
  one class. On the golden batch of 1,500 respondents and 20 anchors with two leaners at
  δ = 0.9, the previous model selected two classes with a BIC gain of 2.4 and the
  corrected one selects one; on its batch with a floor the gain fell from 27.5 to 24.1,
  the leaners flagged as before. A paired pre-check on the supplement's own batches, in the
  container, found no such loss where the tables are read: on eight replicates of three
  leaners at δ = 0.9, N = 3,000, five options, and eight of two leaners at N = 6,000, both
  models find the same batches — all eight, and the same five of eight — their BIC gains
  within 5 of each other, no clean item flagged.
- **What it must show.** The skewed and template cells flag clean items in no more null
  batches than the clean populations do — for templates, of a template the batch did not
  declare, since a declared one is refused. The other cells stay as §8.7 states them, or
  the difference is stated: the power tables and `N_LATENT_MIN` are read from this run.
- **On the owner's machine**, on the commit that carries this section, into a new
  directory, since the harness resumes from the records it finds:
  `cargo run --release -p characterization -- run --study floor-null,floor-power,floor-misspec,floor-dtf --out characterization-t25-d43`,
  a first pass with `--replicates 20` first. T24's DIF studies are not run again: they
  describe the model before D43 on populations that do not guess.
