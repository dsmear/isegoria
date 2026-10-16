//! An epoch's engine outputs as one logged event (`docs/04` §Events and replay, `docs/08`
//! PROTO-014, T73 step 3): replayed whole or refused whole, bound to their inputs' root.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::contested::PoolLimit;
use protocol::events::NodeEvent;
use protocol::node::{Node, NodeError, Outcome, Rejection};
use protocol::probation::SkillTrack;
use protocol::results::{
    answer_leaf, inclusion_proof, inputs_root, rating_leaf, verify_inclusion, EpochResults,
    ResultRecord, ResultsRejected,
};
use scoring::latent::Ability;
use scoring::reputation::CusumParams;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("results_replay")
        .join(format!("{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn issuer() -> identity::credential::IssuerPublic {
    identity::credential::Issuer::new([1u8; 32]).public()
}

fn nym(i: u8) -> Nym {
    Nym([i; 32])
}

/// A three-node ability of mean 0 and variance 1.
fn ability() -> Ability {
    Ability {
        nodes: vec![-1.5, 0.0, 1.5],
        weights: vec![2.0 / 9.0, 5.0 / 9.0, 2.0 / 9.0],
    }
}

fn fit(members: Vec<(Cid, u64)>) -> ResultRecord {
    ResultRecord::ContestedFit {
        pi: vec![0.6, 0.4],
        eta: vec![0.0, 0.5],
        a: vec![vec![1.0, 1.2], vec![1.0, 1.2]],
        b: vec![vec![0.0, 0.3], vec![0.5, 0.3]],
        c: vec![0.0, 0.2],
        ability: ability(),
        members,
    }
}

/// A well-formed contested fit of `classes` equal classes, `items` items and `nodes` ability
/// points: its curves would hold `items · classes² · nodes` values.
fn wide_fit(classes: usize, items: usize, nodes: usize, members: Vec<(Cid, u64)>) -> ResultRecord {
    ResultRecord::ContestedFit {
        pi: vec![1.0; classes],
        eta: vec![0.0; classes],
        a: vec![vec![1.0; items]; classes],
        b: vec![vec![0.0; items]; classes],
        c: vec![0.0; items],
        ability: Ability {
            nodes: (0..nodes).map(|q| q as f64 / nodes as f64).collect(),
            weights: vec![1.0; nodes],
        },
        members,
    }
}

/// Every kind of record but the author's, which need the node's deposits (`author_records.rs`).
fn every_record() -> Vec<ResultRecord> {
    let (x, y) = (cid(b"fact x"), cid(b"fact y"));
    vec![
        ResultRecord::ReviewerScore {
            reviewer: nym(1),
            score: 0.02,
            inclusion: 1.0,
        },
        ResultRecord::ReviewerScore {
            reviewer: nym(1),
            score: -0.4,
            inclusion: 0.05,
        },
        ResultRecord::ReviewerUnobserved { reviewer: nym(1) },
        ResultRecord::ReviewerUnobserved { reviewer: nym(2) },
        ResultRecord::Exposure {
            item: cid(b"item"),
            times: 3,
        },
        ResultRecord::Residual {
            reviewer: nym(2),
            item: 40,
            residual: -0.125,
        },
        ResultRecord::Residual {
            reviewer: nym(1),
            item: 40,
            residual: 0.25,
        },
        fit(vec![(x, 0), (y, 1)]),
        ResultRecord::ContestedRemove { item: y },
    ]
}

fn results(epoch: u64, records: Vec<ResultRecord>) -> NodeEvent {
    NodeEvent::Results(EpochResults {
        epoch,
        inputs_root: [epoch as u8; 32],
        records,
    })
}

/// AT-PRO-12: an epoch's results replay to the state the direct calls give, after a restart.
#[test]
fn at_pro_12_epoch_results_replay_to_the_state_they_produced() {
    let dir = scratch("replay");
    let mut node = Node::open(&dir, issuer()).unwrap();
    assert_eq!(node.submit(results(1, every_record())), Ok(Outcome::Moved));
    node.submit(results(
        2,
        vec![ResultRecord::Exposure {
            item: cid(b"item"),
            times: 2,
        }],
    ))
    .unwrap();
    let reopened = Node::open(&dir, issuer()).unwrap();
    assert_eq!(reopened.state(), node.state());
    let r = reopened.state().results();

    let params = CusumParams::default();
    let mut track = SkillTrack::new();
    track.record_observed(0.02, 1.0, &params);
    track.record_observed(-0.4, 0.05, &params);
    track.record_unobserved();
    assert_eq!(r.track(&nym(1)), Some(&track));
    assert_eq!(r.track(&nym(2)).map(|t| t.reviewed()), Some(1));
    assert!(r.track(&nym(3)).is_none());

    assert_eq!(r.exposure(&cid(b"item")), 5);
    let (residuals, index) = r.residuals();
    assert_eq!(
        (index[&nym(2)], index[&nym(1)]),
        (0, 1),
        "indexed as they first appear"
    );
    assert_eq!(residuals.reviewers(), 2);
    assert!(r.contested().contains(&cid(b"fact x")));
    assert!(!r.contested().contains(&cid(b"fact y")));
    assert_eq!(r.epoch_root(1), Some([1; 32]));
    assert_eq!(r.epoch_root(3), None);
}

/// AT-PRO-12: each refused record refuses the whole event, which leaves no trace.
#[test]
fn at_pro_12_results_are_applied_whole_or_not_at_all() {
    let dir = scratch("refused");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.submit(results(1, every_record())).unwrap();
    let before = Node::open(&dir, issuer()).unwrap();
    let unobserved = ResultRecord::ReviewerUnobserved { reviewer: nym(9) };
    let score = |score, inclusion| ResultRecord::ReviewerScore {
        reviewer: nym(1),
        score,
        inclusion,
    };
    let cases: Vec<(u64, Vec<ResultRecord>, ResultsRejected)> = vec![
        (1, vec![], ResultsRejected::EpochRecorded),
        (
            2,
            vec![score(0.1, 0.0)],
            ResultsRejected::BadInclusion { record: 0 },
        ),
        (
            2,
            vec![score(0.1, 1.5)],
            ResultsRejected::BadInclusion { record: 0 },
        ),
        (
            2,
            vec![score(0.1, f64::NAN)],
            ResultsRejected::BadInclusion { record: 0 },
        ),
        (
            2,
            vec![unobserved, score(f64::NAN, 1.0)],
            ResultsRejected::NotFinite { record: 1 },
        ),
        (
            2,
            vec![ResultRecord::Residual {
                reviewer: nym(1),
                item: 1,
                residual: f64::NAN,
            }],
            ResultsRejected::NotFinite { record: 0 },
        ),
        (
            2,
            vec![ResultRecord::ContestedFit {
                pi: vec![0.6, 0.0],
                eta: vec![0.0, 0.5],
                a: vec![vec![1.0], vec![1.0]],
                b: vec![vec![0.0], vec![0.5]],
                c: vec![0.0],
                ability: ability(),
                members: vec![(cid(b"z"), 0)],
            }],
            ResultsRejected::BadClasses { record: 0 },
        ),
        (
            2,
            vec![ResultRecord::ContestedFit {
                pi: vec![0.6, 0.4],
                eta: vec![0.0, 0.5],
                a: vec![vec![1.0], vec![1.0]],
                b: vec![vec![0.0], vec![0.5]],
                c: vec![1.0],
                ability: ability(),
                members: vec![(cid(b"z"), 0)],
            }],
            ResultsRejected::BadClasses { record: 0 },
        ),
        (
            2,
            vec![ResultRecord::ContestedFit {
                pi: vec![0.6, 0.4],
                eta: vec![0.0, 0.5],
                a: vec![vec![1.0], vec![1.0]],
                b: vec![vec![0.0], vec![0.5]],
                c: vec![0.0],
                ability: Ability {
                    nodes: vec![0.0, 1.0],
                    weights: vec![1.0, -0.5],
                },
                members: vec![(cid(b"z"), 0)],
            }],
            ResultsRejected::BadClasses { record: 0 },
        ),
        (
            2,
            vec![fit(vec![(cid(b"z"), 0), (cid(b"z"), 1)])],
            ResultsRejected::BadMembers { record: 0 },
        ),
        (
            2,
            vec![fit(vec![(cid(b"z"), 2)])],
            ResultsRejected::BadMembers { record: 0 },
        ),
        (
            2,
            vec![fit(vec![(cid(b"z"), 0), (cid(b"y"), 1), (cid(b"x"), 0)])],
            ResultsRejected::BadMembers { record: 0 },
        ),
        (
            2,
            vec![ResultRecord::ContestedFit {
                pi: vec![0.6, 0.0],
                eta: vec![0.0, 0.5],
                a: vec![vec![1.0], vec![1.0]],
                b: vec![vec![0.0], vec![0.5]],
                c: vec![0.0],
                ability: ability(),
                members: vec![],
            }],
            ResultsRejected::BadClasses { record: 0 },
        ),
        (
            2,
            vec![wide_fit(17, 1, 3, vec![(cid(b"wide"), 0)])],
            ResultsRejected::ContestedOutOfDomain {
                record: 0,
                limit: PoolLimit::Classes,
            },
        ),
        (
            2,
            vec![wide_fit(16, 4096, 256, vec![(cid(b"wide"), 0)])],
            ResultsRejected::ContestedOutOfDomain {
                record: 0,
                limit: PoolLimit::Bytes,
            },
        ),
        (
            2,
            vec![wide_fit(16, 4096, 256, vec![])],
            ResultsRejected::ContestedOutOfDomain {
                record: 0,
                limit: PoolLimit::Bytes,
            },
        ),
    ];
    for (epoch, records, why) in cases {
        let mut all = vec![ResultRecord::Exposure {
            item: cid(b"first"),
            times: 1,
        }];
        let shift = |w: ResultsRejected| match w {
            ResultsRejected::EpochRecorded => w,
            ResultsRejected::BadInclusion { record } => {
                ResultsRejected::BadInclusion { record: record + 1 }
            }
            ResultsRejected::NotFinite { record } => {
                ResultsRejected::NotFinite { record: record + 1 }
            }
            ResultsRejected::BadClasses { record } => {
                ResultsRejected::BadClasses { record: record + 1 }
            }
            ResultsRejected::BadMembers { record } => {
                ResultsRejected::BadMembers { record: record + 1 }
            }
            ResultsRejected::ContestedOutOfDomain { record, limit } => {
                ResultsRejected::ContestedOutOfDomain {
                    record: record + 1,
                    limit,
                }
            }
            other => unreachable!("the author's refusals are in author_inputs.rs: {other:?}"),
        };
        all.extend(records);
        assert_eq!(
            node.submit(results(epoch, all)),
            Err(NodeError::Rejected(Rejection::Results(shift(why)))),
            "{why:?}"
        );
        assert_eq!(node.state(), before.state(), "{why:?} left a trace");
    }
    assert_eq!(node.log_len(), 1);
}

/// AT-PRO-12: the inputs' root is a function of their set, and each participant can prove its
/// own input was counted against the root the node recorded.
#[test]
fn at_pro_12_the_inputs_root_binds_the_results_to_their_inputs() {
    let item = cid(b"item");
    let mut leaves: Vec<Vec<u8>> = (0..9u8)
        .map(|i| rating_leaf(nym(i), item, f64::from(i) / 10.0))
        .collect();
    leaves
        .extend((0..6u8).map(|i| answer_leaf(nym(50 + i), cid(b"batch"), u64::from(i % 3), i % 2)));
    let root = inputs_root(&leaves);
    let mut shuffled = leaves.clone();
    shuffled.reverse();
    shuffled.rotate_left(4);
    assert_eq!(inputs_root(&shuffled), root);
    assert_ne!(inputs_root(&leaves[1..]), root);
    assert_ne!(
        rating_leaf(nym(1), item, 0.1),
        rating_leaf(nym(1), item, 0.2)
    );
    assert_ne!(
        rating_leaf(nym(1), item, 0.1),
        rating_leaf(nym(2), item, 0.1)
    );
    assert_ne!(
        answer_leaf(nym(1), item, 0, 1),
        answer_leaf(nym(1), item, 1, 1)
    );
    assert_ne!(
        rating_leaf(nym(1), item, 0.0),
        answer_leaf(nym(1), item, 0, 0)
    );

    let dir = scratch("root");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.submit(NodeEvent::Results(EpochResults {
        epoch: 5,
        inputs_root: root,
        records: vec![],
    }))
    .unwrap();
    let recorded = Node::open(&dir, issuer())
        .unwrap()
        .state()
        .results()
        .epoch_root(5)
        .unwrap();
    for leaf in &leaves {
        let proof = inclusion_proof(&leaves, leaf).expect("an input of the set");
        assert!(verify_inclusion(leaf, &proof, recorded));
        assert!(!verify_inclusion(leaf, &proof, [0; 32]));
    }
    let outsider = rating_leaf(nym(99), item, 0.5);
    assert!(inclusion_proof(&leaves, &outsider).is_none());
    let proof = inclusion_proof(&leaves, &leaves[0]).unwrap();
    assert!(!verify_inclusion(&outsider, &proof, recorded));
    assert_eq!(inputs_root(&[]), inputs_root(&[]));
    assert_ne!(inputs_root(&[]), root);
}

/// AT-PRO-12: results events round-trip; cuts and out-of-range bytes are refused.
#[test]
fn at_pro_12_results_round_trip_and_bad_bytes_are_refused() {
    let event = results(3, every_record());
    let bytes = event.encode();
    let decoded = NodeEvent::decode(&bytes).expect("results decode");
    assert_eq!(decoded.encode(), bytes);
    for cut in 0..bytes.len() {
        assert!(NodeEvent::decode(&bytes[..cut]).is_none(), "cut at {cut}");
    }
    let header = 2 + 8 + 32 + 8;
    let (author, item) = (nym(7), cid(b"item"));
    let authors = vec![
        ResultRecord::AuthorOutcome {
            author,
            item,
            passed: true,
        },
        ResultRecord::AppealFiled { author, item },
        ResultRecord::AppealSettled {
            author,
            item,
            passed: false,
        },
    ];
    let numbers = [1u8, 1, 2, 2, 6, 7, 7, 8, 9, 10, 11, 12];
    let records = every_record().into_iter().chain(authors.clone());
    for (record, number) in records.zip(numbers) {
        let one = results(3, vec![record]).encode();
        assert_eq!(one[header], number);
        let decoded = NodeEvent::decode(&one).expect("one record decodes");
        assert_eq!(decoded.encode(), one);
        // 3 to 5 are the retired item-less author records (`docs/15` C5–C6).
        for bad in [0u8, 3, 4, 5, 13] {
            let mut unknown = one.clone();
            unknown[header] = bad;
            assert!(NodeEvent::decode(&unknown).is_none());
        }
    }
    for flagged in [&authors[0], &authors[2]] {
        let one = results(3, vec![flagged.clone()]).encode();
        let mut bad_flag = one.clone();
        *bad_flag.last_mut().unwrap() = 2;
        assert!(NodeEvent::decode(&bad_flag).is_none());
    }
}
