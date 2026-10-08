//! The fixtures of `docs/20` §§3–4 and §6: what they supply — feeds, cuts, held records, gate
//! inputs, declared counts — apart from what the check derives from the log.

use super::records::Reading::{NotConverged as Nc, Survives as Sv, A as Ra, R as Rr};
use super::records::{
    Attempt, Construction, Group, Outcome, Reading, Reason, Record, Refused, Setup, Shortfall,
    Terminal,
};
use ed25519_dalek::Signature;
use identity::credential::{Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::{cid, Cid};
use network::consortium::{Consortium, Member};
use network::cut::{sign_cut, Cut};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica, SignedEntry, WriterSet};
use protocol::admission::NullifierSet;
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::gate::{bridging_gate, GateOutcome, APPEAL_GAP, EPS, TAU};
use protocol::lifecycle::Event;
use protocol::pilot::{admit_dif_batch, screen, PilotError};
use protocol::revalidation::N_LATENT_MIN;
use protocol::review::commit;
use scoring::latent::Formats;

pub const NET: [u8; 32] = [5; 32];
pub const M0: u8 = 10;
pub const M1: u8 = 11;
pub const RELAY: u8 = 20;
pub const U: u8 = 1;
pub const V: u8 = 2;
pub const W: u8 = 3;
pub const OMEGA: u64 = 4;
pub const BUDGET: u64 = 2;
pub const ANCHORS: u64 = 2;
const STAGE1: u64 = 300;
const STAGE2: u64 = 3_000;
const SHORT: usize = 120;

pub fn member(i: usize) -> Member {
    Member::from_seed([10 + i as u8; 32])
}

pub fn consortium() -> Consortium {
    Consortium::new((0..3).map(|i| member(i).public()).collect(), 2)
}

pub fn issuer() -> Issuer {
    Issuer::new([1; 32])
}

pub fn nym(b: u8) -> Nym {
    Nym([b; 32])
}

pub fn key(seed: u8) -> [u8; 32] {
    FeedWriter::from_seed(NET, [seed; 32]).public().to_bytes()
}

fn writer_set() -> WriterSet {
    let keys: Vec<_> = [M0, M1, 12, RELAY]
        .iter()
        .map(|s| FeedWriter::from_seed(NET, [*s; 32]).public())
        .collect();
    WriterSet::new(NET, &keys)
}

/// Members 0 and 1's signatures of `cut`, as `Ledger::apply` takes them.
pub fn signatures(cut: &Cut) -> Vec<(usize, Signature)> {
    let cp = cut.checkpoint(NET, consortium().member_set_hash());
    (0..2).map(|i| (i, member(i).sign(&cp))).collect()
}

pub type Entry = (SignedEntry, Vec<u8>);

pub struct Feed {
    writer: FeedWriter,
    log: TransparencyLog,
}

impl Feed {
    pub fn new(seed: u8) -> Feed {
        Feed {
            writer: FeedWriter::from_seed(NET, [seed; 32]),
            log: TransparencyLog::new(),
        }
    }

    /// The feed of `seed` as `replica` holds it, ready to sign the next entry.
    pub fn resume(seed: u8, replica: &Replica) -> Feed {
        let mut feed = Feed::new(seed);
        for e in replica.feed(&key(seed)) {
            feed.log.append(e.entry.payload);
        }
        feed
    }

    pub fn sign(&mut self, object: Vec<u8>) -> Entry {
        let entry = self.writer.sign(self.log.append(cid(&object)));
        (entry, object)
    }

    pub fn push(&mut self, replica: &mut Replica, object: Vec<u8>) -> EntryId {
        let (entry, object) = self.sign(object);
        let id = entry.id();
        replica
            .insert(entry, object)
            .expect("an entry of a known writer");
        id
    }
}

pub fn draft(tag: &str) -> Draft {
    Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"synthetic source".to_vec(),
    }
}

/// A deposit by test author `secret`, quota one, with a valid proof (`ledger.rs`'s support).
pub fn deposit(issuer: &Issuer, secret: u8, draft: Draft) -> Vec<u8> {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    let author = pending.finalize(issuer.issue(&req).unwrap());
    let context = deposit_context(draft.content_id(), 0);
    let proof = prove(&author, &issuer.public(), Role::Propose, &context);
    NodeEvent::Deposit {
        epoch: 0,
        quota: 1,
        draft,
        proof,
    }
    .encode()
}

pub fn step(item: Cid, event: Event) -> Vec<u8> {
    NodeEvent::Step { item, event }.encode()
}

fn nonce(n: u8, item: Cid) -> [u8; 32] {
    let mut x = item.0;
    x[0] ^= n;
    x
}

pub fn commit_of(n: u8, prob: f64, item: Cid) -> Event {
    Event::Commit {
        nym: nym(n),
        commitment: commit(prob, &nonce(n, item), nym(n), item),
    }
}

pub fn reveal_of(n: u8, prob: f64, item: Cid) -> Event {
    Event::Reveal {
        nym: nym(n),
        prob,
        nonce: nonce(n, item),
    }
}

pub enum Then {
    Nothing,
    Extra(Vec<u8>, GateOutcome),
    Expires,
}

/// One row of `docs/20` §4's item table; a reveal not listed is ½.
pub struct Spec {
    pub group: u8,
    pub panel: Vec<u8>,
    pub reveals: &'static [(u8, f64)],
    pub missing: Option<u8>,
    pub inputs: (f64, f64, usize),
    pub then: Then,
}

impl Spec {
    pub fn prob(&self, n: u8) -> f64 {
        self.reveals
            .iter()
            .find(|(m, _)| *m == n)
            .map_or(0.5, |(_, p)| *p)
    }
}

const PASS: (f64, f64, usize) = (0.90, 0.05, 3);
const POLARIZED: (f64, f64, usize) = (0.50, 0.40, 2);

fn spec(group: u8, inputs: (f64, f64, usize), then: Then) -> Spec {
    Spec {
        group,
        panel: (4..=10).collect(),
        reveals: &[],
        missing: None,
        inputs,
        then,
    }
}

pub fn specs() -> Vec<Spec> {
    let scored =
        |group: u8, panel: &[u8], reveals: &'static [(u8, f64)], inputs: (f64, f64, usize)| Spec {
            group,
            panel: panel.to_vec(),
            reveals,
            missing: None,
            inputs,
            then: Then::Nothing,
        };
    vec![
        scored(1, &[U, V, W, 4, 5, 6, 7], &[(U, 0.75), (V, 0.25)], PASS),
        scored(
            1,
            &[V, 4, 5, 6, 7, 8, 9],
            &[(V, 0.75), (4, 0.25), (9, 1.0)],
            (0.50, 0.10, 2),
        ),
        scored(2, &[U, V, 4, 5, 6, 7, 8], &[(U, 0.25), (V, 0.75)], PASS),
        spec(
            2,
            (0.80, 0.05, 2),
            Then::Extra(vec![11, 12, 13, 14], GateOutcome::Reject),
        ),
        scored(3, &[U, W, 4, 5, 6, 7, 8], &[(U, 0.75), (W, 0.25)], PASS),
        spec(3, POLARIZED, Then::Expires),
        spec(4, PASS, Then::Nothing),
        spec(4, PASS, Then::Nothing),
        spec(5, PASS, Then::Nothing),
        spec(5, PASS, Then::Nothing),
        Spec {
            missing: Some(U),
            ..scored(6, &[U, 4, 5, 6, 7, 8, 9], &[], PASS)
        },
        spec(6, PASS, Then::Nothing),
        spec(7, POLARIZED, Then::Nothing),
        spec(7, PASS, Then::Nothing),
    ]
}

/// Per group: construction, term (a cut) and, under T2, the cut after which it produces.
pub const GROUPS: [(Construction, u64, Option<u64>); 7] = [
    (Construction::T1, 4, None),
    (Construction::T1, 4, None),
    (Construction::T1, 6, None),
    (Construction::T1, 3, None),
    (Construction::T1, 3, None),
    (Construction::T2, 3, Some(1)),
    (Construction::T1, 6, None),
];

/// Group, stage, attempt, readings in batch order, cut (`None`: held), optimizer runs.
type AttemptRow = (u8, u8, u8, [Reading; 2], Option<u64>, u64);
const ATTEMPTS: [AttemptRow; 12] = [
    (1, 1, 1, [Sv, Sv], Some(2), 0),
    (2, 1, 1, [Sv, Sv], Some(2), 0),
    (3, 1, 1, [Sv, Sv], Some(2), 0),
    (5, 1, 1, [Rr, Sv], Some(2), 0),
    (1, 2, 1, [Rr, Ra], Some(3), 17),
    (2, 2, 1, [Nc, Rr], Some(3), 9),
    (3, 2, 1, [Nc, Nc], Some(3), 25),
    (2, 2, 2, [Nc, Ra], Some(4), 17),
    (3, 2, 2, [Ra, Nc], Some(5), 17),
    (6, 1, 1, [Sv, Sv], None, 0),
    (6, 2, 1, [Nc, Ra], None, 17),
    (6, 2, 2, [Nc, Rr], None, 9),
];

/// Item (0-based), outcome, reason, the (stage, attempt) it rests on, cut (`None`: held).
type TerminalRow = (usize, Outcome, Reason, &'static [(u8, u8)], Option<u64>);
const TERMINALS: [TerminalRow; 9] = [
    (0, Outcome::R, Reason::Concluded, &[(1, 1), (2, 1)], Some(3)),
    (1, Outcome::A, Reason::Concluded, &[(1, 1), (2, 1)], Some(3)),
    (3, Outcome::R, Reason::Concluded, &[(1, 1), (2, 1)], Some(3)),
    (8, Outcome::R, Reason::Concluded, &[(1, 1)], Some(3)),
    (
        2,
        Outcome::I,
        Reason::Exhausted,
        &[(1, 1), (2, 1), (2, 2)],
        Some(4),
    ),
    (
        4,
        Outcome::A,
        Reason::Concluded,
        &[(1, 1), (2, 1), (2, 2)],
        Some(5),
    ),
    (
        5,
        Outcome::I,
        Reason::Exhausted,
        &[(1, 1), (2, 1), (2, 2)],
        Some(5),
    ),
    (
        10,
        Outcome::I,
        Reason::Exhausted,
        &[(1, 1), (2, 1), (2, 2)],
        None,
    ),
    (11, Outcome::A, Reason::Concluded, &[(1, 1), (2, 1)], None),
];

/// The main scenario (`docs/20` §4): M0's feed, cuts 0–5, and G6's records, produced and held.
pub struct Main {
    pub items: Vec<Cid>,
    pub specs: Vec<Spec>,
    pub replica: Replica,
    pub cuts: Vec<Cut>,
    pub held: Vec<Record>,
    pub gate: Vec<((f64, f64, usize), GateOutcome)>,
    pub screen: PilotError,
    pub batch: PilotError,
}

fn gate((score, gap, coverage): (f64, f64, usize)) -> GateOutcome {
    bridging_gate(score, gap, coverage, TAU, EPS, APPEAL_GAP)
}

/// Cut 1's steps for one item; returns the offset of its `Score`.
fn review(steps: &mut Vec<Vec<u8>>, item: Cid, s: &Spec, decision: GateOutcome) -> usize {
    let mut push = |e| steps.push(step(item, e));
    let panel = s.panel.iter().map(|n| nym(*n)).collect();
    push(Event::AssignReviewers { panel, item });
    for n in &s.panel {
        push(commit_of(*n, s.prob(*n), item));
    }
    push(Event::CloseCommits);
    for n in s.panel.iter().filter(|n| Some(**n) != s.missing) {
        push(reveal_of(*n, s.prob(*n), item));
    }
    let at = steps.len();
    steps.push(step(item, Event::Score { outcome: decision }));
    let mut push = |e| steps.push(step(item, e));
    match &s.then {
        Then::Nothing => {}
        Then::Extra(extra, outcome) => {
            let panel = extra.iter().map(|n| nym(*n)).collect();
            push(Event::AssignExtraReviewers { panel });
            for n in extra {
                push(commit_of(*n, 0.5, item));
            }
            push(Event::CloseCommits);
            for n in extra {
                push(reveal_of(*n, 0.5, item));
            }
            push(Event::Resolve { outcome: *outcome });
        }
        Then::Expires => push(Event::AppealExpires),
    }
    at
}

fn respondent(i: usize) -> Nym {
    let mut b = [0xEE; 32];
    b[..8].copy_from_slice(&(i as u64).to_le_bytes());
    Nym(b)
}

impl Main {
    pub fn build() -> Main {
        let issuer = issuer();
        let specs = specs();
        let items: Vec<Cid> = (1..=14)
            .map(|n| draft(&format!("s2 j{n}")).content_id())
            .collect();
        let decisions: Vec<GateOutcome> = specs.iter().map(|s| gate(s.inputs)).collect();
        let mut cuts: Vec<Vec<Vec<u8>>> = vec![Vec::new(); 6];
        let mut score_at = Vec::new();
        for (j, s) in specs.iter().enumerate() {
            score_at.push(review(&mut cuts[1], items[j], s, decisions[j]));
        }
        let in_group = |g: u8| {
            (0..14)
                .filter(|j| specs[*j].group == g)
                .collect::<Vec<usize>>()
        };
        let cut0_len = 1 + GROUPS.len() + 2 * items.len();
        let groups: Vec<Group> = GROUPS
            .iter()
            .zip(1u8..)
            .map(|((construction, term, production), g)| Group {
                group: g,
                items: in_group(g).into_iter().map(|j| items[j]).collect(),
                construction: *construction,
                budget: BUDGET,
                term: *term,
                reveal_close: in_group(g)
                    .into_iter()
                    .map(|j| (cut0_len + score_at[j]) as u64)
                    .collect(),
                production: *production,
            })
            .collect();
        let weight = |b: u8| match b {
            4 => 2,
            9 => 0,
            _ => 1,
        };
        let cohort = |js: &[usize]| js.iter().map(|j| items[*j]).collect();
        let setup = Setup {
            omega: OMEGA,
            weights: (1..=14).map(|b| (nym(b), weight(b))).collect(),
            cohorts: vec![
                (nym(U), cohort(&[0, 2, 4, 10])),
                (nym(V), cohort(&[0, 1, 2])),
                (nym(W), cohort(&[0, 4])),
            ],
        };
        cuts[0].push(Record::Setup(setup).encode());
        cuts[0].extend(groups.iter().map(|g| Record::Group(g.clone()).encode()));
        for (j, item) in items.iter().enumerate() {
            let tag = format!("s2 j{}", j + 1);
            cuts[0].push(deposit(&issuer, 100 + j as u8, draft(&tag)));
            let admit = Event::Admit {
                seed_from_beacon: true,
            };
            cuts[0].push(step(*item, admit));
        }
        let mut held = vec![Record::Start(6)];
        cuts[2].extend((1..=5).map(|g| Record::Start(g).encode()));
        let mut attempt_cids = Vec::new();
        for (g, stage, attempt, readings, cut, runs) in ATTEMPTS {
            let all: Vec<Cid> = in_group(g).into_iter().map(|j| items[j]).collect();
            let batch = if stage == 1 {
                all
            } else {
                let first = ATTEMPTS.iter().find(|a| a.0 == g && a.1 == 1).unwrap();
                all.into_iter()
                    .zip(first.3)
                    .filter(|(_, r)| *r == Sv)
                    .map(|(c, _)| c)
                    .collect()
            };
            let record = Record::Attempt(Attempt {
                group: g,
                stage,
                attempt,
                readings: readings[..batch.len()].to_vec(),
                batch,
                evidence: cid(format!("evidence G{g} s{stage} a{attempt}").as_bytes()).0,
                participations: if stage == 1 { STAGE1 } else { STAGE2 },
                runs,
            });
            attempt_cids.push(((g, stage, attempt), record.cid()));
            match cut {
                Some(c) => cuts[c as usize].push(record.encode()),
                None => held.push(record),
            }
        }
        let mut respondents = NullifierSet::new();
        for i in 0..SHORT {
            respondents.spend(respondent(i)).unwrap();
        }
        let screen = screen(&respondents, &[], &[], &Formats::open(0, 0))
            .expect_err("120 admitted is short of N1_MIN");
        let batch = admit_dif_batch(1, STAGE2 as usize, N_LATENT_MIN)
            .expect_err("a stage-2 batch of one item");
        let (
            PilotError::NotEnoughRespondents { have, need },
            PilotError::BatchTooSmall { items: n },
        ) = (screen, batch)
        else {
            panic!("the gates returned {screen:?} and {batch:?}");
        };
        cuts[3].push(
            Record::Shortfall(Shortfall {
                group: 4,
                stage: 1,
                batch: vec![items[6], items[7]],
                have: have as u64,
                need: need as u64,
            })
            .encode(),
        );
        cuts[3].push(
            Record::Refused(Refused {
                group: 5,
                stage: 2,
                batch: vec![items[9]],
                items: n as u64,
                participations: STAGE2,
            })
            .encode(),
        );
        for (j, outcome, reason, rests, cut) in TERMINALS {
            let g = specs[j].group;
            let record = Record::Terminal(Terminal {
                item: items[j],
                rule: Record::Group(groups[usize::from(g) - 1].clone()).cid(),
                outcome,
                reason,
                attempts: rests
                    .iter()
                    .map(|(s, a)| {
                        attempt_cids
                            .iter()
                            .find(|(k, _)| *k == (g, *s, *a))
                            .unwrap()
                            .1
                    })
                    .collect(),
            });
            match cut {
                Some(c) => cuts[c as usize].push(record.encode()),
                None => held.push(record),
            }
        }
        let mut feed = Feed::new(M0);
        let mut replica = Replica::new(writer_set());
        let mut signed: Vec<Cut> = Vec::new();
        for objects in cuts {
            for o in objects {
                feed.push(&mut replica, o);
            }
            signed.push(Cut::next(&replica, signed.last(), 0, false));
        }
        let mut gate_rows: Vec<_> = specs.iter().map(|s| s.inputs).zip(decisions).collect();
        gate_rows.push(((0.95, 0.00, 0), gate((0.95, 0.00, 0))));
        Main {
            items,
            specs,
            replica,
            cuts: signed,
            held,
            gate: gate_rows,
            screen,
            batch,
        }
    }
}

/// The order fixtures of `docs/20` §6.
#[derive(Clone, Copy, Debug)]
pub enum Fixture {
    O1,
    O2,
    O3,
    O4,
    O5,
    O6a,
    O6b,
}

/// What a verifier holds for an order fixture: its replica, the signed cuts, `k` and `x`.
pub struct Order {
    pub replica: Replica,
    pub cuts: Vec<Cut>,
    pub k: Cid,
    pub x: EntryId,
}

/// `k`'s deposit, admission and assignment to seven synthetic nyms, then its seven `Commit`;
/// apart, its `CloseCommits`.
fn base() -> (Cid, Vec<Vec<u8>>, Vec<u8>) {
    let d = draft("s2 order k");
    let k = d.content_id();
    let panel: Vec<u8> = (4..=10).collect();
    let admit = Event::Admit {
        seed_from_beacon: true,
    };
    let assign = Event::AssignReviewers {
        panel: panel.iter().map(|n| nym(*n)).collect(),
        item: k,
    };
    let mut base = vec![deposit(&issuer(), 200, d), step(k, admit), step(k, assign)];
    base.extend(panel.iter().map(|n| step(k, commit_of(*n, 0.5, k))));
    (k, base, step(k, Event::CloseCommits))
}

/// Where `base` puts the third `Commit`.
const THIRD_COMMIT: usize = 5;

/// `x`: a terminal record of an item of another group, a study object.
pub fn disclosure() -> Vec<u8> {
    Record::Terminal(Terminal {
        item: draft("s2 another item").content_id(),
        rule: cid(b"another group"),
        outcome: Outcome::A,
        reason: Reason::Concluded,
        attempts: Vec::new(),
    })
    .encode()
}

fn cut_after(replica: &Replica, cuts: &mut Vec<Cut>) {
    let next = Cut::next(replica, cuts.last(), 0, false);
    cuts.push(next);
}

fn signature(i: usize, cut: &Cut) -> Vec<u8> {
    sign_cut(&member(i), i, NET, &consortium(), cut).encode()
}

pub fn order(fixture: Fixture) -> Order {
    let (k, base, close) = base();
    let mut replica = Replica::new(writer_set());
    let mut cuts = Vec::new();
    let (mut m0, mut m1, mut relay) = (Feed::new(M0), Feed::new(M1), Feed::new(RELAY));
    let mut on_m0 = |replica: &mut Replica, objects: &[Vec<u8>]| {
        objects
            .iter()
            .map(|o| m0.push(replica, o.clone()))
            .collect::<Vec<_>>()
    };
    let x = match fixture {
        Fixture::O1 => {
            on_m0(&mut replica, &[base, vec![close]].concat());
            let x = on_m0(&mut replica, &[disclosure()])[0];
            cut_after(&replica, &mut cuts);
            x
        }
        Fixture::O2 | Fixture::O6a => {
            let ids = on_m0(&mut replica, &[base, vec![close]].concat());
            cut_after(&replica, &mut cuts);
            m1.push(&mut replica, signature(1, &cuts[0]));
            let x = m1.push(&mut replica, disclosure());
            cut_after(&replica, &mut cuts);
            if matches!(fixture, Fixture::O6a) {
                replica = without(&replica, ids[THIRD_COMMIT]);
            }
            x
        }
        Fixture::O3 | Fixture::O6b => {
            let (entry, object) = relay.sign(disclosure());
            on_m0(&mut replica, &[base, vec![close]].concat());
            cut_after(&replica, &mut cuts);
            let x = entry.id();
            replica.insert(entry, object).unwrap();
            if matches!(fixture, Fixture::O3) {
                cut_after(&replica, &mut cuts);
            }
            x
        }
        Fixture::O4 | Fixture::O5 => {
            on_m0(&mut replica, &base);
            let x = m1.push(&mut replica, disclosure());
            cut_after(&replica, &mut cuts);
            if matches!(fixture, Fixture::O4) {
                on_m0(&mut replica, &[signature(0, &cuts[0]), close]);
                cut_after(&replica, &mut cuts);
            }
            x
        }
    };
    Order {
        replica,
        cuts,
        k,
        x,
    }
}

fn without(replica: &Replica, gone: EntryId) -> Replica {
    let mut kept = Replica::new(writer_set());
    for id in replica.ids().filter(|id| *id != gone) {
        let (e, o) = replica.get(&id).unwrap();
        kept.insert(e.clone(), o.to_vec()).unwrap();
    }
    kept
}
