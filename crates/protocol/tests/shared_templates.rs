//! One template per batch (`docs/01` D43, `docs/05` [7]): the latent re-check refuses a batch
//! in which two columns, anchors included, come from the same template (AT-DIF-15).

use identity::nym::Nym;
use protocol::admission::NullifierSet;
use protocol::exposure::Template;
use protocol::pilot::{PilotError, Templates};
use protocol::revalidation::{revalidate_batch_latent, N_LATENT_MIN};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use scoring::latent::Formats;

const K: usize = 8;
const ANCHORS: usize = 20;

fn respondents(n: usize) -> NullifierSet {
    let mut set = NullifierSet::new();
    for i in 0..n {
        let mut id = [0u8; 32];
        id[..8].copy_from_slice(&(i as u64).to_le_bytes());
        set.spend(Nym(id)).unwrap();
    }
    set
}

/// A batch whose 20 anchors are too few to pass `KR20_MIN`: a refusal for unreliable anchors
/// shows the template check passed, without a fit.
fn batch(seed: u64) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let row = |rng: &mut ChaCha8Rng, len: usize| -> Vec<f64> {
        (0..len).map(|_| f64::from(rng.gen::<bool>())).collect()
    };
    let anchors = (0..N_LATENT_MIN).map(|_| row(&mut rng, ANCHORS)).collect();
    let x = (0..N_LATENT_MIN).map(|_| row(&mut rng, K)).collect();
    (anchors, x)
}

fn check(templates: &Templates) -> Result<Vec<bool>, PilotError> {
    let (anchors, x) = batch(1500);
    let formats = Formats::choice(ANCHORS, K, 4);
    revalidate_batch_latent(
        &respondents(N_LATENT_MIN),
        &anchors,
        &x,
        &formats,
        templates,
        0,
    )
}

fn id(structure: &str) -> Template {
    Template::new(structure.as_bytes().to_vec())
}

/// AT-DIF-15: two trial items of one template are refused, whatever else holds.
#[test]
fn at_dif_15_two_items_of_one_template_are_refused() {
    let shared = id("how many deputies sit in the chamber?");
    let mut templates = Templates::none(ANCHORS, K);
    templates.items[2] = Some(shared.id());
    templates.items[5] = Some(shared.id());
    templates.items[6] = Some(id("another structure").id());
    assert_eq!(
        check(&templates),
        Err(PilotError::SharedTemplate {
            template: shared.id()
        })
    );
}

/// AT-DIF-15: an anchor and a trial item of one template are refused too.
#[test]
fn at_dif_15_an_anchor_and_an_item_of_one_template_are_refused() {
    let shared = id("in which year did the act take effect?");
    let mut templates = Templates::none(ANCHORS, K);
    templates.anchors[7] = Some(shared.id());
    templates.items[0] = Some(shared.id());
    assert_eq!(
        check(&templates),
        Err(PilotError::SharedTemplate {
            template: shared.id()
        })
    );
}

/// AT-DIF-15: distinct templates, or none, pass the check and meet the next gate.
#[test]
fn at_dif_15_distinct_templates_pass_the_check() {
    let mut templates = Templates::none(ANCHORS, K);
    for (j, slot) in templates.items.iter_mut().enumerate() {
        *slot = Some(id(&format!("structure {j}")).id());
    }
    templates.anchors[0] = Some(id("an anchor's own structure").id());
    for t in [Templates::none(ANCHORS, K), templates] {
        assert!(
            matches!(check(&t), Err(PilotError::UnreliableAnchors { .. })),
            "{:?}",
            check(&t)
        );
    }
}

/// Templates are refused unless one per anchor and per trial item.
#[test]
fn templates_that_do_not_describe_the_batch_are_refused() {
    for t in [
        Templates::none(ANCHORS - 1, K),
        Templates::none(ANCHORS, K + 1),
    ] {
        assert_eq!(check(&t), Err(PilotError::BadTemplates));
    }
}
