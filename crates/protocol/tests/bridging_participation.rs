//! The bridging population at the protocol boundary (`docs/15` A7): a founder after an alarm
//! and a probationer past the review floor stay on the axis at weight 0 and change no score,
//! coverage or gate outcome, in the epoch or in the band's extra round.

use identity::nym::Nym;
use protocol::gate::{bridging_gate, supplementary_review, GateOutcome, APPEAL_GAP, EPS, TAU};
use protocol::orchestrator::{
    axis_mask, bridging_weights, epoch_weight_cap, expanded_ratings, weighted_ratings,
    ReviewerStanding,
};
use protocol::probation::{SkillTrack, N_PROBATION};
use scoring::bridging::{bridge_scores, fit, side_balanced, BridgeScores, BridgingParams, Obs};
use scoring::reputation::CusumParams;
use std::fs;
use std::path::PathBuf;

fn read_matrix(name: &str) -> Vec<Vec<f64>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../scoring/tests/fixtures")
        .join(name);
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split(',').map(|c| c.trim().parse().unwrap()).collect())
        .collect()
}

/// A founder whose track the detector sent back to probation.
fn alarmed_founder() -> ReviewerStanding {
    let params = CusumParams::default();
    let mut track = SkillTrack::new();
    for _ in 0..N_PROBATION {
        assert!(track.record(0.5, &params).is_none());
    }
    assert!(track.record(0.2, &params).is_none());
    assert!(track.record(-1.0, &params).is_some());
    track.standing(true)
}

/// A reviewer past the review floor with fewer than `N_PROBATION` scored outcomes.
fn probationer_past_the_floor() -> ReviewerStanding {
    let params = CusumParams::default();
    let mut track = SkillTrack::new();
    for _ in 0..N_PROBATION {
        track.record_unobserved();
    }
    for _ in 0..5 {
        assert!(track.record(0.1, &params).is_none());
    }
    track.standing(false)
}

/// The fixture's epoch, with `extra` rows (their standing, rating every item 0) at `at`.
fn epoch(
    extra: &[(usize, ReviewerStanding)],
) -> (Vec<Vec<f64>>, Vec<Vec<bool>>, Vec<ReviewerStanding>) {
    let mut r = read_matrix("R.csv");
    let mut mask: Vec<Vec<bool>> = read_matrix("mask.csv")
        .iter()
        .map(|row| row.iter().map(|&v| v != 0.0).collect())
        .collect();
    let mut standings = vec![ReviewerStanding::founder(); r.len()];
    let m = r[0].len();
    for &(at, s) in extra {
        r.insert(at, vec![0.0; m]);
        mask.insert(at, vec![true; m]);
        standings.insert(at, s);
    }
    (r, mask, standings)
}

fn scores(extra: &[(usize, ReviewerStanding)]) -> BridgeScores {
    let (r, mask, standings) = epoch(extra);
    let w_max = epoch_weight_cap(&standings);
    let ratings = weighted_ratings(&r, &mask, &standings, w_max).unwrap();
    bridge_scores(&ratings, &BridgingParams::default(), 10, 0.85).unwrap()
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn gate(b: &BridgeScores) -> Vec<GateOutcome> {
    (0..b.robust.len())
        .map(|j| {
            bridging_gate(
                b.robust[j],
                b.full.gap[j],
                b.coverage[j],
                TAU,
                EPS,
                APPEAL_GAP,
            )
        })
        .collect()
}

/// A7: an alarmed founder and a probationer past the floor leave the epoch's scores and gate.
#[test]
fn weight_zero_axis_reviewers_leave_the_epoch_unchanged() {
    let (founder, probationer) = (alarmed_founder(), probationer_past_the_floor());
    assert_eq!(axis_mask(&[founder, probationer]), vec![true, true]);
    assert_eq!(
        bridging_weights(&[founder, probationer], 3.0),
        vec![0.0, 0.0]
    );
    let without = scores(&[]);
    let with = scores(&[(50, founder), (201, probationer)]);
    assert_eq!(bits(&with.full.score), bits(&without.full.score));
    assert_eq!(bits(&with.full.gap), bits(&without.full.gap));
    assert_eq!(bits(&with.robust), bits(&without.robust));
    assert_eq!(with.coverage, without.coverage);
    assert_eq!(gate(&with), gate(&without));
}

/// A7: in the band's extra round, rows added for them change neither `S_j` nor the verdict.
#[test]
fn weight_zero_axis_newcomers_leave_the_re_decision_unchanged() {
    let (r, mask, standings) = epoch(&[]);
    let base = weighted_ratings(&r, &mask, &standings, 3.0).unwrap();
    let j = base.m;
    let mut obs = base.obs.clone();
    for u in [0usize, 20, 40, 60, 80, 100, 120, 140, 160] {
        obs.push(Obs {
            u,
            j,
            r: TAU + 0.02,
        });
    }
    let base = scoring::bridging::Ratings {
        m: base.m + 1,
        obs,
        ..base
    };
    let rows: Vec<Nym> = (0..base.n).map(|u| Nym([u as u8; 32])).collect();
    let outsiders = [Nym([240; 32]), Nym([241; 32])];
    let kinds = [alarmed_founder(), probationer_past_the_floor()];
    let p = BridgingParams::default();
    let with_reveals = |reveals: &[(Nym, f64)]| {
        let expanded = expanded_ratings(
            &base,
            &rows,
            j,
            reveals,
            |n| kinds[(n.0[0] - 240) as usize],
            3.0,
        );
        let s = side_balanced(&fit(&expanded, &p).unwrap()).score[j];
        (
            s.to_bits(),
            supplementary_review(&expanded, &p, j, TAU, APPEAL_GAP).unwrap(),
        )
    };
    let alone = with_reveals(&[]);
    for vote in [0.0, 1.0] {
        let reveals: Vec<(Nym, f64)> = outsiders.iter().map(|&n| (n, vote)).collect();
        assert_eq!(with_reveals(&reveals), alone, "vote {vote}");
    }
    let participants = [
        (rows[3], 0.0),
        (rows[43], 0.0),
        (rows[83], 0.0),
        (rows[123], 0.0),
    ];
    assert_ne!(
        with_reveals(&participants).1,
        alone.1,
        "participants still decide"
    );
}

/// A7: no standing puts a positive weight off the axis; that combination is the scoring API's.
#[test]
fn no_standing_weighs_a_reviewer_off_the_axis() {
    for is_founder in [false, true] {
        for alarms in [0, 1, 2] {
            for reviews in [0, N_PROBATION - 1, N_PROBATION, 400] {
                for judgments in [0, N_PROBATION - 1, N_PROBATION, 400] {
                    if judgments > reviews {
                        continue;
                    }
                    let s = ReviewerStanding {
                        is_founder,
                        judgments_with_outcome: judgments,
                        skill: 0.05,
                        reviews,
                        alarms,
                    };
                    let (axis, w) = (axis_mask(&[s])[0], bridging_weights(&[s], 3.0)[0]);
                    assert!(axis || w == 0.0, "{s:?}: weight {w} off the axis");
                }
            }
        }
    }
}
