# Isegoria — Parameter register (T25, step 4)

| | |
|---|---|
| **Purpose** | The five items `docs/07` §13 asks of every threshold in `docs/02`'s parameter table — why it exists, how it is calibrated, how much the mechanism depends on it, what breaks if it moves, where its value is fixed — so that no threshold is an accidental constant (SC-1, `docs/08` §16.1). |
| **Derived from** | `docs/02` (the parameter table, §A.1–§A.3), `docs/13` §7.4 and §8.7.4 (the measurements), `docs/01` (the decisions), `docs/05` [4]–[5b], `docs/10` T25 step 4. |
| **Status** | Level A written (2026-10-01). Level B waits on the supplement's DIF studies run again on the corrected model (`docs/13` §8.8); Level C gets its procedures next, its values provisional until the pilots (T27). |

## How to read an entry

**Correction dependency.** The guarantee and model findings in
[`15`](15-phase1-review.md) precede interpretation of the affected calibration
procedures. In particular, choosing `DTF_MAX` cannot prove a cross-fit bound, and
choosing an exploration rate cannot repair the information assumptions of properness.
The register records provisional values; it does not close those findings.

- **Reason** — what the threshold decides, and why the mechanism needs it.
- **Calibration** — the procedure that sets the value: the data, the study, the criterion.
  The criterion is an **operating point**: the error rates the network accepts at the
  threshold — near `τ`, the share of items passed that should fail and failed that should
  pass. It is the owner's choice (`docs/10` T83), recorded in `docs/01` before the closed
  calibration pilot (T27); until then every value here is provisional, and its procedure says
  what will set it.
- **Sensitivity** — what the studies measured as the value moves, inside their regime
  (`docs/13` §8.7: outside the populations they draw, the behaviour is not established);
  "not measured" where nothing has.
- **If it moves** — what fails first, in each direction.
- **Location** — the constant in the code, the specification's section, the decision that
  set it. A value changes only through a decision record, and the golden rows are
  regenerated wherever they pin it (`docs/07` §13, INV-7).

## Level A — the bridging gate

### `τ` — the bridging threshold (0.80, provisional)

- **Reason.** The opinion filter's bar (D32): a question passes when the mean of its two
  sides' predicted approval, each side counted once, reaches `τ`, on the probability scale
  the reviewers declare.
- **Calibration.** On the calibration pilot: (1) a probe set of consensus and partisan
  items, as the mirror design of `docs/13` §3.2; (2) the pilot's ratings fitted, and its
  scale read from the fit — reviewers, camp split, ratings per reviewer, residual noise;
  (3) `bridging-sweep` at that scale for `τ` from 0.70 to 0.90; (4) `τ` the lowest value
  whose rates meet the operating point, and at which no partisan probe of the pilot passes.
- **Sensitivity** (`docs/13` §7.4: 100–800 reviewers, camps 50/50 to 80/20). From 0.70 to
  0.90, without the band, the robust score passes at most 0.03% of the items 0.05 or more
  below `τ` with 100 reviewers, 0.003% with 200 and none with 800; at 0.80 it fails 3.6%,
  0.7% and 0.01% of those 0.05 or more above. On the reference simulation the consensus
  items score 0.83–0.86, the partisan ones 0.53–0.56, the mildly partisan one 0.70.
- **If it moves.** Up: consensus items near it fail or go to the band, and the pool, which
  keeps about three items in ten, shrinks; past 0.83 the reference simulation's consensus
  items start failing. Down: mildly partisan items reach it — the reference's at 0.70 —
  and the gate asks less of the cross-side approval D32 requires.
- **Location.** `protocol::gate::TAU`; `docs/02` §A.3; D32 (T49).

### `ε` — the half-width of the uncertainty band (0.02, provisional)

- **Reason.** Scores near `τ` are decided by noise: an item within `ε` of `τ` goes to an
  extra round instead of being decided by its exact value (D26).
- **Calibration.** With `k_extra` and the panel's size `k`: `bridging-extra` at the pilot's
  scale and first panel, over `ε` and `k_extra`; the pair that meets the operating point at
  the fewest extra reviews per item. Its floor is about three times the bootstrap spread of
  `S_j` at that scale — at most 0.006 on the reference fixtures, hence 0.02.
- **Sensitivity** (`docs/13` §8.7.4: first panels of 7 and 11, noise 0.15). At 0.02 with
  `k_extra` = 4, 6.4–11.7% of the items 0.02–0.06 below `τ` pass and 39.9–66.5% of those
  above fail, at 0.5–0.9 extra reviews per item. At 0.06 the false failures fall by 12–16
  points and the false passes rise — to 16.1–22.1% with no extra reviewer, 7.1–12.1% with
  11 — at 3.8–6.1 extra reviews per item with 11. With many ratings per item (§7.4) the band
  holds 13.7–20.0% of the consensus items, and from 200 reviewers up no consensus item
  0.05 or more below `τ` passes.
- **If it moves.** Wider: more items reviewed again, fewer false failures, more false passes
  unless the extra round grows with it. Narrower: items thousandths apart are decided by
  noise.
- **Location.** `protocol::gate::EPS`; `docs/02` §A.3, `docs/05` [5]; D26.

### `k_extra` — the band's extra panel (4, provisional)

- **Reason.** The reviewers drawn outside the first panel for an item in the band; their
  ratings join the first panel's before the score is decided again against `τ` (D26, T60).
- **Calibration.** With `ε`, as above: `bridging-extra` runs `k_extra` from 0 to 11.
- **Sensitivity** (`docs/13` §8.7.4). From none to 11 extra reviewers the false passes near
  `τ` fall from 8.7–14.1% to 4.5–8.0%; the false failures stay (38.3–64.6% and
  40.3–65.9%): they are decided in the first round, below the band.
- **If it moves.** Up: fewer false passes, at more reviews per band item, at most
  `K_EXTRA_MAX` = 11. Down: the band is decided again on nearly the same ratings; at 0 it is
  only a delay.
- **Location.** `protocol::review::K_EXTRA`, bounded by `protocol::lifecycle::K_EXTRA_MAX`;
  `docs/02` §A.3, `docs/05` [5]; D26 (T60).

### `γ_appeal` — the side gap that makes a rejection appealable (0.25, provisional)

- **Reason.** A question rejected with its sides far apart was rejected for polarization,
  not for a defect, and may appeal to the evidence (`docs/05` [5b]): the mitigation of the
  true-but-divisive false negative (`docs/02`, structural limit 1).
- **Calibration.** On the pilot's probes: between the gaps of the consensus probes and those
  of the partisan ones, read on the pilot's fit; `bridging-sweep` at the pilot's scale gives
  the share of partisan items it makes eligible. What an appeal costs — a stake and a place
  in a pilot (D27) — bounds how low it may go.
- **Sensitivity** (`docs/13` §7.4). At 0.25, 99.4–100% of the partisan items are eligible.
  On the reference simulation the consensus items' gaps are at most 0.02, the mildly
  partisan item's 0.30. Not swept.
- **If it moves.** Up: true-but-divisive items lose the appeal, their one path past the gate.
  Down: items rejected for a defect, their gaps noise, appeal too, spending stakes and pilot
  capacity.
- **Location.** `protocol::gate::APPEAL_GAP`; `docs/02` §A.3, `docs/05` [5b]; D32 (T49:
  the gap replaced `|f_j|`, BRIDGE-009), D27.

### The side floor — the fewest reviewers a side holds (5%, rounded up, provisional)

- **Reason.** Without it a side can be a few reviewers far out on the axis: in T24's first
  pass, sides of 2 and 5 reviewers of 800, and in a bootstrap subsample one of 1 of 200,
  which dropped an item's robust score from 0.91 to 0.56 (D42).
- **Calibration.** Below the smallest minority the network must count as a side, above the
  clusters of outliers its fits show: on the pilot's fit, the share of the minority camp
  and the size of the outlying groups at the axis' ends; the floor between them, checked
  with `bridging-sweep` at the pilot's camp split.
- **Sensitivity.** Not swept. At 5% two camps of distinct positions are separated whenever
  each holds 5% or more (`docs/02` §A.3); with camps up to 80/20 the side-balanced score
  leaks 0.00–0.06 of the camp-size effect on average per cell (`docs/13` §7.4).
- **If it moves.** Up: a real minority below it is not a side — it joins the majority's, and
  the score turns majoritarian for it. Down: outliers become a side, and the score swings
  with them.
- **Location.** `scoring::bridging::MIN_SIDE_PER_MILLE`, read by `side_floor`; `docs/02`
  §A.3; D42 (T71).

### `MIN_COVERAGE` — the fewest ratings from the less-rated side (1, provisional)

- **Reason.** An item that no reviewer of one side rated gets that side's mean by
  extrapolation: a partisan item passed at 0.972 with no minority rating (D42). Below the
  floor the gate sends the item to the band's extra round whatever its score, and the
  re-decision cannot pass it (`docs/05` [5]).
- **Calibration.** A `bridging-sweep` run at the pilot's scale that records each item's
  coverage — its records keep today only whether an item fell under the floor: the score's
  error as a function of the coverage; the floor is the smallest coverage past which the
  error is no larger than at full coverage, weighed against the share of items sent to
  review.
- **Sensitivity** (`docs/13` §7.4). At 1 the gate sends 0.1–0.4% of the items to review,
  only with 100 reviewers in camps of 80/20 rating 5 items each. Not swept.
- **If it moves.** Up: more items reviewed again; an item a minority rated little waits for
  more of its ratings. At 0: extrapolated passes come back.
- **Location.** `protocol::gate::MIN_COVERAGE`; `docs/02` §A.3; D42 (T71).

### `λ_b / λ_f` — the fit's regularization (0.15 / 0.03)

- **Reason.** The penalties on the intercepts and on the positions (`docs/02` §A.1–§A.2):
  with `λ_b ≫ λ_f` the fit explains approval by the axis first, so that only cross-side
  approval is left to the score; `λ_f` keeps the axis from overfitting sparse ratings.
- **Calibration.** `bridging-lambda` at the pilot's scale: `λ_f` the smallest value at which
  the axis is recovered — the worst run's `|corr|` — as the operating point asks; `λ_b`
  anywhere in the flat region around it.
- **Sensitivity** (`docs/13` §8.7.4). Within a factor of 3 of the pair the verdicts move by
  at most 7 points. `λ_f` sets the axis' recovery: `|corr|` 0.80–0.91 on average at 0.01,
  0.88–0.93 at 0.03, 0.92–0.95 at 0.09, the worst run 0.36, 0.53 and 0.72. `λ_b` moves
  nothing measurable.
- **If it moves.** `λ_f` down: the axis is lost on sparse ratings, the sides are cut wrong,
  and the score reads the camps' sizes. `λ_f` up, past the range measured: the positions
  shrink and approval moves back into the intercepts, which §A.2 keeps out. `λ_b`: nothing
  measured within a factor of 3; beyond it, not measured.
- **Location.** `scoring::bridging::BridgingParams::default`; `docs/02` §A.1; the golden
  rows pin the fit at these values (AT-BR-04).

### `d` — the number of axes (1)

- **Reason.** The latent positions per reviewer: one axis carries one polarization; a
  population split on two would need a second.
- **Calibration.** On the pilot: fits with one and two axes compared on held-out ratings
  and on the second axis' stability across seeds; a second only if it is stable and the
  score's sides are first defined on it.
- **Sensitivity.** Not measured: every study fits one axis.
- **If it moves.** To 2: the sides of D32 and D42 are cut on one axis, so a second needs the
  score redefined — a decision, not a change of value.
- **Location.** `scoring::bridging::Fit` (one position per reviewer, `f_u: Vec<f64>`);
  `docs/02` §A.1.

### `k` — the first panel's size (7–11)

- **Reason.** The ratings an item gets before the gate, from a panel drawn at random and
  stratified on `f_u`, odd (`docs/05` [4]).
- **Calibration.** With `ε` and `k_extra`, on `bridging-extra`'s first panels: the smallest
  panel that meets the operating point within the network's review capacity.
- **Sensitivity** (`docs/13` §8.7.4). A first panel of 11 instead of 7 lowers the false
  failures by 4–12 points; camps of 80/20 raise them by 9–18; the number of reviewers in
  the epoch changes little, since an item's score rests on its panel.
- **If it moves.** Down: the score's noise grows — more items in the band, more decided
  wrongly. Up: more reviews per item, fewer items reviewed per epoch.
- **Location.** No constant: the protocol takes the panel its caller draws (`k` in
  `review::assign_reviewers` and `assign_diverse`); `docs/02`'s table, `docs/05` [4].

## Level B — the empirical validation

To be written by step 4 once the supplement's DIF studies have run on the corrected model
(`docs/13` §8.8): `N` of the pilot's two stages, `a_min`, `R_PBIS_MIN`, `B_ABS_MAX` and
`C_EXCESS_MAX`, the DIF cuts (`|β₂|`, `Δ_MH`, `DIF_j`) and a cut on `a_gap` or none, the
floor prior's weight, `KR20_MIN` or a minimum number of anchors, `N_LATENT_MIN`, `DTF_MAX`.

## Level C — reputation and coordination

To be written by step 4, with procedures and values provisional until the pilots (T27):
`γ` and `k₀`, the CUSUM's `k` and `h`, `w_max`, `N_PROBATION`, the implemented
reputation decay time `T` (half-life `T ln 2`; the intended convention is a decision,
`15` C6), the honeypot and exploration rates, `α`, `min_shared`, `ρ_min` and `p_max`.
