# Threat model

## Attack surface and countermeasures

| Attack | Primary countermeasure | Ref. |
|---|---|---|
| Sybil | Enrollment layer: externally-certified uniqueness | `03` P1 |
| Whitewashing (abandon a compromised ID) | Non-rotatable ID + deterministic pseudonyms + zero-weight probation | `03` P2, M3 |
| Statistical deanonymization of the proposer | Text normalization, temporal mixing, domain quotas | `03` |
| Joining the proposals register with the votes register | Unlinkable role pseudonyms | `03` M3 |
| Brigading specific items | Random reviewer assignment; item not searchable before the verdict | `05` [4] |
| Herding / information cascades | Commit-reveal, blind judgments, no visibility of others' votes | `05` [4] |
| Majority capture | Bridging: the majority alone is not enough, cross-cutting approval is needed (on master the score still keeps part of the camp-size effect; corrected by the side-balanced score, D32/T49). Under `01` D48 the new review's operational criterion is the consensus of drawn reviewers with equal weights: the draw does not remove this risk (shared errors, coordination are accepted), and whether bridging keeps a role is still to decide | `02` A, `01` D48 |
| Coordinated cartel | Detection on the correlation of model residuals over long histories — a jittered cartel is flagged, like-minded honest reviewers are not (D39, T56); a detected cluster is never seated twice on a panel (D40, T57); the √k discount stays in the engine for analysis | `02` anti-collusion, `05` [4] |
| Long-con (reputation accumulated and spent) | Implemented on the previous design's forecast scores: the CUSUM change detector on the per-item scores sends the reviewer back to probation (D34, T51); `w_max` cap on the odds scale (D33, T50); continuous honeypots. In D48's first implementation every reviewer weighs the same and ordinary reviews update no reviewer reputation, so there is no reviewer weight to accumulate; a later evaluation of reviewers would have to specify its own protection | `02` C.4, `05`, `01` D47, D48 |
| Biased item passing peer review | Field-data DIF (does not depend on opinions) | `02` B.3 |
| Biased item passed off as a contested fact | The source check: the key must follow, by a declared rule, from a primary source cited at deposit — an interpretive key is rejected on DIF as before; and a contested fact reaches a test only in a selection whose admission cost is within the tolerance: the sum of each fit's estimated DTF over its own distribution and counted classes (D38). Cancellation can occur within a fit; this cost does not certify the whole test's DTF (`15` A2) | `02` §B.5, §B.7 |
| Profiling a respondent's camp from the contested facts they miss, joined across batches | **Open** (`10` T69): the respondent pseudonym is the same on every batch, and contested facts are administered continuously since D38; a batch-scoped nullifier, sheets never kept with the id, or a per-epoch rate limit are the options, none chosen | `08` §8.2 PRIV-P8, `10` T69 |
| Grinding the public randomness (the lottery, panels, exploration, honeypot placement, sortition) | Every draw seeds from the epoch's commit-reveal beacon among consortium members, committed before the deposit window closes and revealed after, so no deposit, log order or checkpoint head moves it; the lottery reads the deposits as a set. A member that withholds its reveal is excluded from the epoch's signing set and recorded; the last revealer can still choose between two values, publicly, until the threshold signature (T19) | `04` §The epoch's beacon, `01` D41 |
| Vote-buying | Random assignment + no verifiable receipt | `05` [4] |
| Author drawn onto their own item's panel | Accepted residual risk: the author's judge pseudonym is unlinkable by design, so it cannot be excluded; random assignment makes it one vote in 7–11, bridging needs agreement across camps, and honeypots measure reviewers | `05` [4], `08` §9.1 |
| Item leakage (exposure) | Broad pool, rotation, parametric items, retirement on detection | `05` [9] |
| Meta-level capture (parameters, honeypots, consortium) | Stratified sortition, supermajority + delay, freedom to fork | `05`, `04` |
| Score falsification by a signer | Deterministic reproducible computation: anyone redoes the math and unmasks it | `04` |
| Betrayal by the whole consortium | Freedom to fork: the network abandons it with all the data | `04` |
| Rewriting the past | Hourly anchoring to a public chain | `04` |
| Collusion of the issuing committee | Threshold t-of-n issuance with heterogeneous issuers | `03` M2 |
| Answers or reviews produced through delegation, automation, impersonation or strategic coordination outside the system | **Open** (`01` D45): an open environment is assumed, outside resources — the Internet, documents, AI — are part of the conditions, and these behaviours remain to be distinguished and governed; no rule adopted. The system does not detect, infer or assess the use of outside tools (D47, D48) | `01` D45, D47, D48 |
| A wrong, ambiguous or contestable key stated by the author | **Partly decided** (`01` D48): a reviewer may answer that no alternative is correct, distinct from choosing another alternative; a question is never corrected retroactively and its key never replaced automatically — to correct it, the author submits a new question. What happens to the item, the thresholds and the outcomes are still to define; a balanced split of correct and wrong answers is a signal to study, not a validated criterion or a rejection rule | `01` D47, D48 |
| Reviewers' answers aligned on the author's key or on the majority without independent checks (copying, shared sources or tools, superficial reviews, coordination) | **Accepted risk** (`01` D48): in the first implementation the consensus of equally weighted answers is the review's operational criterion, not a guarantee of truth; shared errors, superficial reviews and coordination are accepted, and the draw eliminates none of them. Agreement is not demonstrated reliability, and none is rewarded (no Schelling point, D6). Commit-reveal and random assignment still apply. A later evaluation of reviewers would need a credible reference and a distinct decision | `01` D47, D48, D45 |
| Reviewers of different competence counted alike, or a pool unsuited to the item's context | **Accepted risk** (`01` D48): reviewers are drawn from the pool with no selection by specialist competence and carry equal weights; that the pool suits the context is a premise, not a property the draw demonstrates | `01` D48 |

---

## Corner cases verified in testing

Results of the simulations in `sim/`. These are the engine's **acceptance tests**: the
implementation must reproduce them.

### Cost of bipartisan corruption

A 60/40 unbalanced network. A cartel pushing a partisan item:

- with majority voting: ~40 nodes from its own camp are enough → PASSES
- with bridging: 40 nodes from its own camp barely move the score (0.52 → 0.54); the
  item passes only once about **60 of the 80 of the opposing camp (75%)** approve it —
  55–60 in 90% of 200 drawn orders (T24, `13` §7.4)

Bridging drastically raises the cost of capture. This is the existing mechanism; under `01`
D48 the new review's criterion is the consensus of equally weighted answers, and whether it keeps
bridging, and this protection, depends on the previous filters' role, still to decide.

### The biased item that passes review

A question with strong DIF (β₂ = 0.72 in the data: item 04 of the reference fixture)
**passes peer review** — side-balanced score `S_j = 0.84` against `τ = 0.80`, both
sides approving it — and is stopped only by Level B. This is the scenario the evidence
filter exists for: human review does not see the bias, the data does.

### Latent axis recovery

Bridging reconstructs the ideological axis with correlation 0.99 against the true one,
with nobody declaring anything. Latent-class DIF reconstructs a hidden axis (education)
with correlation 0.5–0.8 without observing it, provided ≥2 distorted items are in the
batch.

---

## What the system does NOT solve (structural limits, not bugs)

These must be made explicit: they are properties of the design, not implementation
defects.

### L1 — The true-but-divisive false negative

Bridging does not tell a true, polarizing fact from one-sided propaganda: they produce
the same voting pattern, and **no threshold saves it**. This is the most serious
limitation. Partial mitigation: the appeal-to-evidence channel (`05` [5b]); when the
division lies in knowledge of a sourced fact rather than in opinions about the item,
Level B shows DIF too, and the contested-facts pool keeps such a fact in the bank, drawn
only in sets within the admission cost (`05` [7b], `01` D38). The whole-test DTF
guarantee remains open (`15` A2).

### L2 — Bias of the pool, not of the single item

You can build a test in which every item passes DIF but the choice of topics is skewed
(only parliamentary procedure, never labor law). Mitigation: a blueprint of fixed
quotas per domain, by sortition (`05` [8]).

### L3 — The elite-consensus blind spot

An item can pass bridging and political DIF yet be distorted on a socio-economic axis.
Mitigation: multi-axis DIF, which must be actively sought.

### L4 — "Competence" is still a construct chosen by someone

Psychometrics verifies that the test coherently measures *one* thing, not that that
thing is the right thing to measure. In the open environment of `01` D45 answers are produced
with outside resources available; what that one thing means relative to unaided knowledge is
still to be established, and a correct answer alone does not show it.

### L5 — The consortium's initial selection has no technical solution

It is a human decision about whom to trust. Made bearable — not fatal — by
reproducibility of the computation and freedom to fork (`04`).

### L6 — The political constraint of weighted voting

In the electoral use case, weighting the vote by competence (epistocracy) is
contested: Brennan defends it (*Against Democracy*), Estlund criticizes it with the
demographic objection — even a perfectly neutral test weights participation on a
variable correlated with education, income, and native language. It is also in tension
with universal suffrage. This is not a judgment on the project but a design constraint:
if the goal is electoral, the main obstacle will not be technical. Under `01` D45 test answers
are produced in an open environment; using scores to attribute weight or authority needs a
justification not yet established.

The same infrastructure has less contested and more immediately achievable
applications (independent certifications, fact-checking, licensing exams), useful also
as a testbed, with the same underlying problem — no authority is credibly impartial —
but without the constitutional friction.
