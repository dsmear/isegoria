# Storage and network

The network must preserve a verifiable history and make conflicting signed histories
detectable. Availability also requires surviving copies; checking a result requires its
inputs. Neither signatures nor public anchoring alone guarantee either property.

**Final choice.** A P2P network of signed logs (gossip + DHT + append-only + CRDT),
with a consortium of a few dozen heterogeneous signers as the backbone, erasure coding
for durability, anchoring to a public chain for long-term immutability. **No global
permissionless consensus.**

**Phase 2 target (D55).** A small experimental P2P testnet with independent operators,
each administering its nodes and holding its own keys. Single-operator processes are
an intermediate check. The signing set stays defined by the network configuration;
P2P does not mean anyone may sign the state. `10` §2 owns the work order and completion
gates. `05` §Distributed integration proposes the answer-review interface.

**Trust boundary.** The current cut protocol's agreement argument assumes members
follow its rules, with majority quorums for view changes; it is not a Byzantine-fault
tolerance proof. Independent operators do not remove that assumption. §Operating
contract proposes the trial's membership, quorum, fault model and recovery (T88). Preserve
the documented beacon bias and recovery/liveness limits in the trial's claims.

---

## Why not a permissionless blockchain

- **Cost and latency.** Every write costs and has seconds/minutes of latency.
- **Public by construction.** Everything is visible to everyone forever: questions
  under review must stay encrypted until publication, and voting patterns on a public
  chain would be a gift to anyone wanting to deanonymize.
- **Computation too heavy.** Factorization and IRT do not run on-chain; you would end
  up keeping the data on the chain and doing the math elsewhere.
- **Ordering stays with the consortium.** Replication can merge a set of signed entries,
  but conflicting quota uses and state transitions still need a shared order.
  Consortium-signed cuts supply it; nullifiers alone do not.

Anti-censorship comes from the **diversity of operators** and from the
**reproducibility of the computation**, not from permissionless consensus.

---

## The four P2P structures (non-blockchain)

They separate transporting and retaining data from the consortium's agreement on
event order (§Cuts).

| Structure | Purpose |
|---|---|
| **Gossip / epidemic** | propagate public entry identifiers; circle announcements go directly to authorized peers |
| **DHT** (distributed hash table) | find public content: given its CID, which node can serve the object |
| **Signed append-only logs** | immutability: an add-only register, each row containing the previous one's fingerprint; altering one breaks the chain visibly |
| **CRDT** | converge copies edited in parallel, with no arbiter (collaborative editors) |

Public objects are identified by their content hash. The DHT maps a CID to possible
providers; the client checks the returned object and its signed entry. Circle objects
travel only through authorized replication. Signed logs authenticate entries and their
chains; the consortium's signed cuts, not the DHT, establish their agreed order.

Real projects already doing this: the Bluesky protocol (per-user signed logs +
rebuildable indexes), Secure Scuttlebutt (a signed-log, gossip social network that
works offline), Automerge/Yjs (mature CRDTs). None is a blockchain.

### Public object discovery (T75)

A light client knows a network's trusted tier configuration, a CID and at least one
bootstrap address. `p2p::discovery::fetch` uses a separate Kademlia/object-request swarm,
without sync, gossip or a full replica. A tiered node advertises public objects from its
verified replica, including restored and newly synchronized entries. Nodes without tiers
do not offer this service. Discovery can open connections beyond the bootstrap peers;
T77's binding and replication checks apply to those connections too. Only public entries
authorize an object response: knowing a
circle CID, even as a member, does not make it readable through this endpoint. Bytes
also recorded on a public feed are public through that entry.

The per-network protocols are `/isegoria/<network-id-hex>/kad/1` and
`/isegoria/<network-id-hex>/object/1`. A request is version 1 followed by the 32-byte CID.
A missing response is `[1, 0]`; a present one is `[1, 1]`, the 168-byte signed entry and
the object bytes. Reads enforce 33 bytes for requests and `MAX_OBJECT + 170` for responses,
where `MAX_OBJECT` is 16 MiB. The client requires a configured public writer, a signature
bound to its network, the requested CID in the entry and matching object bytes. This
proves content and writer provenance, not feed continuity, inclusion in a cut, freshness
or validity under the application protocol.

Provider records are routing hints, never authority. An invalid or unreachable provider
is skipped while other discovered providers remain. The client tries at most 32 distinct
providers, with one object request active at a time, 3-second Kademlia/request timeouts
and a caller-supplied deadline for the entire operation. Exhaustion or timeout is not a
proof that the object does not exist. Honest nodes originate only public provider records;
a malicious DHT participant can inject arbitrary CID strings as hints. The lookup reveals
the queried CID and transport metadata to contacted peers; it does not use T85's OHTTP.

The local provider catalogue grows with the verified public replica and selected archive
shards (§Archive holder discovery). Its shared announcement queue deduplicates routing
keys and limits application-started concurrent queries to 16. Remote
provider records use a separate bounded store (1,024 keys, up to 20 providers per key),
and arbitrary DHT value records are refused. Kademlia packets are limited to 64 KiB;
object requests have at most eight concurrent streams. These bounds are not a Sybil or
traffic-volume defense. Listening addresses are advertised directly; NAT traversal and
independent-operator reachability are outside this increment.

**Implemented and tested on loopback (2026-10-25, AT-NET-25).** A client retrieves a public object from a previously unknown
provider through a bootstrap that holds no object; circle CIDs are neither advertised nor
served, including a direct request that already knows the CID. Bad hashes, signatures,
private writers and another network are refused. Invalid, stale and silent providers
exercise fallback and the whole-lookup deadline. Restored and synchronized public entries
become discoverable. Codec boundaries, maximum-sized objects and separate local/remote
provider-store limits are tested. The client is a library API; no user-facing CLI, archive
placement/repair (T84) or multi-machine validation is supplied here. No fuzzing or
mutation campaign has been run on the new codecs. Enabling libp2p's `kad` feature adds
`libp2p-kad` 0.48.0, `uint` 0.10.2 and `crunchy` 0.2.4 to `Cargo.lock`; existing
dependency versions are unchanged.

---

## The consortium as backbone

A pure P2P network has an Achilles' heel: a poorly-replicated datum can vanish when
the node holding it goes offline. The consortium solves this: a few dozen operators
**heterogeneous, always-on, in different jurisdictions** each keep a full copy and
periodically sign the state. They are the stable backbone; the citizens' nodes hook on
around them.

### Why few signers

The machines that sign the state must agree, and BFT agreement costs ~n² messages:

| Signers | Messages/agreement | Assessment |
|---|---|---|
| 4–50 | 16–2,500 | efficient |
| 100 | 10,000 | still manageable |
| 300 | 90,000 | slow |
| 1,000 | 1,000,000 | impractical |
| 1,000,000 | 10¹² | impossible |

Beyond a hundred, coordination becomes slow. **But few signers does not mean a small
or poorly-distributed network**: security comes from the diversity of who controls the
machines, not from the number. 30 operators chosen for being diverse (universities,
NGOs, opposing outlets, in different countries) are harder to corrupt all at once than
10,000 servers in the same data center. To censor, a government would have to compel
entities under different laws simultaneously, and one that refuses is enough for the
tampering to become visible.

### Consortium selection and differentiation

**Differentiation (easy, it is math).** You need not declare that an operator is
different: you measure it from behavior correlation (the same tool as bridging). Two
operators that always sign the same things at the same moments are a single block even
if on paper they are two entities. A consortium with two overly-correlated operators
has a problem visible to everyone.

**Selection (hard, no purely technical solution).** A combination of:
- reserved seats per category, including categories of **declaredly opposite**
  orientation (capturing one is not enough)
- an entry deposit against fake identities
- a per-category cap against buying seats

The **two ultimate defenses** matter more than any entry rule:
1. **Reproducibility of the computation** — members holding the private inputs can
   recompute and check results. Outside D54's circle, a reader verifies signatures
   and commitments; a root alone does not prove that the computation was correct.
   Public proofs of computation remain future work.
2. **Freedom to fork** — if the consortium betrays, the public history lives on every node
   and the code is open: the community takes it, abandons that consortium, and restarts
   with a different set of signers. The votes and the drafts under review stay with the
   members (`01` D17); a fork keeps them through a member that joins it, and otherwise
   restarts from the state tree, each nym opening its own leaf (`01` D54, §Who reads). Not
   the barrier to entry, but the freedom to leave, keeps the consortium honest.

System parameters and meta-level composition: **stratified sortition** (see `05`), not
voting.

### The epoch's beacon (D41)

The network supplies one public random value per epoch, its **beacon**.
The retained legacy draws use it; `05` T79-P3 proposes the answer-review draw's binding.
It must be fixed only after the candidates are, and
nobody may be able to choose it. A value computed from the log, as the checkpoint head was
(`01` D29), fails the second condition: whoever orders or includes the last deposits
computes the head of every variant and keeps the one they like. The beacon is therefore
made by the consortium members, apart from the state they sign, by **commit-reveal**
(`01` D41). The round of epoch `e`, on network `N`, with the ordered member set `M` (the
keys the checkpoints' member-set hash commits to) and threshold `t`:

1. **Secret.** Member `i` derives its secret for the epoch from the seed `kᵢ` of its
   signing key, `sᵢ = H("isegoria/beacon/secret/v1" ‖ N ‖ M ‖ e ‖ kᵢ)`, as RFC 8032
   derives a signature's nonce: unpredictable without the key, one per epoch, and
   nothing to keep between commit and reveal.
2. **Commit.** `cᵢ = H("isegoria/beacon/commitment/v1" ‖ N ‖ M ‖ e ‖ pkᵢ ‖ sᵢ)`: the
   commitment binds the member's own key, so a copied commitment opens for nobody else
   (as a review commitment binds its reviewer, INV-12), and the round, so it cannot be
   replayed into another. The member signs
   `H("isegoria/beacon/commit/v1" ‖ N ‖ M ‖ e ‖ cᵢ)` with its consortium key and publishes
   `(N, M, e, i, cᵢ, σᵢ)`. A commit counts
   when `N`, `M` and `e` are the round's, `i` is a member, the signature verifies under
   `pkᵢ` and it is `i`'s first commit of the round.
3. **The commit set is fixed while the deposits are open.** At the commit deadline the
   commit set — the counted commits, in member order — is appended to the log as one
   record, and the first checkpoint covering it (the *commit checkpoint*) fixes it: a
   commit after it is refused. The commit checkpoint comes no later than the checkpoint
   that closes the deposit window of `e`. A member signs it only if the record lists its
   own commit, so under the consortium's assumption (fewer than `t` members collude) every
   threshold-signed commit set holds an honest member's commit, whoever publishes the log.
4. **Reveal after the deposits close.** Once the deposit checkpoint of `e` is signed, each
   member publishes `(i, sᵢ)`. A reveal counts when `i` has a commit in the set and `sᵢ`
   opens it; the commitment authenticates it, no signature is needed. A reveal before the
   deposit checkpoint, one that does not open its commit, and one from a member without a
   commit are refused.
5. **The value.** At the reveal deadline, if at least `t` members revealed, the beacon is
   `B = H("isegoria/beacon/value/v1" ‖ N ‖ M ‖ e ‖ s₀ ‖ … ‖ sₙ₋₁)`, one field per member in
   member order, empty for a member without a counted reveal (every field is
   length-prefixed) — a function of the set of reveals, not of the order they arrived in,
   and of nothing on the log after the commit set. Every draw of `e` seeds from it,
   domain-separated by purpose and index (`H("isegoria/beacon/v2" ‖ B ‖ purpose ‖ index)`).
6. **A member that does not reveal.** A member with a commit in the set and no counted
   reveal at the deadline has *withheld*. The round's outcome — `B`, who revealed, who
   withheld — is appended to the log: a public, permanent record, which the governance of
   the consortium's composition reads (`05`). The withholder is excluded from the signing
   set for the epoch: the checkpoints that publish the epoch's draws and results count no
   signature of it. No money is at stake (invariant 3): the penalty is the exclusion and
   the record.
7. **Fewer than `t` reveals.** No beacon for `e`. The legacy draw policy makes deposits
   and pending draws wait for epoch `e + 1`, whose round starts from new secrets (a retry
   of `e` would reuse already revealed secrets). For the answer review `05` T79-P4
   proposes the same wait, explicitly; the offline contract has no missing-beacon event
   or automatic retry.
   The withholders are recorded as in 6. Requiring `t` reveals is what makes the
   value unpredictable under the consortium's own assumption: with fewer than `t` colluding
   members, any `t` reveals hold an honest secret.

Rules 1, 3 (the member's refusal to sign), 5 (member order) and 7 are decided here; D41
left them open.

**Residual bias (accepted, D41).** The last member to reveal sees every other reveal and
can compute the beacon with and without its own: withholding picks the other value, once
per epoch, at the cost of a public non-reveal. Against a one-shot chance of 9/1000 of
landing on a target's panel, this at most doubles it, to about 1.8% (`1 − 0.991²`). `k`
colluding last revealers choose among up to `2^k` values while `t` still reveal, each of
them recorded. A coalition of `n − t + 1` members can stop the beacon (rule 7): a liveness
attack, public, which also lets it choose between this epoch's value and the next one's. A
unique threshold signature over the epoch number (drand-style) removes both once the
committees have a real distributed key generation (`10` T19); before it, whoever deals the
key could predict every draw.

The round's rules are `network::beacon` (`10` T37) and the draws `protocol::randomness`;
commits and reveals travel between nodes as members' objects and their deadlines are cuts
(§Cuts, `10` T74). On cuts, rule 6 applies to the cuts that follow the round's end within
its epoch (§The beacon on cuts, `10` T76).

---

## A node's own disk: surviving a restart

Before any of the above can hold, a single node must survive its own restart. The **log
is the source of truth**, and the node keeps two append-only files; everything else a node
holds — nullifier sets, quotas, reputation histories, pools — is a deterministic function
of the log's contents, rebuilt by replaying it (`10` T73), never saved as a second copy that
could drift from it.

- **`log`** — a 16-byte header, `isegoria-log-v1` and a newline, then one 104-byte record
  per entry: `seq` (8 bytes, little-endian), `prev`, `payload` (the CID) and `hash`
  (32 bytes each), the entry exactly as the log chains it.
- **`objects`** — a 16-byte header, `isegoria-obj-v1` and a newline, then one record per
  object: its length (8 bytes, little-endian, at most 16 MiB) and its bytes. The store is
  content-addressed: an object's key is the CID of its bytes, recomputed when the file is
  opened and again when an object is read, so the file cannot hand back other bytes under
  a CID.

**Write order.** An append is written in full and synced to the disk before it is
acknowledged and before the node's memory changes; an object is stored before the log
entry that names it. So an acknowledged entry is on the disk, and an entry never names an
object a crash lost.

**Recovery.** Opening the files checks all of them:

1. A header that is not the expected one refuses the file: it is another file or another
   version. A file shorter than the header whose bytes begin it is a creation a crash tore:
   the header is written again.
2. Every log record must chain from the start — `seq` its position, `prev` the hash before
   it, `hash` recomputed. A tail shorter than a record, or a final record that fails the
   check, is a write a crash tore before it was acknowledged: it is cut off and the cut is
   reported. A record that fails *before* the last refuses the file: a crash does not
   produce it, and dropping acknowledged history quietly would hide the damage. A writer
   then sets the file aside, kept for inspection, and takes its history back from the
   replicas (§A writer's own feed).
3. In `objects`, a final record that declares more bytes than the file still holds is torn
   and cut, reported the same way; a declared length above the bound refuses the file
   before anything is sized from it. A torn object whose length survived is only an
   orphan under another CID: the log entry it was written for was never written.

The reopened log is, entry for entry, the log that was acknowledged: its head, its
`contains` and its consistency with a signed checkpoint are those of the log before the
restart.

In code: `network::store` (`10` T13).

### Events and replay

The event catalogue below is the retained legacy `NodeEvent` path. The offline
answer review has its own `AnswerEvent` and durable framing (`05`); `05` §Distributed
integration proposes their integration. This catalogue is not an instruction to restore
legacy filters, results or lifecycle steps to the answer review.

Every change to a node's state is an **event**: an object in `objects`, named by one log
entry. The node's state is what replaying its log produces, event by event, from nothing —
on this node after a restart, or on any node given the same files and the issuer's public
key. Replay re-checks everything the first acceptance checked, proofs included: the log is
trusted to say *what* happened, never that it was valid.

**Encoding (ours, `network::codec`).** Little-endian fixed-width integers; a variable field
is its length (8 bytes) and its bytes, and a length above what the input still holds is
refused before anything is allocated; a decoder must consume the whole input. An event is a
version byte (1), a kind byte, then its fields:

| Kind | Event | Fields | State it changes |
|---|---|---|---|
| 1 | deposit | epoch, quota, the draft's item and primary source, the `Propose` proof | the drafts on record, the proposer's quota for the epoch (`05` [2]) |
| 2 | reviewer admitted | item CID, epoch, the `Judge` proof | the item's panel for the epoch |
| 3 | respondent admitted | batch CID, epoch, the `Respond` proof | the batch's respondents for the epoch |
| 4 | lifecycle step | the item's CID, then one lifecycle event: its number (1 `Admit` … 16 `ExposureLimit`, in the order of `lifecycle::Event`) and its fields | the item's state (`08` §9.1) |
| 5 | epoch results | epoch, the Merkle root of the epoch's inputs, then the records of the engine's outputs (below) | reviewer tracks, author histories and appeal escrows, exposure, residual histories, the contested pool |

A boolean is one byte, 0 or 1; a probability the eight bytes of its IEEE 754 bits; a panel
its nyms as one field whose length is a multiple of 32; a gate outcome one byte (0 pass,
1 supplementary review, 2 appeal-eligible, 3 reject); a re-check reading one byte (0 no DIF
detected, 1 DIF, 2 indeterminate: 0 and 1 are the bytes of the boolean the field held before
`15` A4, so earlier logs replay unchanged); a stage-1 screen reading one byte (0 dropped,
1 kept, 2 indeterminate: 0 and 1 are the bytes of the boolean `passed` before `15` A11, so
earlier logs replay unchanged, and a decoder from before A11 refuses byte 2); a stage-2
reading one byte in the order of the boolean `passed` it replaced, unlike the re-check's
(0 DIF, 1 no DIF detected, 2 indeterminate: earlier logs replay unchanged, and a decoder from
before `15` A4's residual (a) refuses byte 2). A deposit
the node accepts starts its item in `Deposited` and records the author id its proof proved; a
lifecycle step applies `lifecycle::step` to the item's state and is
refused for an unknown item, for an assignment naming another item than the one it moves,
and wherever the §9.1 table refuses it. An `Appeal` step is always refused
(`Rejection::AppealStep`): an appeal starts only with its filing record, below. Its encoding
(number 9) still decodes, so a log holding one is refused on replay. A step out of `Pilot1`
or `Pilot2` into either pool, `Rejected(Screen)` or `Rejected(Dif)` records the item's pilot
conclusion and whether it was appealed (`02` §C.1).

A nullifier proof travels as its role (one byte: 0 propose, 1 judge, 2 respond), the
nullifier and the Schnorr commitment (compressed G1 points, 48 bytes each) and the BBS+
proof (the library's canonical compressed encoding); decoding validates every point and
refuses trailing bytes.

**Writing.** A submitted event is checked and applied to the state in memory; only an
accepted event is written — the object, then the log entry, each synced. A rejected event
leaves no trace. If writing fails after the state changed, the node refuses every further
event until it is reopened, which rebuilds the state from what the disk holds.

**Replay.** Opening a node replays its log: an entry whose object is missing, does not
decode, or is rejected on replay refuses the node — the files say an accepted event
happened that this node cannot accept, and quietly skipping it would give another state.

**Results and their inputs.** The engine's outputs for an epoch enter the node as one
results event, whose records are, by number: 1 a reviewer's observed score (reviewer,
score, inclusion probability: `SkillTrack::record_observed`), 2 a reviewer's unobserved item,
6 exposure (item, administrations), 7 a residual (reviewer, the item's global id, `r − r̂`), 8 a contested fit
(each class's share, ability mean and item parameters, each item's guessing floor, and
the classes' ability histogram, its nodes and weights — `ClassCurves::with_ability`
recomputes the curves from them — and the members with their index in the fit:
`ContestedPool::record`),
9 a contested fact removed, 10 an item's pilot outcome (author, item, passed:
`AuthorHistory::conclude`), 12 an appeal settled (author, item, passed: `settle`), 13 an
appeal started (author, item, and the request's window reading, one byte: `file_appeal`
escrows a zero, and the node moves the item to `Pilot1 { appealed: true }` in the same event).
Retired, so that a log holding one no longer decodes: 3 to 5, the author records that named no
item (none was produced outside tests), and 11, a filing that left its appeal to a later step
(produced only offline, by `5b1ff4c` and `322dd52`; `15` C5–C6). Its bytes are not read as a
13. A results event's epoch is its data/root identifier only; author recording and
assessment use the applying cut's `(epoch, closing count)` (`02` §C.1, `15` C6).
Reviewers and authors are their proven ids (INV-9); the CUSUM parameters and author prior
are unchanged. The initial ledger uses the adopted provisional H = 45 closing configuration;
custom offline nodes pin their H in the versioned log header (`14`). The event is applied
whole or not at all, and refused for a second results event of the same epoch, an inclusion outside `(0, 1]`, a value that is not finite, a second
author record for one item, an author record without applying context, a second accepted
filing event for the same recording epoch, a filing whose window reading is false, or one the
author's `C_a` does not cover once the event's earlier filings are counted, a settlement of an escrow that is not open, and a contested fit whose classes or ability
describe no fit, whose members repeat or fall outside it, or that lies outside the pool's
resource domain (`15` D3). A contested fit with no members passes the same checks and records
nothing. The node also checks each author record against its own log. The author must be the
id the item's accepted deposit proved. An outcome or settlement needs the pilot conclusion the
item's step reached, with that outcome and appeal flag (`node::Conclusion`). A filing needs an
item that is appeal-eligible, and the accepted event moves every filed item to
`Pilot1 { appealed: true }` with its escrow; a refused event moves none (`NodeState::apply`).
`ResultsState::apply` alone records escrows only: it is the piece the node composes, not a
path that starts an appeal. Replay checks the recorded filings in sequence and refuses the
whole event at an invalid one; which of an epoch's requests to record is selected beforehand,
in CID order, a request's window read before its coverage (`authorship::author_records`, `02`
§C.1). The requests themselves are not on the log, so replay cannot check that selection. The
window reading is the producer's explicit input: the node checks the value the record
carries, not the window, whose duration and authentication are undecided.

**Offline node framing (C6, closed on the offline candidate `825cabc`; `15` C6).**
The durable node log now stores `OfflineRecord`: magic `ISEGORIA-C6\0`, then tag 0 and H
for the first immutable configuration record; tag 1 and `(cut number, epoch, closes)` for
Begin; tag 2 and a length-prefixed canonical `NodeEvent` for Event; tag 3 for End. Every Event
needs an active frame; Begin and End enforce progression. A closing increments the count
only on End. Reopening a valid incomplete frame resumes it, and another Begin is refused
until it ends; malformed or repeated boundaries fail. A mismatched H is refused before
replay. Old raw-event node logs do not decode as this format, with no automatic migration.
`Node::log_len` counts accepted protocol events, excluding these metadata records.
This is declared context, with no cut signatures; it is not authenticated ledger history.
Ledger feeds still carry only `NodeEvent` or member objects: offline metadata cannot set
ledger time or configuration. The ledger brackets its events with the verified cut itself.

The item–author link these records carry is the one the deposit already logs: the draft gives
the CID, and the `Propose` proof's id is the author's role pseudonym, the same for every
context. The author records add no join a reader of the log lacks. They restate it, with the
outcome, in the results event, so whatever T77 decides for deposits must cover them too.

The node applies these results as written: it does not recompute them, and the inputs —
every reviewer's ratings and every respondent's answers — are not in the event, since they
are the voting patterns that must not be published (`08` PRIV-004). The event binds the results
to their inputs instead: it carries the RFC 6962 Merkle root (`network::merkle`) of one leaf
per input — a rating (the judge's id, the item, the probability's bits) or an answer (the
respondent's id, the batch, the item's index in it, the answer) — the leaves sorted, so the
root is a function of the set of inputs. Whoever holds the inputs recomputes the engine and
the root and checks both; a reviewer or respondent holding its own leaf and an inclusion
proof checks that its input was counted without receiving anyone else's in the clear, though
this API gives no general guarantee that the others cannot be inferred (`08` PRIV-004.1).

**The ratings are on the log anyway.** A reveal is a lifecycle step (kind 4, event 5) that
carries the judge's nym, its probability and its nonce, so the log holds every rating of a
first panel or an extra round joined to its judge's nym, and since replication every
replica holds them, relays included, for any peer to read (§Who reads). The results event
keeps the engine's inputs off the log; the reveals put the ratings on it, against `01`
D17. `01` D54 keeps every lifecycle step, the reveals included, in the members' circle, and
publishes the inputs' root with the state tree's (`10` T77, T86).

With the four kinds before it, this is a node's whole protocol state (`10` T73); the
issuing committee's registries are the committee's, not a node's.

## Replication between nodes

A node's log is its own; replication makes every infrastructure node hold the same **set**
of signed entries, whatever order they arrive in, over whatever path (`10` T18).

**Who writes.** Only infrastructure nodes sign entries, whose ed25519 keys form the
network's **writer set** (configuration). A person never signs an entry: one key under a
person's deposits and reviews would link its author and evaluator pseudonyms (`00`
invariant 4). The event carries its own nullifier proof, and the writer's signature says
only that this writer logged it. A tiered network's writer set is the members' keys only
(§Who reads, T77); a network without tiers also lists relays, to which persons hand their
events. `01` D54 decides that only the members write: a person sends its event encrypted
to a member through a **blind relay** (§Blind relays, `10` T85).

**Who reads (`01` D54).** Every kind of object belongs to one tier:

| Tier | Who reads | Objects |
|---|---|---|
| public | anyone, light nodes and forks included | cuts and their signatures; the beacon's commits, reveals and outcomes; anchors; each epoch's roots (its inputs, the state tree); each decided item's outcome by content id; retired items, if governance releases them |
| circle | the consortium members | deposits (the drafts), reviewer and respondent admissions, panels, commits, reveals, every lifecycle step, the engine's inputs, the results' records per nym |
| self | one nym, for its own records | its leaf of the state tree and the path proving it |

An item admitted to the pool stays in the circle: it is administered, and publishing it is
the leakage `05` [9] guards against. A member writes public objects on one feed and circle
objects on another, each under its own key; a cut marks both. A node serves a circle feed's
entries only to a peer whose transport key a member's key has bound by signature, and
anyone else learns of a circle feed its marks, never its entries (`10` T77).

The **state tree** is a sparse Merkle tree with one leaf per nym, holding a hiding
commitment to its records; its shape is public, and a nym cannot leave a bad record behind
by claiming to be new (`00` invariant 5). Its contract is §The state tree and the
fork (`10` T86).

**Who reads, as built.** A network configured with tiers enforces them in the transport
(`10` T77, done for tiered networks after review, 2026-10-22; AT-NET-22, AT-NET-23,
loopback evidence only); a network without tiers does not (below). The primitives are in `network`:

- `tier::Tiers` checks the tiered writer set: member `i`'s consortium key writes its public
  feed and a distinct circle key its circle feed, one each, no key twice and no other
  writer. A writer's member and tier come from this configuration, never from a peer.
- `Tiers::bind` signs, with a member's consortium key, a transport key bound to the network
  id, the member-set hash and the member's index; `Tiers::access` grants the circle only to
  the transport key presented with a binding that verifies, and only under the set that
  verified it: an `Access` carries that set's digest (network id, member set, threshold,
  circle keys), and any other set reads it as public (`Tiers::member`).
- `Tiers::serve` answers Summary, Have and Entries with readable entries only: a reader
  without circle access gets no circle identifier or object, whatever its Summary or Want
  names. The cuts it collects still mark the circle feeds (length and head).
- `cut::observed` and `observer::Observer` let such a reader follow the cuts. It checks
  each cut's signatures (rule 6's withholders not counted), number, epoch and every public
  chain, which it must hold: a missing public entry stops it, as it stops a member. Of a
  circle mark it checks what it sees: a known member's circle key, never dropped or
  shortened, the same head at an equal length. It neither holds nor fetches circle
  entries, so it cannot check that a longer circle mark extends the last, that its chain
  exists, or what its events do: the cut's signers attest those (`Observed::attested`).
  It applies the public entries in the members' order and counts beacon messages only on
  their member's public feed, so it reaches the members' beacon values and exclusions; it
  computes no protocol state. An observer is made for one set: other tiers, or a replica
  whose writer set is not theirs, are refused before anything changes. Members keep the
  full check (`added`).

In `p2p` and `node`, a tiered network applies them as §Transport (a network with tiers)
and §A writer's own feed specify: the binding exchanged on each connection and checked
against the peer's authenticated key, every sync message and every announcement limited to
what the peer reads, gossip forwarded only when wholly public, and a member's circle feed
caught up only from a member that granted it. A member collects the cuts with every chain
checked (`p2p::member::Collector`); a node without the circle follows them through an
`Observer` (`p2p::member::PublicCollector`), a later conflict still stopping it. Both
collectors are fixed to one consortium and network. The tests read the bytes an outsider
is sent, not what its client keeps: no circle identifier, entry, object or unmarked hash
reaches it by connection, periodic or explicit sync, gossip, forwarding or reconnection.

A network without tiers is unchanged: a node answers the Summary and Want of any peer that
connects and pulls from it on connect, and a node started without a writer key replicates
the whole set. So any peer reads, in the clear, what the set holds: the drafts under
review, the panels before their verdict, every reveal with its judge's nym and rating, the
batches of each respondent id, and the results' records per nym — against §Why not a
permissionless blockchain, `08` PRIV-P5 and PRIV-P6, and `01` D17. Such a network — every
configuration with relays as writers, until T85 — runs only among machines its operator
controls. Neither kind is yet checked across machines or operators (`10` Milestone 2).

**A signed entry** is a writer's log entry (`seq`, `prev`, `payload`, recomputed `hash`,
exactly as the log chains it) and the writer's signature over
`H("isegoria/feed/entry/v1" ‖ network id ‖ writer ‖ hash)`. On the wire: the writer's key
(32 bytes), `seq` (8), `prev` and `payload` (32 each), the signature (64). An entry travels
with its object, whose CID must be the payload. An entry is identified by
`(writer, seq, hash)`; another signature over the same entry is a duplicate.

**The replica** is the set of entries it accepted, each with its object. It refuses an
entry from a key outside the writer set, with a signature that does not verify
(`verify_strict`), or whose object is above 16 MiB or is not its payload. It keeps every
other entry, so it is a grow-only set and a function of what it received, not of the order.
From the set it reads:

- a writer's **feed**: its entries from `seq` 0 while each position holds one entry and
  chains from the one before (`prev` of `seq` 0 is zeros), stopping at the first gap, fork
  or break;
- **equivocations**: two entries of one writer at one `seq`. The pair, both signatures in
  it, is evidence anyone checks against the writer set; the writer's feed stops before it.
  A replica reports, per position, the two entries with the smallest hashes, so replicas
  holding the same set report the same pairs;
- a **digest** of the set, the hash of its sorted identifiers, equal on two replicas exactly
  when they hold the same set.

**Synchronisation (pull).** A node asks a peer for what it lacks, in two round trips:

1. **Summary → Have.** The node sends, per writer it holds, the number of its entries, the
   digest of them, its feed's length `p` and head. The peer answers with the identifiers of
   its entries of every writer whose digest differs (or that the node lacks), leaving out
   the first `p` entries of its own feed when its feed's `p`-th entry is the node's head:
   the chain commits the node to them.
2. **Want → Entries.** The node asks for the identifiers it lacks; the peer answers with
   those entries and their objects, in identifier order, as many as fit in 32 MiB and at
   least one; the rest are asked for again in the next round.

A sync round with a peer that holds more always brings at least one new entry, so repeated
rounds between two nodes end with both holding the union of their sets. **Gossip** (the
push) announces the identifier of every newly accepted entry to the node's neighbours; a
neighbour that lacks it asks the announcer for it with a Want. Gossip is the fast path;
the pull rounds are what guarantees convergence when gossip drops, reorders or duplicates a
message, or a partition heals.

**Convergence invariant** (`08` §10.3). Two replicas that accepted the same set of entries,
in any order, have the same digest, the same feeds and the same equivocations; two nodes
that complete a sync round in each direction with nothing in between hold the same set.

**Messages.** A version byte (1), a kind byte, the fields: 1 Summary — its records as one
field, 112 bytes each (writer, count, digest, feed length, head), sorted by writer, no
writer twice; 2 Have and 3 Want — identifiers as one field, 72 bytes each (writer, `seq`,
hash), strictly increasing; 4 Entries — the count, then each signed entry and its object as
a field, in strictly increasing identifier order. Anything else is refused, so each message
has one encoding.

**A writer's own feed** is its log on its own disk (§A node's own disk): to publish, it
stores the object, appends the entry, then signs it. A restarted writer replays its log and
signs every entry again — ed25519 signatures are deterministic, so they are the same. Its
files may be lost, damaged or older than its feed, and signing a `seq` its feed already
holds is an equivocation, so (`10` T80):

1. **Its files.** A log or object file that is damaged — a record that fails before the
   last, a length above the bound, an entry whose object is missing or altered — is set
   aside with its companion, renamed `<name>.damaged-<n>`, and the writer starts from empty
   files. A file with a foreign header still refuses the node.
2. **No write before a peer.** After a start, the writer writes nothing — its member
   duties included — until a sync round with a peer has ended in which the peer offered
   no entry of its key that it lacks.
3. **Its feed taken back.** Before writing, it appends to its own files the entries its key
   signed that its replica holds past its log, each chaining on the one before
   (`Replica::feed_beyond`); its own signature vouches for them.
4. **A fork or a gap stops it.** If the replica holds an entry of its key that its log
   contradicts or that does not chain, or one past a missing `seq`, it writes nothing; a gap
   clears when the missing entry arrives, a fork does not. A stored replica that contradicts
   the log refuses the node.

The writer catches up with the first peer that answers: a peer reached later that holds a
longer feed finds the equivocation as evidence, it does not prevent it. A node whose writer
key is not in the writer set does not start.

A member of a tiered network keeps **two feeds**, each with its own files and catch-up
(`own/` the public feed under its consortium key, `circle/` the circle feed under its
circle key). Rule 2 holds for the public feed after a sync round with any peer; for the
circle feed only after one asked of a member whose binding verified and which granted this
node the circle before the request, and answered while that same grant holds. A grant is
withdrawn by a hello or reply whose binding does not verify, by a reply refusing the circle
and by the last connection closing; one given again after is a new grant, and the answer to
a request asked under the old one proves nothing. A peer that serves the public tier alone
proves nothing about the circle feed, which writes nothing until then. Like the public
feed's, this catch-up is only as complete as the answering member's history: an entry of
this feed it never received, or lost, stays undiscovered, and the next write equivocates
against it (T80's residual; T88-P4 proposes more peers).

**Transport.** libp2p 0.56 over TCP, with Noise and Yamux: gossipsub (signed, strict
validation) carries the announcements, at most 512 identifiers per message, on the topic
`isegoria/<network id in hex>/entries`; request-response (`/isegoria/sync/1`, at most
64 MiB a message) carries Summary→Have and Want→Entries. A node pulls from a peer when they
connect, on a fixed period, and again after an Entries response that brought something new.
A node's libp2p key is its transport identity, distinct from its writer key. A message that
does not decode is dropped.

**A network with tiers (`10` T77).** When the configuration holds a tiered writer set
(`network::tier::Tiers`), the transport serves each peer only what its access reads:

- **The binding.** A member node binds its libp2p ed25519 public key with its consortium
  key (`Tiers::bind`, §Who reads). It travels as version 1, the member's index (8 bytes),
  the transport key (32) and the signature (64), 105 bytes, one encoding. On each new
  connection both sides send `/isegoria/bind/1` a hello holding their binding or none, and
  answer one with whether they granted the circle and their own binding. A binding is
  checked against the ed25519 key the peer's id inlines, which Noise authenticated, never
  against a key a message names; before it verifies, and for another transport key,
  network or member set, the peer reads the public tier only. The access ends when the last
  connection to that peer closes, and is checked again on the next; for the grant a member
  gives this node, see §A writer's own feed.
- **Every message.** A Summary, Have, Want or Entries sent to a peer names only writers
  its access reads, whatever it asks (`Tiers::serve`); from a peer, a node takes only
  entries of writers that peer may read.
- **Announcements.** Gossip carries public identifiers only, and a node forwards a gossip
  message only if every identifier in it is public (gossipsub validation), so an
  intermediary outside the circle relays nothing of it. A new circle entry is announced to
  each bound peer directly, as a Have request answered by an empty Have; the peer then asks
  for what it lacks. A topic is not access control: nothing circle goes on gossip.

A network without tiers keeps the transport above unchanged, and so the defect of §Who
reads, as built.

**A replica on disk.** A node given a directory keeps its replica in two content-addressed
object files (§A node's own disk): `objects`, the entries' objects, and `entries`, each
accepted signed entry's 168-byte encoding. An entry is written after its object and before
the node's memory changes; opening re-inserts every stored entry, checked again against the
writer set, so a restarted node holds the set it had.

### Blind relays (T85)

**Status (2026-10-22).** The first increment, the transport channel, is built on T85-P1–P5
at the owner's instruction, P5 as stated below, and reviewed (§As built). Which messages a person sends —
the events, the read requests, their replies — stays `05` T79-P2 and T79-P7, proposed; T85
as a whole stays open with them.

*Decided (D54, `10` T85).* A person sends each event sealed to one member's key through a
relay, as Oblivious HTTP (RFC 9458) over HPKE (RFC 9180): the relay sees where a message
comes from and not what it says, the member what it says and not where it comes from.
Relays do not write; the writer set is the members' keys and a person signs nothing. Any
member may log a person's event, the first in a cut counting (PROTO-015). The members' HPKE
keys belong to the network's configuration, and an audited HPKE library is preferred. A
relay and a member that collude join an address to its content, so relays are run by
operators other than the members.

**The path.** The person encapsulates a request to a member's key configuration (RFC 9458
§4.3) and sends it to a relay; the relay forwards the bytes unchanged to the member that key
names and returns its encapsulated response unchanged (§4.4); the member decapsulates,
checks the request, logs the event on its circle feed (`Handle::publish_circle`) and answers
with the entry or an error. The event then replicates and is cut like any circle entry.

| | sees | does not see |
|---|---|---|
| person | its event and the reply; the relay's address; the key configurations | — |
| relay | the person's IP address, port and transport key; times and sizes; the member, by the key id in the clear header; the gateway's errors in the clear | the event, the reply, any nym |
| member | the event, the nym proofs in it, the reply; the relay's address and key; times and sizes | the person's address |
| others on the path | encrypted flows, sizes and times | content |
| anyone, from the public cut marks | that a member's circle feed grew in a cut | which person or event |

The last line is D54's residual: timing links a request a relay saw to a circle feed's
growth. Batching (`10` T28) and PRIV-003 would blur it; neither is built.

- **T85-P1 — the carrier.** RFC 9458's encapsulated request and response, byte for byte,
  on two libp2p request-response protocols, `/isegoria/ohttp/relay/1` (person → relay) and
  `/isegoria/ohttp/gateway/1` (relay → member), so an HTTP carrier could replace them with
  the cryptography unchanged. Inside, a Binary HTTP request (RFC 9292, known length)
  `POST isegoria:///isegoria/v1/<network id in hex>/event` whose body is the event, and a
  Binary HTTP response: RFC 9458's labels, no bespoke HPKE composition. A person uses a new
  libp2p key for each request, so its transport key links nothing; its address still can.
- **T85-P2 — the dependencies.** `ohttp` 0.8.0 and `bhttp` 0.8.0, pinned with `=`, from
  github.com/martinthomson/ohttp (Martin Thomson co-authored RFC 9458), MIT or Apache-2.0,
  Rust 1.86;
  `ohttp`'s `rust-hpke` backend, the `hpke` crate 0.14.1 (github.com/rozbb/rust-hpke),
  whose README states that nobody has performed a paid audit of it. OSV listed no advisory
  for `ohttp`, `bhttp` or `hpke` on 2026-10-22. Three of their dependencies are held at
  their newest Rust 1.86 releases in `Cargo.lock`: `aes` 0.9.2, `sha2` 0.11.0, `ctutils`
  0.4.2. RFC 9458 Appendix A is the known-answer test (§As built).
- **T85-P3 — the members' keys.** An HPKE key pair per member — X25519, HKDF-SHA256,
  AES-128-GCM, key id the member's index — from 32 bytes kept in `hpke_key_file`, which
  HPKE's own `DeriveKeyPair` (RFC 9180 §7.1.3) turns into the pair; no other derivation
  (T88-P2's is not adopted) and no rotation in the trial. Each key configuration (RFC 9458
  §3) is signed by its member's consortium key over
  `H("isegoria/ohttp/key/v1" ‖ network id ‖ member-set hash ‖ index ‖ key configuration)`.
  A client checks a configuration against the consortium of the network configuration it
  already holds; wherever the configuration came from, a relay is never its authority, nor
  the consortium's. A relay offering its own key therefore reads nothing.
- **T85-P4 — the relays.** The configuration lists the relays' transport keys (`relays`).
  A gateway answers only a peer whose Noise-authenticated key is listed, before
  decapsulating: a member never receives a request from the person's own address. A relay
  is outside the circle (T77 serves it the public tier) and holds no feed: a relay node
  with a member role, its own feed or a circle feed — under a member's key or any other —
  is refused before any file is opened, and every replica refuses an entry its key signs. Refusing the untiered
  configurations, where relays still write, once this path works is a separate decision.
- **T85-P5 — messages.**
  - *Limits.* An encapsulated request at most 1 MiB and a reply 64 KiB, counting the
    encapsulation alone: a reply's `Carried` envelope adds 10 bytes (version, kind, the
    field's length), read with it. A longer one is refused as it is read, before more is
    held, by relay, gateway and client alike; the client refuses an oversized sealed reply
    however it received it.
  - *Refusals in the clear* (the relay's or the gateway's, forwarded unchanged): a peer that
    is not a listed relay, a key id no member's or not the gateway's, a request that does
    not decapsulate, no connection to the member. Nothing is logged.
  - *Sealed answers*: 200 with a receipt; 421 a path naming another network; 400 not this
    request; 403 bytes the admission check refuses; 503 "retry later" while the member's
    circle feed is not caught up; 500 a write that failed. Admission is a predicate the
    embedding supplies — `p2p` keeps no protocol state, and which events it admits is T79.
  - *The receipt* names the circle entry `(writer, seq, hash)` that logged the bytes. It
    attests that registration only: not that the protocol applied or accepted the event,
    which cuts and the ledger decide later (a duplicate refused, the first wins).
  - *Duplicates.* OHTTP does not stop a replay (RFC 9458 §6.5). A gateway answers bytes it
    already logged on its circle feed with that entry's receipt and logs nothing: whether the
    requests come together (its node takes them one at a time), after a restart (its circle
    feed is on disk) or in a new encapsulation of the same bytes (they have one content id).
    "The same answer" is the same result, sealed in the current request's context, never a
    stored ciphertext. Through two members one event makes two entries, the ledger counting
    the first.

**T79-P7.** Whether a read request's proof binds the request's encapsulation (`enc`) is left
to T79-P7; this channel does not change OHTTP to provide it.

**First increment: acceptance.** Over loopback, with synthetic events and an instrumented
relay: the RFC 9458 vector; the whole path to the circle feed; nothing of the event, the
path or the receipt in what the relay receives or sends; the gateway answering only listed
relays; any byte changed in a request or a reply refused; another member's key, another
network's path and a configuration signed for another network or by another key refused;
the limits; duplicates concurrent, re-encapsulated and after a restart; "retry later"
before the circle feed caught up; a relay that cannot write; and replies swapped between two
requests, which each client refuses.

**As built (2026-10-22, first increment).** `p2p::blind` holds the channel: `KeyAnnouncement`
(a member's signed key configuration, `verify` under the client's tiers), `GatewayKey`,
`Gateway` with the embedding's `Admission`, `Carried` and `Refusal`, `Answer`, `seal` and
`Sealed::open`, and `submit`, a person's request from a new transport key. A tiered relay
(`Config::relay`) forwards a request unchanged, by its key id, to the peer whose binding
verified as that member's (§Transport); a member's gateway (`Config::gateway`) checks, in
order, the listed relay, the key id, the decapsulation, the path, admission and its circle
feed's catch-up, finds bytes already logged by their content id on its circle log, and
otherwise publishes them there. The codec refuses a request or reply beyond its limit as it
reads it, and an empty reply is a failed exchange. In `node`: `consortium.hpke`,
`hpke_key_file`, `relays`, `relay`, and `isegoria-node --announce <config>` printing a member's
signed key configuration. **What a node process can do:** forward as a relay, and terminate
the channel as a member, answering every event "not admitted" and logging nothing — no
admission rule exists before T79, and the legacy events are not admitted in its place.
Persons have no client program: `seal` and `submit` are a library.

Three tests check the OHTTP the channel uses against a reference HPKE path, the `hpke`
crate driven directly and only in tests (RFC 9458 §4.3–4.4): Appendix A exactly, through
that path (the open, the exported secret, the key, the nonce and the reply) and through
`ohttp` and `bhttp` decoding its key configuration, request header, length and Binary HTTP
messages; the client, the reference path opening what `seal` sends and `Sealed::open`
reading a reply the reference path seals; the gateway, opening a request the reference path
seals and sealing its answer so the reference path opens it. Appendix A's illustrative
`Content-Length` values (78, 38) differ from its hex (80 and 35 bytes), the second reported
as erratum 7781; `bhttp` writes the empty sections RFC 9292 lets the example's reply omit.
Tests: `p2p/tests/blind_relays.rs` and `node/tests/config.rs` (AT-NET-24); before the node
handled the two roles, the seven path tests failed. Not built: read requests and private
replies (T79-P7), the answer review's events (T79), any limit on a relay's volume, fuzzing of
the new decoders, and a run across machines; untiered networks keep relays as writers.

### The state tree and the fork (T86)

**Contract approved (2026-10-24, D56): T86-P1–P8.** The identifiers below are retained
for review traceability. D56 accepts P6's public residual, P7's frozen-leaf exception to
D54, and P8's pause of enrollment and issuance without continuity. Which records a nym's
leaf holds in the answer review is T79's (`05` §Distributed integration).

*Decided (D54, `10` T86).* A sparse Merkle tree with one leaf per nym at a position its nym
fixes, holding a hiding commitment to its records (records and salt). Its shape — which
positions hold a leaf, and the internal hashes — is public, in the archive (T84). Every live
leaf is re-salted each epoch (D56 excepts frozen leaves until reopening) from a key the nym
derives per role from its credential and hands the circle once, so the public tree does not
show which live nyms acted and the nym can open its leaf at any epoch. Each epoch publishes the tree's root with the inputs' root. A nym proves
its records by its path and their opening; a new nym proves its position empty. A fork with
a member keeps the circle's data, negative records included; one with none restarts from the
last anchored root, each nym opening its own leaf, and a nym with a record cannot enter as
new (invariant 5).

*Consequences.* The answer review's nym is `H(x·H_role)` (`identity::nullifier`): it depends
on the holder's credential secret and the role only, not on the network, the committee or
the cut. The existing Merkle tree (`network::merkle`, RFC 6962) proves inclusion in a list,
not a position's absence. Re-salting changes the live leaf hashes, not their occupied
positions; publishing a full shape each epoch grows the archive by the occupied leaves.

- **T86-P1 — positions.** *The problem:* the same person active in two networks has the same
  nym in both, so a position fixed by the nym alone links occupied positions across the two
  public trees. *Choice:* `H("isegoria/state/position/v1" ‖ lineage ‖ role ‖ nym)`, where the
  lineage is the network id of the history's first genesis, which every fork, and every fork
  of a fork, inherits: a nym keeps its position through forks, and unrelated networks place
  it elsewhere. Each role's nym has its own leaf, as unlinkable as the nyms.
- **T86-P2 — commitment and tree.** Every hash is `network`'s tagged SHA-256 (the domain and
  each field length-prefixed); roles in hashes use the UTF-8 tags `propose`, `judge`,
  `respond`. Wire versions and role tags are bytes (roles 0, 1, 2 respectively), counters
  and lengths are little-endian `u64`, and hashes are fixed 32-byte fields. A leaf commits
  to `C = H("isegoria/state/commit/v1" ‖ lineage ‖ network id ‖ epoch ‖ role ‖ salt ‖ records)`
  and the tree holds
  `H("isegoria/state/leaf/v1" ‖ position ‖ C)`. *Records* are version 1, the role, then the
  payload as a field; the payload stays opaque here and is T79's (for the answer review, the
  author's observations and slots, or a reviewer's pool entry). *The tree:* a position is 256
  bits read from the most significant bit of its first byte; the node at depth `d` (the root
  at 0) leads to its left child where bit `d` is 0, its right where 1. Heights count from
  the leaves: `E₀ = H("isegoria/state/empty/v1")` is an empty leaf,
  `Eₕ₊₁ = H("isegoria/state/node/v1" ‖ Eₕ ‖ Eₕ)` an empty subtree of height `h + 1`, a node
  `H("isegoria/state/node/v1" ‖ left ‖ right)`, and the root has height 256.
- **T86-P3 — salts.** The holder derives `k = H("isegoria/state/salt-key/v1" ‖ secret ‖ role)`
  from the canonical compressed bytes of its credential secret scalar (bespoke, as the nym's
  own derivation), and a salt is
  `H("isegoria/state/salt/v1" ‖ k ‖ lineage ‖ network id ‖ epoch)`. The network id in salt
  and commitment separates forks, while the lineage keeps the position. *E.g.* two forks of
  one parent, both at epoch 12, the same nym with the same `k` and the same records: without
  the network id their leaves would be equal — showing the records match across the forks
  and letting an opening made in one serve in the other; with it they differ, at the same
  position. *The problem:* "hands the circle once" has no moment: a reviewer that joins the
  pool without handing `k` has a leaf the members cannot re-salt. *Choice:* `k` travels
  sealed with the role's first event (T85; its message is T79's), and a member does not log
  a first event without it.
- **T86-P4 — roots and the shape.** The shape lists the occupied leaves by strictly
  increasing position, each once: its position, its leaf hash and its origin — 0 for a live
  leaf, committed in the statement's own context, or 1 for a frozen one followed by the
  network id and epoch it was committed under (T86-P7). It is version 1, the count, then the
  entries; the root follows from it alone. After epoch `e`'s closing cut, every member
  computes the tree and signs `H("isegoria/state/roots/v1" ‖ network id ‖ lineage ‖
  member-set hash ‖ e ‖ cut number ‖ cut digest ‖ inputs root ‖ state root ‖ shape digest)`,
  the shape digest being `H("isegoria/state/shape/v1" ‖ shape)`, as a member object collected
  at `t` like a cut and anchored by T17. A member that computes another root does not sign.
  The inputs root of the answer review is T79's.
- **T86-P5 — proofs, and what they do not prove.** A path is version 1, the position, a
  256-bit bitmap read like a position, whose bit `d` says the sibling at depth `d + 1` — of
  height `255 − d` — is given, then the given siblings from the leaf up. A sibling not given
  is `E₂₅₅₋d`, and a given one equal to it is not an encoding. Verifying folds from the leaf
  hash for `d` from 255 to 0, the running hash on the left where the position's bit `d` is 0.
  An opening adds the records and the salt, under the stated lineage, network id, epoch and
  role; an absence proof is a path folding from `E₀`. The shape being public, anyone builds
  paths and absence proofs; only who holds records and salt opens a leaf — the nym (its
  secret gives the salt, its records it keeps or obtains through T79-P7) or a member. An
  absence proof is about a position: it keeps a nym from passing as new only when the
  verifier derives that position from the nym a valid nullifier proof has just
  authenticated. An opening shows that its presenter knows the leaf's contents, not that it
  is that leaf's nym: re-entry needs both, with the position derived by the verifier.
- **T86-P6 — what stays public (accepted residual).** Re-salting hides
  the records, their size and which live nyms acted. A position hashes the lineage, the role
  and the nym, so it shows the public neither the role nor the nym: the shape shows
  occupancy only. A member, who holds the circle's nyms, computes their positions and reads
  both, as does whoever learns a nym. Occupancy still tells which positions are occupied,
  how many, and which appear or freeze in an epoch. *E.g.* one new position in a quiet epoch
  tells everyone that one nym had its first record then; two new together hint at one
  person's two roles. D56 accepts this residual; the tree does not blur it.
- **T86-P7 — a fork, frozen leaves and their reopening.** A leaf's *context* is the lineage,
  the network id and epoch it was committed under, and its role. *With a member:* it brings
  the events and the salt keys; the fork replays them, reproduces the parent's last anchored
  root before signing its own, and re-salts every live leaf under its own network id. *With
  none:* the fork takes the last anchored statement, rebuilds the tree from that epoch's
  archived shape and checks both the state root and shape digest against that trusted
  statement; lacking the keys, it freezes every live leaf, its
  origin the parent's network id and that epoch. A *frozen* leaf keeps its hash and origin
  and is locked: its nym cannot act; there is no deadline and no default record. *The
  exception:* D54 re-salts every leaf each epoch; a frozen leaf is not, and stays visibly
  static. *Reopening:* the nym authenticates (T86-P5) and opens the frozen leaf under its
  origin's context, using the current tree's public path (an old path may be stale after
  another leaf changes); the fork adopts those records as the nym's current state, receives `k`
  again (T86-P3). Reopening immediately makes the working leaf live and re-commits its
  records under the fork's context; it cannot be consumed a second time before publication.
  The next published epoch re-salts it with the other live leaves. Intermediate working
  shapes are not epoch publications. Only the frozen leaf's own opening counts: an earlier epoch's opening does not fold to it, and once the
  leaf is live an opening under the origin is refused, so an old opening never replaces
  later records. *Fork of a fork:* the origin travels with the leaf. One still frozen when
  its fork forks again keeps its first origin and reopens under it, with a member or not;
  one live there is re-salted by a member or frozen with that fork's origin. A nym that lost
  its records cannot reopen; keeping them is its own part (D54's self tier).
- **T86-P8 — continuity of a nym.** The nym comes from the holder's secret and the role
  alone; what lets it act and keeps it single are three things of the issuing committee
  (`03`), not of the consortium: the *issuer keys* that sign credentials and verify the
  nullifier proofs, the *uniqueness function* — the OPRF key that turns an anchor into a
  label (`identity::enrollment`, `oprf`) — and the *registries* of the labels enrolled and
  issued. *Consortium members* changing, a fork included, touch none of them: the lineage
  keeps the positions, and the fork's configuration lists the parent's issuer keys — a nym
  whose issuer is not listed cannot authenticate, so it neither acts nor reopens. *The
  issuing committee* changing: a new issuer key keeps every nym while the networks still
  accept the old one; imported registries keep de-duplication only under the same uniqueness
  function. A new one computes other labels, so the registries recognize no one and a person
  could obtain a second credential: a fresh nym at an empty position, which no absence proof
  catches (invariant 5). D30 then requires the same function (shares refreshed, not
  re-keyed) or a migration that keeps de-duplication; with no committee left, existing
  credentials still verify, but nothing de-duplicates. *Choice:* where that continuity is
  missing, enrollment and issuance stop; existing nyms go on; a lost secret is a lost nym.
  Specifying the migration is T19 and T22's.

**Dependencies.** T79: the payloads, the inputs root, the first event carrying `k`, the reads
a nym asks for. T85: the sealed channel (its first increment, built). T17: anchoring the
signed statements. T84: the shapes in the archive. T19, T22: the issuing committee's
continuity. Nothing here needs the legacy path.

**First increment (2026-10-24).** In `network`, `state`: positions, salts from a given
key, commitments, the sparse tree with its empty hashes, the shape with live and frozen
leaves, inclusion, absence and opening proofs, the root statement, a rebuild from a shape,
and the reopening of a frozen leaf, each with one encoding; in `identity`, the holder's salt
key. The re-entry check takes an authenticated nym and derives its position. Opaque
synthetic payloads; no signatures collected, no T79. `CallerAuthenticatedNym` is an explicit
caller boundary: its constructor does not verify a credential. The embedding must first
verify the action's nullifier proof against an accepted issuer; it must also authenticate
the anchored `RootStatement`. `Shape::check` checks both commitments, not signatures or
anchor provenance. The in-memory `State` provides the working tree, not persistence or
an epoch publication service. Canonical codecs have property tests, not a fuzz campaign.
The bespoke SHA-256 composition has independent known-answer vectors, not a cryptographic
audit. Acceptance cases (`network/tests/state.rs`, `identity/tests/state_salt.rs`):

1. a nym's records open against the root, and any byte changed in the records, the salt, the
   path, the leaf or the root is refused;
2. an opening under another lineage, network id, epoch, role or position is refused;
3. the same lineage, epoch, key and records under two network ids give one position, two
   salts and two leaves, and an opening valid under one network id only;
4. an empty position proves its absence and an occupied one cannot; re-entry refuses an
   absence proof or an opening at a position other than the authenticated nym's, and an
   opening offered without an authenticated nym;
5. the next epoch changes every live leaf and the root with the records the same; a frozen
   leaf keeps its hash;
6. a frozen leaf reopens under its origin and becomes live; an earlier epoch's opening, or
   the origin's opening again after reopening (also before the next epoch), is refused and the
   later records stay; in a fork of a fork, a leaf frozen at the first fork reopens under its first origin, one frozen at the
   second under the second;
7. a tree rebuilt from its shape matches both the trusted state root and shape digest; an
   entry changed, missing or added changes the shape digest, and a changed position or leaf hash the root too; a shape with
   a repeated or out-of-order entry does not decode; after a rebuild some nyms reopen while
   the rest stay frozen;
8. the empty hashes, the bit order and the bitmap agree with the definitions above on a small
   tree computed by hand; a path giving an empty sibling, or with more or fewer siblings
   than its bitmap, does not decode, and one with two siblings swapped does not verify;
9. the root statement's digest changes with each of its fields, and paths, shapes and
   statements decode only as their one encoding.

### Cuts: the protocol state from the replicated set

Replicas converge on a set, but the protocol state is the result of applying events in an
order, and conflicting events (two deposits of one CID through two relays, one quota spent
on two relays, a second results event for an epoch) resolve by which comes first. The order
is fixed by **cuts** the consortium signs, never by arrival.

- **A cut** is its number (0, 1, 2, … with no gap), the epoch it belongs to, whether it
  **closes** that epoch's deposits, and, per writer it covers, a **mark**:
  the length of that writer's feed and the hash of its last entry — the chain the mark ends
  with is the one counted, even when the writer forked. The marks are sorted by writer, with
  no writer twice and no length 0. Its digest is
  `H("isegoria/cut/v1" ‖ number ‖ epoch ‖ closes ‖ marks)`; the consortium signs it as a checkpoint whose
  height is the number and whose head is the digest, so `t` of the `n` members must sign it
  and a member signing two cuts with one number is caught by the checkpoint rules (NET-006).
- **Extending.** Cut `k + 1` keeps every writer of cut `k`, at a length at least as long,
  and its chain passes through cut `k`'s head. A new writer may appear. Its epoch is cut
  `k`'s, or later; after a cut that closes its epoch, a later one. A cut declares its epoch:
  there is no shared clock, and the members who sign it accept it.
- **The order.** The entries a cut adds are, per writer, those past the previous cut's
  length. The writers are ranked by `H("isegoria/cut/order/v1" ‖ previous cut's digest ‖
  writer)` (the previous digest of cut 0 is zeros), and the entries taken one per writer in
  that rank, round after round, each writer's in `seq` order. So no writer key comes first
  in every cut, and each writer's own order is kept.
- **Applying.** A node applies a cut once it holds every entry it names; until then the cut
  waits and the state does not move. Each entry's object is decoded as an event (§Events and
  replay) and applied to the protocol state in that order. **The first wins**: an event the
  state refuses — a duplicate deposit, a spent quota, a second results event — is recorded as
  refused, with its reason, and the state is left as it was; so is an object that is not an
  event, and an event of a kind only the consortium may log (a lifecycle step, epoch
  results) from a writer outside it. Nothing refused stops the cut. Persons' events
  (deposits, reviewers and respondents admitted) may come from any writer. A deposit counts
  in the epoch of the cut that applies it: one naming another epoch is refused — after its
  epoch's closing cut, as late.
- **Stable.** A state computed from cuts `0…k` does not change when entries arrive late:
  they wait for cut `k + 1`. Two nodes holding the same cuts compute the same state, whatever
  order their replicas received the entries in.

**Members' objects.** The consortium's own messages travel as objects on the members'
feeds, one byte `0xC0` then a kind, so no event decodes as one: 1 a cut signature in view
0 (the cut's encoding, the member's index, its signature over the cut's checkpoint); 2 a
beacon commit (network, member-set hash, epoch, index, commitment, signature: §The epoch's
beacon); 3 a beacon reveal (epoch, index, secret); 4 a cut signature in a view `v ≥ 1` (the
cut's encoding, `v`, the index, the signature, and the members a proposal names, strictly
increasing, none for a co-signer); 5 a view change (the cut's number, the view `v ≥ 1`, the
index). An object counts only on the feed of the member its index names; on any other feed
it is refused as not authorized.

**Collecting a cut.** A node takes, for the next number, the cut that `t` distinct members
signed on their feeds **in one view**; signatures given in different views are not added
up. Two different cuts each signed by `t` members for one number, in any views, are the
consortium equivocating: the node stops there and holds both as evidence (NET-006).

**Proposing and signing.** Cut `k` is proposed in **views** 0, 1, 2, …: the proposer of view
`v` is member `(k + v) mod n`, who signs its proposal first; its epoch and closing are its
call. A member signs a cut for `k` in view `v` — over the cut's checkpoint in that view: the
height its number, the head its digest in view 0 and `H("isegoria/cut/view/v1" ‖ digest ‖ v)`
in a later view — when it is that view's proposal, the cut extends the last collected one,
the member holds every entry it names, the cut counts the member's own beacon messages (a
closing cut its commit for the epoch, the cut after it its reveal, if the member has
published them), the member has not signed for `k` in view `v` already, and it has not
**promised** a later view.

**Changing view (`10` T76).** A member that sees no cut `k` collected after a bounded wait
in its view publishes a **view change** for `(k, v + 1)`, and from then on signs nothing
for `k` in a view before `v + 1`: its promise. A member that sees `t − 1` other members
promised to view `w` or later moves to `w` the same way, so the members stay in step. The
proposer of a view `v ≥ 1` proposes once it holds the view changes for `(k, v)` of `t`
members, and names them in its proposal. Among the cuts those members had signed for `k`
before their view change — read on their own feeds, before that entry — it must propose
the one signed in the latest view (the smallest digest among several), and only if none of
them had signed any may it propose a new cut. A member checks the same before signing.

This is single-decree Paxos, the members its acceptors. If `t` members signed cut `C` in
view `v`, the `t` view changes behind any later proposal include one of them (`2t > n`),
which signed `C` before it promised: every later proposal is `C`. So while every member
follows these rules, two different cuts are never collected for one number; a member that
signs two cuts in one view is the evidence otherwise. With a threshold at most half the
members, two halves could each complete a different cut: there the members never leave
view 0, and a silent proposer still stalls the cuts. **Residual:** a cut that must be
proposed again and does not count a member's own commit or reveal, published after the cut
was built, is refused by that member, and where its signature is needed the number waits.

**The beacon on cuts.** The round of epoch `e` (§The epoch's beacon) runs on the cuts: a
commit counts when a cut of epoch `e` applies it, up to and including the cut that closes
`e` — which is the commit checkpoint and the deposit checkpoint at once; a reveal counts
when the cut right after the closing one applies it, and at the end of that cut the round
finishes: its outcome (the value if `t` revealed, who revealed, who withheld) is part of
the state. A commit or reveal in any other cut, or that the round refuses, is refused.

**The withholders on cuts (rule 6).** When the round of epoch `e` finishes with a value, at
cut `r`, its withholders count toward no cut after `r` in the epoch `r` belongs to — `e + 1`
— that epoch's closing cut included: the cuts that publish the draws `e`'s beacon seeds and
what follows from them (`10` T79 places them there). Collecting a cut and applying it count
no signature of theirs there. A round with no value seeds nothing and excludes no one, its
withholders recorded all the same (rule 7). A round with a value had `t` members revealing,
so the members left can always sign. The rounds are counted by one rule for the ledger and
for the members collecting cuts (`network::rounds`).

Public CID lookup without a full replica is specified in §Public object discovery (T75);
circle objects are never offered through it (§Who reads). The node process
does not run
the answer review between nodes (`10` T79); its proposed contract is
`05` §Distributed integration. In code: `network::replica`,
`network::cut`, `protocol::ledger`, the `p2p` crate.

### A node process

A node runs as one operating-system process, `isegoria-node <config>` (`10` T78), until it
is stopped, and resumes from its files when started again. Its configuration is a TOML file:

| Key | What it is |
|---|---|
| `network_id` | the network's id, 64 hex digits |
| `data_dir` | the node's directory |
| `listen`, `peers` | the multiaddresses it listens on and dials |
| `writers` | the writer set, one public key each (hex) |
| `consortium.members`, `consortium.threshold` | the members' public keys in order, and `t` |
| `key_file` | optional: a file holding the writer's 32-byte seed (hex); a member's key is its writer key |
| `consortium.circle` | optional: the members' circle keys in member order — a tiered network, whose `writers` must be exactly the members' and circle keys |
| `circle_key_file` | a member of a tiered network: its circle seed (hex), whose key must be its own in `consortium.circle` |
| `consortium.hpke` | optional, tiered: the members' signed key configurations in member order, in hex (§Blind relays; `--announce` prints a member's) |
| `hpke_key_file`, `relays` | a tiered member's gateway: its HPKE seed (hex), whose configuration must be its own in `consortium.hpke`, and the relays' transport keys it answers |
| `relay` | `true` for a tiered non-member that forwards persons' requests; needs every member's configuration |
| `member` | optional: `index`, `every_ms`, `cuts_per_epoch`, `patience`, `view_patience` |
| `sync_every_ms`, `status_every_ms` | the periodic pull, and how often the status is written |

Under `data_dir`: `own/`, the writer's feed (§A node's own disk), and `circle/`, a tiered
member's circle feed; `replica/`, its replica on disk (§A replica on disk), with a tiered
node's selected archive shards in `replica/archive/` (§Archive holders and transfer);
`transport.key`, the seed of its libp2p key, made at the first
start, so its peer id stays the same across restarts; and `status`, rewritten whole at each
period (a file written beside it, then renamed): whether each feed is caught up, its
peers and how many of them are bound members, the equivocations its replica holds, whether the consortium equivocated, each collected cut's
digest from number 0, and each finished round's value and withholders. A configuration that
does not hold — a bad key, a threshold outside `1..=n`, a member without a key, circle keys
that are not one distinct key per member, `writers` other than the tiered set's keys, a
tiered member without its own circle key, a circle key file outside one — refuses to start. The node dials its peers at the start and again whenever it has none, so a node
restarted later reaches the others, which dial no one.

The collected cuts are kept, not collected again from number 0 at each look: a node extends
them from the last one, counting the beacon's rounds as it goes (`p2p::member::Collector`).
A cut signed by `t` members for a number already collected, other than the one collected,
is the consortium equivocating: the node keeps the cuts before that number and stops there,
as collecting from 0 would.

## Operating contract (T88) — proposed

**Status (2026-10-22): proposed; not approved.** T88-P1–P4 await the owner's decision before
the independent-operator trial; *approved* and *built* restate D54, D55 and the rules above.
The answer review's side is `05` §Distributed integration (T79-P1–P7).

- **T88-P1 — the genesis.** One canonical file every operator holds: version and family
  (T79-P1), the members in order — each with its consortium, circle-feed and HPKE (T85)
  public keys —, the threshold, the issuer's public key (test credentials until enrollment,
  `10` T19, T22) and the family's configuration. The network id is
  `H("isegoria/genesis/v1" ‖ genesis)`; a node whose configured id differs does not start.
  Operators generate their keys locally and exchange public keys only; each checks its own
  keys and the id. Every entry and cut signature covers the id, so a member's first entry is
  its acceptance, and cut 0 needs `t` of them. No change in band: a new key, member,
  threshold or parameter is a new genesis — a new network, which may name the old one's last
  cut digest. In the trial it starts without the old state; that is an operating choice, not
  D54's fork, which keeps the state through a member that joins it or, with none, from the
  last anchored state root as each nym opens its leaf (T86). Carrying author records over
  otherwise would need its own decision.
- **T88-P2 — keys, feeds, transport.** *Approved (D54):* two feeds per member, each under its
  own writer key; circle entries served only to transport keys a member key binds; relays
  out of the writer set (T85). Proposed: the public feed's key is the consortium key, so
  §Members' objects stays as built; the circle key is a second key; the writer set is
  exactly the members' keys, two each, and an entry's tier is its writer key's. One secret per
  member: its circle, HPKE and libp2p keys derive from the member's seed, domain-separated
  (bespoke, with known-answer tests), so each member has exactly one transport key, bound
  by the consortium key's signature over
  `H("isegoria/transport/bind/v1" ‖ network id ‖ member-set hash ‖ index ‖ transport key)`
  and presented when a connection opens; no revocation within a network. An unbound peer —
  relay, light node, any reader — gets the public tier and the cut marks only, and follows
  the cuts as §Who reads describes. *Built (T77):* the tiered writer set, the binding,
  its exchange on connect and the filtered transport, from keys configured separately
  (`consortium.circle`, `circle_key_file`, `transport.key`); not the derivation.
- **T88-P3 — quorum and faults.** `2t > n`, so views change (T76); recommended `n = 4`,
  `t = 3`: one member may stop, while cuts, the beacon value and closures each need three.
  Each property has its own assumption:
  - *Agreement on the cuts* assumes every member follows the rules, faults being crashes,
    restarts, delay, loss, duplication and partitions. It does not follow from fewer than `t`
    colluding: `2t − n` members signing two cuts in one view (two, for `n = 4`, `t = 3`),
    with the others split, complete both. Two cuts collected for one number are evidence
    and stop every node (NET-006); they are detected, not prevented.
  - *The beacon's unpredictability* assumes fewer than `t` colluding members (§The epoch's
    beacon). It gives no agreement, and leaves the withholding bias and stall.
  - *Authorization* — a cut, or a closure under T79-P5 — needs `t` signatures or requests,
    so fewer than `t` colluding members cannot produce one on their own.

  Progress needs `t` members up, connected and not excluded (rule 6). It stops at a
  consortium conflict or below `t` live members; a member whose feed equivocates is lost
  until a new genesis.
- **T88-P4 — a stale peer.** *Built (T80):* a writer catches up with the first peer that
  answers. Proposed: after a start, a member writes nothing until sync rounds with the nodes
  of `n − t + 1` other members have ended with none offering an entry of its keys it lacks.
  A cut that marked its entries had `t` signers holding them, at least `t − 1` of them others,
  so any `n − t + 1` others include one. This assumes members keep the history they signed
  for: a member whose replica was lost, set aside or restored from an older copy may no
  longer hold those entries, and reaching it then proves nothing. An entry no cut marked,
  held only by nodes not reached, remains a risk; its equivocation is evidence and stops the
  feed. A node that writes nothing
  can only lag behind a stale peer: the cuts it applies are signed.

Node admission (none: any peer reads the public tier), the trial's written procedures and
Milestone 2's evidence stay in `10` T88.

---

## Durability: erasure coding

Erasure coding splits bytes into `k` data shards and `m` parity shards. Any `k`
authentic shards reconstruct the original bytes, so at most `m` unavailable or corrupt
shards can be tolerated. Storage is approximately `(k+m)/k` times the original, plus
padding and metadata. This is a reconstruction bound, not an availability probability:
correlated outages, withholding and placement determine whether enough shards can be
obtained. No churn model or comparison with a fixed number of full copies is established.

**What is coded (`01` D54).** The **public archive**: the cuts and their signatures, the
beacon's records, the anchors, each epoch's roots, the state tree's shape and the released
items. It is the history that must outlive the consortium, so it is spread over every node.
The circle's data is not: Reed–Solomon here is systematic, its first `k` shards are the
object itself cut into pieces, so a shard handed to a node outside the circle discloses
what it holds; every member keeps the circle in full, n copies in different jurisdictions.

**Placement, audit, repair (`10` T84).** The archive is coded per cut — the public entries
the cut adds and their objects, one segment — not per object. A segment's shard manifest
(`network::erasure`, the hash of each shard) is bound to a consortium-signed object, so a
holder checks its shard against something the members signed. Each shard's holder follows
from the segment, the shard's index and the set of storage nodes, so any node computes who
should hold it; holders are audited on a period, and a segment with too few shards left is
rebuilt and its missing shards placed again, while at least `k` authentic shards remain
obtainable. The increments below supply the primitives, transfer, discovery and local
assignment/repair pass and its opt-in periodic runner. Automatic production and deployment
orchestration remain open.

### Public archive segments (T84, first increment)

`network::archive` supplies library primitives; it performs no network I/O or periodic
work. Their transport integration is specified in §Archive holders and transfer below. A caller provides a trusted `Context`: tiers, lineage, the previous cut (or none
at genesis), the exact target cut and view, and the excluded members established by the
verified beacon history. The module does not choose a fork, establish freshness, replay
beacon rounds or verify an external anchor. T79's application rules remain outside it.

**Segment.** One segment contains the two cut boundaries, the target's signatures in
member-index order, exactly the public entries added between them with their objects,
and, for a closing cut, a public state snapshot. Entries are ordered by `EntryId`
(writer, sequence, hash), not by application execution order. From the previous marks,
the verifier checks each public writer's sequence, previous hash, network-bound signature,
object CID and final target head; no full replica is needed. Unknown writers, removed
marks and invalid cut succession or epoch progression are rejected. Circle marks must
not decrease and must retain their head at equal length. Their extension remains attested
by the signers, as for the public observer; no circle entries or objects enter the segment.
A threshold of distinct, non-excluded members must authenticate the cut at its exact view.

A closing cut has exactly one snapshot: a `state::RootStatement` and a `state::Shape`.
The statement must match network, lineage, member set, epoch, cut number and cut digest;
the reconstructed shape must match both its state root and shape digest. Non-closing
cuts carry no snapshot. The shape preserves live and frozen leaves and their origins,
without records, openings or salt keys. The input root and application-state correctness
are attested by the manifest signers, not recomputed here. Segment verification checks
snapshot consistency; authentication of its bytes comes from the signed manifest, not
from the cut certificate alone. This adds no T79 record format.

**Manifest and trust.** A manifest commits to network, lineage, member-set hash, previous
cut digest (or none), target number and digest, view, segment CID, original byte length,
`k` data shards, `m` parity shards, and all ordered shard hashes. Its digest uses the tag
`isegoria/archive/manifest/v1` over its canonical encoding. The existing consortium
checkpoint binds that digest as its head and the target cut number as its height, under
the network and member-set hash. This is a small bespoke composition of the existing
checkpoint and erasure primitives, not a new cryptographic algorithm. Archive signature
checks use Ed25519 `verify_strict` over the existing checkpoint message. A `SignedManifest`
needs a threshold of distinct, non-excluded valid signatures before producing an opaque
`VerifiedManifest`. Audit, recovery and placement recheck it against the supplied context;
a value verified in another context is not an authorization token here.

**Recovery and repair.** Audit labels each index missing, corrupt or valid by its exact
expected length and signed shard hash. Recovery discards corrupt shards and requires at
least `k` authentic ones. After Reed–Solomon reconstruction, it re-encodes the bytes and
checks every manifest hash, including missing parity shards, to reject inconsistent
codewords and noncanonical padding. It then checks the segment CID and the entire segment
contract above. A signed manifest alone cannot bypass cut or public-delta validation.
Repair returns the exact original bytes for missing or corrupt indices. It does not
send them to a holder. With fewer than `k` authentic shards, recovery fails explicitly;
repair must be attempted while enough shards are still obtainable.

**Placement.** The caller supplies distinct 32-byte storage identifiers, at least `k+m`.
For each index in ascending order, choose the unused identifier minimizing the pair
`(H("isegoria/archive/place/v1", manifest_digest, index_le64, identifier), identifier)`.
The result is independent of roster order and gives each shard a different holder. The
roster is external to the manifest, so a changed roster can repair the same archive.
This function supplies neither admission nor independent operators: one actor can control
many identifiers. The local assignment/repair pass below connects this function to
holders. Roster agreement, admission of storage operators, eviction and scheduling remain
external; there is no claim of Sybil-resistant placement.

**Encoding and limits.** Version 1 uses `network::codec`: little-endian `u64` integers,
fixed-width hashes, length-prefixed byte fields and strict 0/1 option tags. Counts,
ordering, field widths and complete consumption are checked; duplicate indices or entries
and trailing bytes are rejected. Decoding establishes canonical structure, not trust.
The segment, manifest and signed envelope have separate version bytes. Their field order is:

- Segment: network, lineage, member set; optional previous cut field; target cut field;
  view; signature count and `(member index, 64-byte signature)` pairs; entry count and
  `(168-byte signed entry, object field)` pairs; optional `(root-statement field, shape field)`.
- Manifest: network, lineage, member set; optional previous cut digest; target number,
  target digest, view, segment CID, original length, `k`, `m`, hash count and shard hashes.
- Signed manifest: manifest field, signature count and ordered index/signature pairs.

The segment is at most 32 MiB, the manifest and its signed envelope at most 64 KiB.
Both shard counts are positive and their sum is at most 256. A shard is at most the
replica object limit (16 MiB); total encoded shard bytes are at most 64 MiB. Checked
arithmetic and structural size preflight precede serialization, copying or erasure
allocation. The 64 MiB bound is on encoded shards, not the process's total working memory.
A whole cut exceeding the segment bound is rejected, never truncated or silently split;
runtime cut sizing remains an integration requirement. Layout is explicit input, with no
deployment default; selection belongs in [14](14-parameter-register.md#archive-layout).

**Acceptance and remaining work.** AT-NET-26 covers public-only extraction, authenticated
recovery without a full replica, context and cut rejection, state shapes, shard loss and
corruption, exact repairs, deterministic distinct placement, canonical codecs and resource
bounds. Tests include manifests signed over inconsistent codewords and over semantically
invalid segments. These are in-process checks, not holder-outage or multi-machine trials.
The holder service below adds persistence and transfer. These primitive tests alone do not
establish live availability. T75 fetches objects with a public writer's signed entry;
archive shards instead use the signed-manifest transfer path below. No new decoder
fuzzing, availability model or T79/T88 approval is implied.

### Archive holders and transfer (T84, second increment)

Tiered `p2p::Handle` nodes expose a read-only archive service. With `Config.store`,
selected shards persist in its `archive/` subdirectory; otherwise they live in memory.
Untiered nodes do not negotiate this protocol. No new TOML setting is needed: the node
process uses the same tiered handle and persistent directory. Admission and retrieval
are library APIs; there is no archive import/client CLI or automatic manifest production.

**Local admission.** `Handle::install_archive` takes the trusted archive context, the
signed manifest, the complete public segment and an explicit set of shard indices. It
verifies the context and certificate, rebuilds the segment's codeword and compares the
whole manifest before retaining only the selected shards. A threshold signature over an
invalid or circle-bearing segment does not authorize local installation. Empty, repeated
or out-of-range indices are rejected. The private prepared value is bound to the node's
network, ordered member/circle keys and threshold; verification under another configuration,
including a lower threshold with the same keys, cannot authorize installation here.

Installation replaces that manifest's selected shard set. A persistent holder writes a
temporary record, syncs it, atomically renames it and syncs the directory before acknowledging
success. A failure before replacement leaves the old committed record intact; after a
rename, a sync failure is an uncertain acknowledgement and a retry is safe. The quota
conservatively reserves room for the old and new records during replacement. Records
encode version `1`, the tier-configuration digest, a length-prefixed signed manifest,
a `u64` count, then strictly increasing `(u64 index, length-prefixed shard)` pairs. They
contain the selected shards, not the complete segment,
replica, circle objects or state openings. The directory header binds the tier configuration;
opening under another configuration fails. On restart and before serving, the holder
checks the record's bounded encoding, manifest certificate, file identity and every retained
shard's length and hash. A damaged record is unavailable as a whole and can be replaced by
a valid local reinstall. Temporary records are not served. Restart certificate validation
checks the configured quorum; it does not reconstruct beacon exclusions, select a lineage
or establish freshness. Clients supply their own trusted context and apply the full
first-increment verification. The local storage directory is controlled by its operator.

**Wire format.** `/isegoria/<network-id-hex>/archive/1` carries only reads. The request is
exactly 41 bytes: version `1`, manifest digest (32 bytes), shard index (`u64` little-endian).
A missing response is `[1, 0]`. A present response is `[1, 1]`, the manifest digest,
index, and the shard bytes. The response limit is 16 MiB plus 42 bytes, checked during
stream reading; extra bytes, invalid tags and noncanonical requests are refused. Requests
never contain a filesystem path, and there is no wire write, admission or repair command.
Archive shards are not advertised as T75 public-feed objects; their separate routing
keys are specified in §Archive holder discovery below.

**Retrieval.** `p2p::archive::fetch` accepts the trusted context, signed manifest, an explicit
bounded list of indexed peer identities and addresses, and a total deadline. It verifies
the manifest before contacting a holder, uses a fresh transport key, and matches each reply
to its request and authenticated peer. Only bytes matching the requested manifest, index,
exact shard length and signed hash count toward `k`. Once enough arrive, existing archive
recovery checks the entire codeword, segment CID and cut/public-delta contract and returns
the segment and repair bytes. Wrong, missing or failed replies do not count; insufficient
shards or the deadline produce an error. Synchronous verification and reconstruction are
not preemptible; the deadline is checked again before returning success. This needs no full
replica. The fresh key does not make retrieval anonymous: holders observe the client's
address, requested archive and timing. Caller-selected holders establish no independence;
one peer may supply several indices. Duplicate indices are refused.

**Resource bounds.** One holder accepts at most 256 archive records, 4,096 retained shards
and 256 MiB of encoded record bytes. Directory scanning is capped at 512 entries, including
temporary and damaged files, whose actual sizes count toward the byte budget. Each damaged
archive conservatively reserves 256 shard slots. A record is at most 64 MiB plus the 64 KiB
signed-manifest bound, 49 header bytes and 16 bytes per possible shard (256). These are
implementation limits, not retention promises or calibrated deployment parameters. There
is no eviction policy: a full holder refuses admission, and a directory already exceeding
these conservative bounds is refused at startup. The client accepts at most 256
holder entries, each with an address of at most 1,024 bytes and 16 components. Each transport
behaviour allows at most eight concurrent streams with a three-second request timeout,
inside the caller's total deadline. These limits do not describe total process memory or
protect against unlimited connections.

**Evidence and remaining work.** AT-NET-27 exercises loopback transfer, persistent restart,
loss of holders, damaged files, local repair, context/admission rejection and wire limits.
It establishes neither independent operators nor Internet reachability. T84 remains open
for automatic manifest assembly/co-signing, roster coordination, process integration,
coordinated repair across holders and an independent-operator trial. The later increments
below supply local audit/repair and a periodic runner.
Returned repair bytes are not automatically sent anywhere; local reinstallation is the
repair operation demonstrated here. T79/T88 remain unapproved and new codecs are not fuzzed.

### Archive holder discovery (T84, third increment)

`p2p::archive::fetch_discovered` takes a trusted archive context, its signed manifest,
bootstrap addresses and a total deadline. The bootstrap need not hold any shard. The
client verifies the manifest before contacting the network, finds providers for its shard
indices through the existing network-scoped Kademlia protocol, and retrieves bytes through
the read-only archive service above. It needs neither a full replica nor a caller-supplied
holder list. The signed manifest and trusted cut context still come from the caller;
this is not manifest discovery or a way to choose the latest cut.

**Announcements.** Archive provider keys are 65 bytes: ASCII `isegoria/archive/shard/1/`,
the 32-byte manifest digest and a `u64` little-endian shard index, distinct from T75's
bare 32-byte CID keys. A tiered holder advertises only its selected, locally admitted
shards. On restart it rebuilds those announcements from
records that pass the bounded certificate, file identity and shard checks. A successful
replacement removes obsolete local keys and queued announcements and stops providing
them before announcing the new selection. Queries already in flight finish naturally
and remain counted against the concurrency limit; they may deliver an obsolete hint
after replacement. Public-object and archive announcements share the existing
application-started query limit. Circle CIDs, private payloads and unselected
shard indices are not added to this catalogue.

Provider records are untrusted routing hints. Remote records may outlive replacement,
loss or disk damage; stopping a local announcement does not erase other peers' caches.
There is no periodic disk audit in this increment, and every read still rechecks the
stored record. The client matches responses to the authenticated peer, request, manifest,
index, exact length and signed shard hash. Failed or invalid providers can be replaced by
other candidates for that index. Each index counts at most once toward `k`, even if several
providers supply it. The same peer may supply multiple indices: discovery establishes
neither distinct operators nor availability. Once `k` authentic indices arrive, the full
codeword, segment CID and cut/public-delta checks are the same as explicit-holder recovery.

**Bounds and exposure.** At most 32 bootstrap addresses are accepted, each no larger than
1,024 bytes or 16 components. The client runs at most eight provider queries and eight
shard requests concurrently, retains at most 32 candidates per index and 256 candidates
in total, and attempts each `(peer, index)` at most once. It retains up to eight bounded
addresses per admitted peer while its provider query is still live, so later retries
do not depend on temporary query state. The address size/component limits also apply here.
These finite budgets can exclude an honest provider when hostile hints fill them;
exhaustion is not proof that an archive
is absent. Three-second query/request timeouts sit inside the caller's total deadline,
which covers verification, discovery and reconstruction and is rechecked before success.
Synchronous verification cannot be preempted. Network protocol names isolate routing;
the manifest and content checks establish authenticity. The fresh client transport key
does not hide its address, queried archive indices or timing from contacted peers. No
OHTTP, NAT traversal, Sybil resistance or traffic-volume defense is added here.

**Evidence and remaining work.** AT-NET-28 exercises five unknown holders through a
bootstrap with no archive, recovery after two holders stop, retained-only restart and
damaged-record exclusion, same-index fallback with checked negative and positive controls,
network/key separation, announcement withdrawal and retry-address lifetime. Invalid
manifests are refused before contact; bootstrap/candidate bounds and the total deadline are
tested. These are loopback and in-process checks, not an independent-operator trial.
The client remains a library API. The local assignment/repair pass below adds placement
and recovery by a holder; the periodic runner follows it. Automatic production/co-signing
and coordinated repair across holders remain separate work. T79/T88 are unchanged;
no new fuzzing or mutation campaign is claimed.

### Local archive assignment and repair (T84, fourth increment)

`Handle::maintain_archive` performs one locally requested pass for a trusted archive
context, signed manifest, storage roster, bootstrap addresses and deadline. It connects
the placement function above to a holder's persistent storage and discovered retrieval.
It neither selects a cut nor creates or co-signs a manifest. The caller still supplies
the trusted lineage, cut, view and exclusions, and chooses the roster. The roster is not
part of the signed manifest: this API establishes no agreement on which nodes should store
an archive, operator independence or Sybil resistance.

**Assignment.** Storage identifiers in this API are Ed25519 transport public keys. The
roster must contain at least `k+m` distinct, decodable keys and at most 1,024. The holder
extracts its own key from its transport PeerId and uses the existing order-independent
placement algorithm to determine its index. An unsupported transport identity is refused.
Manifest/context and roster checks precede network contact, and the node checks the exact
tier-configuration digest, including the threshold, before local admission. A node with
no assigned index returns `Unassigned`, leaves any earlier retained shards intact and
contacts nobody. Roster changes do not authorize deletion or select a new lineage.

**Audit and recovery.** The pass reads and validates the actual archive record, including
its certificate, file identity and retained shard lengths/hashes. Cached catalogue entries
are not evidence of intact bytes. If the assigned shard is valid, it returns `Healthy`
with its index without fetching or rewriting the record. Otherwise it discovers enough
authentic shards, reconstructs and verifies the complete public segment, and prepares a
local installation using the same admission rules as `install_archive`. It retains the
assigned index together with any previously catalogued indices of that manifest, so
repair does not silently evict additional local copies. A record already damaged at
startup has no validated catalogue; only the newly assigned index is then retained.

The existing atomic write, configuration checks and storage quotas apply. A successful
repair returns `Repaired` with the assigned index and announces the stored selection.
Unavailable or invalid sources, fewer than `k` obtainable authentic shards, quota errors
and expiry before commitment do not install a new record. The previous stored record
is preserved on pre-replacement failures. As for ordinary installation, a sync error
after rename is an uncertain acknowledgement, not a promise that the old record remains.
Old DHT hints may still exist while a record is missing or corrupt; reads authenticate
their replies and such hints cannot establish availability.

**Concurrency and deadline.** The method borrows the non-cloneable `Handle` exclusively
for the pass. Other local installation or maintenance calls cannot overtake its audit
and overwrite its result through that handle; the node's network event loop keeps running.
The deadline covers verification, audit, discovery, reconstruction and preparation, with
a final expiry check in the node before installation starts. Once that local commit starts,
the method waits for its durability result rather than reporting a timeout after writing.
Dropping a future after its commit command is queued does not retract that command.
Synchronous validation and filesystem operations cannot be preempted. Bootstrap/discovery
bounds and archive quotas are unchanged. An intact or unassigned holder needs no bootstrap.

**Evidence.** AT-NET-29 covers five empty holders adopting their computed assignments,
recovery by a fresh client after the original source stops, restart loss and corruption,
live repair preserving extra indices, no-contact healthy/unassigned paths, context and
roster rejection, failed recovery preserving old data, and expiry before commitment.
The roster-limit test includes an unselected local identity within a valid roster.
These are loopback and in-process checks; no independent-operator evidence is added.

This single-pass operation has no timer; the optional runner below schedules repeated passes.
A newly assigned or restored holder pulls its own replacement bytes; nobody can write into
another holder through the archive protocol. There is no archive TOML setting, CLI or
remote-write endpoint. T79/T88 and application rules are unchanged.

### Periodic local archive maintenance (T84, fifth increment)

`Handle::run_archive_maintenance` schedules the existing pass over an explicit list of
`archive::MaintenanceJob`s. Each job borrows its trusted context, signed manifest, transport
roster and bootstrap addresses. The caller supplies `MaintenanceSchedule` with a nonzero
pause between sweeps and a nonzero deadline for each pass. There is no deployment default.
The list must contain 1 to 256 distinct manifest digests; invalid list sizes, duplicates,
zero durations and durations outside the timer's range are rejected before any pass.
Each job accepts at most 32 bootstrap addresses, each at most 1,024 bytes and 16 components,
checked before cloning them for a pass. An empty bootstrap is allowed for local-only checks.
Archive/context, roster and local configuration checks remain those of `maintain_archive`;
a rejected job is reported as a pass error, not silently replaced with discovered authority.

**Scheduling and reporting.** The first sweep starts immediately and visits jobs in list
order. One pass finishes before the next starts; after the complete sweep, the runner waits
the configured pause. Slow recovery does not create overlapping work or a backlog of missed
ticks. A failed job does not prevent later jobs, and is tried again on the next sweep using
the same supplied inputs. The non-cloneable handle remains exclusively borrowed while the
runner executes; the node's network event loop continues independently.

After each pass, a `MaintenanceReport` identifies the job and manifest and holds its outcome
or error. A Tokio watch channel retains only the latest report: a slow observer can miss
intermediate outcomes, and this is not an audit log or a per-archive health catalogue.
There is no accumulating report queue, and an absent observer does not block or stop the
runner. Observers must release watch-channel borrows before awaiting: holding a read guard
can block publication under the channel's normal synchronization. Callers needing a complete
history must arrange their own reporting.

**Stop and restart.** A one-shot stop signal, including dropping its sender, prevents the
next pass from starting and interrupts the pause. A stop already signalled before the run
starts performs no pass. An active pass is allowed to finish and report its result; in
particular, the runner awaits an accepted durable write's acknowledgement. Stopping during
discovery can therefore wait for the remaining per-pass deadline; synchronous work and an
accepted durable commit have the limits described above. This graceful stop guarantee
requires awaiting the runner: dropping or aborting its future is not a rollback and cannot
retract a queued commit. No background task is spawned by this API.

The schedule and job list are in memory. After a process restart the caller must supply
them again, and a new run audits the persistent bytes afresh. To change trusted inputs or
rosters, stop and await the current run before starting another; there is no implicit
manifest refresh, roster agreement, eviction or reassignment of a dead holder's identity.

**Evidence.** AT-NET-30 covers schedule and bootstrap bounds, no-work pre-signalled stop,
serial sweeps without catch-up, closed observers, multi-job error isolation, repeated
recovery and repair of live corruption, stopping during discovery and waiting for a durable
commit acknowledgement. These are in-process and loopback checks, not an independent-operator
trial; no new fuzzing or mutation campaign is claimed.

This remains a library API. The node executable does not yet obtain trusted archive jobs or
start this runner from its configuration. Automatic manifest assembly/co-signing, roster
coordination, process integration, repair orchestration across holders and validation by
independent operators remain open. T79/T88 and application rules are unchanged.


---

## Long-term immutability: anchoring to a public chain

The planned anchoring publishes a fingerprint of a checkpoint through OpenTimestamps.
A verified proof ties that fingerprint to the external chain's history, allowing a
reader with the proof and matching data to check the commitment. It does not establish
the correctness of the committed results, ensure data availability, or prevent the
consortium from stopping. The schedule and live submission are still T17.

In code, the OpenTimestamps proof format and its verification (`network::anchoring`);
submitting to a calendar, reading Bitcoin headers and anchoring the checkpoint head on a
schedule are `10` T17.

---

## Node types

| | Person-nodes | Signer-nodes (consortium) | Light-nodes |
|---|---|---|---|
| What they are | pseudonymous participants | always-on servers | any client |
| What they do | propose, judge, answer; keep their own records (`01` D54) | full copy + sign the state | shards of the public archive + verify signatures |
| How many | unlimited | a few dozen (cap ~n²) | unlimited |
| More of them = | safer | slower | slows nothing |
| Security from | quantity + independence | diversity of who controls them | — |

Analogy: few stable trackers, millions of clients (BitTorrent). The roles are in
layers, not in competition.

---

## Future hardenings (order)

See `01` D14. In short:

1. **Anchoring** (above) — integrity, low cost.
2. **Erasure coding** (above) — durability + distribution.
3. **Multiple consortia counter-signing each other** in different jurisdictions —
   distribution of trust, slower cross-coordination.
4. **Cryptographic proofs of the computation** (zk) — the epoch produces a succinct
   proof that the scores are correct; anyone verifies it in an instant without
   downloading the data or re-running it. A mature phase, technically demanding.

**Do NOT adopt:** permissionless consensus with an economic stake (proof-of-stake). It
is the maximum theoretical distribution but the stake is money/tokens: it reintroduces
wealth-based access, the problem to avoid.
