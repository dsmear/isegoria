# 22 — Phase 1: are the archived paper's objectives reachable? A literature survey

| | |
|---|---|
| **Purpose** | Record, for later reference, what the published research says about the four objectives of the [archived paper](../archive/phase1-forecast-pilot/paper/README.md) — bridging, latent-class DIF with a whole-test guarantee, a proper reviewer reputation used as weight, resistance to coordinated blocs — and whether the [answer review](05-question-lifecycle.md#the-answer-review-01-d49d52) that replaced them leaves known room for improvement. |
| **Status** | **Informative, not normative (2026-11-11).** It decides nothing, closes or reopens no finding of [`15`](15-phase1-review.md), changes no contract or parameter and authorizes no experiment. It has had no design review. |
| **Method** | A literature survey run on the owner's request by AI research agents (Claude), one per area, synthesized by another. No source was re-read by a person. The working notes, with a verification level for every source, are kept outside the repository. **Re-read on 2026-11-15** by Claude against the downloaded texts: every arXiv identifier cited was confirmed to exist with the cited title and authors, and each cited theorem or figure of the sources then available was checked in the text ([§1](#1-one-argument-of-observational-equivalence-runs-through-the-four-objectives), *How far the sources were read*); 25 further sources, on strands the first survey did not cover, were read in part ([§8](#8-strands-the-first-survey-did-not-cover)). Still no person's reading and no design review. |
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
information, and each of these meets a fixed constraint. The strands the first survey did not
cover, read in part on 2026-11-15, change none of this (§8).

**General caution.** The re-reading of 2026-11-15 checked the cited statements, not whole papers or
proofs; the sources behind a paywall or on hosts not reached were not re-read; and almost every
"unreachable" entry below is an argument made during the survey, not a published theorem. Recheck a
source before citing it in a decision record.

## 1. One argument of observational equivalence runs through the four objectives

The paper asked for a non-majoritarian acceptance filter, a psychometric verdict with a guarantee
for the whole test, a reviewer reputation in which honesty is the best strategy, and resistance to
coordinated blocs of real persons. The answer review replaced all four (`01` D47–D52). The question
is whether that fallback was forced by the mathematics or by a design that could be bettered. The
five strands of the survey converge on the first answer, by the same scheme in every area: a rule
that is a function of the observed data alone returns the same output in two worlds that produce
the same data.

The scheme is **published** in three cases, as a proved theorem in the first and the third:

- votes and confidences do not identify the correct answer even with ideal Bayesian respondents
  and an infinite sample (Theorem 1 of the supplement, read with its proof). In its model the
  respondents' signals are independent given the answer and the prior carries the evidence common
  to all: the limit comes from shared evidence, not from noise
  ([Prelec, Seung, McCoy 2017](https://gwern.net/doc/statistics/prediction/2017-prelec.pdf));
- labelling algorithms with no gold standard that identify the informed by their agreement always
  admit an equilibrium in which everybody reports a common prejudice (read in full: an argument of
  one paragraph by observational equivalence, with no formal proof, for agents who want to be
  identified as informed)
  ([Della Penna & Reid, arXiv:1204.3511](https://arxiv.org/abs/1204.3511));
- group disparities are not identifiable in general when the protected class is unobserved, even
  with auxiliary proxy data (abstract and the statement of the identification conditions read;
  proved for classifiers, not for DIF)
  ([Kallus, Mao, Zhou, arXiv:1906.00285](https://arxiv.org/abs/1906.00285v2)).

Della Penna and Reid's equilibrium is created by the reward for agreement. The answer review gives
its reviewers none (`01` D48), so that strategic form does not arise in it; what stays is the
unstrategic shared error of Prelec et al.'s theorem.

In the other cases that matter here — true but divisive against partisan, DIF with respect to an
unobserved group stated in psychometric terms, a sincere faction against a cartel — **no published
theorem was found**. They are elementary arguments of the survey, consistent with what the
literature concedes as a design limit, and are presented as such. For the last, the nearest
published statement is a remark, not a theorem: when the reliable raters are a minority, dishonest
raters who copy them on permuted items cannot be told apart without the operator's own ratings
([Steinhardt, Valiant, Charikar](https://arxiv.org/abs/1606.05374), introduction).

**How far the sources were read.** The first survey read in full, among others: Blair et al.,
Selvam et al. (its first 100,000 characters), Truong et al., Chen & Kash, Witkowski et al., Freeman
et al., Hudson, Gao, Wright & Leyton-Brown (arXiv v1), Jecmen et al. 2020 and 2025, FRAUDAR, Wallin,
Chen & Moustaki, Wallin & Huang, Hashimoto et al., Karger, Oh & Shah, and Gao, Lu & Zhou; the rest
at abstract level, through model-generated summaries or from memory.

The re-reading of 2026-11-15 worked on the full texts of the 28 arXiv sources cited here and of
Prelec et al., Blair et al. and FRAUDAR:

- **Read in full:** Della Penna & Reid; the supplement of Prelec et al. up to its Theorem 3.
- **Cited statement, theorem or figure read in its section:** Steinhardt et al., Selvam et al.,
  Blair et al., Chuai et al., Truong et al., Andreou & Sirivianos, Wallin, Chen & Moustaki, Wallin
  & Huang, Hashimoto et al., Chen et al. (arXiv:2110.11112), Kallus et al., Witkowski et al.,
  Freeman et al., Frongillo et al., Kong 2020, Gao, Wright & Leyton-Brown, Jecmen et al. 2020 and
  2025, Dang et al. (arXiv:1804.00399), Gao, Lu & Zhou, and Karger, Oh & Shah.
- **Abstract and introduction only:** Kong 2026, Aznag et al., Xu et al., Burrell & Schoenebeck,
  Boutilier, Arjmandi-Lari et al., Kim et al. and FRAUDAR.
- **Not re-read**, so as the first survey left them: Chen & Kash (its link now redirects to a host
  not reached), the Community Notes documentation, Ovadya & Thorburn, Stout et al., Sen & Cohen,
  Miao et al., Resnick & Sami, Wald, the quadratic-funding summary, the jury-theorem literature
  (Wikipedia), and every source behind a paywall but two — Cohen & Bolt, Shealy–Stout,
  Lindsay–Clogg–Grego, Bauer & Curran, Alexeev et al., Wu, Steger et al. 2020, Ju–Miyagawa–Sakai,
  Othman & Sandholm, Bouchaud & Ramaciotti.
- **Read from copies the owner supplied:** Raju, van der Linden & Fleer (§3.2), Rost (§3.1), and
  two papers the first survey did not cite, Steger, Schroeders & Wilhelm 2021 (§3.3) and
  Ben-Yashar, Koh & Nitzan (§8.1).

Where a text differs from what the first survey wrote, the entry below is corrected. No proof was
checked except Prelec et al.'s Theorem 1. Roughgarden & Schrijvers and Shnayder et al., which the
first survey knew through summaries, and the strands it did not cover are in §8, each with its
level of reading.

**The survey's computations** assume a binary reduction (an answer coincides with the key or does
not) and independent or exchangeable answers. They illustrate sensitivity; they do not estimate a
real pool. Recomputed while writing this record: the effective thresholds of §6.3, the acceptance
of a wrong key at `p = 0.3`, the capture and blocking columns at 9 of 12 in §5.1, the leak shares
and the absent-camp probabilities of §2, and the regret figure of §4.3. Recomputed on 2026-11-15:
the same thresholds, acceptance and capture figures, the two correlation rows of §6.1 (a
beta-binomial with mean 0.3) and its common-source rows, whose base the first survey did not state;
the abstention figures of §5.1 and the budget figure of §6.1 are new. The other figures are as the
survey's notes give them.

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
status on who else is in the fit: 30.2% of displayed notes, 13,202 of 43,782, later lose their
status (its Table 1) ([Chuai et al. 2026](https://arxiv.org/abs/2601.14002)).

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
`ln(m/δ) / (ε² γ³)`, with a sufficient constant of 9000 (Theorem 10). The authors grant that such
worst-case values may be infeasible at the platforms' present scale, and report experiments in
which the estimators work well. What deployed bridging gives in practice is high precision and low
recall: about 10% of notes pass
([Arjmandi-Lari et al. 2025](https://arxiv.org/abs/2510.00650)), and in a simulation with known
truth more than 40% of genuinely helpful notes stay unpublished across its populations of honest
raters with varied polarization and in-group bias (its Figure 2), 30–40% in the baseline with
neither
([Truong et al. 2025](https://arxiv.org/abs/2511.02615)).

At a panel of 12 none of these guarantees is available:

- no sample-complexity result exists for the intercept or for cluster-based bridging at about 10
  ratings;
- the production rule "at least 5 per side" would fail almost every small panel;
- by the survey's computation, if the smaller camp is 20% of the population, a panel of 9 holds
  none of its members with probability about 13%, and at most one with probability about 44%;
- fewer than 10 strategic ratings lift **7.0–10.7% of the notes below the median** over the
  threshold (its Table 2, for rater factors within ±0.4 and ±1), by an attack in two stages (build
  a sparse latent position, then support the target together) that the authors attribute to a
  property of the factorization, not to a defect
  ([Selvam et al. 2026](https://arxiv.org/abs/2607.01824)).

Random assignment, which the project already uses, is the structural mitigation the same authors
recommend and report as deployed: a note's status must hold on the ratings of a random sample of
eligible raters.

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
second latent axis changes the outcomes, the published share of notes with at least 100 ratings
falling from 71.5% to 11.7% as disagreement on it grows
([Andreou & Sirivianos 2026](https://arxiv.org/abs/2609.21496)).

## 3. Objective B — latent classes flag departures from the model, not bias against real groups

**The internal obstacles** are [A2](15-phase1-review.md#a2-current-state-design-decision-deferred)
(per-fit DTF does not compose into a whole-test bound), A8 (weak detection of one biased item), the
spurious classes of `01` D25 and D43, and the sample sizes of [`13`](13-characterization.md) §8.7.

### 3.1 Reachable: three weaker claims, with N in the thousands

The only family of methods with no labels is "latent classes plus IRT within each class". Its
founding paper presents the classes as heuristic: the model finds the partition of the sample for
which the item parameters differ most, cannot assign the all-correct and all-wrong patterns to a
class (18% of its real sample), and recovers a group that guesses as a class of its own; its
simulations use 1,800 simulees and 10 items, and the number of classes is chosen by AIC
([Rost 1990](http://purl.umn.edu/93227), read but for the estimation equations). The family's
recent L1-penalized version (Wallin, Chen & Moustaki, *Psychometrika* 2024, text read) buys
identification with the assumption that DIF is sparse and **proves no theorem** of identifiability
or consistency specific to the latent case. Its simulations are far kinder than this project's
regime: N = 1,000 and 5,000, 25 or 50 items, 40% of the items with DIF of 0.5–1.5 logits, no
guessing parameter, the number of classes known, data generated by the fitted model. There the
items with DIF are recovered well (true-positive rate about 0.95–0.99) while the classification
error of persons stays at 7–26% with two classes and 35–40% with three
([arXiv:2305.00961](https://ar5iv.labs.arxiv.org/html/2305.00961)). The method's authors call it
"an exploratory device" whose classes are "model-based response groups whose substantive meaning
requires external validation", and warn that the DIF it detects is not DIF with respect to the
background variables the classes correlate with
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
  ([Chen, Li, Ouyang, Xu, arXiv:2110.11112](https://arxiv.org/abs/2110.11112), its §2.3).
- **The composition of DTF across batches — A2's obstacle.** In the DFIT framework DTF is the
  focal group's mean of `D²`, where `D` is the difference between an examinee's expected scores as
  a member of the focal and of the reference group, the two sets of item parameters being on a
  common scale. It is additive over the items of one test, `DTF = Σ CDIF_i` with
  `CDIF_i = Cov(d_i, D) + μ_d μ_D`, and an item's share depends on the other items in the test
  (Raju, van der Linden, Fleer 1995: definitions, tests, design and discussion read, the result
  tables not) ([DOI](https://doi.org/10.1177/014662169501900405)). The paper does not treat
  batches. From its definition, for a test made of batches `A` and `B`,
  `DTF = DTF_A + DTF_B + 2 E(D_A D_B)`: the batches' own indexes leave out a cross term that needs
  both differences on the same focal examinees and the same metric. Labels supply that; class `k`
  of batch 1 and class `k` of batch 2 are unrelated objects, the sign of DIF is defined only
  against an arbitrary reference class per batch, and cancellation across batches stays undefined.
  **This is an inference from the definitions:** the survey found no article that poses the
  problem for latent classes, and no impossibility theorem stated in psychometric terms.

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
([Steger et al. 2020](https://econtent.hogrefe.com/doi/10.1027/1015-5759/a000494)). A study by
the same first author gave 315 participants a knowledge test online and then proctored: mean
scores of 0.66 and 0.57, a correlation of .52 between the two, and items meant to be unsolvable
answered at chance when proctored (0.23) and at 0.34 online. It reports the meta-analysis as
finding that those who can look an answer up do so, whatever the stakes or the countermeasures,
and it predicts the score gain from response times and switches of browser tab, which `01` D47
excludes here ([Steger, Schroeders, Wilhelm 2021](https://doi.org/10.1177/1073191120914970):
abstract, introduction, results and discussion read). Looking up is allowed here; what the
correlation shows is that the ranking of persons changes with it. Where every
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
outcome, exact properness needs that utility to be known (Boutilier, abstract and introduction)
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
decimal; a follow-the-regularized-leader mechanism is `4η`-approximately truthful for a step
`η < 1/4`, Frongillo et al.'s Corollary 1)
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
dependence with peers, not agreement with the truth. DMI's dominance needs at least `2C` tasks
answered by both agents of a pair, `C` being the number of choices
([Kong 2020, Theorem 5.1](https://arxiv.org/abs/1911.00272)), and two 2026 preprints make the bound
tight: with `n` possible reports, `2n` tasks are necessary and sufficient for any payment that is
zero on independent reports and does not reward garbling
([Kong 2026](https://arxiv.org/abs/2606.14225)), and with binary reports no such payment exists on
three tasks or fewer, DMI being the only one on four or five
([Aznag et al. 2026](https://arxiv.org/abs/2608.16838)). A question here has at least three possible
answers, so a pair of reviewers would need six or more questions in common, against the 0.3–1.6 an
epoch that random panels give (§5.2). In simulation there is an apparent trade-off between
measuring individual quality and robustness to strategic reports
([Burrell & Schoenebeck, arXiv v2](https://arxiv.org/abs/2108.05521), venue not verified).

With ground truth on a subset, by contrast, rewarding agreement with the trusted outcome alone
makes honest effort a dominant strategy at a checking rate equal to the cost of effort over the
reward gap (Lemma 1, text read) ([Gao, Wright, Leyton-Brown](https://arxiv.org/abs/1606.07042)); a
2024 work finds that the metrics used to compare spot-checking with peer prediction give
contradictory rankings, and proposes to measure a peer-prediction mechanism by the spot-checking
rate it is worth ([Xu et al. 2024](https://arxiv.org/abs/2402.13567)). With no money the gap is
bounded and that rate cannot be lowered by paying more. The first survey found no source on rewards
that are not money; peer grading, which rewards with grades, states the bound: below a ratio of
effort cost to reward no checking rate makes honest effort dominant (§8.1).

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
  that want negligible probabilities use committees of 80 to 600 nodes, sized by the same
  hypergeometric tail ([Dang et al., arXiv:1804.00399](https://arxiv.org/pdf/1804.00399), its §5.2).
- It is a guarantee of *rate*: the coalition does not choose which items it captures, but over
  thousands of items some capture is certain for shares from 20% up.
- Capture and blocking cannot both be made small with at most 12 seats. Where "no quorum" is a
  neutral outcome, blocking degrades throughput, not correctness — the benign direction.
- The "block" columns count dissent. Under the answer review's rule
  ([`05`](05-question-lifecycle.md#the-answer-review-01-d49d52)) an abstention blocks with fewer
  seats whenever fewer than 12 reviewers answer, since it lowers `V` below the quorum (the survey's
  computation, 2026-11-15): with 10 answering, two dissenters leave the question accepted (8 of 10)
  while two abstentions make it insufficient participation; with 9 answering, one abstention does.
  The outcome it forces gives the author no observation. Political economy studies the effect for
  participation quorums; that literature was not read (§8.2).

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
voting is the maximum-likelihood rule and attains the optimal error exponent; with unequal ones its
exponent is strictly worse (Theorem 5.1 and the remark after it)
([Gao, Lu, Zhou 2016](https://ar5iv.labs.arxiv.org/html/1605.07696)). And with a constant number
`r` of tasks per rater the proved advantage of reliability-aware inference disappears: the answers
needed per task return to the order of majority voting, `(1/q²) log(1/ε)` against `(1/q) log(1/ε)`,
"due to the fact that when `r` is small, we cannot efficiently learn the quality of the workers"
([Karger, Oh, Shah](https://ar5iv.labs.arxiv.org/html/1110.3564), after Corollary 2.4; `q` is the
mean of `(2p − 1)²` over raters). Two qualifications, both the survey's reading:

- the guarantee does not reach a panel of 12 whatever `r` is: Corollary 2.4's sufficient budget,
  `(24 + 8/((r − 1)q)) (1/q) ln(2/ε)` answers per task, is at least `24 ln 4 ≈ 33` for any crowd
  and any error below one half;
- `r` counts all the tasks of a rater, and a non-rotatable pseudonym accumulates them over epochs:
  "few questions per reviewer" holds for an epoch, not for a reviewer's history. The same paper
  reports, as an empirical observation it takes to be fundamental and does not prove, that below
  `(ℓ − 1)(r − 1) q² = 1`, with `ℓ` answers per task, nothing does better than majority voting.

By the survey's computation under independence, with 12 usable answers a wrong key that each
reviewer chooses with probability 0.3 is accepted in **0.17%** of cases (3.9% under simple
majority), while a correct key chosen with probability 0.8 is accepted in 79% and ends in divided
answers in 21%.

The protection depends on one unmeasured parameter: the correlation of errors inside a panel.

| | Accepted wrong key |
|---|---|
| Independent answers, `p = 0.3` | 0.17% |
| Intra-panel correlation 0.1, mean 0.3 | 2.7% |
| Intra-panel correlation 0.2, mean 0.3 | 6.7% |

A second model: each reviewer, independently and with probability `f`, defers to a common source
that endorses the wrong key; the others choose it with probability `p`. The first survey gave the
`p = 0.1` column without stating its base, beside the `p = 0.3` rows above.

| Share `f` deferring | Others at `p = 0.1` | Others at `p = 0.3` |
|---|---|---|
| 0.25 | 0.3% | 5.2% |
| 0.5 | 13.5% | 35% |
| 0.75 | 72% | 86% |

The evidence that this is the real regime is solid for models and thin for human panels: over more
than 350 LLMs, models agree in 60% of the cases in which both err on one leaderboard dataset, with
more correlated errors for the more accurate models
([Kim et al. 2025](https://arxiv.org/abs/2506.07962)); no
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
| A little ground truth (gold items, spot checks, outcomes revealed later) | Removes the shared-prejudice equilibria; makes reliability a measure of accuracy, not conformity; reliable curation even with an honest minority `α` of raters, with a number of the operator's own judgments, of order `1/(βε²)` for the best share `β` of items and a tolerance `ε`, independent of the population ([Steinhardt, Valiant, Charikar, Theorem 1](https://arxiv.org/abs/1606.05374)) | Trusted keys indistinguishable from ordinary items, or, in Steinhardt et al., the operator's judgments after the fact on a few items; by the survey's arithmetic about 16–64 gold answers per reviewer for a precision of ±0.1 to ±0.05. Steinhardt et al.'s theorem asks of every rater a number of ratings of order `1/(βα³ε⁴)`, which the authors call likely not optimal: about 160,000 at `α = β = 0.5`, `ε = 0.1` (the survey's arithmetic) | Brings in an authority on truth, the very thing under examination in the author's key; with no money the checking rate cannot be lowered; brings back a reputation that can be spent; does not stop a targeted capture by those who answer the gold items well |
| A second elicited signal (a prediction of the others' answers, "surprisingly popular") | The one route that recovers the truth where the majority is wrong, in the infinite-sample model with Bayesian respondents and a common prior. Proved for two answers (Theorem 2); with more, the correct answer is still more popular than predicted but others may be too, so that criterion is necessary only, and a different statistic, the prediction-normalized vote, selects the answer when each answer has a signal that favours it most (Theorem 3) ([Prelec et al. 2017](https://gwern.net/doc/statistics/prediction/2017-prelec.pdf), supplement) | An informed minority that knows it is one; large samples (at 12 the verdict turns on one answer, the survey's reasoning); honest predictions, hence a score per answer | Scoring the predictions is a reward per reviewer, absent by choice (`01` D48); with no incentive the predictions are a new surface for manipulation; the open environment erodes the structure the method uses (the original studies forbade looking the answers up) |
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

## 8. Strands the first survey did not cover

Recorded on 2026-11-15. The 25 arXiv sources below were read by Claude at the level each entry
states — the abstract and the passages that bear on this project, none in full and no proof
checked. The sources of [§8.2](#82-not-read) were not seen. Nothing here changes an entry of §§1–7
beyond the cross-references made there (§1, §4.3, §5.1).

### 8.1 What the reading adds

**No route to a stronger guarantee inside the fixed constraints was found.** Each strand either
needs an ingredient §6.3 already lists, or measures something without certifying it.

**Debate and refereed protocols.** The one route seen that adds data of another kind than votes:
objections a judge can check, under one honest party in place of an honest majority.

- With optimal play and a polynomial-time judge a debate decides any question in PSPACE, on the
  premise, which the authors call empirical, that it is harder to lie than to refute a lie
  ([Irving, Christiano, Amodei 2018](https://arxiv.org/abs/1805.00899): abstract, the central claim
  and Theorem 1's statement; [Brown-Cohen et al. 2023](https://arxiv.org/abs/2311.14125): abstract).
- Human judges who cannot read a story choose between two answers after two debaters, one assigned
  to the correct answer, argue with verbatim quotes the platform certifies: 84% correct, against
  74% with one adviser who is right half of the time; no difference with GPT-4 as debater. Of the
  errors, 46% follow an honest debater missing evidence and 33% a judge ending early. The authors'
  limits: judges with no prior opinion, no ideologically charged question, an expertise gap that is
  only access to a text ([Michael et al. 2023](https://arxiv.org/abs/2311.08702): abstract,
  introduction, protocol and limitations).
- The same design with LLM debaters and a tool that verifies quotes: human judges 88% with debate,
  78% with one adviser, 60% with none, and 86–92% across the experiments
  ([Khan et al. 2024](https://arxiv.org/abs/2402.06782): abstract, contributions, the passages on
  human judges).

Every measured gain needs evidence the judge can take as verified, an advocate assigned to the
correct answer and a judge who cannot look the answer up. Reviewers here may consult anything and
nobody is known to hold the right answer, so no measured gain transfers; and an exchange of
objections with evidence is an appeal, which `01` D49 excludes. What the figures do show is the
lever (the survey's computation, independent answers, 12 usable): a correct key is accepted in 62%
of rounds if each reviewer finds it with probability 0.74 and in 89% at 0.84.

**Peer grading.** Rewards that are not money are treated there, which corrects §4.3's first
wording, and every guarantee rests on trusted grades.

- With a grade as reward `R` and an effort cost `c`, a mechanism that makes effortful, truthful
  grading a dominant strategy exists only if the informativeness of a student's signal,
  `P(a|a) − P(a)`, is at least `c/R`; it then checks against a teaching assistant's grade with a
  probability proportional to `c/R`, and below that bound none exists
  ([Zarkoob, Fu, Leyton-Brown 2019](https://arxiv.org/abs/1906.05884): abstract, introduction, the
  statements of Theorems 1–2). A bounded reward therefore sets a limit no checking rate lifts.
- Deployed systems lean on the same authority: staff-graded submissions hidden among those every
  student grades, in a course where 43% of submissions had been graded more than 10 points from
  the staff's grade ([Piech et al. 2013](https://arxiv.org/abs/1307.2579): abstract, introduction,
  data); teaching assistants' checks of 474 to 644 essays a course and 60 to 84 calibration
  submissions, with a model of students who report the class average without doing the work
  ([Zarkoob et al. 2022](https://arxiv.org/abs/2209.01242): abstract, introduction, data).

**Reliance on AI advice.** No rate transfers to §6.1's deferring share: the studies differ in
task, stakes and advice, and report their rates in figures.

- Explanations raise the acceptance of an AI's recommendation whether it is right or wrong
  ([Bansal et al. 2020](https://arxiv.org/abs/2006.14779): abstract, set-up); people often accept
  wrong suggestions, and designs that force them to engage reduce it and are the least liked
  ([Buçinca et al. 2021](https://arxiv.org/abs/2102.09692): abstract).
- Over-reliance responds to costs and benefits: it falls when checking is cheap next to the task,
  and varies with the compensation at stake
  ([Vasconcelos et al. 2022](https://arxiv.org/abs/2212.06823): abstract and the studies'
  summaries). The survey's reading: reviewers here have nothing at stake (`01` D48), which is the
  side of that trade-off where deferring costs least.
- 33–46% of crowd workers on one summarization task are estimated to have used an LLM, by keystroke
  detection and a classifier ([Veselovsky et al. 2023](https://arxiv.org/abs/2306.07899): abstract,
  limitations) — the detection `01` D47 excludes here.
- LLM assistants improved 991 forecasters' accuracy by 24–28% over a control, a deliberately noisy
  assistant included, and the data do not consistently show less diverse predictions
  ([Schoenegger et al. 2024](https://arxiv.org/abs/2402.07862): abstract).

**Monoculture.** A published theorem of the form §6.1 needs, in another model: for two firms
ranking candidates, under two conditions on the ranking model, adopting a common algorithm more
accurate than their own evaluators is a dominant strategy for each, while social welfare is higher
if both keep independent human evaluators
([Kleinberg & Raghavan 2021](https://arxiv.org/abs/2101.05853): abstract, Theorem 1's statement).
It is not a theorem on panels. Sharing training data makes systems' outcomes more homogeneous; for
shared models the results are mixed
([Bommasani et al. 2022](https://arxiv.org/abs/2211.13972): abstract, results).

**Accuracy and dependence estimated with no labels.** Not applicable to reviewers, for want of
common items.

- Under conditional independence the accuracies of binary classifiers follow from the covariance
  of their predictions on a large unlabeled set
  ([Parisi et al. 2013](https://arxiv.org/abs/1303.3257): abstract;
  [Jaffe, Nadler, Kluger 2014](https://arxiv.org/abs/1407.7644): abstract, assumptions); groups of
  dependent classifiers are detected from fourth-order statistics
  ([Jaffe et al. 2015](https://arxiv.org/abs/1510.05830): abstract, Lemmas 1–2), and sparse
  dependency structures among sources are learned
  ([Ratner et al. 2016](https://arxiv.org/abs/1605.07723): abstract;
  [Varma et al. 2019](https://arxiv.org/abs/1903.05844): abstract).
- One method takes agreement as evidence of correctness, the premise in question here
  ([Platanios et al. 2017](https://arxiv.org/abs/1705.07086): abstract); copy detection treats
  sources that share many mistakes as dependent, needs the mistakes, and fails when the source
  copied is good ([Li et al. 2015](https://arxiv.org/abs/1505.02463): abstract, the section on
  source dependency).

All of them need every source to answer a large common set; two reviewers share 0.3–1.6 questions
an epoch (§5.2). The survey's inference: the distributions kept with each outcome (`05`) do
identify, as a mixture of binomials, how the share of the pool choosing the key varies across
questions, with no new data. They do not identify how that variation splits between correct and
wrong keys: a wrong key most of the pool believes sits where a correct one does. That split is
§6.1's unmeasured parameter.

**Duplicate panels.** The design that measures a panel's noise with no truth: at NeurIPS 2014 a
tenth of the submissions, 170 papers, went to two independent committees, and about half of the
accepted list would change on a rerun; in 2021 the two committees disagreed on 23% of the papers
([Cortes & Lawrence 2021](https://arxiv.org/abs/2109.09774): abstract, the experiment's
description; [Beygelzimer et al. 2023](https://arxiv.org/abs/2306.03262): abstract). An error both
panels share stays invisible to it, and a second draw on a question is outside `01` D49 and D53.

**Committees.** A working paper compares members who each judge one component of a project with
members who each judge the whole and vote, their skills given: judging the whole is better when
members combine their own component judgments optimally, judging one component each is better
when a member's overall skill is only the mean of the component skills and the committee's rule is
optimal (Ben-Yashar, Koh, Nitzan, *Is Specialization Desirable in Committee Decision Making?*:
introduction, the statements of its propositions and theorems, summary). It does not treat a panel
answering one undivided question, nor selection by competence (`01` D48).

**Mechanisms.** Consistent with §4: a multi-task mechanism makes truthful reports no worse than any
strategy and strictly better than uninformed ones, its detail-free form learning its statistics
from many tasks ([Shnayder et al. 2016](https://arxiv.org/abs/1603.03151): abstract); with experts
who seek credibility, incentive-compatible algorithms correspond to proper scoring rules, and for
the absolute loss no algorithm has vanishing regret
([Roughgarden & Schrijvers 2017](https://arxiv.org/abs/1702.03615): abstract).

### 8.2 Not read

Cited from memory; no text was seen.

| Strand | Bears on | Sources |
|---|---|---|
| Test theory without an answer key (cultural consensus theory): competences and consensus answers estimated together, the informants needed for a given competence, models with "don't know" and with several consensus answers | The panel's size and thresholds ([`14`](14-parameter-register.md#the-answer-review--provisional-uncalibrated)); what a consensus is called | Batchelder & Romney 1988; Romney, Weller & Batchelder 1986; Anders & Batchelder 2012; Oravecz, Anders & Batchelder 2015 |
| Committee design in political economy: strategic voting, costly information with no reward, participation quorums | Superficial reviews (`01` D48); §5.1's abstention | Austen-Smith & Banks 1996; Feddersen & Pesendorfer 1998; Persico 2004; Koriyama & Szentes 2009; Herrera & Mattozzi 2010 |
| Jury theorems with common evidence | §6.1's correlation of errors | Ladha 1992; Dietrich & List 2004; Dietrich & Spiekermann 2013; Pivato 2017 |
| Diagnostic accuracy with no gold standard | §1's equivalence, in biostatistics, with dependent tests | Hui & Walter 1980; Albert & Dodd 2004; Jones et al. 2010 |
| Content-validity panels | How many of `N` experts must agree | Lawshe 1975; Ayre & Scally 2014 |
| Source dependence in data fusion | Copy detection, above | Dong, Berti-Equille & Srivastava 2009 |
| Acceptance sampling and crowdsourced filters | §6.3's staged panel | Dodge & Romig; Parameswaran et al. 2012 |
| Receipt-freeness | Agreements among reviewers that cannot be verified (§5) | Juels, Catalano & Jakobsson 2005 |
