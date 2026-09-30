//! An epoch's engine outputs as one logged event (`docs/04` §Events and replay, `docs/08`
//! PROTO-014, T73 step 3): replayed whole or refused whole, bound to their inputs' root.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::appeal::{AppealOutcome, AuthorHistory};
use protocol::events::NodeEvent;
use protocol::node::{Node, NodeError, Outcome, Rejection};
use protocol::probation::SkillTrack;
use protocol::results::{
    answer_leaf, inclusion_proof, inputs_root, rating_leaf, verify_inclusion, EpochResults,
    ResultRecord, ResultsRejected,
};
use scoring::latent::Ability;
use scoring::reputation::{AuthorPrior, CusumParams};
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

/// Every kind of record, in an order where each is valid.
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
        ResultRecord::AuthorQuality {
            author: nym(7),
            quality: 0.9,
            age_months: 2.0,
        },
        ResultRecord::AppealFiled { author: nym(7) },
        ResultRecord::AppealSettled {
            author: nym(7),
            escrow: 1,
            promoted: Some(0.8),
        },
        ResultRecord::AppealFiled { author: nym(7) },
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

    let prior = AuthorPrior::default();
    let mut author = AuthorHistory::new();
    author.record(0.9, 2.0);
    let escrow = author.file_appeal(&prior).unwrap();
    author.settle(escrow, AppealOutcome::Promoted { quality: 0.8 });
    author.file_appeal(&prior).unwrap();
    assert_eq!(r.author(&nym(7)), Some(&author));
    assert_eq!(r.open_escrows(&nym(7)), vec![2]);

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
    let low = ResultRecord::AuthorQuality {
        author: nym(9),
        quality: 0.0,
        age_months: 0.0,
    };
    let score = |score, inclusion| ResultRecord::ReviewerScore {
        reviewer: nym(1),
        score,
        inclusion,
    };
    let settle = |escrow| ResultRecord::AppealSettled {
        author: nym(7),
        escrow,
        promoted: None,
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
            vec![low.clone(), score(f64::NAN, 1.0)],
            ResultsRejected::NotFinite { record: 1 },
        ),
        (
            2,
            vec![ResultRecord::AuthorQuality {
                author: nym(9),
                quality: f64::INFINITY,
                age_months: 0.0,
            }],
            ResultsRejected::NotFinite { record: 0 },
        ),
        (
            2,
            vec![ResultRecord::AuthorQuality {
                author: nym(9),
                quality: 0.5,
                age_months: f64::NAN,
            }],
            ResultsRejected::NotFinite { record: 0 },
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
            vec![ResultRecord::AppealSettled {
                author: nym(7),
                escrow: 2,
                promoted: Some(f64::NAN),
            }],
            ResultsRejected::NotFinite { record: 0 },
        ),
        (
            2,
            vec![low.clone(), ResultRecord::AppealFiled { author: nym(9) }],
            ResultsRejected::AppealNotCovered { record: 1 },
        ),
        (
            2,
            vec![settle(0)],
            ResultsRejected::NoOpenEscrow { record: 0 },
        ),
        (
            2,
            vec![settle(1)],
            ResultsRejected::NoOpenEscrow { record: 0 },
        ),
        (
            2,
            vec![settle(2), settle(2)],
            ResultsRejected::NoOpenEscrow { record: 1 },
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
            ResultsRejected::AppealNotCovered { record } => {
                ResultsRejected::AppealNotCovered { record: record + 1 }
            }
            ResultsRejected::NoOpenEscrow { record } => {
                ResultsRejected::NoOpenEscrow { record: record + 1 }
            }
            ResultsRejected::BadClasses { record } => {
                ResultsRejected::BadClasses { record: record + 1 }
            }
            ResultsRejected::BadMembers { record } => {
                ResultsRejected::BadMembers { record: record + 1 }
            }
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
    let numbers = [1u8, 1, 2, 2, 3, 4, 5, 4, 6, 7, 7, 8, 9];
    for (record, number) in every_record().into_iter().zip(numbers) {
        let one = results(3, vec![record]).encode();
        assert_eq!(one[header], number);
        for bad in [0u8, 10] {
            let mut unknown = one.clone();
            unknown[header] = bad;
            assert!(NodeEvent::decode(&unknown).is_none());
        }
    }
    let settle = results(
        3,
        vec![ResultRecord::AppealSettled {
            author: nym(7),
            escrow: 1,
            promoted: None,
        }],
    )
    .encode();
    let failed = NodeEvent::decode(&settle).expect("a failed appeal decodes");
    assert_eq!(failed.encode(), settle);
    let mut bad_flag = settle.clone();
    *bad_flag.last_mut().unwrap() = 2;
    assert!(NodeEvent::decode(&bad_flag).is_none());
}
