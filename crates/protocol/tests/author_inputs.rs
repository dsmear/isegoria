//! Author observations, immutable closing ages and atomic filing batches (`docs/15` C5–C6).

use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::appeal::{appeal_floor, AuthorHistory};
use protocol::author_time::{AuthorConfig, AuthorPoint, ContextError, CutContext};
use protocol::results::{EpochResults, ResultRecord, ResultsRejected, ResultsState};
use scoring::reputation::{author_score_closings, AuthorPrior};

fn nym(i: u8) -> Nym {
    Nym([i; 32])
}
fn item(tag: &str) -> Cid {
    cid(tag.as_bytes())
}
fn point(epoch: u64, closings: u64) -> AuthorPoint {
    AuthorPoint { epoch, closings }
}
fn begin(state: &mut ResultsState, number: u64, epoch: u64, closes: bool) {
    state
        .begin_cut(CutContext {
            number,
            epoch,
            closes,
        })
        .unwrap();
}
fn apply(
    state: &mut ResultsState,
    data_epoch: u64,
    records: Vec<ResultRecord>,
) -> Result<(), ResultsRejected> {
    state.apply(&EpochResults {
        epoch: data_epoch,
        inputs_root: [data_epoch as u8; 32],
        records,
    })
}
fn outcome(author: Nym, tag: &str, passed: bool) -> ResultRecord {
    ResultRecord::AuthorOutcome {
        author,
        item: item(tag),
        passed,
    }
}
fn filed(author: Nym, tag: &str) -> ResultRecord {
    ResultRecord::AppealStarted {
        author,
        item: item(tag),
        within_window: true,
    }
}
fn settled(author: Nym, tag: &str, passed: bool) -> ResultRecord {
    ResultRecord::AppealSettled {
        author,
        item: item(tag),
        passed,
    }
}
fn reputation(state: &ResultsState, author: Nym) -> f64 {
    state
        .author(&author)
        .cloned()
        .unwrap_or_default()
        .reputation(
            state.author_point().unwrap(),
            state.author_config(),
            &AuthorPrior::default(),
        )
}
fn expected(q: &[f64], ages: &[u64]) -> f64 {
    author_score_closings(q, ages, 45, &AuthorPrior::default())
}

// C6: a skip and a new unclosed epoch expose an outcome without aging it.
#[test]
fn skipped_identifiers_do_not_age_outcomes() {
    let mut state = ResultsState::default();
    begin(&mut state, 0, 10, false);
    apply(&mut state, 900, vec![outcome(nym(1), "pass", true)]).unwrap();
    assert_eq!(
        state.author(&nym(1)).unwrap().outcome(&item("pass")),
        Some((true, point(10, 0)))
    );
    assert_eq!(reputation(&state, nym(1)), 0.4);
    state.end_cut().unwrap();
    begin(&mut state, 1, 1_000_000, false);
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[1.0], &[0]).to_bits()
    );
    assert_eq!(state.closing_count(), 0);
}

// C6: empty closings count; multiple cuts in one epoch retain the same k until End.
#[test]
fn empty_closings_age_and_the_closing_cut_uses_the_epoch_start_count() {
    let mut state = ResultsState::default();
    begin(&mut state, 0, 10, true);
    apply(&mut state, 2, vec![outcome(nym(1), "first", true)]).unwrap();
    assert_eq!(state.author_point(), Some(point(10, 0)));
    state.end_cut().unwrap();
    begin(&mut state, 1, 20, false);
    let frozen = reputation(&state, nym(1));
    state.end_cut().unwrap();
    begin(&mut state, 2, 20, true);
    assert_eq!(state.author_point(), Some(point(20, 1)));
    assert_eq!(reputation(&state, nym(1)).to_bits(), frozen.to_bits());
    apply(
        &mut state,
        1,
        vec![outcome(nym(2), "closing-record", false)],
    )
    .unwrap();
    assert_eq!(
        state
            .author(&nym(2))
            .unwrap()
            .outcome(&item("closing-record")),
        Some((false, point(20, 1)))
    );
    state.end_cut().unwrap();
    begin(&mut state, 3, 30, true);
    state.end_cut().unwrap();
    begin(&mut state, 4, 40, false);
    assert_eq!(state.author_point(), Some(point(40, 3)));
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[1.0], &[3]).to_bits()
    );
    assert_eq!(
        reputation(&state, nym(2)).to_bits(),
        expected(&[0.0], &[2]).to_bits()
    );
}

// C6: late data epochs set neither evaluation age nor an outcome's recording origin.
#[test]
fn a_late_data_epoch_has_the_same_origin_and_weight() {
    let mut a = ResultsState::default();
    let mut b = ResultsState::default();
    for state in [&mut a, &mut b] {
        begin(state, 0, 100, true);
        state.end_cut().unwrap();
        begin(state, 1, 500, false);
    }
    apply(&mut a, 2, vec![outcome(nym(1), "late", true)]).unwrap();
    apply(&mut b, 400, vec![outcome(nym(1), "late", true)]).unwrap();
    assert_eq!(a.author(&nym(1)), b.author(&nym(1)));
    assert_eq!(
        a.author(&nym(1)).unwrap().outcome(&item("late")),
        Some((true, point(500, 1)))
    );
    for state in [&mut a, &mut b] {
        state.end_cut().unwrap();
        begin(state, 2, 900, false);
    }
    assert_eq!(
        reputation(&a, nym(1)).to_bits(),
        reputation(&b, nym(1)).to_bits()
    );
    assert_eq!(
        reputation(&a, nym(1)).to_bits(),
        expected(&[1.0], &[0]).to_bits()
    );
    apply(&mut a, 1, vec![outcome(nym(1), "still-older-data", false)]).unwrap();
    assert_eq!(
        a.author(&nym(1))
            .unwrap()
            .outcome(&item("still-older-data")),
        Some((false, point(900, 1)))
    );
    assert_eq!(
        apply(&mut a, 1, vec![]),
        Err(ResultsRejected::EpochRecorded)
    );
}

// C5/C6: escrow remains weight one through settlement E; its outcome starts after E.
#[test]
fn settlement_and_quota_preserve_the_epoch_boundary() {
    let mut state = ResultsState::default();
    begin(&mut state, 0, 1, true);
    apply(
        &mut state,
        8,
        vec![outcome(nym(1), "prior-pass", true), filed(nym(1), "appeal")],
    )
    .unwrap();
    assert_eq!(state.quota_reputation(&nym(1)), Some(0.4));
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[0.0], &[0]).to_bits()
    );
    state.end_cut().unwrap();
    begin(&mut state, 1, 2, true);
    let before = state.quota_reputation(&nym(1)).unwrap();
    apply(
        &mut state,
        0,
        vec![settled(nym(1), "appeal", true), filed(nym(1), "second")],
    )
    .unwrap();
    assert_eq!(
        state.quota_reputation(&nym(1)).unwrap().to_bits(),
        before.to_bits()
    );
    let h = state.author(&nym(1)).unwrap();
    assert_eq!(h.outcome(&item("appeal")), Some((true, point(2, 1))));
    assert_eq!(h.outcomes(), 2);
    assert_eq!(h.open_escrows(), vec![item("second")]);
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[1.0, 0.0, 0.0], &[1, 0, 0]).to_bits()
    );
    state.end_cut().unwrap();
    begin(&mut state, 2, 100, false);
    let mut observations = [(item("prior-pass"), 2), (item("appeal"), 1)];
    observations.sort_by_key(|(cid, _)| *cid);
    let ages = [observations[0].1, observations[1].1, 0];
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[1.0, 1.0, 0.0], &ages).to_bits()
    );
    assert_eq!(
        state.quota_reputation(&nym(1)).unwrap().to_bits(),
        reputation(&state, nym(1)).to_bits()
    );
}

// C6: at most one accepted filing event per recording E, including other authors and cuts.
#[test]
fn a_second_filing_batch_is_refused_whole_but_nonfiling_results_remain_legal() {
    let mut state = ResultsState::default();
    begin(&mut state, 0, 5, false);
    apply(
        &mut state,
        100,
        vec![filed(nym(1), "a"), filed(nym(2), "b")],
    )
    .unwrap();
    let before = state.clone();
    assert_eq!(
        apply(
            &mut state,
            101,
            vec![outcome(nym(3), "c", true), filed(nym(3), "d")]
        ),
        Err(ResultsRejected::FilingBatchRecorded)
    );
    assert_eq!(state, before);
    apply(&mut state, 101, vec![outcome(nym(3), "c", true)]).unwrap();
    state.end_cut().unwrap();
    begin(&mut state, 1, 5, true);
    let before = state.clone();
    assert_eq!(
        apply(&mut state, 102, vec![filed(nym(4), "e")]),
        Err(ResultsRejected::FilingBatchRecorded)
    );
    assert_eq!(state, before);
    state.end_cut().unwrap();
    begin(&mut state, 2, 6, false);
    apply(&mut state, 102, vec![filed(nym(4), "e")]).unwrap();
}

// C5/C6: failed batches consume no slot, origin, root or escrow, in either record order.
#[test]
fn invalid_batches_leave_no_trace_and_do_not_consume_the_slot() {
    for records in [
        vec![filed(nym(1), "a"), filed(nym(1), "b")],
        vec![
            filed(nym(2), "a"),
            ResultRecord::AppealStarted {
                author: nym(3),
                item: item("b"),
                within_window: false,
            },
        ],
        vec![
            ResultRecord::AppealStarted {
                author: nym(3),
                item: item("b"),
                within_window: false,
            },
            filed(nym(2), "a"),
        ],
        vec![
            outcome(nym(1), "pass", true),
            filed(nym(2), "a"),
            ResultRecord::ReviewerScore {
                reviewer: nym(5),
                score: 0.0,
                inclusion: 0.0,
            },
        ],
    ] {
        let mut state = ResultsState::default();
        begin(&mut state, 0, 10, true);
        let before = state.clone();
        assert!(apply(&mut state, 1, records).is_err());
        assert_eq!(state, before);
        assert!(!state.filing_batch_at(10));
        apply(&mut state, 1, vec![filed(nym(2), "valid")]).unwrap();
        assert!(state.filing_batch_at(10));
    }
}

// C5: binary observations are item-keyed; duplicate or unescrowed settlement is atomic.
#[test]
fn repeated_items_and_unescrowed_settlements_are_refused() {
    let mut state = ResultsState::default();
    begin(&mut state, 0, 1, false);
    let before = state.clone();
    assert_eq!(
        apply(
            &mut state,
            7,
            vec![outcome(nym(1), "a", true), outcome(nym(1), "a", false)]
        ),
        Err(ResultsRejected::DuplicateItem { record: 1 })
    );
    assert_eq!(state, before);
    assert_eq!(
        apply(&mut state, 7, vec![settled(nym(1), "a", true)]),
        Err(ResultsRejected::NoOpenEscrow { record: 0 })
    );
    assert_eq!(state, before);
    apply(&mut state, 7, vec![outcome(nym(1), "a", false)]).unwrap();
    state.end_cut().unwrap();
    begin(&mut state, 1, 2, false);
    assert_eq!(
        reputation(&state, nym(1)).to_bits(),
        expected(&[0.0], &[0]).to_bits()
    );
}

// C6: authoritative context is monotone and malformed boundaries leave it unchanged.
#[test]
fn context_progression_is_checked_atomically() {
    let mut state = ResultsState::default();
    assert_eq!(state.end_cut(), Err(ContextError::NotOpen));
    let bad = CutContext {
        number: 1,
        epoch: 9,
        closes: true,
    };
    assert_eq!(state.begin_cut(bad), Err(ContextError::Number));
    begin(&mut state, 0, 9, true);
    let before = state.clone();
    assert_eq!(state.begin_cut(bad), Err(ContextError::AlreadyOpen));
    assert_eq!(state, before);
    state.end_cut().unwrap();
    let before = state.clone();
    assert_eq!(state.begin_cut(bad), Err(ContextError::Epoch));
    assert_eq!(state, before);
    begin(&mut state, 1, 90, false);
    assert_eq!(state.author_point(), Some(point(90, 1)));
}

// C6: no context means no author record; rejected records cannot invent an origin.
#[test]
fn author_records_require_an_applying_context() {
    let mut state = ResultsState::default();
    for record in [
        outcome(nym(1), "a", true),
        filed(nym(1), "b"),
        settled(nym(1), "b", false),
    ] {
        assert_eq!(
            apply(&mut state, 1, vec![record]),
            Err(ResultsRejected::MissingContext { record: 0 })
        );
        assert_eq!(state, ResultsState::default());
    }
    assert_eq!(state.quota_reputation(&nym(1)), None);
}

// C6: H is a positive count, default 45; one half-life halves a weight, not C_a.
#[test]
fn the_adopted_half_life_and_fixed_escrow_are_distinct() {
    assert_eq!(AuthorConfig::new(0), None);
    assert_eq!(AuthorConfig::default().half_life_closings(), 45);
    let mut history = AuthorHistory::new();
    history.conclude(item("pass"), true, point(1, 0)).unwrap();
    let at = point(2, 45);
    let config = AuthorConfig::default();
    let p = AuthorPrior::default();
    assert!((history.reputation(at, config, &p) - 2.5 / 5.5).abs() < 1e-15);
    history.file_appeal(item("escrow"), at, config, &p).unwrap();
    let much_later = point(3, 100_000);
    assert_eq!(
        history.reputation(much_later, config, &p).to_bits(),
        expected(&[1.0, 0.0], &[100_000, 0]).to_bits()
    );
    assert!(history.reputation(much_later, config, &p) < appeal_floor(&p));
}

// C6: binary64 rounding can still cover an old failure; it has not become a positive outcome.
#[test]
fn an_old_failure_can_round_to_the_floor() {
    let p = AuthorPrior::default();
    let config = AuthorConfig::default();
    let mut h = AuthorHistory::new();
    h.conclude(item("failure"), false, point(1, 0)).unwrap();
    assert!(!h.covers_stake(point(2, 1), config, &p));
    assert_eq!(
        h.reputation(point(3, 2_400), config, &p).to_bits(),
        appeal_floor(&p).to_bits()
    );
    let no_prior = AuthorPrior {
        alpha0: 0.0,
        beta0: 0.0,
        ..p
    };
    assert_eq!(author_score_closings(&[1.0], &[2_400], 45, &no_prior), 1.0);
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::new();
    for p in permutations(n - 1) {
        for at in 0..=p.len() {
            let mut q = p.clone();
            q.insert(at, n - 1);
            out.push(q);
        }
    }
    out
}

// C6: with no escrow, `C_a` moves monotonically toward 2/5 over closings and never crosses it.
#[test]
fn c_a_moves_toward_the_prior_mean_across_closings() {
    let p = AuthorPrior::default();
    let config = AuthorConfig::default();
    let (mut good, mut bad) = (AuthorHistory::new(), AuthorHistory::new());
    good.conclude(item("a"), true, point(1, 0)).unwrap();
    good.conclude(item("b"), true, point(1, 0)).unwrap();
    bad.conclude(item("c"), false, point(1, 0)).unwrap();
    let (mut high, mut low) = (1.0, 0.0);
    for closings in [1, 5, 20, 80, 400, 4_000] {
        let at = point(2, closings);
        let g = good.reputation(at, config, &p);
        let b = bad.reputation(at, config, &p);
        assert!(g < high && g >= 0.4, "{closings}: {g}");
        assert!(b > low && b <= 0.4, "{closings}: {b}");
        (high, low) = (g, b);
    }
}

// C6: ten passes beside one fixed-weight open escrow cross 2/5 past `45 log₂ 15 ≈ 175.8` closings.
#[test]
fn a_fixed_weight_escrow_over_a_long_pending_pilot_crosses_the_floor() {
    let p = AuthorPrior::default();
    let config = AuthorConfig::default();
    let mut history = AuthorHistory::new();
    for i in 0..10 {
        let tag = format!("pass {i}");
        history.conclude(item(&tag), true, point(1, 0)).unwrap();
    }
    history
        .file_appeal(item("appealed"), point(2, 1), config, &p)
        .unwrap();
    for (closings, covers) in [(1, true), (175, true), (176, false)] {
        let (mut q, mut ages) = (vec![1.0; 10], vec![closings; 10]);
        q.push(0.0);
        ages.push(0);
        let at = point(3, closings);
        assert_eq!(
            history.reputation(at, config, &p).to_bits(),
            expected(&q, &ages).to_bits()
        );
        assert_eq!(history.covers_stake(at, config, &p), covers, "{closings}");
    }
    assert!(history.reputation(point(3, 100_000), config, &p) - 1.0 / 3.0 < 1e-12);
}

// C6: an admissible event replays to one state whatever the order of its records.
#[test]
fn replay_does_not_depend_on_the_order_of_an_event_s_records() {
    let mut base = ResultsState::default();
    begin(&mut base, 0, 1, true);
    let passes = ["p", "q", "r"]
        .iter()
        .map(|t| outcome(nym(1), t, true))
        .collect();
    apply(&mut base, 0, passes).unwrap();
    base.end_cut().unwrap();
    begin(&mut base, 1, 2, true);
    apply(&mut base, 1, vec![filed(nym(1), "x")]).unwrap();
    base.end_cut().unwrap();
    begin(&mut base, 2, 3, false);
    let records = vec![
        settled(nym(1), "x", true),
        outcome(nym(1), "y", false),
        filed(nym(1), "z"),
        filed(nym(1), "w"),
        filed(nym(2), "v"),
    ];
    let mut reference = None;
    for perm in permutations(records.len()) {
        let mut state = base.clone();
        let ordered = perm.iter().map(|&i| records[i].clone()).collect();
        apply(&mut state, 2, ordered).expect("every filing is covered in any order");
        let first = reference.get_or_insert_with(|| state.clone());
        assert_eq!(&state, first, "{perm:?}");
    }
    assert_eq!(permutations(records.len()).len(), 120);
}
