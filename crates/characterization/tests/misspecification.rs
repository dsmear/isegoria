//! T82 on the supplement's own batches (`docs/13` §8.7.2, `docs/01` D43): a skewed null batch
//! the model before D43 read as classes is one class, and a design's template pairs are refused.

use characterization::grid::{Cell, Study, Task};
use characterization::run::{admitted, run, Outcome};

const SKEWED: &str = "n=3000 a=60 k=8 lay=c0 d=0 al=0 pi=0.5 im=0 g=0.2 atk=none m=5 sk=-4";

fn cell(key: &str) -> Cell {
    Cell::parse(Study::FloorMisspec, key).expect("a floor-misspec cell")
}

/// The flags of a replicate of a floor-misspec cell, and its number of classes.
fn fitted(key: &str, replicate: u32) -> (usize, Vec<bool>) {
    let task = Task {
        study: Study::FloorMisspec,
        cell: cell(key),
        replicate,
    };
    match run(&task) {
        Outcome::Dif(o) => (o.classes, o.flags),
        _ => panic!("a DIF outcome"),
    }
}

/// AT-DIF-14: replicate 0 of the skewed null cell, two classes and three flags before D43.
#[test]
fn at_dif_14_a_skewed_batch_read_as_classes_before_d43_is_one_class() {
    assert_eq!(fitted(SKEWED, 0), (1, vec![false; 8]));
}

/// AT-DIF-14: replicates 4 and 5 of the cell, which the model before D43 flagged too.
#[cfg(feature = "calibration")]
#[test]
fn at_dif_14_the_other_skewed_batches_read_as_classes_are_one_class() {
    for replicate in [4, 5] {
        assert_eq!(
            fitted(SKEWED, replicate),
            (1, vec![false; 8]),
            "{replicate}"
        );
    }
}

/// AT-DIF-15: the gates admit a reliable batch unless its trial items come in template pairs.
#[test]
fn at_dif_15_a_design_s_template_pairs_are_not_admitted() {
    let Cell::Dif(pairs) = cell(&format!("{SKEWED} tl=1")) else {
        panic!("a DIF cell")
    };
    let Cell::Dif(single) = cell(SKEWED) else {
        panic!("a DIF cell")
    };
    assert!(admitted(&single, 0.95));
    assert!(!admitted(&pairs, 0.95));
    assert!(!admitted(&single, 0.85));
}
