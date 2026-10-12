//! A12 frozen-table transport and first/supplementary consumers; docs/16 §7.2.

use identity::nym::Nym;
use protocol::affine_weights::{ExpectedAffineEpoch, FrozenAffineEpoch};
use protocol::orchestrator::{affine_first_ratings, affine_supplementary_ratings};
use sha2::{Digest, Sha256};

const FIRST: &[u8] = include_bytes!("fixtures/a12_affine/epoch-0.affine.tsv");
const SECOND: &[u8] = include_bytes!("fixtures/a12_affine/epoch-1.affine.tsv");
const MANIFEST: &str = "78291680ea5d05beb0adf7857a3daea687366c9b99484692f5b7b8ae50a80a51";
const SNAPSHOT: &str = "ef1890ffa314db3f73e7fe1dc0769c0b3585301bd08df82b0941546546cb5bf2";
const FIRST_SHA: &str = "5bce1f8250a423c8f782fa276f16f1ac9820eb4e9965acfb2576c5e87f161b95";
const SECOND_SHA: &str = "ba94ad86e3ef3fce3bb0b8dbfde03d14ac49d886c456bf05fe388aed25b2e47c";

fn expected<'a>(sha: &'a str, epoch: &'a str, cut: u64) -> ExpectedAffineEpoch<'a> {
    ExpectedAffineEpoch {
        artifact_sha256: sha,
        manifest_id: MANIFEST,
        snapshot_id: SNAPSHOT,
        epoch,
        cut,
    }
}
fn nym(n: u8) -> Nym {
    let mut b = [0; 32];
    b[31] = n;
    Nym(b)
}
fn imported() -> FrozenAffineEpoch {
    FrozenAffineEpoch::import(FIRST, &expected(FIRST_SHA, "e0", 100)).unwrap()
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn golden_first_and_supplementary_preserve_frozen_bits() {
    // AT-REP-01: actual exporter artifact supplies every first and supplementary row.
    let epoch = imported();
    let first = affine_first_ratings(
        &[vec![0.75], vec![0.25]],
        &[vec![true], vec![true]],
        &[nym(1), nym(2)],
        &epoch,
    )
    .unwrap();
    assert_eq!(
        first.ratings().weights.iter().map(|w| w.to_bits()).collect::<Vec<_>>(),
        vec![1.0f64.to_bits(), 0.0f64.to_bits()]
    );
    assert_eq!(first.ratings().axis, vec![true, false]);
    let extra = affine_supplementary_ratings(&first, &epoch, 0, &[(nym(3), 0.5), (nym(4), 1.0)])
        .unwrap();
    assert_eq!(first.ratings().n, 2);
    assert_eq!(extra.rows(), &[nym(1), nym(2), nym(3), nym(4)]);
    assert_eq!(extra.ratings().axis, vec![true, false, true, true]);
    for (index, id) in extra.rows().iter().enumerate() {
        assert_eq!(
            extra.ratings().weights[index].to_bits(),
            epoch.weight_bits(*id).unwrap()
        );
    }
    assert_eq!(extra.ratings().obs.len(), 4);
}

#[test]
fn second_epoch_promotes_complete_thirtieth_count() {
    // AT-REP-01: the same producer's next epoch promotes the 29-count probationer.
    let next = FrozenAffineEpoch::import(SECOND, &expected(SECOND_SHA, "e1", 120)).unwrap();
    let first = affine_first_ratings(&[vec![0.5]], &[vec![true]], &[nym(2)], &next).unwrap();
    assert_eq!(first.ratings().axis, vec![true]);
    assert_eq!(first.ratings().weights[0].to_bits(), next.weight_bits(nym(2)).unwrap());
    assert!(first.ratings().weights[0] > 0.0);
}

#[test]
fn existing_unrated_row_is_reused_once() {
    // AT-REP-01: an existing supplementary reviewer keeps its frozen row and bit weight.
    let epoch = imported();
    let first = affine_first_ratings(&[vec![0.5], vec![0.0]],
        &[vec![true], vec![false]], &[nym(1), nym(3)], &epoch).unwrap();
    let next = affine_supplementary_ratings(&first, &epoch, 0, &[(nym(3), 0.25)]).unwrap();
    assert_eq!(next.ratings().n, 2);
    assert_eq!(next.ratings().obs.len(), 2);
    assert_eq!(next.ratings().weights[1].to_bits(), 1.0f64.to_bits());
    assert!(affine_supplementary_ratings(&next, &epoch, 0, &[(nym(3), 0.75)]).is_err());
}

#[test]
fn epoch_context_and_shape_refusals_leave_base_intact() {
    // AT-REP-01: mismatched epoch, invalid panels and observations cannot mutate a frozen base.
    let epoch = imported();
    let next = FrozenAffineEpoch::import(SECOND, &expected(SECOND_SHA, "e1", 120)).unwrap();
    let first = affine_first_ratings(&[vec![0.5]], &[vec![true]], &[nym(1)], &epoch).unwrap();
    let bits = first.ratings().weights[0].to_bits();
    assert!(affine_supplementary_ratings(&first, &next, 0, &[(nym(3), 1.0)]).is_err());
    for reveals in [&[(nym(1), 0.2)][..], &[(nym(3), 0.2), (nym(3), 0.4)][..],
                    &[(nym(4), f64::NAN)][..], &[(nym(8), 0.1)][..]] {
        assert!(affine_supplementary_ratings(&first, &epoch, 0, reveals).is_err());
    }
    assert!(affine_supplementary_ratings(&first, &epoch, 1, &[]).is_err());
    assert!(affine_first_ratings(&[vec![0.3]], &[], &[nym(1)], &epoch).is_err());
    assert!(affine_first_ratings(&[vec![0.3], vec![]], &[vec![true], vec![]],
        &[nym(1), nym(2)], &epoch).is_err());
    assert!(affine_first_ratings(&[vec![0.3]], &[vec![true]], &[nym(8)], &epoch).is_err());
    assert!(affine_first_ratings(&[vec![0.3], vec![0.2]], &[vec![true], vec![true]],
        &[nym(1), nym(1)], &epoch).is_err());
    assert!(affine_first_ratings(&[vec![f64::NAN]], &[vec![true]], &[nym(1)], &epoch).is_err());
    assert_eq!(first.ratings().weights[0].to_bits(), bits);
    assert_eq!(first.ratings().obs.len(), 1);
}

#[test]
fn importer_checks_digest_context_and_closed_counts() {
    // AT-REP-01: the caller pins context and digest; malformed and extra fields are refused.
    assert!(FrozenAffineEpoch::import(FIRST, &expected(SECOND_SHA, "e0", 100)).is_err());
    assert!(FrozenAffineEpoch::import(FIRST, &expected(FIRST_SHA, "e1", 100)).is_err());
    assert!(FrozenAffineEpoch::import(FIRST, &expected(FIRST_SHA, "e0", 120)).is_err());
    let mut context = expected(FIRST_SHA, "e0", 100);
    context.snapshot_id = MANIFEST;
    assert!(FrozenAffineEpoch::import(FIRST, &context).is_err());
    let source = std::str::from_utf8(FIRST).unwrap();
    let row_one = format!("row\t{:064x}", 1);
    let row_two = format!("row\t{:064x}", 2);
    let rows: Vec<_> = source.lines().collect();
    let mut swapped = rows.clone();
    swapped.swap(1, 2);
    let swapped = swapped.join("\n") + "\n";
    for malformed in [
        source.replacen("\t3\t4\n", "\t3\t5\n", 1),
        source.replacen("\t3\t4\n", "\t3\t3\n", 1),
        source.replacen("\t3\t4\n", "\t3\t4\textra\n", 1),
        source.replacen("\t100\t4/5\t0", "\t18446744073709551616\t4/5\t0", 1),
        source.replacen("\t30\t1\t0\t1", "\t18446744073709551616\t1\t0\t1", 1),
        source.replacen("\t30\t1\t0\t1", "\t30\t18446744073709551616\t0\t1", 1),
        source.replacen(&row_two, &row_one, 1),
        swapped,
        format!("{}extra\n", source),
        source.replacen("3ff0000000000000", "7ff8000000000000", 1),
        source.replacen("3ff0000000000000", "3fd9999999999999", 1),
        source.replacen("0000000000000000\t0\t0", "8000000000000000\t0\t0", 1),
    ] {
        let sha = digest(malformed.as_bytes());
        assert!(FrozenAffineEpoch::import(malformed.as_bytes(), &expected(&sha, "e0", 100)).is_err());
    }
}
