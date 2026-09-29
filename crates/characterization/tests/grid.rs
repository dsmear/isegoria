//! The grids of the characterization studies are the ones `docs/13` §4 and §8 specify, every
//! run has its own seed, and a larger replicate count extends a smaller one (`docs/13` §2).

use characterization::grid::{
    cells, parse_studies, replicates, tasks, Cell, DifDesign, Grid, Study, SweepDesign, STUDIES,
    SUPPLEMENT, T24,
};
use std::collections::BTreeSet;

/// The full grid has the cells and replicates of the `docs/13` §4 and §8 tables.
#[test]
fn the_full_grid_has_the_cells_the_specification_lists() {
    let expect = [
        (Study::DifNull, 27, 200),
        (Study::DifPower, 140, 200),
        (Study::DifMisspec, 22, 200),
        (Study::DifNonuniform, 16, 200),
        (Study::DifTwoAxes, 4, 200),
        (Study::DifPoisoning, 15, 200),
        (Study::DifPoolScale, 4, 3),
        (Study::DtfError, 8, 200),
        (Study::BridgingSweep, 36, 200),
        (Study::BridgingCapture, 4, 200),
        (Study::FloorNull, 12, 200),
        (Study::FloorPower, 23, 100),
        (Study::FloorMisspec, 18, 100),
        (Study::FloorDtf, 6, 200),
        (Study::BridgingLambda, 48, 200),
        (Study::BridgingExtra, 12, 200),
    ];
    assert_eq!(expect.len(), STUDIES.len());
    for (study, n_cells, reps) in expect {
        assert_eq!(cells(study, Grid::Full).len(), n_cells, "{}", study.name());
        assert_eq!(replicates(study, Grid::Full), reps, "{}", study.name());
        assert!(!cells(study, Grid::Smoke).is_empty(), "{}", study.name());
    }
    assert_eq!(tasks(&T24, Grid::Full, None, None).len(), 54_412);
    assert_eq!(tasks(&SUPPLEMENT, Grid::Full, None, None).len(), 19_700);
    let groups: BTreeSet<Study> = T24.into_iter().chain(SUPPLEMENT).collect();
    assert_eq!(groups, STUDIES.into_iter().collect());
}

/// A key names the floor fields only when set; an explicit default or one option is refused.
#[test]
fn the_floor_fields_of_a_key_are_named_only_when_set() {
    let plain = Cell::Dif(DifDesign::default()).key();
    assert!(!plain.contains(" m=") && !plain.contains(" gs="), "{plain}");
    let floored = Cell::Dif(DifDesign {
        guess: 0.2,
        options: 5,
        spread: 0.1,
        skew: -4.0,
        testlet: 0.5,
        ..DifDesign::default()
    });
    let key = floored.key();
    assert!(
        key.ends_with(" g=0.2 atk=none m=5 gs=0.1 sk=-4 tl=0.5"),
        "{key}"
    );
    assert_eq!(Cell::parse(Study::FloorMisspec, &key), Some(floored));
    for explicit in [" m=0", " m=1", " gs=0", " sk=0", " tl=0"] {
        let key = format!("{plain}{explicit}");
        assert_eq!(Cell::parse(Study::FloorNull, &key), None, "{key}");
    }
    let tuned = Cell::Sweep(SweepDesign {
        n: 100,
        share: 0.6,
        per_reviewer: 5,
        noise: 0.15,
        lambda: Some((0.15, 0.03)),
    });
    assert!(tuned.key().ends_with(" lb=0.15 lf=0.03"), "{}", tuned.key());
    assert_eq!(
        Cell::parse(Study::BridgingLambda, &tuned.key()),
        Some(tuned)
    );
}

/// The first pass's additions: power with 20 and 40 anchors, non-uniform DIF on three items.
#[test]
fn the_grid_holds_the_cells_the_first_pass_added() {
    let keys =
        |study| -> BTreeSet<String> { cells(study, Grid::Full).iter().map(Cell::key).collect() };
    let power = keys(Study::DifPower);
    for (a, lay, d) in [(20, 2, 0.7), (40, 3, 0.9)] {
        let key = format!("n=3000 a={a} k=8 lay=c{lay} d={d} al=0 pi=0.5 im=0 g=0 atk=none");
        assert!(power.contains(&key), "{key}");
    }
    let nonuniform = keys(Study::DifNonuniform);
    for (n, d, al) in [(3000, 0.0, 1.6), (6000, 0.9, 0.8)] {
        let key = format!("n={n} a=60 k=8 lay=c3 d={d} al={al} pi=0.5 im=0 g=0 atk=none");
        assert!(nonuniform.contains(&key), "{key}");
    }
}

/// A cell's key names it uniquely and parses back to it, in both grids.
#[test]
fn every_cell_key_parses_back_to_its_cell() {
    for study in STUDIES {
        assert_eq!(Study::parse(study.name()), Some(study));
        for grid in [Grid::Full, Grid::Smoke] {
            let all = cells(study, grid);
            let keys: BTreeSet<String> = all.iter().map(Cell::key).collect();
            assert_eq!(
                keys.len(),
                all.len(),
                "{} {grid:?}: a repeated key",
                study.name()
            );
            for cell in all {
                assert_eq!(
                    Cell::parse(study, &cell.key()),
                    Some(cell),
                    "{}",
                    cell.key()
                );
            }
        }
    }
    assert_eq!(Cell::parse(Study::DifNull, "n=3000 nonsense"), None);
}

/// Every run of the full grid has its own seed, and runs come replicate by replicate.
#[test]
fn every_run_has_its_own_seed_and_runs_come_replicate_first() {
    let all = tasks(&STUDIES, Grid::Full, None, None);
    let seeds: BTreeSet<u64> = all.iter().map(|t| t.seed()).collect();
    assert_eq!(seeds.len(), all.len());
    assert!(all.windows(2).all(|w| w[0].replicate <= w[1].replicate));
}

/// Twenty replicates are the first twenty of two hundred, run for run and seed for seed.
#[test]
fn a_larger_replicate_count_extends_a_smaller_one() {
    let few = tasks(&[Study::DifPower], Grid::Full, Some(20), None);
    let many = tasks(&[Study::DifPower], Grid::Full, Some(200), None);
    assert_eq!(few.len(), 140 * 20);
    let many: BTreeSet<(String, u32, u64)> = many
        .iter()
        .map(|t| (t.cell.key(), t.replicate, t.seed()))
        .collect();
    assert!(few
        .iter()
        .all(|t| many.contains(&(t.cell.key(), t.replicate, t.seed()))));
}

/// A study named twice runs once.
#[test]
fn a_study_named_twice_runs_once() {
    let twice = tasks(&[Study::DifNull, Study::DifNull], Grid::Full, Some(1), None);
    assert_eq!(twice.len(), 27);
}

/// A filter keeps the cells whose key contains it.
#[test]
fn a_filter_keeps_the_matching_cells() {
    let kept = tasks(&[Study::DifNull], Grid::Full, Some(1), Some("n=6000"));
    assert_eq!(kept.len(), 9);
    assert!(kept.iter().all(|t| t.cell.key().contains("n=6000")));
}

/// `--study` takes names and the groups `t24`, `t25` and `all`, and refuses an unknown name.
#[test]
fn a_study_list_names_studies_and_groups() {
    assert_eq!(parse_studies("t24"), Ok(T24.to_vec()));
    assert_eq!(parse_studies("t25"), Ok(SUPPLEMENT.to_vec()));
    assert_eq!(parse_studies("all"), Ok(STUDIES.to_vec()));
    assert_eq!(
        parse_studies("floor-null,t24"),
        Ok([&[Study::FloorNull][..], &T24].concat())
    );
    assert!(parse_studies("floor-nul").is_err());
}
