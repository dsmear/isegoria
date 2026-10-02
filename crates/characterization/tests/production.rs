//! The harness reads the production verdict: on a batch the gates admit, its flags are those
//! of `revalidate_batch_latent`; on one they refuse, it records the refusal (`docs/13` §2).

use characterization::generate::{dif_batch, screen_batch, PROBE_SPREAD};
use characterization::grid::{engine_seed, DifDesign, Layout, ScreenDesign};
use characterization::run::{dif, gate_char, redecides, screen};
use identity::nym::Nym;
use protocol::admission::NullifierSet;
use protocol::gate::{MIN_COVERAGE, TAU};
use protocol::pilot::{PilotError, Screening, Templates};
use protocol::revalidation::{revalidate_batch_latent, Recheck};
use scoring::latent::Formats;

fn respondents(n: usize) -> NullifierSet {
    let mut set = NullifierSet::new();
    for i in 0..n {
        let mut id = [0u8; 32];
        id[..8].copy_from_slice(&(i as u64).to_le_bytes());
        set.spend(Nym(id)).unwrap();
    }
    set
}

/// The record's flags read with its `converged` field, as the production re-check reads them.
fn as_rechecks(flags: &[bool], converged: bool) -> Vec<Recheck> {
    flags
        .iter()
        .map(|&dif| match (converged, dif) {
            (false, _) => Recheck::Indeterminate,
            (true, true) => Recheck::Dif,
            (true, false) => Recheck::NoDif,
        })
        .collect()
}

/// An admitted batch: the same flags as the production re-check with the same engine seed.
#[test]
fn an_admitted_batch_gets_the_production_flags() {
    let d = DifDesign {
        n: 3000,
        anchors: 60,
        k: 4,
        layout: Layout::Campaign(2),
        delta: 0.9,
        ..DifDesign::default()
    };
    let seed = 24;
    let batch = dif_batch(&d, seed);
    let outcome = dif(&d, seed);
    assert!(outcome.admitted, "KR-20 {}", outcome.kr20);
    let production = revalidate_batch_latent(
        &respondents(3000),
        &batch.anchors,
        &batch.x,
        &Formats::open(d.anchors, d.k),
        &Templates::none(d.anchors, d.k),
        engine_seed(seed),
    );
    assert_eq!(
        Ok(as_rechecks(&outcome.flags, outcome.converged)),
        production
    );
    assert_eq!(outcome.roles, "++cc");
}

/// A batch whose anchors are unreliable is refused by production and recorded as refused.
#[test]
fn a_refused_batch_is_recorded_as_refused() {
    let d = DifDesign {
        n: 3000,
        anchors: 20,
        k: 2,
        ..DifDesign::default()
    };
    let batch = dif_batch(&d, 5);
    let outcome = dif(&d, 5);
    assert!(!outcome.admitted);
    assert!(matches!(
        revalidate_batch_latent(
            &respondents(3000),
            &batch.anchors,
            &batch.x,
            &Formats::open(d.anchors, d.k),
            &Templates::none(d.anchors, d.k),
            5
        ),
        Err(PilotError::UnreliableAnchors { .. })
    ));
}

/// A batch with a floor gets the production flags of its declared format (`docs/13` §8.2).
#[test]
fn a_batch_with_a_floor_gets_the_production_flags_of_its_format() {
    let d = DifDesign {
        n: 3000,
        anchors: 60,
        k: 4,
        layout: Layout::Campaign(2),
        delta: 0.9,
        guess: 0.1,
        options: 10,
        ..DifDesign::default()
    };
    let seed = 25;
    let batch = dif_batch(&d, seed);
    let outcome = dif(&d, seed);
    assert!(outcome.admitted, "KR-20 {}", outcome.kr20);
    let production = revalidate_batch_latent(
        &respondents(3000),
        &batch.anchors,
        &batch.x,
        &Formats::choice(d.anchors, d.k, 10),
        &Templates::none(d.anchors, d.k),
        engine_seed(seed),
    );
    assert_eq!(
        Ok(as_rechecks(&outcome.flags, outcome.converged)),
        production
    );
    assert!(
        outcome.floors.iter().all(|c| (0.02..0.25).contains(c)),
        "{:?}",
        outcome.floors
    );
}

/// The gate's code is `U` below `MIN_COVERAGE` whatever the score, else the production gate's.
#[test]
fn the_gate_s_code_sends_an_uncovered_item_to_review() {
    let (low, floor) = (MIN_COVERAGE - 1, MIN_COVERAGE);
    assert_eq!(gate_char(0.95, 0.0, low), 'U');
    assert_eq!(gate_char(0.95, 0.0, floor), 'P');
    assert_eq!(gate_char(TAU, 0.0, floor), 'S');
    assert_eq!(gate_char(0.5, 0.3, floor), 'A');
    assert_eq!(gate_char(0.5, 0.1, floor), 'R');
    assert!(redecides(0.5, low) && !redecides(0.5, floor));
    assert!(redecides(TAU - PROBE_SPREAD, floor) && !redecides(TAU + PROBE_SPREAD, floor));
}

/// A pilot's recorded verdicts are those of the production stage-1 gate on the same pilot.
#[test]
fn a_pilot_gets_the_production_screen() {
    let d = ScreenDesign {
        n: 300,
        anchors: 30,
        options: 5,
    };
    for seed in [8, 9] {
        let batch = screen_batch(&d, seed);
        let outcome = screen(&d, seed);
        let production = protocol::pilot::screen(
            &respondents(300),
            &batch.anchors,
            &batch.x,
            &Formats::choice(d.anchors, 10, d.options),
        );
        assert!(
            outcome.converged && outcome.kept[..5].contains(&true),
            "{outcome:?}"
        );
        let readings = outcome.kept.iter().map(|&kept| match kept {
            true => Screening::Pass,
            false => Screening::Fail,
        });
        assert_eq!(Ok(readings.collect()), production);
    }
}
