//! The answer review offline, end to end (`docs/05` §The answer review, `docs/01` D49–D52):
//! creation, deposit and quota, draw, answers, closure, outcome, reputation, epochs, usable set.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::store::MAX_OBJECT;
use protocol::admission::Unproven;
use protocol::answer::{commit_answer, Answer, Malformed, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_node::{AnswerNode, AnswerNodeError};
use protocol::answer_path::{
    commit_context, AnswerConfig, AnswerPath, Applied, Closure, Refused, Round, Slots, POOL_CONTEXT,
};
use protocol::answer_rule::{ConsensusRule, Tally, Verdict};
use protocol::deposit::deposit_context;
use scoring::reputation::{author_score_weighted, AuthorPrior};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use Answer::{Alternative, DontKnow, NoneCorrect};

const KEY: u8 = 0;

fn person(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&req).unwrap())
}

fn question(tag: &str) -> Question {
    Question {
        context: b"civic education, 2026".to_vec(),
        stem: tag.as_bytes().to_vec(),
        alternatives: vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()],
        key: KEY,
    }
}

fn beacon(epoch: u64) -> [u8; 32] {
    [epoch as u8 ^ 0x5a; 32]
}

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("answer_review")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// One drawn reviewer's part in a round.
#[derive(Clone, Copy)]
enum Part {
    Answers(Answer),
    CommitsOnly(Answer),
    Silent,
}

/// A path and its people; every accepted event's encoding is kept, in order.
struct World {
    issuer: Issuer,
    path: AnswerPath,
    reviewers: BTreeMap<Nym, AnonymousCredential>,
    authors: Vec<(Nym, AnonymousCredential)>,
    log: Vec<Vec<u8>>,
}

impl World {
    fn new(reviewers: u8, authors: u8) -> Self {
        Self::joining(100..100 + reviewers, authors)
    }

    /// Reviewers join in the order of `secrets`.
    fn joining(secrets: impl IntoIterator<Item = u8>, authors: u8) -> Self {
        let issuer = Issuer::new([1u8; 32]);
        let mut w = World {
            path: AnswerPath::new(AnswerConfig::default()).unwrap(),
            reviewers: BTreeMap::new(),
            authors: Vec::new(),
            log: Vec::new(),
            issuer,
        };
        for secret in secrets {
            w.join(secret);
        }
        for i in 0..authors {
            let cred = person(&w.issuer, 1 + i);
            let nym = prove(&cred, &w.issuer.public(), Role::Propose, b"id").id();
            w.authors.push((nym, cred));
        }
        w
    }

    fn submit(&mut self, event: AnswerEvent) -> Result<Applied, Refused> {
        let bytes = event.encode();
        let applied = self.path.apply(&event, &self.issuer.public())?;
        self.log.push(bytes);
        Ok(applied)
    }

    fn join(&mut self, secret: u8) -> Nym {
        let cred = person(&self.issuer, secret);
        let proof = prove(&cred, &self.issuer.public(), Role::Judge, POOL_CONTEXT);
        let Applied::Joined(nym) = self.submit(AnswerEvent::JoinPool { proof }).unwrap() else {
            panic!("a join")
        };
        self.reviewers.insert(nym, cred);
        nym
    }

    fn deposit_event(&self, author: usize, q: &Question) -> AnswerEvent {
        let epoch = self.path.epoch();
        let ctx = deposit_context(q.content_id(), epoch);
        AnswerEvent::Deposit {
            epoch,
            question: q.clone(),
            proof: prove(
                &self.authors[author].1,
                &self.issuer.public(),
                Role::Propose,
                &ctx,
            ),
        }
    }

    fn deposit(&mut self, author: usize, q: &Question) -> Result<u64, Refused> {
        match self.submit(self.deposit_event(author, q))? {
            Applied::Deposited { question, .. } => Ok(question),
            other => panic!("{other:?}"),
        }
    }

    fn close_epoch(&mut self) {
        let epoch = self.path.epoch();
        let beacon = beacon(epoch);
        self.submit(AnswerEvent::CloseEpoch { epoch, beacon })
            .unwrap();
    }

    fn draw(&mut self, q: u64) -> Vec<Nym> {
        match self.submit(AnswerEvent::Draw { question: q }).unwrap() {
            Applied::Drawn { group } => group,
            other => panic!("{other:?}"),
        }
    }

    fn commit_event(&self, q: u64, reviewer: Nym, answer: Answer, nonce: [u8; 32]) -> AnswerEvent {
        let commitment = commit_answer(answer, &nonce, reviewer, q);
        let proof = prove(
            &self.reviewers[&reviewer],
            &self.issuer.public(),
            Role::Judge,
            &commit_context(q, &commitment),
        );
        AnswerEvent::Commit {
            question: q,
            commitment,
            proof,
        }
    }

    /// Draws `q`'s group and plays `parts` for its members in order, without closing.
    fn answer(&mut self, q: u64, parts: &[Part]) -> Vec<Nym> {
        let group = self.draw(q);
        assert_eq!(group.len(), parts.len());
        for (i, (&reviewer, part)) in group.iter().zip(parts).enumerate() {
            if let Part::Answers(a) | Part::CommitsOnly(a) = *part {
                let event = self.commit_event(q, reviewer, a, [i as u8; 32]);
                self.submit(event).unwrap();
            }
        }
        self.submit(AnswerEvent::CloseCommits { question: q })
            .unwrap();
        for (i, (&reviewer, part)) in group.iter().zip(parts).enumerate() {
            if let Part::Answers(answer) = *part {
                let event = AnswerEvent::Reveal {
                    question: q,
                    reviewer,
                    answer,
                    nonce: [i as u8; 32],
                };
                self.submit(event).unwrap();
            }
        }
        group
    }

    fn close(&mut self, q: u64) -> Closure {
        match self.submit(AnswerEvent::Close { question: q }).unwrap() {
            Applied::Closed(closure) => closure,
            other => panic!("{other:?}"),
        }
    }

    fn round(&mut self, q: u64, parts: &[Part]) -> Closure {
        self.answer(q, parts);
        self.close(q)
    }

    fn author(&self, i: usize) -> Nym {
        self.authors[i].0
    }

    /// A fresh path fed this world's log, decoded.
    fn replay(&self) -> AnswerPath {
        let mut path = AnswerPath::new(*self.path.config()).unwrap();
        for bytes in &self.log {
            let event = AnswerEvent::decode(bytes).expect("an accepted event decodes");
            path.apply(&event, &self.issuer.public()).unwrap();
        }
        path
    }
}

/// `usable` answers, `coinciding` of them the key; then `abstain` "don't know"; the rest silent.
fn parts(usable: usize, coinciding: usize, abstain: usize) -> Vec<Part> {
    let mut parts = vec![Part::Answers(Alternative(KEY)); coinciding];
    for i in coinciding..usable {
        parts.push(Part::Answers(match i % 3 {
            0 => Alternative(1),
            1 => Alternative(2),
            _ => NoneCorrect,
        }));
    }
    parts.extend(std::iter::repeat_n(Part::Answers(DontKnow), abstain));
    parts.resize(12, Part::Silent);
    parts
}

fn w1() -> f64 {
    (-std::f64::consts::LN_2 / 45.0).exp()
}

// AT-ANS-07: the whole path, creation to usable set, replayed from its log to the same state.
#[test]
fn at_ans_07_the_path_from_creation_to_the_usable_set() {
    let mut w = World::new(14, 1);
    let a = w.author(0);
    let q0 = w.deposit(0, &question("q0")).unwrap();
    let q1 = w.deposit(0, &question("q1")).unwrap();
    assert_eq!(w.path.slots(&a, 0), Some(Slots { quota: 2, used: 2 }));
    assert_eq!(
        w.deposit(0, &question("q2")),
        Err(Refused::OverQuota { quota: 2 })
    );
    w.close_epoch();
    let accepted = w.round(q0, &parts(9, 9, 3));
    assert_eq!(accepted.verdict, Verdict::Accepted);
    let discarded = w.round(q1, &parts(12, 3, 0));
    assert_eq!(discarded.verdict, Verdict::Discarded);
    assert_eq!(w.path.usable().map(|(q, _)| q).collect::<Vec<_>>(), [q0]);
    assert_eq!(w.path.usable().next().unwrap().1, &question("q0"));
    assert_eq!(w.path.reputation(&a), 0.4);
    w.close_epoch();
    let expected = (2.0 + w1()) / (5.0 + 2.0 * w1());
    assert!((w.path.reputation(&a) - expected).abs() < 1e-15);
    assert_eq!(w.path.quota(&a), 2);
    assert_eq!(w.replay(), w.path);
}

// AT-ANS-01: real rounds at both edges of each band, V = 9…12, abstentions and absences mixed.
#[test]
fn at_ans_01_the_thresholds_on_real_rounds() {
    let cases: [(usize, usize, Verdict); 16] = [
        (9, 7, Verdict::Accepted),
        (9, 6, Verdict::Divided),
        (9, 3, Verdict::Divided),
        (9, 2, Verdict::Discarded),
        (10, 8, Verdict::Accepted),
        (10, 7, Verdict::Divided),
        (10, 3, Verdict::Divided),
        (10, 2, Verdict::Discarded),
        (11, 9, Verdict::Accepted),
        (11, 8, Verdict::Divided),
        (11, 3, Verdict::Divided),
        (11, 2, Verdict::Discarded),
        (12, 9, Verdict::Accepted),
        (12, 8, Verdict::Divided),
        (12, 4, Verdict::Divided),
        (12, 3, Verdict::Discarded),
    ];
    let mut w = World::new(12, 8);
    let mut questions = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        let q = w.deposit(i / 2, &question(&format!("t{i}"))).unwrap();
        questions.push((q, case));
    }
    w.close_epoch();
    for (q, &(v, c, verdict)) in questions {
        let abstain = (12 - v) / 2;
        let closure = w.round(q, &parts(v, c, abstain));
        assert_eq!(closure.verdict, verdict, "V={v} C={c}");
        assert_eq!(closure.tally.usable() as usize, v);
        assert_eq!(closure.tally.coinciding(KEY) as usize, c);
        assert_eq!(closure.tally.abstentions as usize, abstain);
        assert_eq!(closure.tally.absences as usize, 12 - v - abstain);
        assert_eq!(closure.rule, ConsensusRule::default());
    }
}

// AT-ANS-02: the quorum counts usable answers only; abstentions and absences are kept, apart.
#[test]
fn at_ans_02_quorum_abstentions_and_absences() {
    let mut w = World::new(12, 2);
    let q: Vec<u64> = (0..4)
        .map(|i| w.deposit(i / 2, &question(&format!("q{i}"))).unwrap())
        .collect();
    w.close_epoch();
    let eight = w.round(q[0], &parts(8, 8, 4));
    assert_eq!(eight.verdict, Verdict::InsufficientParticipation);
    assert_eq!((eight.tally.abstentions, eight.tally.absences), (4, 0));
    let mut silent = parts(9, 9, 0);
    silent[9] = Part::CommitsOnly(Alternative(KEY));
    silent[10] = Part::CommitsOnly(Alternative(1));
    let nine = w.round(q[1], &silent);
    assert_eq!(nine.verdict, Verdict::Accepted);
    assert_eq!((nine.tally.usable(), nine.tally.absences), (9, 3));
    let none = w.round(
        q[2],
        &[vec![Part::Answers(NoneCorrect); 9], vec![Part::Silent; 3]].concat(),
    );
    assert_eq!(none.verdict, Verdict::Discarded);
    assert_eq!(
        none.tally,
        Tally {
            alternatives: vec![0, 0, 0],
            none: 9,
            abstentions: 0,
            absences: 3
        }
    );
    let nobody = w.round(q[3], &[Part::Silent; 12]);
    assert_eq!(nobody.verdict, Verdict::InsufficientParticipation);
    assert_eq!(nobody.tally.absences, 12);
    assert_eq!(nobody.observation, None);
    assert_eq!(w.path.pool().len(), 12);
}

// AT-ANS-03: the round closes only on its event, and nothing enters after.
#[test]
fn at_ans_03_closure_is_explicit_and_final() {
    let mut w = World::new(13, 1);
    let q = w.deposit(0, &question("q")).unwrap();
    assert_eq!(
        w.submit(AnswerEvent::Draw { question: q }),
        Err(Refused::BeaconPending)
    );
    w.close_epoch();
    let group = w.draw(q);
    let early = AnswerEvent::Reveal {
        question: q,
        reviewer: group[0],
        answer: DontKnow,
        nonce: [0; 32],
    };
    assert_eq!(w.submit(early), Err(Refused::WrongRound));
    assert_eq!(
        w.submit(AnswerEvent::Close { question: q }),
        Err(Refused::WrongRound)
    );
    for (i, &r) in group.iter().enumerate() {
        let event = w.commit_event(q, r, Alternative(KEY), [i as u8; 32]);
        w.submit(event).unwrap();
    }
    w.submit(AnswerEvent::CloseCommits { question: q }).unwrap();
    let late = w.commit_event(q, group[0], Alternative(KEY), [99; 32]);
    assert_eq!(w.submit(late), Err(Refused::WrongRound));
    for (i, &r) in group.iter().enumerate() {
        let reveal = AnswerEvent::Reveal {
            question: q,
            reviewer: r,
            answer: Alternative(KEY),
            nonce: [i as u8; 32],
        };
        w.submit(reveal).unwrap();
        assert!(
            matches!(w.path.entry(q).unwrap().round, Round::Revealing { .. }),
            "open after {i}"
        );
    }
    assert_eq!(w.close(q).verdict, Verdict::Accepted);
    let after = AnswerEvent::Reveal {
        question: q,
        reviewer: group[0],
        answer: DontKnow,
        nonce: [0; 32],
    };
    assert_eq!(w.submit(after), Err(Refused::WrongRound));
    assert_eq!(
        w.submit(AnswerEvent::Close { question: q }),
        Err(Refused::WrongRound)
    );
    assert_eq!(
        w.submit(AnswerEvent::Draw { question: q }),
        Err(Refused::WrongRound)
    );
    assert_eq!(
        w.submit(AnswerEvent::Close { question: 9 }),
        Err(Refused::UnknownQuestion)
    );
}

// AT-ANS-04: invalid or duplicate requests spend no slot; slots are never refunded or carried.
#[test]
fn at_ans_04_duplicates_and_the_quotas_consumption() {
    let mut w = World::new(12, 2);
    let a = w.author(0);
    let mut bad = question("bad");
    bad.key = 7;
    assert_eq!(
        w.deposit(0, &bad),
        Err(Refused::Malformed(Malformed::KeyOutOfRange))
    );
    let mut wrong_epoch = w.deposit_event(0, &question("q0"));
    if let AnswerEvent::Deposit { epoch, .. } = &mut wrong_epoch {
        *epoch = 1;
    }
    assert_eq!(w.submit(wrong_epoch), Err(Refused::WrongEpoch));
    let ctx = deposit_context(question("other").content_id(), 0);
    let proof = prove(&w.authors[0].1, &w.issuer.public(), Role::Propose, &ctx);
    let unbound = AnswerEvent::Deposit {
        epoch: 0,
        question: question("q0"),
        proof,
    };
    assert_eq!(
        w.submit(unbound),
        Err(Refused::Unproven(Unproven::BadProof))
    );
    assert_eq!(w.path.slots(&a, 0), None);
    let q0 = w.deposit(0, &question("q0")).unwrap();
    assert_eq!(
        w.deposit(0, &question("q0")),
        Err(Refused::Duplicate { question: q0 })
    );
    assert_eq!(
        w.deposit(1, &question("q0")),
        Err(Refused::Duplicate { question: q0 })
    );
    assert_eq!(w.path.slots(&a, 0), Some(Slots { quota: 2, used: 1 }));
    assert_eq!(w.path.questions(), 1);
    w.close_epoch();
    assert_eq!(w.round(q0, &parts(12, 0, 0)).verdict, Verdict::Discarded);
    let corrected = Question {
        key: 1,
        ..question("q0")
    };
    let q1 = w.deposit(0, &corrected).unwrap();
    assert_eq!(w.path.slots(&a, 0), Some(Slots { quota: 2, used: 1 }));
    assert_eq!(w.path.slots(&a, 1), Some(Slots { quota: 2, used: 1 }));
    w.deposit(0, &question("q2")).unwrap();
    assert_eq!(
        w.deposit(0, &question("q3")),
        Err(Refused::OverQuota { quota: 2 })
    );
    w.close_epoch();
    let new_key = [Part::Answers(Alternative(1)); 12];
    assert_eq!(w.round(q1, &new_key).verdict, Verdict::Accepted);
    assert_eq!(w.path.closure(q0).unwrap().verdict, Verdict::Discarded);
    assert!(w.path.author(&a).unwrap().observation(q0).is_some());
    assert!(w.path.author(&a).unwrap().observation(q1).is_some());
    assert_eq!(w.path.usable().map(|(q, _)| q).collect::<Vec<_>>(), [q1]);
}

// AT-ANS-05: a refused event changes nothing; outcome and observation are recorded together.
#[test]
fn at_ans_05_atomicity() {
    let mut w = World::new(12, 1);
    let a = w.author(0);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    let group = w.answer(q, &parts(10, 2, 0));
    let before = w.path.clone();
    let refusals = [
        AnswerEvent::CloseCommits { question: q },
        AnswerEvent::Draw { question: q },
        AnswerEvent::CloseEpoch {
            epoch: 0,
            beacon: [0; 32],
        },
        AnswerEvent::Reveal {
            question: q,
            reviewer: group[0],
            answer: DontKnow,
            nonce: [0; 32],
        },
        AnswerEvent::Reveal {
            question: q,
            reviewer: group[11],
            answer: DontKnow,
            nonce: [11; 32],
        },
    ];
    for event in refusals {
        assert!(w.submit(event).is_err());
        assert_eq!(w.path, before);
    }
    assert_eq!(w.path.author(&a), None);
    let closure = w.close(q);
    assert_eq!(closure.verdict, Verdict::Discarded);
    let recorded = w.path.author(&a).unwrap().observation(q).copied();
    assert_eq!(closure.observation, recorded);
    assert_eq!(w.path.closure(q), Some(&closure));
    let o = recorded.unwrap();
    assert_eq!((o.success, o.weight, o.origin), (false, 1.0, closure.at));
}

// AT-ANS-06: each closure applies once, live and on replay.
#[test]
fn at_ans_06_single_application() {
    let dir = scratch("single");
    let issuer = Issuer::new([1u8; 32]);
    let mut w = World::new(12, 1);
    let a = w.author(0);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    w.round(q, &parts(12, 6, 0));
    assert_eq!(
        w.submit(AnswerEvent::Close { question: q }),
        Err(Refused::WrongRound)
    );
    w.close_epoch();
    let mut node = AnswerNode::open(&dir, issuer.public(), AnswerConfig::default()).unwrap();
    for bytes in &w.log {
        node.submit(AnswerEvent::decode(bytes).unwrap()).unwrap();
    }
    let reputation = node.state().reputation(&a);
    drop(node);
    let node = AnswerNode::open(&dir, issuer.public(), AnswerConfig::default()).unwrap();
    assert_eq!(node.state(), &w.path);
    assert_eq!(node.events(), w.log.len());
    assert_eq!(node.state().reputation(&a).to_bits(), reputation.to_bits());
    assert_eq!(
        node.state()
            .author(&a)
            .unwrap()
            .counted(protocol::author_time::AuthorPoint {
                epoch: 2,
                closings: 2
            }),
        1
    );
}

// AT-ANS-07: the same events give the same state and draw anywhere; join order does not matter.
#[test]
fn at_ans_07_reproducibility() {
    let dir = scratch("reproducible");
    let mut w = World::new(15, 1);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    let group = w.answer(q, &parts(11, 9, 1));
    w.close(q);
    let issuer = Issuer::new([1u8; 32]);
    let mut node = AnswerNode::open(&dir, issuer.public(), AnswerConfig::default()).unwrap();
    for bytes in &w.log {
        node.submit(AnswerEvent::decode(bytes).unwrap()).unwrap();
    }
    assert_eq!(node.state(), &w.path);
    drop(node);
    let copy = scratch("reproducible-copy");
    fs::create_dir_all(&copy).unwrap();
    for f in ["log", "objects"] {
        fs::copy(dir.join(f), copy.join(f)).unwrap();
    }
    let elsewhere = AnswerNode::open(&copy, issuer.public(), AnswerConfig::default()).unwrap();
    assert_eq!(elsewhere.state(), &w.path);
    let mut reversed = World::joining((100..115).rev(), 1);
    let q = reversed.deposit(0, &question("q")).unwrap();
    reversed.close_epoch();
    assert_eq!(reversed.draw(q), group);
    let mut distinct = group.clone();
    distinct.dedup();
    assert_eq!((distinct.len(), group.len()), (12, 12));
    assert!(group.windows(2).all(|p| p[0] < p[1]));
    assert!(group.iter().all(|r| w.path.pool().contains(r)));
}

// AT-ANS-08: divided answers weigh 1/4 in both sums; a discard weighs 1; too few answers nothing.
#[test]
fn at_ans_08_the_fractional_contribution() {
    let mut w = World::new(12, 3);
    let q: Vec<u64> = (0..3)
        .map(|i| w.deposit(i, &question(&format!("q{i}"))).unwrap())
        .collect();
    w.close_epoch();
    assert_eq!(w.round(q[0], &parts(12, 6, 0)).verdict, Verdict::Divided);
    assert_eq!(w.round(q[1], &parts(12, 0, 0)).verdict, Verdict::Discarded);
    assert_eq!(
        w.round(q[2], &parts(8, 8, 0)).verdict,
        Verdict::InsufficientParticipation
    );
    w.close_epoch();
    let prior = AuthorPrior::default();
    let divided = author_score_weighted(&[0.0], &[0.25], &[1], 45, &prior);
    let discarded = author_score_weighted(&[0.0], &[1.0], &[1], 45, &prior);
    assert_eq!(w.path.reputation(&w.author(0)).to_bits(), divided.to_bits());
    assert_eq!(
        w.path.reputation(&w.author(1)).to_bits(),
        discarded.to_bits()
    );
    assert_eq!(w.path.reputation(&w.author(2)), 0.4);
    assert!((divided - 2.0 / (5.0 + 0.25 * w1())).abs() < 1e-15);
    assert!(discarded < divided && divided < 0.4);
    assert_eq!(w.path.author(&w.author(2)), None);
}

// AT-ANS-09: an outcome counts from the next epoch; the quota is fixed for its epoch.
#[test]
fn at_ans_09_the_epoch_passage() {
    let mut w = World::new(12, 1);
    let a = w.author(0);
    let q0 = w.deposit(0, &question("q0")).unwrap();
    let q1 = w.deposit(0, &question("q1")).unwrap();
    w.close_epoch();
    w.deposit(0, &question("q2")).unwrap();
    assert_eq!(w.path.slots(&a, 1), Some(Slots { quota: 2, used: 1 }));
    w.round(q0, &parts(12, 0, 0));
    w.round(q1, &parts(12, 1, 0));
    assert_eq!(w.path.reputation(&a), 0.4);
    assert_eq!(w.path.quota(&a), 2);
    w.deposit(0, &question("q3")).unwrap();
    assert_eq!(
        w.deposit(0, &question("q4")),
        Err(Refused::OverQuota { quota: 2 })
    );
    w.close_epoch();
    assert!(w.path.reputation(&a) < 0.30);
    assert_eq!(w.path.quota(&a), 1);
    w.deposit(0, &question("q5")).unwrap();
    assert_eq!(
        w.deposit(0, &question("q6")),
        Err(Refused::OverQuota { quota: 1 })
    );
    w.close_epoch();
    assert_eq!(w.path.quota(&a), 1);
    assert_eq!(w.path.closings(), 3);
    assert_eq!(w.replay(), w.path);
}

// AT-ANS-10: a new author gets 2; six acceptances over three epochs lift the quota to 3.
#[test]
fn at_ans_10_the_quota_bands_through_the_path() {
    let mut w = World::new(12, 1);
    let a = w.author(0);
    assert_eq!(w.path.quota(&a), 2);
    let mut pending: Vec<u64> = Vec::new();
    let mut quotas = Vec::new();
    for epoch in 0..5 {
        quotas.push(w.path.quota(&a));
        for q in pending.drain(..) {
            assert_eq!(w.round(q, &parts(12, 12, 0)).verdict, Verdict::Accepted);
        }
        if epoch < 3 {
            for i in 0..2 {
                pending.push(w.deposit(0, &question(&format!("e{epoch}q{i}"))).unwrap());
            }
        }
        w.close_epoch();
    }
    quotas.push(w.path.quota(&a));
    assert_eq!(quotas, [2, 2, 2, 2, 3, 3]);
    assert!(w.path.reputation(&a) >= 0.70);
    assert_eq!(w.path.usable().count(), 6);
}

// AT-ANS-11: a pool short of 12 at the deposit epoch's closure is an explicit unavailability.
#[test]
fn at_ans_11_an_unavailable_group_is_never_reduced() {
    let mut w = World::new(11, 1);
    let a = w.author(0);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    w.join(200);
    assert_eq!(w.path.pool().len(), 12);
    let unavailable = w.submit(AnswerEvent::Draw { question: q }).unwrap();
    assert_eq!(
        unavailable,
        Applied::Unavailable {
            available: 11,
            required: 12
        }
    );
    assert_eq!(
        w.path.entry(q).unwrap().round,
        Round::Unavailable { available: 11 }
    );
    assert_eq!(
        w.submit(AnswerEvent::Draw { question: q }),
        Err(Refused::WrongRound)
    );
    assert_eq!(w.path.closure(q), None);
    assert_eq!(w.path.slots(&a, 0), Some(Slots { quota: 2, used: 1 }));
    let later = w.deposit(0, &question("later")).unwrap();
    w.close_epoch();
    assert_eq!(w.draw(later).len(), 12);
    assert_eq!(w.path.author(&a), None);
}

// AT-ANS-12: only drawn reviewers commit, the proof bound to the commitment; reveals must open it.
#[test]
fn at_ans_12_commit_reveal() {
    let mut w = World::new(13, 1);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    let view = w.path.view(q).unwrap();
    assert_eq!(view, question("q").view());
    assert_eq!(
        view,
        Question {
            key: 2,
            ..question("q")
        }
        .view()
    );
    let group = w.draw(q);
    let outsider = *w.reviewers.keys().find(|r| !group.contains(r)).unwrap();
    let not_drawn = w.commit_event(q, outsider, Alternative(KEY), [0; 32]);
    assert_eq!(w.submit(not_drawn), Err(Refused::NotDrawn));
    let (r0, r1) = (group[0], group[1]);
    let theirs = commit_answer(Alternative(KEY), &[0; 32], r1, q);
    let proof = prove(
        &w.reviewers[&r0],
        &w.issuer.public(),
        Role::Judge,
        &commit_context(q, &theirs),
    );
    let stolen = AnswerEvent::Commit {
        question: q,
        commitment: commit_answer(DontKnow, &[0; 32], r0, q),
        proof,
    };
    assert_eq!(w.submit(stolen), Err(Refused::Unproven(Unproven::BadProof)));
    let first = w.commit_event(q, r0, Alternative(9), [0; 32]);
    w.submit(first).unwrap();
    let again = w.commit_event(q, r0, Alternative(KEY), [1; 32]);
    assert_eq!(w.submit(again), Err(Refused::AlreadyCommitted));
    let second = w.commit_event(q, r1, Alternative(1), [1; 32]);
    w.submit(second).unwrap();
    w.submit(AnswerEvent::CloseCommits { question: q }).unwrap();
    let reveal = |reviewer, answer, nonce| AnswerEvent::Reveal {
        question: q,
        reviewer,
        answer,
        nonce,
    };
    assert_eq!(
        w.submit(reveal(r1, Alternative(2), [1; 32])),
        Err(Refused::RevealMismatch)
    );
    assert_eq!(
        w.submit(reveal(r1, Alternative(1), [0; 32])),
        Err(Refused::RevealMismatch)
    );
    assert_eq!(
        w.submit(reveal(group[2], DontKnow, [2; 32])),
        Err(Refused::NoCommit)
    );
    assert_eq!(
        w.submit(reveal(r0, Alternative(9), [0; 32])),
        Err(Refused::AnswerOutOfRange)
    );
    w.submit(reveal(r1, Alternative(1), [1; 32])).unwrap();
    assert_eq!(
        w.submit(reveal(r1, Alternative(1), [1; 32])),
        Err(Refused::AlreadyRevealed)
    );
    let closure = w.close(q);
    assert_eq!(
        closure.tally,
        Tally {
            alternatives: vec![0, 1, 0],
            none: 0,
            abstentions: 0,
            absences: 11
        }
    );
}

// AT-ANS-13: events re-encode to their bytes; hostile bytes, other configs, oversize refused.
#[test]
fn at_ans_13_codec_and_node_boundaries() {
    let mut w = World::new(12, 1);
    let q = w.deposit(0, &question("q")).unwrap();
    w.close_epoch();
    w.round(q, &parts(9, 9, 0));
    for bytes in &w.log {
        let event = AnswerEvent::decode(bytes).unwrap();
        assert_eq!(&event.encode(), bytes);
        assert!(AnswerEvent::decode(&bytes[..bytes.len() - 1]).is_none());
        assert!(AnswerEvent::decode(&[bytes.as_slice(), &[0]].concat()).is_none());
        let mut version = bytes.clone();
        version[0] = 2;
        assert!(AnswerEvent::decode(&version).is_none());
    }
    let reveal = AnswerEvent::Reveal {
        question: 0,
        reviewer: Nym([1; 32]),
        answer: DontKnow,
        nonce: [0; 32],
    };
    let mut bad_answer = reveal.encode();
    bad_answer[1 + 1 + 8 + 32] = 3;
    assert!(AnswerEvent::decode(&bad_answer).is_none());
    assert!(AnswerEvent::decode(&[1, 9]).is_none());

    let dir = scratch("boundaries");
    let issuer = Issuer::new([1u8; 32]);
    let mut node = AnswerNode::open(&dir, issuer.public(), AnswerConfig::default()).unwrap();
    for bytes in &w.log {
        node.submit(AnswerEvent::decode(bytes).unwrap()).unwrap();
    }
    let huge = Question {
        context: vec![b'x'; MAX_OBJECT],
        ..question("huge")
    };
    let oversize = AnswerEvent::Deposit {
        epoch: 1,
        question: huge,
        proof: prove(&w.authors[0].1, &issuer.public(), Role::Propose, b"x"),
    };
    assert_eq!(
        node.submit(oversize).unwrap_err(),
        AnswerNodeError::TooLarge
    );
    assert_eq!(node.state(), &w.path);
    let refused = node.submit(AnswerEvent::Close { question: q }).unwrap_err();
    assert_eq!(refused, AnswerNodeError::Refused(Refused::WrongRound));
    assert_eq!(node.events(), w.log.len());
    drop(node);
    let other = AnswerConfig {
        divided_weight: 0.5,
        ..AnswerConfig::default()
    };
    assert_eq!(
        AnswerNode::open(&dir, issuer.public(), other).err(),
        Some(AnswerNodeError::Configuration)
    );
    let invalid = AnswerConfig {
        divided_weight: 0.0,
        ..AnswerConfig::default()
    };
    assert!(AnswerPath::new(invalid).is_none());
    let reopened = AnswerNode::open(&dir, issuer.public(), AnswerConfig::default()).unwrap();
    assert_eq!(reopened.state(), &w.path);
}
