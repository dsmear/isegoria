//! Item lifecycles rebuilt from a node's log (`docs/04` §Events and replay, `docs/08`
//! PROTO-014, T73 step 2): every §9.1 event is a logged step, replayed to the same state.

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::{Nym, Role};
use network::cid::Cid;
use network::store::{DurableLog, ObjectStore};
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::gate::GateOutcome;
use protocol::lifecycle::{step, Event, Invalid, State};
use protocol::node::{Node, NodeError, Outcome, Rejection};
use protocol::revalidation::Recheck;
use protocol::review::commit;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const EPOCH: u64 = 4;

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("lifecycle_replay")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

fn author(issuer: &Issuer, secret: u8) -> AnonymousCredential {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    pending.finalize(issuer.issue(&req).unwrap())
}

/// A deposit event for `tag` by a fresh author, and the item's CID.
fn deposit(issuer: &Issuer, secret: u8, tag: &str) -> (NodeEvent, Cid) {
    let draft = Draft {
        item: tag.as_bytes().to_vec(),
        primary_source: b"Gazzetta Ufficiale".to_vec(),
    };
    let item = draft.content_id();
    let proof = prove(
        &author(issuer, secret),
        &issuer.public(),
        Role::Propose,
        &deposit_context(item, EPOCH),
    );
    let event = NodeEvent::Deposit {
        epoch: EPOCH,
        quota: 1,
        draft,
        proof,
    };
    (event, item)
}

fn nyms(from: u8, n: u8) -> Vec<Nym> {
    (from..from + n).map(|i| Nym([i; 32])).collect()
}

/// A full commit-reveal round of `panel` on `item`, each panelist rating `prob`.
fn round(item: Cid, panel: &[Nym], prob: f64) -> Vec<Event> {
    let nonce = |n: &Nym| n.0;
    let mut events: Vec<Event> = panel
        .iter()
        .map(|n| Event::Commit {
            nym: *n,
            commitment: commit(prob, &nonce(n), *n, item),
        })
        .collect();
    events.push(Event::CloseCommits);
    events.extend(panel.iter().map(|n| Event::Reveal {
        nym: *n,
        prob,
        nonce: nonce(n),
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
    events.extend(round(item, &panel, 0.75));
    events.push(Event::Score { outcome });
    events
}

/// Four walks that between them use every lifecycle event: the pool, the band with an
/// appeal, an explored rejection, a contested fact and an expired appeal.
fn walks(items: &[Cid]) -> Vec<(Cid, Vec<Event>)> {
    let pilot = |passed: bool, source_verified: bool| {
        vec![
            Event::Pilot1Batch {
                enough_respondents: true,
                passed: true,
            },
            Event::Pilot2Batch {
                batch_size: 2,
                passed,
                source_verified,
            },
        ]
    };
    let mut pool = reviewed(items[0], GateOutcome::Pass);
    pool.extend(pilot(true, false));
    pool.extend([
        Event::Administer,
        Event::Revalidate {
            dif: Recheck::Indeterminate,
            source_verified: true,
        },
        Event::Revalidate {
            dif: Recheck::NoDif,
            source_verified: false,
        },
        Event::ExposureLimit,
    ]);
    let mut band = reviewed(items[1], GateOutcome::SupplementaryReview);
    let extra = nyms(40, 4);
    band.push(Event::AssignExtraReviewers {
        panel: extra.clone(),
    });
    band.extend(round(items[1], &extra, 0.25));
    band.extend([
        Event::Resolve {
            outcome: GateOutcome::AppealEligible,
        },
        Event::Appeal {
            within_window: true,
            reputation_covers_stake: true,
        },
    ]);
    band.extend(pilot(false, true));
    let mut explored = reviewed(items[2], GateOutcome::Reject);
    explored.push(Event::Explore {
        seed_from_beacon: true,
    });
    explored.extend(pilot(true, false));
    let mut expired = reviewed(items[3], GateOutcome::AppealEligible);
    expired.push(Event::AppealExpires);
    vec![
        (items[0], pool),
        (items[1], band),
        (items[2], explored),
        (items[3], expired),
    ]
}

/// AT-PRO-11: every lifecycle step is logged and a restart, at any point, replays each item
/// to the state `lifecycle::step` gives.
#[test]
fn at_pro_11_item_lifecycles_replay_to_the_state_they_reached() {
    let (issuer, dir) = (issuer(), scratch("walks"));
    let deposits: Vec<(NodeEvent, Cid)> = ["pool", "band", "explored", "expired"]
        .iter()
        .enumerate()
        .map(|(i, tag)| deposit(&issuer, i as u8 + 2, tag))
        .collect();
    let items: Vec<Cid> = deposits.iter().map(|(_, c)| *c).collect();
    let mut node = Node::open(&dir, issuer.public()).unwrap();
    for (event, item) in deposits {
        assert!(matches!(node.submit(event), Ok(Outcome::Admitted(_))));
        assert_eq!(node.state().item(&item), Some(&State::Deposited));
    }
    let mut expected: Vec<State> = vec![State::Deposited; 4];
    let mut submitted = 4;
    for (w, (item, events)) in walks(&items).into_iter().enumerate() {
        for event in events {
            expected[w] = step(expected[w].clone(), event.clone()).expect("a valid walk");
            let got = node.submit(NodeEvent::Step { item, event });
            assert_eq!(got, Ok(Outcome::Moved));
            submitted += 1;
            if submitted % 9 == 0 {
                drop(node);
                node = Node::open(&dir, issuer.public()).unwrap();
            }
            assert_eq!(node.state().item(&item), Some(&expected[w]));
        }
    }
    assert!(matches!(expected[0], State::Retired(_)));
    assert_eq!(expected[1], State::Contested);
    assert!(matches!(expected[2], State::Measured { passed: true, .. }));
    assert!(matches!(expected[3], State::Rejected(_)));
    let reopened = Node::open(&dir, issuer.public()).unwrap();
    assert_eq!(reopened.log_len(), submitted);
    assert_eq!(reopened.state(), node.state());
}

/// AT-PRO-11: a step the node refuses leaves no trace; unknown items and mismatched
/// assignments are refused besides the §9.1 table's own refusals.
#[test]
fn at_pro_11_refused_steps_leave_no_trace() {
    let (issuer, dir) = (issuer(), scratch("refused"));
    let (event, item) = deposit(&issuer, 2, "item");
    let mut node = Node::open(&dir, issuer.public()).unwrap();
    node.submit(event).unwrap();
    let before = Node::open(&dir, issuer.public()).unwrap();
    let refused = |r| Err(NodeError::Rejected(r));
    let score = Event::Score {
        outcome: GateOutcome::Pass,
    };
    assert_eq!(
        node.submit(NodeEvent::Step {
            item,
            event: score.clone()
        }),
        refused(Rejection::Lifecycle(Invalid::UnexpectedEvent))
    );
    let stranger = Cid([9; 32]);
    assert_eq!(
        node.submit(NodeEvent::Step {
            item: stranger,
            event: Event::Admit {
                seed_from_beacon: true
            }
        }),
        refused(Rejection::UnknownItem)
    );
    node.submit(NodeEvent::Step {
        item,
        event: Event::Admit {
            seed_from_beacon: true,
        },
    })
    .unwrap();
    assert_eq!(
        node.submit(NodeEvent::Step {
            item,
            event: Event::AssignReviewers {
                panel: nyms(10, 7),
                item: stranger,
            },
        }),
        refused(Rejection::ItemMismatch)
    );
    assert_eq!(node.log_len(), 2);
    assert_eq!(node.state().item(&item), Some(&State::Admitted));
    assert_ne!(node.state(), before.state());
    assert_eq!(before.state().item(&item), Some(&State::Deposited));
}

/// AT-PRO-11: a logged step this node's table refuses refuses the node at that entry.
#[test]
fn at_pro_11_an_unreplayable_step_refuses_the_node() {
    let (issuer, dir) = (issuer(), scratch("unreplayable"));
    let (event, item) = deposit(&issuer, 2, "item");
    Node::open(&dir, issuer.public())
        .unwrap()
        .submit(event)
        .unwrap();
    let bad = NodeEvent::Step {
        item,
        event: Event::CloseCommits,
    };
    let object = ObjectStore::open(&dir.join("objects"))
        .unwrap()
        .0
        .put(&bad.encode())
        .unwrap();
    DurableLog::open(&dir.join("log"))
        .unwrap()
        .0
        .append(object)
        .unwrap();
    assert_eq!(
        Node::open(&dir, issuer.public()).err(),
        Some(NodeError::ReplayRejected {
            entry: 1,
            rejection: Rejection::Lifecycle(Invalid::UnexpectedEvent),
        })
    );
}

/// AT-PRO-11: every lifecycle event round-trips; out-of-range bytes are refused.
#[test]
fn at_pro_11_lifecycle_events_round_trip_and_bad_bytes_are_refused() {
    let items: Vec<Cid> = (0..4u8).map(|i| Cid([i; 32])).collect();
    let mut seen = std::collections::BTreeSet::new();
    for (item, events) in walks(&items) {
        for event in events {
            let bytes = NodeEvent::Step { item, event }.encode();
            seen.insert(bytes[34]);
            let decoded = NodeEvent::decode(&bytes).expect("a step decodes");
            assert_eq!(decoded.encode(), bytes);
            for cut in 0..bytes.len() {
                assert!(NodeEvent::decode(&bytes[..cut]).is_none(), "cut at {cut}");
            }
        }
    }
    assert_eq!(
        seen,
        (1..=16).collect(),
        "every lifecycle event is exercised"
    );

    let step = |event| NodeEvent::Step {
        item: items[0],
        event,
    };
    let admit = step(Event::Admit {
        seed_from_beacon: true,
    })
    .encode();
    for (dif, byte) in [
        (Recheck::NoDif, 0u8),
        (Recheck::Dif, 1),
        (Recheck::Indeterminate, 2),
    ] {
        let revalidate = step(Event::Revalidate {
            dif,
            source_verified: true,
        })
        .encode();
        assert_eq!(revalidate[34..], [15, byte, 1]);
        let mut unknown = revalidate;
        unknown[35] = 3;
        assert!(NodeEvent::decode(&unknown).is_none());
    }
    let mut bad_bool = admit.clone();
    *bad_bool.last_mut().unwrap() = 2;
    assert!(NodeEvent::decode(&bad_bool).is_none());
    for number in [0u8, 17] {
        let mut unknown = admit.clone();
        unknown[34] = number;
        assert!(NodeEvent::decode(&unknown).is_none());
    }
    let score = step(Event::Score {
        outcome: GateOutcome::Reject,
    })
    .encode();
    let mut bad_outcome = score.clone();
    *bad_outcome.last_mut().unwrap() = 4;
    assert!(NodeEvent::decode(&bad_outcome).is_none());
    let assign = step(Event::AssignExtraReviewers { panel: nyms(1, 2) }).encode();
    let mut ragged = assign[..assign.len() - 1].to_vec();
    ragged[35..43].copy_from_slice(&63u64.to_le_bytes());
    assert!(
        NodeEvent::decode(&ragged).is_none(),
        "a panel field of 63 bytes"
    );
}
