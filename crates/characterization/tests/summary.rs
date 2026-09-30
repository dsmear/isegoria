//! The summaries compute what `docs/13` §5 defines, checked on hand-built records, and every
//! study of the smoke grid runs and is summarized (`docs/13` §2).

use characterization::grid::{
    tasks, CaptureDesign, Cell, DifDesign, ExtraDesign, Grid, Layout, ScreenDesign, Study,
    SweepDesign, STUDIES,
};
use characterization::record::{header, Record};
use characterization::run::{
    CaptureOutcome, DifOutcome, DtfOutcome, ExtraOutcome, Outcome, ScreenOutcome, SweepOutcome,
};
use characterization::runner::{execute, Options};
use characterization::stats::{clustered_rate, quantile, sd, wilson};
use characterization::summary::summarize;
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("characterization-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 5e-4
}

/// The Wilson interval and the type-7 quantile match hand-computed values.
#[test]
fn the_interval_and_the_quantile_match_hand_computed_values() {
    let (lo, hi) = wilson(0, 10);
    assert!(close(lo, 0.0) && close(hi, 0.2775), "{lo} {hi}");
    let (lo, hi) = wilson(5, 10);
    assert!(close(lo, 0.2366) && close(hi, 0.7634), "{lo} {hi}");
    let (lo, hi) = wilson(2, 3);
    assert!(close(lo, 0.2077) && close(hi, 0.9385), "{lo} {hi}");
    assert_eq!(wilson(0, 0), (0.0, 1.0));
    assert_eq!(quantile(&[1.0, 2.0, 3.0, 4.0], 0.5), 2.5);
    assert_eq!(quantile(&[1.0, 2.0, 3.0, 4.0], 0.25), 1.75);
    assert!(close(quantile(&[0.0, 1.2, 1.5], 0.95), 1.47));
    let inf = f64::INFINITY;
    assert_eq!(quantile(&[5.0, 10.0, inf], 0.5), 10.0);
    assert_eq!(quantile(&[5.0, inf], 0.5), inf);
    assert_eq!(quantile(&[inf, inf], 0.2), inf);
    let (p, lo, hi) = clustered_rate(&[(0, 4), (1, 4), (2, 4)]);
    assert!(
        close(p, 0.25) && close(lo, 0.0764) && close(hi, 0.5731),
        "{lo} {hi}"
    );
    let (_, lo, hi) = clustered_rate(&[(0, 4), (0, 16)]);
    assert!(close(lo, 0.0) && close(hi, 0.7232), "{lo} {hi}");
    assert!(clustered_rate(&[]).0.is_nan());
}

fn outcome(
    flags: [bool; 4],
    dif: [f64; 4],
    classes: usize,
    admitted: bool,
    roles: &str,
) -> Outcome {
    Outcome::Dif(DifOutcome {
        kr20: if admitted { 0.91 } else { 0.88 },
        admitted,
        classes,
        non_uniform: false,
        converged: true,
        bic_gain: 1.0,
        pi: vec![1.0],
        eta: vec![0.0],
        dif: dif.to_vec(),
        a_gap: vec![0.0; 4],
        flags: flags.to_vec(),
        roles: roles.to_string(),
        floors: Vec::new(),
        anchor_floor: 0.0,
    })
}

fn write(out: &Path, study: Study, cell: &str, outcomes: Vec<Outcome>) {
    let dir = out.join(study.name());
    fs::create_dir_all(&dir).unwrap();
    let mut text = format!("{}\n", header(study));
    for (i, outcome) in outcomes.into_iter().enumerate() {
        let record = Record {
            study,
            cell: cell.to_string(),
            replicate: i as u32,
            seed: i as u64,
            elapsed_ms: 1000 * (i as u64 + 1),
            outcome,
        };
        text.push_str(&record.line());
        text.push('\n');
    }
    fs::write(dir.join("records.csv"), text).unwrap();
}

/// A row of a study's summary table, by column name.
fn row(out: &Path, study: Study) -> Vec<(String, String)> {
    let text = fs::read_to_string(out.join(study.name()).join("summary.csv")).unwrap();
    let mut lines = text.lines();
    let names: Vec<String> = lines.next().unwrap().split(',').map(String::from).collect();
    let values: Vec<String> = lines.next().unwrap().split(',').map(String::from).collect();
    names.into_iter().zip(values).collect()
}

fn value(row: &[(String, String)], name: &str) -> f64 {
    let (_, v) = row
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("no {name}"));
    v.parse().unwrap()
}

/// Null runs: the batch and item false-positive rates, the admitted view and the gaps.
#[test]
fn the_null_summary_counts_false_positives_per_batch_and_per_item() {
    let out = scratch("null-summary");
    let cell = Cell::Dif(DifDesign {
        k: 4,
        ..DifDesign::default()
    })
    .key();
    let f = false;
    write(
        &out,
        Study::DifNull,
        &cell,
        vec![
            outcome([f, f, f, f], [0.0; 4], 1, true, "cccc"),
            outcome([true, f, f, f], [1.2, 0.1, 0.2, 0.3], 2, true, "cccc"),
            outcome([true, true, f, f], [1.5, 1.1, 0.4, 0.2], 2, false, "cccc"),
        ],
    );
    let markdown = summarize(&out).unwrap();
    assert!(markdown.contains("dif-null"));
    let r = row(&out, Study::DifNull);
    assert_eq!(value(&r, "runs"), 3.0);
    assert!(close(value(&r, "admitted"), 2.0 / 3.0));
    assert!(close(value(&r, "mixtures"), 2.0 / 3.0));
    assert!(close(value(&r, "batch_fp"), 2.0 / 3.0));
    assert!(close(value(&r, "batch_fp_lo"), 0.2077) && close(value(&r, "batch_fp_hi"), 0.9385));
    assert!(close(value(&r, "clean_fp"), 0.25));
    assert!(close(value(&r, "clean_fp_lo"), 0.0764) && close(value(&r, "clean_fp_hi"), 0.5731));
    assert!(close(value(&r, "admitted_batch_fp"), 0.5));
    assert!(close(value(&r, "admitted_batch_fp_lo"), 0.0945));
    assert!(close(value(&r, "max_clean_gap_median"), 1.2));
    assert!(close(value(&r, "max_clean_gap_p95"), 1.47));
    assert!(close(value(&r, "fit_seconds"), 2.0));
    assert!(
        markdown.contains("### The DIF cut on `DIF_j`"),
        "{markdown}"
    );
    let _ = fs::remove_dir_all(out);
}

/// Biased runs: item power, all biased items flagged, clean items flagged, the biased gap.
#[test]
fn the_power_summary_counts_detections_per_item_and_per_batch() {
    let out = scratch("power-summary");
    let cell = Cell::Dif(DifDesign {
        k: 4,
        layout: Layout::Campaign(2),
        delta: 0.9,
        ..DifDesign::default()
    })
    .key();
    let (t, f) = (true, false);
    write(
        &out,
        Study::DifPower,
        &cell,
        vec![
            outcome([t, t, f, f], [1.8, 1.6, 0.1, 0.1], 2, true, "++cc"),
            outcome([t, f, f, t], [1.4, 0.8, 0.2, 1.1], 2, true, "++cc"),
            outcome([f, f, f, f], [0.0; 4], 1, true, "++cc"),
        ],
    );
    summarize(&out).unwrap();
    let r = row(&out, Study::DifPower);
    assert!(close(value(&r, "power"), 0.5));
    assert!(close(value(&r, "all_biased"), 1.0 / 3.0));
    assert!(close(value(&r, "clean_fp"), 1.0 / 6.0));
    assert!(close(
        value(&r, "biased_gap_mean"),
        (1.8 + 1.6 + 1.4 + 0.8) / 6.0
    ));
    assert!(close(value(&r, "true_gap"), 1.8));
    assert!(close(value(&r, "biased_gap_median"), 1.5));
    let markdown = fs::read_to_string(out.join("summary.md")).unwrap();
    assert!(markdown.contains("| 1.50 (1.80) |"), "{markdown}");
    assert!(
        markdown.contains("### The DIF cut on `DIF_j`"),
        "{markdown}"
    );
    let _ = fs::remove_dir_all(out);
}

/// Capture runs: boosters to pass (τ + ε) and to the band (τ − ε), a run that never passes.
#[test]
fn the_capture_summary_reads_crossings_and_never() {
    let out = scratch("capture-summary");
    let cell = Cell::Capture(CaptureDesign {
        item: 7,
        own: 40,
        step: 5,
    })
    .key();
    let run = |robust: [f64; 3]| {
        Outcome::Capture(CaptureOutcome {
            opposing: vec![0, 5, 10],
            full: robust.to_vec(),
            robust: robust.to_vec(),
            plain: vec![0.6, 0.7, 0.8],
        })
    };
    let runs = vec![
        run([0.5, 0.83, 0.9]),
        run([0.5, 0.79, 0.85]),
        run([0.5, 0.6, 0.7]),
    ];
    write(&out, Study::BridgingCapture, &cell, runs);
    summarize(&out).unwrap();
    let r = row(&out, Study::BridgingCapture);
    assert_eq!(value(&r, "pass_median"), 10.0);
    assert_eq!(value(&r, "pass_p95"), f64::INFINITY);
    assert!(close(value(&r, "pass_never"), 1.0 / 3.0));
    assert_eq!(value(&r, "band_median"), 5.0);
    assert!(close(value(&r, "plain_at_zero"), 0.6));
    let _ = fs::remove_dir_all(out);
}

/// Every study of the smoke grid runs end to end and has a section in the summary.
#[test]
fn every_study_of_the_smoke_grid_runs_and_is_summarized() {
    let out = scratch("smoke");
    let all = tasks(&STUDIES, Grid::Smoke, Some(1), None);
    let report = execute(
        &all,
        &Options {
            jobs: 4,
            out: out.clone(),
            progress: false,
        },
    )
    .unwrap();
    assert_eq!((report.ran, report.failed), (all.len(), 0));
    let markdown = summarize(&out).unwrap();
    for study in STUDIES {
        assert!(
            markdown.contains(&format!("## {}", study.name())),
            "{}",
            study.name()
        );
        assert!(
            out.join(study.name()).join("summary.csv").exists(),
            "{}",
            study.name()
        );
    }
    assert!(out.join("summary.md").exists());
    let _ = fs::remove_dir_all(out);
}

/// The standard deviation is NaN for fewer than two values, none included.
#[test]
fn the_deviation_of_fewer_than_two_values_is_nan() {
    assert!(sd(&[]).is_nan() && sd(&[1.0]).is_nan());
    assert!(close(sd(&[1.0, 3.0]), 2f64.sqrt()));
}

/// Capture runs: a pass the score falls back from is not stable; the stable pass is read too.
#[test]
fn the_capture_summary_reads_the_stable_pass() {
    let out = scratch("capture-stable");
    let cell = Cell::Capture(CaptureDesign {
        item: 8,
        own: 0,
        step: 5,
    })
    .key();
    let run = |robust: [f64; 4]| {
        Outcome::Capture(CaptureOutcome {
            opposing: vec![0, 5, 10, 15],
            full: robust.to_vec(),
            robust: robust.to_vec(),
            plain: vec![0.6, 0.7, 0.8, 0.9],
        })
    };
    let runs = vec![
        run([0.5, 0.83, 0.9, 0.95]),
        run([0.5, 0.83, 0.79, 0.85]),
        run([0.5, 0.6, 0.7, 0.75]),
        run([0.5, 0.9, 0.95, 0.7]),
        run([0.5, 0.6, 0.85, 0.9]),
    ];
    write(&out, Study::BridgingCapture, &cell, runs);
    summarize(&out).unwrap();
    let r = row(&out, Study::BridgingCapture);
    assert_eq!(value(&r, "pass_median"), 5.0);
    assert_eq!(value(&r, "stable_median"), 15.0);
    assert!(close(value(&r, "stable_never"), 0.4));
    let _ = fs::remove_dir_all(out);
}

/// DTF runs: a set fitted over the tolerance while truly within it is a false refusal.
#[test]
fn the_dtf_summary_counts_false_refusals_as_well_as_false_admissions() {
    let out = scratch("dtf-refusals");
    let cell = Cell::Dif(DifDesign {
        layout: Layout::Mirror,
        delta: 0.9,
        ..DifDesign::default()
    })
    .key();
    let run = |fitted: f64, truth: f64| {
        let mut f = vec![f64::NAN; 8];
        let mut t = vec![f64::NAN; 8];
        (f[0], t[0]) = (fitted, truth);
        Outcome::Dtf(DtfOutcome {
            classes: 2,
            converged: true,
            flags: vec![false; 8],
            roles: "++++cccc".to_string(),
            fitted: f,
            truth: t,
        })
    };
    let runs = vec![
        run(0.12, 0.05),
        run(0.05, 0.12),
        run(0.05, 0.05),
        run(0.2, 0.2),
    ];
    write(&out, Study::DtfError, &cell, runs);
    summarize(&out).unwrap();
    let r = row(&out, Study::DtfError);
    assert!(close(value(&r, "false_admission"), 0.25));
    assert!(close(value(&r, "false_refusal"), 0.25));
    assert_eq!(value(&r, "within"), 2.0);
    assert!(close(value(&r, "refused_within"), 0.5));
    let _ = fs::remove_dir_all(out);
}

/// Sweep runs: items the gate sends to review for coverage are counted, and no leak is a dash.
#[test]
fn the_sweep_summary_counts_uncovered_items_and_dashes_a_missing_leak() {
    let out = scratch("sweep-uncovered");
    let cell = Cell::Sweep(SweepDesign {
        n: 100,
        share: 0.5,
        per_reviewer: 5,
        noise: 0.07,
        lambda: None,
    })
    .key();
    let run = Outcome::Sweep(SweepOutcome {
        converged: true,
        axis_corr: 0.9,
        q: vec![0.9, 0.9, 0.55, 0.55],
        lean: vec![0.0, 0.0, 0.8, -0.8],
        truth: vec![0.9, 0.9, 0.55, 0.55],
        full: vec![0.9, 0.9, 0.6, 0.5],
        robust: vec![0.88, 0.9, 0.6, 0.5],
        gap: vec![0.0, 0.0, 0.7, 0.7],
        gate: "PUAU".to_string(),
    });
    write(&out, Study::BridgingSweep, &cell, vec![run]);
    let markdown = summarize(&out).unwrap();
    let r = row(&out, Study::BridgingSweep);
    assert!(close(value(&r, "uncovered"), 0.5));
    let line = markdown
        .lines()
        .find(|l| l.starts_with(&format!("| {cell} |")))
        .unwrap();
    assert!(line.ends_with("| — |") && !line.contains("±"), "{line}");
    let _ = fs::remove_dir_all(out);
}

/// Threshold tables: the DIF cut reads the 60-anchor cells, the a-gap cut labels each layout.
#[test]
fn the_threshold_tables_label_every_cell_once() {
    let out = scratch("threshold-labels");
    let f = false;
    let power = |anchors| {
        Cell::Dif(DifDesign {
            anchors,
            layout: Layout::Campaign(2),
            delta: 0.9,
            ..DifDesign::default()
        })
        .key()
    };
    let nonuniform = |count| {
        Cell::Dif(DifDesign {
            layout: Layout::Campaign(count),
            alpha: 0.8,
            ..DifDesign::default()
        })
        .key()
    };
    let one = || {
        vec![outcome(
            [true, f, f, f],
            [1.2, 0.1, 0.1, 0.1],
            2,
            true,
            "+ccc",
        )]
    };
    for (study, cell) in [
        (Study::DifPower, power(60)),
        (Study::DifPower, power(20)),
        (Study::DifNonuniform, nonuniform(2)),
        (Study::DifNonuniform, nonuniform(3)),
    ] {
        let dir = out.join(study.name());
        let existing = fs::read_to_string(dir.join("records.csv")).unwrap_or_default();
        write(&out, study, &cell, one());
        let added = fs::read_to_string(dir.join("records.csv")).unwrap();
        let rows = added
            .lines()
            .skip(usize::from(!existing.is_empty()))
            .collect::<Vec<_>>();
        fs::write(dir.join("records.csv"), existing + &rows.join("\n") + "\n").unwrap();
    }
    let markdown = summarize(&out).unwrap();
    let header = |title: &str| {
        let at = markdown.find(title).unwrap();
        markdown[at..]
            .lines()
            .find(|l| l.starts_with("| cut |"))
            .unwrap()
            .to_string()
    };
    let dif_cut = header("### The DIF cut");
    assert_eq!(
        dif_cut.matches("power N=3000 δ=0.9").count(),
        1,
        "{dif_cut}"
    );
    let csv = fs::read_to_string(out.join("thresholds-dif-cut.csv")).unwrap();
    assert!(
        csv.contains(&power(60)) && !csv.contains(&power(20)),
        "{csv}"
    );
    let a_cut = header("### A cut on the discrimination gap");
    assert!(a_cut.contains("power N=3000 α=0.8, 2 items"), "{a_cut}");
    assert!(a_cut.contains("power N=3000 α=0.8, 3 items"), "{a_cut}");
    let _ = fs::remove_dir_all(out);
}

/// The rows of a study's summary table, each by column name.
fn rows(out: &Path, study: Study) -> Vec<Vec<(String, String)>> {
    let text = fs::read_to_string(out.join(study.name()).join("summary.csv")).unwrap();
    let mut lines = text.lines();
    let names: Vec<String> = lines.next().unwrap().split(',').map(String::from).collect();
    lines
        .map(|l| {
            names
                .iter()
                .cloned()
                .zip(l.split(',').map(String::from))
                .collect()
        })
        .collect()
}

/// Extra-round runs: the band per ε, and false passes and failures per extra panel size.
#[test]
fn the_extra_round_summary_reads_the_band_and_the_re_decisions() {
    let out = scratch("extra-summary");
    let cell = Cell::Extra(ExtraDesign {
        n: 100,
        share: 0.5,
        per_reviewer: 5,
        noise: 0.15,
        panel: 7,
    })
    .key();
    let run = Outcome::Extra(ExtraOutcome {
        truth: vec![0.75, 0.85, 0.84, 0.75],
        robust: vec![0.81, 0.79, 0.83, 0.70],
        gate: "SSPR".to_string(),
        redecided: ["PRP-", "RPP-", "RPP-", "RPP-", "RPP-", "RPP-"]
            .map(String::from)
            .to_vec(),
    });
    write(&out, Study::BridgingExtra, &cell, vec![run]);
    summarize(&out).unwrap();
    let all = rows(&out, Study::BridgingExtra);
    assert_eq!(all.len(), 3 * 6);
    let at = |eps: f64, k: usize| {
        all.iter()
            .find(|r| value(r, "eps") == eps && value(r, "k_extra") == k as f64)
            .unwrap()
    };
    assert!(close(value(at(0.02, 0), "band"), 0.5));
    assert!(close(value(at(0.04, 0), "band"), 0.75));
    assert!(close(value(at(0.02, 0), "false_pass"), 0.5));
    assert!(close(value(at(0.02, 0), "false_fail"), 0.5));
    assert!(close(value(at(0.02, 2), "false_pass"), 0.0));
    assert!(close(value(at(0.02, 2), "false_fail"), 0.0));
    assert!(close(value(at(0.04, 4), "extra_reviews"), 3.0));
    let _ = fs::remove_dir_all(out);
}

/// Several cells of one study in one records file, each run numbered from 0 within its cell.
fn write_cells(out: &Path, study: Study, cells: Vec<(String, Vec<Outcome>)>) {
    let dir = out.join(study.name());
    fs::create_dir_all(&dir).unwrap();
    let mut text = format!("{}\n", header(study));
    for (cell, outcomes) in cells {
        for (i, outcome) in outcomes.into_iter().enumerate() {
            let record = Record {
                study,
                cell: cell.clone(),
                replicate: i as u32,
                seed: i as u64,
                elapsed_ms: 1000,
                outcome,
            };
            text.push_str(&format!("{}\n", record.line()));
        }
    }
    fs::write(dir.join("records.csv"), text).unwrap();
}

/// The header of the first table after `title` in the summary.
fn table_header(markdown: &str, title: &str) -> String {
    let at = markdown.find(title).unwrap_or_else(|| panic!("no {title}"));
    markdown[at..]
        .lines()
        .find(|l| l.starts_with("| cell |") || l.starts_with("| cut |"))
        .unwrap()
        .to_string()
}

/// Floor runs: the fitted floors in each floor table, and the DIF cut per floor and format on
/// the nulls with 40 or more anchors and the two-leaner power cells of that format alone.
#[test]
fn the_floor_summary_reads_the_fitted_floors_and_the_cut_per_format() {
    let out = scratch("floor-summary");
    let key = |guess, options, change: &dyn Fn(DifDesign) -> DifDesign| {
        Cell::Dif(change(DifDesign {
            n: 6000,
            layout: Layout::Campaign(2),
            delta: 0.9,
            guess,
            options,
            ..DifDesign::default()
        }))
        .key()
    };
    let run = |flags: [bool; 4], dif: [f64; 4], classes: usize, roles: &str| {
        let Outcome::Dif(o) = outcome(flags, dif, classes, false, roles) else {
            unreachable!()
        };
        Outcome::Dif(DifOutcome {
            floors: vec![0.18, 0.22, 0.2, 0.2],
            anchor_floor: 0.19,
            ..o
        })
    };
    let (t, f) = (true, false);
    let clean = || vec![run([f; 4], [0.0; 4], 1, "cccc")];
    let flagged = || vec![run([t, f, f, f], [1.2, 0.1, 0.1, 0.1], 2, "cccc")];
    let found = || vec![run([t, t, f, f], [1.8, 1.6, 0.1, 0.1], 2, "++cc")];
    let null = |d: DifDesign| DifDesign {
        n: 3000,
        layout: Layout::Campaign(0),
        delta: 0.0,
        ..d
    };
    write_cells(
        &out,
        Study::FloorNull,
        vec![
            (key(0.2, 5, &null), clean()),
            (
                key(0.2, 5, &|d| DifDesign {
                    anchors: 20,
                    ..null(d)
                }),
                flagged(),
            ),
            (key(0.5, 2, &null), flagged()),
            (key(0.2, 4, &null), flagged()),
        ],
    );
    let off = [
        key(0.2, 5, &|d| DifDesign { pi: 0.3, ..d }),
        key(0.2, 5, &|d| DifDesign { anchors: 40, ..d }),
        key(0.2, 5, &|d| DifDesign {
            layout: Layout::Campaign(3),
            ..d
        }),
        key(0.2, 5, &|d| DifDesign { k: 4, ..d }),
    ];
    let mut power = vec![
        (key(0.2, 5, &|d| d), found()),
        (key(0.5, 2, &|d| d), found()),
    ];
    power.extend(off.iter().map(|k| (k.clone(), found())));
    write_cells(&out, Study::FloorPower, power);
    write_cells(
        &out,
        Study::FloorMisspec,
        vec![(key(0.2, 5, &|d| DifDesign { skew: -4.0, ..d }), found())],
    );
    let markdown = summarize(&out).unwrap();
    let r = row(&out, Study::FloorNull);
    assert!(close(value(&r, "floor_true"), 0.2));
    assert!(close(value(&r, "floor_fitted"), 0.2));
    assert!(close(value(&r, "anchor_floor"), 0.19));
    let floor = "floor: design / items fitted / anchors fitted";
    for (title, column) in [
        ("## floor-null", "| admitted |"),
        ("## floor-power", "| all leaners flagged |"),
        ("## floor-misspec", "| batches with a clean item flagged |"),
    ] {
        let header = table_header(&markdown, title);
        assert!(
            header.contains(floor) && header.contains(column),
            "{title}: {header}"
        );
    }
    assert!(
        markdown.contains("| 0.20 / 0.200 ± 0.016 / 0.190 |"),
        "{markdown}"
    );
    assert_eq!(
        markdown.matches("Floor 0.2, 5 options:").count(),
        1,
        "{markdown}"
    );
    assert_eq!(
        markdown.matches("Floor 0.5, 2 options:").count(),
        1,
        "{markdown}"
    );
    let cut = table_header(&markdown, "Floor 0.2, 5 options:");
    assert_eq!(cut.matches("power N=6000 δ=0.9").count(), 1, "{cut}");
    assert!(!markdown.contains("## dif-null"), "{markdown}");
    let table = fs::read_to_string(out.join("thresholds-dif-cut-floor.csv")).unwrap();
    assert!(
        table.starts_with("floor,options,cut,cell,runs,rate"),
        "{table}"
    );
    assert!(table.contains("\n0.2,5,1,null,1,0,"), "{table}");
    assert!(table.contains("\n0.5,2,1,null,1,0.25,"), "{table}");
    let two = key(0.2, 5, &|d| d);
    assert!(table.contains(&format!("\n0.2,5,1,{two},1,1,")), "{table}");
    assert!(off.iter().all(|k| !table.contains(k.as_str())), "{table}");
    let _ = fs::remove_dir_all(&out);

    let alone = scratch("floor-null-alone");
    write_cells(
        &alone,
        Study::FloorNull,
        vec![(key(0.2, 5, &null), clean())],
    );
    let markdown = summarize(&alone).unwrap();
    assert!(
        markdown.contains("### The DIF cut with a guessing floor"),
        "{markdown}"
    );
    assert!(
        !markdown.contains("### The DIF cut on `DIF_j`"),
        "{markdown}"
    );
    let _ = fs::remove_dir_all(alone);
}

/// The rows of a CSV table, each by column name.
fn csv_rows(path: &Path) -> Vec<Vec<(String, String)>> {
    let text = fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let names: Vec<String> = lines.next().unwrap().split(',').map(String::from).collect();
    lines
        .map(|l| {
            names
                .iter()
                .cloned()
                .zip(l.split(',').map(String::from))
                .collect()
        })
        .collect()
}

fn text(row: &[(String, String)], name: &str) -> String {
    row.iter().find(|(n, _)| n == name).unwrap().1.clone()
}

/// Screen runs: the share kept per kind and per item, and the thresholds table, which
/// replays the recorded verdicts at the production values and moves one threshold at a time.
#[test]
fn the_screen_summary_reads_the_kinds_and_replays_the_verdicts() {
    let out = scratch("screen-summary");
    let nan = f64::NAN;
    let run = |guess_floor: f64, kept: &str| {
        let mut c = vec![0.25; 10];
        c[8] = guess_floor;
        c[9] = nan;
        Outcome::Screen(ScreenOutcome {
            converged: true,
            a: vec![0.8, 1.2, 1.2, 1.6, 1.2, 0.3, 0.45, 1.2, 1.2, nan],
            b: vec![0.0, -1.0, 1.0, 0.9, 2.0, 0.0, 0.0, 3.0, 0.0, nan],
            c,
            rpb: vec![0.3, 0.3, 0.3, 0.3, 0.3, 0.25, 0.25, 0.25, 0.3, -0.3],
            kept: kept.chars().map(|k| k == '1').collect(),
            roles: "gggggffhck".to_string(),
        })
    };
    let key = Cell::Screen(ScreenDesign {
        n: 300,
        anchors: 30,
        options: 4,
    })
    .key();
    write(
        &out,
        Study::FloorScreen,
        &key,
        vec![run(0.32, "1111100010"), run(0.38, "1111100000")],
    );
    let markdown = summarize(&out).unwrap();
    assert!(markdown.contains("## floor-screen"), "{markdown}");
    assert!(
        markdown.contains("### The stage-1 screen's thresholds"),
        "{markdown}"
    );
    let kinds = rows(&out, Study::FloorScreen);
    let kept = |kind: &str| {
        value(
            kinds.iter().find(|r| text(r, "kind") == kind).unwrap(),
            "kept",
        )
    };
    assert_eq!(
        ["good", "flat", "too hard", "guessable", "keyed backwards"].map(kept),
        [1.0, 0.0, 0.0, 0.5, 0.0]
    );
    let items = csv_rows(&out.join(Study::FloorScreen.name()).join("items.csv"));
    assert_eq!(items.len(), 10);
    assert!(close(value(&items[8], "kept"), 0.5) && close(value(&items[8], "c_mean"), 0.35));
    assert_eq!(
        (value(&items[9], "fitted"), value(&items[8], "fitted")),
        (0.0, 1.0)
    );
    let table = csv_rows(&out.join("thresholds-screen.csv"));
    let at = |threshold: &str, v: f64, kind: &str| {
        let row = table.iter().find(|r| {
            text(r, "threshold") == threshold && value(r, "value") == v && text(r, "kind") == kind
        });
        value(row.unwrap(), "kept")
    };
    for (threshold, v) in [("A_MIN", 0.6), ("B_ABS_MAX", 2.5), ("C_EXCESS_MAX", 0.1)] {
        let replayed = ["good", "flat", "too hard", "guessable", "keyed backwards"]
            .map(|kind| at(threshold, v, kind));
        assert_eq!(replayed, [1.0, 0.0, 0.0, 0.5, 0.0], "{threshold}");
    }
    assert_eq!(
        (at("A_MIN", 0.4, "flat"), at("A_MIN", 0.8, "good")),
        (0.5, 1.0)
    );
    assert_eq!(at("A_MIN", 0.5, "flat"), 0.0);
    assert_eq!(
        (
            at("B_ABS_MAX", 3.0, "too hard"),
            at("B_ABS_MAX", 2.75, "too hard")
        ),
        (1.0, 0.0)
    );
    assert_eq!(at("B_ABS_MAX", 2.0, "good"), 1.0);
    let guessable = [0.05, 0.15].map(|v| at("C_EXCESS_MAX", v, "guessable"));
    assert_eq!(guessable, [0.0, 1.0]);
    let _ = fs::remove_dir_all(out);
}
