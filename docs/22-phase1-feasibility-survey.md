# 22 — Phase 1: are the archived paper's objectives reachable? A literature survey

| | |
|---|---|
| **Purpose** | Record, for later reference, what the published research says about the four objectives of the [archived paper](../archive/phase1-forecast-pilot/paper/README.md) — bridging, latent-class DIF with a whole-test guarantee, a proper reviewer reputation used as weight, resistance to coordinated blocs — and whether the [answer review](05-question-lifecycle.md#the-answer-review-01-d49d52) that replaced them leaves known room for improvement. |
| **Status** | **Informative, not normative (2026-11-11).** It decides nothing, closes or reopens no finding of [`15`](15-phase1-review.md), changes no contract or parameter and authorizes no experiment. It has had no design review. |
| **Method** | A literature survey run on the owner's request by AI research agents (Claude), one per area, synthesized by another. No source was re-read by a person. The working notes, with a verification level for every source, are kept outside the repository. |
| **Evidence** | **Theorem**: proved under stated assumptions. **Empirical / simulation**: measured in a stated regime. **Survey's inference or computation**: derived during the survey, not taken from a paper. Each claim below says which it is, and how far its source was read. |
| **Scope** | The state of research up to November 2026, read against the fixed constraints: anonymity with no attribute data, no money, one non-rotatable pseudonym per role, deterministic scoring, no reward for agreeing with the majority ([`CLAUDE.md`](CLAUDE.md)). |

## Summary

**Within the fixed constraints the mechanism is not markedly improvable.** On each of the four
original objectives the core of the obstacle is an identification limit: two worlds that differ in
the very property to certify produce the same data, so no amount of data of the same kind removes
it.

- Ratings alone do not separate a true but divisive item from a partisan one.
- Anonymous responses alone do not certify the absence of bias against a real group, and do not
  compose the DTF of separately fitted batches.
- A sincere faction and a cartel are the same input.
- A correct consensus and a shared error have the same distribution.

What stays reachable are weaker and costlier versions of the objectives: bridging as a *measure* of
cross-camp approval (with many co-rated items and both camps present), the detection of departures
from the model with thousands to tens of thousands of respondents per batch, properness in
expectation under hidden randomization, linear utility and no stake in the outcome, and an exact
bound on the capture of a panel, the hypergeometric tail.

The answer review (12 drawn, equal weights, three quarters of at least 9 usable answers) already
sits on the provable frontier for the fourth objective and, with homogeneous independent reviewers,
is the maximum-likelihood rule; weighting by reliability loses every proved advantage when each
reviewer answers few questions. The improvements the literature supports are few and concern
efficiency. Everything that promises more needs ground truth, a scored second signal or group
information, and each of these meets a fixed constraint.

**General caution.** Many sources were read at abstract level only, the 2026 preprints are
unverified, and almost every "unreachable" entry below is an argument made during the survey, not a
published theorem. Recheck a source before citing it in a decision record.

## 1. One argument of observational equivalence runs through the four objectives

The paper asked for a non-majoritarian acceptance filter, a psychometric verdict with a guarantee
for the whole test, a reviewer reputation in which honesty is the best strategy, and resistance to
coordinated blocs of real persons. The answer review replaced all four (`01` D47–D52). The question
is whether that fallback was forced by the mathematics or by a design that could be bettered. The
five strands of the survey converge on the first answer, by the same scheme in every area: a rule
that is a function of the observed data alone returns the same output in two worlds that produce
the same data.

The scheme exists as a **published theorem** in three cases:

- votes and confidences do not identify the correct answer even with ideal Bayesian respondents
  (main text read, the supplementary theorems not)
  ([Prelec, Seung, McCoy 2017](https://gwern.net/doc/statistics/prediction/2017-prelec.pdf));
- labelling algorithms with no gold standard that treat agreement as evidence always admit
  equilibria in which everybody reports a common prejudice (abstract and a secondary source only)
  ([Della Penna & Reid, arXiv:1204.3511](https://arxiv.org/abs/1204.3511));
- group disparities are not identifiable in general when the protected class is unobserved
  (abstract verified; proved for classifiers, not for DIF)
  ([Kallus, Mao, Zhou, arXiv:1906.00285](https://arxiv.org/abs/1906.00285v2)).

In the other cases that matter here — true but divisive against partisan, DIF with respect to an
unobserved group stated in psychometric terms, a sincere faction against a cartel — **no published
theorem was found**. They are elementary arguments of the survey, consistent with what the
literature concedes as a design limit, and are presented as such.

**How far the sources were read.** In full, among others: Blair et al., Selvam et al. (its first
100,000 characters), Truong et al., Chen & Kash, Witkowski et al., Freeman et al., Hudson, Gao,
Wright & Leyton-Brown (arXiv v1), Jecmen et al. 2020 and 2025, FRAUDAR, Wallin, Chen & Moustaki,
Wallin & Huang, Hashimoto et al., Karger, Oh & Shah, and Gao, Lu & Zhou. **At abstract level or
through secondary sources only:** almost all 2026 preprints, Bouchaud & Ramaciotti, Chuai et al.
(a search snippet), and, through model-generated summaries, Frongillo et al. 2021,
Roughgarden & Schrijvers, Kong 2020 and Shnayder et al.; the jury-theorem literature through
Wikipedia. Several classical psychometric sources (Rost, Lindsay–Clogg–Grego, Shealy–Stout, Raju et
al.) are cited from memory with a DOI and were not re-read. The Community Notes documentation was
read once, automatically, on 2026-11-11.

**The survey's computations** assume a binary reduction (an answer coincides with the key or does
not) and independent or exchangeable answers. They illustrate sensitivity; they do not estimate a
real pool. Recomputed while writing this record: the effective thresholds of §6.3, the acceptance
of a wrong key at `p = 0.3`, the capture and blocking columns at 9 of 12 in §5.1, the leak shares
and the absent-camp probabilities of §2, and the regret figure of §4.3. The other figures are as
the survey's notes give them.

## 2. Objective A — bridging measures cross-camp approval, but not with 12 ratings an item

**The internal obstacles** are the paper's *Majoritarian leak*, *Batch relativity* and
*Observational equivalence* propositions (its §3; the side-balanced score of `01` D32 and D42
answers the first two).

### 2.1 The obstacles are consistent with the literature, which does not state them

No work published up to November 2026 states or proves a theorem on the identifiability of the
Community Notes intercept, or quantifies the leak toward the majority as a function of the
penalties. The nearest material is a closed form (one-dimensional factor, the raters' parameters
held fixed, not a theorem) in which the intercept is an artefact of ridge regression that can even
**rise after a "Not Helpful" rating** from an extreme rater of the same side
([Selvam et al. 2026, a preprint with co-authors from X](https://arxiv.org/abs/2607.01824)).

From the same set-up the survey derives, as its own computation, that if all of an item's raters
sit at one factor value `x̄` the penalty alone splits the one-sided surplus, the intercept taking
the share `λ_f / (λ_f + λ_i x̄²)`: with the documented `λ_i = 0.15`, `λ_f = 0.03`, **56% at
`|x̄| = 0.4`, 44% at 0.5, 17% at 1.0**. This reproduces the leak qualitatively, with the reserve
that production rescales the penalties.

Indirect confirmation comes from the operator's own trajectory: since 2025 the documentation
requires at least 5 raters per side and net minimums per side, and from 2025–2026 a scorer that
approximates a geometric mean of helpfulness along the factor spectrum — corrections that would be
needless if the intercept were balanced by itself
([Community Notes documentation](https://github.com/twitter/communitynotes/blob/main/documentation/under-the-hood/ranking-notes.md)).

Batch relativity **is studied by nobody**. Community Notes differs in the relevant detail: it
penalizes the global mean and fits the whole corpus, so "the scores sum to zero in a batch" belongs
to this project's variant, not to production. The nearest relative is the dependence of a note's
status on who else is in the fit: 30.2% of displayed notes later lose their status (a figure seen
in a snippet only) ([Chuai et al. 2026](https://arxiv.org/abs/2601.14002)).

### 2.2 Reachable: a label-free measure, at the cost of many co-rated items

An axiomatic theory now exists. "The Structure of Bridging", by Blair, de Raaij, Procaccia,
Rubin-Toles, Shah, Si and Wang (text read; an undated manuscript on an author's page, its venue not
confirmed), proves that a bridging score **with no group labels** is well defined: it is the mean,
over the two-way splits the other items induce, of an evaluator that depends only on the approval
rates in the two groups and on their sizes (Theorem 1), and one of its forms equals the mean
pairwise disagreement, on the other items, among those who approve (Theorem 6)
([Blair et al.](https://www.cs.toronto.edu/~nisarg/papers/bridging.pdf)).

The same work proves that "bridging" forks: the Remesh-style minimum is maximized at equal approval
*rates*, the Polis-style product at equal *numbers* of approvers (Theorem 2), and the size-weighted
mean with `p = 1` collapses into plain majority approval. Avoiding majority rule is therefore **a
choice of axiom**, not an automatic property.

The cost is in the data: the estimation guarantee needs a number of voters of order
`ln(m/δ) / (ε² γ³)`, with a sufficient constant of 9000, which the authors themselves judge possibly
infeasible at the platforms' present scale (Theorem 10). What deployed bridging gives in practice
is high precision and low recall: about 10% of notes pass (abstract only)
([Arjmandi-Lari et al. 2025](https://arxiv.org/abs/2510.00650)), and in a simulation with known
truth more than 40% of genuinely helpful notes stay unpublished with no attacker at all
([Truong et al. 2025](https://arxiv.org/abs/2511.02615)).

At a panel of 12 none of these guarantees is available:

- no sample-complexity result exists for the intercept or for cluster-based bridging at about 10
  ratings;
- the production rule "at least 5 per side" would fail almost every small panel;
- by the survey's computation, if the smaller camp is 20% of the population, a panel of 9 holds
  none of its members with probability about 13%, and at most one with probability about 44%;
- fewer than 10 strategic ratings lift **7.0–10.7% of the notes below the median** over the
  threshold, by an attack in two stages (build a sparse latent position, then support the target
  together) that the authors attribute to a property of the factorization, not to a defect
  ([Selvam et al. 2026](https://arxiv.org/abs/2607.01824)).

Random assignment, which the project already uses, is the only structural mitigation the 2026
literature recommends.

### 2.3 Unreachable, and open

Separating a true but divisive item from a partisan one is unreachable from ratings alone, by
observational equivalence: an identification limit. The literature concedes it as a design stance —
the axiomatic notion states that it does not distinguish the sources of disagreement, and Ovadya
and Thorburn judge bridging too neutral, by itself, where information quality matters
([Ovadya & Thorburn 2023](https://knightcolumbia.org/content/bridging-systems)) — but does not
prove it as a theorem.

Of the same kind, and equally the survey's arguments: a camp absent from the panel cannot be
balanced (the honest outcome is "insufficient cross-camp participation"), and feigned positions
cannot be resisted — a rater's side is its history, and a coalition that builds histories on the
other side is indistinguishable from genuine cross-camp agreement. Of this whole group the only
cited theorem is the non-neutrality with respect to the camps' sizes (Blair et al., Theorem 2). The
survey adds an uncomfortable inference: a side-balanced rule is *more* sensitive than simple
majority to a small coalition that feigns the minority side.

Open: a theory of identifiability and leak for penalized-intercept rules; the sample complexity of
any bridging decision at 7–12 ratings; a manipulation bound, or an impossibility, with one identity
per person and random assignment; and the choice of the axis to bridge — on Community Notes a
second latent axis changes the outcomes, the published share falling from 71.5% to 11.7% as
disagreement on it grows (abstract only)
([Andreou & Sirivianos 2026](https://arxiv.org/abs/2609.21496)).

## 3. Objective B — latent classes flag departures from the model, not bias against real groups

**The internal obstacles** are [A2](15-phase1-review.md#a2-current-state-design-decision-deferred)
(per-fit DTF does not compose into a whole-test bound), A8 (weak detection of one biased item), the
spurious classes of `01` D25 and D43, and the sample sizes of [`13`](13-characterization.md) §8.7.

### 3.1 Reachable: three weaker claims, with N in the thousands

The only family of methods with no labels is "latent classes plus IRT within each class". Its
recent L1-penalized version (Wallin, Chen & Moustaki, *Psychometrika* 2024, text read) buys
identification with the assumption that DIF is sparse and **proves no theorem** of identifiability
or consistency specific to the latent case. Its simulations are far kinder than this project's
regime: N = 1,000 and 5,000, 25 or 50 items, 40% of the items with DIF of 0.5–1.5 logits, no
guessing parameter, the number of classes known, data generated by the fitted model. There the
items with DIF are recovered well (true-positive rate about 0.95–0.99) while the classification
error of persons stays at 7–26% with two classes and 35–40% with three
([arXiv:2305.00961](https://ar5iv.labs.arxiv.org/html/2305.00961)). The method's authors warn that
the classes "should not be interpreted as demographic groups" and that it is an exploratory tool
([Wallin & Huang 2026, preprint](https://arxiv.org/html/2601.17612v2)).

Under those reserves the survey finds three claims that can be sustained:

1. A *detection* claim per batch: "no latent-class departure of size at least `δ` involving at
   least two items was found, with power `1 − β` at this N" — assuming sparse DIF, invariant
   anchors, a correct response curve within each class (its lower asymptote included), a
   well-specified ability distribution and local independence. The power must be set by simulation
   in the exact design; no theory supplies it.
2. A *model-fit* claim with no groups: essential unidimensionality and local independence, by
   conditional-covariance tests of the DIMTEST and DETECT kind (asymptotic theory proved; seen
   through a secondary source)
   ([Stout et al.](https://conservancy.umn.edu/items/2e7b3368-aff5-406a-bd71-6d8b6972df43)).
3. A DTF between the fitted classes when all of a test's items are administered to one sample,
   readable only as the scores' robustness to the estimated heterogeneity.

The cost the literature states is more than 2,000 respondents for a stable fit of dichotomous
mixture IRT models, more with more classes (secondary source)
([Sen & Cohen](https://par.nsf.gov/biblio/10345698)). This project's own figure — about 12,000
with a floor of 0.2, and failure for true/false (`13` §8.7.1) — **has no external counterpart**: it
agrees in direction with the survey's arithmetic (an item with `c = 0.5` keeps about a third of the
information of an item with no guessing at its midpoint), but no published study predicts it.

### 3.2 Unreachable: the whole-test guarantee is an identification limit

Three things are out of reach with no group information, and more data does not bring them nearer.

- **Certifying an item or a bank as unbiased with respect to a real social group.** Fitted classes
  and manifest groups are weakly related (an empirical result, seen through its abstract)
  ([Cohen & Bolt 2005](https://scholarhub.vn/publication/A-Mixture-Model-Analysis-of-Differential-Item-Functioning/37d43a43-9e43-4dd5-bde0-8e953b5af257)).
  The guarantees of "fairness without demographics" do not transfer: Hashimoto et al.'s bound needs
  an observable loss per person and a minimum group mass as inputs (text read)
  ([arXiv:1806.08010](https://arxiv.org/html/1806.08010)), while DIF is a difference conditional on
  a latent trait. The survey adds its own argument: a worst-case guarantee over all subgroups is
  vacuous, since those who missed an item form a subgroup on which its success probability is zero.
  A bridge assumption that cannot be tested would be needed.
- **DIF uniform over every item of a batch**, indistinguishable from a difference in mean ability:
  proved by reparametrization, with or without labels
  ([arXiv:2110.11112](https://arxiv.org/abs/2110.11112)).
- **The composition of DTF across batches — A2's obstacle.** In the DFIT framework DTF is additive
  only because labels supply the same focal group, a common metric and the same ability
  distribution in every batch (Raju et al. 1995, cited from memory)
  ([DOI](https://doi.org/10.1177/014662169501900405)). Class `k` of batch 1 and class `k` of
  batch 2 are unrelated objects, the sign of DIF is defined only against an arbitrary reference
  class per batch, and cancellation across batches stays undefined. **This is an inference from
  the definitions:** the survey found no article that poses the problem for latent classes, and no
  impossibility theorem stated in psychometric terms.

Two further obstacles are limits of power, not of identification: a single biased item per batch
leaves a trace only in a perturbation of its own response curve (an inference from Shealy and
Stout's framework, cited from memory), and sparse DIF on true/false items.

Spurious classes are well established by simulation when ability is not normal
([Bauer & Curran 2003](https://pubmed.ncbi.nlm.nih.gov/14596495)) or the within-class family is
wrong
([Alexeev et al. 2011](https://iro.uiowa.edu/esploro/outputs/journalArticle/Spurious-Latent-Classes-in-the-Mixture/9984371266202771)),
and at thesis level for local dependence; for an unmodelled guessing floor and for error in the
ability proxy only analogues were found, no dedicated study. The identifiability of the 3PL is
itself partly disputed: the negative result of San Martín, González and Tuerlinckx (2015) is
contradicted by Wu (2016) and should not be cited in its original form
([Wu 2016](https://resolve.cambridge.org/core/journals/psychometrika/article/note-on-the-identifiability-of-fixedeffect-3pl-models/FE5B2CFD8A44400477335BC01A0D4E2D)).

### 3.3 The open environment

`01` D45's open environment worsens the picture by construction. Unproctored scores were already
higher by Δ = 0.20 before generative AI (a meta-analysis, secondary source)
([Steger et al. 2020](https://econtent.hogrefe.com/doi/10.1027/1015-5759/a000494)). Where every
resource is allowed the trait is assisted performance, and tool use becomes a nuisance dimension
that a mixture model will tend to find as its dominant class, in competition with group DIF. Since
the project does not infer tool use, the two cannot be separated.

Open: identifiability and consistency for 2PL/3PL mixtures with unknown classes and anchors;
post-selection inference for latent DIF; a psychometric formulation of fairness without
demographic data; the linking of latent classes across separate calibrations; a measurement theory
of assisted answers.

## 4. Objective C — honesty is optimal only for a myopic, neutral reviewer who does not know the draw

**The internal obstacles** are A1 (the beacon fixes exploration before the reports) and A12 (the
weight is a nonlinear function of the score), both in [`15`](15-phase1-review.md).

### 4.1 The solved case is narrow, and every qualifier is needed

When the observation of the outcome depends on the decision, strict properness is solved only in a
restricted model. Theorem 4.1 of Chen and Kash (text read) characterizes the proper rules for a
randomized decision rule: the score is divided by the probability of the realized action — exactly
inverse-propensity weighting — and a strictly proper rule exists if the randomization has full
support (Corollary 5.1). The assumptions are one expert, risk-neutral, interested in the score only
and indifferent to the decision
([Chen & Kash 2011](https://dash.harvard.edu/server/api/core/bitstreams/7312037c-918b-6bd4-e053-0100007fdf3b/content)).
For deterministic rules and one expert, strict properness is impossible (Othman and Sandholm, seen
through a secondary source).

A1's obstacle **follows as a corollary**: whoever knows the draw faces a deterministic rule. The
survey found no published theorem with that statement; the support is the corollary and the timing
condition that Witkowski et al. state in a discussion section, not as a numbered theorem: the
mechanism's randomness must not be realized until every prediction has been reported
([Witkowski et al. 2023](https://arxiv.org/abs/2101.01816)).

The price is variance. With an exploration probability `ε`, every explored item enters with weight
`1/ε`; by the survey's arithmetic, at `ε = 0.05` one explored item weighs 20 and dominates an epoch
of 5–10 items. Every practical remedy — clipping, a cap, self-normalization — is a nonlinear
transformation and brings the improperness back. And if the reviewer has a stake of its own in the
outcome, exact properness needs that utility to be known (Boutilier, abstract only)
([arXiv:1106.2489](https://arxiv.org/abs/1106.2489)), which anonymous reviewers exclude.

### 4.2 Reputation as future weight breaks properness, with published counterexamples

A12's obstacle matches published results, read in full. In Freeman, Pennock, Podimata and Wortman
Vaughan (ICML 2020):

- the Multiplicative Weights update with a proper quadratic loss **is not incentive-compatible**
  even for myopic experts who care about their normalized weight (Example 2.1);
- the linear update WSU is, for myopic experts, with regret at most `2√(T ln K)` (Theorem 3.1), but
  **is not for forward-looking ones** (Theorem D.1, a finite counterexample in which reporting
  0.699 for 0.7 raises the expected weight);
- the only algorithm exactly compatible for forward-looking experts, ELF-X, rests on lotteries and
  has no proved regret bound ([Freeman et al. 2020](https://arxiv.org/abs/2002.08837)).

Witkowski et al. also prove that no deterministic mechanism selecting one winner is strictly
compatible (Theorem 1). The structural reason, an inference of the survey, is that properness in
expectation survives only positive affine transformations of the score: a map with a nonlinear
transfer, a cap and a change detector is farther from linear than those that already have
counterexamples, and no theorem covers it.

Two qualifications keep the negative result in proportion. The counterexamples are tiny (the third
decimal; within `4η` for Hedge with a small step, by Frongillo et al., read through a summary)
([arXiv:2102.08358](https://arxiv.org/abs/2102.08358)), so the defensible claim would be
*approximate* honesty, which would need a derivation of its own for the specific map. And no
general impossibility theorem was found for deterministic continuous weight maps: the state is "no
proof, explicit counterexamples for the standard maps". The non-rotatable pseudonym, with its long
horizon, pushes reviewers toward the forward-looking regime, away from the one in which the clean
theorems hold.

### 4.3 With 5–10 reports a period no guarantee says that reputation tracks quality

Properties in expectation hold at any sample size; every guarantee of accuracy, ranking or regret
is vacuous at 5–10 observations. By the survey's arithmetic on the cited bounds, WSU's bound with
10 periods and 9 experts is about 9.4 against a trivial maximum of 10, and ELF's upper bound for
telling reviewers apart with 9 participants, `ε = 0.1` and `δ = 0.1` is of the order of 23,000
events.

The route without verification offers no way out. Peer-prediction mechanisms reward statistical
dependence with peers, not agreement with the truth. DMI's dominance needs at least `2C` tasks per
agent ([Kong 2020, read through a summary](https://arxiv.org/abs/1911.00272)); two 2026 preprints
seen in abstract declare that bound tight ([Kong 2026](https://arxiv.org/abs/2606.14225);
[Aznag et al. 2026](https://arxiv.org/abs/2608.16838)). In simulation the mechanisms robust on
paper measure individual quality badly, and those that measure it well are manipulable (Burrell and
Schoenebeck, secondary source, venue not verified)
([arXiv:2108.05521](https://arxiv.org/abs/2108.05521)).

With ground truth on a subset, by contrast, rewarding agreement with the trusted outcome alone
makes honest effort a dominant strategy at a checking rate equal to the cost of effort over the
reward gap (Lemma 1, text read) ([Gao, Wright, Leyton-Brown](https://arxiv.org/abs/1606.07042)); a
2024 work seen in abstract qualifies the comparison with peer prediction
([Xu et al. 2024](https://arxiv.org/abs/2402.13567)). With no money the gap is bounded and that
rate cannot be lowered by paying more — a reading of the survey, since **no source treats
reputation-only rewards**.

Finally, no published result composes decision-dependent observation, forward-looking agents who
value weight, and simultaneous panels: every article fixes the other two. This is an absence of
evidence after some thirty targeted searches, not a proof of absence. Open: an exactly compatible
mechanism with optimal sample complexity; whether WSU is `ε`-compatible for forward-looking agents;
the optimal exploration probability in a panel.

## 5. Objective D — against blocs of real persons only the hypergeometric tail remains

**The internal obstacles** are the paper's *Evasion pays* and *Sparse overlap* propositions (its §6)
and the false positives on honest like-minded reviewers.

### 5.1 The provable bound is a rate, and at 12 seats it is not negligible

In the peer-review literature the only proved guarantee that holds whatever the manipulation is a
cap on the probability that a given reviewer lands on a given item (Jecmen et al., NeurIPS 2020,
Theorem 1, text read). Controlling the joint probability for pairs is NP-hard in general
(Theorem 2), co-assignment can be excluded only for *known* subsets (Theorem 3), and groups larger
than a pair are left to future work ([arXiv:2006.16437](https://arxiv.org/abs/2006.16437)).

The bound on a panel's capture therefore comes from sampling theory and is **the survey's
computation**: for a coalition fixed before the draw, holding a share `s` of the pool, the
probability of reaching the threshold `t` of `n` seats by itself is the hypergeometric tail,
bounded above by the binomial tail and approximated by `C(n, t) · s^t` for small `s`.

| Coalition's share `s` | Capture, `n` = 9, `t` = 6 | Capture, `n` = 12, `t` = 8 | Capture, `n` = 12, `t` = 9 | Block, `n` = 9 (4 seats) | Block, `n` = 12 (4 seats) |
|---|---|---|---|---|---|
| 0.10 | 6.4·10⁻⁵ | 3.4·10⁻⁶ | 1.7·10⁻⁷ | 0.8% | 2.6% |
| 0.20 | 3.1·10⁻³ | 5.8·10⁻⁴ | 6·10⁻⁵ | 8.6% | 21% |
| 0.30 | 2.5·10⁻² | 9.5·10⁻³ | 1.7·10⁻³ | 27% | 51% |

"9 of 12" is the answer review's threshold with 12 usable answers; "block" is the probability that
the coalition holds enough seats to prevent an acceptance. Binomial values.

Three readings matter.

- Capture falls as the *power* `t` of the share, not exponentially: Byzantine-committee systems
  that want negligible probabilities use committees of hundreds of members (figures seen in
  snippets only) ([arXiv:1804.00399](https://arxiv.org/pdf/1804.00399)).
- It is a guarantee of *rate*: the coalition does not choose which items it captures, but over
  thousands of items some capture is certain for shares from 20% up.
- Capture and blocking cannot both be made small with at most 12 seats. Where "no quorum" is a
  neutral outcome, blocking degrades throughput, not correctness — the benign direction.

All of this holds only if the draw is uniform, cannot be influenced and cannot be predicted until
the item is fixed: if a party can choose among `G` candidate beacon values, the capture probability
of the panel it cares about rises up to `G` times (a union bound, the survey's inference). The
offline answer review takes this as an assumption of its caller
([`05`](05-question-lifecycle.md#the-answer-review-01-d49d52)); it is a premise of the guarantees,
not an optional hardening.

### 5.2 Detecting and discounting coordination does not hold, and the discount rewards evasion

Here the internal obstacles find empirical confirmation and analogies, not dedicated theorems.
Methods with proofs guarantee something only for dense blocks: FRAUDAR detects blocks denser than
the densest normal block and declares detection unlikely below that (text read)
([Hooi et al. 2016](https://bhooi.github.io/papers/fraudar_kdd16.pdf)). On bidding data, far denser
than this project's co-ratings, the best detector reaches on average a **31% overlap** with a ring
of 10 colluders at full density, and groups of *honest* reviewers occupy the same region of
parameters (text read; the article contains no theorem)
([Jecmen et al., TMLR 2025](https://arxiv.org/html/2402.07860v2)).

By the survey's computation, with `r` items per reviewer out of `m` the expected shared items per
pair are `r²/m`, about 0.3–1.6 an epoch, while detecting a residual correlation of 0.2 needs about
153 shared items (765 with a correction for a million pairs). The need grows as `1/ρ²`, so
independent noise that halves the correlation quadruples the data required. No lower bound stated
for the sparse co-rating regime was found.

On the sublinear discount the survey gives a two-line argument of its own, unpublished in this
context: if a recognized cluster's weight is strictly subadditive in its size, appearing as two
clusters raises the weight; if superadditive, feigning correlation pays; only the additive weight —
no discount — is neutral. The published axiomatic analogue is the characterization of division
rules immune to both merging and splitting, which are essentially proportional (seen in abstract)
([Ju, Miyagawa, Sakai 2007](https://ideas.repec.org/a/eee/jetheo/v132y2007i1p1-26.html)). The
quadratic-funding mechanisms do not contradict it: Buterin's bound on the loss grows as `k(k − 1)`
in the coalition's size, and COCM's collusion resistance presupposes known, complete group
memberships (authors' summaries only)
([summary on ethresear.ch](https://ethresear.ch/t/collusion-resistance-and-plurality-in-quadratic-mechanisms-paper-summary/14545)).

What follows is structurally unreachable with no ground truth or exogenous relations: giving a
coalition of unique persons *less* influence than its head count without penalizing a sincere
faction of the same size; making it unprofitable to hide coordination under any nonlinear
weighting; a capture probability of cryptographic grade with 7–12 seats.

The positive counterpart is plain: equal weights, no discount and random assignment form the design
in which hiding coordination gains nothing, and with no reviewer reputation there is no "long con"
on the reviewers' side. The one proved defence against the latter, Resnick and Sami's influence
limiting, costs information discarded from honest raters and seems to need outcomes verified
afterwards (seen in abstract; the ground-truth requirement is a recollection, not confirmed)
([Resnick & Sami 2007](https://presnick.people.si.umich.edu/papers/recsys07/index.html)).

## 6. The answer review against these limits

### 6.1 What the rule certifies, and the one parameter that governs it

A qualified-majority rule has a foundation. With independent homogeneous voters and asymmetric
priors or error costs the optimal rule is a majority with a threshold (Nitzan and Paroush;
Ben-Yashar and Nitzan, seen through Wikipedia)
([Jury theorem](https://en.wikipedia.org/wiki/Jury_theorem)). With equal reliabilities majority
voting is the maximum-likelihood rule and attains the minimax exponent (Theorem 5.1, text read)
([Gao, Lu, Zhou 2016](https://ar5iv.labs.arxiv.org/html/1605.07696)). And with a constant number of
tasks per rater the proved advantage of reliability-aware inference disappears, because raters
cannot be assessed from few tasks ([Karger, Oh, Shah](https://ar5iv.labs.arxiv.org/html/1110.3564)).

By the survey's computation under independence, with 12 usable answers a wrong key that each
reviewer chooses with probability 0.3 is accepted in **0.17%** of cases (3.9% under simple
majority), while a correct key chosen with probability 0.8 is accepted in 79% and ends in divided
answers in 21%.

The protection depends on one unmeasured parameter: the correlation of errors inside a panel.

| | Accepted wrong key |
|---|---|
| Independent answers, `p = 0.3` | 0.17% |
| Intra-panel correlation 0.1 | 2.7% |
| Intra-panel correlation 0.2 | 6.7% |
| A share `f` = 0.25 defers to a common source that endorses the key | 0.3% |
| `f` = 0.5 | 13.5% |
| `f` = 0.75 | 72% |

The evidence that this is the real regime is solid for models and thin for human panels: over more
than 350 LLMs, models agree in 60% of the cases in which both err, with more correlated errors for
the stronger models (abstract verified) ([Kim et al. 2025](https://arxiv.org/abs/2506.07962)); no
controlled study measures the correlation of answers in human multiple-choice panels with free use
of assistants. No threshold, weighting or panel size corrects a shared error: that is the limit
Prelec et al. prove, and the rule of never presenting consensus as truth (`01` D48) coincides with
what the theory can support.

### 6.2 The balance by objective

| Objective | Reachable (assumptions and cost) | Unreachable (nature of the limit) | Open |
|---|---|---|---|
| A, bridging | A label-free measure of cross-camp approval; high precision, low recall. Needs many co-rated items per pair and both camps in the panel | Separating true-divisive from partisan; balancing an absent camp; resisting feigned positions. Identification: more ratings do not help. Only the non-neutrality on the camps' sizes is a cited theorem | Identifiability of the intercept; sample complexity at 7–12; a manipulation bound |
| B, IRT and latent DIF | Detection of departures shared by at least two items; unidimensionality tests; DTF between classes on a common sample. N from thousands to tens of thousands per batch | Absence of bias against real groups; uniform DIF against impact; composition of DTF across batches. Identification. A lone item and true/false: limits of power | Theory for 2PL/3PL mixtures with unknown classes and anchors; linking of classes; measurement of assisted answers |
| C, proper reputation | Properness in expectation with a hidden full-support draw, an unnormalized uncapped score, linear utility, no stake in the outcome; linear weight for myopic agents; lotteries for forward-looking ones. Cost: variance `1/ε`, no guarantee of accuracy | Strict properness under a deterministic rule; exact compatibility with a nonlinear weight (counterexamples, not a general theorem); ranking guarantees at 5–10 reports (a limit of data) | A composition theorem; exact compatibility with optimal complexity; reputation-only rewards |
| D, coordinated blocs | An exact capture bound, the hypergeometric tail, with a draw that cannot be influenced and equal weights; neutrality to evasion with no discount | Influence below the head count; an evasion-proof discount; negligible capture at 7–12 seats. Identification for the first two, arithmetic for the third | A lower bound for sparse co-rating; coalition-proof peer selection |

### 6.3 Refinements the literature supports — none proposed here

Five interventions need no new information and meet no invariant. All improve efficiency or the
provable number; none improves agreement with the truth. **None is proposed, adopted or authorized
by this record:** each would change the contract or the parameters the owner approved (`01`
D49–D53, [`14`](14-parameter-register.md#the-answer-review--provisional-uncalibrated)) and needs a
decision of its own.

1. **A threshold consistent between 9 and 12 usable answers.** The integer rule `4C ≥ 3V` is
   7 of 9, 8 of 10, 9 of 11 and 9 of 12 — 77.8%, 80%, 81.8% and 75%: strictest at 11, loosest at 12
   (`08` AT-ANS-01 records the same boundaries).
2. **A staged panel with early stopping**, justified by the optimality of Wald's sequential test
   under independence (cited from memory)
   ([Wald 1945](https://projecteuclid.org/euclid.aoms/1177731118)). In the worst case adaptivity
   does not improve the order of the requirement, and shared error is untouched. The present
   contract has no early closure.
3. **Using the concentration of dissent**: three dissenters agreeing on one alternative are
   stronger evidence against the key than three scattered ones. A direct consequence of the
   Dawid–Skene likelihood, not a result verified in an article for small panels.
4. **A second, wider, independently drawn panel on appeal**: since capture falls as `s^t`, it is
   the one lever inside the invariants that improves the provable number. The present contract has
   no new round.
5. **A draw that cannot be predicted or influenced** — not an improvement but the premise of every
   bound of §5.

Everything that promises more needs an outside ingredient, and each meets a constraint.

| Ingredient | What it buys, by the literature | What it needs | Conflict with the fixed constraints |
|---|---|---|---|
| A little ground truth (gold items, spot checks, outcomes revealed later) | Removes the shared-prejudice equilibria; makes reliability a measure of accuracy, not conformity; reliable curation even with an honest minority, with a number of trusted checks independent of the population ([Steinhardt, Valiant, Charikar, abstract only](https://arxiv.org/abs/1606.05374)) | Trusted keys indistinguishable from ordinary items; by the survey's arithmetic about 16–64 gold answers per reviewer for a precision of ±0.1 to ±0.05 | Brings in an authority on truth, the very thing under examination in the author's key; with no money the checking rate cannot be lowered; brings back a reputation that can be spent; does not stop a targeted capture by those who answer the gold items well |
| A second elicited signal (a prediction of the others' answers, "surprisingly popular") | The one route that recovers the truth where the majority is wrong, in the infinite-sample model with Bayesian respondents and a common prior ([Prelec et al. 2017](https://gwern.net/doc/statistics/prediction/2017-prelec.pdf)) | An informed minority that knows it is one; large samples (at 12 the verdict turns on one answer, the survey's reasoning); honest predictions, hence a score per answer | Scoring the predictions is a reward per reviewer, absent by choice (`01` D48); with no incentive the predictions are a new surface for manipulation; the open environment erodes the structure the method uses (the original studies forbade looking the answers up) |
| Hidden randomization | Properness in expectation with decision-dependent observation ([Chen & Kash 2011](https://dash.harvard.edu/server/api/core/bitstreams/7312037c-918b-6bd4-e053-0100007fdf3b/content)) | A draw unknown until the reports close, full support, an uncapped inverse-propensity score, a linear weight or lotteries | Does not contradict deterministic computation if the seed is committed before and revealed after (a reading of this record, not of the sources); contradicts a nonlinear weight map; the non-rotatable pseudonym moves away from the myopic regime; with few reports the reputation stays noise |
| Group information | The Remesh-style minimum over declared groups; exclusion of co-assignment (Jecmen et al., Theorem 3); COCM-style discounts; classical DIF and DTF with a focal group; stratified panels | Exogenous labels: affiliation, social graph, attributes | A head-on conflict with invariant 1; inferring the labels from behaviour returns to the detection problem |

**What the literature advises against bringing back.** Weighting by a reliability estimated from
the answers alone is, as a statistic, long-run agreement with one's co-panelists (the survey's
inference from the one-coin model): it stands for accuracy only where the crowd is mostly right and
errors are independent, amplifies whoever shares the dominant source, and is exposed to a
documented attack of disguise and exploitation
([Miao et al. 2018, secondary source](https://cse.buffalo.edu/~lusu/papers/WWW2018.pdf)). Its
largest gain at 12 answers, computed by the survey with reliabilities known exactly, is nil for a
homogeneous panel and large only when the pool mixes excellent and nearly random reviewers (an
error of 12.4% falling to 0.73% with four reviewers at 0.95 and eight at 0.5) — the regime that
needs tens of questions per reviewer.

### 6.4 The verdict

"Markedly improvable" is not realistic. Taken literally, the original objectives are not all
reachable together under the given constraints: the fairness guarantee for the whole test (B) and
resistance to blocs beyond their number (D) stop at identification limits; optimal honesty with
reputation as weight (C) has no theorem to support it and has counterexamples for simpler maps;
bridging (A) can be defined but not estimated with 12 ratings, and never separates truth from
divisiveness. The known margin above the answer review is one of efficiency and of the clarity of
its claims, not of its power to ascertain.

The strongest contrary position deserves stating: the proved limits concern guarantees, not average
performance, and in a heterogeneous and truly independent pool a weighting or a second signal would
improve empirical results. That regime is the one the open environment puts in doubt, and with no
measurement of the correlation of errors there is no way to know in which of the two worlds the
system operates.

## 7. What is left of value

The fallback to the answer review was a convergence on the one region in which the system's claims
can be proved: equal weights, a random draw and consensus declared as an operational criterion are,
in that order, the design neutral to evasion, the only exact capture bound available and the only
reading of consensus that the theory supports. The residual value lies less in stronger mechanisms
than in two things the survey points to without being able to close.

- **Empirical.** The correlation of errors in a panel with free use of tools is the parameter that
  moves wrong acceptance by one or two orders of magnitude, and nobody has measured it for human
  multiple-choice panels. Measuring it is worth more than any change of the rule.
- **Of formulation.** Replace every unreachable guarantee by the strongest claim that stays true:
  "no departure of size `δ` detected with power `1 − β`" for "unbiased", "insufficient cross-camp
  participation" for a rejection.

Every stronger guarantee is a trade against a constraint, and which constraint to yield — an
authority on truth for a few items, a reward for predictions, a group label — is a decision of the
owner's values, not a problem more mathematics can dissolve.

**Internal results with no counterpart found in the literature.** The closed-form split of the
leak toward the majority, the batch relativity of the intercept, the impossibility of composing DTF
across batches with no labels, and the argument that a sublinear discount rewards evasion were
found nowhere in print. They are open gaps, possibly publishable, and are to be cited as this
project's results awaiting independent verification, not as established facts.
