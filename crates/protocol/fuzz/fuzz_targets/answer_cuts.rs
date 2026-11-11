//! The answer review on signed cuts (`docs/05` §Built on cuts): `AnswerLedger::apply_in_view` over
//! members' feeds of genuine, altered and arbitrary objects, forks included, and cuts proposed,
//! altered and signed at will. No panic; the properties asserted are in `README.md`.
#![no_main]

use ed25519_dalek::Signature;
use identity::credential::{AnonymousCredential, Credential, Issuer, IssuerPublic};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use libfuzzer_sys::fuzz_target;
use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{added, Cut, Mark, MemberObject};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica};
use network::tier::{Tier, Tiers};
use protocol::answer::{commit_answer, Answer, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{Accepted, AnswerLedger, AnswerLedgerError, CutReport, MAX_QUESTION};
use protocol::answer_path::{commit_context, network_context, AnswerConfig, POOL_CONTEXT};
use protocol::answer_rule::ConsensusRule;
use protocol::deposit::deposit_context;
use std::collections::BTreeSet;
use std::sync::OnceLock;

const NET: [u8; 32] = [0x79; 32];
const N: usize = 4;
const T: usize = 3;
const MAX_OPS: usize = 64;
const CIRCLE: u8 = 4;

#[derive(Clone)]
enum Op {
    /// `feed`: the member in bits 0–1, `CIRCLE`, then bit 3 for a second branch of the feed.
    Publish {
        feed: u8,
        object: Vec<u8>,
    },
    Cut {
        alter: u8,
        signers: u8,
        view: u8,
    },
}

struct Fixture {
    members: Vec<Member>,
    writers: Vec<FeedWriter>,
    tiers: Tiers,
    issuer: IssuerPublic,
    pool: Vec<Vec<u8>>,
    script: Vec<Op>,
    /// The lengths of the script's prefixes an input may start from.
    stages: Vec<usize>,
}

fn tiers(members: &[Member], writers: &[FeedWriter]) -> Tiers {
    let consortium = Consortium::new(members.iter().map(Member::public).collect(), T);
    let circle: Vec<_> = writers[N..].iter().map(FeedWriter::public).collect();
    Tiers::new(NET, consortium, &circle).expect("a valid tiered set")
}

fn person(issuer: &Issuer, secret: u8) -> (AnonymousCredential, Nym) {
    let holder = Credential::from_secret([secret; 32]);
    let (request, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    let cred = pending.finalize(issuer.issue(&request).expect("a valid request"));
    let judge = prove(&cred, &issuer.public(), Role::Judge, b"nym").id();
    (cred, judge)
}

fn deposit(issuer: &Issuer, author: &AnonymousCredential, stem: &[u8], epoch: u64) -> Vec<u8> {
    let context = if stem.is_empty() { MAX_QUESTION } else { 7 };
    let question = Question {
        context: vec![b'c'; context],
        stem: stem.to_vec(),
        alternatives: vec![b"a".to_vec(), b"b".to_vec()],
        key: 0,
    };
    let context = network_context(&NET, &deposit_context(question.content_id(), epoch));
    AnswerEvent::Deposit {
        epoch,
        proof: prove(author, &issuer.public(), Role::Propose, &context),
        question,
    }
    .encode()
}

/// Each member's beacon commit and reveal for `epoch`, in member order.
fn beacon(members: &[Member], set: [u8; 32], epoch: u64) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let round = RoundId {
        network_id: NET,
        member_set_hash: set,
        epoch,
    };
    let pairs = (members.iter().enumerate()).map(|(i, m)| m.beacon_commit(round, i));
    pairs
        .map(|(commit, reveal)| {
            let reveal = MemberObject::Reveal { epoch, reveal };
            (MemberObject::Commit(commit).encode(), reveal.encode())
        })
        .unzip()
}

fn build() -> Fixture {
    let members: Vec<Member> = (0..N as u8)
        .map(|i| Member::from_seed([10 + i; 32]))
        .collect();
    let writers: Vec<FeedWriter> = (0..2 * N as u8)
        .map(|i| FeedWriter::from_seed(NET, [10 + i % 4 + 10 * (i / 4); 32]))
        .collect();
    let tiers = tiers(&members, &writers);
    let set = tiers.consortium().member_set_hash();
    let issuer = Issuer::new([1; 32]);
    let public = issuer.public();
    let reviewers: Vec<_> = (100..103).map(|s| person(&issuer, s)).collect();
    let author = person(&issuer, 1).0;
    let pool_context = network_context(&NET, POOL_CONTEXT);
    let joins: Vec<Vec<u8>> = (reviewers.iter())
        .map(|(cred, _)| AnswerEvent::JoinPool {
            proof: prove(cred, &public, Role::Judge, &pool_context),
        })
        .map(|e| e.encode())
        .collect();
    let answer = Answer::Alternative(0);
    let commits = |q: u64| -> Vec<Vec<u8>> {
        (reviewers.iter().zip(0u8..))
            .map(|((cred, judge), i)| {
                let commitment = commit_answer(answer, &[i; 32], *judge, q);
                let context = network_context(&NET, &commit_context(q, &commitment));
                AnswerEvent::Commit {
                    question: q,
                    commitment,
                    proof: prove(cred, &public, Role::Judge, &context),
                }
                .encode()
            })
            .collect()
    };
    let reveals = |q: u64| -> Vec<Vec<u8>> {
        (reviewers.iter().zip(0u8..))
            .map(|((_, judge), i)| AnswerEvent::Reveal {
                question: q,
                reviewer: *judge,
                answer,
                nonce: [i; 32],
            })
            .map(|e| e.encode())
            .collect()
    };
    let (commits, reveals) = ([commits(0), commits(1)], [reveals(0), reveals(1)]);
    let deposits = [
        deposit(&issuer, &author, b"q0", 0),
        deposit(&issuer, &author, b"q1", 1),
        deposit(&issuer, &author, b"q2", 2),
        deposit(&issuer, &author, b"", 0),
    ];
    let (commits0, reveals0) = beacon(&members, set, 0);
    let (commits1, reveals1) = beacon(&members, set, 1);
    let (commits2, reveals2) = beacon(&members, set, 2);
    let request = AnswerEvent::RequestCloseEpoch { epoch: 0 }.encode();
    let close_commits = AnswerEvent::CloseCommits { question: 0 }.encode();
    let close = AnswerEvent::Close { question: 0 }.encode();
    let others = [
        AnswerEvent::RequestCloseEpoch { epoch: 1 }.encode(),
        AnswerEvent::RequestCloseEpoch { epoch: 2 }.encode(),
        AnswerEvent::CloseCommits { question: 1 }.encode(),
        AnswerEvent::Close { question: 1 }.encode(),
        AnswerEvent::Draw { question: 0 }.encode(),
        AnswerEvent::CloseEpoch {
            epoch: 0,
            beacon: [0; 32],
        }
        .encode(),
        MemberObject::ViewChange {
            number: 0,
            view: 1,
            member: 0,
        }
        .encode(),
    ];

    let on = |feed: u8, objects: &[Vec<u8>]| -> Vec<Op> {
        let publish = |object: &Vec<u8>| Op::Publish {
            feed,
            object: object.clone(),
        };
        objects.iter().map(publish).collect()
    };
    let own = |objects: &[Vec<u8>]| -> Vec<Op> {
        let each = objects.iter().zip(0u8..);
        each.flat_map(|(o, m)| on(m, std::slice::from_ref(o)))
            .collect()
    };
    let by = |members: u8, object: &Vec<u8>| -> Vec<Op> {
        (0..members)
            .flat_map(|m| on(m | CIRCLE, std::slice::from_ref(object)))
            .collect()
    };
    let cut = || {
        vec![Op::Cut {
            alter: 0,
            signers: 0b0111,
            view: 0,
        }]
    };
    let steps = [
        [
            own(&commits0),
            on(CIRCLE, &joins),
            on(CIRCLE, &deposits[..1]),
        ]
        .concat(),
        [by(3, &request), cut(), cut()].concat(),
        [own(&reveals0), own(&commits1), cut()].concat(),
        [on(1 | CIRCLE, &commits[0]), by(3, &close_commits), cut()].concat(),
        [on(2 | CIRCLE, &reveals[0]), cut(), by(3, &close), cut()].concat(),
    ];
    let mut stages = vec![0];
    let mut script = Vec::new();
    for step in steps {
        script.extend(step);
        stages.push(script.len());
    }
    let pool = [
        joins,
        deposits.to_vec(),
        commits.concat(),
        reveals.concat(),
        commits0,
        reveals0,
        commits1,
        reveals1,
        commits2,
        reveals2,
        vec![request, close_commits, close],
        others.to_vec(),
    ]
    .concat();
    Fixture {
        members,
        writers,
        tiers,
        issuer: public,
        pool,
        script,
        stages,
    }
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let f = build();
        let mut world = World::new(&f);
        for op in &f.script {
            world.run(&f, op);
        }
        let usable = world.ledger.state().closure(0).map(|c| c.tally.usable());
        assert_eq!(
            usable,
            Some(3),
            "the script closes its question on three answers"
        );
        assert!(
            world.ledger.refused().is_empty(),
            "the script's entries count"
        );
        f
    })
}

/// A cut applied, the view its signatures were given in, and the signatures.
type Signed = (Cut, u64, Vec<(usize, Signature)>);

struct World {
    /// One log per feed and branch, by an `Op::Publish`'s `feed`.
    logs: Vec<TransparencyLog>,
    replica: Replica,
    ledger: AnswerLedger,
    applied: Vec<Signed>,
}

fn ledger(f: &Fixture) -> AnswerLedger {
    let config = AnswerConfig {
        rule: ConsensusRule::new(3, 2, 3, 4).expect("a valid rule"),
        ..AnswerConfig::default()
    };
    let tiers = tiers(&f.members, &f.writers);
    AnswerLedger::new(tiers, f.issuer.clone(), config).expect("a valid configuration")
}

/// Marks in `cut` the first `len` entries of `log`, its `writer`'s chain the replica holds.
fn mark(cut: &mut Cut, writer: [u8; 32], log: &TransparencyLog, len: usize) {
    let Some(last) = len.checked_sub(1).and_then(|i| log.entries().get(i)) else {
        return;
    };
    cut.marks.retain(|m| m.writer != writer);
    cut.marks.push(Mark {
        writer,
        len: len as u64,
        head: last.hash,
    });
    cut.marks.sort_by_key(|m| m.writer);
}

/// `alter`: the alteration in bits 0–3, 0 to 6 none; the mark or the feed it touches in bits 4–7.
fn alter(cut: &mut Cut, alter: u8, f: &Fixture, logs: &[TransparencyLog]) {
    let at = usize::from(alter >> 4) % cut.marks.len().max(1);
    let feed = usize::from(alter >> 4);
    let writer = f.writers[feed % 8].public().to_bytes();
    match (alter % 16, cut.marks.get_mut(at)) {
        (7, _) => mark(cut, writer, &logs[feed], logs[feed].len().saturating_sub(1)),
        (8, _) => cut.number = cut.number.wrapping_add(1),
        (9, _) => cut.number = cut.number.wrapping_sub(1),
        (10, _) => cut.epoch = cut.epoch.wrapping_add(1),
        (11, _) => cut.closes ^= true,
        (12, Some(_)) => drop(cut.marks.remove(at)),
        (13, Some(mark)) => mark.head[0] ^= 1,
        (14, _) => mark(cut, writer, &logs[feed], logs[feed].len()),
        (15, _) => cut.marks.reverse(),
        _ => {}
    }
}

impl World {
    fn new(f: &Fixture) -> World {
        World {
            logs: (0..16).map(|_| TransparencyLog::new()).collect(),
            replica: Replica::new(f.tiers.writers().clone()),
            ledger: ledger(f),
            applied: Vec::new(),
        }
    }

    fn run(&mut self, f: &Fixture, op: &Op) {
        match op {
            Op::Publish { feed, object } => self.publish(f, *feed, object.clone()),
            Op::Cut {
                alter,
                signers,
                view,
            } => self.cut(f, *alter, *signers, *view),
        }
    }

    fn publish(&mut self, f: &Fixture, feed: u8, object: Vec<u8>) {
        let feed = usize::from(feed % 16);
        let entry = self.logs[feed].append(cid(&object)).clone();
        let signed = f.writers[feed % 8].sign(&entry);
        let inserted = self.replica.insert(signed, object);
        inserted.expect("a writer of the set");
    }

    /// `signers`: the members signing in bits 0–3, then signed in the next view, the first
    /// signature altered, the first repeated, the first under the next member's index.
    fn cut(&mut self, f: &Fixture, altered: u8, signers: u8, view: u8) {
        let next = self.ledger.next().expect("a next cut");
        let mut cut = Cut::next(&self.replica, self.ledger.last(), next.epoch, next.closes);
        let proposed = cut.clone();
        alter(&mut cut, altered, f, &self.logs);
        let view = u64::from(view % 3);
        let signed_in = view + u64::from(signers >> 4 & 1);
        let set = f.tiers.consortium().member_set_hash();
        let checkpoint = cut.checkpoint_in(signed_in, NET, set);
        let mut signatures: Vec<(usize, Signature)> = (0..N)
            .filter(|i| signers >> i & 1 == 1)
            .map(|i| (i, f.members[i].sign(&checkpoint)))
            .collect();
        let excluded = self.ledger.excluded(&cut);
        let counted = |i: &usize| signed_in == view && !excluded.contains(i);
        let mut valid: BTreeSet<usize> = (signatures.iter().map(|s| s.0)).filter(counted).collect();
        if let Some(first) = signatures.first_mut().filter(|_| signers & 32 != 0) {
            let mut bytes = first.1.to_bytes();
            bytes[0] ^= 1;
            first.1 = Signature::from_bytes(&bytes);
            valid.remove(&first.0);
        }
        if let Some(first) = signatures.first().copied().filter(|_| signers & 64 != 0) {
            signatures.push(first);
        }
        if let Some(first) = signatures.first().copied().filter(|_| signers & 128 != 0) {
            signatures.push(((first.0 + 1) % N, first.1));
        }
        let last = self.ledger.last().cloned();
        let expected = if cut.number != next.number {
            Err(AnswerLedgerError::OutOfSequence {
                expected: next.number,
            })
        } else if valid.len() < T {
            Err(AnswerLedgerError::Unsigned)
        } else {
            let checked = self.ledger.check(&cut);
            checked.and_then(|()| {
                added(&self.replica, last.as_ref(), &cut).map_err(AnswerLedgerError::Cut)
            })
        };
        let closed = self.applied.iter().filter(|(c, ..)| c.closes).count();
        let counted = (self.applied.len() as u64, closed as u64);
        assert_eq!((next.number, next.epoch), counted, "the next cut's place");
        let unsigned = valid.len() < T;
        assert!(
            cut != proposed || unsigned || expected.is_ok(),
            "a proposal refused"
        );
        let before = (self.ledger.digest(), self.ledger.refused().len());
        let result = self
            .ledger
            .apply_in_view(&self.replica, &cut, view, &signatures);
        match (result, expected) {
            (Ok(report), Ok(ids)) => {
                assert_eq!(self.ledger.last(), Some(&cut), "the last cut applied");
                self.reported(f, &cut, &report, &ids, before.1);
                self.applied.push((cut, view, signatures));
            }
            (Err(error), Err(expected)) => {
                assert_eq!(error, expected, "another refusal of the cut");
                let after = (self.ledger.digest(), self.ledger.refused().len());
                assert_eq!(after, before, "a refused cut changed the ledger");
                assert_eq!(self.ledger.last(), last.as_ref(), "a refused cut is last");
            }
            (result, expected) => panic!("{result:?} where {expected:?} was expected"),
        }
    }

    fn reported(&self, f: &Fixture, cut: &Cut, report: &CutReport, ids: &[EntryId], from: usize) {
        let named: Vec<EntryId> = report.entries.iter().map(|(id, _)| *id).collect();
        assert_eq!(named, ids, "the entries the cut added, in its order");
        assert_eq!(report.sealed, cut.closes.then_some(cut.epoch), "the seal");
        let mut refused = Vec::new();
        for (id, result) in &report.entries {
            let (_, object) = self.replica.get(id).expect("a cut's entry is held");
            let (member, tier) = f.tiers.place(&id.writer).expect("a writer of the set");
            match result {
                Ok(Accepted::Member) => {
                    let own = MemberObject::decode(object).is_some_and(|o| o.member() == member);
                    assert!(
                        own && tier == Tier::Public,
                        "a member object not its feed's"
                    );
                }
                Ok(_) => {
                    let derived =
                        |e| matches!(e, AnswerEvent::Draw { .. } | AnswerEvent::CloseEpoch { .. });
                    let logged = AnswerEvent::decode(object).is_some_and(|e| !derived(e));
                    assert!(
                        logged && tier == Tier::Circle,
                        "an event not a circle feed's"
                    );
                }
                Err(refusal) => refused.push((*id, *refusal)),
            }
        }
        let recorded = &self.ledger.refused()[from..];
        assert_eq!(recorded, refused, "the refusals recorded");
    }

    /// A fresh ledger given every applied cut again, over the replica as it ended.
    fn replay(&self, f: &Fixture) {
        let mut again = ledger(f);
        for (cut, view, signatures) in &self.applied {
            let applied = again.apply_in_view(&self.replica, cut, *view, signatures);
            applied.expect("an applied cut applies again");
        }
        assert_eq!(again.digest(), self.ledger.digest(), "the replayed digest");
        assert_eq!(again.state(), self.ledger.state(), "the replayed state");
        assert_eq!(
            again.refused(),
            self.ledger.refused(),
            "the replayed refusals"
        );
    }
}

/// The operations `bytes` spell: three bytes each, a publication's kind taking more.
fn ops(f: &Fixture, bytes: &[u8]) -> Vec<Op> {
    let mut bytes = bytes.iter().copied();
    let mut out = Vec::new();
    while out.len() < MAX_OPS {
        let (Some(kind), Some(feed), Some(b)) = (bytes.next(), bytes.next(), bytes.next()) else {
            break;
        };
        let genuine = || f.pool[usize::from(b) % f.pool.len()].clone();
        let object = match kind % 4 {
            0 => genuine(),
            1 => {
                let mut object = genuine();
                let at = usize::from(bytes.next().unwrap_or(0)) * object.len() / 256;
                let window = usize::from(bytes.next().unwrap_or(0) % 33);
                for (dst, src) in object[at..].iter_mut().zip(bytes.by_ref().take(window)) {
                    *dst = src;
                }
                object
            }
            2 => bytes.by_ref().take(usize::from(b % 65)).collect(),
            _ => {
                let view = bytes.next().unwrap_or(0);
                out.push(Op::Cut {
                    alter: feed,
                    signers: b,
                    view,
                });
                continue;
            }
        };
        out.push(Op::Publish { feed, object });
    }
    out
}

fuzz_target!(|data: &[u8]| {
    let Some((stage, rest)) = data.split_first() else {
        return;
    };
    let f = fixture();
    let mut world = World::new(f);
    let prefix = f.stages[usize::from(*stage) % f.stages.len()];
    for op in &f.script[..prefix] {
        world.run(f, op);
    }
    for op in ops(f, rest) {
        world.run(f, &op);
    }
    world.replay(f);
});
