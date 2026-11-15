# 23 — Gate 2 on four machines: the runbook

| | |
|---|---|
| **Purpose** | What each machine does, command by command, in Milestone 2's gate 2 (`10` §2.4) as the owner set it up (dsmear, in chat, 2026-11-12): four machines he operates, each with an agent session that follows its role here. |
| **Scope** | A runbook. The procedure it follows is `04` §The trial's procedure, steps 1–11, and `04` §Gate 2's checklist; the answer review's side is `05` §Distributed integration. It changes no contract. |
| **Status** | **Written, not run.** The POSIX commands and `scripts/gate2-participants.sh` were run once on one machine over loopback (2026-11-12): one question to its outcome, the four digests equal. Nothing here has run on Windows or macOS, across machines or over a VPN. |

## Rules for every session

1. One operator, the owner, runs the whole gate: it shows no independence (gate 3 does). Do one
   step at a time, when the operator names it, and report what the commands printed.
2. **Secrets never leave the machine and are never printed**: `member.key`, `circle.key`,
   `hpke.key`, `transport.key`, `issuer.key`, a person's `pending`, `credential` and `commitment`
   files. Do not `cat` them, do not copy them into the exchange, do not quote them in a report.
3. Only what §The exchange lists goes into the exchange. A status file is shared only through
   the filter of §Evidence: its `epoch`, `question`, `request` and `request_state` lines are
   circle data.
4. An unexpected output, a non-zero exit, or a status line `conflict true`, `equivocations` other
   than 0, `duties_halted true`, a `stopped` or `refused_cut` other than `none`: stop, change
   nothing, report it. No retry beyond those written here, no fix to the code on the machine.
5. Every node keeps its data directory as it is at every stop (`04`, step 10).

## Machines and roles

| Machine | System | Roles | Listens on |
|---|---|---|---|
| `L` | Pop!_OS | member 0, the relay | `A_L`: 4001 (member), 4002 (relay) |
| `M` | macOS | member 1, the participants' commands | `A_M`: 4001 |
| `W1` | Windows, native | member 2, the test issuer's holder's tool | `A_W1`: 4001 |
| `W2` | Windows, native | member 3 | `A_W2`: 4001 |

`A_x` is machine `x`'s address on the private network, written in the exchange at stage 0. Two
roles on one machine fail together when that machine is cut off or stopped; the record states it.
WSL2 replaces native Windows on `W1` or `W2` only if stage 0 shows the node cannot run there, and
the record says which.

**Native Windows, as the code is (`node/src/lib.rs`, `network/src/store.rs`):** the data
directory's lock is not enforced, directories are not synced after a write, key files are written
without restricted permissions, and the node stops on Ctrl-C only. The node, the client and the
issuer have never run on Windows before this gate; CI runs only `scoring`'s golden test there.

## The exchange

A directory every machine holds a copy of, public material only: the branch
`trial/gate2-exchange` of the repository, checked out beside the code
(`git worktree add ../isegoria-exchange trial/gate2-exchange`), each machine committing only under
its own name and pulling before it reads. A machine that cannot push hands its files to the
operator.

| Path | Written by | Holds |
|---|---|---|
| `<machine>/address.txt` | each machine | its `A_x` |
| `<machine>/preliminary.txt` | each machine | stage 0's results |
| `<machine>/member.pub`, `L/relay.pub` | each node's machine | what `--new-identity` printed: four public keys |
| `W1/issuer.pub` | `W1` | the `issuer` line of `--new-key` |
| `genesis.bin`, `genesis-digest.txt` | `L` | the assembled configuration and what assembly printed |
| `<machine>/confirmation.txt` | each machine | what `--genesis-digest` printed there |
| `announce-<i>.txt` | member `i`'s machine | its `--announce` line |
| `relay-address.txt` | `L` | the relay's `listening on` address |
| `requests/`, `replies/` | `M`, then `W1` | issuance requests and the issuer's replies |
| `evidence/<machine>/<NN-stage>/` | each machine | its nodes' public status lines |
| `faults.txt` | the machine that injects | each fault, its command, its UTC time |

## Stage 0 — each machine, before the gate

1. Clone the repository at the revision the operator pins and build with the pinned toolchain
   (`rust-toolchain.toml`): `cargo build --release -p node -p client -p test-issuer`. On Windows
   this needs the MSVC build tools. The binaries are `target/release/isegoria-node`,
   `isegoria-client` and `isegoria-test-issuer` (`.exe` on Windows).
2. Run `cargo test -p node -p client -p test-issuer -p p2p` and write the result, with every
   failing test's name and message, in `preliminary.txt`. A failure is reported, not fixed.
3. The private network is Tailscale (owner, 2026-11-12): the operator installs it and signs the
   machine in to his tailnet; the session writes `tailscale ip -4` in `address.txt` and, once
   all four are there, pings the other three. On Linux the interface is `tailscale0`.
4. Allow inbound TCP on the listening ports. Windows, in an elevated PowerShell:
   `netsh advfirewall firewall add rule name="isegoria-node" dir=in action=allow protocol=TCP localport=4001`.
5. Choose a run directory outside the repository and the exchange, `RUN` below. In a TOML file
   write Windows paths with forward slashes (`C:/isegoria/run/m2`).

## Stage 1 — identities (`04`, step 1)

On each member's machine, `i` its index: `isegoria-node --new-identity RUN/m<i> > member.pub`.
On `L` also `isegoria-node --new-identity RUN/relay > relay.pub`. On `W1`:
`isegoria-test-issuer --new-key RUN/issuer/issuer.key`, its `issuer` line into `issuer.pub`.
Each output is four (or one) lines of public keys; copy the `.pub` files into the exchange.

## Stage 2 — the configuration (`04`, step 2)

1. `L` writes `RUN/genesis.toml` from the exchange's `.pub` files, members in index order:

   ```toml
   threshold = 3
   family = "answer-review/v1"
   issuer = "<issuer.pub: issuer>"
   relays = ["<relay.pub: transport>"]
   [[member]]
   consortium = "<member 0: consortium>"
   circle = "<member 0: circle>"
   hpke = "<member 0: hpke>"
   # … one [[member]] per member, 1 to 3
   ```

   then `isegoria-node --genesis-assemble RUN/genesis.toml RUN/genesis.bin > genesis-digest.txt`
   and puts both files in the exchange.
2. Every machine copies `genesis.bin` into its `RUN`, runs
   `isegoria-node --genesis-digest RUN/genesis.bin > confirmation.txt`, and checks that the file
   equals `genesis-digest.txt`, that it shows `threshold 3` and four `member` lines, and that its
   own keys stand at its own index (`L` its `relay` line, `W1` the `issuer` line). The four
   confirmations go into the exchange. One operator holding every machine, they prove no origin.
3. Each member's machine writes `RUN/m<i>.toml`, first with `hpke = []`:

   ```toml
   listen = ["/ip4/<A of this machine>/tcp/4001"]
   peers = ["/ip4/<A>/tcp/4001", …]          # the three other members
   key_file = "RUN/m<i>/member.key"
   circle_key_file = "RUN/m<i>/circle.key"
   hpke_key_file = "RUN/m<i>/hpke.key"
   relays = ["<relay.pub: transport>"]
   network_id = "<genesis-digest.txt: network_id>"
   genesis = "RUN/genesis.bin"
   family = "answer-review/v1"
   answer_config = "<genesis-digest.txt: answer_config_hex>"
   issuer = "<genesis-digest.txt: issuer>"
   data_dir = "RUN/m<i>"
   writers = [<the four consortium keys, then the four circle keys, in index order>]
   discovery_and_archive = false
   sync_every_ms = 500
   status_every_ms = 1000
   [consortium]
   members = [<the four consortium keys>]
   threshold = 3
   circle = [<the four circle keys>]
   hpke = []
   [member]
   index = <i>
   every_ms = 500
   patience = 10
   view_patience = 20
   ```

   then `isegoria-node --announce RUN/m<i>.toml > announce-<i>.txt` into the exchange. When the
   four are there, every member sets `hpke = [<the four announce lines, in index order>]`.
4. `L` writes `RUN/relay.toml`: `listen = ["/ip4/<A_L>/tcp/4002"]`, `peers` the four members'
   addresses, `relay = true`, then the lines from `network_id` through `[consortium]`'s `hpke`
   as a member's, with `data_dir = "RUN/relay"` and no key file and no `[member]`.

The timing values are this run's choice, five times gate 1's periods, and are stated in the
record; no parameter is calibrated (`14`).

## Stage 3 — first start (`04`, step 3)

Each node: `isegoria-node RUN/<its>.toml`, standard error kept in `RUN/node-<name>.log`. **On
Windows the operator starts it in a terminal of his own**, where Ctrl-C can stop it. The first
line of the log is `listening on <address>`; `L` writes the relay's into `relay-address.txt`.

A node's status is the file `status` in its data directory. Wait until every member's shows
`caught_up true`, `circle_caught_up true` and at least two `cut` lines, and the relay's
`bound 4`. Take evidence stage `01-started` (§Evidence).

## Stage 4 — credentials (`04`, step 4)

On `M`, write `RUN/participants.env` as the head of `scripts/gate2-participants.sh` says
(`DIR` under `RUN`, `EXCHANGE` the exchange), then:

1. `M`: `scripts/gate2-participants.sh requests` — fourteen files in `requests/`.
2. `W1`, for each slot `k` from 0 to 13:
   `isegoria-test-issuer --issue RUN/issuer/issuer.key requests/slot-<k>.request replies/slot-<k>.reply`,
   each printing `slot <k>`; it records the slots it answered.
3. `M`: `scripts/gate2-participants.sh finalize` — `credentials 14`.

## Stage 5 — the review, with the faults

The operator calls each line; `M` runs the script's steps, a member's machine its closure
requests, `isegoria-node --request RUN/m<i>.toml <command> <n>`, which prints the request's
file name. Members are named by index. After each line marked ▣ every machine takes an evidence
stage of that name.

| # | Who | What | Expected |
|---|---|---|---|
| 1 | `L` | *Delay.* `sudo tc qdisc add dev tailscale0 root netem delay 30ms 10ms loss 1%`, kept to the end; noted in `faults.txt` | `tc qdisc show` lists it |
| 2 | `M` | `gate2-participants.sh join` — thirteen joins, the second person's sent to two members | each `applied`; `p1 join` shows two entries, one `AlreadyInPool` |
| 3 | `M` | `gate2-participants.sh deposit 0 0` | `applied question 0` |
| 4 | 0, 1, 2 | `--request … close-epoch 0` | every member's status gains a `digest` line and `beacon 0 <value> []`; the four digests equal ▣ `02-epoch-0-closed` |
| 5 | `M` | once every member shows `question 0 committing`: `gate2-participants.sh drawn 0` | `drawn 12` |
| 6 | `W2` | *3–1.* `tailscale down` (member 3 apart); noted | the others' pings to `A_W2` fail |
| 7 | `M` | `gate2-participants.sh commit 0 0 1 2` | each commit `applied` |
| 8 | 0, 1, 2 | `--request … close-commits 0` | members 0–2 show `question 0 revealing`; member 3's `cut` lines do not grow ▣ `03-partition-3-1` |
| 9 | `W2` | *Rejoin.* `tailscale up`; noted | member 3 reaches the others' cuts and `question 0 revealing`; note how long it took ▣ `04-rejoined-3-1` |
| 10 | operator | *2–2.* In the tailnet's access controls, replace the policy's `acls` with the two rules below; noted with its UTC time | `L` cannot ping `A_W1` or `A_W2`; `M` neither |
| 11 | `M` | `gate2-participants.sh reveal first 0` | a `logged` receipt |
| 12 | all | wait 30 s, then compare each member's last `cut` line over the next 30 s | no member gains a cut made after the split |
| 13 | `M` | `gate2-participants.sh pending 0` | the reveal's entry `pending` ▣ `05-partition-2-2` |
| 14 | operator | *Rejoin.* Restore the saved policy; noted | cuts resume on all four ▣ `06-rejoined-2-2` |
| 15 | `W1` | *Crash.* `taskkill /F /IM isegoria-node.exe` (member 2); noted | the process is gone |
| 16 | `M` | `gate2-participants.sh reveal 0 1 3` | every reveal `applied`, the first one too |
| 17 | `W1` | start member 2 again; noted | it shows `caught_up true`, `circle_caught_up true`, the digest it had and the others' cuts ▣ `07-member-2-restarted` |
| 18 | `L` | *The relay away.* Stop the relay (SIGTERM); noted | — |
| 19 | `M` | `gate2-participants.sh author 0` | it fails |
| 20 | `L` | start the relay; noted. `M`: `gate2-participants.sh author 0` until it answers | `question 0 revealing` |
| 21 | 1, 2, 3 | `--request … close 0` | every member shows `question 0 closed` |
| 22 | `M` | `gate2-participants.sh author 3` | `question 0 closed accepted observation success 1 1 …` |
| 23 | 0, 1, 3 | `--request … close-epoch 1` | a second `digest` line and `beacon 1 <value> []` on each; the four equal ▣ `08-epoch-1-closed` |
| 24 | all | stop every node (SIGTERM; Ctrl-C on Windows); `L`: `sudo tc qdisc del dev tailscale0 root` | ▣ `09-stopped` |

Line 10's rules, the operator saving the policy he had before them; the four addresses are the
machines' own, each rule letting one side talk only within itself:

```json
"acls": [
  {"action": "accept", "src": ["<A_L>", "<A_M>"],   "dst": ["<A_L>:*", "<A_M>:*"]},
  {"action": "accept", "src": ["<A_W1>", "<A_W2>"], "dst": ["<A_W1>:*", "<A_W2>:*"]}
]
```

The pings of line 10 are the check that the rules took effect; if they still answer, the
partition did not happen and the run waits. If Windows cannot run the commands of lines 6, 9 and 15 as written, the session reports it and
the operator decides; nothing is improvised. With four machines the 3–1 partition is also the
checklist's one member disconnected and back. A step that does not give its expected result
stops the run (rule 4): the record then says where and why, and it is not a passed gate.

## Evidence

At each stage a machine writes, for each of its nodes, the node's public status lines into
`evidence/<machine>/<NN-stage>/<node>.status`: the lines whose first word is one of `caught_up`,
`circle_caught_up`, `peers`, `bound`, `equivocations`, `conflict`, `duties_halted`, `stopped`,
`synced`, `circle_synced`, `cut`, `beacon`, `digest`, `refused_cut` — the list of
`trial/tests/support/mod.rs`'s `PUBLIC`. POSIX:

```sh
grep -E '^(caught_up|circle_caught_up|peers|bound|equivocations|conflict|duties_halted|stopped|synced|circle_synced|cut|beacon|digest|refused_cut)( |$)' RUN/m<i>/status
```

The exchange also gets the pinned revision and whether tracked files differ from it
(`git rev-parse HEAD`, `git status --porcelain --untracked-files=no`), each machine's system and
`rustc --version`, and `faults.txt`. Logs are shared only after the operator has read them for
secrets (`04`, step 11). The persons' files, the reads' outputs and whole status files stay on
their machines.

After line 24 `L` checks, over the whole exchange, that no file holds a secret key of any
identity and that no evidence file holds the question's text or a line starting with `epoch`,
`question`, `request` or `request_state`; each machine checks its own keys.

## What the record will say

`16-milestone2-trial.md` gets gate 2's record from the exchange: the revision, the configuration
and its digest, the machines and systems, the faults with their times and lengths, what each
showed against `04` §Gate 2's checklist, and what it did not show — one operator, the relay and
the participants sharing members' machines, the tailnet every packet crossed, delay
injected on one machine only, the native-Windows limits above, synthetic participants, one
question, the test issuer (D65). The owner accepts or refuses the record.
