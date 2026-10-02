//! The baselines of a band item's reviewers (`docs/02` §C.2, `docs/16` C5): a first panelist
//! against the other first panelists, an extra reviewer against the frozen first panel, no
//! extra report in any baseline, no baseline where no weight remains.

use identity::nym::Nym;
use network::cid::cid;
use protocol::exploration::{outcome_of, record_outcome, Observation, Scored, EXPLORATION_RATE};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, State, K_MIN};
use protocol::orchestrator::{
    bridging_weights, epoch_weight_cap, extra_round, review_round, ExtraRound, Judgment,
    ReviewerStanding,
};
use protocol::panel_scores::{
    extra_round_baseline, first_panel_baselines, item_scores, Forecast, ItemScores,
};
use protocol::pilot::Screening;
use protocol::probation::SkillTrack;
use protocol::revalidation::Recheck;
use scoring::reputation::CusumParams;

fn panel() -> Vec<Forecast> {
    [(0.8, 2.0), (0.3, 0.5), (0.6, 1.5), (0.1, 0.0)]
        .iter()
        .map(|&(prob, weight)| Forecast { prob, weight })
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

/// By hand: (0.5·0.3 + 1.5·0.6)/2, (2·0.8 + 1.5·0.6)/3.5, (2·0.8 + 0.5·0.3)/2.5, 2.65/4.
const FIRST: [f64; 4] = [0.525, 2.5 / 3.5, 0.7, 0.6625];
/// By hand: (2·0.8 + 0.5·0.3 + 1.5·0.6 + 0·0.1)/4.
const EXTRA: f64 = 0.6625;

/// A1 C5: the baselines are the hand-computed weighted means of the forecasts each may read.
#[test]
fn the_baselines_are_the_weighted_means_they_may_read() {
    let got = first_panel_baselines(&panel());
    for (g, want) in got.iter().zip(FIRST) {
        assert!(close(g.unwrap(), want), "{got:?}");
    }
    assert!(close(extra_round_baseline(&panel()).unwrap(), EXTRA));
}

/// A1 C5: a first panelist's own report never moves its own baseline.
#[test]
fn a_first_panelist_s_own_report_leaves_its_baseline() {
    for u in 0..4 {
        let mut moved = panel();
        moved[u].prob = 1.0 - moved[u].prob;
        assert_eq!(
            first_panel_baselines(&moved)[u],
            first_panel_baselines(&panel())[u]
        );
    }
}

/// A1 C5: extra reports, added or changed, move no first baseline and no extra baseline.
#[test]
fn extra_reports_enter_no_baseline() {
    let first = panel();
    let alone = item_scores(&first, &[], 1.0);
    for extra in [vec![0.2], vec![0.9, 0.05], vec![0.0, 1.0, 0.5]] {
        let scores = item_scores(&first, &extra, 1.0);
        assert_eq!(scores.first, alone.first);
        for (k, &p) in extra.iter().enumerate() {
            let by_hand = (EXTRA - 1.0f64).powi(2) - (p - 1.0f64).powi(2);
            assert!(close(scores.extra[k].unwrap(), by_hand), "{k}: {scores:?}");
        }
    }
}

/// A1 C5: a first report of positive weight moves the extra baseline; one of weight 0 does not.
#[test]
fn the_extra_baseline_follows_the_weighted_first_panel() {
    let mut moved = panel();
    moved[1].prob = 0.9;
    let want = (2.0 * 0.8 + 0.5 * 0.9 + 1.5 * 0.6) / 4.0;
    assert!(close(extra_round_baseline(&moved).unwrap(), want));
    assert_ne!(extra_round_baseline(&moved), extra_round_baseline(&panel()));
    let mut weightless = panel();
    weightless[3].prob = 0.95;
    assert_eq!(
        extra_round_baseline(&weightless),
        extra_round_baseline(&panel())
    );
    assert_eq!(
        first_panel_baselines(&weightless)[..3],
        first_panel_baselines(&panel())[..3]
    );
}

/// A1 C5: no weight left is no baseline and no score, never the reviewer's own forecast or 0.
#[test]
fn no_weight_left_is_no_baseline() {
    let lone = [Forecast {
        prob: 0.7,
        weight: 1.0,
    }];
    assert_eq!(first_panel_baselines(&lone), vec![None]);
    let weightless = [
        Forecast {
            prob: 0.7,
            weight: 0.0,
        },
        Forecast {
            prob: 0.2,
            weight: 0.0,
        },
    ];
    assert_eq!(first_panel_baselines(&weightless), vec![None, None]);
    assert_eq!(extra_round_baseline(&weightless), None);
    assert_eq!(
        item_scores(&weightless, &[0.4], 0.0),
        ItemScores {
            first: vec![None, None],
            extra: vec![None]
        }
    );
    let pair = [
        Forecast {
            prob: 0.7,
            weight: 1.0,
        },
        Forecast {
            prob: 0.2,
            weight: 0.0,
        },
    ];
    assert_eq!(first_panel_baselines(&pair), vec![None, Some(0.7)]);
    assert_eq!(item_scores(&[], &[0.4], 1.0).extra, vec![None]);
}

/// A1 C5, D33: the difference score of both kinds of reviewer, by hand, outcome 0 and 1.
#[test]
fn both_kinds_of_reviewer_get_the_difference_score() {
    for o in [0.0, 1.0f64] {
        let scores = item_scores(&panel(), &[0.35], o);
        for (u, f) in panel().iter().enumerate() {
            let by_hand = (FIRST[u] - o).powi(2) - (f.prob - o).powi(2);
            assert!(close(scores.first[u].unwrap(), by_hand), "{u}, o = {o}");
        }
        assert!(close(
            scores.extra[0].unwrap(),
            (EXTRA - o).powi(2) - (0.35 - o).powi(2)
        ));
    }
}

/// A1 C5: a band item walked through the lifecycle to the pool, scored and recorded per kind.
#[test]
fn a_band_item_is_scored_through_the_lifecycle() {
    let standings = [
        ReviewerStanding::founder(),
        ReviewerStanding::established(0.02),
        ReviewerStanding::established(-0.05),
        ReviewerStanding::established(0.05),
        ReviewerStanding::founder(),
        ReviewerStanding::established(0.0),
        ReviewerStanding {
            is_founder: false,
            judgments_with_outcome: 3,
            skill: 0.0,
            reviews: 3,
            alarms: 0,
        },
    ];
    let weights = bridging_weights(&standings, epoch_weight_cap(&standings));
    assert_eq!(weights[6], 0.0);
    let probs = [0.75, 0.9, 0.6, 0.85, 0.7, 0.8, 0.2];
    let first: Vec<Judgment> = (0..7)
        .map(|u| Judgment {
            nym: Nym([u as u8 + 1; 32]),
            prob: probs[u],
            nonce: [u as u8; 32],
        })
        .collect();
    let item = cid(b"band item");
    let admitted = step(
        deposit(true, true, true, true).unwrap(),
        Event::Admit {
            seed_from_beacon: true,
        },
    )
    .unwrap();
    let revealed = review_round(
        admitted,
        item,
        first.iter().map(|j| j.nym).collect(),
        &first,
    )
    .unwrap();
    let State::Revealing { reveals, .. } = &revealed else {
        panic!("not revealing")
    };
    let forecasts: Vec<Forecast> = first
        .iter()
        .zip(&weights)
        .map(|(j, &weight)| {
            let prob = reveals.iter().find(|(n, _)| *n == j.nym).unwrap().1;
            Forecast { prob, weight }
        })
        .collect();
    let band = step(
        revealed,
        Event::Score {
            outcome: GateOutcome::SupplementaryReview,
        },
    )
    .unwrap();
    let extra: Vec<Judgment> = (0..4)
        .map(|k| Judgment {
            nym: Nym([40 + k; 32]),
            prob: 0.3 + 0.1 * k as f64,
            nonce: [k; 32],
        })
        .collect();
    let mut s = extra_round(
        band,
        &ExtraRound {
            panel: extra.iter().map(|j| j.nym).collect(),
            judgments: extra.clone(),
        },
    )
    .unwrap();
    let State::SupplementaryReview {
        reveals: extra_reveals,
        ..
    } = &s
    else {
        panic!("not in the band")
    };
    let extra_probs: Vec<f64> = extra_reveals.iter().map(|&(_, p)| p).collect();
    for event in [
        Event::Resolve {
            outcome: GateOutcome::Pass,
        },
        Event::Pilot1Batch {
            enough_respondents: true,
            screen: Screening::Pass,
        },
        Event::Pilot2Batch {
            batch_size: K_MIN,
            dif: Recheck::NoDif,
            source_verified: false,
        },
    ] {
        s = step(s, event).unwrap();
    }
    assert_eq!(s, State::ActivePool);
    let Scored::Observed(Observation { outcome, inclusion }) = outcome_of(&s, EXPLORATION_RATE)
    else {
        panic!()
    };
    assert_eq!((outcome, inclusion), (1.0, 1.0));
    let scores = item_scores(&forecasts, &extra_probs, outcome);
    let total: f64 = weights.iter().sum();
    let extra_baseline: f64 = weights.iter().zip(probs).map(|(w, p)| w * p).sum::<f64>() / total;
    for (k, &p) in extra_probs.iter().enumerate() {
        assert!(close(
            scores.extra[k].unwrap(),
            (extra_baseline - 1.0).powi(2) - (p - 1.0).powi(2)
        ));
    }
    for (who, score) in [
        (0usize, scores.first[0].unwrap()),
        (1, scores.extra[1].unwrap()),
    ] {
        let mut track = SkillTrack::new();
        record_outcome(
            &mut track,
            &s,
            EXPLORATION_RATE,
            |_| score,
            &CusumParams::default(),
        );
        assert_eq!(
            (track.skill(), track.scored()),
            (score, 1),
            "reviewer {who}"
        );
    }
}
