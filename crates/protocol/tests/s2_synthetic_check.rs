//! S2's synthetic-event technical check (`docs/20`): fixtures F1–F9 and O1–O7 through the real
//! feeds, cuts, ledger, lifecycle, gates and baselines, against `docs/20`'s expected values; the
//! absent baseline (§10) and A1's adaptive verification (§11) through its scoring path.

mod s2;

use network::cid::{cid, Cid};
use network::cut::{Cut, CutError, Mark, MemberObject};
use protocol::events::NodeEvent;
use protocol::gate::{bridging_gate, GateOutcome, APPEAL_GAP, EPS, MIN_COVERAGE, TAU};
use protocol::ledger::{LedgerError, Refusal};
use protocol::lifecycle::{Event, Invalid, RejectReason, State};
use protocol::node::Rejection;
use protocol::pilot::PilotError;
use s2::adaptive::{self, Construction, Law, ONE, Q, ZERO};
use s2::fixture::{
    consortium, deposit, disclosure, draft, issuer, key, nym, order, step, Feed, Fixture, Main,
    Then, M0, M1, OMEGA, RELAY, U, V, W,
};
use s2::order::{
    commitments, existence_level, replay_level, Evidence, Precedence, References, Witness,
};
use s2::records::{Attempt, Outcome, Reading, Reason, Record, Terminal};
use s2::replay::LogView;
use s2::study::{
    cohorts_complete, member, t1_start, t2_hold, validate, Case, Cohort, CohortError, ItemState,
    Pending, SequenceError, Snapshot, Start, StartError, TerminalError, Undefined, Value,
};
use std::collections::BTreeMap;
use std::sync::OnceLock;

struct Run {
    main: Main,
    view: LogView,
    at_omega: Snapshot,
    omega_later: Snapshot,
    after: Snapshot,
}

/// The snapshot at `Ω` taken before cut 5 is applied, then again from the replay through cut
/// 5, and the state after cut 5.
fn replayed(main: Main) -> Run {
    let through_omega = LogView::replay(&main.replica, &main.cuts[..=4], &main.items);
    let at_omega = Snapshot::of(&main, &through_omega, OMEGA);
    let view = LogView::replay(&main.replica, &main.cuts, &main.items);
    let omega_later = Snapshot::of(&main, &view, OMEGA);
    let after = Snapshot::of(&main, &view, 5);
    Run {
        main,
        view,
        at_omega,
        omega_later,
        after,
    }
}

/// The main scenario, built once.
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| replayed(Main::build()))
}

/// `docs/20` §10's variant: `S0` weighs `w` 1 and every other nym 0, so (w, j1), (v, j2) and
/// (w, j5) get no baseline.
fn only_w() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| replayed(Main::build_with(|b| u64::from(b == W))))
}

fn q(num: i64, den: i64) -> f64 {
    num as f64 / den as f64
}

fn near(x: f64, num: i64, den: i64) -> bool {
    (x - q(num, den)).abs() <= 1e-12
}

fn cut_of(pos: u64) -> u64 {
    run().view.examined[pos as usize].cut
}

fn terminal(s: &Snapshot, j: usize) -> (Outcome, u64) {
    match s.items[j].state {
        ItemState::Terminal { outcome, pos } => (outcome, cut_of(pos)),
        other => panic!("j{} is {other:?}", j + 1),
    }
}

fn cases(s: &Snapshot, j: usize) -> Vec<u8> {
    let of = s.assignments.iter().filter(|a| a.item == j);
    of.map(|a| a.case.number()).collect()
}

fn by_case(s: &Snapshot) -> BTreeMap<u8, usize> {
    let mut counts = BTreeMap::new();
    for a in &s.assignments {
        *counts.entry(a.case.number()).or_default() += 1;
    }
    counts
}

fn cohort(s: &Snapshot, n: u8) -> &Cohort {
    s.cohorts.iter().find(|c| c.nym == nym(n)).unwrap()
}

/// The logged attempt records of `group` before `below`, as `validate` reads them.
fn attempts(group: u8, below: u64) -> Vec<(Attempt, Cid, u64)> {
    let records = run().view.records(5);
    let of = records.into_iter().filter(|(e, _)| e.pos < below);
    of.filter_map(|(e, r)| match r {
        Record::Attempt(a) if a.group == group => Some((a, cid(&e.object), e.cut)),
        _ => None,
    })
    .collect()
}

fn logged_record(pred: impl Fn(&Record) -> bool) -> (u64, Record) {
    let records = run().view.records(5);
    let (e, r) = records.into_iter().find(|(_, r)| pred(r)).unwrap();
    (e.pos, r)
}

fn the_terminal(j: usize) -> (u64, Terminal) {
    let item = run().main.items[j];
    match logged_record(|r| matches!(r, Record::Terminal(t) if t.item == item)) {
        (pos, Record::Terminal(t)) => (pos, t),
        _ => unreachable!(),
    }
}

/// F1: j1 `R` and j2 `A` come from their terminal records in cut 3, whatever the gate decided.
#[test]
fn f1_conclusive_outcomes_from_completed_reports() {
    let r = run();
    let s = &r.at_omega;
    assert_eq!(terminal(s, 0), (Outcome::R, 3));
    assert_eq!(terminal(s, 1), (Outcome::A, 3));
    assert_eq!(cases(s, 0), vec![4; 7]);
    assert_eq!(cases(s, 1), vec![3; 7]);
    let pilot1 = State::Pilot1 { appealed: false };
    assert_eq!(s.items[0].lifecycle, Some(pilot1));
    assert_eq!(
        s.items[1].lifecycle,
        Some(State::Rejected(RejectReason::Defect))
    );
    for e in &r.view.examined {
        match NodeEvent::decode(&e.object) {
            Some(NodeEvent::Step { event, .. }) => assert!(
                !matches!(
                    event,
                    Event::Pilot1Batch { .. }
                        | Event::Pilot2Batch { .. }
                        | Event::Explore { .. }
                        | Event::Administer
                        | Event::Revalidate { .. }
                        | Event::ExposureLimit
                ),
                "a pilot or pool step at {}",
                e.pos
            ),
            Some(NodeEvent::Deposit { .. }) | None => {}
            Some(_) => panic!("an admission or results event at {}", e.pos),
        }
    }
    for s in [&r.at_omega, &r.after] {
        for item in &s.items {
            assert!(!matches!(
                item.lifecycle,
                Some(
                    State::ActivePool
                        | State::Contested
                        | State::Explored { .. }
                        | State::Measured { .. }
                )
            ));
        }
    }
}

/// F2: j3's `I` is verifiable from its record and both stage-2 attempts; j4 keeps its first `R`.
#[test]
fn f2_verifiable_inconclusiveness() {
    let r = run();
    let s = &r.at_omega;
    assert!(s.invalid.is_empty(), "{:?}", s.invalid);
    assert_eq!(terminal(s, 2), (Outcome::I, 4));
    let (pos, t) = the_terminal(2);
    let (g_pos, group) = logged_record(|r| matches!(r, Record::Group(g) if g.group == 2));
    assert_eq!(t.rule, group.cid());
    assert_eq!((t.outcome, t.reason), (Outcome::I, Reason::Exhausted));
    let before = attempts(2, pos);
    assert_eq!(before.len(), 3);
    for (a, c, _) in &before {
        assert!(t.attempts.contains(c));
        assert_ne!(a.evidence, [0; 32]);
        let reading = a.readings[a.batch.iter().position(|i| *i == t.item).unwrap()];
        let expected = if a.stage == 1 {
            Reading::Survives
        } else {
            Reading::NotConverged
        };
        assert_eq!(reading, expected);
    }
    assert!(before.iter().all(|(_, _, cut)| *cut <= 4));
    let Record::Group(group) = group else {
        unreachable!()
    };
    let refs: Vec<_> = before.iter().map(|(a, c, cut)| (a, *c, *cut)).collect();
    assert_eq!(
        validate(&t, &group, cid(&group_bytes(2)), &refs),
        Ok(Outcome::I)
    );
    let first_commit = r.view.examined.iter().find(|e| {
        matches!(NodeEvent::decode(&e.object), Some(NodeEvent::Step { item, event: Event::Commit { .. } }) if item == t.item)
    });
    assert!(
        g_pos < first_commit.unwrap().pos,
        "the term is fixed before the reports"
    );
    assert_eq!(cases(s, 2), vec![5; 7]);
    for n in [U, V] {
        let m = cohort(s, n).members.iter().find(|m| m.item == 2).unwrap();
        assert_eq!(
            (m.case, m.baseline, m.contribution),
            (Case::Inconclusive, None, Some(0.0))
        );
    }
    assert_eq!(terminal(s, 3), (Outcome::R, 3));
    assert_eq!(s.items[3].unused, vec![(2, 2, Reading::A)]);
    assert_eq!(cases(s, 3), vec![4; 11]);
}

fn group_bytes(g: u8) -> Vec<u8> {
    let (_, record) = logged_record(|r| matches!(r, Record::Group(x) if x.group == g));
    record.encode()
}

/// F2: an `I` after one non-conclusion, or missing an attempt's reference, fails validation.
#[test]
fn f2_an_i_without_its_evidence_is_refused() {
    let (pos, t) = the_terminal(2);
    let (j4_pos, _) = the_terminal(3);
    let Record::Group(group) = Record::decode(&group_bytes(2)).unwrap() else {
        unreachable!()
    };
    let rule = cid(&group_bytes(2));
    let early = attempts(2, j4_pos);
    let early: Vec<_> = early.iter().map(|(a, c, cut)| (a, *c, *cut)).collect();
    let one = Terminal {
        attempts: early.iter().map(|(_, c, _)| *c).collect(),
        ..t.clone()
    };
    assert_eq!(
        validate(&one, &group, rule, &early),
        Err(TerminalError::Mismatch(s2::study::Procedure::Pending))
    );
    let all = attempts(2, pos);
    let all: Vec<_> = all.iter().map(|(a, c, cut)| (a, *c, *cut)).collect();
    assert_eq!(
        validate(&one, &group, rule, &all),
        Err(TerminalError::Unreferenced(2, 2))
    );
    let blind: Vec<Attempt> = all
        .iter()
        .map(|(a, _, _)| Attempt {
            evidence: [0; 32],
            ..(*a).clone()
        })
        .collect();
    let blind: Vec<_> = blind
        .iter()
        .zip(&all)
        .map(|(a, (_, c, cut))| (a, *c, *cut))
        .collect();
    assert_eq!(
        validate(&t, &group, rule, &blind),
        Err(TerminalError::NoEvidence(1, 1))
    );
}

/// F3: one indeterminate attempt leaves j5 and j6 pending at `Ω`; cut 5 concludes j5, exhausts j6.
#[test]
fn f3_an_indeterminate_attempt_is_not_exhaustion() {
    let r = run();
    let s = &r.at_omega;
    for j in [4, 5] {
        let pending = ItemState::Pending(Pending::AttemptIndeterminate);
        assert_eq!(s.items[j].state, pending);
        assert_eq!(cases(s, j), vec![2; 7]);
    }
    let g3 = s.groups.iter().find(|g| g.group == 3).unwrap();
    assert_eq!((g3.executed, g3.term), ([1, 1], 6));
    assert_eq!(terminal(&r.after, 4), (Outcome::A, 5));
    assert_eq!(terminal(&r.after, 5), (Outcome::I, 5));
}

/// F4: the gates' own refusals, a stage-1 shortfall and a refused stage-2 batch, are not `I`.
#[test]
fn f4_a_shortfall_and_a_refused_batch_apart_from_i() {
    let r = run();
    let s = &r.at_omega;
    let short = PilotError::NotEnoughRespondents {
        have: 120,
        need: 300,
    };
    assert_eq!(r.main.screen, short);
    assert_eq!(r.main.batch, PilotError::BatchTooSmall { items: 1 });
    let items = &r.main.items;
    let (pos, shortfall) = logged_record(|r| matches!(r, Record::Shortfall(_)));
    let Record::Shortfall(shortfall) = shortfall else {
        unreachable!()
    };
    assert_eq!(cut_of(pos), 3);
    assert_eq!((shortfall.have, shortfall.need), (120, 300));
    assert_eq!(shortfall.batch, vec![items[6], items[7]]);
    let (pos, refused) = logged_record(|r| matches!(r, Record::Refused(_)));
    let Record::Refused(refused) = refused else {
        unreachable!()
    };
    assert_eq!(cut_of(pos), 3);
    assert_eq!((refused.items, refused.batch.clone()), (1, vec![items[9]]));
    for (j, kind) in [
        (6, Pending::Shortfall),
        (7, Pending::Shortfall),
        (9, Pending::RefusedBatch),
    ] {
        assert_eq!(s.items[j].state, ItemState::Pending(kind));
        assert_eq!(cases(s, j), vec![2; 7]);
    }
    assert_eq!(terminal(s, 8), (Outcome::R, 3));
    let g = |n| s.groups.iter().find(|g| g.group == n).unwrap();
    assert_eq!(
        (g(4).executed, g(4).refused, g(4).term),
        ([0, 0], [1, 0], 3)
    );
    assert_eq!(
        (g(5).executed, g(5).refused, g(5).term),
        ([1, 0], [0, 1], 3)
    );
    assert!(s.groups.iter().all(|g| g.survivors_batched));
    assert_eq!(s.costs.searches.total(), 6, "the refusals ran no search");
}

/// F5: u's missing report on j11 is case 6 beside the held `I`; no record of G6 is logged.
#[test]
fn f5_a_missing_report_beside_a_held_outcome() {
    let r = run();
    for s in [&r.at_omega, &r.after] {
        let u = s
            .assignments
            .iter()
            .find(|a| a.item == 10 && a.nym == nym(U))
            .unwrap();
        assert_eq!(
            (u.case, u.committed, u.report),
            (Case::MissingReport, true, None)
        );
        let others: Vec<u8> = cases(s, 10).into_iter().filter(|c| *c != 6).collect();
        assert_eq!(others, vec![7; 6]);
        assert_eq!(cases(s, 11), vec![2; 7]);
        for j in [10, 11] {
            assert_eq!(s.items[j].state, ItemState::Pending(Pending::Held));
        }
        assert_eq!(s.held, vec![(10, Ok(Outcome::I)), (11, Ok(Outcome::A))]);
        let k_u = cohort(s, U);
        let m = k_u.members.iter().find(|m| m.item == 10).unwrap();
        assert_eq!(
            (m.case, m.baseline, m.contribution),
            (Case::MissingReport, None, None)
        );
        assert!(
            !s.baselines.contains(&10),
            "no baseline over the six reveals present"
        );
        let g6 = s.groups.iter().find(|g| g.group == 6).unwrap();
        assert_eq!(
            (g6.freeze, g6.start, g6.held),
            (None, Ok(Start::Held { production: 1 }), 6)
        );
    }
    assert_eq!(
        cohort(&r.at_omega, U).o,
        2,
        "the held `I` adds no observed outcome"
    );
    let logged = r.view.records(5);
    let g6 = &r.main.items[10..12];
    assert!(logged.iter().all(|(_, rec)| match rec {
        Record::Start(g) => *g != 6,
        Record::Attempt(a) => a.group != 6,
        Record::Terminal(t) => !g6.contains(&t.item),
        _ => true,
    }));
    for held in &r.main.held {
        let c = held.cid();
        assert!(r
            .main
            .replica
            .ids()
            .all(|id| r.main.replica.get(&id).unwrap().0.entry.payload != c));
    }
    let second = r.main.held.iter().find_map(|rec| match rec {
        Record::Attempt(a) if (a.stage, a.attempt) == (2, 2) => Some(a),
        _ => None,
    });
    let second = second.unwrap();
    assert_eq!(
        second.batch,
        g6.to_vec(),
        "K4 keeps both items in the batch"
    );
    assert_eq!(
        second.readings[1],
        Reading::R,
        "j12's second reading, recorded"
    );
    assert_eq!(
        r.at_omega.held[1],
        (11, Ok(Outcome::A)),
        "j12's first `A` stands"
    );
}

/// F6: unreached freezes: j13 appealable with no appeal, G7 not started, G6 held.
#[test]
fn f6_freezes_not_reached() {
    let s = &run().at_omega;
    assert_eq!(s.items[12].freeze, None);
    assert_eq!(s.items[12].lifecycle, Some(State::AppealEligible));
    assert_eq!(
        s.items[12].state,
        ItemState::Pending(Pending::FreezeUnreached)
    );
    assert_eq!(s.items[13].state, ItemState::Pending(Pending::NotStarted));
    let g = |n| s.groups.iter().find(|g| g.group == n).unwrap();
    assert_eq!(
        (g(7).freeze, g(7).start, g(7).executed),
        (None, Ok(Start::NotStarted), [0, 0])
    );
    assert_eq!(g(6).freeze, None);
    assert_eq!(cases(s, 12), vec![7; 7]);
    assert_eq!(cases(s, 13), vec![2; 7]);
    assert!(!s.baselines.contains(&12));
}

/// F7: cut 5's records update the state; the snapshot at `Ω` stays as it was.
#[test]
fn f7_a_late_record_changes_the_state_not_the_snapshot() {
    let r = run();
    assert_eq!(r.at_omega, r.omega_later);
    assert_eq!(r.at_omega.prefix, OMEGA);
    for j in [4, 5] {
        assert!(matches!(r.at_omega.items[j].state, ItemState::Pending(_)));
        assert!(matches!(r.after.items[j].state, ItemState::Terminal { .. }));
    }
    assert_eq!((r.at_omega.ii.len(), r.after.ii.len()), (70, 98));
}

/// F8: the B-b terms, counts and cohort values of `docs/20` §5, at `Ω` and after cut 5.
#[test]
fn f8_cohorts_and_contributions() {
    let r = run();
    let (s, later) = (&r.at_omega, &r.after);
    let member = |s: &Snapshot, n, j| {
        let m = cohort(s, n)
            .members
            .iter()
            .find(|m| m.item == j)
            .unwrap()
            .clone();
        (m.case, m.baseline.unwrap(), m.contribution.unwrap())
    };
    let (case, b, g) = member(s, U, 0);
    assert!(case == Case::Negative && near(b, 13, 28) && near(g, -17, 49));
    let (case, b, g) = member(s, V, 0);
    assert!(case == Case::Negative && near(b, 15, 28) && near(g, 11, 49));
    let (case, b, g) = member(s, W, 0);
    assert!(case == Case::Negative && near(b, 1, 2) && g == 0.0);
    let (case, b, g) = member(s, V, 1);
    assert!(case == Case::Positive && near(b, 5, 12) && near(g, 5, 18));
    assert!(
        !near(b, 13, 24) && !near(g, 85, 576),
        "the weighted mean, not the unweighted"
    );
    let (case, b, g) = member(later, U, 4);
    assert!(case == Case::Positive && near(b, 13, 28) && near(g, 11, 49));
    let (case, b, g) = member(later, W, 4);
    assert!(case == Case::Positive && near(b, 15, 28) && near(g, -17, 49));

    let counts = |c: &Cohort| (c.n, c.o, c.v);
    let k_v = cohort(s, V);
    assert_eq!(counts(k_v), (3, 3, 2));
    assert!(matches!(k_v.value, Value::Final(x) if near(x, 443, 2646)));
    assert_eq!(cohort(later, V), k_v);
    let k_u = cohort(s, U);
    assert_eq!(counts(k_u), (4, 2, 1));
    assert!(near(k_u.known, -17, 49));
    assert_eq!(k_u.value, Value::Unresolved(Case::MissingReport));
    let k_u = cohort(later, U);
    assert_eq!(counts(k_u), (4, 3, 2));
    assert!(near(k_u.known, -6, 49));
    assert_eq!(k_u.value, Value::Unresolved(Case::MissingReport));
    let k_w = cohort(s, W);
    assert_eq!(counts(k_w), (2, 1, 1));
    assert_eq!(k_w.known, 0.0);
    let bound = Value::Bound {
        sum: (-1.0, 1.0),
        mean: (-0.5, 0.5),
    };
    assert_eq!(k_w.value, bound);
    assert!(matches!(cohort(later, W).value, Value::Final(x) if near(x, -17, 98)));
    assert!(cohorts_complete(&r.view, 5).is_empty());
}

/// F8, K2: an assignment of `u` after `S0` (cut 6) is reported, and enlarges no cohort.
#[test]
fn f8_a_later_assignment_enlarges_no_cohort() {
    let (view, s, j15) = variant();
    assert_eq!(
        cohorts_complete(&view, 6),
        vec![CohortError::Undeclared {
            nym: nym(U),
            item: j15
        }]
    );
    assert_eq!(cohort(&s, U).n, 4);
    assert_eq!(cohort(&s, U).members, cohort(&run().after, U).members);
}

/// Cut 6 after the main scenario: j15 deposited, admitted and assigned with `u` on its panel,
/// and a synthetic start of G7 while `Φ_G7` is unreached.
fn variant() -> (LogView, Snapshot, Cid) {
    let main = &run().main;
    let mut replica = main.replica.clone();
    let mut feed = Feed::resume(M0, &replica);
    let d = draft("s2 j15");
    let j15 = d.content_id();
    let panel = [U, 4, 5, 6, 7, 8, 9].map(nym).to_vec();
    let admit = Event::Admit {
        seed_from_beacon: true,
    };
    for object in [
        deposit(&issuer(), 115, d),
        step(j15, admit),
        step(j15, Event::AssignReviewers { panel, item: j15 }),
        Record::Start(7).encode(),
    ] {
        feed.push(&mut replica, object);
    }
    let mut cuts = main.cuts.clone();
    cuts.push(Cut::next(&replica, cuts.last(), 0, false));
    let view = LogView::replay(&replica, &cuts, &main.items);
    assert!(view.error.is_none());
    let s = Snapshot::of(main, &view, 6);
    (view, s, j15)
}

/// F9: `bridging_gate` on the supplied inputs; each logged `Score` is its decision.
#[test]
fn f9_the_gate_on_supplied_inputs() {
    use GateOutcome::{AppealEligible as Ap, Pass as P, Reject as Rj, SupplementaryReview as Sr};
    let r = run();
    assert_eq!(TAU + EPS, 0.8200000000000001);
    let expected = [P, Rj, P, Sr, P, Ap, P, P, P, P, P, P, Ap, P, Sr];
    let decisions: Vec<_> = r.main.gate.iter().map(|(_, d)| *d).collect();
    assert_eq!(decisions, expected);
    for ((score, gap, coverage), decision) in &r.main.gate {
        assert_eq!(
            bridging_gate(*score, *gap, *coverage, TAU, EPS, APPEAL_GAP),
            *decision
        );
        let away = |a: f64, b: f64| (a - b).abs() >= 0.01;
        assert!(away(*score, TAU + EPS) && away(*score, TAU - EPS) && away(*gap, APPEAL_GAP));
    }
    assert!(r.main.gate[14].0 .2 < MIN_COVERAGE);
    for (j, item) in r.main.items.iter().enumerate() {
        let scores: Vec<GateOutcome> = r
            .view
            .examined
            .iter()
            .filter_map(|e| match NodeEvent::decode(&e.object) {
                Some(NodeEvent::Step {
                    item: i,
                    event: Event::Score { outcome },
                }) if i == *item => Some(outcome),
                _ => None,
            })
            .collect();
        assert_eq!(scores, vec![expected[j]], "j{}", j + 1);
    }
    let resolve = r
        .view
        .examined
        .iter()
        .find_map(|e| match NodeEvent::decode(&e.object) {
            Some(NodeEvent::Step {
                event: Event::Resolve { outcome },
                ..
            }) => Some(outcome),
            _ => None,
        });
    let Then::Extra(_, supplied) = &r.main.specs[3].then else {
        panic!("j4's extra round");
    };
    assert_eq!(
        resolve,
        Some(*supplied),
        "the fixture's supplied outcome, logged as given"
    );
    assert_eq!(*supplied, GateOutcome::Reject);
    let j4 = r.at_omega.items[3].lifecycle.clone();
    assert_eq!(j4, Some(State::Rejected(RejectReason::Borderline)));
}

/// `docs/20` §5: the 102 assignments by case, at `Ω` and after cut 5.
#[test]
fn assignments_by_case() {
    let r = run();
    let at = BTreeMap::from([(2, 49), (3, 7), (4, 25), (5, 7), (6, 1), (7, 13)]);
    let after = BTreeMap::from([(2, 35), (3, 14), (4, 25), (5, 14), (6, 1), (7, 13)]);
    assert_eq!(by_case(&r.at_omega), at);
    assert_eq!(by_case(&r.after), after);
    let extra = r.at_omega.assignments.iter().filter(|a| a.extra).count();
    assert_eq!((r.at_omega.assignments.len(), extra), (102, 4));
}

/// `docs/20` §4's common expectations: every study object refused as `NotAnEvent`, the only
/// other refusal j11's `Score`, no results event or per-nym score on the log.
#[test]
fn study_records_stay_outside_the_lifecycle() {
    let r = run();
    let mut other = Vec::new();
    for e in &r.view.examined {
        let record = Record::decode(&e.object);
        assert!(NodeEvent::decode(&e.object).is_none() || record.is_none());
        if let Some(record) = record {
            assert!(MemberObject::decode(&e.object).is_none());
            assert_eq!(Record::decode(&record.encode()), Some(record));
            assert_eq!(r.view.refusal(&e.id), Some(&Refusal::NotAnEvent));
        } else if !e.applied {
            other.push((e.pos, r.view.refusal(&e.id).unwrap()));
        }
    }
    let partial = Refusal::Rejected(Rejection::Lifecycle(Invalid::PartialEpoch));
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].1, &partial);
    let NodeEvent::Step { item, event } =
        NodeEvent::decode(&r.view.examined[other[0].0 as usize].object).unwrap()
    else {
        unreachable!()
    };
    assert!(item == r.main.items[10] && matches!(event, Event::Score { .. }));
    for held in &r.main.held {
        let bytes = held.encode();
        assert!(NodeEvent::decode(&bytes).is_none() && MemberObject::decode(&bytes).is_none());
    }
}

/// K5: each group record declares, per item, its first panel's `Score` position in cut 1.
#[test]
fn k5_reveal_close_is_the_score_position() {
    let r = run();
    for (_, record) in r.view.records(0) {
        let Record::Group(g) = record else { continue };
        for (item, close) in g.items.iter().zip(&g.reveal_close) {
            let e = &r.view.examined[*close as usize];
            let step = NodeEvent::decode(&e.object);
            let is_score = matches!(step, Some(NodeEvent::Step { item: i, event: Event::Score { .. } }) if i == *item);
            assert!(is_score && e.cut == 1, "G{}", g.group);
        }
    }
}

/// K7, T1: each started group's synthetic start follows its `Φ_G` and precedes its records.
#[test]
fn t1_starts_follow_their_freezes() {
    let s = &run().at_omega;
    for g in s.groups.iter().filter(|g| g.group <= 5) {
        let Ok(Start::Ordered { freeze, start }) = g.start else {
            panic!("G{}: {:?}", g.group, g.start);
        };
        assert_eq!((cut_of(freeze), cut_of(start)), (1, 2), "G{}", g.group);
    }
    assert_eq!(
        t1_start(Some(10), Some(5), Some(12)),
        Err(StartError::BeforeFreeze { start: 5 })
    );
    assert_eq!(
        t1_start(Some(3), Some(5), Some(4)),
        Err(StartError::RecordBeforeStart { record: 4 })
    );
    assert_eq!(
        t1_start(Some(3), None, Some(6)),
        Err(StartError::RecordWithoutStart { record: 6 })
    );
    assert_eq!(
        t2_hold(None, Some(7), 1),
        Err(StartError::LoggedWhileHeld { record: 7 })
    );
    let (view, s, _) = variant();
    let g7 = s.groups.iter().find(|g| g.group == 7).unwrap();
    let start = view
        .records(6)
        .iter()
        .find(|(_, r)| *r == Record::Start(7))
        .unwrap()
        .0
        .pos;
    assert_eq!(g7.start, Err(StartError::BeforeFreeze { start }));
}

/// `docs/20` §7: the declared counts, by source, at `Ω` and after cut 5.
#[test]
fn costs_as_declared() {
    let r = run();
    let (at, after) = (&r.at_omega.costs, &r.after.costs);
    let split = |t: s2::study::Tally| (t.log, t.held, t.total());
    assert_eq!(split(at.participations[0]), (1_320, 300, 1_620));
    assert_eq!(split(at.participations[1]), (15_000, 6_000, 21_000));
    assert_eq!(split(at.answers), (62_280, 25_200, 87_480));
    assert_eq!(split(at.fits), (4, 1, 5));
    assert_eq!(split(at.searches), (4, 2, 6));
    assert_eq!(split(at.runs), (68, 26, 94));
    assert_eq!(split(at.refused), (2, 0, 2));
    assert_eq!(split(after.participations[0]), (1_320, 300, 1_620));
    assert_eq!(split(after.participations[1]), (18_000, 6_000, 24_000));
    assert_eq!(split(after.answers), (74_280, 25_200, 99_480));
    assert_eq!((after.searches.total(), after.runs.total()), (7, 111));
    assert!(after.search_runs.iter().all(|n| [9, 17, 25].contains(n)));
    let batches = vec![
        (1, 1, 300),
        (2, 1, 300),
        (3, 1, 300),
        (5, 1, 300),
        (1, 2, 3_000),
        (2, 2, 3_000),
        (3, 2, 3_000),
        (4, 1, 120),
        (5, 2, 3_000),
        (2, 2, 3_000),
        (6, 1, 300),
        (6, 2, 3_000),
        (6, 2, 3_000),
    ];
    assert_eq!(at.batches, batches);
}

fn classify(fixture: Fixture) -> (LogView, Evidence, Evidence) {
    let o = order(fixture);
    let view = LogView::replay(&o.replica, &o.cuts, &[o.k]);
    let prefix = o.cuts.len() as u64 - 1;
    let relevant = commitments(&view, prefix, o.k);
    let refs = References::new(&o.replica, &consortium(), &view, prefix);
    let r = replay_level(&view, prefix, &relevant, &[o.x]);
    let e = existence_level(&view, prefix, &refs, &relevant, &[o.x]);
    (view, r, e)
}

/// O1: one feed; `x` refused as `NotAnEvent`, verified at (R) and (E) by one chain.
#[test]
fn o1_one_feed() {
    let (view, r, e) = classify(Fixture::O1);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Verified, Precedence::Verified)
    );
    let x = view.examined.last().unwrap();
    assert_eq!(view.refusal(&x.id), Some(&Refusal::NotAnEvent));
    assert!(view.examined.iter().all(|e| e.id.writer == key(M0)));
}

/// O2: `x` follows, on M1, member 1's signature of a cut counting the commitments.
#[test]
fn o2_two_feeds_by_a_cut_signature() {
    let (view, r, e) = classify(Fixture::O2);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Verified, Precedence::Verified)
    );
    let on_m1: Vec<_> = view
        .examined
        .iter()
        .filter(|e| e.id.writer == key(M1))
        .collect();
    let signature = MemberObject::decode(&on_m1[0].object);
    assert!(
        matches!(signature, Some(MemberObject::CutSignature { member: 1, cut, .. }) if cut == view.cuts[0])
    );
}

/// O3: a relay's `x` carries no reference: verified at (R), indeterminate at (E).
#[test]
fn o3_a_relay() {
    let (_, r, e) = classify(Fixture::O3);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Verified, Precedence::Indeterminate)
    );
}

/// O4: `CloseCommits` follows a signature of a cut counting `x`: contrary at both levels.
#[test]
fn o4_contrary_order() {
    let (_, r, e) = classify(Fixture::O4);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Contrary, Precedence::Contrary)
    );
}

/// O5: O4 through cut 0: the precondition is absent in the prefix at both levels.
#[test]
fn o5_precondition_absent() {
    let (_, r, e) = classify(Fixture::O5);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Absent, Precedence::Absent)
    );
}

/// O6: a commitment missing from the replica, or `x` counted by no cut: indeterminate.
#[test]
fn o6_incomplete_evidence() {
    let (view, r, e) = classify(Fixture::O6a);
    let missing = LedgerError::Cut(CutError::Missing { writer: key(M0) });
    assert_eq!(view.error, Some(missing));
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Indeterminate, Precedence::Indeterminate)
    );
    let (view, r, e) = classify(Fixture::O6b);
    assert!(view.examined.iter().all(|e| e.id.writer == key(M0)));
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Indeterminate, Precedence::Indeterminate)
    );
}

/// O7 and the main scenario's (ii) pairs at `Ω`: 70 verified at both levels; the 30 (iv) pairs
/// absent in the prefix, no reference binding answers being logged.
#[test]
fn o7_pairs_of_the_main_scenario() {
    let s = &run().at_omega;
    assert_eq!(s.ii.len(), 70);
    assert!(s.ii.iter().all(|p| (p.r.outcome, p.e.outcome)
        == (Precedence::Verified, Precedence::Verified)
        && p.r.uncovered.is_empty()
        && p.e.uncovered.is_empty()));
    assert_eq!(s.iv.len(), 30);
    assert!(s
        .iv
        .iter()
        .all(|p| (p.r.outcome, p.e.outcome) == (Precedence::Absent, Precedence::Absent)));
    let groups: std::collections::BTreeSet<usize> = s.iv.iter().map(|p| p.target).collect();
    assert_eq!(groups, (1..=6).collect());
    let records: std::collections::BTreeSet<usize> = s.ii.iter().map(|p| p.record).collect();
    assert_eq!(records, [0, 1, 2, 3, 8].into());
}

fn g2() -> (s2::records::Group, Cid) {
    let Some(Record::Group(group)) = Record::decode(&group_bytes(2)) else {
        unreachable!()
    };
    (group, cid(&group_bytes(2)))
}

/// K4, K6: one attempt recorded twice, or two records claiming it, exhausts nothing.
#[test]
fn k4_a_repeated_attempt_is_not_a_second_one() {
    let (group, rule) = g2();
    let all = attempts(2, u64::MAX);
    let (s1, s2) = (&all[0], &all[1]);
    let (_, j3) = the_terminal(2);
    let t = Terminal {
        attempts: vec![s1.1, s2.1],
        ..j3.clone()
    };
    let twice = vec![
        (&s1.0, s1.1, s1.2),
        (&s2.0, s2.1, s2.2),
        (&s2.0, s2.1, s2.2),
    ];
    let repeated = SequenceError::Repeated {
        stage: 2,
        attempt: 1,
    };
    let refused = Err(TerminalError::Sequence(repeated));
    assert_eq!(
        validate(&t, &group, rule, &twice),
        refused,
        "a repeated record exhausted j3"
    );
    let other = Attempt {
        evidence: cid(b"another reading of G2 s2 a1").0,
        ..s2.0.clone()
    };
    let other_cid = Record::Attempt(other.clone()).cid();
    let t = Terminal {
        attempts: vec![s1.1, s2.1, other_cid],
        ..j3
    };
    let claimed = vec![
        (&s1.0, s1.1, s1.2),
        (&s2.0, s2.1, s2.2),
        (&other, other_cid, 3),
    ];
    let conflicting = SequenceError::Conflicting {
        stage: 2,
        attempt: 1,
    };
    let refused = Err(TerminalError::Sequence(conflicting));
    assert_eq!(
        validate(&t, &group, rule, &claimed),
        refused,
        "two records, one attempt"
    );
}

/// K4: a conclusion recorded before two non-conclusions is not hidden by a higher number.
#[test]
fn k4_a_misnumbered_sequence_is_refused() {
    let (group, rule) = g2();
    let all = attempts(2, u64::MAX);
    let early = Attempt {
        attempt: 3,
        readings: vec![Reading::A, Reading::R],
        evidence: cid(b"G2 s2 a3").0,
        ..all[1].0.clone()
    };
    let early_cid = Record::Attempt(early.clone()).cid();
    let (_, j3) = the_terminal(2);
    let recorded = vec![
        (&all[0].0, all[0].1, all[0].2),
        (&early, early_cid, 3),
        (&all[1].0, all[1].1, all[1].2),
        (&all[2].0, all[2].1, all[2].2),
    ];
    let misnumbered = SequenceError::Misnumbered {
        stage: 2,
        attempt: 3,
    };
    let refused = Err(TerminalError::Sequence(misnumbered));
    assert_eq!(
        validate(&j3, &group, rule, &recorded),
        refused,
        "j3's `A` hidden"
    );
}

/// K4, K6: G3's first stage-2 attempt recorded again, on M1 in a cut 4', counts once.
#[test]
fn k6_a_repeated_record_counts_once() {
    let main = &run().main;
    let mut replica = main.replica.clone();
    let (_, record) = logged_record(
        |r| matches!(r, Record::Attempt(a) if (a.group, a.stage, a.attempt) == (3, 2, 1)),
    );
    let again = Feed::new(M1).push(&mut replica, record.encode());
    let mut cut4 = main.cuts[3].clone();
    cut4.number = 4;
    cut4.marks.push(Mark {
        writer: again.writer,
        len: 1,
        head: again.hash,
    });
    cut4.marks.sort_by_key(|m| m.writer);
    let mut cuts = main.cuts[..=3].to_vec();
    cuts.push(cut4);
    let view = LogView::replay(&replica, &cuts, &main.items);
    assert!(view.error.is_none());
    let s = Snapshot::of(main, &view, 4);
    let g3 = s.groups.iter().find(|g| g.group == 3).unwrap();
    assert_eq!(g3.executed, [1, 1]);
    assert_eq!(s.costs.participations[1].log, 12_000);
    let repeated = SequenceError::Repeated {
        stage: 2,
        attempt: 1,
    };
    assert_eq!(g3.sequence, Err(repeated));
    for j in [4, 5] {
        assert_eq!(s.items[j].state, ItemState::Pending(Pending::Incoherent));
    }
    assert!(s
        .groups
        .iter()
        .filter(|g| g.group != 3)
        .all(|g| g.sequence.is_ok()));
}

/// `17` §13.2: O4's contrary order stays reported beside an entry of `x_O` no cut counts (R)
/// or the verifier lacks (E).
#[test]
fn o4_a_contrary_order_survives_incomplete_coverage() {
    let o = order(Fixture::O4);
    let mut relay = Feed::new(RELAY);
    let (uncounted, object) = relay.sign(disclosure());
    let (lacking, _) = relay.sign(b"a disclosure the verifier lacks".to_vec());
    let mut replica = o.replica.clone();
    replica.insert(uncounted.clone(), object).unwrap();
    let view = LogView::replay(&replica, &o.cuts, &[o.k]);
    let relevant = commitments(&view, 1, o.k);
    let refs = References::new(&replica, &consortium(), &view, 1);
    let r = replay_level(&view, 1, &relevant, &[o.x, uncounted.id()]);
    let e = existence_level(&view, 1, &refs, &relevant, &[o.x, lacking.id()]);
    assert_eq!(
        (r.outcome, e.outcome),
        (Precedence::Contrary, Precedence::Contrary)
    );
    for (level, gap) in [(&r, uncounted.id()), (&e, lacking.id())] {
        let Some(Witness::Before { x, c }) = level.witness else {
            panic!("no witness: {level:?}");
        };
        assert!(x == o.x && relevant.entries.contains(&c));
        assert_eq!(level.uncovered, vec![gap]);
    }
}

/// `17` §13.2: O5's absent precondition stays reported beside an entry without evidence; O1's
/// order is not verified while an entry lacks it.
#[test]
fn o5_and_o1_with_incomplete_coverage() {
    for (fixture, expected) in [
        (Fixture::O5, Precedence::Absent),
        (Fixture::O1, Precedence::Indeterminate),
    ] {
        let o = order(fixture);
        let mut relay = Feed::new(RELAY);
        let (uncounted, object) = relay.sign(disclosure());
        let (lacking, _) = relay.sign(b"a disclosure the verifier lacks".to_vec());
        let mut replica = o.replica.clone();
        replica.insert(uncounted.clone(), object).unwrap();
        let view = LogView::replay(&replica, &o.cuts, &[o.k]);
        let prefix = o.cuts.len() as u64 - 1;
        let relevant = commitments(&view, prefix, o.k);
        let refs = References::new(&replica, &consortium(), &view, prefix);
        let r = replay_level(&view, prefix, &relevant, &[o.x, uncounted.id()]);
        let e = existence_level(&view, prefix, &refs, &relevant, &[o.x, lacking.id()]);
        assert_eq!((r.outcome, e.outcome), (expected, expected), "{fixture:?}");
        let witness = (expected == Precedence::Absent).then_some(Witness::Unclosed { x: o.x });
        assert_eq!((r.witness, e.witness), (witness, witness));
        assert_eq!(
            (r.uncovered, e.uncovered),
            (vec![uncounted.id()], vec![lacking.id()])
        );
    }
}

/// The member `(n, j)` of `n`'s cohort: its case, baseline and term.
fn term(s: &Snapshot, n: u8, j: usize) -> (Case, Option<f64>, Option<f64>) {
    let m = cohort(s, n).members.iter().find(|m| m.item == j).unwrap();
    (m.case, m.baseline, m.contribution)
}

/// `17` §14.4: verdicts without a baseline leave `K_w` with no final value, never `Final(0)`.
#[test]
fn a_cohort_of_verdicts_without_baselines_has_no_final_value() {
    let s = &only_w().after;
    assert_eq!(term(s, W, 0), (Case::Negative, None, None));
    assert_eq!(term(s, W, 4), (Case::Positive, None, None));
    let k_w = cohort(s, W);
    assert_eq!((k_w.n, k_w.o, k_w.v), (2, 2, 2));
    assert_eq!(k_w.known, 0.0);
    assert!(
        !matches!(k_w.value, Value::Final(_) | Value::Bound { .. }),
        "{:?}",
        k_w.value
    );
}

/// `17` §14.4: (v, j2)'s undefined term does not vanish from `K_v`'s sum beside defined ones.
#[test]
fn an_undefined_term_does_not_vanish_from_a_mixed_cohort() {
    let r = only_w();
    for s in [&r.at_omega, &r.after] {
        let k_u = cohort(s, U);
        assert_eq!(k_u.value, Value::Unresolved(Case::MissingReport));
        let (case, b, g) = term(s, U, 0);
        assert!(case == Case::Negative && b == Some(0.5) && near(g.unwrap(), -5, 16));
        let (case, b, g) = term(s, V, 0);
        assert!(case == Case::Negative && b == Some(0.5) && near(g.unwrap(), 3, 16));
        assert_eq!(term(s, V, 1), (Case::Positive, None, None));
        assert_eq!(term(s, V, 2), (Case::Inconclusive, None, Some(0.0)));
    }
    assert!(near(cohort(&r.at_omega, U).known, -5, 16));
    assert!(near(cohort(&r.after, U).known, 3, 16));
    let k_v = cohort(&r.at_omega, V);
    assert_eq!((k_v.n, k_v.o, k_v.v), (3, 3, 2));
    assert!(near(k_v.known, 3, 16));
    assert!(
        !matches!(k_v.value, Value::Final(_) | Value::Bound { .. }),
        "{:?}",
        k_v.value
    );
}

/// `17` §14.4: (w, j1)'s undefined term is not absorbed into the bound of the pending (w, j5).
#[test]
fn an_undefined_term_is_not_absorbed_into_a_pending_bound() {
    let s = &only_w().at_omega;
    assert_eq!(term(s, W, 0), (Case::Negative, None, None));
    assert_eq!(term(s, W, 4), (Case::Pending, None, None));
    let k_w = cohort(s, W);
    assert_eq!((k_w.n, k_w.o, k_w.v), (2, 1, 1));
    assert_eq!(k_w.known, 0.0);
    assert!(
        !matches!(k_w.value, Value::Final(_) | Value::Bound { .. }),
        "{:?}",
        k_w.value
    );
}

/// `17` §14.4: the undefined term is named apart from cases 2, 6 and 7, and listed beside them.
#[test]
fn an_undefined_term_is_named_apart_from_pending_missing_and_unfrozen() {
    let r = only_w();
    let no = Value::Undefined(Undefined::NoBaseline);
    for (s, n, items) in [
        (&r.at_omega, V, vec![1]),
        (&r.after, V, vec![1]),
        (&r.at_omega, W, vec![0]),
        (&r.after, W, vec![0, 4]),
    ] {
        let k = cohort(s, n);
        assert_eq!((&k.value, &k.undefined), (&no, &items));
        let named = k.members.iter().filter(|m| m.undefined.is_some());
        assert!(named.map(|m| m.item).eq(items));
    }
    assert!(cohort(&r.after, U).undefined.is_empty());
    for s in [&run().at_omega, &run().after] {
        assert!(s.cohorts.iter().all(|c| c.undefined.is_empty()));
    }
    let lacking = member(0, Case::Negative, Some(0.75), None);
    assert_eq!(lacking.undefined, Some(Undefined::NoBaseline));
    let i = member(2, Case::Inconclusive, Some(0.25), None);
    assert_eq!((i.contribution, i.undefined), (Some(0.0), None));
    let missing = member(10, Case::MissingReport, None, None);
    let both = s2::study::cohort(nym(U), vec![lacking.clone(), missing]);
    assert_eq!(
        (both.value, both.undefined),
        (Value::Unresolved(Case::MissingReport), vec![0])
    );
    let pending = member(4, Case::Pending, Some(0.75), Some(0.25));
    let mixed = s2::study::cohort(nym(U), vec![lacking, pending]);
    assert_eq!((mixed.value, mixed.undefined), (no, vec![0]));
}

const TOL: f64 = 1e-12;

fn fr(n: i128, d: i128) -> Q {
    Q::new(n, d)
}

/// `docs/20` §11.1's checks on every strategy of `k`: the truthful value, the strict and
/// indifferent counts, the least and the greatest strict loss.
fn verified(k: &Construction) -> (Q, [usize; 2], Q, Q) {
    assert!(k.baselines_invariant(), "C5, pathwise");
    let laws = k.laws();
    let truthful = adaptive::truthful(&laws);
    assert!(laws.iter().filter_map(|l| l.q).all(|q| k.grid.contains(&q)));
    let best = k.predicted(&laws, &truthful);
    let top = k.expectation(&truthful);
    assert!((top - best.f()).abs() <= TOL);
    let (mut counts, mut losses) = ([0, 0], Vec::new());
    for s in k.strategies() {
        let (exact, path) = (k.predicted(&laws, &s), k.expectation(&s));
        assert!(
            (path - exact.f()).abs() <= TOL,
            "{s:?}: {path} against {exact:?}"
        );
        assert!(path <= top + TOL, "{s:?} beats the truthful strategy");
        let mut changed = laws
            .iter()
            .zip(s.iter().zip(&truthful))
            .filter(|(_, (a, b))| a != b);
        let changed: Vec<&Law> = changed.by_ref().map(|(l, _)| l).collect();
        if changed.is_empty() {
            continue;
        }
        if changed.iter().any(|l| l.cell > ZERO && l.c > ZERO) {
            assert!(top - path > TOL && exact < best, "{s:?}");
            counts[0] += 1;
            losses.push(best - exact);
        } else {
            assert!((path - top).abs() <= TOL && exact == best, "{s:?}");
            counts[1] += 1;
        }
    }
    let (least, most) = (losses.iter().min(), losses.iter().max());
    (best, counts, *least.unwrap(), *most.unwrap())
}

/// `17` §14.6 (A), P1: no joint rule on `(s, d)` beats `q_c`; strict where `c > 0`, equal at 0.
#[test]
fn p1_no_adaptive_joint_rule_beats_the_truthful_reports() {
    let k = adaptive::p1();
    let law = |c, q: Option<Q>, t| Law {
        cell: fr(1, 4),
        c,
        q,
        t,
    };
    let (low, high) = (Some(fr(1, 4)), Some(fr(3, 4)));
    let laws = vec![
        law(ONE, low, fr(13, 64)),
        law(ONE, low, fr(1, 4)),
        law(fr(2, 3), low, fr(13, 96)),
        law(ZERO, None, ZERO),
        law(ONE, high, fr(13, 64)),
        law(ONE, high, fr(1, 4)),
        law(fr(2, 3), high, fr(13, 96)),
        law(ZERO, None, ZERO),
    ];
    assert_eq!(k.laws(), laws);
    assert_eq!((k.states.len(), k.strategies().count()), (24, 6_561));
    assert_eq!(
        verified(&k),
        (fr(17, 768), [6_552, 8], fr(1, 192), fr(1, 6))
    );
    let half = vec![fr(1, 2); 8];
    let mut unconditional = adaptive::truthful(&laws);
    unconditional[6] = fr(1, 2);
    for (s, value) in [(half, fr(-5, 256)), (unconditional, fr(13, 768))] {
        assert_eq!(k.predicted(&laws, &s), value);
        assert!((k.expectation(&s) - value.f()).abs() <= TOL);
    }
}

/// `17` §14.6 (A), P2: three items of one group, `D` another item's gate decision; strict loss.
#[test]
fn p2_no_rule_on_a_gate_decision_beats_the_truthful_reports() {
    let k = adaptive::p2();
    let law = |cell, q| Law {
        cell,
        c: fr(1, 2),
        q: Some(q),
        t: fr(61, 512),
    };
    let laws: Vec<Law> = [fr(3, 8), fr(5, 8)]
        .into_iter()
        .flat_map(|q| [law(fr(1, 2), q); 3])
        .collect();
    assert_eq!(k.laws(), laws);
    assert_eq!((k.states.len(), k.strategies().count()), (36, 729));
    assert_eq!(verified(&k), (fr(1, 512), [728, 0], fr(1, 768), fr(1, 32)));
    let half = vec![fr(1, 2); 6];
    assert_eq!(k.predicted(&laws, &half), fr(-3, 512));
    assert!((k.expectation(&half) - fr(-3, 512).f()).abs() <= TOL);
}

/// `17` §14.6 (B), §7.3 for B-b, outside design A: `j`'s term −6/25, and 0 once `k` enters.
#[test]
fn b_an_entry_channel_moves_one_term() {
    assert!(near(adaptive::expected(&adaptive::load(false)), -6, 25));
    assert_eq!(adaptive::expected(&adaptive::load(true)), 0.0);
}

/// `17` §14.6 (B), §9.4: a supplied `b(p)` gives 3/16 at ¾, 0 at ½; `panel_scores`' stays put.
#[test]
fn b_a_report_dependent_baseline_moves_one_term() {
    assert_eq!(adaptive::replacement(0.5), 0.0);
    assert!(near(adaptive::replacement(0.75), 3, 16));
    let other = adaptive::Seat {
        outcome: Outcome::A,
        others: vec![(fr(1, 2), 1)],
    };
    let b = |p| adaptive::baseline(p, &other);
    assert_eq!((b(0.5), b(0.75)), (Some(0.5), Some(0.5)));
}

/// `17` §14.6 (B), §11.3's timing: the band moves the outcome's law, 1/25 at 2/5 against 0.
#[test]
fn b_a_timing_channel_moves_one_term() {
    assert_eq!(adaptive::expected(&adaptive::timing(fr(3, 5).f())), 0.0);
    assert!(near(adaptive::expected(&adaptive::timing(0.4)), 1, 25));
}

/// `17` §14.6 (B), §11.3's disclosure: a signal reaching the baseline's reports, −¼ against 0.
#[test]
fn b_a_disclosure_moves_one_term() {
    assert_eq!(adaptive::expected(&adaptive::disclosure(false)), 0.0);
    assert!(near(adaptive::expected(&adaptive::disclosure(true)), -1, 4));
}

/// `17` §14.6 (B), §12.4: a partly informative signal across groups, `−δ²` against 0.
#[test]
fn b_a_signal_across_groups_moves_one_term() {
    assert_eq!(adaptive::expected(&adaptive::across(None)), 0.0);
    for (delta, loss) in [(fr(1, 4), 16), (fr(1, 2), 4)] {
        let e = adaptive::expected(&adaptive::across(Some(delta)));
        assert!(near(e, -1, loss), "{delta:?}: {e}");
    }
}
