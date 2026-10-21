//! C5–C6 composed offline (`docs/15` C5–C6): the author a deposit proves and the conclusion a
//! step reaches become an epoch's author records, which the node checks, applies and replays.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::Cid;
use protocol::appeal::InsufficientReputation;
use protocol::author_time::{AuthorConfig, AuthorPoint, CutContext};
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

/// C5 fixtures use nonclosing cuts; no identifier difference ages an outcome.
fn scale() -> AuthorConfig {
    AuthorConfig::new(45).expect("a positive E_m")
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

fn open(dir: &Path, issuer: &Issuer, scale: Option<AuthorConfig>) -> Result<Node, NodeError> {
    match scale {
        Some(scale) => Node::open_with_config(dir, issuer.public(), scale),
        None => Node::open(dir, issuer.public()),
    }
}

/// A node holding one deposit per `(author, walk)`, each item walked to its end.
struct Setup {
    node: Node,
    dir: PathBuf,
    issuer: Issuer,
    scale: Option<AuthorConfig>,
    items: Vec<(String, Cid, Nym)>,
    cut: u64,
}

impl Setup {
    fn new(name: &str, walks: &[(u8, &str)], scale: Option<AuthorConfig>) -> Self {
        let (issuer, dir) = (issuer(), scratch(name));
        let mut node = open(&dir, &issuer, scale).unwrap();
        node.begin_cut(CutContext {
            number: 0,
            epoch: 0,
            closes: false,
        })
        .unwrap();
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
            cut: 0,
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

    fn records(
        &mut self,
        epoch: u64,
        appeals: &[&str],
    ) -> Result<AuthorRecords, AuthorRecordsError> {
        let within: Vec<(&str, bool)> = appeals.iter().map(|t| (*t, true)).collect();
        self.requests(epoch, &within)
    }

    /// The records for the requests `(tag, within_window)`.
    fn requests(
        &mut self,
        epoch: u64,
        appeals: &[(&str, bool)],
    ) -> Result<AuthorRecords, AuthorRecordsError> {
        self.context(epoch);
        let appeals = appeals
            .iter()
            .map(|(t, within_window)| {
                let within_window = *within_window;
                (self.item(t).0, AppealRequest { within_window })
            })
            .collect();
        author_records(self.node.state(), point(epoch), &appeals)
    }

    fn context(&mut self, epoch: u64) {
        if self.node.state().results().author_point().unwrap().epoch != epoch {
            self.node.end_cut().unwrap();
            self.cut += 1;
            self.node
                .begin_cut(CutContext {
                    number: self.cut,
                    epoch,
                    closes: false,
                })
                .unwrap();
        }
    }

    fn submit(&mut self, epoch: u64, records: Vec<ResultRecord>) -> Result<Outcome, NodeError> {
        self.context(epoch);
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
    if let NodeEvent::Results(ref results) = event {
        setup.context(results.epoch);
    }
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
        assert_eq!(
            history.outcome(item),
            recorded.map(|p| (p, point(2))),
            "{tag}"
        );
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
        (2, Some((true, point(3))))
    );
    assert!(history.open_escrows().is_empty());
    let p = AuthorPrior::default();
    let with_escrow = author_score(&[1.0, 0.0], &[0.0, 0.0], &p);
    assert_eq!(
        history.reputation(point(3), scale(), &p).to_bits(),
        with_escrow.to_bits()
    );
    let mut both = [(earlier, 0.0), (appealed, 0.0)];
    both.sort_by_key(|(item, _)| *item);
    let ages: Vec<f64> = both.iter().map(|(_, a)| *a).collect();
    let passed = author_score(&[1.0, 1.0], &ages, &p);
    assert_eq!(
        history.reputation(point(4), scale(), &p).to_bits(),
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
            let out = author_records(node.state(), point(2), &appeals).unwrap();
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

/// C6: the explicit provisional default is H = 45 closings.
#[test]
fn the_default_config_allows_a_covered_filing() {
    let mut setup = Setup::new("default-config", &[(2, "eligible")], None);
    assert_eq!(
        setup
            .node
            .state()
            .results()
            .author_config()
            .half_life_closings(),
        45
    );
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    setup.submit(2, out.records).unwrap();
    assert_eq!(setup.reopen().state(), setup.node.state());
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
    assert_eq!(results.quota_reputation(&author), Some(0.4));
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
        history.reputation(point(2), scale(), &p).to_bits(),
        third.to_bits()
    );
    assert!(!history.covers_stake(point(3), scale(), &p));
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
        (1, Some((false, point(3))))
    );
    assert!(history.open_escrows().is_empty());
    let p = AuthorPrior::default();
    let escrow = author_score(&[0.0], &[0.0], &p);
    assert_eq!(
        history.reputation(point(3), scale(), &p).to_bits(),
        escrow.to_bits()
    );
    let aged = author_score(&[0.0], &[0.0], &p);
    assert_eq!(
        history.reputation(point(4), scale(), &p).to_bits(),
        aged.to_bits()
    );
    assert_eq!(setup.records(4, &[]).unwrap(), AuthorRecords::default());
    assert_eq!(setup.reopen().state(), setup.node.state());
}

/// C6: configuration mismatch is refused before replay, even without a filing.
#[test]
fn replay_holds_under_the_same_time_configuration_only() {
    let one = AuthorConfig::new(1).unwrap();
    let mut setup = Setup::new("same-config", &[(2, "pool")], Some(one));
    let out = setup.records(2, &[]).unwrap();
    setup.submit(2, out.records).unwrap();
    setup.node.end_cut().unwrap();
    setup.cut += 1;
    setup
        .node
        .begin_cut(CutContext {
            number: setup.cut,
            epoch: 3,
            closes: true,
        })
        .unwrap();
    setup.node.end_cut().unwrap();
    setup.cut += 1;
    setup
        .node
        .begin_cut(CutContext {
            number: setup.cut,
            epoch: 4,
            closes: false,
        })
        .unwrap();
    let (_, author) = setup.item("pool by 2");
    let at = AuthorPoint {
        epoch: 4,
        closings: 1,
    };
    let value = |node: &Node| {
        node.state()
            .results()
            .author(&author)
            .unwrap()
            .reputation(at, one, &AuthorPrior::default())
            .to_bits()
    };
    let replayed = setup.reopen();
    assert_eq!(replayed.state(), setup.node.state());
    assert_eq!(value(&replayed), value(&setup.node));
    assert_eq!(
        value(&replayed),
        scoring::reputation::author_score_closings(&[1.0], &[1], 1, &AuthorPrior::default(),)
            .to_bits()
    );
    assert_eq!(
        open(&setup.dir, &setup.issuer, Some(scale())).err(),
        Some(NodeError::Configuration)
    );
    assert_eq!(
        open(&setup.dir, &setup.issuer, None).err(),
        Some(NodeError::Configuration)
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

fn point(epoch: u64) -> AuthorPoint {
    AuthorPoint { epoch, closings: 0 }
}

// C6: verified cuts compose filing, pilot, settlement and B-b snapshot transitions.
#[test]
fn ledger_and_snapshot_follow_atomic_filing_records() {
    use network::consortium::{Consortium, Member};
    use network::cut::Cut;
    use network::log::TransparencyLog;
    use network::replica::{FeedWriter, Replica, WriterSet};
    use protocol::cohort_snapshot::{snapshot, CutInput, Incoherence, Slot};
    use protocol::ledger::{CutReport, Ledger, Refusal};
    use protocol::offline_log::OfflineRecord;
    const NET: [u8; 32] = [6; 32];
    let issuer = issuer();
    let members: Vec<_> = (10..13).map(|s| Member::from_seed([s; 32])).collect();
    let make_consortium = || Consortium::new(members.iter().map(|m| m.public()).collect(), 2);
    let signer = FeedWriter::from_seed(NET, [10; 32]);
    let keys = [signer.public()];
    let mut replica = Replica::new(WriterSet::new(NET, &keys));
    let mut feed = TransparencyLog::new();
    let mut publish = |bytes: Vec<u8>, replica: &mut Replica| {
        let entry = signer.sign(&feed.append(network::cid::cid(&bytes)).clone());
        let id = entry.id();
        replica.insert(entry, bytes).unwrap();
        id
    };
    let (deposit_a, a) = deposit(&issuer, &credential(&issuer, 2), "ledger-a");
    let (deposit_b, b) = deposit(&issuer, &credential(&issuer, 3), "ledger-b");
    let author_a = match &deposit_a {
        NodeEvent::Deposit { proof, .. } => proof.id(),
        _ => unreachable!(),
    };
    let author_b = match &deposit_b {
        NodeEvent::Deposit { proof, .. } => proof.id(),
        _ => unreachable!(),
    };
    let mut objects = vec![deposit_a, deposit_b];
    for item in [a, b] {
        objects.extend(
            walk(item, "eligible")
                .into_iter()
                .map(|event| on(item, event)),
        );
    }
    for object in objects {
        publish(object.encode(), &mut replica);
    }
    let invalid_batch = publish(
        results(
            500,
            vec![
                start(author_a, a),
                ResultRecord::ReviewerScore {
                    reviewer: Nym([40; 32]),
                    score: 0.0,
                    inclusion: 0.0,
                },
            ],
        )
        .encode(),
        &mut replica,
    );
    publish(
        results(999, vec![start(author_a, a)]).encode(),
        &mut replica,
    );
    let duplicate_batch = publish(
        results(501, vec![start(author_b, b)]).encode(),
        &mut replica,
    );
    let legacy = on(
        b,
        Event::Appeal {
            within_window: true,
            reputation_covers_stake: true,
        },
    );
    let legacy_id = publish(legacy.encode(), &mut replica);
    let injected = publish(
        OfflineRecord::Begin(CutContext {
            number: 0,
            epoch: 900,
            closes: true,
        })
        .encode(),
        &mut replica,
    );
    for event in [screen(Screening::Pass), stage2(Recheck::NoDif, false)] {
        publish(on(a, event).encode(), &mut replica);
    }
    publish(
        results(
            0,
            vec![ResultRecord::AppealSettled {
                author: author_a,
                item: a,
                passed: true,
            }],
        )
        .encode(),
        &mut replica,
    );
    let sign = |cut: &Cut| {
        let cp = cut.checkpoint(NET, make_consortium().member_set_hash());
        members[..2]
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect::<Vec<_>>()
    };
    let input = |cut: &Cut, report: CutReport, replica: &Replica| CutInput {
        number: cut.number,
        epoch: cut.epoch,
        closes: cut.closes,
        digest: cut.digest(),
        slots: report
            .applied
            .iter()
            .map(|id| Slot::Applied {
                id: *id,
                object: replica.get(id).unwrap().1.to_vec(),
            })
            .collect(),
        refused: report.refused,
    };
    let mut ledger = Ledger::new(NET, make_consortium(), issuer.public());
    let first = Cut {
        epoch: DEPOSITS,
        closes: true,
        ..Cut::of(&replica, 0)
    };
    let report = ledger.apply(&replica, &first, &sign(&first)).unwrap();
    assert_eq!(
        report.refused,
        vec![
            (
                invalid_batch,
                Refusal::Rejected(Rejection::Results(ResultsRejected::BadInclusion {
                    record: 1
                }))
            ),
            (
                duplicate_batch,
                Refusal::Rejected(Rejection::Results(ResultsRejected::FilingBatchRecorded))
            ),
            (legacy_id, Refusal::Rejected(Rejection::AppealStep)),
            (injected, Refusal::NotAnEvent),
        ]
    );
    assert_eq!(ledger.state().item(&a), Some(&State::ActivePool));
    assert_eq!(ledger.state().item(&b), Some(&State::AppealEligible));
    assert_eq!(ledger.state().results().closing_count(), 1);
    assert_eq!(
        ledger
            .state()
            .results()
            .author(&author_a)
            .unwrap()
            .outcome(&a),
        Some((true, point(DEPOSITS)))
    );
    let mut cuts = vec![input(&first, report, &replica)];
    let snap = snapshot(&cuts).unwrap();
    assert!(snap
        .items
        .iter()
        .find(|item| item.cid == a)
        .unwrap()
        .frozen
        .is_some());
    assert!(snap
        .items
        .iter()
        .find(|item| item.cid == b)
        .unwrap()
        .frozen
        .is_none());
    let requests = BTreeMap::from([(
        b,
        AppealRequest {
            within_window: true,
        },
    )]);
    assert_eq!(
        author_records(
            ledger.state(),
            AuthorPoint {
                epoch: DEPOSITS,
                closings: 0
            },
            &requests
        ),
        Err(AuthorRecordsError::FilingBatchRecorded)
    );
    let planned = AuthorPoint {
        epoch: 2,
        closings: 1,
    };
    let out = author_records(ledger.state(), planned, &requests).unwrap();
    assert_eq!(out.admitted, vec![b]);
    publish(results(1001, out.records).encode(), &mut replica);
    let second = Cut {
        epoch: 2,
        closes: false,
        ..Cut::of(&replica, 1)
    };
    let report = ledger.apply(&replica, &second, &sign(&second)).unwrap();
    assert!(report.refused.is_empty());
    assert_eq!(ledger.state().item(&b), Some(&STARTED));
    cuts.push(input(&second, report, &replica));
    let snap = snapshot(&cuts).unwrap();
    assert!(snap.items.iter().all(|item| item.frozen.is_some()));
    let fake = CutInput {
        number: 0,
        epoch: 1,
        closes: false,
        digest: [0; 32],
        slots: vec![Slot::Applied {
            id: legacy_id,
            object: legacy.encode(),
        }],
        refused: vec![],
    };
    assert!(matches!(
        snapshot(&[fake]),
        Err(protocol::cohort_snapshot::Error::Incoherent(
            Incoherence::AppealStep(_)
        ))
    ));
}
