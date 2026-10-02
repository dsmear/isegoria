//! A stage-1 screen whose fit did not converge is indeterminate: the item stays where it was,
//! pending, on the entering and on the explored path (`docs/15` A11, `docs/02` §B.2).

use identity::nym::Nym;
use network::cid::cid;
use protocol::appeal::AuthorHistory;
use protocol::exploration::{outcome_of, record_outcome, FalseNegatives, Scored, EXPLORATION_RATE};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, Invalid, RejectReason, State, K_MIN};
use protocol::orchestrator::{review_round, run_item, settle_appeal, ItemVerdicts, Judgment};
use protocol::pilot::{stage1_verdicts, Screening, Stage1Fit};
use protocol::probation::SkillTrack;
use scoring::latent::Formats;
use scoring::reputation::{AuthorPrior, CusumParams};
use scoring::Convergence;

/// Two items whose parameters pass every cut, from a fit of `status`.
fn fit(status: Convergence) -> Stage1Fit {
    Stage1Fit {
        status,
        rpb: vec![0.5; 2],
        a: vec![1.2; 2],
        b: vec![0.0; 2],
        c: vec![0.0; 2],
    }
}

fn batch(screen: Screening) -> Event {
    Event::Pilot1Batch {
        enough_respondents: true,
        screen,
    }
}

/// The states awaiting a screen: entered on its own account or on appeal, or explored.
fn awaiting() -> Vec<State> {
    let explored = [
        RejectReason::Defect,
        RejectReason::Polarized,
        RejectReason::Borderline,
    ]
    .map(|reason| State::Explored {
        reason,
        screened: false,
    });
    [false, true]
        .map(|appealed| State::Pilot1 { appealed })
        .into_iter()
        .chain(explored)
        .collect()
}

/// A11: an unconverged stage-1 fit leaves both paths pending, never a rejection or outcome 0.
#[test]
fn an_unconverged_screen_leaves_both_paths_pending() {
    for status in [Convergence::MaxIters, Convergence::LineSearchFailed] {
        let readings = stage1_verdicts(&fit(status), &Formats::open(2, 2));
        assert_eq!(readings, vec![Screening::Indeterminate; 2], "{status:?}");
        for state in awaiting() {
            let after = step(state.clone(), batch(readings[0])).unwrap();
            assert_eq!(after, state);
            assert_eq!(outcome_of(&after, EXPLORATION_RATE), Scored::Pending);
        }
    }
}

/// A11: a pending screen scores nothing, records no review and is no concluded exploration.
#[test]
fn a_pending_screen_feeds_no_track_and_no_measurement() {
    let params = CusumParams::default();
    let mut track = SkillTrack::new();
    track.record_observed(0.2, 1.0, &params);
    track.record_unobserved();
    let mut false_negatives = FalseNegatives::default();
    for state in awaiting() {
        let pending = step(state, batch(Screening::Indeterminate)).unwrap();
        let before = track.clone();
        let alarm = record_outcome(
            &mut track,
            &pending,
            EXPLORATION_RATE,
            |_| unreachable!("no outcome to score"),
            &params,
        );
        assert_eq!((alarm, &track), (None, &before));
        false_negatives.record(&pending);
    }
    assert_eq!(false_negatives, FalseNegatives::default());
}

/// A11: a converged fit screens as before; respondents and phase are checked before the reading.
#[test]
fn determinate_readings_and_refusals_are_unchanged() {
    let readings = stage1_verdicts(&fit(Convergence::Converged), &Formats::open(2, 2));
    assert_eq!(readings, vec![Screening::Pass; 2]);
    let defect = |screened| State::Explored {
        reason: RejectReason::Defect,
        screened,
    };
    for appealed in [false, true] {
        let pilot1 = State::Pilot1 { appealed };
        assert_eq!(
            step(pilot1.clone(), batch(Screening::Pass)),
            Ok(State::Pilot2 { appealed })
        );
        assert_eq!(
            step(pilot1, batch(Screening::Fail)),
            Ok(State::Rejected(RejectReason::Screen))
        );
    }
    assert_eq!(
        step(defect(false), batch(Screening::Pass)),
        Ok(defect(true))
    );
    assert_eq!(
        step(defect(false), batch(Screening::Fail)),
        Ok(State::Measured {
            reason: RejectReason::Defect,
            passed: false
        })
    );
    for screen in [Screening::Pass, Screening::Fail, Screening::Indeterminate] {
        for state in awaiting() {
            let short = Event::Pilot1Batch {
                enough_respondents: false,
                screen,
            };
            assert_eq!(step(state, short), Err(Invalid::NotEnoughRespondents));
        }
        for state in [
            State::Pilot2 { appealed: false },
            defect(true),
            State::ActivePool,
            State::Rejected(RejectReason::Screen),
        ] {
            assert_eq!(step(state, batch(screen)), Err(Invalid::UnexpectedEvent));
        }
    }
}

fn reviewed() -> State {
    let item = cid(b"an item");
    let admitted = step(
        deposit(true, true, true, true).unwrap(),
        Event::Admit {
            seed_from_beacon: true,
        },
    )
    .unwrap();
    let judgments: Vec<Judgment> = (1..=7u8)
        .map(|i| Judgment {
            nym: Nym([i; 32]),
            prob: 0.7,
            nonce: [i; 32],
        })
        .collect();
    let panel = judgments.iter().map(|j| j.nym).collect();
    review_round(admitted, item, panel, &judgments).unwrap()
}

/// An item's verdicts with an indeterminate screen and a stage-2 batch the machine would refuse.
fn indeterminate(gate: GateOutcome, appealed: bool, explored: bool) -> ItemVerdicts {
    ItemVerdicts {
        gate,
        appealed,
        appeal_within_window: true,
        author_reputation: 0.6,
        appeal_floor: 0.4,
        enough_respondents: true,
        screen: Screening::Indeterminate,
        dif_passed: true,
        source_verified: false,
        pilot2_batch_size: K_MIN - 1,
        explored,
    }
}

/// A11: `run_item` stops at a pending screen, before stage 2; the appeal's escrow stays open.
#[test]
fn run_item_stops_at_a_pending_screen_and_settles_no_appeal() {
    let run = |v: &ItemVerdicts| run_item(reviewed(), v, None, |_| unreachable!());
    let pass = indeterminate(GateOutcome::Pass, false, false);
    assert_eq!(run(&pass), Ok(State::Pilot1 { appealed: false }));
    let explored = indeterminate(GateOutcome::Reject, false, true);
    assert_eq!(
        run(&explored),
        Ok(State::Explored {
            reason: RejectReason::Defect,
            screened: false
        })
    );
    let appealed = indeterminate(GateOutcome::AppealEligible, true, false);
    let pending = run(&appealed).unwrap();
    assert_eq!(pending, State::Pilot1 { appealed: true });

    let prior = AuthorPrior::default();
    let mut author = AuthorHistory::new();
    author.record(0.9, 6.0);
    let escrow = author.file_appeal(&prior).unwrap();
    let filed = author.clone();
    let escrow = settle_appeal(&mut author, escrow, &pending, 0.85).unwrap_err();
    assert_eq!(author, filed);
    let pilot2 = step(pending, batch(Screening::Pass)).unwrap();
    let pool = step(
        pilot2,
        Event::Pilot2Batch {
            batch_size: K_MIN,
            passed: true,
            source_verified: false,
        },
    )
    .unwrap();
    settle_appeal(&mut author, escrow, &pool, 0.85).unwrap();
    assert_eq!(author.qualities(), &[0.9, 0.85]);
}
