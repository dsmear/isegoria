//! A founder's weight 1 is the bootstrap's: a change-detector alarm sends a founder back to
//! probation like anyone, through the standing and the bridging weights (`docs/15` A6, D34).

use identity::nym::Nym;
use protocol::orchestrator::{
    axis_mask, bridging_weights, epoch_weight_cap, expanded_ratings, weighted_ratings,
    ReviewerStanding,
};
use protocol::probation::{Alarm, SkillTrack, Status, N_PROBATION};
use scoring::bridging::Ratings;
use scoring::reputation::{odds_weight, CusumParams, EvaluatorParams};

/// A binding cap: the recovered skill of 0.5 has an odds weight near 57.
const W_MAX: f64 = 3.0;

/// The score of an established stretch; a drop to 0.2 then −1.0 makes the detector fire.
const GOOD: f64 = 0.5;

fn record(track: &mut SkillTrack, score: f64, times: usize) {
    for _ in 0..times {
        assert!(track.record(score, &CusumParams::default()).is_none());
    }
}

/// From a fresh stretch: `N_PROBATION` good scores, then the drop the detector catches.
fn establish_then_drop(track: &mut SkillTrack) -> Alarm {
    record(track, GOOD, N_PROBATION);
    record(track, 0.2, 1);
    track
        .record(-1.0, &CusumParams::default())
        .expect("the detector fires")
}

fn alarmed() -> SkillTrack {
    let mut track = SkillTrack::new();
    assert_eq!(establish_then_drop(&mut track).count, 1);
    track
}

fn skill_weight(skill: f64, k: usize) -> f64 {
    odds_weight(skill, k, &EvaluatorParams::default())
}

/// A6: a founder with no alarm weighs 1 until the threshold, then the capped skill weight.
#[test]
fn a_founder_seeds_at_one_then_weighs_its_skill() {
    let mut track = SkillTrack::new();
    record(&mut track, GOOD, N_PROBATION - 1);
    assert_eq!(track.status(true), Status::Founder);
    assert_eq!(
        (track.weight(true, W_MAX), track.weight(false, W_MAX)),
        (1.0, 0.0)
    );
    record(&mut track, GOOD, 1);
    assert_eq!(track.status(true), Status::Established);
    assert_eq!(track.weight(true, W_MAX), W_MAX);
    assert_eq!(
        track.weight(true, f64::INFINITY),
        skill_weight(GOOD, N_PROBATION)
    );
}

/// A6, D34: after a real alarm founder and non-founder alike are on probation at weight 0.
#[test]
fn an_alarm_sends_a_founder_back_to_probation_like_anyone() {
    let track = alarmed();
    for is_founder in [true, false] {
        assert_eq!(track.status(is_founder), Status::Probation);
        assert_eq!(track.weight(is_founder, W_MAX), 0.0);
        let standing = track.standing(is_founder);
        assert_eq!((standing.is_founder, standing.alarms), (is_founder, 1));
    }
}

/// A6: weight 0 after `N_PROBATION − 1` new outcomes; at the threshold the capped skill weight.
#[test]
fn a_founder_recovers_its_skill_weight_not_the_seed() {
    let mut track = alarmed();
    record(&mut track, GOOD, N_PROBATION - 1);
    assert_eq!(track.status(true), Status::Probation);
    assert_eq!(track.weight(true, W_MAX), 0.0);
    record(&mut track, GOOD, 1);
    assert_eq!(track.status(true), Status::Established);
    assert_eq!(track.weight(true, W_MAX), W_MAX);
    assert_eq!(
        track.weight(true, f64::INFINITY),
        skill_weight(GOOD, N_PROBATION)
    );
    let mut modest = alarmed();
    record(&mut modest, 0.02, N_PROBATION);
    let w = modest.weight(true, W_MAX);
    assert_eq!(w, skill_weight(0.02, N_PROBATION));
    assert!(w > 1.0 && w < W_MAX, "{w}");
}

/// A6: reviews without an observed outcome do not end the probation.
#[test]
fn unobserved_reviews_do_not_end_the_probation() {
    let mut track = alarmed();
    for _ in 0..2 * N_PROBATION {
        track.record_unobserved();
    }
    record(&mut track, GOOD, N_PROBATION - 1);
    assert_eq!(track.reviewed(), 3 * N_PROBATION - 1);
    assert_eq!(track.status(true), Status::Probation);
    assert_eq!(track.weight(true, W_MAX), 0.0);
}

/// A6: a second alarm sends the recovered founder back to probation, never to the seed.
#[test]
fn a_second_alarm_does_not_restore_the_seed() {
    let mut track = alarmed();
    assert_eq!(establish_then_drop(&mut track).count, 2);
    assert_eq!(track.status(true), Status::Probation);
    assert_eq!(track.weight(true, W_MAX), 0.0);
    record(&mut track, GOOD, N_PROBATION - 1);
    assert_eq!(track.weight(true, W_MAX), 0.0);
    record(&mut track, GOOD, 1);
    assert_eq!(track.weight(true, W_MAX), W_MAX);
}

/// A6: the alarm reaches the bridging rows as the track's own weight, an extra-round row too.
#[test]
fn the_alarm_reaches_the_bridging_weights_through_the_standing() {
    let founder = alarmed();
    let other = alarmed();
    let mut recovered = alarmed();
    record(&mut recovered, GOOD, N_PROBATION);
    let standings = [
        founder.standing(true),
        other.standing(false),
        recovered.standing(true),
        ReviewerStanding::founder(),
    ];
    let direct = vec![
        founder.weight(true, W_MAX),
        other.weight(false, W_MAX),
        recovered.weight(true, W_MAX),
        1.0,
    ];
    assert_eq!(direct, vec![0.0, 0.0, W_MAX, 1.0]);
    assert_eq!(bridging_weights(&standings, W_MAX), direct);
    let rows = weighted_ratings(
        &vec![vec![0.7, 0.4]; 4],
        &vec![vec![true; 2]; 4],
        &standings,
        W_MAX,
    )
    .unwrap();
    assert_eq!(rows.weights, direct);
    assert_eq!(rows.axis, vec![true, false, true, true]);

    let base = Ratings::from_dense(&[vec![0.7, 0.4]], &[vec![true; 2]]);
    let newcomer = Nym([9; 32]);
    let extra = expanded_ratings(
        &base,
        &[Nym([1; 32])],
        1,
        &[(newcomer, 0.2)],
        |_| standings[0],
        W_MAX,
    );
    assert_eq!((extra.weights[1], extra.axis[1]), (0.0, true));
}

/// A6: the epoch's cap leaves out an alarmed founder; its axis membership stays.
#[test]
fn an_alarmed_founder_carries_no_weight_into_the_cap() {
    let founder = alarmed().standing(true);
    assert_eq!(epoch_weight_cap(&[founder]), f64::INFINITY);
    let established = ReviewerStanding::established(0.02);
    let cap = epoch_weight_cap(&[founder, established]);
    assert_eq!(cap, 3.0 * skill_weight(0.02, N_PROBATION));
    assert_ne!(
        cap,
        epoch_weight_cap(&[ReviewerStanding::founder(), established])
    );
    assert_eq!(axis_mask(&[founder]), vec![true]);
}
