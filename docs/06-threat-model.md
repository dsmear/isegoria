# Threat model

This page describes the **offline answer review (D49–D53)**. Its contract is in
[05](05-question-lifecycle.md#the-answer-review-01-d49d52); the owner's accepted risks
are in [01, D48](01-decisions.md). Earlier filters and network mechanisms are separate.

## Attack surface and countermeasures

| Risk | Protection or limit in the answer review |
|---|---|
| Invalid or repeated actions | Credential proofs, role pseudonyms and state-transition checks; per-author duplicate detection and proposal quotas. These do not establish that enrollment issued only one credential per real person. |
| Copying revealed answers during commitment | Commit-reveal separates the phases. It does not prevent private communication, shared sources or coordination outside the program. |
| Learning the key | The reviewer view omits the key and content id; deposits do not consult another author's duplicates. The offline state and log still contain the key in clear. No confidentiality is promised to their readers. |
| Choosing a favourable panel | Pool and beacon are fixed at the deposit epoch's closure; the draw is reproducible from them and the question number. The caller can try beacon values: honest supply of the random input is an assumption, not an enforced property. |
| Majority capture, shared errors, superficial reviews, coordinated reviewers | Accepted risks. Equal-weight consensus can accept a wrong key or reject a correct one. Bridging, DIF and coordination detection do not change this path's outcome. |
| Unequal expertise or an unsuitable pool | Accepted risk. Selection is random, not by competence; suitability to the context must come from the people maintaining the pool. |
| Reviewer reputation accumulated to gain influence | Ordinary reviews neither update reviewer reputation nor increase weight. This removes that weight-accumulation channel, not the possibility of coordinated participation. |
| Author reviewing their own question | Separate role pseudonyms do not identify the same person across roles, so the draw does not exclude that person. Equal votes and random assignment do not eliminate this risk. |
| Wrong or unclear key | Reviewers can choose another alternative or "none is correct". The tally determines the outcome; divided answers do not prove ambiguity. No automatic correction or source check is performed. |
| Abstention or absence to block a round | No replacement or penalty. Insufficient participation produces no author observation. New rounds are outside the contract. |
| Too few reviewers | A pool too small at the fixed closure leaves the question permanently unavailable, with its slot consumed and no author observation (D53). |
| Statistical identification or identity-layer failure | Role pseudonyms alone do not prevent identification through text, topic or behaviour. Requirements and residual gaps are in [03](03-identity-enrollment.md); the offline path does not certify end-to-end anonymity. |
| Biased coverage of the final quiz | Not addressed by item consensus. Quiz composition, exposure and retirement are outside this closure; A2 remains unresolved in its separate scope. |

Internet, documents and AI are part of the operating conditions. The system does not
detect, infer or assess their use, and reviewers do not record consulted sources.
Agreement does not establish independent checking. D45 leaves broader policies about
delegation, automation and impersonation unsettled; D48 accepts coordination as a risk
of this first mechanism.

## Corner cases verified in testing

Answer-review checks and their execution/review provenance are recorded in
[15](15-phase1-review.md#answer-review-t87-final-review-and-closure-2026-10-21).
The key-disclosure regression is AT-ANS-14. These checks establish the recorded contract
properties, not error rates in a real reviewer population. Parameters remain provisional
and uncalibrated; the decoder has not been fuzzed.

### Cost of bipartisan corruption

The bridging simulations belong to the earlier mechanism. Their assumptions and measured
rates are in [13 §7.4](13-characterization.md); they provide no protection to this path.

### The biased item that passes review

The earlier prototype's peer-review/DIF example is documented in [sim](../sim/README.md).
There is no DIF gate in the answer review.

### Latent axis recovery

Historical recovery experiments are documented in [sim](../sim/README.md) and
[13](13-characterization.md). They do not establish political-group identification or
reviewer competence in the current open environment.

## What the system does NOT solve (structural limits, not bugs)

### L1 — The true-but-divisive false negative

A correct key can fail consensus. The earlier appeal and contested-fact mechanisms are
specified in [02](02-scoring-engine.md) and [05](05-question-lifecycle.md); neither applies here.

### L2 — Bias of the pool, not of the single item

Acceptable individual questions do not establish representative subject coverage (D44).
The usable set is not yet a composed quiz.

### L3 — The elite-consensus blind spot

Shared knowledge and blind spots can produce a confident but unsuitable consensus.
Random selection within a pool does not correct how that pool was formed (D48).

### L4 — "Competence" is still a construct chosen by someone

Correctness in an open environment does not establish unaided knowledge or autonomous
understanding. D46's assessment objectives remain unvalidated by this path.

### L5 — The consortium's initial selection has no technical solution

Network governance belongs to [04](04-storage-network.md), outside this offline closure.
The offline caller's trust assumptions are stated above.

### L6 — The political constraint of weighted voting

Using performance to assign political weight or authority needs a separate justification
(D45). Nothing in this closure establishes one.

## Phase 2 network boundary

The node process handles replication, cuts and the beacon, not the answer-review
lifecycle. Configured with tiers, it serves a peer outside the circle the public tier
only, its binding checked against the peer's authenticated transport key (T77, tested on
loopback); without tiers it still serves its whole set to any connected peer. The blind
relay channel exists (T85's first increment, tested on loopback) but a node process admits
no event through it before T79; the state tree (T86) is unimplemented, and relays still
write in a network without tiers: run such a network only among machines its operator
controls. Draft and panel privacy, unlinkability by network address and the private fork
path are design obligations, not protections the runtime already provides.

D55 targets independently operated experimental nodes; independent administration
does not make the present cut protocol Byzantine-tolerant. Its agreement argument
assumes members follow the rules, and view changes require majority quorums. The
beacon retains withholding bias/stall risk; writer recovery still relies on the first
peer that answers, which can be stale. T88's approved contract (`01` D57,
[04 §Operating contract](04-storage-network.md#operating-contract-t88)) states these limits
and requires two verified members and no automatic resumption after loss. Increment A
(`01` D58) builds the stop after a detected loss for tiered networks; the two members are not
built. Both must be exercised before the independent-operator trial, not hidden by a
single-operator demo.

A member sees the circle's plaintext. Public roots bind records but do not let an
outsider recompute private results or guarantee their availability. Independent
operators and blind relays do not eliminate member leaks or relay/member collusion.

Decisions are in [01, D54–D55](01-decisions.md). Network contracts and residual limits
are in [04](04-storage-network.md); the proposed integration contract in
[05](05-question-lifecycle.md#distributed-integration-t79--proposed-contract).
[10 §2](10-roadmap.md#phase-2--p2p-network) defines the planned trial; its evidence
has not been established by the existing loopback tests.
