//! C5–C6's current author inputs (`docs/15` C5–C6): replay appends each quality record as
//! written, naming no item, and no epoch ages a record or an escrow.

use identity::nym::Nym;
use protocol::appeal::{appeal_floor, AuthorHistory};
use protocol::results::{EpochResults, ResultRecord, ResultsRejected, ResultsState};
use scoring::reputation::{author_score, AuthorPrior};

fn nym(i: u8) -> Nym {
    Nym([i; 32])
}

fn apply(state: &mut ResultsState, epoch: u64, records: Vec<ResultRecord>) {
    let results = EpochResults {
        epoch,
        inputs_root: [0; 32],
        records,
    };
    state.apply(&results).expect("valid records");
}

fn quality(author: Nym, quality: f64, age_months: f64) -> ResultRecord {
    ResultRecord::AuthorQuality {
        author,
        quality,
        age_months,
    }
}

fn reputation(state: &ResultsState, author: Nym) -> f64 {
    let history = state.author(&author).expect("a history");
    history.reputation(&AuthorPrior::default())
}

/// C6: a record keeps its written age whatever epoch it lands in and however many follow.
#[test]
fn a_record_keeps_its_written_age_across_epochs() {
    let mut early = ResultsState::default();
    apply(&mut early, 1, vec![quality(nym(1), 1.0, 6.0)]);
    let at_once = reputation(&early, nym(1));
    for epoch in 2..=60 {
        apply(&mut early, epoch, vec![]);
    }
    let mut late = ResultsState::default();
    apply(&mut late, 1_000, vec![quality(nym(1), 1.0, 6.0)]);
    assert_eq!(early.author(&nym(1)), late.author(&nym(1)));
    assert_eq!(reputation(&early, nym(1)).to_bits(), at_once.to_bits());
    let want = author_score(&[1.0], &[6.0], &AuthorPrior::default());
    assert_eq!(at_once.to_bits(), want.to_bits());
}

/// C6: an escrow settled forty epochs after filing counts at age 0, promoted or failed.
#[test]
fn an_escrow_settled_epochs_after_filing_counts_at_age_zero() {
    let (promoted, failed) = (nym(1), nym(2));
    let mut state = ResultsState::default();
    let mut filed = Vec::new();
    for author in [promoted, failed] {
        filed.push(quality(author, 1.0, 18.0));
        filed.push(ResultRecord::AppealFiled { author });
    }
    apply(&mut state, 1, filed);
    for epoch in 2..=40 {
        apply(&mut state, epoch, vec![]);
    }
    let settle = |author, promoted| ResultRecord::AppealSettled {
        author,
        escrow: 1,
        promoted,
    };
    apply(
        &mut state,
        41,
        vec![settle(promoted, Some(0.8)), settle(failed, None)],
    );
    let p = AuthorPrior::default();
    let want = author_score(&[1.0, 0.8], &[18.0, 0.0], &p);
    assert_eq!(reputation(&state, promoted).to_bits(), want.to_bits());
    let want = author_score(&[1.0, 0.0], &[18.0, 0.0], &p);
    assert_eq!(reputation(&state, failed).to_bits(), want.to_bits());
}

/// C5: a quality record names no item, so a repeat counts again and a later value replaces nothing.
#[test]
fn quality_records_name_no_item_so_a_repeat_counts_again() {
    let mut state = ResultsState::default();
    let record = quality(nym(1), 0.6, 0.0);
    apply(&mut state, 1, vec![record.clone(), record.clone()]);
    apply(&mut state, 2, vec![record, quality(nym(1), 0.1, 0.0)]);
    let history = state.author(&nym(1)).expect("a history");
    assert_eq!(history.qualities(), &[0.6, 0.6, 0.6, 0.1]);
    let want = author_score(&[0.6, 0.6, 0.6, 0.1], &[0.0; 4], &AuthorPrior::default());
    assert_eq!(reputation(&state, nym(1)).to_bits(), want.to_bits());
}

/// C5: replay clamps a finite quality outside [0, 1] and a negative age instead of refusing them.
#[test]
fn an_out_of_range_quality_or_age_is_clamped_not_refused() {
    let mut state = ResultsState::default();
    let records = vec![quality(nym(1), 1.7, -3.0), quality(nym(1), -0.2, 0.0)];
    apply(&mut state, 1, records);
    let mut direct = AuthorHistory::new();
    direct.record(1.0, 0.0);
    direct.record(0.0, 0.0);
    assert_eq!(state.author(&nym(1)), Some(&direct));
}

/// C6: replay checks filings in order, so a fresh author's second filing in one epoch is refused.
#[test]
fn a_second_filing_in_one_epoch_meets_the_first_escrow() {
    let mut state = ResultsState::default();
    let filed = ResultRecord::AppealFiled { author: nym(1) };
    let twice = EpochResults {
        epoch: 1,
        inputs_root: [0; 32],
        records: vec![filed.clone(), filed.clone()],
    };
    let refused = Err(ResultsRejected::AppealNotCovered { record: 1 });
    assert_eq!(state.apply(&twice), refused);
    apply(&mut state, 1, vec![filed]);
    assert!((reputation(&state, nym(1)) - 1.0 / 3.0).abs() < 1e-12);
}

/// C6: one `q = 0` gives 1/3 at age 0; at 720 months binary64 rounds `C_a` to the floor 0.4.
#[test]
fn a_zero_at_720_months_rounds_to_the_floor_and_covers_the_stake() {
    let p = AuthorPrior::default();
    let mut fresh = AuthorHistory::new();
    fresh.record(0.0, 0.0);
    assert!((fresh.reputation(&p) - 1.0 / 3.0).abs() < 1e-12);
    assert!(!fresh.covers_stake(&p));
    let mut old = AuthorHistory::new();
    old.record(0.0, 720.0);
    assert_eq!(old.reputation(&p).to_bits(), appeal_floor(&p).to_bits());
    assert!(old.covers_stake(&p));
    let no_prior = AuthorPrior {
        alpha0: 0.0,
        beta0: 0.0,
        ..p
    };
    // The weight is positive, not underflowed: the index is `w / w`.
    assert_eq!(author_score(&[1.0], &[720.0], &no_prior), 1.0);
}
