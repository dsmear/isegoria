# Scoring engine — mathematical specification

The engine is **deterministic**: given the same input (node×question ratings,
respondent×question answers), it produces the same output. The Rust implementation
has scoped Python oracles and deterministic golden-output tests; `sim/` also retains
retired models and is not a complete executable specification of this version.

**Reading status.** This document distinguishes implemented formulas from intended
guarantees. Open disagreements and correction criteria are in
[`15-phase1-review.md`](15-phase1-review.md). Documentation alignment does not resolve
those findings or change the code. Measurement claims retain their model/version scope.

Notation: `u,v` nodes/reviewers; `j` questions (items); `i` respondents; `θ_i` the
respondent's latent competence.

> **Revisions from the working paper (decided 2026-09-24).** The working paper
> (`paper/`, version 0.2) found properties of this specification that `docs/01` D32–D41
> correct; the tasks are `docs/10` Phase 1.1 (T49–T57), the first phase of the roadmap.
> D32–D40 are implemented (T49–T57, 2026-09-24/25): the text describes their formulas
> and intended contracts, subject to the open findings in `15`, with measured limits —
> the side-balanced score and
> the appeal by the side gap (§A.3), the target latent model and its anchor precondition
> (§B.3), the contested-facts pool (§B.5, §B.7), the evaluator score with exploration
> (§C.2), the change detector and the cap (§C.4), coordination on residuals and panel
> diversification (anti-collusion). D41, the beacon, is done too (`10` T37). `paper/README.md` lists
> what changed since the paper's snapshot.

---

## Level A — Bridging consensus

### A.1 Model

Matrix factorization with asymmetric regularization. Let `r_uj ∈ [0,1]` be node `u`'s
judgment of question `j`:

```
r̂_uj = μ + b_u + b_j + ⟨f_u , f_j⟩          f ∈ ℝ^d,  d = 1 or 2
```

- `μ` : global mean
- `b_u` : reviewer bias (individual severity/generosity)
- `b_j` : question intercept, reported by the fit; the score the gate reads is the
  side-balanced approval of §A.3 (D32)
- `f_u` : reviewer latent position (ideological axis, discovered from the data)
- `f_j` : how much the question "speaks" to that axis

Objective function, over `Ω` = the set of observed judgments:

```
L = Σ_(u,j)∈Ω (r_uj − r̂_uj)²  +  λ_b (Σ b_u² + Σ b_j²)  +  λ_f (Σ‖f_u‖² + Σ‖f_j‖²)
```

**with `λ_b ≫ λ_f`** — indicatively `λ_b = 0.15`, `λ_f = 0.03`.

### A.2 Why asymmetric regularization is the trick

By penalizing the intercepts heavily, the model is forced to explain approval *first*
through the polarization factors `⟨f_u,f_j⟩`. Only approval that **cannot** be
explained as "my faction likes it" survives in `b_j`.

- A question that one camp likes a lot → large `f_j`: the two sides' predicted approval
  differs and its mean stays low → **discarded**.
- A question approved by reviewers with opposite-sign `f_u` → both sides' predicted
  approval is high → **passes**.

### A.3 Score and threshold

**Bridge score (D32, T49; D42, T71): the side-balanced predicted approval.** After the
fit, the reviewers are split into two sides by the exact one-dimensional 2-means of
`f_u`: of the cuts of the sorted positions that split no run of equal values and leave
each side at least 5% of the reviewers (rounded up; `MIN_SIDE_PER_MILLE = 50`,
provisional, T25), the one with the largest between-side sum of squares. For each
question the model's predicted ratings `r̂_uj` — every reviewer's, whether or not they
rated it — are clipped to [0, 1] and averaged within each side, `A_j` and `B_j`, and the
score is

```
S_j = (A_j + B_j) / 2
```

so each side counts once, whatever its size, and the score lives on the scale of the
declared probability. Accepted if `S_j ≥ τ`, with `τ ≈ 0.80` **provisional**: on the
reference simulation the consensus items score 0.83–0.86, the partisan items 0.53–0.56
and the mildly partisan one 0.70 (`sim/bridging_irt_dif.py`). Calibrate on the pilot
(T25).

*Why not the intercept.* `b_j` is relative to its batch (`Σ_j b_j = 0` at every
stationary point, with `μ` unpenalized) and partly majoritarian: its origin is a gauge
fixed only by the penalties, which at the default `λ_b / λ_f` leaves 53–87% of the
camp-size effect in the score (paper §3.3–3.4, `docs/08` BRIDGE-008/009). The
predictions depend on neither. On mirror-image partisan items the side-balanced score
leaks at most 0.1 of the camp-size effect from 200 reviewers up (`AT-BR-08`); with
50–100 reviewers — a handful of minority ratings per item — the fit shrinks `f` and a
residual leak of 0.1–0.2 remains, a fraction of the intercept's. With a minority side of
about ten reviewers the side means are noisy: on the review's dataset at 95/5 one
consensus item in eight fell to 0.78. A floor on the minority side is a calibration
item (T25).

*The sides and the coverage (D42, T71).* The first pass of the characterization (`13`)
found three weaknesses in the score as T49 first built it, each confirmed on the engine's
own fits. The 2-means iteration started from the extremes of `f_u` stopped at a local
optimum when a few reviewers sat far out, and made them a side of their own: sides of 2
and 5 reviewers of 800, and in a bootstrap subsample 1 of 200, which dropped an item's
robust score from 0.91 to 0.56 while its full fit stayed at 0.91. An item that no
reviewer of one side had rated got that side's mean by extrapolation: a partisan item,
camp-balanced value 0.54, passed at 0.972 with no minority rating. Unclipped
predictions put scores outside [0, 1] (−0.28 to 1.31). The exact cut replaces the
iteration: it does not depend on the axis' origin or scale, a sign flip swaps the sides,
and two camps of distinct positions are separated whenever each holds at least 5% of the
reviewers. The predictions are clipped. And an item's **coverage** — the ratings its
less-rated side gave it, counting the axis reviewers with a positive weight — must reach
`MIN_COVERAGE = 1` (provisional, T25): below it the gate sends the item to the band's
extra round whatever its score, and the re-decision cannot pass it (`05` [5]). The panels
of `05` [4] are stratified on `f_u`, so an item one side never rated is rare; the floor
keeps it from being decided on an extrapolation when it happens.

**Polarization.** The gap `|A_j − B_j|` between the two sides is the question's
polarization. It feeds the appeal rule of `docs/05` [5b] — a question rejected with a
wide gap was rejected for polarization, not for a defect — with a provisional threshold
of 0.25, between the reference simulation's consensus items (≤ 0.02) and its mildly
partisan one (0.30). It replaces `|f_j|`, which *falls* as the camps become unequal
(`docs/08` §0-quinquies, BRIDGE-009).

**Uncertainty band (correction from testing).** Scores cluster near the threshold:
questions separated by thousandths end up one inside and one outside for pure noise.
Do not use a hard cut: define a band `[τ−ε, τ+ε]` in which questions go to
supplementary review instead of being decided by the exact value: an extra round of
`k_extra` reviewers drawn outside the first panel (`k_extra = 4`, provisional), whose
ratings are added to the first panel's before the score is re-decided against the plain
`τ` (`01` D26, T60; `05` [5]). `ε ≈ 0.02` provisional, about three times the bootstrap
spread of `S_j` on the reference fixtures (≤ 0.006); to be tuned with `k_extra`, `τ` and
the panel's size (T25 step 4): on items rated by a panel of 7 or 11 the extra round lowers
the false passes near `τ`, not the false failures (`13` §8.7.4).

*History.* Until T49 the gate read the intercept `b_j` against `τ ≈ 0.08` with
`ε ≈ 0.008`: the Community Notes reference value (0.40, on binary votes) proved
inadequate at this scale, and the useful value on the intercept was around 0.08.

### A.4 Robustness

- The objective is non-convex and has distinct local minima: a single start can land in
  a worse one, sometimes on the other side of `τ`. Fit from several deterministic
  starts (8 in the reference implementation) and keep the lowest objective; report `f`
  in a canonical sign (it is identified only up to sign).
- Run the fit on `m = 10` bootstrap subsamples (random removal of ~15% of judgments)
  and take the minimum of the side-balanced score over them, `min_s S_j^(s)`
  (pessimistic estimate): a question must pass in all repetitions.
- **`d = 2` is descoped** (`01` D31, recorded here by T39): the model has one latent
  axis. A second dimension — if a deployment shows one fracture axis is not enough (e.g.
  right/left + urban/rural), chosen empirically by explained variance on historical data
  — is a future option, not a plan. Community Notes essentially bridges on a binary
  axis; `d = 2` would be the generalization.
- **Reviewer floor (T39).** Nodes with fewer than `n_min = 30` reviews on record do not
  contribute to defining the `f` space, only to filling it: they are absent from the
  core fit — the axis `f_j`, the levels `b_j` and everyone else's position are exactly
  those of the fit without them — and are then *placed* on the fixed axis by ridge least
  squares over their own ratings (`Ratings::axis`, `orchestrator::axis_mask`,
  `N_MIN_REVIEWS`): a position of their own, no influence on anyone else, and no side in
  the side-balanced score. (Pinning their `f_u` at 0 instead would not do: the model is
  invariant to `(f_u, b_j) → (f_u + c, b_j − c·f_j)`, so a single pinned reviewer with a
  few extreme ratings drags the whole axis's origin to itself.) A founder defines the
  axis from the start (the founder set is declared heterogeneous, `05` §Cold start),
  otherwise the first epochs would have no axis. Until it is established a newcomer
  weighs 0 anyway (`01` D36) and is assigned from the position it has (`05` [4]).
  A reviewer drawn into the band's extra round with no row in the epoch's ratings gets
  the row its standing gives, as the epoch's rows do: on the axis only as a founder or
  past the floor, at its review weight (`orchestrator::expanded_ratings`; `15` A5).
  Provisional (T25).

### A.5 Optimization

L-BFGS-B with an analytic gradient. The gradient with respect to each parameter block
is in the prototype (`sim/bridging_irt_dif.py`, function `fit`). Attention point for
reproducibility: fix the initialization seed and the iteration order. The transcendental
functions (`exp`, `ln`, `ln_1p`, `cos`, `pow`) come from one pure-Rust implementation
(the `libm` crate, `scoring::fmath`), not from the platform's libm, so the same input
gives the same bits on every platform and in every build profile (INV-7, `docs/08`
AT-BR-04).

---

## Level B — Empirical validation

Here nobody votes. You measure, on the pilot data. **Always in batches of questions,
never on a single item** (see the latent-class DIF section and
`sim/latent_dif_and_capacity.py`).

### B.1 IRT model

3-parameter model (3PL), the floor `c_j` set by the item's declared format (`01` D25):

```
P(X_ij = 1 | θ_i) = c_j + (1 − c_j) · [ 1 + exp(−a_j (θ_i − b_j)) ]⁻¹
```

- `a_j` : **discrimination** — how well the question separates those who know from
  those who don't
- `b_j` : **difficulty**
- `c_j` : **pseudo-guessing** — the chance that someone who knows nothing answers right

**Formats.** Every item, anchor or trial, declares its format, and the format sets `c_j`:

- an **open answer** — the respondent gives the datum, which the template's key rule
  checks (§B.5) — has nothing to guess: `c_j = 0`, the 2PL;
- a **choice among `m` options, one of them keyed** — true/false is `m = 2` — has a floor
  near the chance level `1/m`. The floor is estimated with the other parameters, under a
  prior with its mode at `1/m` and the weight of `w` pseudo-observations
  (`w = 20`, provisional, T25):

  ```
  ln p(c) = w · [ (1/m) · ln c + (1 − 1/m) · ln(1 − c) ]        (a Beta(1 + w/m, 1 + w(1 − 1/m)))
  ```

  The floor is not fixed at `1/m`: distractors pull more or less than chance, item by
  item, and the latent-class model (§B.3) reads an item's misfit at low ability as a class
  that finds the item easier — a false DIF on that item. No format with more than one keyed
  option is specified.

Without the floor, the latent re-check fitted to a population that guesses — a floor of
0.2 on every item — selects a mixture in 72.5–98.0% of the null batches and flags 12–17%
of the clean items (T24, `13` §7.2); every item the system asks is true/false or a choice
among options. Since the ability's shape is estimated (§B.3, `01` D43) the same batches
declared open select one class, the histogram taking the guessing as a right skew, but
the metric bends: on four seeds the anchors' mean discrimination falls to 1.0 against 1.3
with the floors (drawn around 1.25), and the ability reads skewed by +0.55 to +0.80. The
floor keeps the metric the thresholds are expressed in.

**The metric.** Every threshold on `a`, `b` and the DIF gap is in the metric of the target
model (§B.3): logistic with no scaling constant (`D = 1`), `θ` of unit variance within each
latent class, its origin the mean of the first class — in a one-class fit, of the batch's
respondents — and its shape estimated, the same in every class (§B.3, `01` D43). The
metric is the batch's own: nothing links the metrics of two fits (§B.7).
The pilot's stage 1 reads the same model with one class (§B.2, T25's second step), so its
thresholds are in this metric too.

### B.2 Retention criteria

| Statistic | Threshold | Meaning of failure |
|---|---|---|
| `a_j` | ≥ 0.6 | the question distinguishes nothing: it is noise |
| `b_j` | −2.5 ≤ b_j ≤ 2.5 | too easy or too hard to be informative |
| `c_j` | ≤ `1/m_j` + 0.10 (0.35 with four options) | answer is guessable |
| Infit/Outfit MNSQ | 0.7 – 1.3 | the question is not coherent with the construct |
| `r_pbis` (point-biserial) | ≥ 0.20 | same, classical version |

**Stage 1 reads the target model (`01` D25, T25's second step).** The pilot's first stage
drops an item whose `r_pbis` against the anchors' total is below `R_PBIS_MIN`, then fits
the target model of §B.3 with one class on the pilot's anchors and the other items, each
column declared with its format, and keeps an item when the fit converged, `a ≥ A_MIN`,
`|b| ≤ B_ABS_MAX` and a choice item's floor is at most `C_EXCESS_MAX` = 0.10 (provisional)
above its chance level `1/m`. The point-biserial comes first because the model's curves
rise with ability: fitted with the rest, an item keyed backwards ran its slope to 0 and
its difficulty to 100 and beyond, and the fit, which never converged, kept no item of the
pilot — in 13 of 16 simulated pilots of 300 with five options, and in most of the
screen's pre-check pilots of five options at every size (`13` §8). At the
pilot's ~300 respondents the ability's shape is held at the normal, the metric's variance
and origin unchanged: on four simulated pilots of that size the histogram of D43 did not
converge twice and traded slope for floor on good items (`a` 10–12, `c` 0.37), where the
normal converged every time. The infit/outfit row above is not implemented. Before, stage
1 read a 2PL slope on the standardized anchor total, a proxy metric in which the fixture's
item 02 — `a = 1.6` with a floor of 0.25, like every item of the reference sim — read 0.47
and failed; it now passes, and the fixture's pool is items 01, 02 and 07, as the sim finds.
The screen's study (`13` §8.7.5) measured this rule on simulated pilots. It drops wrong
keys and flat and too hard items. But `R_PBIS_MIN` also drops good items whose floor
lowers their point-biserial — most true/false items and most items near `b = 2` — and the
fit reads an item easier to guess than its format allows as an easier item, which
`C_EXCESS_MAX` does not catch at these sizes. T25's fourth step sets the thresholds.

**Point-biserial**: correlation between "correct answer to this item" (0/1) and total
score on the rest of the test. If **negative**, the answer key is almost always wrong
(those who know more get it wrong more) — in testing this correctly caught an item
with an inverted key.

### B.3 DIF analysis on latent groups — the neutrality test

Establishes whether a question is politically biased **without knowing anyone's
identity or attributes**.

Idea: two people with the *same* competence `θ` but from different groups must have
the same probability of answering correctly. If they don't, the question is measuring
group membership.

**Variant 1 — logistic regression on continuous `f`** (uses the Level A latent axis):

```
logit P(X_ij = 1) = β₀ + β₁ θ_i + β₂ f_i + β₃ (θ_i · f_i)
```

- `β₂ ≠ 0` → at equal competence, the question favors one end of the axis (uniform DIF)
- `β₃ ≠ 0` → the advantage varies with competence level (non-uniform DIF)

Operational threshold: `|β₂| > 0.40` → reject. In parallel, discretizing `f` into
tertiles, Mantel–Haenszel with ETS classification:

```
Δ_MH = −2.35 · ln(α_MH)
|Δ_MH| < 1.0        → class A   accepted
1.0 ≤ |Δ_MH| < 1.5  → class B   accepted with monitoring
|Δ_MH| ≥ 1.5        → class C   REJECTED
```

**Variant 2 — latent-class IRT mixture** (independent of Level A):

> **Revised by D37 and D38 (T53, T54 and T55 done).** Error in the ability proxy creates
> spurious latent classes (paper §4.5): the re-check runs only when the anchors' KR-20 on
> the batch's respondents is ≥ 0.90 (`irt::KR20_MIN`, enforced by `revalidate_batch_latent`
> — T53), and since T54 the fit is the target model below — the anchors inside the
> likelihood and `θ` integrated out (`scoring::latent`) — which finds no mixture where
> the proxy did. The differential gap of the retired proxy model is a diagnostic only
> (`MixtureDif::differential`): it inverts in a campaign. An item whose DIF concerns
> knowledge of a fact established by a primary source becomes a *contested fact* in a
> balanced pool instead of being rejected (D38, T55, done): §B.7 defines the pool, its DTF
> statistic and the balanced draw, §B.5 the source check that classifies.

```
P(x_i) = Σ_g π_g Σ_q w_q Π_{a∈A} P_a(x_ia | η_g + u_q) · Π_{j∈J} P_jg(x_ij | η_g + u_q),   η_0 = 0
P_a(x = 1 | θ)  = c_a + (1 − c_a) [1 + exp(−a_a (θ − b_a))]⁻¹      the anchors: one parameter set for every class
P_jg(x = 1 | θ) = c_j + (1 − c_j) [1 + exp(−a_jg (θ − b_jg))]⁻¹    the trial items: a and b per class, the floor shared
DIF_j = max_{g,h} | b_jg − b_jh |                       reject if DIF_j > 1.0  (provisional, see below)
```

The population is a mixture of `G` classes with proportions `π_g`; the classes have no
label and do not need one. The anchors (`A`, the DIF-free items the respondents also
answered) enter the likelihood with class-invariant parameters and each class has its own
ability mean `η_g`, so a class-wide shift is attributed to ability, not to the trial
items: DIF is a trial item's departure from the anchors' account of the classes. Every
class's ability has the same shape, a histogram estimated with the other parameters (`01`
D43): weights `w_q` over a fixed grid of 41 nodes on `[−5, 5]`, whose nodes `u_q` are the
grid standardized by the weights' own mean and standard deviation, so the shape has mean 0
and variance 1 and class `g`'s ability is `η_g + u_q` with probability `w_q`. A normal cannot
take a skewed shape and a mixture of two normals approximates it, so with a fixed normal a
skewed ability selected classes that do not exist (`13` §8.7.2); with the shape estimated,
one class fits in the post-D43 fixtures described in `13` §8.8; the full study is pending.

**Implemented fit and selection.** The objective includes the floors' priors and a
penalty on the histogram's raw-grid mean and variance. BIC reads the unpenalized
likelihood at the resulting fitted parameters. The implementation counts `Q−3`
histogram parameters (`Q=41`), the same count in each histogram candidate. A shared
parameter-count term cancels in comparisons, but estimated shape can still change
the likelihood differences and the selected number of classes (`13` §8.8).

**Open interpretation (`15` A3, B1–B3).** Calling that penalty a pure gauge and
subtracting two moment degrees of freedom requires justification: changing weights
can change standardized shape. The fit/selection procedure and its sensitivity to the
floor prior must be evaluated as implemented. The floor `c_j` is shared across
classes; class-specific guessing can be misspecification and is not generally
represented exactly by a difficulty shift. Fitted anonymous classes are statistical
components, not identified social groups or a guarantee that all bias is detected.

*Why the anchors are in the likelihood.* The first implementation fitted the trial items
on a proxy — the standardized anchor total as `θ` (`dif::mixture_dif`). Error in the
proxy makes the trial items positively dependent even without DIF (paper Prop. 10), so
on null batches the proxy model finds classes that do not exist: at N = 6,000 it selects
two classes with 10, 20 and 30 anchors (KR-20 0.68–0.89), flagging 8, 2 and 0 clean
items; the target model selects one class at every anchor count. In a campaign — most
of a batch shifted the same way — the proxy's differential gap inverts, while the target
model flags exactly the shifted items (`08` AT-DIF-12). The proxy model is retired from
the production path (`revalidate_batch_latent` runs the target model on the anchors it
admits) and kept for the fixtures and the sim reproduction.

*Threshold.* The literature value for `DIF_j` is 0.5 logit. The reference
implementation rejects at **1.0** on the gap, and only when the selected fit converged
and the BIC prefers a mixture (two or more classes) over one class. On the target model
the null gaps are 0 (one class selected) or far below 1.0 and the campaign gaps far
above it, so 1.0 is kept, provisional. The false-positive / power study (T24, `13` §7)
measured it: on 2PL populations no clean item is flagged at any cut from 0.5 to 1.5, and
at 0.6 and below the cut no longer binds — power is then the share of fits that select a
mixture. With a guessing floor in the population 12–17% of the clean items are flagged
at 1.0, so T25 sets the value after the guessing correction (D25; `08` DIF-006, DIF-008),
which the model carries since T25's first step (§B.1, `08` AT-DIF-13). In the
clean-population cells of the pre-D43 guessing supplement, no clean item was
flagged at cuts from 0.5 to 1.5; lowering the cut below 1.0 gained at most 3 points of
power (`13` §8.7.1). The misspecified populations had different results (§8.7.2).
These are historical observations, not a guarantee or a post-D43 threshold decision.
The cut remains provisional pending the candidate model's measurements and T83.

**Fit status is not a clean verdict (`15` A4).** `LatentDif::flags` returns false for
an unconverged fit as well as when no item crosses the rule. The current
`revalidate_batch_latent` result loses that distinction. Preserving a non-evaluable
outcome through the lifecycle is an open protocol correction.

*Reference implementation (T40, T54).* `G ∈ {1, …, 4}` and uniform (shared `a_j`) vs
non-uniform (per-class `a_jg`) DIF are chosen together by BIC, each candidate fitted from
several seeded starts with an analytic gradient (the EM artificial data: per node the
expected respondents, per respondent the class posterior and its first `θ`-moment, and for
an item with a floor its expected correct answers per node, since its log-odds are no
longer linear in `θ`); a floor is fitted on the logit scale from the chance level; a
class holding under 5% of the respondents does not define `DIF_j` (its difficulties are
unidentified). The verdict reads the difficulty gap only, as above: a per-class
*discrimination* gap is estimated and reported (`a_gap`) but has no threshold yet. T24
found that one would add detection only for a large discrimination gap in a large sample
(`13` §7.2, §7.5); T25 decides whether to adopt it.

**Protocol requirement: validate in batches.** T24's no-floor study did not find one
leaning item of eight at the tested sizes/shifts, while multiple leaning items were
more detectable (`13` §7.1). This is evidence about power in that regime, not a proof
that a single biased item's response curve is indistinguishable from the null family
(`15` A8). Batch validation and periodic re-checks remain requirements; their detection
limits must be measured on the candidate model and format.

**One template per batch (`01` D43).** No two columns of a batch — its trial items and the
anchors its respondents answered — come from the same template (`05` [9]). Items of one
template are answered alike beyond ability, and the mixture cannot tell that from a class
that finds both easier: with pairs so answered, null batches flagged clean items in 41% of
the cases (`13` §8.7.2). Each column declares its template with its format, and the gate
refuses a batch in which two share one (`PilotError::SharedTemplate`) before any fit.

**Multiple sources of bias.** A question can be neutral on one known axis and
distorted on another; the historical education example motivates this concern.
The anonymous mixture does not label its classes or certify discovery of every
social axis. Such sensitivity is a characterization question (`13`), with anchor
invariance and misspecification limits recorded in `15` B2/B6.

### B.4 Anchor validity and iterative purification

The metric depends on **anchor items** treated as invariant. Contaminated anchors
can alter the reference against which trial-item DIF is judged. The group-labelled
calibration/proxy procedure uses iterative purification:

```
1. estimate θ using all items (or the anchors)
2. identify the items with DIF
3. remove them
4. re-estimate θ using only the clean items (anchors)
5. re-test all items with the new θ
6. repeat until the set of discarded items stops changing
```

The historical prototype estimates `θ` from 30 anchor totals. The current anonymous
model instead fits anchor parameters in the likelihood and integrates over ability
(§B.3); it does not perform that proxy loop as its production re-check. Its admission
gate still requires `KR20_MIN = 0.90`, a provisional reliability rule. KR-20 does not
establish anchor invariance or the absence of latent DIF; the guessing supplement
shows why the gate needs reassessment (§B.6; `15` B2/B5).

### B.5 Upstream admissibility

Psychometrics does not save you from badly conceived questions. Strict taxonomy:

- **Admitted**: verifiable procedural and institutional facts, with a **mandatory
  citation to a primary source** (Official Gazette, parliamentary act, official
  dataset).
- **Rejected by construction**: normative items ("is it right that…"), predictive,
  counterfactual, and items that attribute causes to contested phenomena.

Disputes over an item are resolved by an evidentiary procedure (comparison with the
source), not by a vote.

**The source check (`01` D38).** The evidentiary procedure as it decides whether an item
the latent check flags (§B.3) is a *contested fact* (§B.7). It answers one question —
does the cited primary source establish the item's key? — from the record, in steps
anyone can redo, and asks nobody whether the item is true, fair or important:

1. *The citation was committed at deposit.* The primary source is a structured citation
   (`03`: an act identifier, never an arbitrary URL): the registry, the act's identifier
   in it, the locator inside the act (article, paragraph, table cell) and the passage or
   data found there. It is part of the draft's content id (`deposit::Draft::content_id`),
   so it cannot be changed once the item has shown DIF.
2. *The registry is a primary one:* it is on the closed list of primary registries — the
   Official Gazette, the parliamentary records, the official statistical datasets — kept
   by meta-governance (`05` §Meta-level governance).
3. *The passage is authentic:* the act the identifier names, as the registry publishes
   it, holds the cited passage or data at the locator.
4. *The key follows from the passage:* the item is built from a template (`05` [9])
   whose key is a declared, deterministic function of the cited data — the datum itself,
   or a comparison of two figures — and that function, applied to them, gives the keyed
   answer.

The item is a contested fact only if all four steps hold. A key that rests on an
interpretation of the source — something a reader must judge rather than a function can
compute — fails step 4, and the item is rejected on DIF as before. Step 4 limits the
contested pool to data-bound facts (dates, figures, votes, the text of a provision),
which the taxonomy above already favours. The check runs only on flagged items, at the
pilot and at each re-validation (an act amended since no longer holds the cited passage
and fails step 3; a repealed one makes the item obsolete, `05` [9]). The
protocol takes its verdict as an input (`source_verified` in `lifecycle::Event`); the
check itself in code — the citation type, the registry list, the template's key rule — is
Phase 3 work (`10` T68).

### B.6 Sample sizes and the minimum viable network

**Admission floors and required power are different quantities.** The current code
uses the following minimum numbers of distinct respondents:

| Path | Implemented floor | Meaning |
|---|---:|---|
| Stage-1 screen | 300 | Initial filter with point-biserial and a one-class, format-aware fit; ability shape held normal. The floor and cuts are provisional. |
| Group-labelled stage 2 | 1,500 | Calibration path with observed groups; not the anonymous deployment path. |
| Anonymous latent re-check | 3,000 | `revalidation::N_LATENT_MIN`, followed by anchor/template admission. It is not a guarantee of sufficient power. |

Sources: `protocol::pilot::{N1_MIN,N2_MIN}` and
`protocol::revalidation::N_LATENT_MIN`. The identity gate counts respondent
pseudonyms, not repeated answer rows (T65). T25 must determine format-specific
requirements on the candidate model; the historical 1,500–3,000 range is not a
minimum viable network claim.

**Historical measurements.** T24's no-floor model and T25 step 3's guessing-aware
model both predate D43. For the supplement's equal-class cells with 60 anchors,
8 trial items and two leaning items at `δ=0.9` (difficulty gap 1.8):

| Format | N | Historical per-item power | Batch admission rate |
|---|---:|---:|---:|
| 5 options | 3,000 | 9% | 13% |
| 5 options | 6,000 | 70% | 17% |
| 5 options | 12,000 | 97% | 12% |
| 4 options | 6,000 | 39% | 0% |
| 4 options | 12,000 | 90% | 0% |
| True/false | 12,000 | 5% | 0% |

Source: [`floor-power-summary.csv`](../verification/reports/t25/floor-power-summary.csv),
rows with `a=60`, `k=8`, `lay=c2`, `d=0.9`, `pi=0.5` and the listed format/N;
`13` §8.7.1 explains the design. The harness fits batches even when its recorded
admission gate fails, so power and admission must be read together. An unadmitted
batch is not a clean verdict. These values do not characterize D43 or set a new floor;
the rerun and its status are in `13` §8.8.

**Screen and anchors.** Stage 1's historical study found that the point-biserial cut
discards some good items and that excess guessing is poorly distinguished at N=300
(`13` §8.7.5). The current KR-20 floor of 0.90 rejects many guessing batches; this is
not repaired simply by assuming that a particular anchor count proves validity.
T25 must assess anchor information, admission and the resulting decisions together
(`15` B1–B5). An anchor-count alternative remains a proposal.

**Capacity accounting.** A complete batch with N respondents, A anchors and K trial
items requires `N × (A + K)` answers if each respondent answers every column, as in
the characterization generator. Its answer cost divided among trial items is
`N × (A + K) / K`; cost per accepted item also includes losses and repeated attempts.
For N=12,000, A=60, K=8, that is 816,000 answers, or 102,000 per trial before such
losses (exact arithmetic, not a deployment benchmark).

The old illustration of 10,000 nodes answering 50 items/month gives 500,000 answers,
but dividing by 1,500 and promising 333 usable items/month omits anchors and uses an
unsupported current sample requirement. No replacement throughput promise is
established. Reused anchor information or planned missingness would require a
validated administration design; they are not assumed savings.

**Separate feasibility constraints.** The network must supply the required distinct
respondents, enough overlapping reviews for Level A, and an adequate anonymity set.
They are different requirements. Neither a sample-size floor nor the simulations
establish a general privacy guarantee. The lifecycle permits an external respondent
panel (`05`); whether one is available, and the acceptable error/resource budgets,
are owner decisions (`10` T83; `15` D2/E).

### B.7 Contested facts and differential test functioning (`01` D38)

DIF shows that an item measures a second dimension on which the latent classes differ;
whether that dimension is a nuisance is a judgment (paper §4.7). When the key is a fact
a primary source establishes, the second dimension is knowledge of the fact: one class
is misinformed about it. Rejecting such items would bar every fact a camp disputes from
the bank, and the appeal of `05` [5b] could not recover them — Level B would reject them
for the reason Level A did. So an item the latent check flags (§B.3) goes one of two
ways:

- its key passes the source check (§B.5) → a **contested fact**, kept in a separate
  *contested pool* (from the pilot, or from the active pool at re-validation);
- otherwise → rejected at the pilot, retired at re-validation, as before.

The implementation draws contested facts using the per-fit DTF cost below and a
provisional tolerance. The intended guarantee is to control the whole test's
differential functioning. That guarantee is **open**, for the cross-fit and
active-item reasons below (`15` A2); selection by the implemented cost alone does
not establish it.

**DTF within one fit.** Let `F` be a target-model fit (§B.3) in which the items of a set
`S` were trial items, with counted classes `g` (share ≥ 5%, as for `DIF_j`), shares
`π_g` renormalized over them, ability means `η_g` and per-class curves
`P_jg(θ) = c_j + (1 − c_j) σ(a_jg (θ − b_jg))` — the floor shrinks a class gap by
`1 − c_j`. At ability `θ` the expected-score difference between
classes `g` and `h` is `Δ_gh(S; θ) = Σ_{j∈S} [P_jg(θ) − P_jh(θ)]`, and

```
DTF_F(S) = max_{g,h} Σ_{c,q} π_c w_q |Δ_gh(S; η_c + u_q)|
```

— the unsigned DTF (Chalmers, Counsell & Flora, 2016) on the number-correct scale, over
the batch's own ability distribution, at the worst pair of classes: every class's nodes,
`η_c + u_q` with weight `π_c w_q`, the fit's histogram shifted to each class (§B.3, D43;
`scoring::dtf`).
Items that lean the same way add up. Items that lean opposite ways cancel, but only
where their curves overlap: the absolute value inside the integral does not let a set
favour one class at low ability and the other at high ability. One item has the DTF of
its own curves; two mirror items cancel exactly; one counted class gives 0.

**Across fits: implemented cost and open bound claim.** Class labels are arbitrary
per fit. Class-invariant anchor parameters do not by themselves establish a common
class correspondence or target distribution. Matching through respondents would
also raise the profiling concern in invariant #1 and `03`. A common-population DTF
is not obtained merely by adding fit-local statistics. For a test `T`, the
implementation uses the cost

```
D(T) = Σ_F DTF_F(T ∩ F)          summed over the fits of its contested facts
```

as an admissibility check. It was described as a universal upper bound by the
triangle inequality. That interpretation is not established: the integral's target
measure and the class correspondence must be compatible across fits before applying
such a bound. Active-pool items are omitted from this implemented cost, but passing a
DIF cut does not make their true contribution zero (`15` A2).

Contested facts can cancel within a common fit. Periodic re-validation re-fits them
together; an item's recorded curves are those of its latest fit. A valid whole-test
guarantee still needs a specified target population, the contribution of every item,
and an uncertainty treatment. Those are prerequisites to interpreting the tolerance,
not issues solved only by changing its value.

**Tolerance.** A test's contested facts are admissible when `D(T) ≤ DTF_MAX`,
`DTF_MAX = 0.10` score points, provisional (T24/T25): about the DTF of one item of
negligible DIF — at the ETS class-A boundary (`|Δ_MH| = 1`, a log-odds gap of 0.43) a
mid-difficulty item has 0.08. For scale, with `a = 1.25` and two equal classes: one item
with a difficulty gap of 1.8 has 0.41 at difficulty 0 and 0.21 at difficulty 2; two such
items leaning opposite ways have 0.00 at equal difficulty, 0.04 at 0.25 apart, 0.07 at
0.5 and 0.14 at 1.0. The bound is computed on the fitted curves, and it is a point
estimate: on a batch fitted at N = 3,000 the fitted DTF of each set was within 0.03 of the
true one, and on two batches fitted through the production gate a drawn pair with a fitted
bound of 0.076 had a true DTF of 0.104 (`08` AT-PRO-08). T24 measured its sampling error
(`13` §7.3): the fitted DTF is biased upward where the true one is small — the four
clean items of the mirror layout, true DTF 0, are fitted at 0.025–0.065 on average —
sets far over the tolerance are refused, and within about 0.05 of it the decision is
noisy both ways; with a guessing floor the same, the clean items fitted at 0.03–0.05, and
with true/false more sets near the tolerance decided in error (`13` §8.7.3). The
tolerance, and any margin below it, are T25's.

**The balanced draw.** A test with `n` contested slots draws them from the beacon
(INV-10; `randomness::CONTESTED`, keyed on the test's number) among the selections with
`D(T) ≤ DTF_MAX`. The fits are visited in a seeded order; for each, the candidates are
its subsets of at most `n` contested facts whose own DTF is within the tolerance (the
empty one included); a table of the least bound that completes `n` from the fits not yet
visited keeps only the candidates that can still be completed, and one is drawn
uniformly among them. So the draw fails only when no balanced selection of `n` exists —
it then reports the sizes that do — and every balanced selection has a positive
probability. The bound is summed in fixed point (each fit's DTF rounded up to `2⁻³²`
score points), so the budget is exact (`protocol::contested`). The draw is a function
of the pool's content and the seed alone: the pool keeps a canonical order — within a
fit, the members by content id; the fits by their least member, unique since a fact
belongs to one fit — so the seeded visiting order, the enumeration of each fit's
candidates and the order of the drawn facts owe nothing to the order in which facts were
recorded, re-measured or retired. Two replicas holding the same facts in the same fits
draw the same test from the same seed, and whoever records the facts cannot pick the
test by the order of recording (the order-dependence the lottery had until T37).

**Scores.** A contested fact is admitted to the bank. Its Level B outcome for the
evaluator score is 1, as for an item that reaches the pool (§C.2) — scoring it 0 would
pay reviewers to predict the rejection of true facts a camp disputes — and an appealed
item that becomes a contested fact promotes the appeal (§C.1, `05` [5b]).

---

## Level C — Node reputation

Two **separate** scores, on unlinkable pseudonyms (see `01` D5). Never combined.

### C.1 Author score `C_a`

**Implemented score.** Let `q_j ∈ [0,1]` be an item's quality input. The code uses a
regularized, time-discounted weighted average:

```
          α₀ + Σ_j w_j q_j
C_a  =  ─────────────────────      w_j = exp(−Δt_j / T)
        α₀ + β₀ + Σ_j w_j

α₀ = 2, β₀ = 3; T = 18 months (exponential decay time, provisional)
```

This is the formula in `reputation::author_score`. Its half-life is `T ln 2`, not T.
The earlier Beta–Beta hierarchy `q_j | ψ_a ~ Beta(κψ_a, κ(1−ψ_a))`,
`ψ_a ~ Beta(α₀,β₀)` does **not** have this posterior mean (`15` C5).
Describing the implemented index correctly does not decide whether a different
inferential model is wanted. The meaning of continuous `q_j` and the intended decay
policy remain explicit design/calibration questions (`08` REPUTATION-001; `15` C5–C6).

Shrinkage is indispensable: a node with 2 of 2 items accepted must not be worth as
much as one with 180 of 200. Example with `Beta(2,3)`: author A (2/2) → 4/7 ≈ 57%;
author B (180/200) → 182/205 ≈ 89%.

**Use.** `C_a` governs the **rate limit on proposals**, not the vote weight:

```
q_a = q_min + (q_max − q_min) · C_a
```

**Appeal stake (`01` D27, `05` [5b]).** An appeal is a pseudo-observation *inside* this
average, not a deduction from it: filing appends `q = 0` at `Δt = 0` (the escrow); the
verdict replaces it with the item's real `q_j` if the pilot promotes the item — to the
active pool, or to the contested pool (§B.7) — and leaves it otherwise. The stake floor
is the prior mean `α₀ / (α₀ + β₀)` (0.4 with `Beta(2,3)`): an author files only while
`C_a` covers it, so a failed appeal costs the next one until the evidence has restored
the average. There is no additive gain — the reward for being
right is the good observation itself (`protocol::appeal`, T61). Floor provisional (T25).

### C.2 Evaluator score `S_u` and review weight `w_u`

> **Revised by D33 and D35 (T50 and T52, done).** The ratio-form Brier skill score of the
> first design is not a proper scoring rule (paper Prop. 12): it paid a dissenter to move
> toward the crowd. Since T50 the score is the leave-one-out difference score below and
> the weight lives on the odds scale. Since T52 the scored items are the live outcomes
> too, with inverse-probability weighting intended to correct gate-dependent
> observation. Its protocol-level incentive guarantee remains open (`15` A1).

The reviewer does not give a binary judgment: they **declare a probability** `p_uj`
that the item passes Level B empirical validation. On every scored item — a golden item
(`05` §Golden items) or a live item whose outcome `o_j ∈ {0,1}` is known (`01` D35, T52) —
the difference rule below is **strictly proper conditional on a report-independent
baseline and a fixed outcome law**. Its transfer to selected live outcomes also needs
the information/selection assumptions discussed under Exploration:

```
S_uj = (p̄_{−u,j} − o_j)² − (p_uj − o_j)²      p̄_{−u,j} = Σ_{v≠u} w_v p_vj / Σ_{v≠u} w_v
```

`p̄_{−u,j}` is the weight-adjusted mean forecast of the *other* panelists (`01` D23's
crowd baseline minus the reviewer scored). The score is the reviewer's Brier improvement
over the crowd: positive when they are right where the crowd is wrong, exactly 0 for a
reviewer who reports the crowd's forecast, negative for noise or block voting. Its
expectation is maximized by the true belief, by exactly `Σ_j (p_uj − q_uj)²` over any
other report under those conditions (`08` AT-REP-05). With no other positive panel
weight, the implemented fallback uses the reviewer's own forecast as baseline,
producing zero for every report; this case is uninformative, not strictly proper
(`15` A10).

**Fundamental property.** Someone who replicates the consensus gets `S_u = 0`. You
gain reputation only by being right **when the crowd is wrong**. This is the incentive
needed against majority capture.

`S_u` is the mean of `S_uj` over the reviewer's `k_u` scored items — the symmetric
long-window mean of `01` D34 (the change detector on the per-item scores is T51).

**Exploration (`01` D35, T52).** Golden items alone are too few (about one every two
epochs per reviewer). A reviewer is therefore scored on every reviewed item whose Level
B outcome is known: the golden items, every live item that reaches a pilot (a pass — a
contested fact is one, §B.7 — or a screen or DIF rejection: `o_j = 0`), and a random
`ε = 5%` of the items the gate rejects, drawn from the public beacon (`protocol::exploration`, keyed on the admitted
slot) and piloted for measurement only — never entering the pool. Scoring only the
outcomes the gate lets through would not be proper: the report then decides whether its
own outcome is observed, and the bare observed score pays a reviewer to report on the
gate's side (`08` AT-REP-06: 0.08 for reporting 0.50 against 0.0045 for an honest 0.40).
Each observed score enters the mean at its inverse inclusion probability `1/π_j` — 1
for an item that entered the pilot on its own account, `1/ε` for an explored rejection —
over every reviewed item, observed or not:

```
S_u = ( Σ_{j observed} S_uj / π_j ) / N_u        N_u = the reviewer's reviewed items after the gate
```

The IPW identity requires inclusion probabilities conditional on the information
available when the report is chosen, with positive observation probability under the
allowed strategy. The current exploration draw is computable from the same public
epoch beacon used for assignment; domain separation does not make it unknown until
reports are committed. The simplified adaptive-report counterexample in `15` A1
therefore leaves the protocol-level properness claim open, despite the fixed-report
expectation checks. No randomness schedule is changed by this documentation pass.
`k_u`, the count that decides probation and the shrinkage, is the observed items. The
change detector (D34) reads the *unweighted* observed scores against their own mean, so
one explored item cannot fire it by its weight. Exploration also measures the gate's
false-negative rate — how many rejected items would have passed Level B — and costs
about `ε` of pilot capacity. The draw is grind-free: it reads the commit-reveal beacon of
`01` D41 (`04` §The epoch's beacon, T37).

**Use.** `S_u` weights the review vote, on the odds scale and shrunk by the number of
scored items:

```
w_u = exp( γ · S_u · k_u / (k_u + k₀) )       γ ≈ 35,  k₀ ≈ 100   (provisional, T25)
w_u ← min(w_max, w_u),   w_max = 3 × median(w)  over the reviewers who carry weight
```

A crowd-level reviewer weighs 1; one reliably 0.02 better than the crowd weighs about
double; the cap binds on an outlier (it never did on a score in `(0,1)`, `08` G-12).
Shrinkage stops luck from buying weight: with 16 scored items one standard error of luck
(0.025) is worth ×2.4 without it and ×1.13 with it. A new pseudonym has weight 0 until
30 scored outcomes (`01` D36, `03` P2), then the shrinkage takes over.

*History.* Until T50 the score was the Brier skill score against the crowd's mean
forecast, `BSS_u = 1 − Σ_j (p_uj − o_j)² / Σ_j (p̄_j − o_j)²`, squashed by
`E_u = σ(γ·BSS_u)` and used as `w_u = min(w_max, E_u)`. A ratio of two sums is not an
expectation of a score: with one item the optimal report satisfies
`logit p* = logit q + 2 logit b` (paper Prop. 12) — a reviewer who believes 0.30 while
the crowd says 0.65 was best off reporting 0.60. The fixture oracle `levelc_bss.csv`
still reproduces that function (`08` REPUTATION-002); on it the difference score tells
the same story (the expert beats the crowd, the followers do not) and stays proper.

### C.3 Judgments without verifiable truth

For dimensions that never receive an empirical verdict (formal clarity, source
quality, tone), there is no `o_j`. A **peer-prediction** mechanism with correlated
agreement (Dasgupta–Ghosh), incentive-compatible without ground truth. For reviewer
`p` and reference reviewer `q`:

```
Score(p) = 1[ x_p(shared_item) = x_q(shared_item) ]
         − 1[ x_p(t_1) = x_q(t_2) ]          t_1, t_2 non-shared items
```

The second term subtracts baseline agreement (chance or common bias). Reporting the
honest signal is the highest-payoff equilibrium among symmetric strategies.

Richer variant: **Bayesian Truth Serum** (Prelec). Each reviewer declares (a) their
own judgment, (b) the expected distribution of the others. The **surprisingly common**
answer is rewarded — more frequent than the group predicted. It extracts information
from the informed minority.

### C.4 Temporal dynamics and cap

> **Revised by D33, D34 and D36 (T50, T51 — done).** The cap applies on the odds scale,
> where it binds, probation lasts 30 scored outcomes, and the asymmetric update of the
> first design is replaced by the symmetric mean of §C.2 plus a change detector.

- **Change detector (`01` D34).** The weight reads `S_u`, the mean of the reviewer's
  per-item scores (`probation::SkillTrack`). Against that mean, a one-sided CUSUM on the
  per-item scores watches for a sustained *drop* — a reviewer who has built a reputation
  and starts spending it:

  ```
  s ← max(0, s + (S_u − S_uj) − k)        alarm when s > h;   k = 0.03, h = 1.5 (provisional, T25)
  ```

  It runs only once the reviewer is out of probation (a mean over few items is no
  reference). On an alarm the reviewer returns to probation: the mean, the count and the
  statistic restart, so the weight is 0 until 30 new scored outcomes and shrunk again
  afterwards. This is the intended contract: the current founder path restores
  `Founder` weight 1 after the reset, an open mismatch (`15` A6), not an approved
  exception to that contract. It reacts to a change, not to variance: a cautious
  reviewer with noisy scores around a good mean raises nothing. In the paper's simulation, `k = 0.03`,
  `h = 1.5` give 0.07 false alarms per 1,000 scored items and catch a reviewer who starts
  flipping 20% of forecasts after a median of 36 items (`08` AT-REP-07).
- `w_max = 3 × median(w)`, a hard cap recomputed each epoch over the reviewers who
  carry weight (founders at 1 and established reviewers at their odds weight, not
  probationers at 0; `orchestrator::epoch_weight_cap`). Limits the damage of a single
  event.

*History.* Until T51 `E_u` rose slowly and fell fast (an asymmetric moving average),
meant to make the long-con attack unprofitable. It penalized variance, not error: its
stationary level sat far below the true mean, and a cautious reviewer who beat the crowd
(true +0.009) was held at −0.061 while a crowd copier stayed at 0 (paper §5.5).

---

## Anti-collusion (coordination detection)

> **Revised by D39 and D40 (T56 and T57, done).** Within an epoch two reviewers share
> under one item (paper §6.3), and raw correlations cannot separate a cartel from
> like-minded honest reviewers. Detection reads the correlation of model residuals over
> long histories; a detected cluster limits panel assignment (at most one member per
> panel, `05` [4]) instead of losing weight. The sublinear discount below is kept in the
> engine for analysis and is not applied on the protocol path.

**Detection (`01` D39, T56).** For every rating the bridging fit leaves a residual,
`e_uj = r_uj − r̂_uj` with `r̂_uj = μ + b_u + b_j + f_u·f_j`. The model already explains
the agreement of two honest reviewers who share a position: their residuals do not
correlate. A cartel agrees beyond the model: its residuals do.

```
1. residuals accumulate per reviewer and item across epochs   (ResidualHistory)
2. a pair (u, v) is read only once it shares ≥ 30 items         (min_shared)
3. ρ_uv = Pearson correlation of the shared residuals
4. flagged if ρ_uv ≥ ρ_min = 0.7 and the permutation p-value ≤ 0.001: (1 + hits)/(P + 1)
   over P = 999 permutations, seeded per pair by seed·0x9E3779B97F4A7C15 ⊕ (u ≪ 32 | v)
5. clusters by average linkage over the flagged pairs: the two groups with the highest mean
   correlation over all their cross pairs (an unflagged pair counting 0) merge — on a tie,
   the pair first in member order — while that mean is ≥ ρ_min
```

On the paper's dataset (200 reviewers in two camps, 60 items, a cartel of 10 minority
members rating 5 majority-favoured items at 1.0 with jitter σ = 0.05): honest pairs of
the same camp correlate at +0.015 on residuals (+0.93 on raw ratings, level with the
cartel's +0.92), the cartel at +0.89; every cartel pair is flagged and no honest pair,
where the raw rule — `|ρ| ≥ threshold` with connected components — chains the whole
majority camp into one cluster. `ρ_min` is 0.7 because at 0.5 four of the 18,000 honest
pairs reach it by chance. Average linkage means one spurious pair never chains an honest
reviewer to a cartel, and opposite camps, whose residuals are uncorrelated, are never
joined. All thresholds provisional (T25).

**What a cluster does (`01` D40, T57).** A panel holds at most one member of each
cluster, the band's extra round included. Nobody's weight changes: false positives cost
their members only co-assignment, and splitting a cartel to evade detection buys nothing
that assignment does not already deny.

**Sublinear discount (`01` D7, analysis only).** The individual cap does not stop a
cartel of coordinated nodes; the engine keeps the discount as a measure of a cluster's
influence:

```
W(G) = ( Σ_{u∈G} w_u )^α,   α ≈ 0.5
```

A coalition of `k` nodes voting identically counts as `√k`: 500 coordinated ≈ 22
independent. It is not applied to the protocol's weights: it can be evaded by splitting
(paper Prop. 17) and it penalizes honest like-minded reviewers, which the residual
detector no longer confuses with a cartel.

---

## Initial parameters

Each threshold's reason, calibration procedure, sensitivity, what breaks if it moves and its
location are in `docs/14` (SC-1): Level A's are written, Level B's and C's come with T25's
step 4.

| Parameter | Value | Notes |
|---|---|---|
| `λ_b / λ_f` | 0.15 / 0.03 | ratio ≈ 5:1; within a factor of 3 of it the verdicts move by at most 7 points, and `λ_f` below 0.03 costs axis recovery (`13` §8.7.4) |
| `τ` (bridging threshold, on `S_j`) | ~0.80 (provisional, D32) | absolute, on the probability scale; **calibrate on the pilot**, not fixed; characterized for 0.70–0.90 in `13` §7.4 |
| `ε` (uncertainty band) | ~0.02 (provisional) | questions in the band → supplementary review; ≈ 3× the bootstrap spread of `S_j`; error rates near `τ` in `13` §8.7.4 |
| `k_extra` (extra panel) | 4 (provisional) | reviewers drawn outside the first panel for a band item; their ratings join the first panel's before the re-decision (D26, T60); set with `ε`, `τ` and the panel's size (`13` §8.7.4) |
| `γ_appeal` (side gap for appeal) | 0.25 (provisional) | a rejected question with a wider gap was rejected for polarization: appealable (`05` [5b]) |
| side floor | 5% of the reviewers, rounded up (provisional) | the fewest reviewers a side of the split holds (`MIN_SIDE_PER_MILLE = 50`, D42) |
| `MIN_COVERAGE` | 1 rating (provisional) | an item with fewer from either side goes to supplementary review whatever its score (D42) |
| `d` (factors) | 1 | a second axis requires evidence and a score definition (`14`, `d`); not an automatic parameter change |
| `k` (reviewers/item) | 7–11 | odd, random assignment stratified on `f_u` |
| `N` pilot stage 1 | 300, provisional floor | point-biserial prefilter and one-class format-aware fit; limitations in §B.6 |
| `N` pilot stage 2 | 1,500 labelled calibration; 3,000 anonymous, provisional floors | required power/sample by format remains T25 work (§B.6) |
| `a_min` | 0.6 | minimum discrimination |
| `\|β₂\|` max DIF | 0.40 | logistic regression |
| `DIF_j` max (latent classes) | 1.0 logit (provisional; literature 0.5) | IRT mixture; see §B.3; characterized for 0.5–1.5 in `13` §7.5, on items with no floor |
| `w` (floor prior) | 20 pseudo-observations (provisional) | the weight of the prior on a choice item's floor, its mode at the chance level `1/m` (§B.1, D25) |
| `Δ_MH` max | 1.5 | ETS class C = reject |
| `α` (cluster discount) | 0.5 | square root; analysis only, not applied to the protocol's weights (D40) |
| `min_shared` (coordination) | 30 | shared items before a pair's residual correlation is read (D39); at the design scale two reviewers share under one item per epoch, so a pair is read only after many epochs |
| `ρ_min`, `p_max` (coordination) | 0.7, 0.001 | a flagged pair's residual correlation and permutation p-value (999 permutations); 0.5 flags honest pairs by chance |
| `w_max` | 3× median | individual cap |
| `T` (implemented exponential decay time) | 18 months, provisional | `exp(−age/T)`; half-life `T ln 2`; intended decay policy open (`15` C6) |
| `η` (honeypot rate) | 5% | see `05` |
| `ε` (exploration rate) | 5% of gate rejections | measurement only, never the pool; the observed score at weight `1/ε` (D35) |

---

## What the engine does NOT solve (structural limits from testing)

1. **The true-but-divisive false negative.** Bridging does not tell a true,
   polarizing fact from one-sided propaganda: they produce the same voting pattern,
   and **no threshold saves it**. This is the most serious limitation. Mitigation: the
   appeal-to-evidence channel (`05`), not a change to the engine.
2. **The elite-consensus blind spot.** A question can pass bridging and political DIF
   yet be strongly distorted on a socio-economic axis. Mitigation: multi-axis DIF
   (B.3), which must be actively sought.
3. **Fixture yield is not production yield.** The historical example sends 3 of 10
   questions to the pool; it does not establish a general proposal-to-acceptance ratio.
4. **The threshold is a blade.** See the uncertainty band (A.3).
5. **Human judgment predicts validity poorly.** Level A is in effect anti-spam against
   partisan questions, not a quality indicator. The real verdict is Level B.
