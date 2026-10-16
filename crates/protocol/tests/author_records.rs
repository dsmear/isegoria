//! C5–C6 composed offline (`docs/15` C5–C6): the author a deposit proves and the conclusion a
//! step reaches become an epoch's author records, which the node checks, applies and replays.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::Cid;
use protocol::appeal::EpochScale;
use protocol::authorship::{author_records, AuthorRecords, AuthorRecordsError};
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
use std::collections::BTreeSet;
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

fn appeal() -> Event {
    Event::Appeal {
        within_window: true,
        reputation_covers_stake: true,
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
        let appeals = appeals.iter().map(|t| self.item(t).0).collect();
        author_records(self.node.state(), epoch, &appeals)
    }

    fn submit(&mut self, epoch: u64, records: Vec<ResultRecord>) -> Result<Outcome, NodeError> {
        self.node.submit(NodeEvent::Results(EpochResults {
            epoch,
            inputs_root: [epoch as u8; 32],
            records,
        }))
    }
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

/// C5: the node refuses an author record its deposits and steps do not support, whole, no trace.
#[test]
fn the_node_refuses_author_records_its_log_does_not_support() {
    let walks = [
        (2, "pool"),
        (2, "pending"),
        (2, "explored"),
        (2, "defect"),
        (2, "eligible/appealed"),
        (3, "eligible"),
    ];
    let mut setup = Setup::new("refused", &walks, Some(scale()));
    let (pool, author) = setup.item("pool by 2");
    let (other, _) = setup.item("eligible by 3");
    let (appealed, _) = setup.item("eligible/appealed by 2");
    let filed = ResultRecord::AppealFiled {
        author,
        item: appealed,
    };
    setup.submit(2, vec![filed]).unwrap();
    for event in [
        appeal(),
        screen(Screening::Pass),
        stage2(Recheck::NoDif, false),
    ] {
        setup.step(appealed, event).unwrap();
    }
    let outcome = |item, author, passed| ResultRecord::AuthorOutcome {
        author,
        item,
        passed,
    };
    let item = |tag: &str| setup.item(tag).0;
    let valid = outcome(pool, author, true);
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
            vec![
                valid.clone(),
                ResultRecord::AppealFiled { author, item: pool },
            ],
            ResultsRejected::NotAppealable { record: 1 },
        ),
    ];
    let before = setup.reopen();
    for (records, why) in cases {
        let refused = Err(NodeError::Rejected(Rejection::Results(why)));
        assert_eq!(setup.submit(3, records), refused);
        assert_eq!(setup.node.state(), before.state());
        assert_eq!(setup.node.log_len(), before.log_len());
    }
    assert_eq!(setup.submit(3, vec![valid]), Ok(Outcome::Moved));
}

/// C5/C6: an appeal filed through the node settles its escrow; the item counts once, from `e + 1`.
#[test]
fn an_appeal_through_the_node_settles_its_escrow_and_counts_once() {
    let mut setup = Setup::new("appeal", &[(2, "pool"), (2, "eligible")], Some(scale()));
    let ((earlier, author), (appealed, _)) = (setup.item("pool by 2"), setup.item("eligible by 2"));
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    assert_eq!(out.admitted, vec![appealed]);
    let filing = ResultRecord::AppealFiled {
        author,
        item: appealed,
    };
    assert_eq!(out.records.last(), Some(&filing));
    setup.submit(2, out.records).unwrap();
    for event in [
        appeal(),
        screen(Screening::Pass),
        stage2(Recheck::NoDif, false),
    ] {
        setup
            .node
            .submit(NodeEvent::Step {
                item: appealed,
                event,
            })
            .unwrap();
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
    for arrival in [[x, y, z], [z, y, x], [y, z, x]] {
        let appeals: BTreeSet<Cid> = arrival.into_iter().collect();
        for node in [&setup.node, &reversed.node] {
            let out = author_records(node.state(), 2, &appeals).unwrap();
            assert_eq!((&out.admitted, &out.refused), (&admitted, &vec![high]));
        }
    }
    let both = vec![
        ResultRecord::AppealFiled { author, item: high },
        ResultRecord::AppealFiled { author, item: low },
    ];
    let refused = Rejection::Results(ResultsRejected::AppealNotCovered { record: 1 });
    let log = setup.node.log_len();
    assert_eq!(setup.submit(2, both), Err(NodeError::Rejected(refused)));
    assert_eq!(
        setup.node.log_len(),
        log,
        "the admissible first filing is not kept"
    );
    let appeals: BTreeSet<Cid> = [x, y, z].into_iter().collect();
    let out = author_records(setup.node.state(), 2, &appeals).unwrap();
    setup.submit(2, out.records.clone()).unwrap();
    reversed.submit(2, out.records).unwrap();
    assert_eq!(
        setup.node.state().results(),
        reversed.node.state().results()
    );
    assert_eq!(
        setup.node.state().results().open_escrows(&author),
        vec![low]
    );
    for item in admitted {
        setup
            .node
            .submit(NodeEvent::Step {
                item,
                event: appeal(),
            })
            .unwrap();
    }
    let expire = NodeEvent::Step {
        item: high,
        event: Event::AppealExpires,
    };
    setup.node.submit(expire).unwrap();
    let polarized = State::Rejected(RejectReason::Polarized);
    assert_eq!(setup.node.state().item(&high), Some(&polarized));
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
    records.push(ResultRecord::AppealFiled { author, item });
    let refused = Rejection::Results(ResultsRejected::NoEpochScale { record: 1 });
    assert_eq!(
        setup.submit(2, records.clone()),
        Err(NodeError::Rejected(refused))
    );
    records.pop();
    assert_eq!(setup.submit(2, records), Ok(Outcome::Moved));
}

/// Submits `event` on `item` and checks the node refused it, its state and log as before.
fn refused_without_trace(setup: &mut Setup, item: Cid, event: Event) -> NodeError {
    let before = setup.reopen();
    let got = setup.node.submit(NodeEvent::Step { item, event });
    let Err(e @ NodeError::Rejected(_)) = got else {
        panic!(
            "accepted: {got:?}, the item now {:?}",
            setup.node.state().item(&item)
        );
    };
    assert_eq!(setup.node.state(), before.state());
    assert_eq!(setup.node.log_len(), before.log_len());
    assert_eq!(setup.reopen().state(), before.state());
    e
}

/// C5: an appeal step with no escrow open for its item is refused, state and log unchanged.
#[test]
fn an_appeal_without_its_escrow_is_refused_without_trace() {
    let mut setup = Setup::new("no-escrow", &[(2, "eligible")], Some(scale()));
    let (item, _) = setup.item("eligible by 2");
    refused_without_trace(&mut setup, item, appeal());
    assert_eq!(setup.node.state().item(&item), Some(&State::AppealEligible));
}

/// C5: once its filing is recorded, the item's appeal cannot expire; state and log unchanged.
#[test]
fn an_expiry_after_a_recorded_filing_is_refused_without_trace() {
    let mut setup = Setup::new("filed-expiry", &[(2, "eligible")], Some(scale()));
    let (item, _) = setup.item("eligible by 2");
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    assert_eq!(out.admitted, vec![item]);
    setup.submit(2, out.records).unwrap();
    refused_without_trace(&mut setup, item, Event::AppealExpires);
    assert_eq!(setup.node.state().item(&item), Some(&State::AppealEligible));
}

/// C5: a fresh author's filing leaves `C_a` at 1/3, yet its appeal starts; the window still holds.
#[test]
fn a_filed_appeal_starts_although_c_a_is_now_below_the_floor() {
    let mut setup = Setup::new("fresh", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    setup.submit(2, out.records).unwrap();
    let history = setup
        .node
        .state()
        .results()
        .author(&author)
        .unwrap()
        .clone();
    let p = AuthorPrior::default();
    let third = author_score(&[0.0], &[0.0], &p);
    assert_eq!(
        history.reputation(2, scale(), &p).to_bits(),
        third.to_bits()
    );
    assert!(!history.covers_stake(3, scale(), &p));
    let late = Event::Appeal {
        within_window: false,
        reputation_covers_stake: true,
    };
    let closed = Rejection::Lifecycle(Invalid::AppealWindowClosed);
    assert_eq!(
        refused_without_trace(&mut setup, item, late),
        NodeError::Rejected(closed)
    );
    assert_eq!(setup.step(item, appeal()), Ok(Outcome::Moved));
    let pilot = State::Pilot1 { appealed: true };
    assert_eq!(setup.node.state().item(&item), Some(&pilot));
}

/// C5: another item's escrow authorizes no appeal; an item with no filing may still expire.
#[test]
fn only_the_item_s_own_escrow_starts_its_appeal_or_bars_its_expiry() {
    let walks = [(2, "eligible/x"), (2, "eligible/y")];
    let mut setup = Setup::new("own-escrow", &walks, Some(scale()));
    let (x, y) = (
        setup.item("eligible/x by 2").0,
        setup.item("eligible/y by 2").0,
    );
    let out = setup.records(2, &["eligible/x by 2"]).unwrap();
    assert_eq!(out.admitted, vec![x]);
    setup.submit(2, out.records).unwrap();
    let no_escrow = NodeError::Rejected(Rejection::NoEscrow);
    assert_eq!(refused_without_trace(&mut setup, y, appeal()), no_escrow);
    let open = NodeError::Rejected(Rejection::EscrowOpen);
    assert_eq!(
        refused_without_trace(&mut setup, x, Event::AppealExpires),
        open
    );
    assert_eq!(setup.step(y, Event::AppealExpires), Ok(Outcome::Moved));
    let polarized = State::Rejected(RejectReason::Polarized);
    assert_eq!(setup.node.state().item(&y), Some(&polarized));
    assert_eq!(setup.step(x, appeal()), Ok(Outcome::Moved));
}

/// C5/C6: a pending or indeterminate pilot settles nothing; a failure settles once, seen at `e+1`.
#[test]
fn a_pending_pilot_settles_nothing_until_it_fails_and_counts_once() {
    let mut setup = Setup::new("pending", &[(2, "eligible")], Some(scale()));
    let (item, author) = setup.item("eligible by 2");
    let out = setup.records(2, &["eligible by 2"]).unwrap();
    setup.submit(2, out.records).unwrap();
    let pending = [
        appeal(),
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
