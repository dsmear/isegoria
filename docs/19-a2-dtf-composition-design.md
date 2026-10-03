# 19 — A2: DTF across fits and the whole-test claim

| | |
|---|---|
| **Status** | Diagnosis and design, **pending design review**. A2 stays open (`15`). No production code, threshold, golden output or historical result changes. |
| **Baseline** | `docs/phase1-review-alignment` at `16e862c`. Line references are to that commit. |
| **Scope** | `15` A2: the contested pool's cost `D(T)`, a sum of per-fit DTFs, and the guarantee the docs attach to it for a test. B1–B3 (identification, BIC) only where A2 needs them. |
| **Evidence** | **L** read; **D** proved here; **C** recalculated (`sim/dtf_composition.py`); **E** executed. E is labelled *API* (real code on hand-built curves), *fit* (a real latent fit) or *frequency* (none here). |

## 1. What the code computes (L)

- **Per fit.** `ClassCurves::of` keeps the classes with share `π_g ≥ 0.05` (`dtf.rs:158–184`,
  `dif.rs:142`) and builds, by `with_ability` (`dtf.rs:104–133`), the measure
  `μ_F = Σ_{c∈C_F} π̃_c Σ_q w_q δ_{η_c + u_q}`: every counted class's mean plus the fit's
  histogram, masses normalized (`at`, `:135–156`), so the excluded classes' mass is redistributed.
  `with_floors` (`:80–102`, behind `ClassCurves::new`) uses instead normal densities at 41 fixed
  nodes; no persisted path uses it. Curves `P_jc(θ) = c_j + (1 − c_j) σ(a_jc(θ − b_jc))` are
  evaluated for every class at every point.
- **The statistic.** `DTF_F(S) = max_{g,h∈C_F} ∫ |Σ_{j∈S} (P_jg − P_jh)(θ)| dμ_F(θ)`
  (`dtf.rs:194–223`): the unsigned expected-score gap on the number-correct scale, worst pair of
  counted classes, over the batch's estimated population. 0 with one counted class.
- **The pool.** `ContestedPool` holds contested facts grouped by the fit that last measured them
  (`contested.rs:31–36,75–105`). `dtf(T) = Σ_F DTF_F(T ∩ F)`, each term rounded up to `2⁻³²`
  (`:12–17,126–143`); `draw` picks uniformly among the selections with that cost at most the
  tolerance, by an exact dynamic programme over per-fit subsets (`:145–235`). Only contested facts
  enter; active-pool items never do. The `2⁻³²` rounding makes the computed sum an upper bound of
  the real sum of the per-fit values; it says nothing about what those values bound.
- **Persistence.** A results record `ContestedFit` carries `π`, `η`, `a`, `b`, `c`, the histogram
  and the members; it is rebuilt with `with_ability` directly, without the 5% filter
  (`results.rs:51–61,237–254`), so the record must already hold the counted classes.
- **What runs.** `draw` and `draw_from_beacon` are called only by tests
  (`protocol/tests/contested_facts.rs`); `blueprint::assemble_test` fills quotas without any DTF
  (`blueprint.rs:87–121`); no code composes active items and contested facts into a test. The
  composition is an API, not a runtime.

**Claims attached to it.** `01` D38's implementation note: "a test's DTF is bounded by the sum of
its per-fit DTFs" (`01:782–785`); `08` DIF-011 "RESOLVED … drawn into a test only in selections
whose DTF bound … is at most `DTF_MAX`" (`08:122–131`) and its status row (`08:1512`);
`ARCHITECTURE.md:185` "DTF bound (the sum of per-fit DTFs)"; the module doc and `dtf`'s doc
(`contested.rs:1–3,126`). `02` §B.7 (`:596–600,622–643`), `05` [7b] (`:179–184`) and the paper
(`065-revisions.tex:321–328`) already state the whole-test guarantee as open.

## 2. The quantity one would want to bound

A test `T` (active items `A` and contested facts `C`) is answered by a **target population** with
ability `θ` on a **common scale** and reference measure `μ`; the **groups** are a family of
subpopulations `𝒢` (the latent classes the batches find, or any social partition); the **score**
is the number correct `S = Σ_{j∈T} X_j`, unweighted. The target is
`DTF_μ(T) = max_{g,h∈𝒢} ∫ |E[S | θ, g] − E[S | θ, h]| dμ(θ)`.
The docs leave open which population (the test's respondents or each batch's), which groups
(fitted classes are statistical components, `02` §B.3) and which scale (each fit fixes its own:
mean 0 and variance 1 for its first class's shape). The proposition below names each.

## 3. A sufficient proposition (D)

Within one fit, the per-fit statistic is subadditive: pointwise `|Δ(S ∪ S')| ≤ |Δ(S)| + |Δ(S')|`
on the same measure and pairs, so `DTF_F(S ∪ S') ≤ DTF_F(S) + DTF_F(S')` (E API:
`scoring/tests/dtf.rs::the_dtf_is_bounded_and_subadditive`). Across fits:

**Proposition.** Let `T = ⋃_F S_F` be partitioned over fits `F`. Assume
- **H1 (common measure).** For each `F` an increasing affine `τ_F` with `μ_F = μ ∘ τ_F⁻¹`: the
  fit's measure, counted classes renormalized, is the target population's in `F`'s scale.
- **H2 (group representation).** For each `F`, `g` and `θ`, probability weights `λ^F_g(θ)` on
  `C_F` with `E[Σ_{j∈S_F} X_j | θ, g] = Σ_c λ^F_{gc}(θ) Σ_{j∈S_F} P^F_jc(τ_F θ)`.
- **H3 (coverage).** Every item of `T`, active or contested, belongs to some `S_F`.

Then `DTF_μ(T) ≤ Σ_F E_F(S_F)`, with the envelope `E_F(S) = ∫ max_{c,c'} |Δ_cc'(S; t)| dμ_F(t)`;
and if every `λ^F_g` is constant in `θ` (a group is a class, or a fixed mixture of classes),
`DTF_μ(T) ≤ Σ_F DTF_F(S_F)`.

*Proof.* For groups `g, h`, `E[S|θ,g] − E[S|θ,h] = Σ_F Σ_c (λ^F_gc − λ^F_hc)(θ) Σ_{j∈S_F} P^F_jc`.
Two probability vectors differ by `Σ_{c,c'} γ_cc' (e_c − e_c')` with `γ ≥ 0`, `Σγ ≤ 1` (a coupling
of their difference), so fit `F`'s term is at most `Σ γ |Δ_cc'(S_F; τ_F θ)| ≤ max_{c,c'} |Δ_cc'|` in
absolute value. Summing over `F` at the same `θ` and integrating over `μ`, H1 turns each integral
into one over `μ_F`: the envelope. With `γ` constant, `∫ Σγ|Δ_cc'| = Σγ ∫|Δ_cc'| ≤ DTF_F(S_F)`. ∎

**No label matching is needed** for this bound: each fit's maximum over its own pairs dominates
whatever pair the groups map to, so classes of different fits never have to be identified. The
price is conservatism: the worst pairs of two fits may be different contrasts. **No linking is
computed either**: the integral is invariant under a common affine change of curves and measure;
H1 and H2 only require that one `τ_F` exists for both. Anchors with class-invariant parameters
make such a `τ_F` plausible when the anchors are DIF-free; they do not establish H1, which is
about the populations, nor H2, which is about the classes.

**The envelope is needed when group composition varies with ability** (D, C). Three classes at
two equally weighted points, curves `(1, 0, ½)` and `(½, 0, 1)`: the pairs' DTFs are `¾, ½, ¾`, so
`DTF_F = ¾`; the envelope is 1; a group that is class 0 at the first point and class 2 at the
second, against class 1, has gap 1. An API-level example of the bound's logic, not a fitted model.

**What the current APIs guarantee of H1–H3: none.**
- H1: each `μ_F` is one batch's estimated population; nothing makes batches share a population,
  and dropping classes under 5% changes the measure even within a fit.
- H2: classes under 5% are left out (`scoring/tests/dtf.rs::a_class_below_the_share_floor_does_not_count`),
  so a group they carry is not represented; a one-class fit represents no contrast at all.
- H3: active items are not in the cost. Passing the cut `DIF_j ≤ 1.0` is no zero contribution.
- And the estimate: the curves are fitted; T24 measured an upward bias near 0 and noise within
  0.05 of the tolerance (`13` §7.3, §8.7.3); a draw that selects sets by their fitted cost also
  selects their estimation error.

## 4. Counterexamples and evidence

| # | What it isolates | Kind | Result |
|---|---|---|---|
| 1 | the integration measure | E API, `scoring/tests/dtf_composition.rs::one_item_s_dtf_depends_on_the_population_it_is_integrated_over` | one item, `a = 1.25`, gap 1.8, two equal classes: 0.09419 at difficulty 3 (a batch for which it is hard), 0.34381 at difficulty 1 (a population two units abler) |
| 2 | the pool's sum under different measures | E API, `protocol/tests/a2_dtf_composition.rs::per_batch_dtfs_can_admit_a_test_its_population_would_refuse` | two facts, each at difficulty 4 for its own batch: cost 0.06876, drawn; leaning the same way at difficulty 2 on a common abler population: 0.41551 |
| 3 | an active item under the cut | E API, `dtf_composition.rs::an_item_under_the_flag_cut_can_exceed_the_tolerance`; C | gap 0.9 under the cut 1.0, floor 0.2: 0.1698619472 on the 41-node grid (`15` A2's figure), 0.1698619004 by continuous quadrature, over `DTF_MAX` alone |
| 4 | a zero imposed by one class | E fit, `dtf_composition.rs::a_one_class_fit_reads_zero_where_two_items_lean` | `golden.rs`'s open batch (n = 1,500, two items leaning `δ = 0.9`): the fit selects one class, fitted DTF of the two items 0; their true curves 0.78196 |
| 5 | a class left out | E API, existing `dtf.rs::a_class_below_the_share_floor_does_not_count` | a class under 5% does not count and the rest is renormalized |
| 6 | max of integrals against the envelope | D, C (rationals) | `¾` against 1, §3 |

Rows 1–3 and 5–6 are mathematical constructions run through the real code where possible;
row 4 is a real fit with known truth; **no frequency is measured**. Row 2 needs H1 to fail —
batches whose populations differ by two units of ability — which is a construction, not a
measured distribution of batches. Label non-comparability is **not** a defect of its own here:
under H2 the per-fit maximum absorbs it (§3); it costs only conservatism.

## 5. Three directions

**R1 — say what is computed (rectification, no guarantee).** Call `D(T)` the *contested-facts
cost*: the sum of per-batch plug-in DTFs of a test's contested facts, each over its batch's
fitted population and counted classes, an admissibility heuristic for that subset. Remove
"bound" and "the test's DTF" from `01` D38's note, `08` DIF-011 (status no longer RESOLVED for
the test-level claim) and its row, `ARCHITECTURE.md`, and the docs of `contested.rs` (module and
`dtf`). Behaviour, records and selection unchanged.
- Guarantee: none on the test; honest description. Residues: all of H1–H3 and estimation.
- Acceptance: no active document or comment calls `D(T)` a bound on a test's DTF; the existing
  tests pass unchanged.

**R2 — co-measured contested facts (removes the cross-fit sum).** Draw a test's contested facts
only from one fit's members (or re-fit the drawn set jointly before use, as `05` [7b]'s
re-measurement already does at re-validation), so the cost is a single `DTF_F` or `E_F`.
- Guarantee: §3 with one fit, H1 reduced to "the batch's population is the target's"; H2 still
  assumed; H3 still fails (active items); estimation open.
- Data: the pool already groups members by fit; no linking. Effects: `draw` restricted to one
  fit (or a re-fit step), fewer admissible selections, possibly `NoBalancedDraw` more often;
  records unchanged. Cost: the per-fit enumeration only.
- Acceptance: a test with members of two fits is refused (or re-fitted) by a test on the API.

**R3 — a test-level fit (realizes the target under named hypotheses).** Fit the whole assembled
test, active items and contested facts, jointly with anchors on a batch of the target population;
accept it when an upper confidence bound of `E_F(T)` (or of `DTF_F(T)` under constant-composition
groups) is within `DTF_MAX`.
- Guarantee: §3 with one fit and `S_F = T`: H1 by sampling the target population, H3 by
  construction; H2 remains a model assumption (classes capture the groups, mixtures allowed with
  the envelope); estimation covered by the bound's uncertainty, which must be specified.
- Data: one admissible batch (`N_LATENT_MIN` respondents, reliable anchors) per test form; no
  cross-fit linking. Effects: new API (test-level fit, envelope, confidence bound), a form record,
  selection by draw-then-verify. Cost: one latent fit per form and a bootstrap or equivalent for
  the bound; respondents per form.
- Acceptance: on a constructed population with known curves, the accepted forms' true DTF is
  within the tolerance up to the stated confidence; the cases of §4 are refused.

## 6. Recommendation

**R1 now**, since no hypothesis of §3 is guaranteed and the docs still call `D(T)` a bound; it is a
rectification, not a fix of the guarantee. **R3 as the design that realizes the whole-test
guarantee**, to be specified with Astra (the envelope or the class-pair maximum, the uncertainty
bound, how forms are drawn and verified). R2 removes only the cross-fit sum and leaves active
items out, so it is an intermediate step at best. The one product trade-off for the owner arises
with R3: whether a whole-test guarantee is worth a calibration batch of respondents per test form.
`DTF_MAX` stays as it is; no empirical margin replaces the missing hypotheses.

## 7. Open

The groups the guarantee is about (latent classes, social groups, both); H2's plausibility for
social groups (classes are statistical components); the uncertainty treatment of a fitted DTF and
of selection on it; whether small classes should count in a guarantee (they are excluded because
their parameters are poorly identified, `02` §B.3); privacy of contested facts (`05` [7b], `10`
T69), untouched here.
