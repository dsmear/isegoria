# 25 — Public archive (T84)

| | |
|---|---|
| **Purpose** | The public history coded into shards and spread over the storage nodes: segments, holders, transfer, discovery, repair and the operators' offline tools. |
| **Scope** | Moved unchanged from `04` §Durability (2026-11-25); `04` keeps erasure coding's principle and what is coded. T84 is frozen and outside Milestone 2 (`10` §2.4); the trial exercises no archive (`01` D57). |

## Public archive segments (T84, first increment)

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

## Archive holders and transfer (T84, second increment)

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
for automatic manifest assembly/co-signing, roster coordination, coordinated repair across
holders and an independent-operator trial. The later increments below supply local
audit/repair and a periodic runner, including explicit node-process configuration.
Returned repair bytes are not automatically sent anywhere; local reinstallation is the
repair operation demonstrated here. It decides nothing of T79 or T88, and new codecs are not
fuzzed.

## Archive holder discovery (T84, third increment)

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

## Local archive assignment and repair (T84, fourth increment)

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
another holder through the archive protocol. The process configuration below invokes this
local API; it adds no remote-write endpoint. T79/T88 and application rules are unchanged.

## Periodic local archive maintenance (T84, fifth increment)

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

The node executable can invoke this library runner through the explicit configuration
below. Automatic manifest assembly/co-signing, roster coordination, repair orchestration
across holders and validation by independent operators remain open. T79/T88 and application
rules are unchanged.

## Archive maintenance in a node process (T84, sixth increment)

`isegoria-node <config>` optionally starts the periodic runner above from an
`[archive_maintenance]` table. It is accepted only in a tiered network; without the table,
archive maintenance is disabled and the existing network operation is unchanged. Both
`interval_ms` (pause after a sweep) and `deadline_ms` (budget for each pass) are required,
nonzero and within the timer's range. No timing or archive jobs are supplied by default.

The table contains 1 to 256 `[[archive_maintenance.jobs]]` entries:

| Field | Operator-supplied input |
|---|---|
| `lineage` | 32-byte lineage identifier, in hex |
| `target_cut_file` | raw canonical `network::cut::Cut::encode` bytes for the trusted target |
| `previous_cut_file` | the previous cut in the same encoding; optional only when the target is cut 0 |
| `view` | the target cut's trusted view number |
| `excluded` | required array of excluded member indices; empty is an explicit choice |
| `manifest_file` | raw canonical `network::archive::SignedManifest::encode` bytes |
| `roster` | required array of storage nodes' Ed25519 transport public keys, in hex |
| `bootstrap` | required array of discovery multiaddresses; empty is allowed for local-only checks |

**Trust and admission.** Tiers, network id and consortium threshold come only from the
node's configuration. Lineage, cuts, view and exclusions are independently supplied by the
operator: they are not inferred from the manifest or fetched from providers. Loading these
files does not select the latest cut, authenticate the configuration's provenance or verify
an external anchor. The operator must obtain and pin the intended context through their
trusted procedure; distributed configuration agreement remains T88.

Before creating the data directory or transport key, planning loads and validates every
job. Each binary input must be a regular file of at most 64 KiB, read with a byte limit and
decoded in its existing canonical format. The configuration itself is bounded before TOML
deserialization. Relative paths retain the node's existing working-directory semantics.
All files are read once; no input file is watched or reloaded during the run. Local files
and their permissions are under the operator's control, not an adversarial filesystem API.

The signed manifest must verify against the independently loaded context and the node's
tiers, including exclusions. A noninitial cut requires its predecessor. Exclusions must be
distinct valid member indices. Each roster has `k+m` to 1,024 distinct decodable transport
keys; bootstrap bounds are 32 addresses, 1,024 bytes for both the textual and encoded
address, and 16 components per address. Duplicate
manifest digests, malformed encodings, context/certificate mismatches or invalid settings
refuse startup. The earlier archive admission and recovery checks still apply to fetched
bytes; valid configuration alone does not establish that any shard is available.

**Runtime and status.** The runner borrows the sole mutable `Handle`. A restricted
`p2p::Monitor` allows status queries and peer dialing alongside it, with no publication or
archive-installation API. Network activity, periodic normal status and reconnection continue
while maintenance waits for shards. The node task lives until the last handle or monitor is
dropped; the monitor cannot bypass the exclusive archive-maintenance borrow.

`data_dir/archive-status` is a separate latest-result file. Before starting the network,
startup replaces the previous run's report with `pending`, or `disabled` when the section
is absent; a failure to write this initial state refuses startup. Reports identify the
job index and manifest digest and print the maintenance result or error. They are diagnostic
output, not a stable wire format or a complete history: the underlying watch channel can
skip intermediate reports. Each update uses a sibling temporary file and rename; later write errors are reported on
standard error. A report
is the last completed observation, not a continuous availability certificate. The existing
`status` format and parser are unchanged.

On SIGTERM, Ctrl-C or the embedding's stop future, the process signals the runner and awaits
its graceful completion before final status writes. A recovery can consume its remaining
per-pass deadline, and a committed write is awaited; forced process termination has no such
guarantee. Restart loads the explicit configuration again and audits the persistent shards
afresh. Reconfiguring a roster or trusted context requires a restart and does not delete
previously retained shards.

**Evidence.** AT-NET-31 checks context and certificate mismatches, missing/invalid settings,
canonical decoding and file/count/address bounds, including nonregular files. Separate
executables exercise initial repair, live corruption, loss followed by restart, ordinary
status progress during recovery, graceful SIGTERM with an active pass and pending/disabled
status reset. These are controlled local processes on loopback, not independent operators.

This integrates operator-configured jobs into the executable; it does not automatically
create or co-sign manifests, discover trusted contexts, agree a roster, transfer a lost
node's identity to a replacement or demonstrate independent operation. Manual preparation
and co-signing tools are specified below. T79/T88 and the answer-review rules are unchanged.

## Manual archive preparation and co-signing (T84, seventh increment)

Three offline commands turn an explicitly supplied public `Segment` into an erasure-coded
candidate, independently reproduced member signatures and a quorum certificate:

```text
isegoria-node --archive-prepare <node.toml> <production.toml> <bundle-dir>
isegoria-node --archive-sign <node.toml> <production.toml> <manifest.bin> <share.bin>
isegoria-node --archive-combine <node.toml> <production.toml> <manifest.bin> <signed-manifest.bin> <share.bin>...
```

Each command prints the manifest digest as lowercase hex on success. An error prints an
`archive:` diagnostic and exits with code 2. They do not start networking, open a node's
durable replica, create its transport key or modify its data directory. The node TOML is
used for the configured tiered network and consortium; signing additionally requires the
member's configured index and `key_file`. Unrelated runtime settings are not a startup
validation by these offline commands. No private key is needed to prepare or combine.

`production.toml` requires `lineage`, `target_cut_file`, `view`, `excluded`, `segment_file`,
`data_shards` and `parity_shards`; `previous_cut_file` is required for a noninitial target
and absent for cut 0. Its snapshot fields are read only by extraction (§Segment extraction
from a stopped node's replica). Paths are relative to the working directory, as in node
configuration. The cuts are raw canonical `Cut::encode` bytes, and the segment is raw
canonical `Segment::encode` bytes. All context fields are independently pinned by the operator;
they are never inferred from the candidate manifest or its producer. Each signer supplies
their own trusted inputs, even when the candidate is received from another operator.

**Preparation.** The tool verifies the segment against the configured tiers and the pinned
lineage, predecessor, target, view and exclusions, then builds the exact manifest and shards
with the explicitly chosen `(k, m)`. Existing checks cover the cut certificate, public-only
delta, signatures, chain continuity and payload hashes. Circle entries cannot enter the
archive. For a closing cut, the segment must contain the T86 shape and matching root
statement; nothing creates a placeholder snapshot. The root's input/state correctness
remains the signers' external premise pending T79, as in the archive primitive contract.

The new bundle directory contains `manifest.bin` (the raw unsigned `Manifest`), canonical
`segment.bin`, `target.cut`, optional `previous.cut`, and `shard-000.bin` through the final
shard index. These are preparation outputs, not installed holder records; their manual
installation is specified below. Extraction from a stopped node's replica is manual (below);
automatic extraction from a live replica is not supplied. A `COMPLETE` marker records logical bundle completion, with this exact text:

```text
isegoria/archive/bundle/v1
manifest <64 lowercase hex digits>
```

The marker ends with a newline. It is neither authentication nor a quorum certificate.
Operators consuming a prepared bundle must require the marker and still authenticate its
contents. The signing and combining commands accept a raw candidate manifest file and
independently reconstruct it; they do not infer trust or completeness from this marker.

**Signing.** A member rebuilds the archive from their own production inputs and requires
the supplied candidate to be byte-for-byte canonical and equal to that rebuilt manifest.
Only then may the configured member sign its existing archive checkpoint. The seed must
match the configured consortium key at that index; a relay, missing/mismatched member key
or excluded member cannot sign through this tool. The output is an existing `SignedManifest`
encoding with exactly one signature. It is not generally a quorum certificate.

**Combination.** The combiner also independently reproduces the candidate. Every supplied
share must be canonical, name exactly that manifest and contain exactly one valid signature
from a distinct nonexcluded configured member. An invalid extra is refused even when other
shares already reach quorum. Strict verification uses the existing checkpoint message
(`Checkpoint::verify_signature`); membership, exclusion and quorum are separate checks.
Signatures are sorted by index, the complete `SignedManifest` is checked against the pinned
context, and only a valid quorum certificate is published. Its output can be supplied as
`manifest_file` to node archive maintenance. Supplying the certificate does not seed holders
or prove that any listed shard is currently available.

**Bounds and publication.** Both TOML inputs are regular files bounded to 1 MiB. Cuts are
bounded to 64 KiB; manifests, shares and the combined certificate to 64 KiB; the segment to
32 MiB; the member seed file to 128 bytes. Existing layout and total codeword limits apply.
Combination accepts at most 910 share paths and no more than the configured member count;
the complete certificate must still fit its encoding limit. Files are read with a byte
limit before decoding, and canonical re-encoding is checked. Duplicate consortium keys,
invalid exclusions and any other validation error refuse the operation before publication.

Existing output files and directories, including empty directories, are never overwritten.
Files are staged and synced beside the destination, then published with a no-clobber hard
link; the filesystem must support hard links. For a bundle, `create_dir` reserves a new
destination atomically, its staged files are linked, and `COMPLETE` is linked last. This is
logical completion, not atomic visibility of an entire directory: an interrupted bundle
can remain without the marker and must not be used. Pre-marker cleanup removes only this
operation's linked files and its own empty directory, never recursively removes an existing
destination. Unix directory syncs persist the publication order; power-loss durability is
not established by the tests, and no directory-sync guarantee is added on other platforms.
A sync error after publishing a file or marker is an uncertain acknowledgement: output may
exist despite the reported error. Local directories and permissions are operator-controlled.

**Evidence.** AT-NET-32 exercises the executable prepare/sign/combine workflow with a nonempty
public delta and private entries excluded, two of three member signatures, a certificate
accepted by the existing maintenance planner and recovery from the produced shards. It also
checks independent-context/candidate mismatches, invalid or duplicate extra shares, signer
authorization, closing-snapshot consistency, canonical input bounds and preservation of
existing output files and empty directories. These are controlled offline processes and
in-process checks, not automatic publication between members or independent-operator evidence.

These are manual preparation and certificate tools. Automatic segment extraction, manifest
production/co-signing between running members, roster agreement and the independent-operator
trial remain open. T79/T88 and application rules are unchanged.

## Initial archive installation on a holder (T84, eighth increment)

One offline command installs a prepared, certified bundle's assigned shard into a stopped
holder's own store, so that its node process serves and maintains it through the paths above:

```text
isegoria-node --archive-install <node.toml> <bundle-dir>
```

On success it prints the manifest digest as lowercase hex, then `shard <index> installed`,
or `shard <index> present` when nothing was written. An error prints an `archive:` diagnostic
and exits with code 2. The command starts no networking and sends nothing: there is no remote
upload, and no holder writes into another. Each holder's operator installs their own shard.

**Trusted inputs.** The node TOML is the holder's own, tiered, with an `[archive_maintenance]`
table (§Archive maintenance in a node process); its whole plan is loaded and checked as at
startup. The bundle's `COMPLETE` marker must be present, with the exact text of §Manual
archive preparation and co-signing, and name the manifest digest of one configured job. That job alone supplies the independently pinned
lineage, cuts, view and exclusions, the quorum certificate (`manifest_file`) and the roster;
tiers, network id and threshold come from the node configuration. The marker only selects a
job and records that preparation finished; it authenticates nothing. The bundle's
`manifest.bin` and cut copies are not read. Unrelated runtime settings are not validated.

**Assignment.** The holder's identity is its existing `data_dir/transport.key`, read, as at
node start, from a regular file of at most 128 bytes (the 64-digit hex seed and whitespace);
this command never creates or modifies it. The existing placement over the job's roster
gives that transport key at most one index; an unassigned key is refused before anything is
written. A later roster can assign another index of the same archive.

**Roster key.** `isegoria-node --transport-public-key <node.toml>` prints one line on stdout:
the 32-byte Ed25519 public key of the existing `data_dir/transport.key` in lowercase hex, the
form of a `roster` entry. Besides the node TOML, which must only parse, it reads that key
alone, with the bound and decoding above, and never prints the seed. It makes or changes no
key, directory, lock, status or store, plans and starts nothing, needs no archive job or
member role, and also runs beside a running node. A missing, malformed, oversized or
non-regular key prints a diagnostic naming the key file on stderr and exits with code 2. The
command makes no identity: the node makes its key at its first start (`04` §A node process).

**Verification before availability.** `segment.bin` is read within the 32 MiB segment bound
and must be canonical. The admission of `Handle::install_archive`, exposed for a stopped holder
as `p2p::archive::Admitted`, verifies the certificate against the pinned context and tiers,
rebuilds the codeword from the segment under the full segment contract and compares the whole
manifest. `shard-<index>.bin`, with the index in three digits, is read within the 16 MiB shard
bound and must match the signed hash of the assigned index. Every check precedes the lock and
the store, so a refused input writes nothing.

**Storage.** The holder store is `data_dir/replica/archive/` with its existing configuration
header, scan bounds and quotas (§Archive holders and transfer). If a valid record of the
manifest already retains the assigned index, nothing is written. Otherwise, as in maintenance,
the record is replaced by one retaining its previously valid indices and the assigned index,
or only the assigned index if the old record is damaged. Other manifests' records are not
touched. The existing temporary write, sync, rename and directory sync, failure semantics and
record format apply unchanged: a refusal before rename keeps the committed record, and a sync
error after rename is an uncertain acknowledgement that a retry resolves.

**Concurrency with a node process.** Only one process may use a data directory's store.
`isegoria-node <config>` takes an exclusive advisory lock on `data_dir/lock` before it resets
`archive-status` and hands it to the node's task (`Handle::spawn_holding`), which owns the
stores and drops the lock after them when it ends. `node::run` returns only after that task
has ended, also on an error after the task started; if `run` is dropped, the task first
answers its queued commands, such as an accepted installation or maintenance commit, and only
then releases the lock. The installer takes the same lock without waiting, after its checks,
and holds it while it opens and writes the store. While a node runs, the installer is refused
without writing; while an installation runs, a node, like a second node on the same
directory, refuses to start (exit 1) before resetting its status. The procedure is to stop
the node, install, and start it again. The operating system releases the lock when its
process ends, including a forced kill. It is a Unix `flock` on a local filesystem; elsewhere
nothing enforces it and the operator must stop the node. In-process users of
`p2p::Handle::spawn` are outside this lock.

**After installation.** At its next start the node validates the record, announces the
retained shards and serves them; the configured job's pass audits the actual bytes and reports
`Healthy` without fetching. Installing one holder does not establish that other holders, or
any `k` of them, hold their shards.

**Evidence.** AT-NET-33 runs the real prepare/sign/combine/install commands for three holders
of a nonempty public delta, serves the segment from their processes, recovers after one holder
stops and audits the installed shard as healthy after restart. It checks refusal of altered
context, below-quorum or unconfigured certificates, missing or mismatched markers, altered
segment and shard bytes, and unassigned or missing identities; a running node refuses both
the installer and a second node. The lock stays held until the storage task ends after a
stop, an error or the cancellation of `run`, and a queued installation completes before it is
released; oversized, non-regular or malformed transport keys are refused without blocking or
writing. `--transport-public-key` reproduces RFC 8032's first test vector, its lines used as a
roster assign each holder a shard, it reads beside a running node, and its refusals write
nothing. Repetition writes nothing; other indices and archives are preserved; full or
differently bound stores refuse without change. These are controlled local processes, not
independent operators.

This step is manual and local. Remote upload, automatic extraction from the feeds, provisioning
of other holders, roster agreement, repair orchestration across holders and the
independent-operator trial remain open. T79/T88 and application rules are unchanged.

## Segment extraction from a stopped node's replica (T84, ninth increment)

One offline command extracts the public `Segment` of an operator-pinned cut from a stopped
node's own replica and publishes it as the `segment_file` that §Manual archive preparation
and co-signing then prepares, certifies and installs:

```text
isegoria-node --archive-extract <node.toml> <production.toml>
```

On success it prints the segment CID as lowercase hex. An error prints an `archive:`
diagnostic and exits with code 2. A refusal before publication publishes nothing; an error
after the segment is linked is an uncertain acknowledgement, as for every offline output
(§Manual archive preparation and co-signing): the segment may exist despite the error. The
command starts no networking and fetches nothing.

**Inputs.** The node TOML is the node's own, tiered; tiers, network id, threshold and
`data_dir` come from it. `production.toml` supplies the pinned lineage, cuts, view and
exclusions, checked as for preparation; `data_shards` and `parity_shards` are not used. A
closing target requires two optional fields, `root_statement_file` and `shape_file`: its T86
snapshot as canonical `state::RootStatement::encode` and `state::Shape::encode` bytes, read
within 64 KiB and 32 MiB. A non-closing target refuses them; preparation, signing and
combination do not read them. The command reads only these files and the replica's
`data_dir/replica/entries` and `data_dir/replica/objects`.

**Replica access.** After every input check, the command takes the data directory's lock
without waiting, as the installer does (§Initial archive installation on a holder), and
holds it while it reads the replica: it is refused while a node runs, and a node does not
start during the read. `DurableReplica::read` opens both files, which must be regular,
read-only and never creates, cuts or writes them. As at node start, it rehashes every
record and reinserts every entry against the writer set; a torn header or tail, which a
writable open would cut, is refused as `StoreError::Torn`, as are damaged files and refused
entries. Starting the node once recovers a torn file.

**Certificate.** The existing collection (`cut::collect_with`) over the replica's member
objects, with the pinned exclusions, must yield exactly the pinned target in the pinned
view; a pending collection, a conflicting cut for that number, or another cut or view is
refused. Nothing selects the latest cut or resolves a conflict, and the predecessor is
pinned, not collected. The segment carries every signature of that collection in member
order, so replicas holding different signature sets yield different segments; a signer
whose own extraction differs refuses the candidate and can instead verify the producer's
segment against their own pinned context.

**Extraction.** `Segment::build` takes each public writer's chain back from its target head
to the predecessor's length and verifies the whole segment contract (§Public archive
segments). Circle entries and objects never enter it. A missing public entry or object, an
incoherent chain, an invalid certificate or snapshot, or a segment over 32 MiB is refused;
nothing is omitted, fetched or repaired. The snapshot is checked against the target, not
produced: input and state roots remain T79's.

**Effects.** The segment is published at `segment_file` like the other offline outputs:
staged beside it, synced and linked without overwriting. The replica, feeds, keys, status
and archive store are not modified; the lock file is created if absent.

**Evidence.** AT-NET-34 extracts from a durable replica holding public, circle and later
member entries: an initial and a successor cut give exactly their public deltas, and the
successor's segment passes the real prepare, sign, combine and install commands. Missing
or altered entries and objects, torn files, wrong context, pending, conflicting or other-view
certificates, and closing cuts with a missing, mismatched or unexpected snapshot are
refused. The replica and existing outputs are unchanged, and on Unix a running node excludes
the command. The replica is written through `DurableReplica`, not by consortium processes.

Extraction from a running node, roster agreement and the independent-operator trial remain
open. T79/T88 and application rules are unchanged.

## Collected cut export from a stopped node's replica (T84, tenth increment)

One offline command writes a cut that a stopped node's replica collects, as the raw
`Cut::encode` bytes that `target_cut_file` and `previous_cut_file` take:

```text
isegoria-node --archive-export-cut <node.toml> <cut.toml> <cut-file>
```

`cut.toml` requires `number`, `view` and `excluded`: distinct indices of consortium members,
where empty is an explicit choice. It is a regular file of at most 1 MiB. Network id,
consortium, threshold and tiers come from the node's own tiered TOML. Nothing chooses the
latest cut, a view or exclusions, and each cut is exported on its own: a predecessor takes its
own number, view and exclusions, never a target's.

**Checks.** Replica access is that of §Segment extraction from a stopped node's replica: the
shared lock, `DurableReplica::read`, no recovery, no write and no network. The existing
collection (`cut::collect_with`) of `number` with the given exclusions must be ready in
exactly `view`; a pending collection, a conflicting cut or a cut collected in another view
is refused. That collection counts only `verify_strict` signatures, and the command then
applies the segment's own certificate check (`archive::verify_certificate`): `verify_strict`
signatures over the cut's checkpoint in that view, in member order, from at least the
threshold of distinct non-excluded members. A cut collected without such a quorum is
refused. Collection rules and signature sets are unchanged.

**Output.** On success the command prints the cut digest as lowercase hex and publishes the
canonical bytes at `<cut-file>` like every offline output: never over an existing file, and
an error after the link is an uncertain acknowledgement. Other errors print an `archive:`
diagnostic, exit with code 2 and publish nothing.

**What the file is.** The file holds the cut alone, without its signatures, view or
exclusions. It is neither a certificate nor a self-standing proof, and exporting it is no
trust decision: the operator still pins it, with its view and exclusions, as context. The
command establishes only that this replica collects the cut with a strict quorum under the
given view and exclusions. That the cut succeeds a given predecessor with a valid epoch, that
its writers are known and its public delta remain to segment verification against the pinned
context (§Public archive segments), at extraction and preparation. That the exclusions are the
beacon history's stays the operator's premise, as for every archive context; one replica
shows neither freshness nor other replicas' cuts.

**Evidence.** AT-NET-35 exports a predecessor and a target with different views and
exclusions, byte for byte their `Cut::encode`, and feeds them to `--archive-extract` and
`--archive-prepare`. It refuses an uncollected number, another view, a quorum lost to
exclusions or threshold, conflicting cuts, a quorum reached only by a signature that lenient
verification accepts and `verify_strict` refuses, as not collected (extraction too), missing or
invalid exclusions, an untiered configuration, missing or damaged replica files and an existing
output, without changing any file; on Unix a running node excludes the command. Three tiered
member processes then collect cuts 0 to 5, member 2 stopped by A from its start on a commit
of its key placed in every replica, so its cuts 2 and 5 go to a later view and, once it
withholds its reveal, cut 5 excludes it (AT-NET-39). Each cut is exported from member 0's
stopped replica with the number, view and exclusions of that member's status file and
matches the digest reported there; another view is refused; cuts 4 and 5, with their
reported views and exclusions, feed extraction and preparation as predecessor and target.

Agreement on contexts between operators (T88), roster agreement and the independent-operator
trial remain open. T79/T88 and application rules are unchanged.
