# Identity and enrollment

**Scope.** Identity-layer requirements and design assumptions. The offline answer review
checks credentials and role pseudonyms; it does not by itself establish end-to-end
person uniqueness or anonymity. Implementation status is mapped in
[Architecture](../ARCHITECTURE.md#identity--anonymous-enrollment) and the
[audit](08-formal-specification.md). Review and quota rules belong in
[05](05-question-lifecycle.md#the-answer-review-01-d49d52).

**Design assumption.** Nodes are pseudonymous IDs. No personal, demographic, or
affiliation information ever enters the system. The fact that behind each ID there is
exactly one real person is guaranteed by this layer, which must preserve anonymity at
every stage.

Guiding principle, from which everything else follows: **the state authenticates, but
does not issue.** CIE/SPID (national eID) only serve to prove "I am a real and unique
person"; the credential the node participates with is issued by a different party, and
the two do not talk to each other.

---

## The three properties the layer must guarantee

Without these, the engine's math (`02`) is decorative.

**(P1) Uniqueness.** One person → at most one active ID per role.

**(P2) Non-rotatability.** The ID must be permanent and non-reissuable. It is the most
often forgotten and most damaging requirement: if a person can abandon an ID with a
ruined reputation and get a new one, negative reputation does not exist (the Friedman
& Resnick result on cheap pseudonyms). This requirement concerns credential and pseudonym
uniqueness, not reviewer probation: the answer review has no reviewer reputation update.

**(P3) Unlinkability.** The issuer must not be able to link the credential to the
derived IDs, nor the two role IDs of the same person to each other.

---

## Enrollment flow

```
 CIE            SPID           other eID (EUDI, passport)
  │  document     │  federated    │
  │  with NFC     │  identity     │
  └───────┬───────┴────────┬──────┘
          ▼                 ▼
     ┌─────────────────────────┐
     │  common anchor          │   tax code (codice fiscale)
     │  (one per person)       │   — to which both CIE and SPID are bound
     └───────────┬─────────────┘
                 ▼
     ┌─────────────────────────┐
     │  issuing committee      │   heterogeneous, threshold t-of-n
     │  (NOT the state)        │   computes the uniqueness label in a
     └───────────┬─────────────┘   distributed, encrypted form; no single
                 ▼                  actor reads or inverts it
     ┌─────────────────────────┐
     │  anonymous credential   │   severed from the identity
     └───────────┬─────────────┘
       ┌─────────┼─────────┐
       ▼         ▼         ▼
    propose    judge     answer         three DETERMINISTIC pseudonyms
   pseudonym  pseudonym pseudonym       unlinkable to each other or to the person
      1          2          3
```

---

## The three mechanisms, one per problem

### M1 — Multiple sources without double enrollment

**Risk.** The same person enrolls once with CIE and once with SPID.

**Solution.** All sources converge on a **canonical anchor**, in Italy the tax code
(both CIE and SPID are bound to it). From there a **uniqueness label** is derived with
a computation distributed across the committee. Whether you arrive from CIE or SPID
you get the same label; the second enrollment is recognized as a duplicate.

**Beware the brute-forceable space.** The tax code is derivable from name + date +
place of birth + sex: its space is small and enumerable offline. The label must
therefore be computed with a **threshold OPRF** (oblivious PRF with a key distributed
across the committee):

- no issuer learns the tax code (it is blinded in the OPRF)
- no issuer learns the label
- the user gets a deterministic label they **cannot compute alone** (the committee is
  required)
- uniqueness is verified by checking the label is not already in the set (or the user
  proves its freshness in ZK)

Adding a new source = writing an **adapter** that extracts the anchor and feeds it into
the same OPRF → label → credential pipeline. The core does not change. It is
consistent with the pluggable model of eIDAS 2.0 / the EUDI Wallet.

### M2 — The state cannot link

**Solution.** Whoever authenticates (CIE/SPID = the state) is distinct from whoever
issues (the committee). The state knows you enrolled, but:

- does not see the label (computed in distributed, encrypted form)
- does not see the pseudonyms
- does not see the votes

**Blind** issuance (blind signature or BBS+): the committee certifies but cannot
recognize the credential when it meets it in circulation. The threshold committee
requires the agreement of `t` of `n` heterogeneous issuers: no sub-threshold coalition
can link. Same philosophy as the storage consortium (`04`): security comes from
diversity.

### M3 — One role, one pseudonym, non-rotatable

**Solution.** From the credential three pseudonyms are derived with a **per-context
nullifier**:

```
nym_propose   = H(secret, "propose")
nym_judge     = H(secret, "judge")
nym_answer    = H(secret, "answer")
```

Properties of a nullifier, all necessary:

1. **always equal** for the same context → you cannot vote/propose twice as different
   identities
2. **not traceable** to the secret key → nobody links back to you
3. **different for different contexts** → the three pseudonyms are not linkable to each
   other

Every action is accompanied by a zero-knowledge proof that the pseudonym derives from
a valid credential, without revealing which one.

**Closing whitewashing.** The three pseudonyms are deterministic: for you there exists
**only one** possible judge-identity, always the same. You cannot burn its reputation
and derive a fresh one. The uniqueness label (M1) prevents a second credential; the
determinism prevents a second pseudonym per role. Negative reputation becomes
inescapable without sacrificing anonymity.

---

## Pseudonymous does not mean anonymous

An ID that has proposed 200 questions is **deanonymizable statistically** even with
perfect cryptography. Mandatory countermeasures if the threat model includes a
state-level actor:

| Leakage channel | Mitigation |
|---|---|
| Stylometry on the question text | Mandatory structural template + automatic linguistic normalization before publication: every item comes out in the same voice |
| Timing of submission and voting | Batched publication with random delay; queue mixing; no precise timestamp in the public log |
| Topic choice (one ID only on one domain) | Per-author domain quotas; partial assignment of topics by lottery |
| Free metadata, source formatting | No public free-text field; citations only in structured form (act identifier, not an arbitrary URL) |
| Author↔evaluator correlation | M3 — separate role pseudonyms |

---

## Residual gaps (not to be hidden)

**F1 — Enrollment metadata.** SPID tells its identity provider that you authenticated
*with the enrollment service*: the fact that you enrolled is visible (not what you
do). **CIE via NFC is preferable to SPID**: reading the chip verifies the document
against the Ministry's CA without a call revealing *where* you are enrolling. It does
not eliminate the gap, it narrows it. Offer both but push CIE.

**F2 — Cross-source double enrollment for foreigners.** Someone with both an Italian
tax code and a foreign passport could enroll twice, because the two anchors live in
separate spaces. A niche case with imperfect mitigations.

---

## Cost of proposing

There is no monetary stake (D13). The current author quota and authenticated deposit
checks are specified in [05](05-question-lifecycle.md#the-answer-review-01-d49d52).

---

## Committee setup for the trial (T19) — approved contract (D63)

**Status (2026-11-11): approved (`01` D63), rectified (D66); built but the participant's side
(below, *Built*), its tests not yet run, unreviewed.**
*Decided* restates `01` D61 or an earlier decision; *consequence* is what the current code
forces; each **T19-Pn** is a choice D63 approves. Only the issuing committee is in scope (D61); the uniqueness
committee's setup stays open in T19, outside Milestone 2.

- *Decided (D61).* The trial's four consortium operators form the issuing committee.
  Credentials are synthetic, for a written list of participant slots, exchanged by hand.
- *Decided (D60).* Gate 3 runs under a new genesis whose issuer key is the committee's.
- *Consequence.* `ThresholdIssuer::new` deals every share and every pair's base OT from one
  seed, and `issue` runs the first `t` members' three phases in one process: the dealer holds
  the whole key, no message has an encoding and nothing authenticates a sender. It has no
  once-per-label check (`Issuer::issue_once` only), and `IssuanceRequest` and `BlindSignature`
  have only the test issuer's tagged encodings, unsigned (`05` §Distributed integration,
  credentials before T19). The aggregate key already has `IssuerPublic`'s shape and the genesis's
  encoding (AT-ID-09), so the genesis's schema does not change.
- *Consequence.* The genesis holds the issuer key and the network id derives from it (`04`
  T88-P1): the key exists before the network, so the setup cannot run over that network's
  bound connections.
- *Consequence.* The libraries already used carry the parts: a Feldman-based key generation
  with a proof of knowledge, pairwise base OT between two parties, and `bbs_plus::threshold`'s
  two interactive phases for a batch of signatures followed by one non-interactive share per
  signature. Composing them across machines is **bespoke** (`docs/CLAUDE.md`).

**T19-P1 — committee and threshold** (approved). The committee is the gate-3 genesis's four
members in its order, threshold 3, the consortium's `t`: issuing outside the list takes the
coalition that signs a wrong cut.
*Rejected:* 2 of 4 — weaker than a cut. *Rejected:* 4 of 4 — one lost share ends issuance.

**T19-P2 — the ceremony, before the genesis** (approved). The order is: each operator
generates its member keys (`--new-identity`); the four run the ceremony under them; the
genesis is assembled with the resulting issuer key and confirmed as `04` T88-P1 asks. The
ceremony, each operator on its own machine with its own system randomness, is the key
generation with each contribution's proof of knowledge, the base OT of every pair, and the
interactive phases for one signature per listed slot, made by three members the procedure
names. Each member keeps its share, its pairwise OT state and its part of each prepared
signature in private files; no party ever holds the key or two shares. Every member
computes the public key, and the ceremony counts only if all four confirm the same key and
the same list.
*To confirm at the first implementation step, else back to the owner:* that the library's key
generation yields `IssuerPublic` under the crate's fixed parameters, and the number of rounds.

**T19-P3 — messages** (approved). Each message has one encoding, strictly decoded, and is
signed by its sender's consortium key, under a tag of its own, over the ceremony's id, the
round, the sender and the recipient. The id is a tagged hash of the members' consortium and HPKE keys in order, the
threshold, the slot list's digest and a fresh value from each member, so no message of
another ceremony, committee or list is accepted. A message for one member is sealed to that
member's HPKE key (RFC 9180 base mode, as `05` T79-P7's inner seal). A message that fails
its decoding, signature, id, round or the library's own check is refused and ends the
ceremony for that member.

**T19-P4 — transport: files, by hand** (approved). A command-line tool apart from the node
process reads the files received and writes the files to send, one step per round; the
operators carry them over any channel. Signed and sealed messages make the carrier
untrusted: it can withhold, nothing else. No network service is added.
*Rejected:* a channel between the members' nodes — the network needs the key first (above),
and a second network before the genesis is a runtime path used once.

**T19-P5 — issuance** (approved). The procedure lists the slots, each a label; the list's
digest is in the ceremony's id. A label is an opaque slot number, never a name or an
attribute of a person (invariant 1). A participant generates its secret with the client and
sends its request for one slot. Each of the three members checks the request's proof, that
the label is listed and that it has answered no request for that slot — recorded durably
before its share leaves — and returns its share; the participant combines three shares and
verifies the credential under the genesis's issuer key. A prepared signature serves one
request, never two: a second request for a slot is refused, also after a restart, and a lost
record of the slots answered ends that member's issuance. *Rectified (`01` D66):* a request
byte-identical to the one answered gets the same share again, kept with the record. The ceremony's id fixes the
list: a slot outside it is another ceremony, so the list carries the spares the trial needs.

**T19-P6 — abort, loss and compromise** (approved). The faults are D57's: crashes, delays,
lost and repeated files. Any refusal, a different public key or list, or the operators'
decision aborts; nothing of an aborted ceremony is reused — new id, new randomness, new
shares. A lost share is not recovered: the three members left prepare signatures again
among themselves for the slots no member has answered, and a slot already answered stays
spent. *Rectified (`01` D66):* a lost share suspends the trial as a compromised one does, with
no re-preparation. A compromised share suspends the trial; continuing is a new ceremony and a new
genesis (`04` T88-P2). No refresh, resharing or rotation: T22. A member
that deviates is caught only where the library verifies its message; no robustness,
identifiable abort or Byzantine tolerance is claimed, and a member able to choose its
contribution last can bias the key's distribution.

*First step (spike, 2026-11-10):* the crates already pinned, default features:
`secret_sharing_and_dkg` 0.16.0 (`frost_dkg`: Feldman VSS with a Schnorr proof of knowledge),
`oblivious_transfer_protocols` 0.12.0 (`base_ot_multi_party_pairwise`), `bbs_plus` 0.25.0
(`threshold`). In-process exchange of each message's bytes, unsigned and unsealed
(`crates/identity/src/credential/ceremony_spike.rs`).
*Key: confirmed.* Run over G2 with base `g2` of the crate's fixed parameters, the key
generation among the four, threshold 3, yields a point that is the BBS+ verifying key as is:
version byte ‖ compressed point is accepted by `IssuerPublic::decode` and re-encodes
identically; the prepared signatures of members 1, 3, 4 issue credentials that verify, and
whose nullifier proofs verify, under it and not under another key; two shares do not
(`the_ceremonys_key_is_a_genesis_issuer_key`). *Rounds: counted; P2–P4 fix no number.* The
library needs 11 flights between members: key generation 2 (contributions with proofs, then
shares; 12 messages each), base OT 5 per pair in alternating directions (6 messages each),
phase 1 2 and phase 2 2 among the three signers (6 and 3 messages each); the member outside
the three sends nothing after base OT; issuance is one request and one share per signer per
slot (`the_ceremony_takes_eleven_rounds_between_members`). P3's id, which needs each member's
fresh value before the first signed message, and P2's confirmation add one step each: 13 in
sequence. Only phase 2 needs the base OT's output, so key generation and phase 1 can ride its
files: 7 library flights at the least. The library finishes a key generation that lacks a
member's contribution (`keygen_without_a_member_still_finishes`) and aggregates fewer than `t`
shares without error: the tool checks completeness itself, as P2's confirmation and the
holder's verification do.

*Settled (`01` D64):* the ceremony runs the 13 steps in sequence, none merged; the tool refuses
to continue without every member's contribution and every expected message; the base OT's
generator is derived by hashing a fixed label, the same for every ceremony.

*Built (2026-11-11; tests AT-ID-15–24 written, not yet run).* One member's side is
`identity::credential::ceremony`, **bespoke** and unreviewed: `Setup` (members, threshold,
named signers, slots), and `Party::advance`, which takes flight `k − 1`'s payloads by sender
and gives flight `k`'s (2 to 12; at 13 it finishes the prepared signatures). A missing payload
is refused with nothing changed (`Missing`); an unexpected one, one that does not decode or one
the library refuses is refused by its sender. Each payload is one library object in its
compressed, validated arkworks encoding, refused unless it re-encodes identically; the party's
state has one encoding. Key generation is `frost_dkg` over G2 with the ceremony's id as the
proof's context, the base OT's generator hashes `isegoria/ceremony/base-ot/v1` (D64), phase 1's
protocol id is the ceremony's id. The completeness the library does not check is checked here:
every contribution and every share before the key, every pair's five OT messages, each signer's
phase outputs at the list's length. `SlotRequest`, `Party::answer` and `combine` are issuance.
The tool is `isegoria-ceremony` (crate `ceremony`), exit code 0 or 2:

- `init <procedure.toml> <identity-dir> <ceremony-dir>` — step 1: reads `member.key` and
  `hpke.key` from the member's `--new-identity` directory, finds the member by its keys, makes
  the new directory (`0700`) and its fresh value (32 bytes of system randomness), and writes it
  to every other member.
- `step <ceremony-dir> <identity-dir> [<message-file>...]` — steps 2 to 14: takes the previous
  step's messages to this member and writes this step's; step 14 takes the confirmations and
  prints `done` and `issuer <hex>`, the 97 bytes `--genesis-assemble` takes (`04` T88-P1).
- `status <ceremony-dir>` — `member`, `phase` (`running`, `done`, `aborted`), `next` and
  `expects` (the members whose messages the next step needs), `ceremony`, `issuer`, a signer's
  `answered` count.
- `answer <ceremony-dir> <request-file> <share-file>` — a signer's share for one request (P5).
- `abort <ceremony-dir>` — the operators' decision (P6).

*Files.* The procedure TOML holds `threshold`, `signers` (member indices from 0 in genesis
order, increasing), `slots` (an increasing list of numbers below `2^32`) and one `[[member]]`
per member, in genesis order, with the `consortium` and `hpke` keys its `--new-identity`
printed; a key that is not valid or is repeated is refused by its field, and so is a shape
`Setup` refuses. The ceremony directory holds `state` (`0600`, replaced whole through
`state.new`), `lock`, `out/NN-i-j.msg` (step `NN` from member `i` to `j`, `0600`) and, for a
signer once the ceremony is done, `answered/<slot>` (`0600`, the request's SHA-256 then the
share answered, in its encoding below). A message is the header — version 1, the id, the step,
the sender's and the recipient's indices, 36 bytes — the encapsulated key (32), then as a
length-prefixed field the seal of the signature (64) and the payload. The signature is Ed25519
by the sender's consortium key over the tagged SHA-256 under `isegoria/ceremony/message/v1` of
the header and the payload, checked strictly; the seal is RFC 9180 base mode with the members'
OHTTP suite (X25519-HKDF-SHA256, HKDF-SHA256, AES-128-GCM), info `isegoria/ceremony/seal/v1`,
the header as associated data. The slots' digest is tagged under `isegoria/ceremony/slots/v1`
over the count (8 bytes) and the slots (4 each); the id under `isegoria/ceremony/id/v1` over `t`
and `n` (8 bytes each), each member's consortium then HPKE key, the slots' digest and each
member's fresh value; step 1's messages carry, in the id's place, the same fields but the fresh
values under `isegoria/ceremony/opening/v1`. The confirmation (step 13) is the issuer key's 97
bytes, the slots' digest, the signers' count and indices. A request is version 1, the slot (4
bytes), the commitment, the proof's `t` and its response; a share is version 1, the slot, the
library's share; each object is an 8-byte length and its compressed arkworks encoding. A slot's
label is its number's 4 bytes, little-endian, then 28 zero bytes.

*Behaviour.* A file byte-identical to one already taken is skipped. One that fails its decoding,
seal, signature, recipient, sender, id or step, a second message from one sender, the library's
check, or a confirmation unlike the member's own aborts: the state keeps `aborted`, the party
with its share dropped, and every later command is refused. A step lacking a message reports the
members missing and changes nothing. A step writes its messages before its state: interrupted
before the state, it runs again with fresh randomness, the earlier files not yet sent. `answer`
refuses before the ceremony is done, for a non-signer, without `answered/` (a lost record ends
that member's issuance), for an output that exists, a request that does not decode, a slot
recorded for another request, a slot not listed or a proof that fails — none of these spends the
slot; it creates `answered/<slot>` with the share, synced with its directory, before it writes
the share. A request whose SHA-256 is the recorded one gets the recorded share again, also after
a restart; a record not holding this member's share for that slot is refused (`Io("record")`).
No slot yields a second distinct share. A lost share, like a compromised one, suspends the trial
(P6): the tool prepares nothing again, and continuing is a new ceremony, under a new id, and a
new genesis. An advisory lock serializes invocations on one directory. Tests: `identity`'s
`credential::ceremony` unit tests, `ceremony/tests/` (`formats.rs`, `ceremony.rs`,
`processes.rs`), with `--new-identity`'s directories and `--genesis-assemble` (`08` §12).

*Choices left to the implementation, taken here.* Step 1's messages are signed and sealed as the
others, bound to the opening digest, the id not existing yet. Every message is sealed to its
recipient, the public ones included. The signers are not in the id, which P3 fixes; the
confirmation carries them, so two members that name different signers do not finish. A repeated
request is given the recorded share's bytes, not a share computed again. Non-signers run steps
10 to 12 with no message, so every member counts the same steps.

*Not built.* The participant's side in `isegoria-client` (`10` T79, increment 3): its secret,
the request's file, the three shares' combination and the credential's file; the library's
functions exist. *Residuals:* a payload the library decodes but mishandles could end the process
with no state written (T89's fuzzing, D62); a restored copy of a member's directory erases its
later answers, as `04`'s identity record.

**Out of scope.** The uniqueness committee, real enrollment (T20), selective-disclosure
presentation, and a threshold-signature beacon, which needs a key and a contract of its own.

---

## Implementation notes

- Prefer mature, audited building blocks: Semaphore for group nullifiers; BBS+
  schemes for credentials with selective disclosure; a threshold OPRF for anchoring.
  Bespoke cryptography is allowed when it serves the design (e.g. composing a
  threshold scheme from a vetted single-party one), but only kept small, built on
  audited primitives, and tested against known answers — not invented from scratch.
- eIDAS 2.0 mandates every EU state a digital identity wallet by the end of 2026, with
  selective disclosure: it is the infrastructure to lean on in steady state.
- Document verifiers (adapters): CIE via NFC (CieID / chip reading), SPID via
  federated IdPs, EUDI Wallet, ICAO 9303 e-passport. Each is an adapter toward the
  same pipeline.
