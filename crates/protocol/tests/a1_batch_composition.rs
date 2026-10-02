//! A1, C3 (`docs/17`): one target's latent DIF verdict read in two admissible batches that
//! share its respondents and its answers and differ only in the other trial items.

use protocol::exploration::{outcome_of, Observation, Scored, EXPLORATION_RATE};
use protocol::lifecycle::{step, Event, RejectReason, State};
use protocol::pilot::{stage1_verdicts, Stage1Fit};
use scoring::latent::Formats;
use scoring::Convergence;

#[cfg(feature = "calibration")]
mod fitted {
    use identity::nym::Nym;
    use protocol::admission::NullifierSet;
    use protocol::pilot::Templates;
    use protocol::revalidation::{latent_batch, target_rechecks, Recheck, N_LATENT_MIN};
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use scoring::latent::Formats;

    fn sigmoid(z: f64) -> f64 {
        1.0 / (1.0 + (-z).exp())
    }

    /// Items `(a, b, lean)` answered by two equal classes `z = ±1` at `b − lean·z`, with 60
    /// anchors; every respondent answers every item. Returns (anchors, responses).
    fn population(items: &[(f64, f64, f64)], seed: u64) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let normal = |rng: &mut ChaCha8Rng| {
            let u1: f64 = 1.0 - rng.gen::<f64>();
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * rng.gen::<f64>()).cos()
        };
        let aa: Vec<f64> = (0..60).map(|_| rng.gen_range(0.9..1.6)).collect();
        let ba: Vec<f64> = (0..60).map(|_| normal(&mut rng)).collect();
        let (mut anchors, mut x) = (Vec::new(), Vec::new());
        for _ in 0..N_LATENT_MIN {
            let (t, z) = (normal(&mut rng), if rng.gen::<bool>() { 1.0 } else { -1.0 });
            let mut bit = |p: f64| f64::from(rng.gen::<f64>() < p);
            anchors.push((0..60).map(|j| bit(sigmoid(aa[j] * (t - ba[j])))).collect());
            x.push(
                items
                    .iter()
                    .map(|&(a, b, lean)| bit(sigmoid(a * (t - b + lean * z))))
                    .collect(),
            );
        }
        (anchors, x)
    }

    /// The target's (column 0's) reading in the batch made of `columns`.
    fn target_in(anchors: &[Vec<f64>], x: &[Vec<f64>], columns: &[usize]) -> Recheck {
        let mut people = NullifierSet::new();
        for i in 0..N_LATENT_MIN {
            let mut id = [0u8; 32];
            id[..8].copy_from_slice(&(i as u64).to_le_bytes());
            people.spend(Nym(id)).unwrap();
        }
        let batch: Vec<Vec<f64>> = x
            .iter()
            .map(|row| columns.iter().map(|&c| row[c]).collect())
            .collect();
        let k = columns.len();
        let fit = latent_batch(
            &people,
            anchors,
            &batch,
            &Formats::open(60, k),
            &Templates::none(60, k),
            0,
        )
        .expect("the gates admit the batch");
        target_rechecks(&fit)[0]
    }

    /// A1 C3: the same target, answers and respondents, read with leaning or with clean companions.
    #[test]
    fn a_target_s_verdict_depends_on_its_batch() {
        let items = [
            (1.25, 0.0, 0.9),
            (1.2, 0.8, 0.9),
            (1.3, 0.1, 0.9),
            (1.15, -0.7, 0.9),
            (1.1, 0.4, 0.0),
            (1.4, -0.3, 0.0),
            (1.0, 0.9, 0.0),
            (1.3, -1.0, 0.0),
            (1.2, 0.2, 0.0),
            (1.35, -0.5, 0.0),
            (1.05, 0.6, 0.0),
        ];
        let (anchors, x) = population(&items, 83);
        let campaign = target_in(&anchors, &x, &[0, 1, 2, 3, 4, 5, 6, 7]);
        let alone = target_in(&anchors, &x, &[0, 4, 5, 6, 7, 8, 9, 10]);
        println!("target with three leaning companions: {campaign:?}; with clean ones: {alone:?}");
        assert_ne!(campaign, alone, "this realization shows no dependence");
    }
}

/// A1 C2: composed as the APIs allow, a stage-1 fit that did not converge scores outcome 0.
#[test]
fn an_unconverged_screen_composes_into_outcome_zero() {
    let fit = Stage1Fit {
        status: Convergence::MaxIters,
        rpb: vec![0.5; 2],
        a: vec![1.2; 2],
        b: vec![0.0; 2],
        c: vec![0.0; 2],
    };
    let kept = stage1_verdicts(&fit, &Formats::open(2, 2));
    assert_eq!(kept, vec![false; 2]);
    let screened = step(
        State::Pilot1 { appealed: false },
        Event::Pilot1Batch {
            enough_respondents: true,
            passed: kept[0],
        },
    )
    .unwrap();
    assert_eq!(screened, State::Rejected(RejectReason::Screen));
    assert_eq!(
        outcome_of(&screened, EXPLORATION_RATE),
        Scored::Observed(Observation {
            outcome: 0.0,
            inclusion: 1.0
        })
    );
}
