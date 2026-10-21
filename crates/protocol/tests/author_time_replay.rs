//! Offline C6 frames pin configuration and declared context; they authenticate no cut.

use identity::credential::{Issuer, IssuerPublic};
use identity::nym::Nym;
use network::store::{DurableLog, ObjectStore};
use protocol::author_time::{AuthorConfig, AuthorPoint, ContextError, CutContext};
use protocol::events::NodeEvent;
use protocol::node::{Node, NodeError, Rejection};
use protocol::offline_log::OfflineRecord;
use protocol::results::{EpochResults, ResultRecord, ResultsRejected};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn issuer() -> IssuerPublic {
    Issuer::new([1; 32]).public()
}
fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("author_time_replay")
        .join(format!(
            "{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}
fn cut(number: u64, epoch: u64, closes: bool) -> CutContext {
    CutContext {
        number,
        epoch,
        closes,
    }
}
fn event(data_epoch: u64) -> NodeEvent {
    NodeEvent::Results(EpochResults {
        epoch: data_epoch,
        inputs_root: [1; 32],
        records: vec![ResultRecord::ReviewerUnobserved {
            reviewer: Nym([2; 32]),
        }],
    })
}
fn append(dir: &Path, bytes: &[u8]) {
    let id = ObjectStore::open(&dir.join("objects"))
        .unwrap()
        .0
        .put(bytes)
        .unwrap();
    DurableLog::open(&dir.join("log"))
        .unwrap()
        .0
        .append(id)
        .unwrap();
}

// C6: replay resumes an incomplete valid frame and counts its closing only on End.
#[test]
fn restart_in_a_closing_frame_can_resume_without_aging_early() {
    let dir = scratch("resume");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.begin_cut(cut(0, 10, true)).unwrap();
    node.submit(event(900)).unwrap();
    drop(node);
    let mut resumed = Node::open(&dir, issuer()).unwrap();
    assert_eq!(resumed.state().results().closing_count(), 0);
    assert_eq!(
        resumed.state().results().author_point(),
        Some(AuthorPoint {
            epoch: 10,
            closings: 0
        })
    );
    assert_eq!(
        resumed.begin_cut(cut(1, 11, false)),
        Err(NodeError::Context(ContextError::AlreadyOpen))
    );
    resumed.end_cut().unwrap();
    assert_eq!(resumed.state().results().closing_count(), 1);
    resumed.begin_cut(cut(1, 100, false)).unwrap();
    resumed.submit(event(2)).unwrap();
    resumed.end_cut().unwrap();
    let reopened = Node::open(&dir, issuer()).unwrap();
    assert_eq!(reopened.state(), resumed.state());
    assert_eq!(reopened.log_len(), 2);
    assert_eq!(reopened.state().results().closing_count(), 1);
}

// C6: every offline Event needs a frame before mutation, including nonauthor results.
#[test]
fn runtime_and_replay_reject_an_event_outside_a_frame() {
    let dir = scratch("outside");
    let mut node = Node::open(&dir, issuer()).unwrap();
    assert_eq!(
        node.submit(event(1)),
        Err(NodeError::Context(ContextError::NotOpen))
    );
    assert_eq!(node.log_len(), 0);
    assert_eq!(node.state().results().epoch_root(1), None);
    drop(node);
    append(&dir, &OfflineRecord::Event(event(1)).encode());
    assert_eq!(
        Node::open(&dir, issuer()).err(),
        Some(NodeError::Context(ContextError::NotOpen))
    );
}

// C6: a repeated End or missing End before a new Begin is refused, without aging twice.
#[test]
fn malformed_frame_boundaries_are_rejected() {
    let dir = scratch("extra-end");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.begin_cut(cut(0, 5, true)).unwrap();
    node.end_cut().unwrap();
    assert_eq!(
        node.end_cut(),
        Err(NodeError::Context(ContextError::NotOpen))
    );
    drop(node);
    append(&dir, &OfflineRecord::End.encode());
    assert_eq!(
        Node::open(&dir, issuer()).err(),
        Some(NodeError::Context(ContextError::NotOpen))
    );
    let dir = scratch("nested");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.begin_cut(cut(0, 5, true)).unwrap();
    drop(node);
    append(&dir, &OfflineRecord::Begin(cut(1, 6, false)).encode());
    assert_eq!(
        Node::open(&dir, issuer()).err(),
        Some(NodeError::Context(ContextError::AlreadyOpen))
    );
}

// C6: the log's immutable H is checked even if it holds no author observations.
#[test]
fn configuration_mismatch_is_refused_before_event_replay() {
    let dir = scratch("config");
    let config = AuthorConfig::new(90).unwrap();
    let mut node = Node::open_with_config(&dir, issuer(), config).unwrap();
    node.begin_cut(cut(0, 0, true)).unwrap();
    node.submit(event(1)).unwrap();
    node.end_cut().unwrap();
    assert_eq!(
        Node::open_with_config(&dir, issuer(), config)
            .unwrap()
            .state(),
        node.state()
    );
    assert_eq!(
        Node::open(&dir, issuer()).err(),
        Some(NodeError::Configuration)
    );
    drop(node);
    append(&dir, &OfflineRecord::Config(config).encode());
    assert_eq!(
        Node::open_with_config(&dir, issuer(), config).err(),
        Some(NodeError::Configuration)
    );
}

// C6: a pre-C6 raw event log is rejected; no implicit migration or age reinterpretation.
#[test]
fn legacy_unframed_logs_are_incompatible() {
    let dir = scratch("legacy");
    std::fs::create_dir_all(&dir).unwrap();
    append(&dir, &event(1).encode());
    assert_eq!(
        Node::open(&dir, issuer()).err(),
        Some(NodeError::Undecodable { entry: 0 })
    );
}

// C6: an invalid results event leaves the active frame unchanged; its empty closing still counts.
#[test]
fn rejected_event_does_not_prevent_a_valid_closing_from_counting() {
    let dir = scratch("refusal");
    let mut node = Node::open(&dir, issuer()).unwrap();
    node.begin_cut(cut(0, 10, true)).unwrap();
    let bad = NodeEvent::Results(EpochResults {
        epoch: 1,
        inputs_root: [0; 32],
        records: vec![ResultRecord::ReviewerScore {
            reviewer: Nym([3; 32]),
            score: 0.0,
            inclusion: 0.0,
        }],
    });
    assert_eq!(
        node.submit(bad),
        Err(NodeError::Rejected(Rejection::Results(
            ResultsRejected::BadInclusion { record: 0 }
        )))
    );
    assert_eq!(node.state().results().closing_count(), 0);
    assert_eq!(node.log_len(), 0);
    node.end_cut().unwrap();
    let replayed = Node::open(&dir, issuer()).unwrap();
    assert_eq!(replayed.state(), node.state());
    assert_eq!(replayed.state().results().closing_count(), 1);
}
