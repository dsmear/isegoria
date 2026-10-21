//! The snapshot of B-b's scorer's inputs (`docs/17` §14.10): every cut of the prefix, the ledger's
//! refusals and the proposed records apart; refused records keep the state; the register, the
//! cohorts and the counts over the valid items; truncations, determinism and the digest's limit.

use identity::credential::{Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::{cid, Cid};
use network::consortium::{Consortium, Member};
use network::cut::{Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica, WriterSet};
use protocol::cohort_scores::{
    score, Case, Cause, Cohort, Counts, Extra, Item, Outcome, Panelist, Report, Selection, Value,
};
use protocol::cohort_snapshot::{
    snapshot, CohortEntry, CutInput, Design, Error, Incoherence, Invalidity, Judged, Position,
    Proposed, Refused, ReviewerCounts, Slot, Snapshot,
};
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::gate::GateOutcome;
use protocol::ledger::{CutReport, Ledger, Refusal};
use protocol::lifecycle::{Event, Invalid};
use protocol::node::Rejection;
use protocol::review::commit;
use std::collections::{BTreeMap, BTreeSet};

const NET: [u8; 32] = [5; 32];
const U: u8 = 1;
const PANEL: [u8; 7] = [1, 2, 3, 4, 5, 6, 7];

fn nym(n: u8) -> Nym {
    Nym([n; 32])
}

fn at(cut: u64, slot: usize) -> Position {
    Position { cut, slot }
}

fn draft(tag: &str) -> Draft {
    Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"Gazzetta Ufficiale".to_vec(),
    }
}

fn item(tag: &str) -> Cid {
    draft(tag).content_id()
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

/// `tag`'s deposit in `epoch` by author `secret`, with a real proof for its context.
fn deposit(tag: &str, secret: u8, epoch: u64) -> NodeEvent {
    let issuer = issuer();
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    let author = pending.finalize(issuer.issue(&req).unwrap());
    let draft = draft(tag);
    let context = deposit_context(draft.content_id(), epoch);
    let proof = prove(&author, &issuer.public(), Role::Propose, &context);
    NodeEvent::Deposit {
        epoch,
        quota: 1,
        draft,
        proof,
    }
}

fn nonce(n: u8, item: Cid) -> [u8; 32] {
    cid(&[&[n][..], &item.0].concat()).0
}

fn commit_of(n: u8, p: f64, item: Cid) -> Event {
    Event::Commit {
        nym: nym(n),
        commitment: commit(p, &nonce(n, item), nym(n), item),
    }
}

fn reveal_of(n: u8, p: f64, item: Cid) -> Event {
    Event::Reveal {
        nym: nym(n),
        prob: p,
        nonce: nonce(n, item),
    }
}

/// Admission, the first panel `seats` (nym, report) assigned, committed, closed and revealed.
fn first_round(item: Cid, seats: &[(u8, f64)]) -> Vec<Event> {
    let mut e = vec![
        Event::Admit {
            seed_from_beacon: true,
        },
        Event::AssignReviewers {
            panel: seats.iter().map(|s| nym(s.0)).collect(),
            item,
        },
    ];
    e.extend(seats.iter().map(|&(n, p)| commit_of(n, p, item)));
    e.push(Event::CloseCommits);
    e.extend(seats.iter().map(|&(n, p)| reveal_of(n, p, item)));
    e
}

fn extra_round(item: Cid, seats: &[(u8, f64)]) -> Vec<Event> {
    let mut e = vec![Event::AssignExtraReviewers {
        panel: seats.iter().map(|s| nym(s.0)).collect(),
    }];
    e.extend(seats.iter().map(|&(n, p)| commit_of(n, p, item)));
    e.push(Event::CloseCommits);
    e.extend(seats.iter().map(|&(n, p)| reveal_of(n, p, item)));
    e
}

fn seats(panel: &[u8], p: f64) -> Vec<(u8, f64)> {
    panel.iter().map(|&n| (n, p)).collect()
}

fn decided(outcome: GateOutcome) -> Event {
    Event::Score { outcome }
}

fn rule(tag: &str) -> Cid {
    cid(format!("rule of {tag}").as_bytes())
}

fn group(tag: &str) -> Cid {
    cid(format!("group of {tag}").as_bytes())
}

fn design(tag: &str, design: Design) -> Proposed {
    Proposed::Design {
        item: item(tag),
        group: group(tag),
        design,
        rule: rule(tag),
    }
}

fn weights(epoch: u64, of: &[(u8, f64)]) -> Proposed {
    Proposed::Weights {
        epoch,
        weights: of.iter().map(|&(n, w)| (nym(n), w)).collect(),
    }
}

fn unit_weights(epoch: u64) -> Proposed {
    weights(epoch, &seats(&PANEL, 1.0))
}

fn terminal(tag: &str, outcome: Outcome, evidence: &str) -> Proposed {
    Proposed::Terminal {
        item: item(tag),
        group: group(tag),
        rule: rule(tag),
        outcome,
        references: vec![cid(evidence.as_bytes())],
        verified: true,
    }
}

fn draw(tag: &str, round: u64, selected: bool) -> Proposed {
    Proposed::Draw {
        item: item(tag),
        round,
        selected,
    }
}

/// A prefix built by hand, standing for the ledger's output: entry ids made up, digests of the
/// metadata alone, as a cut's digest covers no proposed record.
struct Log {
    cuts: Vec<CutInput>,
    seq: u64,
}

fn digest(number: u64, epoch: u64, closes: bool) -> [u8; 32] {
    let bytes = [number.to_le_bytes(), epoch.to_le_bytes()].concat();
    cid(&[&bytes[..], &[u8::from(closes)]].concat()).0
}

impl Log {
    fn new() -> Log {
        Log {
            cuts: Vec::new(),
            seq: 0,
        }
    }

    fn cut(&mut self, epoch: u64, closes: bool) -> &mut Self {
        let number = self.cuts.len() as u64;
        self.cuts.push(CutInput {
            number,
            epoch,
            closes,
            digest: digest(number, epoch, closes),
            slots: Vec::new(),
            refused: Vec::new(),
        });
        self
    }

    fn last(&mut self) -> &mut CutInput {
        self.cuts.last_mut().unwrap()
    }

    fn here(&self) -> Position {
        let c = self.cuts.last().unwrap();
        at(c.number, c.slots.len() - 1)
    }

    fn id(&mut self, object: &[u8]) -> EntryId {
        self.seq += 1;
        EntryId {
            writer: [9; 32],
            seq: self.seq,
            hash: cid(object).0,
        }
    }

    fn object(&mut self, object: Vec<u8>) -> &mut Self {
        let id = self.id(&object);
        self.last().slots.push(Slot::Applied { id, object });
        self
    }

    fn event(&mut self, e: NodeEvent) -> &mut Self {
        self.object(e.encode())
    }

    fn deposit(&mut self, tag: &str, secret: u8) -> &mut Self {
        let epoch = self.cuts.last().unwrap().epoch;
        self.event(deposit(tag, secret, epoch))
    }

    fn steps(&mut self, tag: &str, events: Vec<Event>) -> &mut Self {
        for event in events {
            self.event(NodeEvent::Step {
                item: item(tag),
                event,
            });
        }
        self
    }

    fn record(&mut self, r: Proposed) -> &mut Self {
        self.last().slots.push(Slot::Proposed(r));
        self
    }
}

fn snap(cuts: &[CutInput]) -> Snapshot {
    snapshot(cuts).expect("a coherent prefix")
}

fn case_of(s: &Snapshot, n: u8, scorer_item: usize) -> Case {
    let a = s.scores.assignments.iter();
    a.filter(|a| a.nym == nym(n) && a.item == scorer_item)
        .map(|a| a.case)
        .next()
        .unwrap()
}

fn cohort_of(s: &Snapshot, n: u8, epoch: u64) -> &CohortEntry {
    s.cohorts
        .iter()
        .find(|c| c.nym == nym(n) && c.epoch == epoch)
        .unwrap()
}

fn value_of(s: &Snapshot, n: u8, epoch: u64) -> Option<&Value> {
    let k = cohort_of(s, n, epoch).scored?;
    Some(&s.scores.cohorts[k].value)
}

fn refused(s: &Snapshot) -> Vec<(Position, Option<Refused>)> {
    s.records.iter().map(|j| (j.at, j.refused)).collect()
}

// The real ledger: writers 0–2 are the consortium members' keys, 3–4 relays.

fn members() -> Vec<Member> {
    (0..3).map(|i| Member::from_seed([10 + i; 32])).collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

fn writers() -> Vec<FeedWriter> {
    [10u8, 11, 12, 20, 21]
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
        .collect()
}

fn signed(cut: &Cut) -> Vec<(usize, ed25519_dalek::Signature)> {
    let cp = cut.checkpoint(NET, consortium().member_set_hash());
    members()[..2]
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.sign(&cp)))
        .collect()
}

struct Net {
    replica: Replica,
    ws: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    ledger: Ledger,
    last: Option<Cut>,
}

impl Net {
    fn new() -> Net {
        let ws = writers();
        let keys: Vec<_> = ws.iter().map(|w| w.public()).collect();
        Net {
            replica: Replica::new(WriterSet::new(NET, &keys)),
            logs: (0..ws.len()).map(|_| TransparencyLog::new()).collect(),
            ws,
            ledger: Ledger::new(NET, consortium(), issuer().public()),
            last: None,
        }
    }

    fn write(&mut self, w: usize, object: Vec<u8>) -> EntryId {
        let entry = self.ws[w].sign(&self.logs[w].append(cid(&object)).clone());
        let id = entry.id();
        self.replica.insert(entry, object).unwrap();
        id
    }

    fn steps(&mut self, w: usize, tag: &str, events: Vec<Event>) {
        for event in events {
            let step = NodeEvent::Step {
                item: item(tag),
                event,
            };
            self.write(w, step.encode());
        }
    }

    /// The next cut, applied by the real ledger, as the snapshot reads it, with `records` placed
    /// at the given slots, in ascending order.
    fn cut(&mut self, epoch: u64, closes: bool, records: Vec<(usize, Proposed)>) -> CutInput {
        let cut = Cut::next(&self.replica, self.last.as_ref(), epoch, closes);
        let report: CutReport = self
            .ledger
            .apply(&self.replica, &cut, &signed(&cut))
            .unwrap();
        let mut slots: Vec<Slot> = report
            .applied
            .iter()
            .map(|id| Slot::Applied {
                id: *id,
                object: self.replica.get(id).unwrap().1.to_vec(),
            })
            .collect();
        for (k, r) in records {
            slots.insert(k, Slot::Proposed(r));
        }
        self.last = Some(cut.clone());
        CutInput {
            number: cut.number,
            epoch: cut.epoch,
            closes: cut.closes,
            digest: cut.digest(),
            slots,
            refused: report.refused,
        }
    }
}

const J1: [(u8, f64); 7] = [
    (1, 0.8),
    (2, 0.6),
    (3, 0.4),
    (4, 0.5),
    (5, 0.7),
    (6, 0.3),
    (7, 0.5),
];
const J2: [(u8, f64); 7] = [
    (1, 0.3),
    (2, 0.5),
    (3, 0.9),
    (4, 0.1),
    (5, 0.6),
    (6, 0.5),
    (7, 0.2),
];
const J2_EXTRA: [(u8, f64); 3] = [(8, 0.9), (9, 0.2), (10, 0.6)];
const W: [(u8, f64); 7] = [
    (1, 1.0),
    (2, 2.0),
    (3, 1.0),
    (4, 0.5),
    (5, 1.0),
    (6, 1.0),
    (7, 3.0),
];

/// j1 decided `Pass`, j2 through the band and `Resolve { Pass }`, by the real ledger in epoch 0;
/// weights and A's designs before the assignments, terminals `A` and `R` in a closing cut.
fn valid_path() -> Vec<CutInput> {
    let mut net = Net::new();
    net.write(3, deposit("j1", 2, 0).encode());
    net.write(3, deposit("j2", 3, 0).encode());
    let cut0 = net.cut(
        0,
        false,
        vec![
            (2, weights(0, &W)),
            (3, design("j1", Design::A)),
            (4, design("j2", Design::A)),
        ],
    );
    let mut j1 = first_round(item("j1"), &J1);
    j1.push(decided(GateOutcome::Pass));
    net.steps(0, "j1", j1);
    let mut j2 = first_round(item("j2"), &J2);
    j2.push(decided(GateOutcome::SupplementaryReview));
    j2.extend(extra_round(item("j2"), &J2_EXTRA));
    j2.push(Event::Resolve {
        outcome: GateOutcome::Pass,
    });
    net.steps(0, "j2", j2);
    let cut1 = net.cut(0, false, Vec::new());
    let cut2 = net.cut(
        0,
        true,
        vec![
            (0, terminal("j1", Outcome::A, "e1")),
            (1, terminal("j2", Outcome::R, "e2")),
        ],
    );
    vec![cut0, cut1, cut2]
}

/// A1 `17` §14.10: the real ledger's prefix maps to the items and cohorts built by hand.
#[test]
fn the_ledger_s_applied_prefix_maps_to_the_items_built_by_hand() {
    let cuts = valid_path();
    assert_eq!(cuts[1].slots.len(), 45);
    assert!(cuts[2].slots.iter().all(|s| matches!(s, Slot::Proposed(_))));
    let s = snap(&cuts);
    let weight = |n: u8| W.iter().find(|w| w.0 == n).unwrap().1;
    let first = |seats: &[(u8, f64)]| -> Vec<Panelist> {
        let seat = |&(n, p): &(u8, f64)| Panelist {
            nym: nym(n),
            report: Report::Revealed(p),
            weight: weight(n),
        };
        seats.iter().map(seat).collect()
    };
    let hand = vec![
        Item {
            first: first(&J1),
            extra: Vec::new(),
            frozen: true,
            selection: Selection::Selected {
                inclusion: 1.0,
                outcome: Some(Outcome::A),
            },
        },
        Item {
            first: first(&J2),
            extra: J2_EXTRA
                .iter()
                .map(|&(n, p)| Extra {
                    nym: nym(n),
                    report: Report::Revealed(p),
                })
                .collect(),
            frozen: true,
            selection: Selection::Selected {
                inclusion: 1.0,
                outcome: Some(Outcome::R),
            },
        },
    ];
    let mut cohorts: Vec<Cohort> = (1..=7)
        .map(|n| Cohort {
            nym: nym(n),
            items: vec![0, 1],
        })
        .collect();
    cohorts.extend((8..=10).map(|n| Cohort {
        nym: nym(n),
        items: vec![1],
    }));
    assert_eq!(s.inputs, hand);
    assert_eq!(s.scorer_cohorts, cohorts);
    assert_eq!(s.scores, score(&hand, &cohorts).unwrap());
    assert_eq!((s.cut, s.digest), (2, cuts[2].digest));
    assert_eq!(s.scored, vec![0, 1]);
    assert_eq!(
        s.items
            .iter()
            .map(|i| (i.cid, i.assigned, i.epoch, i.frozen, i.status.clone()))
            .collect::<Vec<_>>(),
        vec![
            (item("j1"), at(1, 1), 0, Some(at(1, 17)), Ok(0)),
            (item("j2"), at(1, 19), 0, Some(at(1, 44)), Ok(1)),
        ]
    );
    assert_eq!(s.register.len(), 17);
    let extra: Vec<_> = s.register.iter().filter(|a| a.extra).collect();
    assert_eq!(extra.len(), 3);
    assert!(extra.iter().all(|a| a.item == 1 && a.at == at(1, 36)));
    let ids = |rs: &[Proposed]| rs.iter().map(Proposed::id).collect::<Vec<_>>();
    let placed = vec![
        weights(0, &W),
        design("j1", Design::A),
        design("j2", Design::A),
        terminal("j1", Outcome::A, "e1"),
        terminal("j2", Outcome::R, "e2"),
    ];
    let judged: Vec<Judged> = [at(0, 2), at(0, 3), at(0, 4), at(2, 0), at(2, 1)]
        .iter()
        .zip(ids(&placed))
        .map(|(&at, id)| Judged {
            at,
            id,
            refused: None,
        })
        .collect();
    assert_eq!(s.records, judged);
    assert_eq!(
        s.counts(nym(U)),
        ReviewerCounts {
            assignments: 2,
            unscored: 0,
            valid: Counts { n: 2, o: 2, v: 2 },
        }
    );
    assert_eq!(
        s.counts(nym(8)),
        ReviewerCounts {
            assignments: 1,
            unscored: 0,
            valid: Counts { n: 1, o: 1, v: 1 },
        }
    );
    assert!(s.cohorts.iter().all(|c| c.closed && c.invalid.is_empty()));
    assert!(s.refusals.is_empty());
}

/// A1 `17` §14.10: the ledger's refusals are listed, change no state, and the snapshot is the same.
#[test]
fn the_ledger_s_refusals_are_listed_and_change_nothing() {
    let mut net = Net::new();
    net.write(3, deposit("j1", 2, 0).encode());
    let cut0 = net.cut(
        0,
        false,
        vec![(1, unit_weights(0)), (2, design("j1", Design::A))],
    );
    let j1 = item("j1");
    let all = seats(&PANEL, 0.5);
    let mut steps = first_round(j1, &all);
    steps.retain(|e| !matches!(e, Event::Reveal { nym: n, .. } if *n == nym(U)));
    net.steps(0, "j1", steps);
    let forged = Event::Reveal {
        nym: nym(U),
        prob: 0.5,
        nonce: [0; 32],
    };
    let bad = net.write(
        0,
        NodeEvent::Step {
            item: j1,
            event: forged,
        }
        .encode(),
    );
    let outside = NodeEvent::Step {
        item: j1,
        event: reveal_of(U, 0.5, j1),
    };
    let relay = net.write(4, outside.encode());
    let cut1 = net.cut(0, false, Vec::new());
    let cut2 = net.cut(0, true, Vec::new());
    let mismatch = Refusal::Rejected(Rejection::Lifecycle(Invalid::RevealMismatch));
    assert_eq!(cut1.refused.len(), 2);
    assert!(cut1.refused.contains(&(bad, mismatch)));
    assert!(cut1.refused.contains(&(relay, Refusal::NotAuthorized)));
    let supplied: Vec<(u64, EntryId)> = cut1.refused.iter().map(|(id, _)| (1, *id)).collect();
    let without: Vec<CutInput> = [&cut0, &cut1, &cut2]
        .iter()
        .map(|c| CutInput {
            number: c.number,
            epoch: c.epoch,
            closes: c.closes,
            digest: c.digest,
            slots: c.slots.clone(),
            refused: Vec::new(),
        })
        .collect();
    let with = snap(&[cut0, cut1, cut2]);
    let mut plain = snap(&without);
    assert_eq!(with.refusals, supplied);
    assert!(plain.refusals.is_empty());
    plain.refusals = with.refusals.clone();
    assert_eq!(with, plain);
    assert_eq!(with.inputs[0].first[0].report, Report::Open);
    assert!(!with.inputs[0].frozen);
    assert_eq!(case_of(&with, U, 0), Case::Unfrozen);
    assert_eq!(
        value_of(&with, U, 0),
        Some(&Value::Unavailable(vec![(0, Cause::Unfrozen)]))
    );
}

/// The base of the incoherent prefixes: j1 deposited, weighted, designed and assigned in cut 0.
fn base() -> Log {
    let mut log = Log::new();
    log.cut(0, false)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::A));
    let assign = first_round(item("j1"), &seats(&PANEL, 0.5))[..2].to_vec();
    log.steps("j1", assign);
    log
}

/// A1 `17` §14.10: an input outside the checked preconditions gives no snapshot, its first failure.
#[test]
fn incoherent_prefixes_give_no_snapshot() {
    let incoherent = |cuts: &[CutInput]| match snapshot(cuts) {
        Err(Error::Incoherent(i)) => i,
        other => panic!("expected an incoherent prefix, got {other:?}"),
    };
    assert_eq!(incoherent(&[]), Incoherence::NoCut);
    let mut log = base();
    log.cut(0, false);
    log.last().number = 2;
    let found = incoherent(&log.cuts);
    assert_eq!(
        found,
        Incoherence::Number {
            found: 2,
            expected: 1
        }
    );
    let mut log = Log::new();
    log.cut(1, false).cut(0, false);
    assert_eq!(incoherent(&log.cuts), Incoherence::Epoch { cut: 1 });
    let mut log = Log::new();
    log.cut(0, true).cut(0, false);
    assert_eq!(incoherent(&log.cuts), Incoherence::Epoch { cut: 1 });
    let mut log = base();
    log.steps(
        "j1",
        vec![commit_of(U, 0.5, item("j1")), Event::CloseCommits],
    );
    let forged = Event::Reveal {
        nym: nym(U),
        prob: 0.5,
        nonce: [0; 32],
    };
    log.steps("j1", vec![forged]);
    let lifecycle = Incoherence::Lifecycle(at(0, 7), Invalid::RevealMismatch);
    assert_eq!(incoherent(&log.cuts), lifecycle);
    let mut log = base();
    let Slot::Applied { id, object } = log.cuts[0].slots[0].clone() else {
        unreachable!()
    };
    log.cut(0, false);
    log.last().slots.push(Slot::Applied {
        id,
        object: object.clone(),
    });
    assert_eq!(incoherent(&log.cuts), Incoherence::Repeated(id));
    let mut log = base();
    let Slot::Applied { id: own, .. } = log.cuts[0].slots[0].clone() else {
        unreachable!()
    };
    log.last().refused.push((own, Refusal::NotAuthorized));
    assert_eq!(incoherent(&log.cuts), Incoherence::AppliedAndRefused(own));
    let mut log = base();
    log.object(b"not an event".to_vec());
    assert_eq!(incoherent(&log.cuts), Incoherence::Undecodable(at(0, 5)));
    let mut log = base();
    log.steps("j9", vec![Event::CloseCommits]);
    assert_eq!(incoherent(&log.cuts), Incoherence::UnknownItem(at(0, 5)));
    let mut log = base();
    log.deposit("j1", 2);
    assert_eq!(incoherent(&log.cuts), Incoherence::Redeposited(at(0, 5)));
    let mut log = Log::new();
    log.cut(0, false).deposit("j1", 2).steps(
        "j1",
        vec![
            Event::Admit {
                seed_from_beacon: true,
            },
            Event::AssignReviewers {
                panel: PANEL.iter().map(|&n| nym(n)).collect(),
                item: item("j2"),
            },
        ],
    );
    assert_eq!(incoherent(&log.cuts), Incoherence::ItemMismatch(at(0, 2)));
}

/// j1 (A, reviewed, no terminal) in epoch 0; j2 (no design) in epoch 1; j3 (no weights) in 2.
fn absent() -> Vec<CutInput> {
    let mut log = Log::new();
    let decided_round = |tag: &str| {
        let mut e = first_round(item(tag), &seats(&PANEL, 0.5));
        e.push(decided(GateOutcome::Pass));
        e
    };
    log.cut(0, false)
        .deposit("j1", 2)
        .deposit("j2", 3)
        .deposit("j3", 4)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .record(design("j3", Design::A))
        .steps("j1", decided_round("j1"));
    log.cut(1, false)
        .record(unit_weights(1))
        .steps("j2", decided_round("j2"));
    log.cut(2, true).steps(
        "j3",
        first_round(item("j3"), &seats(&PANEL, 0.5))[..2].to_vec(),
    );
    log.cuts
}

/// A1 `17` §14.10: no terminal is pending, no design or weights an invalid item, never imputed.
#[test]
fn absent_records_leave_an_item_pending_or_invalid() {
    let s = snap(&absent());
    let status: Vec<_> = s.items.iter().map(|i| i.status.clone()).collect();
    assert_eq!(
        status,
        vec![
            Ok(0),
            Err(vec![Invalidity::NoDesign]),
            Err(vec![Invalidity::NoWeights])
        ]
    );
    assert_eq!(case_of(&s, U, 0), Case::Pending);
    assert_eq!(
        value_of(&s, U, 0),
        Some(&Value::Bound {
            sum: Ok((-1.0, 1.0)),
            mean: (-1.0, 1.0),
        })
    );
    for (epoch, member) in [(1, 1), (2, 2)] {
        let c = cohort_of(&s, U, epoch);
        assert_eq!(
            (c.members.clone(), c.closed, c.invalid.clone(), c.scored),
            (vec![member], true, vec![member], None)
        );
    }
    assert_eq!(
        s.counts(nym(U)),
        ReviewerCounts {
            assignments: 3,
            unscored: 2,
            valid: Counts { n: 1, o: 0, v: 0 },
        }
    );
}

/// A1 `17` §14.10: late, duplicate, conflicting or malformed records are refused, the state kept.
#[test]
fn late_duplicate_and_conflicting_records_are_refused_keeping_the_state() {
    let mut log = Log::new();
    let panel3 = [1, 2, 3, 4, 5, 6, 8];
    let panel4 = [1, 2, 3, 4, 5, 6, 9];
    let assign = |panel: &[u8], tag: &str| first_round(item(tag), &seats(panel, 0.5))[..2].to_vec();
    let mut w0 = seats(&PANEL, 1.0);
    w0.push((9, -1.0));
    log.cut(0, false);
    for (k, tag) in ["j1", "j2", "j3", "j4", "j5"].iter().enumerate() {
        log.deposit(tag, 2 + k as u8);
    }
    let mut expected = Vec::new();
    let mut place = |log: &mut Log, r: Proposed, why: Option<Refused>| {
        log.record(r);
        expected.push((log.here(), why));
    };
    place(&mut log, weights(0, &w0), None);
    let w0_at = log.here();
    place(&mut log, design("j1", Design::A), None);
    let d1 = log.here();
    let dup = Some(Refused::Duplicate { first: d1 });
    place(&mut log, design("j1", Design::A), dup);
    let conflict = Some(Refused::Conflict { accepted: d1 });
    place(
        &mut log,
        design("j1", Design::C { inclusion: 0.5 }),
        conflict,
    );
    let c15 = Design::C { inclusion: 1.5 };
    place(&mut log, design("j2", c15), Some(Refused::Inclusion));
    place(&mut log, design("j3", Design::A), None);
    place(&mut log, design("j4", Design::A), None);
    place(&mut log, design("j5", Design::A), None);
    log.steps("j1", assign(&PANEL, "j1"))
        .steps("j2", assign(&PANEL, "j2"));
    place(&mut log, design("j2", Design::A), Some(Refused::Late));
    log.steps("j3", assign(&panel3, "j3"))
        .steps("j4", assign(&panel4, "j4"));
    let other = Some(Refused::Conflict { accepted: w0_at });
    place(&mut log, weights(0, &seats(&PANEL, 2.0)), other);
    log.cut(1, false).steps("j5", assign(&PANEL, "j5"));
    place(&mut log, unit_weights(1), Some(Refused::Late));
    log.cut(1, true);
    let s = snap(&log.cuts);
    assert_eq!(refused(&s), expected);
    let status: Vec<_> = s.items.iter().map(|i| i.status.clone()).collect();
    assert_eq!(
        status,
        vec![
            Ok(0),
            Err(vec![Invalidity::NoDesign]),
            Err(vec![Invalidity::Unweighted(nym(8))]),
            Err(vec![Invalidity::Weight(nym(9))]),
            Err(vec![Invalidity::NoWeights]),
        ]
    );
    assert_eq!(
        s.inputs[0].selection,
        Selection::Selected {
            inclusion: 1.0,
            outcome: None
        }
    );
    assert!(s.inputs[0].first.iter().all(|p| p.weight == 1.0));
    assert_eq!(case_of(&s, U, 0), Case::Unfrozen);
}

/// A1 `17` §14.10: an extra-round assignment starts its epoch; a later weights record is late.
#[test]
fn a_weights_record_after_an_extra_assignment_of_its_epoch_is_late() {
    let mut log = Log::new();
    let mut e = first_round(item("j1"), &seats(&PANEL, 0.5));
    e.push(decided(GateOutcome::SupplementaryReview));
    log.cut(0, false)
        .deposit("j1", 2)
        .deposit("j2", 3)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .record(design("j2", Design::A))
        .steps("j1", e);
    log.cut(1, false)
        .steps("j1", extra_round(item("j1"), &seats(&[8, 9, 10], 0.5)))
        .record(unit_weights(1))
        .steps(
            "j2",
            first_round(item("j2"), &seats(&PANEL, 0.5))[..2].to_vec(),
        );
    let s = snap(&log.cuts);
    assert_eq!(
        refused(&s),
        vec![
            (at(0, 2), None),
            (at(0, 3), None),
            (at(0, 4), None),
            (at(1, 8), Some(Refused::Late)),
        ]
    );
    let status: Vec<_> = s.items.iter().map(|i| i.status.clone()).collect();
    assert_eq!(status, vec![Ok(0), Err(vec![Invalidity::NoWeights])]);
    let extra = cohort_of(&s, 8, 1);
    assert_eq!((extra.members.clone(), extra.closed), (vec![0], false));
    assert_eq!(
        s.register
            .iter()
            .filter(|a| a.extra && a.at == at(1, 0))
            .count(),
        3
    );
}

/// j1 under C at ½: a draw before `Φ`, the freeze, a later draw of another round, a terminal `A`.
fn drawn() -> Vec<CutInput> {
    let mut log = Log::new();
    log.cut(0, false)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::C { inclusion: 0.5 }))
        .steps("j1", first_round(item("j1"), &J1))
        .record(draw("j1", 1, true));
    log.cut(0, false)
        .steps("j1", vec![decided(GateOutcome::Pass)]);
    log.cut(0, false).record(draw("j1", 2, true));
    log.cut(0, true).record(terminal("j1", Outcome::A, "e1"));
    log.cuts
}

/// A1 `17` §14.10: a draw before `Φ_j` is refused, the item unfrozen; a later distinct draw counts.
#[test]
fn a_draw_before_the_freeze_is_refused_and_a_later_one_counts() {
    let cuts = drawn();
    let cases: Vec<Case> = (0..4).map(|n| case_of(&snap(&cuts[..=n]), U, 0)).collect();
    assert_eq!(
        cases,
        vec![Case::Unfrozen, Case::Undrawn, Case::Pending, Case::Positive]
    );
    let s = snap(&cuts);
    assert_eq!(
        refused(&s),
        vec![
            (at(0, 1), None),
            (at(0, 2), None),
            (at(0, 20), Some(Refused::BeforeFreeze)),
            (at(2, 0), None),
            (at(3, 0), None),
        ]
    );
    assert_eq!(
        s.inputs[0].selection,
        Selection::Selected {
            inclusion: 0.5,
            outcome: Some(Outcome::A)
        }
    );
    assert!(matches!(value_of(&s, U, 0), Some(Value::Final(_))));
    assert_eq!(value_of(&snap(&cuts[..3]), U, 0), None);
}

/// A1 `17` §14.10: a draw needs C's design and no non-selection at certainty; the state kept.
#[test]
fn a_draw_needs_c_s_design_and_no_non_selection_at_certainty() {
    let mut log = Log::new();
    let decided_round = |tag: &str| {
        let mut e = first_round(item(tag), &seats(&PANEL, 0.5));
        e.push(decided(GateOutcome::Pass));
        e
    };
    log.cut(0, false)
        .deposit("j1", 2)
        .deposit("j2", 3)
        .record(unit_weights(0))
        .record(design("j1", Design::C { inclusion: 1.0 }))
        .record(design("j2", Design::A))
        .steps("j1", decided_round("j1"))
        .steps("j2", decided_round("j2"));
    log.cut(0, true);
    for r in [
        draw("j1", 1, false),
        draw("j2", 1, true),
        draw("j9", 1, true),
    ] {
        log.record(r);
    }
    let undrawn = snap(&log.cuts);
    log.record(draw("j1", 2, true));
    let s = snap(&log.cuts);
    assert_eq!(
        refused(&s)[3..],
        [
            (at(1, 0), Some(Refused::NotSelectedAtCertainty)),
            (at(1, 1), Some(Refused::Shape)),
            (at(1, 2), Some(Refused::NoDesign)),
            (at(1, 3), None),
        ]
    );
    assert_eq!(undrawn.inputs[0].selection, Selection::Undrawn);
    assert_eq!(case_of(&undrawn, U, 0), Case::Undrawn);
    assert_eq!(
        s.inputs[0].selection,
        Selection::Selected {
            inclusion: 1.0,
            outcome: None
        }
    );
    assert_eq!(
        s.inputs[1].selection,
        Selection::Selected {
            inclusion: 1.0,
            outcome: None
        }
    );
}

/// j1 under A: a terminal before `Φ`, the freeze, a valid `A`, a different `R`, a copy of `A`;
/// j2 under A, frozen, given invalid terminals; j3 under C at ½, frozen, undrawn, given one.
fn terminals() -> Vec<CutInput> {
    let mut log = Log::new();
    let decided_round = |tag: &str| {
        let mut e = first_round(item(tag), &seats(&PANEL, 0.5));
        e.push(decided(GateOutcome::Pass));
        e
    };
    log.cut(0, false)
        .deposit("j1", 2)
        .deposit("j2", 3)
        .deposit("j3", 4)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .record(design("j2", Design::A))
        .record(design("j3", Design::C { inclusion: 0.5 }))
        .steps("j1", first_round(item("j1"), &J1))
        .record(terminal("j1", Outcome::A, "early"));
    log.cut(0, false)
        .steps("j1", vec![decided(GateOutcome::Pass)])
        .steps("j2", decided_round("j2"))
        .steps("j3", decided_round("j3"));
    let unreferenced = Proposed::Terminal {
        item: item("j2"),
        group: group("j2"),
        rule: rule("j2"),
        outcome: Outcome::I,
        references: Vec::new(),
        verified: true,
    };
    let failed = Proposed::Terminal {
        item: item("j2"),
        group: group("j2"),
        rule: rule("j2"),
        outcome: Outcome::I,
        references: vec![cid(b"e2")],
        verified: false,
    };
    let foreign = Proposed::Terminal {
        item: item("j2"),
        group: group("j2"),
        rule: rule("j1"),
        outcome: Outcome::A,
        references: vec![cid(b"e2")],
        verified: true,
    };
    log.cut(0, false)
        .record(terminal("j1", Outcome::A, "e1"))
        .record(terminal("j1", Outcome::R, "e1"))
        .record(terminal("j1", Outcome::A, "e1"))
        .record(unreferenced)
        .record(failed)
        .record(foreign)
        .record(terminal("j3", Outcome::A, "e3"));
    log.cut(0, true);
    log.cuts
}

/// A1 `17` §14.10: a premature, second or invalid terminal is refused, the state before it kept.
#[test]
fn a_premature_second_or_invalid_terminal_keeps_the_state_before_it() {
    let cuts = terminals();
    let cases: Vec<Case> = (0..4).map(|n| case_of(&snap(&cuts[..=n]), U, 0)).collect();
    assert_eq!(
        cases,
        vec![
            Case::Unfrozen,
            Case::Pending,
            Case::Positive,
            Case::Positive
        ]
    );
    let s = snap(&cuts);
    let valid = at(2, 0);
    assert_eq!(
        refused(&s)[4..],
        [
            (at(0, 24), Some(Refused::BeforeFreeze)),
            (valid, None),
            (at(2, 1), Some(Refused::Conflict { accepted: valid })),
            (at(2, 2), Some(Refused::Duplicate { first: valid })),
            (at(2, 3), Some(Refused::Unreferenced)),
            (at(2, 4), Some(Refused::Unverified)),
            (at(2, 5), Some(Refused::Rule)),
            (at(2, 6), Some(Refused::Unselected)),
        ]
    );
    assert_eq!(case_of(&s, U, 1), Case::Pending);
    assert_eq!(case_of(&s, U, 2), Case::Undrawn);
    assert_eq!(
        s.inputs[0].selection,
        Selection::Selected {
            inclusion: 1.0,
            outcome: Some(Outcome::A)
        }
    );
}

fn grouped_terminal() -> Log {
    let mut log = Log::new();
    log.cut(0, true)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .steps("j1", first_round(item("j1"), &J1))
        .steps("j1", vec![decided(GateOutcome::Pass)]);
    log.cut(1, false);
    log
}

/// A1 `17` §14.10: a matching rule cannot admit a terminal declaring another group.
#[test]
fn a_terminal_with_the_right_rule_and_wrong_group_keeps_the_previous_state() {
    let mut log = grouped_terminal();
    let mut wrong = terminal("j1", Outcome::A, "e1");
    if let Proposed::Terminal {
        group: declared, ..
    } = &mut wrong
    {
        *declared = group("j2");
    }
    let prior = snap(&log.cuts);
    log.record(wrong.clone());
    let mut rejected = snap(&log.cuts);
    assert_eq!(
        rejected.records.last().unwrap().refused,
        Some(Refused::Group)
    );
    assert_eq!(case_of(&rejected, U, 0), Case::Pending);
    assert_eq!(rejected.records.pop().unwrap().id, wrong.id());
    assert_eq!(rejected, prior);
    log.record(wrong);
    log.record(terminal("j1", Outcome::A, "e1"));
    let accepted = snap(&log.cuts);
    assert_eq!(
        accepted.records[3].refused,
        Some(Refused::Duplicate { first: at(1, 0) })
    );
    assert_eq!(accepted.records[4].refused, None);
    assert_eq!(case_of(&accepted, U, 0), Case::Positive);
    invariants(&log.cuts);
}

/// A1 `17` §14.10: a terminal with the declared item, group and rule is accepted.
#[test]
fn a_terminal_with_the_declared_group_and_rule_is_accepted() {
    let mut log = grouped_terminal();
    let record = terminal("j1", Outcome::A, "e1");
    log.record(record.clone());
    let s = snap(&log.cuts);
    assert_eq!(
        s.records.last(),
        Some(&Judged {
            at: at(1, 0),
            id: record.id(),
            refused: None
        })
    );
    assert_eq!(
        s.inputs[0].selection,
        Selection::Selected {
            inclusion: 1.0,
            outcome: Some(Outcome::A)
        }
    );
    assert_eq!(case_of(&s, U, 0), Case::Positive);
}

/// j1 reviewed, decided and terminal `A` in cut 0 of epoch 0, not closing; `then` the next cut.
fn closing(then: impl FnOnce(&mut Log)) -> Vec<CutInput> {
    let mut log = Log::new();
    let mut e = first_round(item("j1"), &J1);
    e.push(decided(GateOutcome::Pass));
    log.cut(0, false)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .steps("j1", e)
        .record(terminal("j1", Outcome::A, "e1"));
    then(&mut log);
    log.cuts
}

/// A1 `17` §14.10: a cut applying no `NodeEvent` closes its epoch's cohorts, or a later epoch's.
#[test]
fn a_cut_with_no_new_event_closes_the_cohorts() {
    let signature = {
        let cut = Cut {
            number: 0,
            epoch: 0,
            closes: false,
            marks: Vec::new(),
        };
        let cp = cut.checkpoint(NET, consortium().member_set_hash());
        MemberObject::CutSignature {
            cut,
            member: 0,
            signature: members()[0].sign(&cp),
        }
        .encode()
    };
    let refused_only = |log: &mut Log| {
        let id = log.id(b"refused");
        log.last().refused.push((id, Refusal::NotAuthorized));
    };
    let variants: Vec<Vec<CutInput>> = vec![
        closing(|log| {
            log.cut(0, true);
        }),
        closing(|log| {
            log.cut(0, true).object(signature.clone());
        }),
        closing(|log| {
            log.cut(0, true);
            refused_only(log);
        }),
        closing(|log| {
            log.cut(1, false);
        }),
    ];
    for cuts in &variants {
        let open = snap(&cuts[..1]);
        let c = cohort_of(&open, U, 0);
        assert_eq!((c.closed, c.scored), (false, None));
        let closed = snap(cuts);
        let c = cohort_of(&closed, U, 0);
        assert_eq!(
            (c.members.clone(), c.closed, c.scored),
            (vec![0], true, Some(0))
        );
        assert!(matches!(value_of(&closed, U, 0), Some(Value::Final(_))));
        assert!(closed.register.iter().all(|a| a.at.cut == 0));
    }
}

/// j1's first round with `u` unrevealed, in cut 0; cut 1 of epoch 0, closing as `closes`.
fn unrevealed(closes: bool) -> Vec<CutInput> {
    let mut log = Log::new();
    let mut e = first_round(item("j1"), &seats(&PANEL, 0.5));
    e.retain(|e| !matches!(e, Event::Reveal { nym: n, .. } if *n == nym(U)));
    log.cut(0, false)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .steps("j1", e);
    log.cut(0, closes);
    log.cuts
}

/// A1 `17` §14.10: the same events under other closing metadata leave a cohort open or closed.
#[test]
fn the_same_events_under_other_metadata_leave_a_cohort_open_or_closed() {
    let mut cuts = unrevealed(false);
    let open = snap(&cuts);
    cuts[1].closes = true;
    cuts[1].digest = digest(1, 0, true);
    let closed = snap(&cuts);
    assert_eq!(open.register, closed.register);
    assert_eq!(open.inputs, closed.inputs);
    assert_eq!(open.inputs[0].first[0].report, Report::Open);
    let (a, b) = (cohort_of(&open, U, 0), cohort_of(&closed, U, 0));
    assert_eq!(
        (a.members.clone(), a.closed, a.scored),
        (vec![0], false, None)
    );
    assert_eq!(
        (b.members.clone(), b.closed, b.scored),
        (vec![0], true, Some(0))
    );
    assert_eq!(
        value_of(&closed, U, 0),
        Some(&Value::Unavailable(vec![(0, Cause::Unfrozen)]))
    );
}

/// A1 `17` §14.10: without a reveal close a report stays `Open` at every prefix, never `Missing`.
#[test]
fn an_unrevealed_report_stays_open_at_every_prefix() {
    let mut cuts = unrevealed(true);
    let mut log = Log { cuts, seq: 1000 };
    log.cut(1, false).cut(2, true);
    cuts = log.cuts;
    for n in 0..cuts.len() {
        let s = snap(&cuts[..=n]);
        assert_eq!(s.inputs[0].first[0].report, Report::Open);
        assert!(!s.inputs[0].frozen);
        assert_eq!(case_of(&s, U, 0), Case::Unfrozen);
        let closed = cohort_of(&s, U, 0).closed;
        assert_eq!(closed, n >= 1);
    }
}

/// `17` §14.10's criterion: `u` on j1 (A, `A`), j2 (A, pending), j3 (no design) in epoch 0 and
/// j4 (A, `I`) in epoch 1, weights for both, both closed.
fn j1_to_j4() -> Vec<CutInput> {
    let mut log = Log::new();
    let decided_round = |tag: &str| {
        let mut e = first_round(item(tag), &seats(&PANEL, 0.5));
        e.push(decided(GateOutcome::Pass));
        e
    };
    log.cut(0, false);
    for (k, tag) in ["j1", "j2", "j3", "j4"].iter().enumerate() {
        log.deposit(tag, 2 + k as u8);
    }
    log.record(unit_weights(0))
        .record(design("j1", Design::A))
        .record(design("j2", Design::A))
        .steps("j1", decided_round("j1"))
        .steps("j2", decided_round("j2"))
        .steps("j3", decided_round("j3"))
        .record(terminal("j1", Outcome::A, "e1"));
    log.cut(1, false)
        .record(unit_weights(1))
        .record(design("j4", Design::A))
        .steps("j4", decided_round("j4"))
        .record(terminal("j4", Outcome::I, "e4"));
    log.cut(1, true);
    log.cuts
}

/// A1 `17` §14.10: the register keeps all four assignments; the counts are the valid subset's.
#[test]
fn the_register_keeps_every_assignment_and_the_counts_are_the_valid_subset_s() {
    let s = snap(&j1_to_j4());
    let status: Vec<_> = s.items.iter().map(|i| (i.cid, i.status.clone())).collect();
    assert_eq!(
        status,
        vec![
            (item("j1"), Ok(0)),
            (item("j2"), Ok(1)),
            (item("j3"), Err(vec![Invalidity::NoDesign])),
            (item("j4"), Ok(2)),
        ]
    );
    assert_eq!(s.scored, vec![0, 1, 3]);
    assert_eq!(
        s.counts(nym(U)),
        ReviewerCounts {
            assignments: 4,
            unscored: 1,
            valid: Counts { n: 3, o: 2, v: 1 },
        }
    );
    assert_eq!(s.scores.assignments.len(), 21);
    let mine: Vec<usize> = s
        .register
        .iter()
        .filter(|a| a.nym == nym(U))
        .map(|a| a.item)
        .collect();
    assert_eq!(mine, vec![0, 1, 2, 3]);
    for n in PANEL {
        let first = cohort_of(&s, n, 0);
        assert_eq!(
            (
                first.members.clone(),
                first.closed,
                first.invalid.clone(),
                first.scored
            ),
            (vec![0, 1, 2], true, vec![2], None)
        );
        let second = cohort_of(&s, n, 1);
        let k = usize::from(n - 1);
        assert_eq!(
            (second.members.clone(), second.closed, second.scored),
            (vec![3], true, Some(k))
        );
        assert_eq!(
            s.scorer_cohorts[k],
            Cohort {
                nym: nym(n),
                items: vec![2]
            }
        );
        let c = &s.scores.cohorts[k];
        assert_eq!((c.size, c.o, c.v, &c.value), (1, 1, 0, &Value::Final(0.0)));
    }
}

/// The same cuts and entries as `cuts`, every slot cloned; hand-built, so no refusal to carry.
fn copy(cuts: &[CutInput]) -> Vec<CutInput> {
    let one = |c: &CutInput| {
        assert!(c.refused.is_empty());
        CutInput {
            number: c.number,
            epoch: c.epoch,
            closes: c.closes,
            digest: c.digest,
            slots: c.slots.clone(),
            refused: Vec::new(),
        }
    };
    cuts.iter().map(one).collect()
}

/// A1 `17` §14.10: equal cut digests with other proposed records or slots are read on their data.
#[test]
fn equal_digests_with_other_records_are_read_on_their_own_data() {
    let mut log = Log::new();
    let mut e = first_round(item("j1"), &J1);
    e.push(decided(GateOutcome::Pass));
    log.cut(0, false)
        .deposit("j1", 2)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .steps("j1", e);
    log.cut(0, true).record(terminal("j1", Outcome::A, "e1"));
    let a = log.cuts;
    let mut r = copy(&a);
    r[1].slots[0] = Slot::Proposed(terminal("j1", Outcome::R, "e1"));
    let mut late = copy(&a);
    let moved = late[0].slots.remove(2);
    late[0].slots.push(moved);
    let applied = |cuts: &[CutInput]| -> Vec<Slot> {
        let slots = cuts.iter().flat_map(|c| c.slots.iter());
        slots
            .filter(|s| matches!(s, Slot::Applied { .. }))
            .cloned()
            .collect()
    };
    assert_eq!(applied(&a), applied(&r));
    assert_eq!(applied(&a), applied(&late));
    let (sa, sr, sl) = (snap(&a), snap(&r), snap(&late));
    assert_eq!((sa.cut, sa.digest), (sr.cut, sr.digest));
    assert_eq!((sa.cut, sa.digest), (sl.cut, sl.digest));
    assert_eq!(case_of(&sa, U, 0), Case::Positive);
    assert_eq!(case_of(&sr, U, 0), Case::Negative);
    assert_ne!(sa.records, sr.records);
    assert_eq!(sl.items[0].status, Err(vec![Invalidity::NoDesign]));
    assert!(sl.inputs.is_empty());
    let late_design = (at(0, 20), Some(Refused::Late));
    let orphan = (at(1, 0), Some(Refused::NoDesign));
    assert_eq!(refused(&sl)[1..], [late_design, orphan]);
}

/// A1 `17` §14.10: a cohort record is judged once, against the rule, after its cohort closes.
#[test]
fn a_cohort_record_is_checked_against_the_rule() {
    let mut log = Log::new();
    let decided_round = |tag: &str| {
        let mut e = first_round(item(tag), &seats(&PANEL, 0.5));
        e.push(decided(GateOutcome::Pass));
        e
    };
    let listed = |n: u8, tags: &[&str]| Proposed::Cohort {
        nym: nym(n),
        epoch: 0,
        items: tags.iter().map(|t| item(t)).collect(),
    };
    log.cut(0, false)
        .deposit("j1", 2)
        .deposit("j2", 3)
        .record(unit_weights(0))
        .record(design("j1", Design::A))
        .record(design("j2", Design::A))
        .steps("j1", decided_round("j1"))
        .steps("j2", decided_round("j2"))
        .record(listed(U, &["j1", "j2"]));
    let early = log.here();
    log.cut(1, false)
        .record(listed(U, &["j2"]))
        .record(listed(U, &["j1", "j2"]))
        .record(listed(2, &["j1"]));
    let s = snap(&log.cuts);
    assert_eq!(
        refused(&s)[3..],
        [
            (early, Some(Refused::CohortOpen)),
            (at(1, 0), Some(Refused::Membership)),
            (at(1, 1), Some(Refused::Duplicate { first: early })),
            (at(1, 2), Some(Refused::Membership)),
        ]
    );
    assert_eq!(cohort_of(&s, U, 0).members, vec![0, 1]);
}

/// A1 `17` §14.10: a proposed record's encoding is canonical, its id its CID, no ledger object.
#[test]
fn a_proposed_record_s_encoding_is_canonical_and_decodes_as_no_ledger_object() {
    let forward: BTreeMap<Nym, f64> = [(nym(2), 1.0), (nym(1), 0.5)].into_iter().collect();
    let backward: BTreeMap<Nym, f64> = [(nym(1), 0.5), (nym(2), 1.0)].into_iter().collect();
    let mut records = vec![
        Proposed::Weights {
            epoch: 0,
            weights: forward,
        },
        design("j1", Design::A),
        design("j1", Design::C { inclusion: 0.5 }),
        draw("j1", 1, true),
        draw("j1", 2, true),
        terminal("j1", Outcome::I, "e1"),
        Proposed::Cohort {
            nym: nym(1),
            epoch: 0,
            items: [item("j1"), item("j2")].into_iter().collect(),
        },
    ];
    let reordered = Proposed::Weights {
        epoch: 0,
        weights: backward,
    };
    assert_eq!(records[0].encode(), reordered.encode());
    let design_bytes = [
        &[0xE0, 2][..],
        &item("j1").0,
        &group("j1").0,
        &rule("j1").0,
        &[0],
    ]
    .concat();
    assert_eq!(records[1].encode(), design_bytes);
    let terminal_bytes = [
        &[0xE0, 4][..],
        &item("j1").0,
        &group("j1").0,
        &rule("j1").0,
        &[2],
        &1_u64.to_le_bytes(),
        &cid(b"e1").0,
        &[1],
    ]
    .concat();
    assert_eq!(records[5].encode(), terminal_bytes);
    for i in [1, 5] {
        let mut other_group = records[i].clone();
        match &mut other_group {
            Proposed::Design {
                group: declared, ..
            }
            | Proposed::Terminal {
                group: declared, ..
            } => {
                *declared = group("j2");
            }
            _ => unreachable!(),
        }
        assert_ne!(records[i].encode(), other_group.encode());
        assert_ne!(records[i].id(), other_group.id());
        records.push(other_group);
    }
    let ids: BTreeSet<Cid> = records.iter().map(Proposed::id).collect();
    assert_eq!(ids.len(), records.len());
    for r in &records {
        let bytes = r.encode();
        assert_eq!(bytes, r.clone().encode());
        assert_eq!(r.id(), cid(&bytes));
        assert!(NodeEvent::decode(&bytes).is_none());
        assert!(MemberObject::decode(&bytes).is_none());
    }
}

fn forward(a: Case, b: Case) -> bool {
    use Case::*;
    a == b
        || match a {
            Unfrozen => true,
            Undrawn => matches!(
                b,
                NotSelected | Pending | Positive | Negative | Inconclusive
            ),
            Pending => matches!(b, Positive | Negative | Inconclusive),
            _ => false,
        }
}

fn reordered_assignments(invalid: bool) -> Vec<CutInput> {
    let pi = f64::from_bits(23_u64 << 52);
    let first0 = seats(&[2, 3, 4, 5, 6, 7, 8], 0.0);
    let mut first1 = seats(&PANEL, 1.0);
    first1[0].1 = 0.0;
    let mut first2 = seats(&PANEL, 0.0);
    first2[0].1 = 1.0;
    let mut log = Log::new();
    log.cut(0, false)
        .deposit("j0", 2)
        .deposit("j1", 3)
        .deposit("j2", 4)
        .record(weights(0, &seats(&[1, 2, 3, 4, 5, 6, 7, 8], 1.0)));
    for (tag, inclusion) in [("j0", pi), ("j1", pi), ("j2", 1.0)] {
        if !invalid || tag == "j2" {
            log.record(design(tag, Design::C { inclusion }));
        }
    }
    log.steps("j0", first_round(item("j0"), &first0))
        .steps("j0", vec![decided(GateOutcome::SupplementaryReview)]);
    log.cut(0, false);
    for (tag, first) in [("j1", first1), ("j2", first2)] {
        log.steps(tag, first_round(item(tag), &first))
            .steps(tag, vec![decided(GateOutcome::Pass)]);
        if !invalid || tag == "j2" {
            log.record(draw(tag, 1, true))
                .record(terminal(tag, Outcome::A, tag));
        }
    }
    log.cut(0, false)
        .steps("j0", extra_round(item("j0"), &[(U, 1.0)]))
        .steps(
            "j0",
            vec![Event::Resolve {
                outcome: GateOutcome::Pass,
            }],
        );
    if !invalid {
        log.record(draw("j0", 1, true))
            .record(terminal("j0", Outcome::A, "j0"));
    }
    log.cut(0, true).cut(1, false);
    log.cuts
}

/// A1 `17` §14.10: an extra assignment can insert an earlier item into an open cohort.
#[test]
fn an_extra_assignment_keeps_members_in_item_register_order() {
    let cuts = reordered_assignments(false);
    let before = snap(&cuts[..2]);
    assert_eq!(cohort_of(&before, U, 0).members, [1, 2]);
    let after = snap(&cuts[..3]);
    assert_eq!(cohort_of(&after, U, 0).members, [0, 1, 2]);
    assert!(!cohort_of(&after, U, 0).closed);
    assert!(after.register.starts_with(&before.register));
    invariants(&cuts);
}

/// A1 `17` §14.10: the scorer sums in item register order, retaining the unit after cancellation.
#[test]
fn item_register_order_reaches_the_scorer() {
    let cuts = reordered_assignments(false);
    let s = snap(&cuts);
    let k = cohort_of(&s, U, 0).scored.unwrap();
    let c = &s.scores.cohorts[k];
    assert_eq!(c.known, Ok(1.0));
    assert_eq!(c.value, Value::Final(1.0 / 3.0));
    assert_eq!(s.scorer_cohorts[k].items, [0, 1, 2]);
    let pi = f64::from_bits(23_u64 << 52);
    let large = f64::from_bits(2023_u64 << 52);
    let first = |panel: &[u8], p: &dyn Fn(u8) -> f64| {
        panel
            .iter()
            .map(|&n| Panelist {
                nym: nym(n),
                report: Report::Revealed(p(n)),
                weight: 1.0,
            })
            .collect()
    };
    let selected = |inclusion| Selection::Selected {
        inclusion,
        outcome: Some(Outcome::A),
    };
    let inputs = vec![
        Item {
            first: first(&[2, 3, 4, 5, 6, 7, 8], &|_| 0.0),
            extra: vec![Extra {
                nym: nym(U),
                report: Report::Revealed(1.0),
            }],
            frozen: true,
            selection: selected(pi),
        },
        Item {
            first: first(&PANEL, &|n| if n == U { 0.0 } else { 1.0 }),
            extra: Vec::new(),
            frozen: true,
            selection: selected(pi),
        },
        Item {
            first: first(&PANEL, &|n| if n == U { 1.0 } else { 0.0 }),
            extra: Vec::new(),
            frozen: true,
            selection: selected(1.0),
        },
    ];
    assert_eq!(s.inputs, inputs);
    let expected = score(
        &inputs,
        &[Cohort {
            nym: nym(U),
            items: vec![0, 1, 2],
        }],
    )
    .unwrap();
    assert_eq!(*c, expected.cohorts[0]);
    assert_eq!(
        c.members.iter().map(|a| a.baseline).collect::<Vec<_>>(),
        [Some(0.0), Some(1.0), Some(0.0)]
    );
    assert_eq!(
        c.members.iter().map(|a| a.term).collect::<Vec<_>>(),
        [Ok(Some(large)), Ok(Some(-large)), Ok(Some(1.0))]
    );
    let chronological = score(
        &inputs,
        &[Cohort {
            nym: nym(U),
            items: vec![1, 2, 0],
        }],
    )
    .unwrap();
    assert_eq!(chronological.cohorts[0].known, Ok(0.0));
    assert_eq!(chronological.cohorts[0].value, Value::Final(0.0));
}

/// A1 `17` §14.10: invalid members use item register order and keep the cohort unscored.
#[test]
fn invalid_members_also_follow_item_register_order() {
    let cuts = reordered_assignments(true);
    let s = snap(&cuts);
    let c = cohort_of(&s, U, 0);
    assert_eq!(c.invalid, [0, 1]);
    assert_eq!(c.members, [0, 1, 2]);
    assert_eq!(c.scored, None);
    assert_eq!(s.scored, [2]);
    assert_eq!(s.counts(nym(U)).assignments, 3);
    assert_eq!(s.counts(nym(U)).unscored, 2);
    invariants(&cuts);
}

/// `17` §14.10's invariants between the snapshots of every truncation of `cuts`.
fn invariants(cuts: &[CutInput]) {
    let snaps: Vec<Snapshot> = (0..cuts.len()).map(|n| snap(&cuts[..=n])).collect();
    for (n, a) in snaps.iter().enumerate() {
        for b in &snaps[n + 1..] {
            assert!(b.register.starts_with(&a.register));
            assert!(b.scored.starts_with(&a.scored));
            for (x, y) in a.items.iter().zip(&b.items) {
                assert_eq!(
                    (x.cid, x.assigned, x.epoch, &x.status),
                    (y.cid, y.assigned, y.epoch, &y.status)
                );
                assert!(x.frozen.is_none() || x.frozen == y.frozen);
            }
            for x in &a.scores.assignments {
                let same =
                    |y: &&protocol::cohort_scores::Assignment| (y.nym, y.item) == (x.nym, x.item);
                let y = b.scores.assignments.iter().find(same).unwrap();
                assert!(forward(x.case, y.case), "{:?} to {:?}", x.case, y.case);
            }
            for (x, y) in a.inputs.iter().zip(&b.inputs) {
                let reports = x.first.iter().zip(&y.first);
                for (p, q) in reports {
                    assert!(p.report == Report::Open || p.report == q.report);
                }
            }
            for c in &a.cohorts {
                let d = b
                    .cohorts
                    .iter()
                    .find(|d| (d.nym, d.epoch) == (c.nym, c.epoch))
                    .unwrap();
                assert!(c.members.iter().all(|m| d.members.contains(m)));
                assert!(c.members.windows(2).all(|w| w[0] < w[1]));
                assert!(d.members.windows(2).all(|w| w[0] < w[1]));
                assert!(!c.closed || (d.closed && d.members == c.members));
                assert!(c.invalid.is_empty() || !d.invalid.is_empty());
            }
            assert!(b.records.starts_with(&a.records));
            assert!(b.refusals.starts_with(&a.refusals));
        }
    }
}

/// A1 `17` §14.10: every truncation keeps the invariants; recomputation is identical bit for bit.
#[test]
fn truncations_keep_the_invariants_and_recomputation_is_identical() {
    let inputs = vec![
        valid_path(),
        absent(),
        drawn(),
        terminals(),
        j1_to_j4(),
        unrevealed(true),
    ];
    for cuts in &inputs {
        invariants(cuts);
        let (a, b) = (snap(cuts), snap(cuts));
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
        assert_eq!(a, b);
    }
}
