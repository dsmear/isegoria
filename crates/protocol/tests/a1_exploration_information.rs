//! A1 diagnostics (`docs/16`, `docs/15` A1): the exploration bit is a function of the public
//! beacon and the slot, known before a report is committed; under the simplified gate an
//! adaptive report beats the truth, and a draw the reports cannot read removes the gain.

mod common;

use identity::nym::Nym;
use network::cid::cid;
use protocol::exploration::{explore_from_beacon, outcome_of, Scored, EXPLORATION_RATE};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, RejectReason, State, K_MIN};
use protocol::orchestrator::{review_round, Judgment};
use protocol::probation::SkillTrack;
use protocol::review::{assign_from_beacon, Reviewer};
use scoring::reputation::{difference_score, inverse_probability_mean, CusumParams};

const Q: f64 = 0.4;
const B: f64 = 0.7;
const GATE: f64 = 0.5;
const EPS: f64 = EXPLORATION_RATE;

/// The simplified gate of `exploration_weights.rs`: the report alone decides.
fn passes(report: f64) -> bool {
    report >= GATE
}

/// The reviewer's IPW mean after one reviewed item, through `SkillTrack` (D35).
fn ipw_after(report: f64, explored: bool, outcome: f64) -> f64 {
    let mut track = SkillTrack::new();
    let params = CusumParams::default();
    if passes(report) || explored {
        let pi = if passes(report) { 1.0 } else { EPS };
        let d = difference_score(report, B, outcome);
        track.record_observed(d, pi, &params);
        assert_eq!(track.skill(), inverse_probability_mean(&[(d, pi)], 1));
    } else {
        track.record_unobserved();
    }
    track.skill()
}

/// The exact expectation over the draw (probability `EPS`) and the outcome (belief `q`) of a
/// report rule that may read the draw.
fn expected(q: f64, rule: impl Fn(bool) -> f64) -> f64 {
    let mut total = 0.0;
    for (explored, px) in [(true, EPS), (false, 1.0 - EPS)] {
        for (outcome, po) in [(1.0, q), (0.0, 1.0 - q)] {
            total += px * po * ipw_after(rule(explored), explored, outcome);
        }
    }
    total
}

/// A1: knowing the draw, reporting 0.4 when explored and 0.5 otherwise earns 0.166, not 0.09.
#[test]
fn knowing_the_draw_the_adaptive_report_beats_the_truth() {
    let truthful = expected(Q, |_| Q);
    let adaptive = expected(Q, |explored| if explored { Q } else { GATE });
    println!("truthful {truthful:.6}, adaptive {adaptive:.6}");
    assert!((truthful - 0.09).abs() < 1e-12, "{truthful}");
    assert!((adaptive - (EPS * (0.09 / EPS) + (1.0 - EPS) * 0.08)).abs() < 1e-12);
    assert!((adaptive - 0.166).abs() < 1e-12, "{adaptive}");
    let best_fixed = (0..=100)
        .map(|k| expected(Q, |_| k as f64 / 100.0))
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (best_fixed - truthful).abs() < 1e-12,
        "no report fixed before the draw beats it"
    );
}

/// A1: panel and bit both follow from the public beacon and slot; the machine takes the report.
#[test]
fn the_draw_is_known_to_the_panel_before_it_commits() {
    let beacon = common::beacon(7, 3);
    let candidates: Vec<Reviewer> = (0..40u8)
        .map(|i| Reviewer {
            nym: Nym([i; 32]),
            f_u: (i as f64 - 20.0) / 10.0,
        })
        .collect();
    for explored in [true, false] {
        let slot = (0..1000u64)
            .find(|&s| explore_from_beacon(&beacon, s, EPS) == explored)
            .expect("a slot of each kind");
        let panel = assign_from_beacon(&candidates, 9, &beacon, slot);
        let recomputed = common::beacon(7, 3);
        assert_eq!(explore_from_beacon(&recomputed, slot, EPS), explored);
        let report = if explore_from_beacon(&beacon, slot, EPS) {
            Q
        } else {
            GATE
        };
        let judgments: Vec<Judgment> = panel
            .iter()
            .map(|r| Judgment {
                nym: r.nym,
                prob: report,
                nonce: r.nym.0,
            })
            .collect();
        let admitted = step(
            deposit(true, true, true, true).unwrap(),
            Event::Admit {
                seed_from_beacon: true,
            },
        )
        .unwrap();
        let item = cid(&slot.to_le_bytes());
        let revealed = review_round(
            admitted,
            item,
            panel.iter().map(|r| r.nym).collect(),
            &judgments,
        )
        .expect("the machine cannot tell an adaptive report");
        let State::Revealing { reveals, .. } = revealed else {
            panic!("not revealing");
        };
        assert!(reveals.iter().all(|&(_, p)| p == report));
    }
}

/// A1, proposed sequence: a draw no rule can read gives every rule the full-information mean.
#[test]
fn a_draw_the_reports_cannot_read_restores_the_identity() {
    let signals = [(0.7, 0.25), (0.3, 0.65)];
    let truthful: f64 = signals.iter().map(|&(pz, q)| pz * (B - q).powi(2)).sum();
    let mut best = (f64::NEG_INFINITY, (0.0, 0.0));
    for a in 0..=20 {
        for c in 0..=20 {
            let rule = [a as f64 / 20.0, c as f64 / 20.0];
            let mut ipw = 0.0;
            let mut full = 0.0;
            for (z, &(pz, q)) in signals.iter().enumerate() {
                ipw += pz * expected(q, |_| rule[z]);
                full += pz * ((B - q).powi(2) - (rule[z] - q).powi(2));
            }
            assert!((ipw - full).abs() < 1e-12, "rule {rule:?}: {ipw} vs {full}");
            if ipw > best.0 + 1e-12 {
                best = (ipw, (rule[0], rule[1]));
            }
        }
    }
    assert!((best.0 - truthful).abs() < 1e-12);
    assert_eq!(best.1, (0.25, 0.65));
}

/// A1: the observed count (probation, shrinkage) still grows 1 on the gate's side, `ε` below.
#[test]
fn the_observed_count_still_depends_on_the_report() {
    let count = |report: f64| {
        let mut n = 0.0;
        for (explored, px) in [(true, EPS), (false, 1.0 - EPS)] {
            let mut track = SkillTrack::new();
            if passes(report) || explored {
                track.record_observed(
                    0.0,
                    if passes(report) { 1.0 } else { EPS },
                    &CusumParams::default(),
                );
            } else {
                track.record_unobserved();
            }
            n += px * track.scored() as f64;
        }
        n
    };
    assert_eq!(count(GATE), 1.0);
    assert!((count(Q) - EPS).abs() < 1e-15);
}

/// The inclusion `outcome_of` records for an item that the gate made appeal-eligible.
fn appeal_inclusion(appeal: bool, explored: bool) -> Option<f64> {
    let admitted = step(
        deposit(true, true, true, true).unwrap(),
        Event::Admit {
            seed_from_beacon: true,
        },
    )
    .unwrap();
    let panel: Vec<Nym> = (1..=7u8).map(|i| Nym([i; 32])).collect();
    let judgments: Vec<Judgment> = panel
        .iter()
        .map(|&nym| Judgment {
            nym,
            prob: 0.3,
            nonce: nym.0,
        })
        .collect();
    let reviewed = review_round(admitted, cid(b"divisive"), panel, &judgments).unwrap();
    let mut s = step(
        reviewed,
        Event::Score {
            outcome: GateOutcome::AppealEligible,
        },
    )
    .unwrap();
    s = if appeal {
        step(
            s,
            Event::Appeal {
                within_window: true,
                reputation_covers_stake: true,
            },
        )
        .unwrap()
    } else {
        let s = step(s, Event::AppealExpires).unwrap();
        assert_eq!(s, State::Rejected(RejectReason::Polarized));
        if !explored {
            return None;
        }
        step(
            s,
            Event::Explore {
                seed_from_beacon: true,
            },
        )
        .unwrap()
    };
    s = step(
        s,
        Event::Pilot1Batch {
            enough_respondents: true,
            passed: true,
        },
    )
    .unwrap();
    s = step(
        s,
        Event::Pilot2Batch {
            batch_size: K_MIN,
            passed: true,
            source_verified: false,
        },
    )
    .unwrap();
    match outcome_of(&s, EPS) {
        Scored::Observed(o) => Some(o.inclusion),
        other => panic!("{other:?}"),
    }
}

/// A1: an appeal reading the draw gives an honest reviewer `E[I/π]` = 1.95; one before it, 1.
#[test]
fn an_appeal_that_reads_the_draw_biases_an_honest_reviewer() {
    let weight = |appeal: bool, explored: bool| {
        appeal_inclusion(appeal, explored).map_or(0.0, |pi| 1.0 / pi)
    };
    let reads_the_draw = (1.0 - EPS) * weight(true, false) + EPS * weight(false, true);
    assert!((reads_the_draw - 1.95).abs() < 1e-12, "{reads_the_draw}");
    for rate in [0.0, 0.3, 1.0] {
        let before_the_draw = rate * weight(true, false)
            + (1.0 - rate) * (EPS * weight(false, true) + (1.0 - EPS) * weight(false, false));
        assert!(
            (before_the_draw - 1.0).abs() < 1e-12,
            "appeal rate {rate}: {before_the_draw}"
        );
    }
}
