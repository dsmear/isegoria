//! C5–C6 composed offline (`docs/15` C5–C6): the author a deposit proves and the conclusion a
//! step reaches become an epoch's author records, which the node checks, applies and replays.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::Cid;
use protocol::appeal::{EpochScale, InsufficientReputation};
use protocol::authorship::{
    author_records, AppealRefusal, AppealRequest, AuthorRecords, AuthorRecordsError,
};
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::gate::GateOutcome;
use protocol::lifecycle::{Event, Invalid, RejectReason, State};
use protocol::node::{Node, NodeError, Outcome, Rejection};
use protocol::pilot::Screening;
use protocol::results::{EpochResults, ResultRecord, ResultsRejected};
use protocol::revalidation::Recheck;
use protocol::review::commit;
use scoring::reputation::{author_score, AuthorPrior};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const DEPOSITS: u64 = 1;

/// A fixture `E_m` of four epochs a month, not a decided value (`docs/15` C6).
fn scale() -> EpochScale {
    EpochScale::new(4.0).expect("a positive E_m")
}

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("author_records")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

fn credential(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&req).unwrap())
}

fn deposit(issuer: &Issuer, author: &AnonymousCredential, tag: &str) -> (NodeEvent, Cid) {
    let draft = Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"Gazzetta Ufficiale".to_vec(),
    };
    let item = draft.content_id();
    let context = deposit_context(item, DEPOSITS);
    let proof = prove(author, &issuer.public(), Role::Propose, &context);
    let event = NodeEvent::Deposit {
        epoch: DEPOSITS,
        quota: 32,
        draft,
        proof,
    };
    (event, item)
}

fn nyms(from: u8, n: u8) -> Vec<Nym> {
    (from..from + n).map(|i| Nym([i; 32])).collect()
}

fn round(item: Cid, panel: &[Nym]) -> Vec<Event> {
    let mut events: Vec<Event> = panel
        .iter()
        .map(|n| Event::Commit {
            nym: *n,
            commitment: commit(0.75, &n.0, *n, item),
        })
        .collect();
    events.push(Event::CloseCommits);
    events.extend(panel.iter().map(|n| Event::Reveal {
        nym: *n,
        prob: 0.75,
        nonce: n.0,
    }));
    events
}

/// From `Deposited` through a scored first round.
fn reviewed(item: Cid, outcome: GateOutcome) -> Vec<Event> {
    let panel = nyms(10, 7);
    let mut events = vec![
        Event::Admit {
            seed_from_beacon: true,
        },
        Event::AssignReviewers {
            panel: panel.clone(),
            item,
        },
    ];
    events.extend(round(item, &panel));
    events.push(Event::Score { outcome });
    events
}

fn screen(screen: Screening) -> Event {
    Event::Pilot1Batch {
        enough_respondents: true,
        screen,
    }
}

fn stage2(dif: Recheck, source_verified: bool) -> Event {
    Event::Pilot2Batch {
        batch_size: 2,
        dif,
        source_verified,
    }
}

/// The steps from `Deposited` to the end of the walk `name` names before any `/`.
fn walk(item: Cid, name: &str) -> Vec<Event> {
    let pass = [screen(Screening::Pass), stage2(Recheck::NoDif, false)];
    let kind = name.split('/').next().unwrap();
    let (gate, tail): (GateOutcome, Vec<Event>) = match kind {
        "pool" => (GateOutcome::Pass, pass.to_vec()),
        "contested" => (
            GateOutcome::Pass,
            vec![screen(Screening::Pass), stage2(Recheck::Dif, true)],
        ),
        "screen" => (GateOutcome::Pass, vec![screen(Screening::Fail)]),
        "dif" => (
            GateOutcome::Pass,
            vec![screen(Screening::Pass), stage2(Recheck::Dif, false)],
        ),
        "revalidated" => (GateOutcome::Pass, {
            let mut t = pass.to_vec();
            t.push(Event::Revalidate {
                dif: Recheck::Dif,
                source_verified: true,
            });
            t.push(Event::Revalidate {
                dif: Recheck::Dif,
                source_verified: false,
            });
            t
        }),
        "defect" => (GateOutcome::Reject, vec![]),
        "polarized" => (GateOutcome::AppealEligible, vec![Event::AppealExpires]),
        "borderline" => return borderline(item),
        "explored" => (GateOutcome::Reject, {
            let mut t = vec![Event::Explore {
                seed_from_beacon: true,
            }];
            t.extend(pass.clone());
            t
        }),
        "pending" => (
            GateOutcome::Pass,
            vec![
                screen(Screening::Pass),
                stage2(Recheck::Indeterminate, false),
            ],
        ),
        "eligible" => (GateOutcome::AppealEligible, vec![]),
        _ => unreachable!("no walk {name}"),
    };
    let mut events = reviewed(item, gate);
    events.extend(tail);
    events
}

fn borderline(item: Cid) -> Vec<Event> {
    let mut events = reviewed(item, GateOutcome::SupplementaryReview);
    let extra = nyms(40, 3);
    events.push(Event::AssignExtraReviewers {
        panel: extra.clone(),
    });
    events.extend(round(item, &extra));
    events.push(Event::Resolve {
        outcome: GateOutcome::Reject,
    });
    events
}

fn open(dir: &Path, issuer: &Issuer, scale: Option<EpochScale>) -> Result<Node, NodeError> {
    match scale {
        Some(scale) => Node::open_with_scale(dir, issuer.public(), scale),
        None => Node::open(dir, issuer.public()),
    }
}

/// A node holding one deposit per `(author, walk)`, each item walked to its end.
struct Setup {
    node: Node,
    dir: PathBuf,
    issuer: Issuer,
    scale: Option<EpochScale>,
    items: Vec<(String, Cid, Nym)>,
}

impl Setup {
    fn new(name: &str, walks: &[(u8, &str)], scale: Option<EpochScale>) -> Self {
        let (issuer, dir) = (issuer(), scratch(name));
        let mut node = open(&dir, &issuer, scale).unwrap();
        let creds: Vec<(u8, AnonymousCredential)> = walks
            .iter()
            .map(|(s, _)| *s)
            .collect::<BTreeSet<u8>>()
            .into_iter()
            .map(|s| (s, credential(&issuer, s)))
            .collect();
        let mut items = Vec::new();
        for (secret, name) in walks {
            let cred = &creds.iter().find(|(s, _)| s == secret).unwrap().1;
            let (event, item) = deposit(&issuer, cred, &format!("{name} by {secret}"));
            let Ok(Outcome::Admitted(author)) = node.submit(event) else {
                panic!("the deposit is accepted");
            };
            for event in walk(item, name) {
                node.submit(NodeEvent::Step { item, event }).unwrap();
            }
            items.push((format!("{name} by {secret}"), item, author));
        }
        Setup {
            node,
            dir,
            issuer,
            scale,
            items,
        }
    }

    fn item(&self, tag: &str) -> (Cid, Nym) {
        let (_, item, author) = self.items.iter().find(|(t, ..)| t == tag).unwrap();
        (*item, *author)
    }

    fn reopen(&self) -> Node {
        open(&self.dir, &self.issuer, self.scale).unwrap()
    }

    fn step(&mut self, item: Cid, event: Event) -> Result<Outcome, NodeError> {
        self.node.submit(NodeEvent::Step { item, event })
    }

    fn records(&self, epoch: u64, appeals: &[&str]) -> Result<AuthorRecords, AuthorRecordsError> {
        let within: Vec<(&str, bool)> = appeals.iter().map(|t| (*t, true)).collect();
        self.requests(epoch, &within)
    }

    /// The records for the requests `(tag, within_window)`.
    fn requests(
        &self,
        epoch: u64,
        appeals: &[(&str, bool)],
    ) -> Result<AuthorRecords, AuthorRecordsError> {
        let appeals = appeals
            .iter()
            .map(|(t, within_window)| {
                let within_window = *within_window;
                (self.item(t).0, AppealRequest { within_window })
            })
            .collect();
        author_records(self.node.state(), epoch, &appeals)
    }

    fn submit(&mut self, epoch: u64, records: Vec<ResultRecord>) -> Result<Outcome, NodeError> {
        self.node.submit(results(epoch, records))
    }

    fn escrows(&self, author: &Nym) -> Vec<Cid> {
        self.node.state().results().open_escrows(author)
    }

    fn state(&self, item: &Cid) -> Option<&State> {
        self.node.state().item(item)
    }
}

fn results(epoch: u64, records: Vec<ResultRecord>) -> NodeEvent {
    NodeEvent::Results(EpochResults {
        epoch,
        inputs_root: [epoch as u8; 32],
        records,
    })
}

fn on(item: Cid, event: Event) -> NodeEvent {
    NodeEvent::Step { item, event }
}

fn start(author: Nym, item: Cid) -> ResultRecord {
    ResultRecord::AppealStarted {
        author,
        item,
        within_window: true,
    }
}

const STARTED: State = State::Pilot1 { appealed: true };

/// Submits `event` and checks the node refused it, its state and log as before, also reopened.
fn refused_without_trace(setup: &mut Setup, event: NodeEvent) -> Rejection {
    let before = setup.reopen();
    let got = setup.node.submit(event);
    let Err(NodeError::Rejected(why)) = got else {
        panic!("accepted: {got:?}");
    };
    assert_eq!(setup.node.state(), before.state());
    assert_eq!(setup.node.log_len(), before.log_len());
    assert_eq!(setup.reopen().state(), before.state());
    why
}

const EVERY_WALK: [&str; 11] = [
    "pool",
    "contested",
    "screen",
    "dif",
    "revalidated",
    "defect",
    "polarized",
    "borderline",
    "explored",
    "pending",
    "eligible",
];

/// C5: only a pilot's conclusion records an outcome — pools 1, screen or DIF 0 — once; replayed.
#[test]
fn pilot_conclusions_reach_the_record_from_their_deposit_and_step() {
    let walks: Vec<(u8, &str)> = EVERY_WALK
        .iter()
        .enumerate()
        .map(|(i, w)| (2 + i as u8 % 2, *w))
        .collect();
    let mut setup = Setup::new("conclusions", &walks, Some(scale()));
    let concluded = [
        ("pool", true),
        ("contested", true),
        ("screen", false),
        ("dif", false),
        ("revalidated", true),
    ];
    let mut want: Vec<ResultRecord> = concluded
        .iter()
        .map(|(name, passed)| {
            let tag = walks
                .iter()
                .find(|(_, w)| w == name)
                .map(|(s, w)| format!("{w} by {s}"))
                .unwrap();
            let (item, author) = setup.item(&tag);
            ResultRecord::AuthorOutcome {
                author,
                item,
                passed: *passed,
            }
        })
        .collect();
    want.sort_by_key(|r| match r {
        ResultRecord::AuthorOutcome { item, .. } => *item,
        _ => unreachable!(),
    });
    let out = setup.records(2, &[]).unwrap();
    assert_eq!(
        out,
        AuthorRecords {
            records: want.clone(),
            ..AuthorRecords::default()
        }
    );
    assert_eq!(setup.submit(2, out.records), Ok(Outcome::Moved));
    let results = setup.node.state().results();
    for (tag, item, author) in &setup.items {
        let history = results.author(author).unwrap();
        let recorded = want.iter().find_map(|r| match r {
            ResultRecord::AuthorOutcome {
                item: i, passed, ..
            } if i == item => Some(*passed),
            _ => None,
        });
        assert_eq!(history.outcome(item), recorded.map(|p| (p, 2)), "{tag}");
    }
    let reopened = setup.reopen();
    assert_eq!(reopened.state(), setup.node.state());
    assert_eq!(setup.records(3, &[]).unwrap(), AuthorRecords::default());
}

/// C5: the node refuses an author record its log does not support, whole, filings' moves included.
#[test]
fn the_node_refuses_author_records_its_log_does_not_support() {
    let walks = [
        (2, "pool"),
        (2, "pending"),
        (2, "explored"),
        (2, "defect"),
        (2, "eligible/appealed"),
        (2, "eligible/second"),
        (3, "eligible"),
    ];
    let mut setup = Setup::new("refused", &walks, Some(scale()));
    let (pool, author) = setup.item("pool by 2");
    let (other, third) = setup.item("eligible by 3");
    let (appealed, _) = setup.item("eligible/appealed by 2");
    let (second, _) = setup.item("eligible/second by 2");
    setup.submit(2, vec![start(author, appealed)]).unwrap();
    for event in [screen(Screening::Pass), stage2(Recheck::NoDif, false)] {
        setup.step(appealed, event).unwrap();
    }
    let outcome = |item, author, passed| ResultRecord::AuthorOutcome {
        author,
        item,
        passed,
    };
    let item = |tag: &str| setup.item(tag).0;
    let valid = outcome(pool, author, true);
    let late = ResultRecord::AppealStarted {
        author: third,
        item: other,
        within_window: false,
    };
    let unbounded = ResultRecord::ReviewerScore {
        reviewer: Nym([5; 32]),
        score: 0.5,
        inclusion: 0.0,
    };
    let cases = vec![
        (
            vec![outcome(pool, Nym([9; 32]), true)],
            ResultsRejected::NotTheProposer { record: 0 },
        ),
        (
            vec![outcome(Cid([9; 32]), author, true)],
            ResultsRejected::NotTheProposer { record: 0 },
        ),
        (
            vec![outcome(other, author, true)],
            ResultsRejected::NotTheProposer { record: 0 },
        ),
        (
            vec![outcome(pool, author, false)],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![outcome(item("pending by 2"), author, true)],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![outcome(item("explored by 2"), author, true)],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![outcome(item("defect by 2"), author, false)],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![outcome(appealed, author, true)],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![ResultRecord::AppealSettled {
                author,
                item: appealed,
                passed: false,
            }],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![ResultRecord::AppealSettled {
                author,
                item: pool,
                passed: true,
            }],
            ResultsRejected::OutcomeMismatch { record: 0 },
        ),
        (
            vec![start(Nym([9; 32]), other)],
            ResultsRejected::NotTheProposer { record: 0 },
        ),
        (
            vec![start(author, Cid([9; 32]))],
            ResultsRejected::NotTheProposer { record: 0 },
        ),
        (
            vec![valid.clone(), start(author, pool)],
            ResultsRejected::NotAppealable { record: 1 },
        ),
        (
            vec![start(author, appealed)],
            ResultsRejected::NotAppealable { record: 0 },
        ),
        (
            vec![late],
            ResultsRejected::AppealWindowClosed { record: 0 },
        ),
        (
            vec![start(author, second)],
            ResultsRejected::AppealNotCovered { record: 0 },
        ),
        (
            vec![start(third, other), start(third, other)],
            ResultsRejected::DuplicateItem { record: 1 },
        ),
        (
            vec![start(third, other), outcome(pool, author, false)],
            ResultsRejected::OutcomeMismatch { record: 1 },
        ),
        (
            vec![start(third, other), unbounded],
            ResultsRejected::BadInclusion { record: 1 },
        ),
    ];
    for (records, why) in cases {
        let refused = refused_without_trace(&mut setup, results(3, records));
        assert_eq!(refused, Rejection::Results(why));
    }
    assert_eq!(setup.state(&other), Some(&State::AppealEligible));
    let accepted = setup.submit(3, vec![valid, start(third, other)]);
    assert_eq!(accepted, Ok(Outcome::Moved));
    assert_eq!(setup.state(&other), Some(&STARTED));
}

/// C5/C6: an appeal filed through the node settles its escrow; the item counts once, from `e + 1`.
#[test]
fn an_appeal_through_the_node_settles_its_escrow_and_counts_once() {
    let mut setup = Setup::new("appeal", &[(2, "pool"), (2, "eligible")], Some(scale()));
    let ((earlier, author), (appealed, _)) = (setup.item("pool by 2"), setup.item("eligible by 2"));
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    assert_eq!(out.admitted, vec![appealed]);
    assert_eq!(out.records.last(), Some(&start(author, appealed)));
    setup.submit(2, out.records).unwrap();
    assert_eq!(setup.state(&appealed), Some(&STARTED));
    for event in [screen(Screening::Pass), stage2(Recheck::NoDif, false)] {
        setup.step(appealed, event).unwrap();
    }
    let out = setup.records(3, &[]).unwrap();
    let settled = ResultRecord::AppealSettled {
        author,
        item: appealed,
        passed: true,
    };
    assert_eq!(out.records, vec![settled]);
    setup.submit(3, out.records).unwrap();
    let history = setup
        .node
        .state()
        .results()
        .author(&author)
        .unwrap()
        .clone();
    assert_eq!(
        (history.outcomes(), history.outcome(&appealed)),
        (2, Some((true, 3)))
    );
    assert!(history.open_escrows().is_empty());
    let p = AuthorPrior::default();
    let with_escrow = author_score(&[1.0, 0.0], &[0.25, 0.0], &p);
    assert_eq!(
        history.reputation(3, scale(), &p).to_bits(),
        with_escrow.to_bits()
    );
    let mut both = [(earlier, 0.5), (appealed, 0.25)];
    both.sort_by_key(|(item, _)| *item);
    let ages: Vec<f64> = both.iter().map(|(_, a)| *a).collect();
    let passed = author_score(&[1.0, 1.0], &ages, &p);
    assert_eq!(
        history.reputation(4, scale(), &p).to_bits(),
        passed.to_bits()
    );
    assert_eq!(setup.records(4, &[]).unwrap(), AuthorRecords::default());
    assert_eq!(setup.reopen().state(), setup.node.state());
}

/// C6: one epoch's appeals are selected in CID order whatever their arrival; replay is atomic.
#[test]
fn appeals_in_one_epoch_are_selected_in_cid_order_whatever_their_arrival() {
    let walks = [(2, "eligible/x"), (2, "eligible/y"), (3, "eligible/z")];
    let mut setup = Setup::new("order", &walks, Some(scale()));
    let mut reversed = Setup::new(
        "order-reversed",
        &[walks[2], walks[1], walks[0]],
        Some(scale()),
    );
    let (x, author) = setup.item("eligible/x by 2");
    let (y, z) = (
        setup.item("eligible/y by 2").0,
        setup.item("eligible/z by 3").0,
    );
    let (low, high) = (x.min(y), x.max(y));
    let mut admitted = vec![low, z];
    admitted.sort();
    let short = InsufficientReputation {
        reputation: author_score(&[0.0], &[0.0], &AuthorPrior::default()),
        floor: 0.4,
    };
    let within = AppealRequest {
        within_window: true,
    };
    for arrival in [[x, y, z], [z, y, x], [y, z, x]] {
        let appeals: BTreeMap<Cid, AppealRequest> =
            arrival.into_iter().map(|c| (c, within)).collect();
        for node in [&setup.node, &reversed.node] {
            let out = author_records(node.state(), 2, &appeals).unwrap();
            let refused = vec![(high, AppealRefusal::Uncovered(short))];
            assert_eq!((&out.admitted, &out.refused), (&admitted, &refused));
        }
    }
    let both = vec![start(author, high), start(author, low)];
    let uncovered = Rejection::Results(ResultsRejected::AppealNotCovered { record: 1 });
    assert_eq!(
        refused_without_trace(&mut setup, results(2, both)),
        uncovered,
        "the admissible first filing and its move are not kept"
    );
    let tags = ["eligible/x by 2", "eligible/y by 2", "eligible/z by 3"];
    let out = setup.records(2, &tags).unwrap();
    setup.submit(2, out.records.clone()).unwrap();
    reversed.submit(2, out.records).unwrap();
    assert_eq!(
        setup.node.state().results(),
        reversed.node.state().results()
    );
    assert_eq!(setup.escrows(&author), vec![low]);
    for item in [x, y, z] {
        let want = if item == high {
            State::AppealEligible
        } else {
            STARTED
        };
        assert_eq!(setup.state(&item), Some(&want));
        assert_eq!(reversed.state(&item), Some(&want));
    }
    setup.step(high, Event::AppealExpires).unwrap();
    let polarized = State::Rejected(RejectReason::Polarized);
    assert_eq!(setup.state(&high), Some(&polarized));
}

/// C6: a request after its window, first in CID order, spends no coverage; the next one starts.
#[test]
fn a_late_request_spends_no_coverage_of_the_next_in_cid_order() {
    let walks = [(2, "eligible/x"), (2, "eligible/y")];
    let mut setup = Setup::new("late-first", &walks, Some(scale()));
    let mut tags = ["eligible/x by 2", "eligible/y by 2"];
    tags.sort_by_key(|t| setup.item(t).0);
    let [(low, author), (high, _)] = tags.map(|t| setup.item(t));
    let out = setup
        .requests(2, &[(tags[0], false), (tags[1], true)])
        .unwrap();
    let want = AuthorRecords {
        records: vec![start(author, high)],
        admitted: vec![high],
        refused: vec![(low, AppealRefusal::WindowClosed)],
    };
    assert_eq!(out, want);
    setup.submit(2, out.records).unwrap();
    assert_eq!(setup.state(&high), Some(&STARTED));
    assert_eq!(setup.escrows(&author), vec![high]);
    assert_eq!(setup.step(low, Event::AppealExpires), Ok(Outcome::Moved));
}

/// C6: without `E_m` the producer refuses appeals and the node a filing; outcomes still record.
#[test]
fn without_a_scale_filings_are_refused_and_outcomes_recorded() {
    let mut setup = Setup::new("unscaled", &[(2, "pool"), (2, "eligible")], None);
    assert_eq!(
        setup.records(2, &["eligible by 2"]),
        Err(AuthorRecordsError::NoEpochScale)
    );
    let out = setup.records(2, &[]).unwrap();
    assert_eq!(out.records.len(), 1);
    let (item, author) = setup.item("eligible by 2");
    let mut records = out.records;
    records.push(start(author, item));
    let unscaled = Rejection::Results(ResultsRejected::NoEpochScale { record: 1 });
    let refused = refused_without_trace(&mut setup, results(2, records.clone()));
    assert_eq!(refused, unscaled);
    records.pop();
    assert_eq!(setup.submit(2, records), Ok(Outcome::Moved));
    assert_eq!(setup.state(&item), Some(&State::AppealEligible));
}

/// C5: an appeal step is refused without trace, whatever its inputs; its item may still expire.
#[test]
fn an_appeal_step_is_refused_without_trace() {
    let mut setup = Setup::new("step", &[(2, "eligible")], Some(scale()));
    let (item, _) = setup.item("eligible by 2");
    for within_window in [true, false] {
        let event = Event::Appeal {
            within_window,
            reputation_covers_stake: true,
        };
        let refused = refused_without_trace(&mut setup, on(item, event));
        assert_eq!(refused, Rejection::AppealStep);
    }
    assert_eq!(setup.step(item, Event::AppealExpires), Ok(Outcome::Moved));
}

/// C5: a started appeal neither expires nor files again, without trace.
#[test]
fn a_started_appeal_neither_expires_nor_files_again() {
    let mut setup = Setup::new("started", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    setup.submit(2, out.records).unwrap();
    let expiry = refused_without_trace(&mut setup, on(item, Event::AppealExpires));
    assert_eq!(expiry, Rejection::Lifecycle(Invalid::UnexpectedEvent));
    let again = refused_without_trace(&mut setup, results(3, vec![start(author, item)]));
    let not_appealable = ResultsRejected::NotAppealable { record: 0 };
    assert_eq!(again, Rejection::Results(not_appealable));
}

/// C5: a fresh author's valid request files and starts at once, `C_a` 0.4 then 1/3; no midway.
#[test]
fn a_fresh_author_s_appeal_files_and_starts_at_once() {
    let mut setup = Setup::new("fresh", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let p = AuthorPrior::default();
    let results = setup.node.state().results();
    assert_eq!(results.quota_reputation(&author, 2), Some(0.4));
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    assert_eq!(out.records, vec![start(author, item)]);
    let before = setup.reopen();
    assert_eq!(before.state().item(&item), Some(&State::AppealEligible));
    assert!(before.state().results().open_escrows(&author).is_empty());
    setup.submit(2, out.records).unwrap();
    for node in [&setup.node, &setup.reopen()] {
        assert_eq!(node.state().item(&item), Some(&STARTED));
        assert_eq!(node.state().results().open_escrows(&author), vec![item]);
    }
    let history = setup.node.state().results().author(&author).unwrap();
    let third = author_score(&[0.0], &[0.0], &p);
    assert_eq!(
        history.reputation(2, scale(), &p).to_bits(),
        third.to_bits()
    );
    assert!(!history.covers_stake(3, scale(), &p));
}

/// C5/C6: a pending or indeterminate pilot settles nothing; a failure settles once, seen at `e+1`.
#[test]
fn a_pending_pilot_settles_nothing_until_it_fails_and_counts_once() {
    let mut setup = Setup::new("pending", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    setup.submit(2, out.records).unwrap();
    assert_eq!(setup.records(3, &[]).unwrap(), AuthorRecords::default());
    let pending = [
        screen(Screening::Indeterminate),
        screen(Screening::Pass),
        stage2(Recheck::Indeterminate, false),
    ];
    for event in pending {
        setup.step(item, event).unwrap();
        assert_eq!(setup.records(3, &[]).unwrap(), AuthorRecords::default());
    }
    setup.step(item, stage2(Recheck::Dif, false)).unwrap();
    let out = setup.records(3, &[]).unwrap();
    let failed = ResultRecord::AppealSettled {
        author,
        item,
        passed: false,
    };
    assert_eq!(out.records, vec![failed]);
    setup.submit(3, out.records).unwrap();
    let history = setup
        .node
        .state()
        .results()
        .author(&author)
        .unwrap()
        .clone();
    assert_eq!(
        (history.outcomes(), history.outcome(&item)),
        (1, Some((false, 3)))
    );
    assert!(history.open_escrows().is_empty());
    let p = AuthorPrior::default();
    let escrow = author_score(&[0.0], &[0.0], &p);
    assert_eq!(
        history.reputation(3, scale(), &p).to_bits(),
        escrow.to_bits()
    );
    let aged = author_score(&[0.0], &[0.25], &p);
    assert_eq!(history.reputation(4, scale(), &p).to_bits(), aged.to_bits());
    assert_eq!(setup.records(4, &[]).unwrap(), AuthorRecords::default());
    assert_eq!(setup.reopen().state(), setup.node.state());
}

/// C6: a log replays under the `E_m` it was written with; another can refuse a filing it admitted.
#[test]
fn replay_holds_under_the_same_time_configuration_only() {
    let one = EpochScale::new(1.0).expect("a fixture E_m");
    let walks = [
        (2, "screen/a"),
        (2, "screen/b"),
        (2, "pool"),
        (2, "eligible"),
    ];
    let mut setup = Setup::new("same-scale", &walks, Some(one));
    let outcome = |s: &Setup, tag: &str, passed| {
        let (item, author) = s.item(tag);
        ResultRecord::AuthorOutcome {
            author,
            item,
            passed,
        }
    };
    let fails = vec![
        outcome(&setup, "screen/a by 2", false),
        outcome(&setup, "screen/b by 2", false),
    ];
    setup.submit(2, fails).unwrap();
    let pass = vec![outcome(&setup, "pool by 2", true)];
    setup.submit(12, pass).unwrap();
    let out = setup.records(22, &["eligible by 2"]).unwrap();
    assert_eq!(out.admitted.len(), 1, "covered with ages 20 and 10 months");
    setup.submit(22, out.records).unwrap();
    assert_eq!(setup.reopen().state(), setup.node.state());
    let entry = setup.node.log_len() as u64 - 1;
    let replayed = |scale| open(&setup.dir, &setup.issuer, scale).err();
    let uncovered = Rejection::Results(ResultsRejected::AppealNotCovered { record: 0 });
    assert_eq!(
        replayed(Some(scale())),
        Some(NodeError::ReplayRejected {
            entry,
            rejection: uncovered
        })
    );
    let unscaled = Rejection::Results(ResultsRejected::NoEpochScale { record: 0 });
    assert_eq!(
        replayed(None),
        Some(NodeError::ReplayRejected {
            entry,
            rejection: unscaled
        })
    );
}

/// C5: a request after its window starts nothing: no escrow, no trace, its item free to expire.
#[test]
fn a_late_appeal_leaves_no_escrow_and_its_item_free_to_expire() {
    let mut setup = Setup::new("late", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let out = setup.requests(2, &[("eligible by 2", false)]).unwrap();
    let refused = vec![(item, AppealRefusal::WindowClosed)];
    assert_eq!((out.records, out.refused), (vec![], refused));
    let late = ResultRecord::AppealStarted {
        author,
        item,
        within_window: false,
    };
    let closed = ResultsRejected::AppealWindowClosed { record: 0 };
    let refused = refused_without_trace(&mut setup, results(2, vec![late]));
    assert_eq!(refused, Rejection::Results(closed));
    assert_eq!(setup.escrows(&author), Vec::<Cid>::new());
    assert_eq!(setup.step(item, Event::AppealExpires), Ok(Outcome::Moved));
}
