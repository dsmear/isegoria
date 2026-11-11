# Isegoria — Milestone 2 trial records

| | |
|---|---|
| **Purpose** | The record of each Milestone 2 gate run against a pinned revision and configuration (`10` Milestone 2): what ran, on what, what it showed and what it did not. |
| **Scope** | Records only. The procedure is `04` §The trial's procedure; gate 1's test is `05` §Gate 1 on one machine; decisions are in `01` (D55, D57, D60–D68). |
| **Status** | Gate 1 recorded and passed, and accepted by the owner (dsmear, in chat, 2026-11-11). Gates 2 and 3 not run. |

Each record keeps the shared evidence beside it in `evidence/milestone2/`, holding no secret and
no circle content (`04` §The trial's procedure, step 11; `01` D68). Private evidence stays with
its operator.

## Gate 1 — controlled processes on one machine (2026-11-11)

*What the gate asks* (`10` Milestone 2): persistence, replication, cuts and beacon agree; restarts
preserve the accepted history.

*Pinned revision.* `a842b9091e1c2fe575492ebc3097efa534aeaae5` (the merge of PR #19 into
`feat/phase2-runtime`), tracked files clean, from a fresh worktree.

*Machine and toolchain.* macOS 15.4 on arm64; `rustc 1.86.0 (05f9846f8 2025-03-31)`, the pinned
toolchain. One operator, every process on loopback.

*Command.* `scripts/gate1.sh` (`cargo test -p trial --test gate1 -- --nocapture`), started
2026-11-11T16:44:11Z; the test ran in 16.27 s after a 46 s build. Result: **passed**
(AT-T79-50–59).

*Configuration.* The genesis the run assembled: digest
`93efa5703818b90ab97c63e23f2425f1109ec6b52e80c3522293ed6d5922a177`, network id
`2675de0f9b054ae1135ef4a29a8d19c94a131330b6aed7edd81871ebf7653fb1`, version 1, four members,
threshold 3, one relay, the provisional test issuer (D65); timing and sizing as `05` §Gate 1 on
one machine lists them.

*What it showed.* The four members and the relay ran as separate processes of the binaries; 14
credentials were issued by files; the participants' events reached the cuts through the relay;
the operators' closures, the draw after the beacon, commits, reveals and the outcome followed,
with the author's record and next-epoch quota. Member 3 was stopped (SIGTERM) and restarted
mid-way while the other three progressed, and rebuilt the same history. At the end the four
members held the same 51 cuts, the same beacon values for rounds 0 and 1 with no withholder,
and the same state digests at both closing cuts:

| Closing cut | State digest (every member) |
|---|---|
| 21 | `c2d7d74fe17ec4b4cd368b714cc53ae05f36c822008b92900660542d156506a4` |
| 48 | `fb49851fa0c5cb1e10b0e80ba396afdec10d38583fd62cd618ab33b00bc2e25e` |

No member reported a conflict, an equivocation, halted duties or a refused cut.

*Shared evidence.* `evidence/milestone2/gate1-a842b90/`: `revision.txt`, the genesis and its
digest, and each node's public status lines at nine stages (started, joined and deposited, epoch
0 closed, member 3 stopped, reveals open with member 3 down, member 3 restarted, question 0
closed, epoch 1 closed, stopped). The test checks that it holds none of the run's secrets, no
question text, reviewer view, author record, event id or circle status line.

*What it does not show.* Independence: one operator held every key. Separate machines or
networks, delays, drops, duplicates beyond one event sent twice, partitions and crashes (the
restart was an orderly stop): gate 2. Independent operators and the committee's key: gate 3.
Participants were scripted and synthetic, one author and one question, below the trial's sizing
(D60); the issuer is the provisional test issuer (D65). The beacon's bias and stall limits, and
no Byzantine tolerance, are as `04` and `01` D57 state.

*Status.* Passed on the pinned revision; gate 1 is closed when the owner accepts this record.
