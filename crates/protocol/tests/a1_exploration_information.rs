//! A1 diagnostics (`docs/16`, `docs/15` A1): the exploration bit is a function of the public
//! beacon and the slot, known before a report is committed; under the simplified gate an
//! adaptive report beats the truth, and a draw the reports cannot read removes the gain.

mod common;

use identity::nym::Nym;
use network::beacon::BeaconRound;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use protocol::exploration::{explore_from_beacon, outcome_of, Scored, EXPLORATION_RATE};
use protocol::gate::GateOutcome;
use protocol::lifecycle::{deposit, step, Event, RejectReason, State, K_MIN};
use protocol::orchestrator::{review_round, Judgment};
use protocol::pilot::Screening;
use protocol::probation::{SkillTrack, N_PROBATION};
use protocol::randomness::Beacon;
use protocol::revalidation::Recheck;
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
            screen: Screening::Pass,
        },
    )
    .unwrap();
    s = step(
        s,
        Event::Pilot2Batch {
            batch_size: K_MIN,
            dif: Recheck::NoDif,
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

/// The beacon of one round of `members` at threshold `t` when exactly `revealers` reveal.
fn round_with(members: &[Member], t: usize, revealers: &[usize]) -> Option<Beacon> {
    let consortium = Consortium::new(members.iter().map(Member::public).collect(), t);
    let mut round = BeaconRound::open(&consortium, [7; 32], 11);
    let reveals: Vec<_> = members
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let (commit, reveal) = m.beacon_commit(round.id(), i);
            round.commit(&commit).expect("an honest commit");
            reveal
        })
        .collect();
    round.close_commits().expect("the commit deadline");
    round.close_deposits().expect("the deposit deadline");
    for &i in revealers {
        round.reveal(&reveals[i]).expect("a valid reveal");
    }
    Beacon::from_outcome(&round.finish().expect("the reveal deadline"))
}

fn members(n: u8) -> Vec<Member> {
    (1..=n).map(|i| Member::from_seed([i; 32])).collect()
}

/// A1: two colluders after three honest reveals pick among four beacons; one decides a slot.
#[test]
fn two_colluders_after_the_honest_reveals_choose_among_four_beacons() {
    let five = members(5);
    let beacons: Vec<Beacon> = [vec![], vec![3], vec![4], vec![3, 4]]
        .iter()
        .map(|own| {
            let revealers: Vec<usize> = [0, 1, 2].iter().chain(own).copied().collect();
            round_with(&five, 3, &revealers).expect("at least t reveals")
        })
        .collect();
    let mut values: Vec<u64> = beacons.iter().map(|b| b.seed(b"probe", 0)).collect();
    values.sort_unstable();
    values.dedup();
    assert_eq!(values.len(), 4, "four distinct valid beacons");
    let bits = |slot: u64| -> Vec<bool> {
        beacons
            .iter()
            .map(|b| explore_from_beacon(b, slot, EPS))
            .collect()
    };
    let slot = (0..10_000u64)
        .find(|&s| bits(s).contains(&true) && bits(s).contains(&false))
        .expect("a slot the colluders decide");
    println!(
        "slot {slot}: the four candidates explore it as {:?}",
        bits(slot)
    );
}

/// A1: with two honest reveals at `t = 3`, two colluders choose among three beacons or none.
#[test]
fn colluders_who_can_block_choose_a_beacon_or_none() {
    let four = members(4);
    assert!(
        round_with(&four, 3, &[0, 1]).is_none(),
        "no beacon: the fallback"
    );
    let mut values: Vec<u64> = [vec![0, 1, 2], vec![0, 1, 3], vec![0, 1, 2, 3]]
        .iter()
        .map(|r| {
            round_with(&four, 3, r)
                .expect("t reveals")
                .seed(b"probe", 0)
        })
        .collect();
    values.sort_unstable();
    values.dedup();
    assert_eq!(values.len(), 3);
}

/// `P(Binomial(n, p) ≥ k)`, summed exactly.
fn at_least(n: u32, p: f64, k: u32) -> f64 {
    let mut pmf = (1.0 - p).powi(n as i32);
    let mut tail = 0.0;
    for x in 0..=n {
        if x >= k {
            tail += pmf;
        }
        pmf *= (n - x) as f64 / (x + 1) as f64 * p / (1.0 - p);
    }
    tail
}

/// A1: `Σ I/π` gains 1 a judgment on average on both paths yet crosses `N_PROBATION` unalike.
#[test]
fn an_ipw_count_does_not_cross_the_threshold_alike() {
    let needed = (N_PROBATION as f64 * EPS).ceil() as u32;
    assert_eq!(needed, 2, "two explorations of weight 1/ε reach 30");
    for (n, explored) in [(10u32, 0.086_138), (30, 0.446_458), (60, 0.808_447)] {
        let ordinary = if n as usize >= N_PROBATION { 1.0 } else { 0.0 };
        let p = at_least(n, EPS, needed);
        println!("{n} judgments: entering {ordinary}, explored {p:.6}; mean count {n} on both");
        assert!((p - explored).abs() < 5e-7, "{n}: {p}");
        let closed =
            1.0 - (1.0 - EPS).powi(n as i32) - n as f64 * EPS * (1.0 - EPS).powi(n as i32 - 1);
        assert!((p - closed).abs() < 1e-12);
        assert!((ordinary - p).abs() > 0.08);
    }
    assert!(
        at_least(60, EPS, N_PROBATION as u32) < 1e-20,
        "observed count, explored path"
    );
}
