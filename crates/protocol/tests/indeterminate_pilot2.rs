//! A pilot whose stage-2 fit did not converge is indeterminate, and an appeal settles only on a
//! concluded pilot (`docs/15` A4 residual (a) and A11 (f), `docs/02` §B.3).

use identity::nym::Nym;
use network::cid::cid;
use protocol::appeal::AuthorHistory;
use protocol::exploration::{outcome_of, record_outcome, FalseNegatives, Scored, EXPLORATION_RATE};
use protocol::exposure::RetirementReason;
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, Invalid, RejectReason, State, K_MIN};
use protocol::orchestrator::{review_round, run_item, settle_appeal, ItemVerdicts, Judgment};
use protocol::pilot::Screening;
use protocol::probation::SkillTrack;
use protocol::revalidation::{target_rechecks, Recheck};
use scoring::latent::{Ability, LatentDif};
use scoring::reputation::{AuthorPrior, CusumParams};
use scoring::Convergence;

/// A one-class fit of two trial items that ran out of iterations, built by hand.
fn unconverged() -> LatentDif {
    LatentDif {
        classes: 1,
        non_uniform: false,
        pi: vec![1.0],
        eta: vec![0.0],
        ability: Ability {
            nodes: vec![0.0],
            weights: vec![1.0],
        },
        anchor_a: Vec::new(),
        anchor_b: Vec::new(),
        anchor_c: Vec::new(),
        item_a: vec![vec![1.2; 2]],
        item_b: vec![vec![0.0; 2]],
        item_c: vec![0.0; 2],
        dif: vec![0.0; 2],
        a_gap: vec![0.0; 2],
        posterior: Vec::new(),
        bic_gain: 0.0,
        candidates: Vec::new(),
        status: Convergence::MaxIters,
    }
}

fn batch(dif: Recheck, source_verified: bool) -> Event {
    Event::Pilot2Batch {
        batch_size: K_MIN,
        dif,
        source_verified,
    }
}

/// The states awaiting stage 2: entered on its own account or on appeal, or explored.
fn awaiting() -> Vec<State> {
    let explored = [
        RejectReason::Defect,
        RejectReason::Polarized,
        RejectReason::Borderline,
    ]
    .map(|reason| State::Explored {
        reason,
        screened: true,
    });
    [false, true]
        .map(|appealed| State::Pilot2 { appealed })
        .into_iter()
        .chain(explored)
        .collect()
}

/// A4(a): an unconverged stage-2 re-check leaves both paths pending, a verified source or not.
#[test]
fn an_unconverged_stage_2_leaves_both_paths_pending() {
    let rechecks = target_rechecks(&unconverged());
    assert_eq!(rechecks, vec![Recheck::Indeterminate; 2]);
    for state in awaiting() {
        for source_verified in [false, true] {
            let after = step(state.clone(), batch(rechecks[0], source_verified)).unwrap();
            assert_eq!(after, state, "source verified {source_verified}");
            assert_eq!(outcome_of(&after, EXPLORATION_RATE), Scored::Pending);
        }
    }
}

/// A4(a): a pending stage 2 scores nothing, records no review and is no concluded exploration.
#[test]
fn a_pending_stage_2_feeds_no_track_and_no_measurement() {
    let params = CusumParams::default();
    let mut track = SkillTrack::new();
    track.record_observed(0.2, 1.0, &params);
    track.record_unobserved();
    let mut false_negatives = FalseNegatives::default();
    for state in awaiting() {
        let pending = step(state, batch(Recheck::Indeterminate, true)).unwrap();
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

/// A4(a): `NoDif` and `Dif` move items as `passed` did; the batch floor and phase come first.
#[test]
fn determinate_readings_and_refusals_are_unchanged() {
    for appealed in [false, true] {
        let pilot2 = State::Pilot2 { appealed };
        for (dif, source_verified, reached) in [
            (Recheck::NoDif, false, State::ActivePool),
            (Recheck::NoDif, true, State::ActivePool),
            (Recheck::Dif, false, State::Rejected(RejectReason::Dif)),
            (Recheck::Dif, true, State::Contested),
        ] {
            assert_eq!(
                step(pilot2.clone(), batch(dif, source_verified)),
                Ok(reached)
            );
        }
    }
    let explored = State::Explored {
        reason: RejectReason::Polarized,
        screened: true,
    };
    for (dif, source_verified, passed) in [
        (Recheck::NoDif, false, true),
        (Recheck::NoDif, true, true),
        (Recheck::Dif, false, false),
        (Recheck::Dif, true, true),
    ] {
        let measured = State::Measured {
            reason: RejectReason::Polarized,
            passed,
        };
        assert_eq!(
            step(explored.clone(), batch(dif, source_verified)),
            Ok(measured)
        );
    }
    for dif in [Recheck::NoDif, Recheck::Dif, Recheck::Indeterminate] {
        for state in awaiting() {
            let small = Event::Pilot2Batch {
                batch_size: K_MIN - 1,
                dif,
                source_verified: true,
            };
            assert_eq!(step(state, small), Err(Invalid::BatchTooSmall));
        }
        let unscreened = State::Explored {
            reason: RejectReason::Defect,
            screened: false,
        };
        for state in [
            State::Pilot1 { appealed: true },
            unscreened,
            State::ActivePool,
            State::Rejected(RejectReason::Dif),
        ] {
            assert_eq!(step(state, batch(dif, true)), Err(Invalid::UnexpectedEvent));
        }
    }
}

fn reviewed() -> State {
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
    review_round(admitted, cid(b"an item"), panel, &judgments).unwrap()
}

fn indeterminate(gate: GateOutcome, appealed: bool, explored: bool) -> ItemVerdicts {
    ItemVerdicts {
        gate,
        appealed,
        appeal_within_window: true,
        author_reputation: 0.6,
        appeal_floor: 0.4,
        enough_respondents: true,
        screen: Screening::Pass,
        dif: Recheck::Indeterminate,
        source_verified: true,
        pilot2_batch_size: K_MIN,
        explored,
    }
}

/// A4(a): `run_item` stops at a pending stage 2 on both paths, a verified source or not.
#[test]
fn run_item_stops_at_a_pending_stage_2() {
    let run = |v: &ItemVerdicts| run_item(reviewed(), v, None, |_| unreachable!());
    let pass = indeterminate(GateOutcome::Pass, false, false);
    assert_eq!(run(&pass), Ok(State::Pilot2 { appealed: false }));
    let appealed = indeterminate(GateOutcome::AppealEligible, true, false);
    assert_eq!(run(&appealed), Ok(State::Pilot2 { appealed: true }));
    let explored = indeterminate(GateOutcome::Reject, false, true);
    assert_eq!(
        run(&explored),
        Ok(State::Explored {
            reason: RejectReason::Defect,
            screened: true
        })
    );
}

/// Every state but a pilot's conclusion: before, during and outside the appealed item's pilot.
fn not_settling() -> Vec<State> {
    let item = cid(b"an item");
    let panel = vec![Nym([1; 32])];
    vec![
        State::Deposited,
        State::Admitted,
        State::InReview {
            item,
            panel: panel.clone(),
            commits: Vec::new(),
        },
        State::Revealing {
            item,
            panel: panel.clone(),
            commits: Vec::new(),
            reveals: Vec::new(),
        },
        State::SupplementaryReview {
            item,
            panel,
            extra_panel: Vec::new(),
            commits: Vec::new(),
            commits_closed: false,
            reveals: Vec::new(),
        },
        State::AppealEligible,
        State::Pilot1 { appealed: true },
        State::Pilot2 { appealed: true },
        State::Rejected(RejectReason::Defect),
        State::Rejected(RejectReason::Polarized),
        State::Rejected(RejectReason::Borderline),
        State::Explored {
            reason: RejectReason::Polarized,
            screened: true,
        },
        State::Measured {
            reason: RejectReason::Polarized,
            passed: true,
        },
        State::Retired(RetirementReason::Exposure),
        State::Retired(RetirementReason::EmergingDif),
    ]
}

fn screen(screen: Screening) -> Event {
    Event::Pilot1Batch {
        enough_respondents: true,
        screen,
    }
}

/// A11 (f): only a concluded pilot settles; the escrow handed back settles once it concludes.
#[test]
fn an_appeal_settles_only_on_its_pilot_s_conclusion() {
    let pending = [screen(Screening::Indeterminate), screen(Screening::Pass)];
    let stage2 = |last| [batch(Recheck::Indeterminate, true), last];
    let walks: [(Vec<Event>, State, [f64; 2]); 4] = [
        (
            [
                pending.to_vec(),
                stage2(batch(Recheck::NoDif, false)).to_vec(),
            ]
            .concat(),
            State::ActivePool,
            [0.9, 0.85],
        ),
        (
            [pending.to_vec(), stage2(batch(Recheck::Dif, true)).to_vec()].concat(),
            State::Contested,
            [0.9, 0.85],
        ),
        (
            [
                pending.to_vec(),
                stage2(batch(Recheck::Dif, false)).to_vec(),
            ]
            .concat(),
            State::Rejected(RejectReason::Dif),
            [0.9, 0.0],
        ),
        (
            vec![screen(Screening::Indeterminate), screen(Screening::Fail)],
            State::Rejected(RejectReason::Screen),
            [0.9, 0.0],
        ),
    ];
    for (events, concluded, qualities) in walks {
        let mut author = AuthorHistory::new();
        author.record(0.9, 6.0);
        let mut escrow = author.file_appeal(&AuthorPrior::default()).unwrap();
        let filed = author.clone();
        for state in not_settling() {
            escrow = settle_appeal(&mut author, escrow, &state, 0.85).unwrap_err();
        }
        let mut state = State::Pilot1 { appealed: true };
        for event in events {
            escrow = settle_appeal(&mut author, escrow, &state, 0.85).unwrap_err();
            assert_eq!(author, filed, "{state:?}");
            state = step(state, event).unwrap();
        }
        assert_eq!(state, concluded);
        settle_appeal(&mut author, escrow, &state, 0.85).unwrap();
        assert_eq!(author.qualities(), qualities);
    }
}
