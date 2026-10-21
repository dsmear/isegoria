# Glossary

Terms used by the current answer review and the retained earlier design.
The [overview](00-overview.md) explains the product; specifications and evidence
stay in their own documents. A concept listed here is not necessarily used by the current path.

## Sybil attack

One participant acts through multiple identities to gain influence. Equal votes do not
prevent it; credential uniqueness is a separate requirement ([03](03-identity-enrollment.md)).

## Matrix factorization

Approximating a table by a combination of lower-dimensional factors. In the legacy
bridging model, reviewer and item factors explain shared rating patterns. They are not
identified political groups. Model and limits: [02 §A](02-scoring-engine.md).

## The model, piece by piece

The legacy prediction combines a global mean, reviewer and item intercepts, and an
interaction between their latent factors. The notation and equation belong in
[02 §A.1](02-scoring-engine.md#a1-model). The answer review uses no such fit.

## Regularization — and the asymmetric trick

A penalty on model parameters that constrains the fit. The legacy bridging model uses
different penalties for reviewer and item effects; this affects the fitted scores and
does not itself establish neutrality. See [02 §A](02-scoring-engine.md).

## Bootstrap (statistical)

Resampling observed data to study variation of an estimator. It estimates variation
under the chosen resampling scheme; it does not repair an unrepresentative sample or
validate a model's interpretation.

## Psychometrics and the latent construct

Statistical modelling of responses in relation to a proposed ability or trait. A
model's latent parameter is not automatically the competence one intends to measure.
In D45's open environment, correct answers alone do not establish unaided knowledge;
D46 states assessment objectives, not a validated measurement instrument.

## Item Response Theory (IRT)

Models an answer's probability as a function of a respondent's latent parameter and
item parameters such as difficulty and discrimination. The legacy engine's equations,
format-specific guessing floors and assumptions are in [02 §B.1](02-scoring-engine.md#b1-irt-model).
IRT is not an acceptance gate in the answer review.

## Point-biserial correlation (`r_pbis`)

Correlation between a binary item response and a numerical score. It is a diagnostic
of association in a sample, not evidence that the key is factually correct. The legacy
retention criteria are in [02 §B.2](02-scoring-engine.md#b2-retention-criteria).

## Infit and Outfit MNSQ

Mean-square response-fit diagnostics, differing in how they weight residuals. Their
intended role and implementation status belong in
[02 §B.2](02-scoring-engine.md#b2-retention-criteria).

## DIF — Differential Item Functioning

Different modelled response probabilities across groups at equal estimated ability.
Such a difference is not automatically unfair wording: it can reflect pertinent
knowledge, omitted dimensions or estimation error (D44). The legacy detector and its
limits are in [02 §B.3](02-scoring-engine.md); it does not gate the answer review.

## Mantel–Haenszel and ETS classification

A method of comparing group response odds across ability strata, and a convention for
classifying an effect size. These terms describe earlier DIF approaches; the retained
engine's actual detector is specified in [02 §B.3](02-scoring-engine.md).

## Anchor items and iterative purification

Reference items used to estimate or align ability; purification revisits the reference
set after detecting suspect items. Successful fitting does not establish that the
anchors measure the intended construct. Contract: [02 §B.4](02-scoring-engine.md).

## Mixture IRT / latent classes

A response model with unobserved components whose membership is inferred from data.
Components need not correspond to political or demographic groups. Identifiability,
finite-sample detection and semantic interpretation are distinct questions; see
[02](02-scoring-engine.md) and the scoped findings in [15](15-phase1-review.md).

## Contested facts and DTF — Differential Test Functioning

In the legacy path, a sourced fact can remain eligible despite DIF. DTF concerns
differences in expected test scores, rather than one item's response probability.
The implemented selection cost does not certify A2's whole-test guarantee. A2 remains
unresolved and is not required for the answer review (D52). Contract: [02 §B.7](02-scoring-engine.md).

## Prior and posterior

A prior describes uncertainty before observations; a posterior updates it under a
specified probabilistic model. Calling a regularized score Bayesian does not by itself
establish a posterior interpretation for that score.

## Beta distribution

A family of distributions on [0, 1], often used for uncertainty about a probability.
The author index's prior contribution and formula are specified in
[02 §C.1](02-scoring-engine.md#c1-author-score-c_a); they do not measure objective truth.

## Shrinkage

Pulling an estimate toward a reference value when its evidence is limited. Both the
reference and strength are modelling choices. The current author index and legacy
reviewer weight have different contracts in [02 §C](02-scoring-engine.md).

## Hierarchical model

A model with parameters at several levels, for example items nested under authors.
It does not automatically yield a closed-form posterior mean. The author score is the
regularized weighted average in [02 §C.1](02-scoring-engine.md#c1-author-score-c_a);
the previously claimed Beta–Beta derivation was corrected in [15, C5](15-phase1-review.md).

## Reviewers' answers and their consensus (`01` D47–D52)

The **key** is the alternative the author states is correct. Reviewers answer the item
without seeing it in their view; the program compares their answers with it. Consensus
is the operational acceptance criterion, not proof of truth or reviewer reliability.
"None is correct" differs from "don't know". Counts, outcomes and author-reputation
effects are specified in [05](05-question-lifecycle.md#the-answer-review-01-d49d52).
Reviewers have equal weight and receive no reputation update. No source is required
from author or reviewers, and external-tool use is not detected or assessed.

## Proper scoring rule

A forecast score is proper when reporting one's probability maximizes expected score;
strict propriety makes it the unique maximizer. This property depends on the information,
outcome law and selection assumptions of the scoring setup. Exploration and inverse
probability weighting alone do not establish it for a protocol. The legacy analysis and
A1's conditional closure are in [02 §C.2](02-scoring-engine.md) and [15](15-phase1-review.md).
The answer review elicits answers, not probability forecasts.

## Brier Skill Score

The Brier loss is the squared error of a probability forecast. The retired ratio-form
skill score and the retained difference of Brier scores are distinct. The difference is
strictly proper only under the conditions in [02 §C.2](02-scoring-engine.md), including a
defined baseline whose joint law with the outcome is not moved by the report. It gives
zero to copying that baseline; the fallback can be uninformative (A10). These are
single-item score properties, not guarantees for reputation or review weights. A12's
conditional offline result does not certify the answer review.

## Peer prediction

Mechanisms that score reports using other participants' reports when no independent
outcome is available. Incentive results depend on their assumptions; agreement may also
come from shared sources or tools. No peer-prediction update is used by the answer review.
The legacy proposal is in [02 §C.3](02-scoring-engine.md).

## Bayesian Truth Serum (Prelec, 2004)

A mechanism eliciting an answer and a prediction of others' answers, then scoring
their relation. It is a research reference, not an implemented part of the answer review
or a general solution to identifying truth without a reference.

## Sublinear weight / quadratic funding

A group's influence can be modelled as growing more slowly than its size, for example
with its square root. The legacy engine retains this measure for analysis; its panel
diversification is a different mechanism. Neither changes the equal-weight answer review.
See [02, Anti-collusion](02-scoring-engine.md#anti-collusion-coordination-detection).

## Spectral clustering

Grouping objects using eigenvectors of a similarity matrix. In the legacy coordination
work, the similarity definition matters: correlated behaviour alone does not establish
coordination. See [02, Anti-collusion](02-scoring-engine.md#anti-collusion-coordination-detection).

## Hash function

Maps arbitrary bytes to a fixed-length digest. A cryptographic hash can bind a recorded
value, but authenticates neither its producer nor its truth.

## Commit-reveal

Participants first submit commitments and later reveal the committed values and
nonces. The answer review closes the commitment phase before accepting reveals.
This prevents adapting a committed answer to protocol reveals, not private coordination.
Contract: [05](05-question-lifecycle.md#the-answer-review-01-d49d52).

## Merkle tree and transparency log

A hash tree supports compact proofs that data belong to a committed collection.
Authenticating a root and establishing the collection's completeness are separate
obligations. Storage contracts and limits are in [04](04-storage-network.md).

## Sparse Merkle tree and proof of absence

A Merkle tree with one position for every possible key, almost all of them empty: a key's
position is fixed by the key itself. Besides proving that a record is in the tree, you can
prove that a key has **no** record, by showing its position empty.

The planned state tree uses this property to prevent a pseudonym with a bad record
from re-entering a fork as new. It is decided, not implemented (D54, T86;
[04](04-storage-network.md)).

## Blind relay (Oblivious HTTP)

A message travels sealed for its final recipient through an intermediary. The intermediary
knows **where** it comes from but cannot read it; the recipient reads it but sees only the
intermediary as its sender. Standardized as Oblivious HTTP (RFC 9458), over HPKE (RFC 9180).

The planned channel separates addresses from event contents, assuming the relay and
member do not collude. It is decided, not implemented (D54, T85;
[04](04-storage-network.md)).

## Content-addressed storage

Identifies data by a digest of their contents. Integrity checks do not themselves
provide availability or confidentiality. In the answer review, duplicate detection is
per author, not a global plagiarism check ([05](05-question-lifecycle.md)).

## BFT — Byzantine Fault Tolerance

The ability of a distributed protocol to meet stated guarantees despite some faulty
or malicious participants, under explicit bounds and network assumptions. It is not
provided by the single-node answer-review path.

## Zero-knowledge proof

A proof of a specified statement that does not reveal its private witness under the
scheme's assumptions. Proving possession of a credential does not by itself prove
real-person uniqueness. See [03](03-identity-enrollment.md).

## Blind signature

A signature issued without the signer learning the unblinded message during issuance,
under the scheme's assumptions. It is a building block, not an end-to-end anonymity
guarantee on its own.

## BBS+ and anonymous credentials

Cryptographic building blocks for credentials and proofs of possession. The repository's
constructions, trust assumptions and implementation status are mapped in
[Architecture](../ARCHITECTURE.md#identity--anonymous-enrollment).

## Nullifier

A context-bound identifier used to recognize repeated use without revealing the
underlying credential. The project's role-pseudonym requirements include determinism
within a role and unlinkability across roles ([03](03-identity-enrollment.md)).

## Stratified sortition

Random selection within defined strata to meet a composition rule. This belongs to
earlier panel or governance designs; the answer review draws from its registered pool
without stratification by competence or political position.

## Whitewashing and the Friedman–Resnick result

Discarding a bad reputation by obtaining a fresh identity. Deterministic role pseudonyms
prevent rotation from one credential; preventing fresh credentials for the same person
also requires enrollment uniqueness ([03](03-identity-enrollment.md)).

## Stylometry

Inferring authorship from writing patterns. Cryptographic pseudonyms do not eliminate
this channel or identification through topics and timing; see [03](03-identity-enrollment.md).

## Epistocracy

Assigning political authority according to a notion of knowledge or competence. It is
a possible application discussed in the earlier design, not an outcome validated by
the answer review. D45 requires a separate justification for attributing authority
from observed performance.
