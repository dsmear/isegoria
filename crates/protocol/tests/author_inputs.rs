//! C5–C6's author history and its replay (`docs/02` §C.1, `docs/15` C5–C6): one binary outcome
//! per item, aged from its results epoch; open escrows at weight 1; filings checked in turn.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::appeal::{appeal_floor, AuthorHistory, EpochScale};
use protocol::results::{EpochResults, ResultRecord, ResultsRejected, ResultsState};
use scoring::reputation::{author_score, AuthorPrior};

/// A fixture `E_m` of four epochs a month, not a decided value (`docs/15` C6).
fn scale() -> EpochScale {
    EpochScale::new(4.0).expect("a positive E_m")
}

fn nym(i: u8) -> Nym {
    Nym([i; 32])
}

fn item(tag: &str) -> Cid {
    cid(tag.as_bytes())
}

fn apply(
    state: &mut ResultsState,
    epoch: u64,
    records: Vec<ResultRecord>,
) -> Result<(), ResultsRejected> {
    let results = EpochResults {
        epoch,
        inputs_root: [0; 32],
        records,
    };
    state.apply(&results)
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

fn reputation(state: &ResultsState, author: Nym, epoch: u64) -> f64 {
    let history = state.author(&author).cloned().unwrap_or_default();
    history.reputation(epoch, scale(), &AuthorPrior::default())
}

/// `author_score` of `qualities` in CID order of `tags`, every one `age` months old.
fn expected(mut tagged: Vec<(&str, f64)>, age: f64) -> f64 {
    tagged.sort_by_key(|(tag, _)| item(tag));
    let qualities: Vec<f64> = tagged.iter().map(|(_, q)| *q).collect();
    author_score(
        &qualities,
        &vec![age; qualities.len()],
        &AuthorPrior::default(),
    )
}

/// C5/C6: a pass counts 1, a failure 0, from the epoch after their results epoch, aged from it.
#[test]
fn outcomes_are_binary_and_age_from_their_results_epoch() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let records = vec![outcome(nym(1), "a", true), outcome(nym(1), "b", false)];
    apply(&mut state, 3, records).unwrap();
    let history = state.author(&nym(1)).unwrap();
    assert_eq!(history.outcome(&item("a")), Some((true, 3)));
    assert_eq!(history.outcome(&item("b")), Some((false, 3)));
    assert_eq!(
        reputation(&state, nym(1), 3),
        0.4,
        "not counted in its own epoch"
    );
    for (epoch, age) in [(4, 0.25), (11, 2.0), (75, 18.0)] {
        let want = expected(vec![("a", 1.0), ("b", 0.0)], age);
        assert_eq!(reputation(&state, nym(1), epoch).to_bits(), want.to_bits());
    }
}

/// C6: with no escrow, `C_a` moves monotonically toward 2/5 across epochs and never crosses it.
#[test]
fn c_a_moves_toward_the_prior_mean_across_epochs() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let records = vec![
        outcome(nym(1), "a", true),
        outcome(nym(1), "b", true),
        outcome(nym(2), "c", false),
    ];
    apply(&mut state, 1, records).unwrap();
    let (mut high, mut low) = (1.0, 0.0);
    for epoch in [2, 5, 20, 80, 400, 4_000] {
        let (good, bad) = (
            reputation(&state, nym(1), epoch),
            reputation(&state, nym(2), epoch),
        );
        assert!(good < high && good >= 0.4, "{epoch}: {good}");
        assert!(bad > low && bad <= 0.4, "{epoch}: {bad}");
        (high, low) = (good, bad);
    }
}

/// C5: a second record for one item is refused, in the same event or a later one, whole.
#[test]
fn a_repeated_item_is_refused_whole() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let (a, b) = (outcome(nym(1), "a", true), outcome(nym(1), "b", true));
    let twice = vec![a.clone(), outcome(nym(1), "a", false)];
    assert_eq!(
        apply(&mut state, 1, twice),
        Err(ResultsRejected::DuplicateItem { record: 1 })
    );
    assert_eq!(state, ResultsState::with_epoch_scale(scale()));
    apply(&mut state, 1, vec![a.clone(), filed(nym(1), "b")]).unwrap();
    let before = state.clone();
    let cases = [
        (
            vec![a.clone()],
            ResultsRejected::DuplicateItem { record: 0 },
        ),
        (
            vec![filed(nym(1), "a")],
            ResultsRejected::DuplicateItem { record: 0 },
        ),
        (
            vec![b.clone()],
            ResultsRejected::DuplicateItem { record: 0 },
        ),
        (
            vec![filed(nym(1), "b")],
            ResultsRejected::DuplicateItem { record: 0 },
        ),
        (
            vec![settled(nym(1), "a", true)],
            ResultsRejected::NoOpenEscrow { record: 0 },
        ),
        (
            vec![settled(nym(2), "b", true)],
            ResultsRejected::NoOpenEscrow { record: 0 },
        ),
    ];
    for (records, why) in cases {
        assert_eq!(apply(&mut state, 2, records), Err(why));
        assert_eq!(state, before);
    }
    apply(&mut state, 2, vec![settled(nym(1), "b", false)]).unwrap();
    let again = vec![settled(nym(1), "b", true)];
    assert_eq!(
        apply(&mut state, 3, again),
        Err(ResultsRejected::NoOpenEscrow { record: 0 })
    );
    assert_eq!(
        apply(&mut state, 3, vec![b]),
        Err(ResultsRejected::DuplicateItem { record: 0 })
    );
}

/// C5: an appealed item's settlement is its only observation: the escrow becomes it.
#[test]
fn an_appealed_item_counts_once() {
    let mut state = ResultsState::with_epoch_scale(scale());
    apply(&mut state, 1, vec![filed(nym(1), "a")]).unwrap();
    apply(&mut state, 3, vec![settled(nym(1), "a", true)]).unwrap();
    let history = state.author(&nym(1)).unwrap();
    assert_eq!(history.outcomes(), 1);
    assert!(history.open_escrows().is_empty());
    let want = author_score(&[1.0], &[0.25], &AuthorPrior::default());
    assert_eq!(reputation(&state, nym(1), 4).to_bits(), want.to_bits());
}

/// C6: an open escrow keeps weight 1; settled in `e`, it counts through `e`, the outcome after.
#[test]
fn a_settled_escrow_counts_through_its_epoch_and_its_outcome_from_the_next() {
    let mut state = ResultsState::with_epoch_scale(scale());
    apply(&mut state, 1, vec![filed(nym(1), "a")]).unwrap();
    let third = reputation(&state, nym(1), 1);
    assert!((third - 1.0 / 3.0).abs() < 1e-15);
    for epoch in [2, 5, 1_000] {
        assert_eq!(reputation(&state, nym(1), epoch).to_bits(), third.to_bits());
    }
    apply(&mut state, 5, vec![settled(nym(1), "a", true)]).unwrap();
    assert_eq!(reputation(&state, nym(1), 5).to_bits(), third.to_bits());
    let want = author_score(&[1.0], &[0.25], &AuthorPrior::default());
    assert_eq!(reputation(&state, nym(1), 6).to_bits(), want.to_bits());
}

/// C6: ten passes beside one fixed-weight open escrow cross 2/5 at `18 ln 15 ≈ 48.74` months.
#[test]
fn a_fixed_weight_escrow_over_a_long_pending_pilot_crosses_the_floor() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let tags: Vec<String> = (0..10).map(|i| format!("pass {i}")).collect();
    let passes = tags.iter().map(|t| outcome(nym(1), t, true)).collect();
    apply(&mut state, 0, passes).unwrap();
    apply(&mut state, 1, vec![filed(nym(1), "appealed")]).unwrap();
    let history = state.author(&nym(1)).unwrap().clone();
    let p = AuthorPrior::default();
    for (epoch, covers) in [(1, true), (194, true), (196, false)] {
        let months = epoch as f64 / 4.0;
        let mut qualities = vec![1.0; 10];
        let mut ages = vec![months; 10];
        qualities.push(0.0);
        ages.push(0.0);
        let want = author_score(&qualities, &ages, &p);
        assert_eq!(
            history.reputation(epoch, scale(), &p).to_bits(),
            want.to_bits()
        );
        assert_eq!(history.covers_stake(epoch, scale(), &p), covers, "{epoch}");
    }
    assert!(history.reputation(4_000, scale(), &p) - 1.0 / 3.0 < 1e-12);
}

/// C6: replay checks an event's filings in turn and refuses it whole at the first uncovered.
#[test]
fn a_second_filing_in_one_epoch_meets_the_first_and_replay_refuses_the_event_whole() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let both = vec![filed(nym(1), "x"), filed(nym(1), "y")];
    let refused = Err(ResultsRejected::AppealNotCovered { record: 1 });
    assert_eq!(apply(&mut state, 1, both), refused);
    assert_eq!(
        state,
        ResultsState::with_epoch_scale(scale()),
        "the first is not kept"
    );
    apply(&mut state, 1, vec![filed(nym(1), "x")]).unwrap();
    assert!((reputation(&state, nym(1), 1) - 1.0 / 3.0).abs() < 1e-15);
    // Checked against the frozen history both would pass and leave 2/7.
    let frozen = author_score(&[0.0, 0.0], &[0.0, 0.0], &AuthorPrior::default());
    assert!((frozen - 2.0 / 7.0).abs() < 1e-15);
}

/// C6: an admissible event replays to one state whatever the order of its records.
#[test]
fn replay_does_not_depend_on_the_order_of_an_event_s_records() {
    let mut base = ResultsState::with_epoch_scale(scale());
    let passes = ["p", "q", "r"]
        .iter()
        .map(|t| outcome(nym(1), t, true))
        .collect();
    apply(&mut base, 0, passes).unwrap();
    apply(&mut base, 1, vec![filed(nym(1), "x")]).unwrap();
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

/// C6: the quota's `C_a` for `e` ignores filings in `e`; one settled in `e − 1` counts as outcome.
#[test]
fn the_quota_snapshot_ignores_the_epoch_s_filings() {
    let mut state = ResultsState::with_epoch_scale(scale());
    apply(
        &mut state,
        1,
        vec![outcome(nym(1), "a", true), filed(nym(1), "b")],
    )
    .unwrap();
    let quota = |s: &ResultsState, e| s.quota_reputation(&nym(1), e).unwrap();
    assert_eq!(quota(&state, 1), 0.4, "a fresh author before epoch 1");
    let at_two = quota(&state, 2);
    apply(
        &mut state,
        2,
        vec![filed(nym(1), "c"), settled(nym(1), "b", true)],
    )
    .unwrap();
    assert_eq!(quota(&state, 2).to_bits(), at_two.to_bits());
    let p = AuthorPrior::default();
    let mut want = [("a", 1.0), ("b", 1.0)];
    want.sort_by_key(|(t, _)| item(t));
    let (qualities, ages): (Vec<f64>, Vec<f64>) = want
        .iter()
        .map(|(t, q)| (*q, if *t == "a" { 0.5 } else { 0.25 }))
        .chain([(0.0, 0.0)])
        .unzip();
    assert_eq!(
        quota(&state, 3).to_bits(),
        author_score(&qualities, &ages, &p).to_bits()
    );
    assert_eq!(state.quota_reputation(&nym(9), 3), Some(0.4));
}

/// C6: with no `E_m` configured outcomes and settlements replay, a filing is refused.
#[test]
fn a_filing_needs_a_configured_scale() {
    let mut state = ResultsState::default();
    apply(&mut state, 1, vec![outcome(nym(1), "a", true)]).unwrap();
    let refused = Err(ResultsRejected::NoEpochScale { record: 0 });
    assert_eq!(apply(&mut state, 2, vec![filed(nym(1), "b")]), refused);
    assert_eq!(state.quota_reputation(&nym(1), 2), None);
}

/// C5: a filing whose request came after its window is refused whole, its coverage untouched.
#[test]
fn a_filing_outside_its_window_is_refused_whole() {
    let mut state = ResultsState::with_epoch_scale(scale());
    let late = ResultRecord::AppealStarted {
        author: nym(1),
        item: item("x"),
        within_window: false,
    };
    let refused = Err(ResultsRejected::AppealWindowClosed { record: 1 });
    assert_eq!(
        apply(&mut state, 1, vec![filed(nym(2), "y"), late]),
        refused
    );
    assert_eq!(state, ResultsState::with_epoch_scale(scale()));
    apply(&mut state, 1, vec![filed(nym(1), "y")]).unwrap();
}

/// C6: an author record from an epoch before one that author's history recorded is refused.
#[test]
fn an_author_record_from_an_earlier_epoch_is_refused() {
    let mut state = ResultsState::with_epoch_scale(scale());
    apply(&mut state, 5, vec![outcome(nym(1), "a", true)]).unwrap();
    let late = vec![outcome(nym(2), "b", true), outcome(nym(1), "c", true)];
    assert_eq!(
        apply(&mut state, 3, late),
        Err(ResultsRejected::AuthorEpochBehind { record: 1 })
    );
    apply(&mut state, 3, vec![outcome(nym(2), "b", true)]).unwrap();
}

/// C6: one failure gives about 1/3 an epoch later; 720 months on, binary64 rounds `C_a` to 0.4.
#[test]
fn a_zero_at_720_months_rounds_to_the_floor_and_covers_the_stake() {
    let p = AuthorPrior::default();
    let mut history = AuthorHistory::new();
    history.conclude(item("a"), false, 0).unwrap();
    assert!((history.reputation(1, scale(), &p) - 1.0 / 3.0).abs() < 1e-3);
    assert!(!history.covers_stake(1, scale(), &p));
    let old = 720 * 4;
    let floor = appeal_floor(&p);
    assert_eq!(
        history.reputation(old, scale(), &p).to_bits(),
        floor.to_bits()
    );
    assert!(history.covers_stake(old, scale(), &p));
    let no_prior = AuthorPrior {
        alpha0: 0.0,
        beta0: 0.0,
        ..p
    };
    // The weight is positive, not underflowed: the index is `w / w`.
    assert_eq!(author_score(&[1.0], &[720.0], &no_prior), 1.0);
}

/// C6: `E_m` must be finite and positive; an age is the epoch difference over it.
#[test]
fn the_epoch_scale_is_explicit_and_positive() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(EpochScale::new(bad), None);
    }
    assert_eq!(scale().epochs_per_month(), 4.0);
    assert_eq!(scale().age_months(3, 11), 2.0);
}
