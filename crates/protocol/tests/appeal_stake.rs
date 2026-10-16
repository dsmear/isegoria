//! The appeal stake is a pseudo-observation inside the author's average (`docs/01` D27,
//! `docs/08` REPUTATION-007, T61): filing escrows a zero that the item's pilot outcome
//! settles; an author below the floor cannot file.

use identity::nym::Nym;
use network::cid::{cid, Cid};
use protocol::appeal::{
    appeal_floor, AuthorError, AuthorHistory, EpochScale, InsufficientReputation,
};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, Invalid, RejectReason, State};
use protocol::orchestrator::{
    review_round, run_item, settle_appeal, ItemVerdicts, Judgment, Unsettled,
};
use protocol::pilot::Screening;
use protocol::revalidation::Recheck;
use scoring::reputation::{author_score, AuthorPrior};

/// `run_item` for an item that is not in the band: no extra round, no re-decision.
fn run(reviewed: State, v: &ItemVerdicts) -> Result<State, Invalid> {
    run_item(reviewed, v, None, |_| unreachable!("no band item here"))
}

fn prior() -> AuthorPrior {
    AuthorPrior::default()
}

/// A fixture `E_m`, not a decided value (`docs/15` C6).
fn scale() -> EpochScale {
    EpochScale::new(4.0).unwrap()
}

fn reputation(author: &AuthorHistory, epoch: u64) -> f64 {
    author.reputation(epoch, scale(), &prior())
}

fn file(author: &mut AuthorHistory, item: Cid, epoch: u64) -> Result<(), AuthorError> {
    author.file_appeal(item, epoch, scale(), &prior())
}

// ------------------------------ the escrow ------------------------------

#[test]
fn filing_escrows_a_zero_and_lowers_the_reputation_at_once() {
    // A fresh author sits at the prior mean, 2 / (2 + 3) = 0.4: exactly the floor.
    let mut author = AuthorHistory::new();
    assert!((reputation(&author, 1) - 0.4).abs() < 1e-12);
    assert!((appeal_floor(&prior()) - 0.4).abs() < 1e-12);
    assert!(author.covers_stake(1, scale(), &prior()));
    file(&mut author, cid(b"a"), 1).unwrap();
    assert_eq!(author.open_escrows(), vec![cid(b"a")]);
    // (2 + 0) / (2 + 3 + 1), at once and for every later epoch while open.
    assert!((reputation(&author, 1) - 1.0 / 3.0).abs() < 1e-12);
    assert!((reputation(&author, 90) - 1.0 / 3.0).abs() < 1e-12);

    author.settle(cid(b"a"), false, 1).unwrap();
    assert!(author.open_escrows().is_empty());
    assert_eq!(author.outcome(&cid(b"a")), Some((false, 1)));
    assert!((reputation(&author, 1) - 1.0 / 3.0).abs() < 1e-12);
    let aged = author_score(&[0.0], &[0.25], &prior());
    assert_eq!(reputation(&author, 2).to_bits(), aged.to_bits());
}

#[test]
fn a_passed_appeal_replaces_the_escrow_with_a_one() {
    let mut author = AuthorHistory::new();
    file(&mut author, cid(b"a"), 1).unwrap();
    author.settle(cid(b"a"), true, 2).unwrap();
    // The escrow counts through the settling epoch, the outcome from the next.
    assert!((reputation(&author, 2) - 1.0 / 3.0).abs() < 1e-12);
    let after = reputation(&author, 3);
    assert_eq!(
        after.to_bits(),
        author_score(&[1.0], &[0.25], &prior()).to_bits()
    );
    assert!(after > 0.4);
    assert_eq!(author.outcomes(), 1);
}

#[test]
fn an_author_below_the_floor_cannot_file() {
    let mut author = AuthorHistory::new();
    file(&mut author, cid(b"a"), 1).unwrap();
    author.settle(cid(b"a"), false, 1).unwrap();
    // About 1/3 < 0.4: a failed appeal costs the next one …
    assert!(!author.covers_stake(2, scale(), &prior()));
    let before = author.clone();
    let Err(AuthorError::Uncovered(InsufficientReputation { reputation, floor })) =
        file(&mut author, cid(b"b"), 2)
    else {
        panic!("an author below the floor filed an appeal");
    };
    assert!((reputation - 1.0 / 3.0).abs() < 1e-3);
    assert!((floor - 0.4).abs() < 1e-12);
    assert_eq!(author, before, "a refused filing escrows nothing");
    // … until the evidence restores the average: one passed item, about (2 + 0 + 1) / 7 ≥ 0.4.
    author.conclude(cid(b"c"), true, 2).unwrap();
    assert!(author.covers_stake(3, scale(), &prior()));
    assert!(file(&mut author, cid(b"b"), 3).is_ok());
}

/// A failed appeal bars the next one only while `C_a` stays below the floor (`docs/15` C5).
#[test]
fn a_failed_appeal_bars_the_next_only_below_the_floor() {
    let mut author = AuthorHistory::new();
    for i in 0..10u8 {
        author.conclude(cid(&[i]), true, 1).unwrap();
    }
    file(&mut author, cid(b"a"), 2).unwrap();
    author.settle(cid(b"a"), false, 2).unwrap();
    // About (2 + 10) / (5 + 11) = 0.75, above the floor 0.4.
    assert!((reputation(&author, 3) - 0.75).abs() < 0.01);
    assert!(file(&mut author, cid(b"b"), 3).is_ok());
}

// ------------------------------ the orchestrator ------------------------------

fn item() -> Cid {
    cid(b"item")
}

fn panel() -> Vec<Nym> {
    (1..=9).map(|i| Nym([i; 32])).collect()
}

fn reviewed() -> State {
    let admitted = step(
        deposit(true, true, true, true).unwrap(),
        Event::Admit {
            seed_from_beacon: true,
        },
    )
    .unwrap();
    let judgments: Vec<Judgment> = panel()
        .into_iter()
        .map(|nym| Judgment {
            nym,
            prob: 0.7,
            nonce: [nym.0[0]; 32],
        })
        .collect();
    review_round(admitted, item(), panel(), &judgments).unwrap()
}

/// An appealed polarization rejection that the evidence would vindicate.
fn appealed() -> ItemVerdicts {
    ItemVerdicts {
        gate: GateOutcome::AppealEligible,
        appealed: true,
        appeal_within_window: true,
        author_reputation: 0.6,
        appeal_floor: 0.4,
        enough_respondents: true,
        screen: Screening::Pass,
        dif: Recheck::NoDif,
        source_verified: false,
        pilot2_batch_size: 8,
        explored: false,
    }
}

#[test]
fn run_item_derives_the_appeal_checks_from_the_verdicts() {
    assert_eq!(run(reviewed(), &appealed()).unwrap(), State::ActivePool);
    assert_eq!(
        run(
            reviewed(),
            &ItemVerdicts {
                appeal_within_window: false,
                ..appealed()
            }
        ),
        Err(Invalid::AppealWindowClosed)
    );
    assert_eq!(
        run(
            reviewed(),
            &ItemVerdicts {
                author_reputation: 0.3,
                ..appealed()
            }
        ),
        Err(Invalid::InsufficientReputation)
    );
    // Exactly at the floor covers the stake.
    assert_eq!(
        run(
            reviewed(),
            &ItemVerdicts {
                author_reputation: 0.4,
                ..appealed()
            }
        )
        .unwrap(),
        State::ActivePool
    );
}

/// A history with one passed item, concluded in epoch 1.
fn established() -> AuthorHistory {
    let mut author = AuthorHistory::new();
    author.conclude(cid(b"earlier"), true, 1).unwrap();
    author
}

#[test]
fn the_terminal_state_settles_the_escrow() {
    // Promoted.
    let mut author = established();
    let before = reputation(&author, 2);
    file(&mut author, item(), 2).unwrap();
    let terminal = run(reviewed(), &appealed()).unwrap();
    settle_appeal(&mut author, item(), &terminal, 3).unwrap();
    assert_eq!(author.outcome(&item()), Some((true, 3)));
    assert!(reputation(&author, 4) > before);

    // Failed at the DIF stage.
    let mut author = established();
    let before = reputation(&author, 2);
    file(&mut author, item(), 2).unwrap();
    let terminal = run(
        reviewed(),
        &ItemVerdicts {
            dif: Recheck::Dif,
            source_verified: false,
            ..appealed()
        },
    )
    .unwrap();
    assert_eq!(terminal, State::Rejected(RejectReason::Dif));
    settle_appeal(&mut author, item(), &terminal, 3).unwrap();
    assert_eq!(author.outcome(&item()), Some((false, 3)));
    assert!(reputation(&author, 4) < before);

    // Settled once: a second settlement finds no open escrow.
    let again = settle_appeal(&mut author, item(), &terminal, 4);
    assert_eq!(again, Err(Unsettled::Refused(AuthorError::NoOpenEscrow)));
}

/// An appeal whose item ends in the contested pool is promoted (D38, `docs/05` [5b]).
#[test]
fn an_appeal_that_ends_in_the_contested_pool_is_promoted() {
    let mut author = established();
    let before = reputation(&author, 2);
    file(&mut author, item(), 2).unwrap();
    let terminal = run(
        reviewed(),
        &ItemVerdicts {
            dif: Recheck::Dif,
            source_verified: true,
            ..appealed()
        },
    )
    .unwrap();
    assert_eq!(terminal, State::Contested);
    settle_appeal(&mut author, item(), &terminal, 3).unwrap();
    assert_eq!(author.outcome(&item()), Some((true, 3)));
    assert!(reputation(&author, 4) > before);
}
