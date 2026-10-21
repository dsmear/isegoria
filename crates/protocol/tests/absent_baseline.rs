//! A10's absent baseline (`docs/15` A10): the golden-item fallback scores every report 0, and for
//! the node a score of 0, an unobserved item and no record are three different records.

use identity::nym::Nym;
use protocol::honeypot::reviewer_skills;
use protocol::probation::{SkillTrack, Status, N_PROBATION};
use protocol::results::{EpochResults, ResultRecord, ResultsState};
use scoring::reputation::CusumParams;

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

fn scores(reviewer: Nym, score: f64, times: usize) -> Vec<ResultRecord> {
    let record = ResultRecord::ReviewerScore {
        reviewer,
        score,
        inclusion: 1.0,
    };
    vec![record; times]
}

fn track(state: &ResultsState, reviewer: Nym) -> &SkillTrack {
    state.track(&reviewer).expect("a track")
}

/// A10: on the golden items a reviewer with no weighted other scores 0, whatever it reports.
#[test]
fn a_golden_reviewer_without_weighted_others_scores_zero_whatever_it_reports() {
    let outcomes: Vec<f64> = (0..10).map(|i| f64::from(i % 2)).collect();
    let expert: Vec<f64> = outcomes.iter().map(|&o| 0.05 + 0.9 * o).collect();
    let contrarian: Vec<f64> = expert.iter().map(|p| 1.0 - p).collect();
    for report in [&expert, &contrarian, &vec![0.5; 10]] {
        let alone = reviewer_skills(&[report.clone()], &[1.0], &outcomes);
        assert_eq!(alone, vec![0.0]);
        let peers = [report.clone(), expert.clone()];
        assert_eq!(reviewer_skills(&peers, &[1.0, 0.0], &outcomes)[0], 0.0);
    }
    let paired = reviewer_skills(&[expert, contrarian], &[1.0, 1.0], &outcomes);
    assert!(paired[0] > 0.0 && paired[1] < 0.0);
}

/// A10: thirty scores of 0 end a new reviewer's probation at weight 1; unobserved items do not.
#[test]
fn zero_scores_unobserved_items_and_no_record_differ_on_probation() {
    let (zeros, unobserved, absent) = (nym(1), nym(2), nym(3));
    let mut state = ResultsState::default();
    let mut records = scores(zeros, 0.0, N_PROBATION);
    records.extend(vec![
        ResultRecord::ReviewerUnobserved {
            reviewer: unobserved
        };
        N_PROBATION
    ]);
    apply(&mut state, 1, records);
    let z = track(&state, zeros);
    assert_eq!(
        (z.reviewed(), z.scored(), z.skill()),
        (N_PROBATION, N_PROBATION, 0.0)
    );
    assert_eq!(
        (z.status(false), z.weight(false, 3.0)),
        (Status::Established, 1.0)
    );
    let u = track(&state, unobserved);
    assert_eq!((u.reviewed(), u.scored(), u.skill()), (N_PROBATION, 0, 0.0));
    assert_eq!(
        (u.status(false), u.weight(false, 3.0)),
        (Status::Probation, 0.0)
    );
    assert!(state.track(&absent).is_none());
}

/// A10: past probation a score of 0 feeds the change detector, and eight reset a mean of 0.25.
#[test]
fn a_zero_score_moves_the_change_detector_and_an_unobserved_item_does_not() {
    let k = CusumParams::default().k;
    let (zero, unobserved, absent) = (nym(1), nym(2), nym(3));
    let mut state = ResultsState::default();
    let history = [zero, unobserved, absent].map(|r| scores(r, 0.25, N_PROBATION));
    apply(&mut state, 1, history.concat());
    let mut next = scores(zero, 0.0, 1);
    next.push(ResultRecord::ReviewerUnobserved {
        reviewer: unobserved,
    });
    apply(&mut state, 2, next);
    let (z, u, a) = (
        track(&state, zero),
        track(&state, unobserved),
        track(&state, absent),
    );
    assert_eq!(
        (z.statistic(), u.statistic(), a.statistic()),
        (0.25 - k, 0.0, 0.0)
    );
    assert_eq!((z.scored(), u.scored(), a.scored()), (31, 30, 30));
    assert_eq!((z.reviewed(), u.reviewed(), a.reviewed()), (31, 31, 30));
    assert_eq!(
        (z.skill(), u.skill(), a.skill()),
        (7.5 / 31.0, 7.5 / 31.0, 0.25)
    );
    apply(&mut state, 3, scores(zero, 0.0, 6));
    assert_eq!(track(&state, zero).alarms(), 0);
    apply(&mut state, 4, scores(zero, 0.0, 1));
    let z = track(&state, zero);
    assert_eq!(
        (z.alarms(), z.scored(), z.status(false)),
        (1, 0, Status::Probation)
    );
}
