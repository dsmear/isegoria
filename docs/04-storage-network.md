# Storage and network

The property we need is not "distributed" in the abstract, it is: **nobody can delete
or rewrite questions and votes, and nobody can falsify the scores without it being
visible**. This is what makes the network independent of a government. The question is
which architecture achieves it at the lowest cost.

**Final choice.** A P2P network of signed logs (gossip + DHT + append-only + CRDT),
with a consortium of a few dozen heterogeneous signers as the backbone, erasure coding
for durability, anchoring to a public chain for long-term immutability. **No global
permissionless consensus.**

---

## Why not a permissionless blockchain

- **Cost and latency.** Every write costs and has seconds/minutes of latency.
- **Public by construction.** Everything is visible to everyone forever: questions
  under review must stay encrypted until publication, and voting patterns on a public
  chain would be a gift to anyone wanting to deanonymize.
- **Computation too heavy.** Factorization and IRT do not run on-chain; you would end
  up keeping the data on the chain and doing the math elsewhere.
- **Consensus is almost unnecessary.** Writes almost never conflict: two different
  proposals get added, two votes get summed. The only delicate case, double-voting, is
  solved with the nullifier (`03`), not with consensus. Removing global consensus
  removes the bulk of the cost and slowness.

Anti-censorship comes from the **diversity of operators** and from the
**reproducibility of the computation**, not from permissionless consensus.

---

## The four P2P structures (non-blockchain)

They combine, they do not exclude each other. They cleanly separate *transporting
data* from *agreeing on an order* (consensus, the expensive part we avoid here).

| Structure | Purpose |
|---|---|
| **Gossip / epidemic** | propagate questions and votes: each node repeats to its neighbors until everyone knows |
| **DHT** (distributed hash table) | find content: given the hash, which node has the object (like BitTorrent) |
| **Signed append-only logs** | immutability: an add-only register, each row containing the previous one's fingerprint; altering one breaks the chain visibly |
| **CRDT** | converge copies edited in parallel, with no arbiter (collaborative editors) |

How they fit together: questions and votes are objects identified by their hash. The
DHT is the hash→node registry. Gossip is the courier. The signed log is what makes it
all immutable — it is the "transparency log", which lives distributed on every node,
not on a server.

Real projects already doing this: the Bluesky protocol (per-user signed logs +
rebuildable indexes), Secure Scuttlebutt (a signed-log, gossip social network that
works offline), Automerge/Yjs (mature CRDTs). None is a blockchain.

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
1. **Reproducibility of the computation** — a signer signs the result of a public
   deterministic computation; if it cheats, anyone redoes the math and unmasks it. It
   need not be chosen well to be caught.
2. **Freedom to fork** — if the consortium betrays, the data lives on every node and
   the code is open: the community takes the whole history, abandons that consortium,
   and restarts with a different set of signers. Not the barrier to entry, but the
   freedom to leave, keeps the consortium honest.

System parameters and meta-level composition: **stratified sortition** (see `05`), not
voting.

### The epoch's beacon (D41)

Every draw of an epoch — the admission lottery, reviewer assignment and the band's extra
round, exploration, contested facts, honeypot placement, sortition — reads one public
random value, the epoch's **beacon**. It must be fixed only after the candidates are, and
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
7. **Fewer than `t` reveals.** No beacon for `e`: the epoch draws nothing, and its deposits
   and pending draws wait for epoch `e + 1`, whose round starts from new secrets (a retry
   of `e` would reuse the secrets already revealed and give the withholders a second
   look). The withholders are recorded as in 6. Requiring `t` reveals is what makes the
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
(§Cuts, `10` T74). On cuts, rule 6 is not applied yet: a withholder's signature still
counts toward the cuts that publish its epoch's draws and results (`10` T76).

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
   produce it, and dropping acknowledged history quietly would hide the damage. The node
   is then to recover its history from its peers and check it against its signed
   checkpoint (`log::verify_extends`); nothing does it yet, and the node does not start
   (`10` T80).
3. In `objects`, a final record that declares more bytes than the file still holds is torn
   and cut, reported the same way; a declared length above the bound refuses the file
   before anything is sized from it. A torn object whose length survived is only an
   orphan under another CID: the log entry it was written for was never written.

The reopened log is, entry for entry, the log that was acknowledged: its head, its
`contains` and its consistency with a signed checkpoint are those of the log before the
restart.

In code: `network::store` (`10` T13).

### Events and replay

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
the node accepts starts its item in `Deposited`; a lifecycle step applies `lifecycle::step` to the item's state and is
refused for an unknown item, for an assignment naming another item than the one it moves,
and wherever the §9.1 table refuses it.

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
3 an author's quality (author, quality, age in months: `AuthorHistory::record`), 4 an appeal
filed (author: `file_appeal`, which escrows a zero observation), 5 an appeal settled (author,
the escrow's position, and failed, or promoted with the measured quality), 6 exposure (item,
administrations), 7 a residual (reviewer, the item's global id, `r − r̂`), 8 a contested fit
(each class's share, ability mean and item parameters, each item's guessing floor, and
the classes' ability histogram, its nodes and weights — `ClassCurves::with_ability`
recomputes the curves from them — and the members with their index in the fit:
`ContestedPool::record`),
9 a contested fact removed. Reviewers and authors are their proven ids (INV-9); the CUSUM
parameters and the author prior are the node's configuration. The event is applied whole or
not at all, and refused for a second results event of the same epoch, an inclusion outside
`(0, 1]`, a value that is not finite, an appeal the author's reputation does not cover, a
settlement of an escrow that is not open, and a contested fit whose classes or ability
describe no fit or whose members repeat or fall outside it.

The node applies these results as written: it does not recompute them, and the inputs —
every reviewer's ratings and every respondent's answers — are not in the event, since they
are the voting patterns that must not be published (`08` PRIV-004). The event binds the results
to their inputs instead: it carries the RFC 6962 Merkle root (`network::merkle`) of one leaf
per input — a rating (the judge's id, the item, the probability's bits) or an answer (the
respondent's id, the batch, the item's index in it, the answer) — the leaves sorted, so the
root is a function of the set of inputs. Whoever holds the inputs recomputes the engine and
the root and checks both; a reviewer or respondent holding its own leaf and an inclusion
proof checks that its input was counted, without seeing anyone else's.

**The ratings are on the log anyway.** A reveal is a lifecycle step (kind 4, event 5) that
carries the judge's nym, its probability and its nonce, so the log holds every rating of a
first panel or an extra round joined to its judge's nym, and since replication every
replica holds them, relays included, for any peer to read (§Who reads). The results event
keeps the engine's inputs off the log; the reveals put the ratings on it, against `01`
D17 (`10` T77).

With the four kinds before it, this is a node's whole protocol state (`10` T73); the
issuing committee's registries are the committee's, not a node's.

## Replication between nodes

A node's log is its own; replication makes every infrastructure node hold the same **set**
of signed entries, whatever order they arrive in, over whatever path (`10` T18).

**Who writes.** Only infrastructure nodes sign entries: the consortium members and the
relays, whose ed25519 keys form the network's **writer set** (configuration; how a relay
joins it is the consortium's decision). A person never signs an entry: one key under a
person's deposits and reviews would link its author and evaluator pseudonyms (`00`
invariant 4). A person hands its event to a relay; the event carries its own nullifier
proof, and the relay's signature says only that this relay logged it.

**Who reads.** Nothing restricts it yet. A node answers the Summary and Want of any peer
that connects and pulls from it on connect, and a node started without a writer key
replicates the whole set; the transport authenticates a peer's libp2p key, which is bound to
no writer key. So any peer reads, in the clear, what the set holds: the drafts under review
(deposits), the panels before their verdict (assignments and reviewer admissions), every
reveal with its judge's nym and rating, the batches of each respondent id, and the results'
records per nym — against §Why not a permissionless blockchain, `08` PRIV-P5 and
PRIV-P6, and `01` D17: votes are never published in the clear, and until proofs of the
computation only the consortium holds them, to re-run the engine. How the set keeps them
— closed replication, confidential objects, or the voting patterns off the log — and
whether a relay may hold them are open (`10` T77); until then, a network runs only among
machines its operator controls.

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
signs every entry again — ed25519 signatures are deterministic, so they are the same — and
continues at the next `seq`; signing a fresh feed after losing its files would be an
equivocation. Nothing prevents it yet: a writer started with its files lost signs `seq` 0
again at its next publish (`10` T80). A node whose writer key is not in the writer set does
not start.

**Transport.** libp2p 0.56 over TCP, with Noise and Yamux: gossipsub (signed, strict
validation) carries the announcements, at most 512 identifiers per message, on the topic
`isegoria/<network id in hex>/entries`; request-response (`/isegoria/sync/1`, at most
64 MiB a message) carries Summary→Have and Want→Entries. A node pulls from a peer when they
connect, on a fixed period, and again after an Entries response that brought something new.
A node's libp2p key is its transport identity, distinct from its writer key. A message that
does not decode is dropped.

**A replica on disk.** A node given a directory keeps its replica in two content-addressed
object files (§A node's own disk): `objects`, the entries' objects, and `entries`, each
accepted signed entry's 168-byte encoding. An entry is written after its object and before
the node's memory changes; opening re-inserts every stored entry, checked again against the
writer set, so a restarted node holds the set it had.

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
feeds, one byte `0xC0` then a kind, so no event decodes as one: 1 a cut signature (the
cut's encoding, the member's index, its signature over the cut's checkpoint); 2 a beacon
commit (network, member-set hash, epoch, index, commitment, signature: §The epoch's
beacon); 3 a beacon reveal (epoch, index, secret). An object counts only on the feed of
the member its index names; on any other feed it is refused as not authorized.

**Collecting a cut.** A node takes, for the next number, the cut that `t` distinct members
signed on their feeds. Two different cuts each signed by `t` members for one number are the
consortium equivocating: the node stops there and holds both as evidence (NET-006).

**Proposing and signing.** Cut `k` is proposed by member `k mod n`, who signs it first; its
epoch and closing are its call. A member co-signs the proposer's cut for the next number
when the cut extends the last collected one, it holds every entry the cut names, and the
cut counts its own beacon messages: a closing cut must include its commit for the epoch,
and the cut after it its reveal, if the member has published them. A proposer that stays
silent stalls the cuts; replacing it is not specified yet (`10` T76).

**The beacon on cuts.** The round of epoch `e` (§The epoch's beacon) runs on the cuts: a
commit counts when a cut of epoch `e` applies it, up to and including the cut that closes
`e` — which is the commit checkpoint and the deposit checkpoint at once; a reveal counts
when the cut right after the closing one applies it, and at the end of that cut the round
finishes: its outcome (the value if `t` revealed, who revealed, who withheld) is part of
the state. A commit or reveal in any other cut, or that the round refuses, is refused.

Finding an object by its CID without a full replica, the DHT, is `10` T75. Nothing runs an
epoch's work between nodes yet — the draws, the reviews, the engine, and the lifecycle steps
and results they produce, which the members log (`10` T79) — and a node is started only by
a test (`10` T78). In code: `network::replica`, `network::cut`, `protocol::ledger`, the
`p2p` crate.

---

## Durability: erasure coding

The intuition "more reliable = more copies = more expensive" is wrong. With erasure
coding each datum is split into fragments scattered across many nodes, and only a
fraction is needed to reconstruct it. At equal storage it beats replication. With 30%
churn (a node unreachable 30% of the time):

| Method | Storage | Availability |
|---|---|---|
| replication ×3 | 3× | 0.973 |
| replication ×5 | 5× | 0.998 |
| erasure (10,20) | 2× | 0.983 |
| erasure (10,30) | 3× | ~1.000 |

`erasure (10,30)` uses 3× and is safer than 12 full copies. And durability no longer
lives only on the signers but on **all** nodes, even the light ones that come and go.
Slower to write (encoding + distribution) and read (fragment gathering), but
**simultaneously more distributed and more reliable**.

---

## Long-term immutability: anchoring to a public chain

Nothing runs on a blockchain, but every so often (an hour, a day) a single fingerprint
summarizing the whole network state is published on Bitcoin/Ethereum. It costs a few
cents each time. The gain: to rewrite the network's history an attacker would have to
rewrite *also* the most expensive public chain in the world. It turns "nobody can
falsify the past" from a consortium promise into a fact anchored to a chain the
consortium does not control. Technique: OpenTimestamps or equivalent. **It is the
addition with the best value/slowness ratio.**

In code, the OpenTimestamps proof format and its verification (`network::anchoring`);
submitting to a calendar, reading Bitcoin headers and anchoring the checkpoint head on a
schedule are `10` T17.

---

## Node types

| | Person-nodes | Signer-nodes (consortium) | Light-nodes |
|---|---|---|---|
| What they are | pseudonymous participants | always-on servers | any client |
| What they do | propose, judge, answer | full copy + sign the state | slice of data + verify signatures |
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
