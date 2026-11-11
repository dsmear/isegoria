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

## Distributed integration (T79) — approved contract (D60)

**Status (2026-11-11): approved by the owner (`01` D60); the state on signed cuts, a
member's node running it and the protocol side of participants — gateway admission, private
reads and their inner seal — are built and wired into the member's gateway (§Built on cuts,
§Built in the node, §Built for participants), with the provisional test issuer (D65) and the
participant's client (§Built for participants); the trial (`10` §2.3's increment 4) is not.**
*Approved* restates an earlier decision or a built network rule; *consequence* is what the
current code forces; each **T79-Pn**, and each choice marked *D60*, is the owner's decision
recorded in D60. The operators' side — keys, genesis, quorum, faults, recovery — is `04`
§Operating contract (T88-P1–P4, approved in `01` D57, the genesis's schema in D60; increments
A and B built under D58 and D59). Nothing here changes D49–D53 or the offline `AnswerNode`
and its `CloseEpoch`.

**Configuration and authorization.**
- *Approved.* Consortium-signed cuts order every event; the first of conflicting events wins,
  and a refused one is recorded without stopping the cut (`04` §Cuts). A person signs
  nothing; members log persons' events (D54, T85).
- *Consequence.* The headers of `AnswerEvent` and `NodeEvent` share the version byte and the
  kind numbers 1–5; their bodies differ, and strict decoding refuses almost every byte string
  of the other family, but no tag keeps the two families apart. The legacy ledger decodes
  `NodeEvent` only, the answer review's `AnswerEvent` only (§Built on cuts). A question's number
  counts the accepted deposits before it in cut order, the same on every node.
- **T79-P1 — one family per network** (approved). The genesis (`04` T88-P1, which holds its
  schema) names the answer review v1 and pins `AnswerConfig`, D49–D51's unchanged, in the
  offline record's encoding; the network id is the genesis hash, so every feed entry and cut
  signature binds the configuration, and changing a parameter is a new network. Its ledger
  decodes member objects and `AnswerEvent`s only; anything else is `NotAnEvent`.
  *Consequence:* the proofs' contexts (pool, deposit, commit) name no network, the deposit's
  (`cid ‖ epoch`, shared with the legacy path) not even a domain, so a proof replays into
  another network under the same issuer. This family prefixes each with the network id and a
  family tag, the offline contexts and the nyms unchanged (invariant 5); a `Reveal`, with no
  proof, opens only its prefixed commit.
- **T79-P2 — who logs what** (approved), always on a member's circle feed, the member counted
  by its circle key's index in the configuration; an answer event on a public feed is refused:

  | Event | Logged by | Applied |
  |---|---|---|
  | `JoinPool`, `Deposit`, `Commit`, `Reveal` | any member, for a person (T85) | under the offline checks; a deposit only in its epoch's cuts |
  | `CloseCommits`, `Close`, an epoch-closure request (new) | members only | as T79-P5 |
  | `Draw`, `CloseEpoch` | nobody: refused from every feed | derived, T79-P3 |

  A transition applies only where it is defined at its point in cut order: a `CloseCommits`
  for a question still `Pending` is refused and not counted.
- *Question size (D60).* In this family a question's encoding is bounded so that its view
  fits one reply (T79-P7): about 48 KiB, the exact value set at implementation and recorded
  in `14` as provisional — checked deterministically by the ledger and by the gateway; an
  oversized deposit is an invalid request and consumes no slot (D51). The offline path has no
  such bound.
- *Gateway admission (D60).* Without state: a gateway admits a well-formed `AnswerEvent` of a
  kind a person sends, its proof verified under the genesis's issuer key in the prefixed
  context, for a deposit also `Question::check` and the size bound; a `Reveal`, which carries
  no proof, if it decodes. Cut order alone decides what the state accepts, the same on every
  node. A relay's volume stays unbounded (T85).
- *Credentials before T19 (D60).* Persons' credentials come from a single-key test issuer
  (`identity::credential::Issuer`) held by a party that is no member, named in the trial's
  procedure before gate 2, its public key in the genesis. It establishes no real-person
  uniqueness (D55) and can mint nyms without limit, so it could fill the pool and steer the
  groups: a declared risk. Gate 1 runs every role under one operator and claims no
  independence; gate 3 uses T19's committee key under a new genesis (`01` D61; the
  committee's contract, approved in D63, is `03` §Committee setup for the trial).
  `AnonymousCredential` has a private test encoding (a file with mode 0600), `IssuerPublic`
  a public encoding with a known-answer test (built with the genesis, `04` T88-P1).
  *Provisional (owner, `01` D65):* the test issuer and its unlimited issuance are a temporary
  arrangement for tests, replaced by T19's committee; the tool says so on every run, and the
  person's commands (`--request`, `--finalize`) are `isegoria-client`'s (§Built for
  participants).
  *Built (`10` T79, increment 3, 2026-11-11):* `isegoria-test-issuer`, a crate of its own
  (`crates/test-issuer`) so that no node links the issuer's secret key; `isegoria-node` has no
  issuer command. The issuer runs `--new-key <issuer.key>`: a fresh 32-byte seed, 64 hex
  digits, in a new 0600 file, and the key printed as `issuer <hex>`, the form
  `--genesis-assemble`'s `issuer` takes. A person runs `isegoria-client --request <issuer-hex>
  <slot> <pending> <request>`: a fresh secret, its blind request for the slot, and the state that finishes it in
  a new 0600 file; the issuer runs `--issue <issuer.key> <request> <reply>`, which checks the
  request's proof and prints the slot; the person runs `isegoria-client --finalize <issuer-hex>
  <pending> <reply> <credential>`, which writes the credential to a new 0600 file only if it verifies
  under the key. The files are carried by hand; no command writes over an existing file; the
  pending file holds the secret too and stays private. A slot is an opaque number, its label
  the number in 8 little-endian bytes then 24 zero bytes, never a name or an attribute of a
  person (invariant 1): the issuer learns the slot and nothing of the secret. It signs every
  valid request, the same slot's again too: no uniqueness and no limit, the declared risk
  above. Each file is one strict encoding, its kind's tag then fixed fields, points compressed
  and scalars little-endian, decoded only as itself (`identity::credential`): the credential
  `isegoria/credential/v1` ‖ signature (`A`, `e`, `s`) ‖ secret ‖ label scalar, 198 bytes;
  the request `isegoria/issuance/request/v1` ‖ commitment ‖ label ‖ `t` ‖ the proof's two
  responses, 220; the reply `isegoria/issuance/reply/v1` ‖ signature, 138; the pending state
  `isegoria/issuance/pending/v1` ‖ secret ‖ blinding ‖ label scalar, 124. Tests: AT-ID-10–14,
  AT-T79-42–43 (`08` §12).

**The closing cut, the beacon and the author's time.**
- *Approved.* The cut closing epoch `e` is its commit and deposit checkpoint; reveals count
  in the next cut, at whose end the round finishes (`04` §The beacon on cuts).
- *D53 in this family (D60).* D53's three rules hold here unchanged, by D60, not by D53, whose
  scope stays offline: joining the pool is availability, with no exit; a question is drawn
  after its deposit epoch's closure; with fewer than 12 in the pool fixed there, it stays
  unavailable for good.
- *Consequence.* Offline `CloseEpoch` records pool and beacon together; on cuts the value
  exists one cut later. `AnswerPath` and `02` §C.1 count one closing per epoch with no skipped
  number, whereas a cut may skip epochs, may pass to the next epoch with no closing cut, and
  cut 0 may carry any epoch; the members' checks below exclude all three.
- **T79-P3 — derived transitions** (approved). At the end of each cut, after its entries,
  every node applies: if the cut closes `e`, `e`'s seal — the pool's length, epoch `e + 1`,
  one more closing — as offline `CloseEpoch` without its value; when the round of epoch `f`
  finishes with a value, every sealed epoch up to `f` still lacking one takes it, and their
  pending questions are drawn in question order (12 distinct, or D53's unavailability). A
  value never goes to a later epoch: members know it from the reveals before the cut applying
  it is signed, and could still choose that epoch's joins and deposits. Offline
  `CloseEpoch { e, B }` is the two steps at once, so what applies between them behaves as
  after the offline closure: a join misses `e`'s pool, a deposit of `e + 1` fixes its quota
  from observations of epochs up to `e`, and a round closed there records its observation in
  `e + 1`.
- *Members' checks before co-signing* (approved), beyond `04` §Proposing and signing: cut 0 is
  epoch 0; cut `k + 1` has cut `k`'s epoch, plus one if `k` closed; a cut of epoch `e` closes
  `e` exactly when the cuts before it of epoch `e` counted `t` distinct members' requests to
  close `e` (T79-P5); the member's ledger has applied every earlier cut. Since no derived step
  is written, a member still co-signs entries the state refuses (the first wins). T79's "an
  invalid derived step is not co-signed" then means: a proposal failing these checks is not
  co-signed, and a `Draw` or `CloseEpoch` object is refused on every node. A collected cut
  failing them — `t` members colluding — is not applied: the node stops progress there and
  keeps the evidence, as at a conflict (`04` T88-P3).

**No beacon.**
- *Approved.* Fewer than `t` reveals give no value and exclude no one (`04` rule 7); a round is
  not retried with revealed secrets. D53's unavailability concerns the pool, not the beacon.
- **T79-P4 — wait for the next value** (approved). A sealed epoch without a value keeps its
  pool; its questions stay `Pending` until the next round that ends with a value, which seeds
  each by its number (T79-P3). No event, deadline, state or review round is added, and no
  author's slot is lost to a consortium failure. Residual (`04` §The epoch's beacon): `n − t + 1`
  members can stall a round, publicly, choosing between two values or delaying the draws
  without limit. Testing it needs a test-only fault switch that withholds members' reveals
  while they keep signing cuts; reveals are automatic today.
  *Rejected:* unavailability for good — a veto over any epoch for `n − t + 1` members.
  *Rejected:* a retry on the same epoch — its secrets are spent.

**Closures: epochs, commits, rounds.**
- *Approved.* Closures are explicit: none at a count of answers, by deadline or by retry, and
  nothing enters after one (D49).
- *Consequence.* Without the hook (§Built in the node) a proposer closes an epoch after
  `cuts_per_epoch` cuts, a setting local to each proposer while the role rotates; co-signers check only that the epoch does not go
  back, never `closes` (`04` §Proposing and signing), so any proposer can close or skip an
  epoch at any cut, and `patience` has it propose a closing cut with beacon commits missing:
  one member's power over the deposit window and, through D53's rules, over permanent
  unavailability.
- **T79-P5 — `t` members, on their operators' command** (approved). A closure takes effect at
  the `t`-th distinct member's request in cut order: the `t`-th `CloseCommits` or `Close`
  performs the offline transition, a member's repeated request is refused; after the `t`-th
  request to close `e`, the next cut closes `e`. A request to close `e` counts only in a cut of
  epoch `e`. A member excluded under rule 6 still counts: a request is no signature, and the
  closure still takes `t` members. A node emits a request only on its operator's explicit
  command (`01` D60, through a request spool); an answer-review network uses neither
  `cuts_per_epoch` nor any automatic trigger. A commit or reveal ordered after its closure is
  refused. An early closure takes `t` colluding members, as a wrong cut does; `n − t + 1`
  operators who decline stall it, visibly. When operators request is the trial's written
  procedure (T88), not code.
  *Rejected:* one member's request, the first winning — one member could force insufficient
  participation.

**Tiers.**
- *Approved.* D54's tiers (`04` §Who reads), rectified for the answer review by D60; the
  reviewer's view omits the key and the content id (§The answer review).
- **T79-P6 — every answer-review object in the circle** (approved). Joins, deposits, commits,
  reveals, closure requests and the derived state (pool, seals, groups, tallies, verdicts,
  observations, quotas) are circle; the genesis, cuts, beacon objects and, with T86, each
  epoch's roots, which commit to the outcomes, are public; a nym's own records are self. No
  object carrying a content id leaves the circle, and no per-question outcome is public: the
  content id covers the key, so whoever holds the view — a reviewer, later any respondent —
  would find the key in at most 256 tries against a public outcome, and an identical deposit
  by another author (AT-ANS-14) still under review would lose its key. Until T86 produces
  roots for this family, nothing public commits to the outcomes.
  *Rejected:* a public verdict by question number — whoever knows the number reads it.
  *Rejected:* a public verdict under a salted commitment to the content id — salts to keep.

**Private requests and responses (T85).**
- **T79-P7** (approved). A person sends an event, or a read request on a path `…/read` beside
  `…/event`, sealed to one member's HPKE key through a blind relay (the channel: `04` §Blind
  relays). A read request carries a proof under its role whose context binds the network, the
  request's kind and a fresh response key; the member answers from its state, logs nothing,
  and seals the reply to that key inside the OHTTP reply, so a proof copied from a member's
  plaintext yields replies only its original sender can open. The inner seal is RFC 9180's
  base mode, single-shot, through the `hpke` crate behind `ohttp` (T85-P2): a **bespoke**
  composition of primitives (`docs/CLAUDE.md`), with known-answer tests. Replies: to an event,
  the circle entry `(writer, seq)` that logged it, then whether a cut applied or refused it
  and why (a duplicate names only the requester's own question, AT-ANS-14), and only to a
  request proved under the nym the event names, a refusal's reason being circle data; to a
  reviewer (`Judge`), each question whose group holds its nym, one per reply, by number and
  view — never the key, content id, group or others' answers — and whether its own commit and
  reveal counted, and (*rectified, `01` D66*) its question's phase; to an author (`Propose`), its questions' rounds, verdicts and observations,
  its `C_a` and quota. The tally stays in the circle. Retries are safe: the state refuses a
  repeated join, deposit, commit or reveal whichever member logs it, and reads change nothing.
  Author and reviewer use separate requests and response keys (invariant 4).
  *Rejected:* a proof binding the request's `enc` — `ohttp` 0.8.0 would need a bespoke OHTTP.
  *Rejected:* a context of network and kind alone — a bearer token valid for good.

**Where the state is computed (D60).** `p2p` keeps no protocol state and does not depend on
`protocol`: one hook trait in `p2p`, implemented in `node`, applies a collected cut, says
whether to sign a proposal after the last cut (`may_sign`) and whether a cut closes its epoch
(`closes`), admits a gateway's bytes (`admit`) and answers a read (`read`). The answer state
is rebuilt at each start from the replica and the signed cuts (`04` §A replica on disk), and
not persisted.

**Delay, duplicates and restart.** *Consequences of the cut rules:* a cut waits for the
entries it names; a signed entry received twice is one entry; a person's event logged by two
members applies once, the second refused with its reason on every node; an event after its
window is refused on every node. *Requirement:* a restarted node rebuilds the answer state
as above, so one history gives the same draws, outcomes, records and quotas; this holds for
`AnswerLedger` (AT-T79-9, AT-T79-12) and for a member's node (AT-T79-17). Nothing signs the
derived state before T86's roots: until then gate 3 compares each member's *state digest* (D60, D64),
a tagged SHA-256 of a canonical encoding of the whole derived state, reported in its status at
each closing cut.

**Built on cuts (`10` T79, first increment, 2026-11-04).**
`protocol::answer_ledger::AnswerLedger`, beside the legacy `protocol::Ledger`, which keeps its
contract, holds the state of one network: its `Tiers`, the issuer's key and an `AnswerConfig`
(the node pins D49–D51's, `04` T88-P1). A cut applies on a replica of that writer set (else
`Replica`), when it has the next number (else `OutOfSequence`: an earlier, repeated or later
cut, no evidence of collusion), `t` signatures in its view with rule 6's withholders not counted
(else `Unsigned`), the members' checks (else `Checks`: `t` members signed a cut they forbid) and
its entries held (else `Cut`, to wait); each error changes nothing. Then come its entries in cut
order, its seal, and the value of the round it finished; each entry's `CutReport` line names
what it did (`Accepted`) or why not (`Refusal`). The choices the contract left to the
implementation:
- *Contexts.* `answer_path::network_context`: the tag `isegoria/answer-review/network/v1`, the
  network id, then the offline context; offline contexts and nyms unchanged.
- *The request kind.* `AnswerEvent::RequestCloseEpoch { epoch }`, kind 9; the offline path
  refuses it (`Refused::NetworkOnly`), its other events unchanged.
- *An entry's checks, in order.* A member object counts only on its member's public feed; then
  the decoding (else `NotAnEvent`); `Draw` and `CloseEpoch` are `Derived` on any feed; any other
  event on a public feed is `PublicFeed`; a `CloseCommits` or `Close` is refused for an unknown
  question, then where its round is not the one it closes (not counted), then if `Repeated`;
  a deposit past the size bound is `TooLarge`; then the offline checks. A request to close an
  epoch counts in a cut of that epoch only (`WrongEpoch`), once per member; one past the `t`-th
  counts and changes nothing. Requests stay in the state.
- *The members' checks.* `answer_ledger::Next::after(last, requested, t)` and `Next::check`, a
  pure function of the last cut and the requests counted for the next cut's epoch, giving
  `OutOfSequence` or `Checks`; `AnswerLedger::next` and `check` apply them to the ledger, for
  the proposer and co-signers.
- *Question size.* `answer_ledger::MAX_QUESTION`, 49,152 bytes (48 KiB) of the question's
  encoding, checked by `fits` (`14`).
- *State digest.* `AnswerLedger::digest`: SHA-256 tagged `isegoria/answer-review/state/v1` over
  the network id, the path's canonical state (configuration, epoch, closings, pool, seals with
  their values, every question with its round and closure, slots, author records; the indexes
  rebuilt from them left out), the requests counted, the beacon rounds (each open round's
  counted commits and reveals, each finished round's cut, epoch and outcome record — value,
  revealers, withholders), the last cut's number, every refusal with its entry and reason in
  cut order (`AnswerLedger::refused`), and the reviews (each drawn reviewer's question with
  whether its commit and its reveal counted, §Built for participants); not the view a cut was
  signed in, which nodes can see differently for one history (`01` D64).
- *For the gateway (increment 3).* The ledger counts a `CloseCommits`, `Close` or
  `RequestCloseEpoch` on member `i`'s circle feed as `i`'s request, so the gateway must refuse
  those kinds from persons; `answer_gateway::admit` does (§Built for participants).

*The mapping to the offline path*, which the differential test checks: each entry a cut applies
is the same offline event at the same point — a person's event with its offline proof, the
`t`-th distinct member's `CloseCommits` or `Close` the offline one, every other entry none; a
closing cut of `e` is offline `CloseEpoch { e, B }` at its end, `B` the value `e` later takes
(the first round from `e` on that has one); at the end of the cut after `f`'s closing cut, when
round `f` has a value, offline `Draw` of each pending question of the sealed epochs up to `f`
still without one, in question order. *Evidence:* `protocol/tests/answer_on_cuts.rs` and
`answer_on_cuts_model.rs`, the modules' unit tests, `p2p/tests/blind_relays.rs` (AT-T79-1–16,
`08` §12). Gateway admission and private reads are built and wired into the member's gateway,
and the client sends both (§Built for participants); the test issuer is built (*Credentials
before T19*, above). *Not built:* gate 1 end to end (increment 4).

**Built in the node (`10` T79, second increment, 2026-11-10).** `p2p::hook::Hook` is the hook:
`apply`, `may_sign` and `closes` as above; `admit(event)` and `read(replica, request)`, which
refuse by default, are the gateway's (§Built for participants, *In the gateway*).
`node::answer::AnswerHook` implements it over an `AnswerLedger` under the genesis's tiers, issuer
and `AnswerConfig`. The choices the contract
left to the implementation:
- *Which nodes.* Every member of a tiered network (one with a genesis) runs the hook; its
  `[member]` table sets no `cuts_per_epoch`, which a start refuses there (`CutsPerEpoch`), and
  needs elsewhere. Relays and other non-members compute no answer state: they read no circle.
- *Following the cuts.* `p2p::hook::Follower` applies each cut the node's collector holds, once,
  in order from cut 0 at each start, in the view and with the signatures it was first collected
  under (`member::Collector::signed`), before the duties look and at each status. A cut the
  ledger refuses stops it there: nothing after it is applied, the duties produce nothing more,
  the request spool included (`duties_halted`), and the cut stays in the replica and in the
  status's cut lines as evidence, with `refused_cut <number> <reason>` (`checks`,
  `out-of-sequence`, `unsigned`, `replica` or `cut`).
- *The duties.* `MemberRole::duties_with` takes the hook: a proposer's `closes` is the hook's,
  and every signature the duties give, proposal or co-signature, in any view, needs `may_sign`,
  which holds only once the ledger has applied the cut before it. `patience` is unchanged for the trial (`01` D64).
- *The digest in the status.* `digest <cut number> <hex>` for each closing cut applied, the
  ledger's digest right after it, in cut order; computed again from cut 0 at each start.
- *Operators' commands.* `isegoria-node --request <node.toml> close-commits <q> | close <q> |
  close-epoch <e>` writes the event's encoding (`CloseCommits`, `Close`, `RequestCloseEpoch`) to
  one file of `data_dir/requests/`, named `<20-digit nanoseconds>-<command>-<n>`, written under a
  dot name then renamed; it refuses other words or numbers, a configuration that is no tiered
  member's and a missing data directory, reads nothing of the running node, takes no lock, and
  checks nothing of the state: the ledger decides whether a request counts (`WrongEpoch`,
  `WrongRound`, `Repeated` are refusals in the state). The duties, once the member writes and its
  circle feed caught up, log each file in name order on the circle feed, then move it to
  `requests/done/`. A request repeated by its operator is logged again, as a refused one may
  need to be; a pending file whose bytes the feed holds once more than `done/` does, logged
  before a stop that left it unmoved, is not (AT-T79-33). The status lists each executed request
  as `request <name> <seq>`, its circle entry — the `k`-th executed file with given bytes the
  `k`-th entry holding them — rebuilt at a start from `done/`. The spool
  reads files up to 4 KiB whose names start with no dot, and logs their bytes as they are.
- *Withheld reveals, test builds only* (T79-P4's switch). `MemberRole::faults`, a
  `p2p::member::Faults`, is empty by default; `Faults::withholding_reveals(epochs)` exists only
  with `p2p`'s cargo feature `fault-injection`, which only `node`'s dev-dependency enables — as
  `network`'s feature of that name (`04` §A node's own disk), so a `cargo build` or release of
  `isegoria-node` lacks it. A member so set still commits and signs as before and publishes no
  reveal of those epochs. No configuration key, flag or variable sets it: `node::plan` leaves it
  empty and a `[member]` table naming one is refused, so no trial configuration withholds, even
  in a test build. Tests set it on the role a node's configuration planned, in process.

*Evidence:* `node/tests/answer_node.rs` (AT-T79-17–22, AT-T79-30–32, `08` §12), on loopback in
process; the member processes of `node/tests/cut_export.rs` close epoch 0 by their operators'
requests.

**Built for participants (`10` T79, third increment, 2026-11-11).** What a member computes for
a person, in `protocol`, and its wiring: `AnswerHook::admit` is `answer_gateway::admit` under the
genesis's network id and issuer, `AnswerHook::read` is `answer_read::answer` over the ledger and
the node's replica. The choices the contract left to the implementation:
- *Gateway admission.* `answer_gateway::admit(bytes, network id, issuer)`, without state: the
  decoding (else `NotAnEvent`); `CloseCommits`, `Close` and `RequestCloseEpoch` are
  `MemberOnly`, `Draw` and `CloseEpoch` `Derived`; a deposit past `MAX_QUESTION` is `TooLarge`,
  then `Question::check` (`Malformed`); then the proof of a join, deposit or commit under its role
  in the prefixed context (`Unproven`); a `Reveal` is admitted once it decodes. Epochs, the pool,
  groups and repeats are the state's, in cut order.
- *Read requests.* `answer_read::ReadRequest`: version 1, the kind — 1 an event's status, 2 a
  reviewer's question, 3 an author's records —, the response key (an X25519 public key, 32
  bytes), the kind's body, then the proof as a field. A status names the event by the content id
  of its bytes, the payload of the circle entries holding it; a reviewer's read names the question
  number to start from; an author's names nothing. The proof's context is
  `network_context(network id, "isegoria/answer-read/v1" ‖ kind ‖ response key)`: the body is not
  bound, so a proof copied with another body still yields replies only to its sender. A status is
  proved under the role of the nym its event names (`Judge` for a join, commit or reveal,
  `Propose` for a deposit), a reviewer's read under `Judge`, an author's under `Propose`; a
  request whose proof fails gets no reply (`answer_read::reply` gives `None`).
- *An event's status.* Every circle entry whose payload is that content id, in entry order,
  each `(writer, seq)` with `Pending` (no cut applied counts it), `Applied` (a deposit with its
  question's number, as a duplicate's refusal names it) or `Refused` with the ledger's reason, in
  the digest's encoding. No entry at all for bytes no circle entry holds, for an event naming
  another nym or role, or for a member's request: one answer for each, so the reply is no oracle.
- *A reviewer's question.* The first question numbered from the request's on whose drawn group
  holds the nym: its number and view (the bytes AT-T79-13 bounds), whether its commit and its
  reveal counted, and that question's phase (`01` D66) — `ReviewPhase` 0 commits open, 1 reveals
  open, 2 closed, from its round in the path, without verdict or counts; none past the last. A
  closed round keeps neither group nor commits, so the ledger records, for each drawn reviewer
  and question, whether its commit and its reveal counted (`AnswerLedger::review`), in the state
  digest; nothing about the others, the content id or the key.
- *An author's records.* `C_a` (its binary64 bits) and the quota in the open epoch, then each of
  its questions by number with its phase only — pending, unavailable, committing, revealing, or
  closed with its verdict and observation: no group, pool size, count or answer.
- *Replies.* `answer_read::Reply`: version 1, the kind it answers, its body; counts as 8 bytes,
  flags 0 or 1, the quota within 32 bits, verdicts 0–3, a reviewer's phase 0–2 as one byte after
  its two flags, at most 256 alternatives; a strict decoder for the client.
  `answer_read::answer(ledger, replica, bytes)` decodes a request, computes its reply and seals
  it: the hook's `read`, which needs the replica for a status. Reads change nothing and log
  nothing; a status scans the replica's circle entries.
- *The inner seal* (**bespoke**, `docs/CLAUDE.md`). `answer_seal`: RFC 9180 base mode, single shot,
  DHKEM(X25519, HKDF-SHA256), HKDF-SHA256, AES-128-GCM — the suite of the gateways' OHTTP keys —
  with `info` `isegoria/answer-review/read-reply/v1` and an empty `aad`; the sealed reply is
  `enc` (32 bytes), then the ciphertext and its tag, 48 bytes beyond the reply.
  `ResponseKey::generate` makes a fresh pair, `derive` RFC 9180's `DeriveKeyPair`, `open` opens;
  `seal(response key, reply)` refuses a key the KEM refuses. It uses `hpke` 0.14.1, the version
  `ohttp` 0.8.0 pins, now a dependency of `protocol`: `node` and the client both depend on
  `protocol`, and `p2p` carries the sealed bytes inside the OHTTP reply without depending on it
  (D60). Its known answers: RFC 9180 A.1.1, and the reply seal's own vector, computed with an
  independent RFC 9180 implementation (Python, standard library only) that reproduces A.1.1.
- *Limits.* The author's reply lists every question of its author, 36 bytes each when closed
  with an observation, with no cursor: within `MAX_REPLY` up to about 1,700 questions.
- *In the gateway* (`04` §Blind relays). A hooked member's gateway admits a person's event on
  `…/event` by `Hook::admit` in place of its own admission, then logs it as before and answers
  with its receipt `(writer, seq, hash)`; refused bytes get 403 and are not logged. A read is
  posted on `…/read` (`blind::read_path`, `blind::seal_read`): the gateway applies the cuts its
  collector holds, hands the request's bytes to `Hook::read` with its replica, logs nothing, and
  answers 200 whose body is the hook's sealed reply; where the hook gives none — a request that
  does not decode or whose proof fails — it answers 403, the refusal an event gets, so a failed
  proof and an undecodable request look alike. `Sealed::open_read` gives the client a
  `blind::ReadAnswer`: the sealed reply, or the answer without one. A node without a hook
  answers every read 403. Relays forward reads as events, unchanged, by key id.

*Evidence:* `protocol/tests/answer_participants.rs` (AT-T79-23–28), the unit tests of
`answer_seal` (AT-T79-29) and `answer_ledger` (AT-T79-15, AT-T79-28); the wiring in
`node/tests/answer_node.rs` (AT-T79-34–37, AT-T79-41) and `p2p/tests/blind_relays.rs`
(AT-T79-38–40), on loopback in process; not yet run (the owner runs them with the other
branches).

**The client (`10` T79, third increment, 2026-11-11).** `isegoria-client`, a crate of its own
(`crates/client`, D60), so a person's secrets stay out of members' processes; it reads only the
files named on its command line and prints to standard output, exit code 0 or 2. Every file it
writes is new (never over an existing one, and of two outputs neither is left when one cannot be
made), 0600 where it holds a secret or a person's event; every file it reads is bounded and
strictly decoded as its kind alone. It takes the genesis document (`04` T88-P1) and checks it as
a node does — the family, D49–D51's `AnswerConfig`, the issuer key, the tiers — and every
credential is decoded and verified under the genesis's issuer key before any proof is built from
it (`Unverified` otherwise). No command asks for or writes a name or an attribute of a person.
- *Credentials.* `--request` and `--finalize` (*Credentials before T19*, above), and the
  committee's participant side (`03` T19-P5): `--slot-request <genesis> <slot> <pending>
  <request>` writes a fresh secret's `SlotRequest` for a listed slot (below `2^32`), the one file
  the three signers answer, and its `PendingIssuance` in the pending file's encoding above;
  `--combine <genesis> <pending> <credential> <share>...` combines the signers'
  `SignatureShare` files with `ceremony::combine` and writes the credential only once it verifies
  under the genesis's issuer key (`Combine(Shares | Slot | Signature)` otherwise).
- *Events.* `--join <genesis> <credential> <event>`, `--deposit <genesis> <credential> <epoch>
  <question.toml> <event>`, `--commit <genesis> <credential> <question> <answer> <commitment>
  <event>` and `--reveal <genesis> <credential> <commitment> <event>` build a `JoinPool`
  (`Judge`), a `Deposit` (`Propose`), a `Commit` (`Judge`, a fresh 32-byte nonce) and the
  `Reveal` that opens it, each proof in the network-prefixed context (T79-P1), the reveal's nym
  the `Judge` proof's id; each is checked with `answer_gateway::admit` before it is written, so a
  deposit malformed or past `MAX_QUESTION` writes nothing, and each prints `event <content id in
  hex>`. A question's file is TOML with exactly `context`, `stem`, `alternatives` (strings, their
  UTF-8 bytes) and `key` (the stated alternative's index). An answer is an alternative's index
  `0`–`255`, `none` or `dont-know`. The commitment file (0600) is `isegoria/client/commit/v1` ‖
  the question (8 bytes, little-endian) ‖ the answer's code (2) ‖ the nonce (32), 67 bytes.
- *Sending.* `--send <genesis> <announcement-hex> <relay> <event>`: the announcement is a
  member's signed key configuration as `isegoria-node --announce` prints it, verified under the
  genesis's tiers and carrying that member's HPKE key in the genesis; the relay is a multiaddress
  ending in `/p2p/` and the peer id of a transport key the genesis lists; the event is admitted as
  above, then `p2p::blind::seal` and `submit` from a new transport key; a receipt prints `logged
  <writer> <seq> <hash>` in hex, any other answer exits 2 as `NotLogged`.
- *Reads.* `--read <genesis> <credential> <announcement-hex> <relay> status <event> | reviewer
  <from> | author` builds one `answer_read::ReadRequest` with a fresh `answer_seal::ResponseKey`,
  proved under the event's role for a status (`Judge` for a join, commit or reveal, `Propose` for
  a deposit), `Judge` for a reviewer's read, `Propose` for an author's: each request its own key,
  the two roles never in one request (invariant 4). The reply opens only under that key and only
  as a reply of the read's kind; it prints one fact per line (`entries`, `entry <writer> <seq>
  pending | applied [question <q>] | refused <reason>`; `question none` or `question`, `phase`,
  `committed`, `revealed`, `context`, `stem`, `alternative` lines, texts quoted and escaped;
  `reputation`, `quota`, then `question <q> <phase>` with a closed question's verdict and
  observation). `client::read::send_read` seals the request on `…/read` (`p2p::blind::seal_read`),
  submits it through the relay and opens the gateway's answer (`Sealed::open_read`): a 200's body
  is the sealed reply, any other answer the error `NoReply` with it.

*Evidence:* `client/tests/` (`credentials.rs`, `events.rs`, `reads.rs`) and `test-issuer/tests/
files.rs` (AT-T79-42–49, `08` §12); not yet run. Not over a relay on loopback: no test sends an
event or a read through a running relay and gateway.

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
