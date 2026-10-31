# Question lifecycle and protocol

The current Phase 1 contract is the answer review below (D49–D53). The later
numbered steps [1]–[9] specify the separate, retained forecast-and-pilot path; none is
an additional acceptance gate for the answer review.

## The answer review (`01` D49–D52)

The offline path of the first implementation, on the owner's approved contract. It is separate
from the steps [1]–[9] below and reuses none of their filters as a condition of acceptance:
bridging, the band, the appeal, the pilot, IRT, DIF, contested facts, the source check and the
admission lottery have no role in it, and the coordination detector changes nothing in it
automatically (D52). Its parameters — the group of 12, the quorum of 9, the threshold of three
quarters, the weight 1/4 and the quota bands — are provisional and uncalibrated (`14`).

1. **Creation.** The author supplies the question, its context, its alternatives and the
   alternative stated as correct (the key). No source is asked for, and nothing checks one. The
   question is immutable: a correction is a new question, and the key is never replaced.
2. **Deposit and quota.** An accepted deposit consumes one of the author's slots for the epoch:
   1, 2 or 3 from the author's reputation, 2 for a new author, fixed once per epoch (D51, `02`
   §C.1). An invalid request consumes none; a duplicate creates no other question and consumes
   no second slot; no slot is refunded, carried over or changed within the epoch.
3. **Draw.** 12 distinct reviewers are drawn at random from the available pool, with no
   competence filter and equal weights. If 12 distinct reviewers cannot be drawn, the draw returns
   an explicit unavailability; the group is never silently reduced.
4. **Answers.** Each drawn reviewer commits, then reveals, one of: an alternative, "none of the
   alternatives is correct", "don't know". The key is not shown before the commitment, and no
   reviewer sees another's answer before committing. Reviewers may use any resource; their
   sources are not collected and their tools not detected. No reviewer is replaced, and none is
   penalized for an abstention or an absence; ordinary reviews update no reviewer reputation.
5. **Closure.** An explicit event closes the round; it never closes by itself at the ninth
   answer, and there is no real deadline, early closure or automatic retry. After the closure no
   answer enters the round.
6. **Outcome.** `V` is the number of usable answers (alternatives and "none"), `C` the number
   that coincide with the key; "don't know" (an abstention) and a drawn reviewer with no answer
   (an absence) count in neither:
   - `V < 9`: insufficient participation — not a discard;
   - `V ≥ 9` and `4C ≥ 3V`: accepted;
   - `V ≥ 9` and `4(V − C) ≥ 3V`: discarded — "none" counts against the key, and the dissenters
     need not agree on an alternative;
   - otherwise: divided answers — not a demonstrated ambiguity.

   The full distribution — each alternative's count, "none", the abstentions and the absences —
   is kept with the outcome. There is no new round.
7. **Reputation.** The outcome and the author's observation are recorded together, atomically and
   once per question: accepted 1 at weight 1, discarded 0 at weight 1, divided 0 at weight 1/4,
   insufficient participation no observation (D50, `02` §C.1). An observation counts from the
   next epoch; a corrected question adds its own and erases none.
8. **Epochs.** Epochs are logical and close by an explicit offline event; the next epoch's
   quotas read the observations recorded before it.
9. **Usable questions.** Only accepted questions enter the answer review's usable set. That set
   says nothing of the whole quiz: no fairness of the quiz is guaranteed, A2's whole-test
   guarantee is not required here and A2 stays unresolved, and accepted questions do not show
   that a quiz measures D45's and D46's capabilities (D52).

Consensus is the operational criterion, not a guarantee of truth: an accepted key may be wrong,
a discarded one right (D48's accepted risks: shared errors, different competences under equal
weights, superficial reviews, coordination).

**Implementation (`10` T87, offline; done 2026-10-21 after Astra's final review, which closes T87
and Phase 1 in this path's perimeter only: `15`).** `protocol::answer_path::AnswerPath` is the
path's state, a function of its accepted events (`protocol::answer_events::AnswerEvent`) applied in
order, each checked whole before it applies; `protocol::answer_node::AnswerNode` pins the
configuration (`AnswerConfig`: the rule, the weight 1/4, the bands, `H`) in its log's first record,
writes each accepted event and replays the log on open. The outcome rule and the distribution are
`protocol::answer_rule`, the question and answers `protocol::answer`, the author's record and quotas
`protocol::answer_author` over `scoring::reputation::author_score_weighted`. Technical choices the
contract leaves to the implementation:
- *Question.* Well formed when its context and text are not empty, it has 2 to 256 alternatives,
  none empty and no two equal, and its key names one of them; otherwise the deposit is an invalid
  request. Its content id covers context, text, alternatives and key.
- *Duplicates.* A duplicate is the same author's repeated deposit of the same content id: no new
  question, no second slot, also at an exhausted quota. It is read per author, the author being
  the one the valid proof establishes, so no answer to a deposit depends on another author's
  question or its key (AT-ANS-14): two authors depositing the same content make two questions,
  and a question that differs in its key alone is a new one. This is the idempotence of an
  author's requests, not a global plagiarism check. (A global check, keyed by content and read
  before the quota, revealed another author's key; Astra found it, corrected in `822fa22`.)
- *Identities.* An author deposits under the `Propose` pseudonym its proof establishes, bound to
  the content id and the epoch (`deposit::deposit_context`); a reviewer joins the pool and commits
  under its `Judge` pseudonym, each proof bound to its action (INV-9).
- *What reviewers see.* A question is known to its reviewers by its number and its view
  (`Question::view`): context, text and alternatives, no key, and no content id, from which the
  key could be recovered. The offline state and log hold the key in clear: neither the view nor
  the per-author duplicates give any confidentiality toward whoever can read them.
- *Pool and draw.* The pool is append-only. The open epoch's explicit closure records the epoch's
  beacon value, an input the offline path takes from its caller, and how many reviewers have
  joined. A question is drawn after its deposit epoch closes, from those reviewers, seeded from
  that epoch's beacon value and the question's number (`randomness::ANSWER_DRAW`): 12 distinct,
  uniformly (`lottery::admit`), in canonical order, whatever the order they joined in. With fewer
  than 12 the round is `Unavailable` and has no group; since its inputs are fixed, the draw is
  not repeated. The code makes the draw reproducible from these inputs and lets nothing change
  them after the closure; it does not ensure that the caller did not choose the beacon value by
  trying several. **Assumption of the offline path:** the program that drives it supplies the
  random input correctly, without selecting it for a desired group. The owner confirmed for the
  first offline version (`01` D53) that joining the pool is availability, with no exit or
  availability per epoch; that the draw follows the deposit epoch's closure; and that an
  unavailable question stays so for good — its slot spent, no observation, no new attempt, not
  recovered by reviewers who join later, the same author's identical deposit still a duplicate.
- *Commit-reveal.* Drawn reviewers commit `H(answer, nonce, reviewer, question)` with a proof bound
  to the commitment; an explicit end of commits opens the reveals, so protocol reveals cannot occur before
  the commitment phase closes. A reviewer who commits and does not reveal is an absence. The closure
  needs the end of commits.
- *Epochs and time.* Epochs are numbered from 0; each explicit closure is a closing, the age unit
  of the author's observations, as `02` §C.1's contract with one closing per epoch and no skipped
  identifier. An observation records the epoch and closings of its round's closure.
- *Quota.* Fixed at the author's first accepted deposit of an epoch and kept with its count; a new
  author is one with no observation counting in the epoch.
- *Results.* Each closure carries its distribution, verdict, the rule that decided it and its
  observation with its weight; the configuration is in the log. Accepted questions are usable
  from their closure.

## Distributed integration (T79) — proposed contract

**Status (2026-10-22): proposed; not approved, not implemented.** *Approved* restates a
decision or a built network rule; *consequence* is what the current code forces; each
**T79-Pn** is a new choice awaiting the owner's decision before T79 is implemented. The
operators' side — keys, genesis, quorum, faults, recovery — is `04` §Operating contract
(T88-P1–P4, approved in `01` D57, not implemented). Nothing here changes D49–D53 or the offline `AnswerNode` and its `CloseEpoch`.

**Configuration and authorization.**
- *Approved.* Consortium-signed cuts order every event; the first of conflicting events wins,
  and a refused one is recorded without stopping the cut (`04` §Cuts). A person signs
  nothing; members log persons' events (D54, T85).
- *Consequence.* `AnswerEvent` and `NodeEvent` share a version byte and kind numbers, so bytes
  cannot tell them apart; the ledger decodes `NodeEvent` only. A question's number counts the
  accepted deposits before it in cut order, the same on every node.
- **T79-P1 — one family per network.** The genesis (T88-P1) names the answer review v1 and
  pins `AnswerConfig` in the offline record's encoding; the network id is the genesis hash,
  so every feed entry and cut signature binds the configuration. Its ledger decodes member
  objects and `AnswerEvent`s only; anything else is `NotAnEvent`. *Consequence:* the proofs'
  contexts (pool, deposit, commit) do not name a network, so one network's proof replays into
  another under the same issuer; this family prefixes them with the network id, the offline
  contexts unchanged.
- **T79-P2 — who logs what**, always on a member's circle feed; an answer event on a public
  feed is refused:

  | Event | Logged by | Applied |
  |---|---|---|
  | `JoinPool`, `Deposit`, `Commit`, `Reveal` | any member, for a person (T85) | under the offline checks; a deposit only in its epoch's cuts |
  | `CloseCommits`, `Close`, an epoch-closure request (new) | members only | as T79-P5 |
  | `Draw`, `CloseEpoch` | nobody: refused from every feed | derived, T79-P3 |

**The closing cut, the beacon and the author's time.**
- *Approved.* The cut closing epoch `e` is its commit and deposit checkpoint; reveals count
  in the next cut, at whose end the round finishes (`04` §The beacon on cuts). D53 fixes the
  pool at the deposit epoch's closure, and the draw follows it.
- *Consequence.* Offline `CloseEpoch` records pool and beacon together; on cuts the value
  exists one cut later. `AnswerPath` and `02` §C.1 count one closing per epoch with no skipped
  number, while cuts may skip epochs.
- **T79-P3 — derived transitions.** At the end of each cut, after its entries, every node
  applies: if the cut closes `e`, `e`'s seal — the pool's length, epoch `e + 1`, one more
  closing — as offline `CloseEpoch` without its value; when a round finishes with a value,
  every sealed epoch still lacking one takes it, and their pending questions are drawn in
  question order (12 distinct, or D53's unavailability). Offline `CloseEpoch { e, B }` is the
  two steps at once, so what applies between them behaves as after the offline closure: a
  join misses `e`'s pool, a deposit of `e + 1` fixes its quota from observations of epochs up
  to `e`, and a round closed there records its observation in `e + 1`.
- *Members' checks before co-signing*, beyond `04` §Proposing and signing: cut 0 is epoch 0;
  cut `k + 1` has cut `k`'s epoch, plus one if `k` closed; a cut closes exactly when T79-P5's
  requests for its epoch are counted; the member's ledger has applied every earlier cut. Since
  no derived step is written, a member still co-signs entries the state refuses (the first
  wins). T79's "an invalid derived step is not co-signed" then means: a proposal failing
  these checks is not co-signed, and a `Draw` or `CloseEpoch` object is refused on every node.

**No beacon.**
- *Approved.* Fewer than `t` reveals give no value and exclude no one (`04` rule 7); a round is
  not retried with revealed secrets. D53's unavailability concerns the pool, not the beacon.
- **T79-P4 — wait for the next value (recommended).** A sealed epoch without a value keeps its
  pool; its questions stay `Pending` until the next round that ends with a value, which seeds
  each by its number. No event, deadline, state or review round is added, and no author's
  slot is lost to a consortium failure. Residual (`04` §The epoch's beacon): `n − t + 1`
  members can stall a round, publicly, choosing between two values or delaying the draws
  without limit. *Rejected:* unavailability for good (D53 stretched to a consortium failure,
  and a veto for `n − t + 1` members); waiting on the same epoch (its secrets are spent).

**Closures: epochs, commits, rounds.**
- *Approved.* Closures are explicit: none at a count of answers, by deadline or by retry, and
  nothing enters after one (D49).
- *Consequence.* Today the proposer alone closes an epoch, after `cuts_per_epoch` cuts: a timer
  cadence, and one member's power over the deposit window and, through D53, over permanent
  unavailability.
- **T79-P5 — `t` members, on their operators' command.** A closure takes effect at the `t`-th
  distinct member's request in cut order: the `t`-th `CloseCommits` or `Close` performs the
  offline transition, a member's repeated request is refused; after the `t`-th epoch-closure
  request for `e`, the next cut closes `e`. A node emits a request only on its operator's
  explicit instruction; an answer-review network uses neither `cuts_per_epoch` nor any
  automatic trigger. A commit or reveal ordered after its closure is refused. An early
  closure takes `t` colluding members, as a wrong cut does; `n − t + 1` operators who decline
  stall it, visibly. When operators request is the trial's written procedure (T88), not code.
  *Alternative:* one member's request (first wins) — faster, but one member could force
  insufficient participation.

**Tiers.**
- *Approved.* D54's tiers (`04` §Who reads); the reviewer's view omits the key and the content
  id (§The answer review).
- **T79-P6 — every answer-review object in the circle.** Joins, deposits, commits, reveals,
  closure requests and the derived state (pool, seals, groups, tallies, verdicts,
  observations, quotas) are circle; the genesis, cuts, beacon objects and, with T86, each
  epoch's roots are public; a nym's own records are self. *Conflict:* D54 makes each decided
  item's outcome public by content id, but this content id covers the key. Whoever holds the
  view — a reviewer, later any respondent — hashes each alternative as the key until one
  matches, and an identical deposit by another author (AT-ANS-14) still under review would
  take its key from an earlier public outcome. Recommended: no object carrying a content id
  leaves the circle and no per-question outcome is public; the epoch roots commit to them.
  The options left publish a verdict per question: by number alone, or by a salted
  commitment to the content id.

**Private requests and responses (T85).**
- **T79-P7.** A person sends an event, or a read request, sealed to one member's HPKE key
  through a blind relay (the channel: `04` §Blind relays). Whether the proof binds the OHTTP
  request's encapsulation (`enc`) rather than a separate response key is part of this proposal. A read request carries a proof under its role whose context binds the
  network, the request's kind and a fresh response key; the member answers from its state,
  logs nothing, and seals the reply to that key, so a replay is answered only to the sender.
  Replies: to an event, the circle entry `(writer, seq)` that logged it, then whether a cut
  applied or refused it and why (a duplicate names only the requester's own question,
  AT-ANS-14); to a reviewer (`Judge`), each question whose group holds its nym, by number and
  view — never the key, content id, group or others' answers — and whether its own commit and
  reveal counted; to an author
  (`Propose`), its questions' rounds, verdicts and observations, its `C_a` and quota. The
  tally stays in the circle (recommended; disclosing it to the author is a separate choice).
  Retries are safe: the state refuses a repeated join, deposit, commit or reveal whichever
  member logs it, and reads change nothing. Author and reviewer use separate requests and
  response keys (invariant 4).

**Delay, duplicates and restart** (*consequences of the cut rules*). A cut waits for the
entries it names; a signed entry received twice is one entry; a person's event logged by two
members applies once, the second refused with its reason on every node; an event after its
window is refused on every node. A restarted node rebuilds the answer state from its replica
and the signed cuts (`04` §A replica on disk), so one history gives the same draws, outcomes,
records and quotas. Nothing signs the derived state before T86's roots: until then gate 3
compares a digest of it that each node's status reports per closing cut (T79).

---

## Previous forecast-and-pilot path

The remaining sections specify the retained legacy path. Its mandatory source,
lottery, reviewer forecasts, filters and appeals do not apply to the answer review.

## [2] Deposit and [3] admission by lottery

The cost of proposing is reputation and rate limits,
never money.

**Why a lottery** (`01` D10). Admission must stay within measured validation capacity.
The original illustration used 10,000 nodes × 50 answers/month = 500,000 answers.
Dividing that budget by a historical respondent count per trial ignores anchor answers
and unsuccessful batches; it is not a current throughput estimate. Use the accounting
and model-version limits in `02` §B.6 when setting admission quotas.

The lottery gives equal access in expected value and keeps the queue bounded. Anyone
can propose as much as they want; each epoch a drawn subset enters the pipeline.

**Nobody picks the draw.** The lottery, like every draw of the epoch below (reviewer
assignment, the band's extra round, exploration, contested facts, honeypot placement,
sortition), seeds from the epoch's **beacon**: a value the consortium members commit to
before the deposit window closes and reveal after it (`04` §The epoch's beacon, `01` D41),
so no deposit — not even the last one — can move it. The lottery reads the deposits as a
set, in the order of their content ids, not in the order the log lists them, which whoever
publishes the log decides. An epoch whose beacon does not form (fewer than `t` members
reveal) draws nothing; its deposits wait for the next epoch.

---

## [4] Review: random, blind, commit-reveal assignment

> **Revised by D40 (done, T57) and D41 (done, T37).** A panel holds at most one member of
> each cluster flagged by the coordination detector (D39, `review::assign_diverse`), the
> band's extra round included; every draw uses the epoch's beacon, made by commit-reveal
> among consortium members ([3] above; a threshold signature after T19).

- **Random assignment** of the `k` reviewers (odd, 7–11), stratified on the position
  `f_u` → the batch mirrors all positions of the axis. Candidates at the same position are
  ordered by pseudonym, so the panel depends on who the candidates are and on the beacon,
  never on the order a list gives them (T72; the same for the sortition). Prevents **brigading**: nobody
  chooses what to review, and the item is not searchable before the verdict. A newcomer
  — fewer than `n_min = 30` reviews on record (`02` §A.4, T39) — has the position the
  fit projects for it on the axis it does not define (the origin, with no ratings yet)
  and is a candidate like any other, at weight 0 until it is established (`01` D36): it
  fills the panel and builds its record without defining the axis.
- **Blind**: the reviewer does not see the author → prevents voting on the person.
- **Commit-reveal**: first you publish the hash of your judgment (commitment), then
  once the phase is closed you reveal it → prevents copying others and information
  cascades.
- Vote-buying neutralized: you do not know in advance what you will judge, and there
  is no verifiable receipt.

**Under D47 the reviewer answers the item.** The code still has the reviewer **declare a
probability** that the item passes validation (not a yes/no): the previous design's input for
bridging and for the evaluator score `S_u` (`02` §C.2). Reviewers work in an open environment
(`01` D45) and may consult any resource; they do not state or record the sources they consult,
and the system does not detect, infer or assess their use of outside tools (D47). The author's
source at [1] is a separate, open question. Under D48 the drawn reviewers' answers carry equal
weights, a reviewer may answer that no alternative is correct, and their consensus is the review's
operational criterion, specified for the answer review [above](#the-answer-review-01-d49d52); the
weights, probation and stratified draw described above are the existing path's.
A panel's agreement, with the key or among its members, does not show that the key is true, that
its checks were independent or that its reviewers are reliable (D48's accepted risks: shared
errors, different competences under equal weights, superficial reviews, coordination).

---

## [5b] Appeal to evidence (the key correction)

Bridging does not tell "true but divisive" from "one-sided propaganda": they produce
the same voting pattern, and no threshold saves the former (`02`, limit 1). Without an
appeal the politically most important category is systematically lost.

**Mechanism.** A question discarded at [5] *for polarization* — a wide gap between the
two sides' predicted approval (`02` §A.3), not a formal defect — whether it was scored
below the band or failed the band's re-decision (`01` D26, amended by T59), can, at the
author's initiative, **skip review and go straight to the pilot**:

- filing escrows a zero pseudo-observation, at weight 1, for the appealed item in the
  author's history, so `C_a` falls at once; an author whose `C_a` is below the stake
  floor — the prior mean of `02` §C.1 — cannot file (`01` D27, T61). An epoch's requests
  are checked one at a time in item-CID order, each counting the escrows admitted before
  it, and a request outside the appeal window is refused before its coverage is read (`02`
  §C.1). Filing starts the appeal: the record that escrows the zero moves the item to the
  pilot in the same event, or neither happens, so no stake is held for an appeal that
  cannot start; the stake is not checked a second time
- if the Level B psychometrics **promote** it — to the active pool, or to the contested
  pool when its DIF concerns a sourced fact ([7b]) — the escrow becomes a pass, the
  author's observation for having been right against the opinion filter; there is no
  additive bonus, and the item counts once (`02` §C.1)
- if it **fails**, the zero stands as the item's outcome, and it bars the next appeal
  while `C_a` stays below the floor
- only the pilot's conclusion settles the appeal: while the pilot is pending, as after an
  indeterminate stage, the escrow stays open at weight 1 (`15` A11 (f), C5–C6)

It is the only way to recover "true but inconvenient", moving the decision from peer
judgment to the data.

---

## [6]-[7] Two-stage pilot

**Stage 1 (provisional floor: 300 respondents).** The point-biserial prefilter excludes
items below `R_PBIS_MIN`; the survivors enter a one-class fit with declared formats,
guessing floors and ability shape held normal. The verdict applies the provisional
cuts on discrimination, difficulty and excess floor (`02` §B.2). The screen's study
found losses of good items and weak detection of excess guessing at this size
(`13` §8.7.5). These cuts are implemented; reliable exclusion of every guessable item
is not established. A fit that did not converge gives no verdict: the screen is
indeterminate and the item stays in `Pilot1` or `Explored`, neither rejected nor
measured; this schedules no new attempt and does not guarantee the pilot ends (`02` §B.2,
`15` A11).

Pilot answers are produced in the open environment of `01` D45. The fitted parameters are the
declared model's on those answers; what they mean relative to unaided knowledge or the use of
outside resources is still to be established, and they do not show how an answer was
produced.

**Stage 2, only for survivors.** The anonymous re-check has a code admission floor of
3,000 distinct respondents. The 1,500 floor belongs to the group-labelled calibration
path; it is not an alternative anonymous production guarantee. Required sample sizes
depend on format, effects, anchors and admission gates. T25 must determine them on the
candidate model (`02` §B.6; `13` §8.8), together with the acceptable errors of T83. A
re-check whose fit did not converge decides nothing: the item stays in stage 2, a verified
source or not, with no new attempt scheduled (`02` §B.3, `15` A4).

Requirements:

- **In batches, never a single item**: retain the protocol requirement. Historical
  single-item failures show low power in those experiments, not a general
  non-identifiability theorem (`15` A8).
- **One template per batch**: no two items of the same template ([9]) among a batch's
  trial items and the anchors its respondents answer — items answered alike beyond
  ability read as a latent class (`01` D43, `02` §B.3). Batches are composed with the
  templates apart, and the gate refuses one that is not.
- **Coverage of latent differences**: validate the detector against multiple sources
  of bias. Fitted anonymous classes do not identify named social attributes or
  guarantee that every axis of bias is found (`15` B2/B6).

Respondents are the scarce resource: they can be the same nodes under the third
pseudonym (`nym_answer`), or a separate panel-style sample. Each respondent proves that
pseudonym for the batch and the epoch before its answers count, and the sample floors
count those pseudonyms, not answer sheets: one person cannot fill a sample. The proof
does not yet cover the sheet itself, so a forwarder could alter it (`10` T70).

**Administration**: the question under pilot is mixed with already-validated ones and
the answer does not count toward the respondent's score. Whoever answers does not know
which one is on trial.

---

## [7b] Contested facts (`01` D38)

DIF does not always mean a biased item. When one camp is systematically misinformed
about a true fact, an item stating that fact shows DIF by construction. Rejecting it
would bar every fact a camp disputes from the bank, and the appeal of [5b] could never
recover it: Level B would reject it for the same reason Level A did.

**Classification, not a vote.** An item the batch flags for DIF (`02` §B.3) goes through
the source check (`02` §B.5): four steps anyone can redo from the citation committed at
deposit. If the cited primary source establishes the key, the item is a *contested
fact* and enters the contested pool; otherwise it is rejected, as before. The same rule
applies at re-validation ([8]): an active item whose DIF emerges moves to the contested
pool if its source passes the check, and retires otherwise.

**Use: selection under a DTF cost.** A test draws contested facts from the beacon
among selections whose implemented cost is below the tolerance (`02` §B.7).
Contributions can cancel within one fit; the cost adds across fits. This does not
require every item to have an opposite-leaning partner. The intended whole-test
guarantee remains open: its cross-fit bound and the contribution of active items
need correction (`15` A2). A contested fact is administered, counts exposure and
retires at the exposure limit like any pool item. Even a valid score-balance guarantee
would not protect the respondent's privacy: which contested facts
a person misses reveals their latent class, and with a respondent pseudonym that is the
same on every batch the answer sheets can be joined into a profile — open, `10` T69.

**Re-measurement.** The contested pool is re-validated in batches like the active pool.
A fact whose DIF has gone returns to the active pool; one still showing DIF stays,
measured on the new fit — which gives contested facts from different batches one set of
classes, where they can balance each other; one whose source no longer passes the check
(the act was amended) retires, as does one made obsolete (the act was repealed, [9]).

**Scores.** A contested fact is admitted to the bank: in the implemented forecast path the
reviewers who judged it are scored as for an item that reaches the pool (`o_j = 1`, `01` D35;
previous design, D47), and an appeal that ends in the contested pool is promoted ([5b], `01`
D27).

---

## [8] Active pool and re-validation

- **Periodic re-validation of the whole pool**, the contested pool included ([7b]):
  scattered distortions add up over time until they become visible; an item accepted
  today can develop DIF as the context changes.
- **Topic coverage**: DIF removes the bias of a single item, not that of the *pool*.
  You can build a test in which every item passes DIF but the choice of topics is
  skewed. Constrain coverage upstream with a **blueprint of fixed quotas per domain**,
  itself defined by stratified sortition, not by voting.

---

## [9] Retirement and item leakage (exposure)

An item used a lot gets memorized and circulates: it loses value. Countermeasures:

- a broad pool and rotation
- **parametric** items generated from templates (same structure, different values) —
  never two of one template in the same DIF batch ([6]-[7], `01` D43)
- automatic retirement on exposure detection

---

## Golden items (honeypot)

> **D47 (2026-10-21).** Golden items and exploration scored the previous design's forecast.
> Exploration is suspended; whether and how golden items serve the reviewers' reliability belongs
> to the evaluation D48 defers. The implemented paths below stay current behaviour.

> **Extended by D35 (T52, done).** Golden items stay at 5%, but alone they are too few to
> learn an evaluator's skill (about one every two epochs per reviewer). Evaluators are
> also scored on every reviewed item that reaches Level B and on a random 5% of gate
> rejections the beacon draws for the pilot, for measurement only (`protocol::exploration`:
> `Rejected → Explored → Measured`, never the pool; the outcome weighted `1/0.05` in the
> score, `02` §C.2). The explored items also measure the gate's false-negative rate.

A simple and powerful mechanism, **always active**, not only at startup. A fraction `η
≈ 5%` of the items in the review queue are of known quality (excellent or deliberately
defective: ambiguous, factually wrong, with known DIF), indistinguishable from the
rest. They give a **continuous, direct** measure of `S_u` without waiting for the
empirical cycle, and immediately catch nodes voting at random or in blocks.

**Meta-level defense.** Whoever controls the honeypots controls `S_u`. The golden
items must be produced by a committee drawn by **sortition** (stratified on the
position `f_u`, so it mirrors all positions) and rotated quickly. Sortition is the
defense against meta-level capture: they must not be choosable by anyone.

---

## Cold start (bootstrap)

- **A founder set** publicly declared, deliberately heterogeneous in orientation and
  provenance, all with identical weight `w = 1` for the bootstrap: until 30 scored
  outcomes, and never again after a change-detector alarm (`01` D34).
- No differential weight before 30 scored outcomes per node (the probation period,
  `03` P2; `01` D36, T50 — was 200), then the shrinkage of the odds weight toward 1 as
  evidence accumulates (`02` §C.2). These count forecast scores: implemented, suspended as a
  specification by D47; D48's first implementation weighs reviewers equally, with no reputation
  update from ordinary reviews.
- Start from a **low-political-temperature domain** (e.g. verifiable administrative
  procedures) to calibrate `τ`, `λ`, `k`, `ε` on real data before tackling hot
  questions.

---

## Meta-level governance

Everything that controls the system itself — scoring parameters, honeypot committee
composition, coverage blueprint, consortium composition — is decided by **stratified
sortition** and not by voting, with a qualified supermajority and a time delay for
changes (e.g. 2/3 + 30 days). Sortition is the recurring defense against capture by
whoever controls the rules: whoever can *choose* who tunes the system, controls the
system.
