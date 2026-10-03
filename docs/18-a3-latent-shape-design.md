# 18 — A3: the latent ability histogram — parametrization, gauge, penalty, count

| | |
|---|---|
| **Status** | Diagnosis reviewed by Astra on `deb4e5e`; P1 implemented (§8) and **approved on `16e862c`: A3 closed** within the moment penalty's interpretation and the nominal count (`15`); B1–B3 keep the residues. The model, its objective and gradients, the selection criterion, the thresholds and the historical results are unchanged. |
| **Baseline** | `docs/phase1-review-alignment`; written on `af17eb3`, first committed at `deb4e5e`. Line references in §§1–7 are to `af17eb3` (the diagnosis' tests sit after its line 1387 of `latent.rs`); §8 gives P1's lines. |
| **Scope** | `15` A3: the histogram of `01` D43 in `scoring::latent` — what is optimized, what the "gauge" does, what the BIC counts, and whether the count affects selection. B1–B3 (identification and BIC under misspecification) are touched only where A3 needs them. |
| **Evidence** | **L** read in the source; **D** proved here; **C** recalculated (`sim/latent_shape_dimension.py`, numpy); **E** executed as a Rust test in `crates/scoring/src/latent.rs`. |

## Design review (Astra)

Review on `deb4e5e`:

- Approved: the dimension `Q − 1` of the family of ability distributions, on the open simplex;
  its distinction from identifiability through the responses, not proved; the penalty read as a
  regularization proportional to `n`; the mathematical cancellation of the common offset in the
  current comparisons; P1 as the correction to implement.
- Evidence: reading of the diff, the code and the tests; a mathematical check; a run of
  `sim/latent_shape_dimension.py` and an arithmetic example of the rounding. The Rust tests and
  the two diagnostic fits were not re-run.
- Not approved, and not attributed to the review: identifiability from the responses, the
  penalty's intensity, the BIC's validity for penalized mixture fits. A3 stays open until the
  implementation is reviewed.
- Corrected after the review: §3's sensitivities and curvatures are conditional on the other
  parameters held fixed; §6's promise of bit-identical `bic_gain` and records is withdrawn (§8
  measures the actual effect).

## 1. The finding and the exact question

A3 (`15`): "The histogram's claimed gauge can affect standardized shape. Its penalty and `Q−3`
parameter count require justification"; the review added that a constant two-parameter offset
across histogram candidates does not alone change their BIC ordering, and gave the 3-node
example of `15` (*Small reproductions*). The claims under review are the code's and the docs':
the penalty is "the histogram's gauge: its moments on the grid, free under the standardized
nodes" (`latent.rs:827–828`), the count is "the histogram's weights less their sum, mean and
variance" (`:214–217`, test `:1247–1284`), and D43 says the weights "are the same parameters in
every candidate model … so they do not decide the number of classes" (`01:984–986`).

Questions: (i) does standardizing the nodes make the grid's mean and variance gauge directions;
(ii) what does the penalty do; (iii) what dimension does the represented family have; (iv) does
the count change any selection the code makes.

## 2. The model as implemented (L)

- **Grid.** `θ_q = −T + 2Tq/(Q−1)`, `q = 0…Q−1`, `Q = 41`, `T = 5` (`latent.rs:33–44,241–248`).
- **Histogram.** Free logits `ℓ ∈ R^Q`, the last block of the parameter vector (`:177–178,190`);
  `w = softmax(ℓ)`; grid moments `m(w) = Σ w_q θ_q`, `v(w) = Σ w_q (θ_q − m)²`; nodes
  `u_q = (θ_q − m)/√v` (`:251–264`). Class `g`'s ability is `η_g + u_q` with probability `w_q`,
  `η_0 = 0` (`:228–234,491–495`). Without a histogram (`nodes = 0`) the same map is applied to the
  logits `−θ²/2`, a standardized discretized normal (`:274–286`).
- **Likelihood.** `P(x_i) = Σ_g π_g Σ_q w_q Π_a P_a(x_ia | η_g + u_q) Π_j P_jg(x_ij | η_g + u_q)`
  (`02` §B.3; `evaluate`, `latent.rs:479–804`).
- **Objective.** `fit_from` minimizes `[NLL + Pen_c + Pen_G] / n` (`:847–875`), with the floors'
  priors `Pen_c = Σ_f w_f [c_f softplus(−γ_f) + (1 − c_f) softplus(γ_f)]` (`:815–825`) and the
  "gauge" `Pen_G = n [m(w)² + (v(w) − 1)²]` (`:829–843`). The NLL returned is the likelihood's
  alone, at the penalized optimum (`:873`).
- **Selection.** `BIC = 2·NLL + k ln n`, `k = free_params = len − 3` when the histogram is
  estimated (`:215–217,924`). The candidates are one class and `G = 2…4`, each with shared and
  with per-class `a`, all with `nodes = one.nodes` (`:927–938,974–981`); a converged candidate with
  a lower BIC replaces the best (`:1022`); the search stops when a class count lowers nothing
  (`:1026–1028`); `bic_gain = BIC₁ − BIC_best` (`:1078`). Stage 1 holds the shape (`nodes = 0`)
  with one class (`pilot.rs:312–316`).
- **Gradient.** A logit moves its weight and every node: `∂u_q/∂ℓ_r = −(∂m/∂ℓ_r)/√v −
  u_q (∂v/∂ℓ_r)/(2v)`, `∂m/∂ℓ_r = w_r(θ_r − m)`, `∂v/∂ℓ_r = w_r((θ_r − m)² − v)` (`:782–803`; the
  penalty's `:836–841`). `nll_and_gradient_match_the_reference` (`:1286–1340`) checks NLL, floor
  priors and penalty against central differences, histogram models included (E, re-run).

## 3. Gauge and dimension

**(a) Redundancy of the parametrization.** `ℓ → ℓ + c·1` leaves `w`, `m`, `v`, `u`, the NLL and
`Pen_G` unchanged (D: softmax). It is the only one. *Proposition (D).* On the open simplex the map
`w ↦ D(w) = Σ_q w_q δ_{u_q(w)}` is injective with injective derivative. *Proof.* The atoms of
`D(w)` are the image of the equally spaced grid under the increasing affine map
`θ ↦ (θ − m)/√v`, each with positive mass. If `D(w) = D(w')`, the atoms coincide as sets with
their masses, so the increasing affine map taking one image to the other maps the grid onto itself;
an increasing affine bijection of a finite equally spaced set is the identity, hence `w' = w`. A
tangent vector `(dw, du) ≠ 0` changes either the mass of a fixed atom or the position of an atom of
positive mass, so `D` changes at first order. ∎ The histogram therefore represents a family of
ability distributions of dimension `Q − 1 = 40`, not `Q − 3`. Support (C): the map `ℓ ↦ (w, u)` at
random logits has rank 40, its smallest singular value `2.7·10⁻¹⁰` the shift. This is the dimension
of the ability family, not a proof that the 40 directions are identified by the responses: that
needs the map from the parameters to the response distribution to have full rank, which is not
proved here (below, *Generic dimension*).

**(b) Identification constraints actually imposed.** `η_0 = 0`, and every class's shape has mean 0
and variance 1 for every `w`. These remove the metric's affine indeterminacy exactly: an affine
change of `θ` absorbed by `(a, b)` would leave the shape unstandardized unless it is the identity.
They constrain the distribution of `θ` (the metric of D25), not the weights.

**(c) Normalization of the nodes.** `u_q` is a deterministic function of `w`: the nodes move with
the weights. It cannot remove dimensions of `w`, since `D` is an immersion of the simplex (a).

**(d) Regularization.** `Pen_G` reads `m` and `v`, which are functions of `D(w)`: the atoms of `D`
are spaced `2T/((Q−1)√v)` apart and offset by `−m/√v`. It therefore chooses between different
distributions. The review's example in the implemented map (C, E
`standardized_nodes_do_not_make_the_grid_moments_a_gauge`): on nodes `−1, 0, 1`, weights
`(¼, ½, ¼)` and `(⅛, ¾, ⅛)` both standardize to mean 0 and variance 1, at nodes `±√2` and `±2`,
with fourth moments 2 and 4 and penalties `¼` and `9/16` per respondent. A 3-node toy: it shows
that the grid moments are not gauge directions, not how much they matter at `Q = 41`.

**Near-redundancy, and where it fails.** For a smooth histogram `w_q ∝ f(θ_q)`, tilting the logits
by `tθ` or `tθ²` (the location and scale directions of a normal `f`) changes `D` only through the
lattice's offset and spacing. For the response patterns of eight 2PL items (C): change per unit
tilt `8.1·10⁻⁷` along `θ` and `5.7·10⁻⁶` along `θ²`, against `3.2·10⁻²` along `θ³`, a level set by
the normal's truncation at `±5` (its density there is `1.5·10⁻⁶`). At a rough histogram the same
tilts give `5.8·10⁻²` and `3.5·10⁻²` against `3.8·10⁻²`: the patterns are sensitive to them. These
are sensitivities with the item parameters held fixed; they prove no joint identifiability.

**Fitted histograms are rough** (E, `the_moment_penalty_holds_a_fitted_histogram`, `--ignored`,
11 s; one class, 30 anchors, 6 items, seed 7). On a normal batch of 2,000 the lightest weight is
`8.0·10⁻⁸` and the largest second difference of `ln w` 12.0; on a skewed batch of 3,000,
`8.9·10⁻²⁹` and 101.7. The NLL's curvature per respondent and per unit tilt variance is 0.119
and 0.082 along `θ` and `θ²` (normal batch) and 0.073 and 0.194 (skewed), against 0.346 and 0.248
along `θ³`. These curvatures hold every other parameter at its fitted value: along the grid moments
the conditional curvature is of the order of a genuine shape direction's, on these two fits. A
conditional curvature does not establish that the data identify those directions jointly.

**Generic dimension, and its limits.** `Q − 1` for the ability family at interior points; whether
the response distribution identifies all of it is open. It drops at
weights on the boundary (the fitted ones reach `10⁻²⁹`, numerically there), at coincident classes
(equal `η_g` and item parameters) and at `π_g → 0`; label switching is a discrete symmetry and
removes no dimension. Asymptotic identifiability is not information in the sample: with the 16
parameters of the eight items added, the singular values of the pattern map fall from 1 to
`10⁻¹¹` without a gap (26 or 27 of 57 above `10⁻⁶`, at either histogram; C). The histogram's
fine structure is weakly informed as a whole, so a numerical rank settles neither the generic
dimension nor the identifiability from responses here. A counting argument only shows that it is
not excluded: with 60 anchors, as in the tests, the response distribution has more than `2^60`
cells against a few hundred parameters.

## 4. The penalty

- **Acts on** `m(w)` and `v(w)`, the moments of the weights on the fixed grid; it is invariant
  under the logit shift, the only exact redundancy (D).
- **Centre** `(0, 1)` in the grid's coordinates; **intensity** `n`, i.e. weight 1 per respondent in
  the objective the optimizer sees (`:854,864`). It grows with the sample like the likelihood, so it
  keeps a non-zero relative weight however large `n` is: unlike a fixed prior, whose influence
  vanishes, it moves the estimator's target to `argmax E[ln P] − m² − (v − 1)²`.
- **Depends on the representation**: on the grid's range and node count, not on the logits' shift.
- **Classification.** It removes no redundancy: none exists for it to remove (§3 (a)). It imposes
  information, at a weight that scales with `n`, on the lattice of the represented distribution,
  and changes the family the fit favours: its hard limit `m = 0, v = 1` would leave the nodes on
  the grid and restrict the weights to the `(Q − 3)`-dimensional set with grid mean 0 and variance
  1. Between nearly equivalent smooth shapes it picks a representative, the role D43 meant; on the
  two fitted, rough histograms it holds the grid moments where the NLL, other parameters fixed,
  would move them (below).
- **Effect measured** (E, same test): an unpenalized refit from the penalized optimum lowers the
  NLL by 0.172 and 0.211 nats; the grid mean moves to −0.065 and 0.079, the variance to 1.063 and
  0.935; skewness 0.0002 → 0.0027 and −0.783 → −0.795; kurtosis 3.166 → 3.195 and 3.460 → 3.524;
  no anchor's `a` or `b` moves more than 0.0034, no item's `b` more than 0.0018. Two one-class
  fits, not a characterization: the penalty's curvature (2.0–5.4 per respondent) is 17 to 53
  times the NLL's along the same directions, and holds `|m|`, `|v − 1|` below `10⁻³`.
- **Consistency.** Objective, gradient and node movement agree (`:782–803,836–841`; E re-run). The
  BIC reads the NLL without the penalties (`:873`).

## 5. Count and selection

| Term | Current count | Dimension of the implemented family | Same in every candidate |
|---|---|---|---|
| class logits `ζ` | `G − 1` | `G − 1` | no |
| class means `η` | `G − 1` | `G − 1` | no |
| anchors `a`, `b` | `2A` | `2A` | yes |
| item `a` | `K`, or `KG` per class | same | no |
| item `b` | `KG` | `KG` | no |
| floors `γ` (with priors) | `F` | `F` | yes |
| histogram | `Q − 3 = 38` | `Q − 1 = 40`; `Q − 3` only in the hard-constraint limit | yes |

- **Where the +2 cancels** (D, L): every candidate of a call carries the same histogram
  (`:933–937,980`), so the offset vanishes in the comparison `b < best.0` (`:1022`), in the stopping
  rule (`:1026`), in `bic_gain` (`:1078`) and so in every consumer of it (characterization records,
  `level_b.rs`). Stage 1 has one candidate. This confirms the review's own note that a common
  offset cancels, and delimits A3 to it; neither the review nor this analysis claims a distortion
  of the selection. Mathematically the ordering is unchanged. Numerically the comparisons are on
  floating-point sums, so adding `2 ln n` to both sides of `b < best.0` can reverse only a tie
  within rounding (BICs of order `10⁵` have spacing about `10⁻¹¹`); the differences themselves,
  `bic_gain` included, can move by rounding without changing the model selected (§8 measures it).
- **Where it does not**: the absolute BIC of each candidate (`LatentDif::candidates`), recorded in
  the golden rows `latent.candidates` and `floor.candidates` (`scoring/tests/golden.rs:171–172,
  204–205`), shifts by exactly `2 ln n`; and any future comparison of a held-shape model
  (`nodes = 0`) with a histogram model, which no current path makes.
- **What the count does not settle.** The BIC reads the NLL at a penalized optimum, whose cost is
  a candidate's own (0.17–0.21 nats on the two one-class fits, not measured for mixtures); the BIC
  is a heuristic for mixtures (paper `04-level-b.tex:53–55`); the histogram's 38 or 40 dimensions
  are nominal against the information the data carry (§3). These are B1–B3, not closed by a count.

## 6. Two corrections

**P1 — keep the family; correct the interpretation, the documentation and the count.** Describe
`Pen_G` as a regularizing penalty on the grid moments that grows with `n`, not a gauge (`02` §B.3,
the code comments at `latent.rs:214,827–828,1247`); count `Q − 1` for the histogram, the dimension of
the ability family the code fits, with two notes: the offset cancels in every current comparison,
and the identifiability of those dimensions from the responses is not proved.
- Solves: the count and the description match the implemented family.
- Leaves open: whether the responses identify all `Q − 1` directions; the penalty's effect (small on
  two one-class fits, unmeasured for mixtures and verdicts), the
  roughness of the fitted histograms, the BIC's validity and weak identification (B1–B3).
- Effects: objective and optimization unchanged, so the fitted parameters of every candidate are
  unchanged; in exact arithmetic each histogram candidate's BIC rises by `2 ln n` and every
  difference is unchanged; in floating point the differences, `bic_gain` included, can move by
  rounding without changing the model selected, so the effect is measured (§8).
- Compatibility: the golden rows of the candidates' BICs regenerated, and any other row the
  rounding moves; the same for characterization records, which carry `bic_gain` (§8).

**P2 — explicit moment constraints.** Weights on the fixed grid with `Σ w = 1`, `Σ wθ = 0`,
`Σ wθ² = 1`, nodes not moving: a family of dimension exactly `Q − 3`, so the count would be right.
- A change of statistical family, not a clean-up: the standardized lattice is fixed at the grid's
  spacing, distributions the current fit can take are excluded, and the optimizer needs a
  constrained parametrization (two tilting multipliers solved per evaluation, or a projection).
- Effects: every fit changes; golden rows, AT-DIF-14 and the D43 studies (`13` §8.8) would be run
  again; convergence behaviour unknown.
- Compatibility: none with the current fits.

**Recommendation: P1.** It corrects what A3 found without changing what is fitted or selected. P2
is warranted only if the design wants the hard family for its own sake, and then as a decision.

## 7. Conclusion

- **Confirmed:** the grid moments are not a gauge (§3 (a), (d)); the penalty is a regularizer that
  grows with `n`, not a gauge; `Q − 3` is not the dimension of the implemented ability family,
  which is `Q − 1`.
- **Verified and delimited:** the review's note that a common offset cancels holds in every current
  comparison (§5): the ordering is unchanged mathematically, only each candidate's absolute BIC
  moves, and numerically a decision could differ only at a tie within rounding, while the
  differences can move by rounding. The review did not claim a distortion of the selection, and
  none is attributed to it.
- **Restricted:** "can affect standardized shape" — yes; on the two one-class fits measured the
  penalty moves skewness by at most 0.012, kurtosis by at most 0.064 and the item parameters by at
  most 0.0034. These two cases support no general conclusion on mixtures, on other batches or on
  DIF verdicts.
- **Open:** whether the responses identify all `Q − 1` directions; fitted histograms are rough and
  the histogram is weakly informed; whether the penalty should keep weight 1 per respondent; the
  BIC's use with penalized fits (B1–B3). None is decided here; P1, approved for implementation, is
  implemented (§8) and awaits Astra's review.

## 8. P1 as implemented (after `deb4e5e`, pending Astra's review)

**Changes** (`crates/scoring/src/latent.rs`). `free_params` subtracts 1, not 3, when the shape is
estimated (`:214–218`): a nominal count of the parametrized family, documented as such; with the
shape held (`nodes = 0`) the count is unchanged. `gauge` is renamed `moment_penalty`, its comment
describing an `n`-scaled regularizer (`:828–830`, call `:866`; the tests' calls). The count test
expects 40 histogram parameters (`:1249–1253`). Formulas, gradients, the penalty's intensity, the
grid, the starts, the optimizer, the selection, the stopping rule, thresholds and seeds are
untouched. Golden rows regenerated: `latent.candidates` and `floor.candidates`, three each.

**Comparison with `deb4e5e`** (E: the two golden fits of the latent model, `golden.rs`, n = 1,500,
20 anchors, 8 items, two starts, at most two classes, captured before and after the change).

| Output | Bits | Printed value | Decision |
|---|---|---|---|
| fitted parameters (π, η, histogram, anchors, items, floors, gaps, posterior) | identical (one digest per fit) | identical | — |
| candidates visited | identical: one class, two classes shared and per-class `a` | — | identical |
| selected model, convergence | one class / two classes shared `a`; `Converged` | identical | identical |
| DIF flags at 1.0 | identical (none / the first two items) | identical | identical |
| `bic_gain` | identical: 0 and `0x4038252f5ce03000` (24.145253948910977) | identical | — |
| each candidate's BIC | changed, all six | `+14.6264407741…` | — |

- Each increment equals `2 ln 1500 = 14.626440774180603` within rounding: four are
  `14.626440774176444` (−0.57 ulp of the new value) and two `14.62644077418372` (+0.43 ulp); the
  `ulp` at `4.6–5.0·10⁴` is `7.3·10⁻¹²`.
- Margins of the comparisons, unchanged at the precision reported: on the open batch one class
  beats two classes with shared `a` by 0.523 and with per-class `a` by 50.87; on the batch with
  floors two classes with shared `a` beat one class by 24.15 and per-class `a` by 45.65. In their
  bits the margins between candidates with the same increment are identical, and those towards
  the per-class candidates move by `7.28·10⁻¹²`, the difference of the two increments. Each margin
  is at least `7·10¹⁰` times that.
- Characterization: the pinned records of five studies (`dif-power`, `dtf-error`, `floor-power`,
  `floor-dtf`, `floor-screen`, one smoke task each) are identical line by line, `bic_gain`
  included; `harness.rs::a_record_of_each_kind_is_pinned` passes with its pins unchanged.
- Scope: these results hold for the two golden fits verified; they are no bit-level guarantee
  for any other batch, where a difference could move by rounding in its last bits, and a tie
  within rounding, which would change a decision, was not met.

**Residues assigned to B1–B3:** whether the responses identify the `Q − 1` histogram directions;
how weakly the data inform them; what the moment penalty does to fits, mixtures and verdicts; the
BIC's use with penalized mixture fits.
