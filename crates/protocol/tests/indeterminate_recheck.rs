//! A latent re-check whose fit did not converge is indeterminate, never a clean verdict, and
//! leaves the item where it was (`docs/15` A4, `docs/02` §B.3, `docs/05` [8]).

use identity::nym::Nym;
use protocol::admission::NullifierSet;
use protocol::exposure::RetirementReason;
use protocol::lifecycle::{step, Event, State};
use protocol::pilot::{PilotError, Templates};
use protocol::revalidation::{
    revalidate_batch_latent, target_flags, target_rechecks, Recheck, N_LATENT_MIN,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::dif::MIXTURE_DIF_MAX;
use scoring::latent::{latent_dif, DifFlags, Formats, LatentDif};
use scoring::Convergence;

const K: usize = 8;

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// `n` respondents of one 2PL population on `na` open anchors and `K` open trial items.
fn batch(n: usize, na: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(0);
    let normal = |rng: &mut ChaCha8Rng| {
        let u1: f64 = 1.0 - rng.gen::<f64>();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * rng.gen::<f64>()).cos()
    };
    let b_anchor: Vec<f64> = (0..na).map(|_| normal(&mut rng)).collect();
    let (mut anchors, mut x) = (Vec::new(), Vec::new());
    for _ in 0..n {
        let t = normal(&mut rng);
        let mut answer = |p: f64| f64::from(rng.gen::<f64>() < p);
        let row: Vec<f64> = b_anchor
            .iter()
            .map(|b| answer(sigmoid(1.3 * (t - b))))
            .collect();
        anchors.push(row);
        x.push(
            (0..K)
                .map(|j| answer(sigmoid(1.2 * (t - 0.3 * j as f64))))
                .collect(),
        );
    }
    (anchors, x)
}

/// The production fit, `latent_dif` at its default settings, on [`batch`].
fn fitted(n: usize, na: usize) -> LatentDif {
    let (anchors, x) = batch(n, na);
    latent_dif(&anchors, &x, &Formats::open(na, K), 0).unwrap()
}

/// Thirty respondents on ten anchors: a fit that runs out of iterations.
fn unconverged() -> LatentDif {
    let fit = fitted(30, 10);
    assert_ne!(fit.status, Convergence::Converged);
    fit
}

fn revalidate(state: State, dif: Recheck, source_verified: bool) -> State {
    step(
        state,
        Event::Revalidate {
            dif,
            source_verified,
        },
    )
    .unwrap()
}

/// A4: an unconverged fit reads as indeterminate for every item, not as no DIF detected.
#[test]
fn an_unconverged_fit_is_indeterminate_not_a_clean_verdict() {
    let fit = unconverged();
    assert_eq!(target_flags(&fit), DifFlags::Indeterminate(fit.status));
    assert_eq!(target_rechecks(&fit), vec![Recheck::Indeterminate; K]);
}

/// A4: an unconverged re-check keeps either pool's item there, sourced or not; exposure retires it.
#[test]
fn an_indeterminate_re_check_leaves_the_item_where_it_was() {
    for dif in target_rechecks(&unconverged()) {
        for source_verified in [false, true] {
            for pool in [State::Contested, State::ActivePool] {
                let after = revalidate(pool.clone(), dif, source_verified);
                assert_eq!(after, pool, "{dif:?}, source verified {source_verified}");
                assert_eq!(
                    step(after, Event::ExposureLimit),
                    Ok(State::Retired(RetirementReason::Exposure))
                );
            }
        }
    }
}

/// A4: converged re-checks read and move items as before, the one-class fit included.
#[test]
fn a_converged_re_check_keeps_its_verdicts_and_transitions() {
    let one_class = fitted(60, 5);
    assert_eq!(one_class.status, Convergence::Converged);
    assert_eq!(one_class.classes, 1);
    let clean = target_rechecks(&one_class);
    assert_eq!(clean, vec![Recheck::NoDif; K]);
    for source_verified in [false, true] {
        for pool in [State::Contested, State::ActivePool] {
            assert_eq!(
                revalidate(pool, clean[0], source_verified),
                State::ActivePool
            );
        }
    }

    // A converged two-class verdict, its gaps set by the test: two items over the cut.
    let mut mixture = one_class.clone();
    mixture.classes = 2;
    mixture.dif = (0..K)
        .map(|j| if j < 2 { 2.0 } else { 0.5 } * MIXTURE_DIF_MAX)
        .collect();
    let flagged = target_rechecks(&mixture);
    assert_eq!(flagged[..2], [Recheck::Dif; 2]);
    assert_eq!(flagged[2..], [Recheck::NoDif; K - 2]);
    for pool in [State::Contested, State::ActivePool] {
        assert_eq!(revalidate(pool.clone(), flagged[0], true), State::Contested);
        assert_eq!(
            revalidate(pool.clone(), flagged[0], false),
            State::Retired(RetirementReason::EmergingDif)
        );
        assert_eq!(revalidate(pool, flagged[2], false), State::ActivePool);
    }
}

/// A4: a batch below the gate's floors is refused as an input error, not read as indeterminate.
#[test]
fn a_refused_batch_is_an_error_not_an_indeterminate_re_check() {
    let (anchors, x) = batch(30, 10);
    let mut people = NullifierSet::new();
    for i in 0..30u8 {
        people.spend(Nym([i; 32])).unwrap();
    }
    assert_eq!(
        revalidate_batch_latent(
            &people,
            &anchors,
            &x,
            &Formats::open(10, K),
            &Templates::none(10, K),
            0
        ),
        Err(PilotError::NotEnoughRespondents {
            have: 30,
            need: N_LATENT_MIN
        })
    );
}
