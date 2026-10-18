//! The A2 probe's harness (`docs/19` §13.11): seeds, routing, draws and forms on constructed
//! readings; records of every outcome; the supervisor's stops on stand-in children.

use characterization::generate::dif_batch;
use characterization::probe::{
    assess, generating_dtf, probe_seed, raw_gap, record_of, route, run, stopped_record, tasks,
    Case, Cost, Draw, Items, Outcome, Phase, Removal, Route, Task, CASES,
};
use characterization::probe_jobs::{read_ledger, supervise, write_part, Caps, Refusal};
use protocol::revalidation::Recheck;
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::{latent_dif, Ability, Formats, LatentDif};
use scoring::Convergence;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REF_B: [f64; 8] = [
    -0.9204723266115274,
    -0.5322879354113255,
    -0.2932658466688016,
    -0.0943864107661024,
    0.0943864107661024,
    0.2932658466688016,
    0.5322879354113255,
    0.9204723266115274,
];

fn reference(case: Case) -> Items {
    let d = case.design(Phase::Confirm);
    let signs = match case {
        Case::C2 => [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Case::C3 | Case::C5 => [1.0; 8],
        Case::C4 => [1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0],
        Case::C0 | Case::C1 => [0.0; 8],
    };
    Items {
        a: vec![1.25; 8],
        b: REF_B.to_vec(),
        signs: signs.to_vec(),
        delta: d.delta,
        impact: d.impact,
        pi: d.pi,
    }
}

fn normal() -> Ability {
    let nodes: Vec<f64> = (0..41).map(|q| -5.0 + q as f64 * 0.25).collect();
    let weights: Vec<f64> = nodes.iter().map(|t| (-0.5 * t * t).exp()).collect();
    let total: f64 = weights.iter().sum();
    Ability {
        weights: weights.iter().map(|w| w / total).collect(),
        nodes,
    }
}

/// Two equal classes, eight items of slope 1.25 at `b ∓ lean[j]`.
fn curves(lean: [f64; 8]) -> ClassCurves {
    let b: Vec<Vec<f64>> = [-1.0, 1.0]
        .iter()
        .map(|z| (0..8).map(|j| REF_B[j] + lean[j] * z).collect())
        .collect();
    ClassCurves::with_ability(
        &[0.5, 0.5],
        &[0.0, 0.0],
        &normal(),
        &vec![vec![1.25; 8]; 2],
        &b,
        &[0.0; 8],
    )
    .unwrap()
}

fn fit(status: Convergence, classes: usize, gap: [f64; 8]) -> LatentDif {
    let b: Vec<Vec<f64>> = (0..classes)
        .map(|g| {
            (0..8)
                .map(|j| REF_B[j] + if g == 1 { gap[j] } else { 0.0 })
                .collect()
        })
        .collect();
    LatentDif {
        classes,
        non_uniform: false,
        pi: vec![1.0 / classes as f64; classes],
        eta: vec![0.0; classes],
        ability: normal(),
        anchor_a: vec![1.0; 10],
        anchor_b: vec![0.0; 10],
        anchor_c: vec![0.0; 10],
        item_a: vec![vec![1.25; 8]; classes],
        item_b: b,
        item_c: vec![0.0; 8],
        dif: if classes > 1 {
            gap.to_vec()
        } else {
            vec![0.0; 8]
        },
        a_gap: vec![0.0; 8],
        posterior: Vec::new(),
        bic_gain: 0.0,
        candidates: vec![(1, false, 0.0)],
        status,
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("a2-probe-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn task(case: Case, replicate: u32) -> Task {
    Task {
        phase: Phase::Smoke,
        case,
        replicate,
    }
}

fn caps(run_ms: u64, wall_ms: u64) -> Caps {
    Caps {
        cpu_ms: 3_600_000,
        wall_ms,
        run_ms,
        rss_kib: 1 << 30,
    }
}

/// Paired seeds: every case of a replicate shares the population seed; phases never do.
#[test]
fn every_case_of_a_replicate_shares_its_seed_and_phases_do_not() {
    let all = tasks(Phase::Confirm, 8);
    assert_eq!(all.len(), 48);
    assert!(all
        .chunks(6)
        .all(|r| r.iter().all(|t| t.seed() == r[0].seed())));
    assert_eq!(all[6].replicate, 1);
    assert_ne!(probe_seed(Phase::Explore, 0), probe_seed(Phase::Confirm, 0));
    assert_ne!(probe_seed(Phase::Confirm, 0), probe_seed(Phase::Confirm, 1));
}

/// The harness's Simpson DTF meets `sim/a2_probe_reference.json`'s method-B values.
#[test]
fn the_generating_dtf_meets_the_oracle_s_reference_values() {
    let all: Vec<usize> = (0..8).collect();
    let expected = [
        (Case::C0, 0.0, 1e-15),
        (Case::C1, 0.0, 1e-15),
        (Case::C2, 0.7419526749, 1e-9),
        (Case::C3, 0.6126785418, 1e-9),
        (Case::C4, 0.0832184134, 1e-6),
        (Case::C5, 0.1443847409, 1e-9),
    ];
    for (case, want, tol) in expected {
        let got = generating_dtf(&reference(case), &all);
        assert!((got - want).abs() < tol, "{case:?}: {got} against {want}");
    }
    assert!((raw_gap(&reference(Case::C1), &all) - 1.7072066242).abs() < 1e-9);
}

/// Stage-2 routing is the lifecycle's: active, contested or rejected by source, or staying.
#[test]
fn items_are_routed_by_the_lifecycle() {
    assert_eq!(route(8, Recheck::NoDif, false), Route::Active);
    assert_eq!(route(8, Recheck::Dif, true), Route::Contested);
    assert_eq!(route(8, Recheck::Dif, false), Route::Rejected);
    assert_eq!(route(8, Recheck::Indeterminate, true), Route::Pilot);
    assert_eq!(route(1, Recheck::NoDif, true), Route::Invalid);
}

/// Two same-side contested leaners: the draw of both is refused, the reduced draw is empty.
#[test]
fn a_refused_draw_is_read_at_its_largest_feasible_size_and_leaves_an_incomplete_form() {
    let mut rechecks = [Recheck::NoDif; 8];
    rechecks[..2].fill(Recheck::Dif);
    let c = curves([0.9, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let r = assess(&rechecks, Some(&c), true, 7);
    assert_eq!(
        r.draw,
        Draw::Reduced {
            requested: 2,
            feasible: vec![0],
            members: vec![]
        }
    );
    assert_eq!(r.composed.completeness(), "incomplete");
    assert_eq!(
        r.composed.removed,
        vec![(0, Removal::NotDrawn), (1, Removal::NotDrawn)]
    );
    let sel = r.selection.unwrap();
    assert_eq!((sel.completeness(), sel.reference), ("empty", 2));
    let s0 = assess(&rechecks, Some(&c), false, 7);
    assert_eq!(
        (s0.draw, s0.composed.removed[0]),
        (Draw::NotNeeded, (0, Removal::Rejected))
    );
}

/// A mirror pair that cancels is drawn whole, its cost within the tolerance.
#[test]
fn a_cancelling_pair_is_drawn_whole_within_the_tolerance() {
    let mut rechecks = [Recheck::NoDif; 8];
    rechecks[3..5].fill(Recheck::Dif);
    let c = curves([0.0, 0.0, 0.0, 0.9, -0.9, 0.0, 0.0, 0.0]);
    let r = assess(&rechecks, Some(&c), true, 7);
    assert_eq!(r.draw, Draw::Selected(vec![3, 4]));
    assert!(r.cost.unwrap() <= DTF_MAX);
    assert_eq!(r.composed.completeness(), "complete");
}

/// Indeterminate items empty the form; contested items without curves are a failed draw.
#[test]
fn an_emptied_form_is_empty_and_a_draw_without_curves_fails() {
    let r = assess(&[Recheck::Indeterminate; 8], None, true, 7);
    assert_eq!(
        (r.active.completeness(), r.composed.completeness()),
        ("empty", "empty")
    );
    assert!(r
        .composed
        .removed
        .iter()
        .all(|&(_, why)| why == Removal::Indeterminate));
    let mut rechecks = [Recheck::NoDif; 8];
    rechecks[0] = Recheck::Dif;
    let r = assess(&rechecks, None, true, 7);
    assert_eq!(r.draw, Draw::NoCurves);
    assert_eq!(r.composed.removed, vec![(0, Removal::DrawFailed)]);
}

/// Every outcome's record carries the truth and the cost; only a read one carries readings.
#[test]
fn every_outcome_is_recorded_with_its_truth() {
    let t = task(Case::C2, 0);
    let d = t.case.design(t.phase);
    let batch = dif_batch(&d, t.seed());
    let cost = || Some(Cost::default());
    let gap = [2.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let read = Ok(fit(Convergence::Converged, 2, gap));
    let stuck = Ok(fit(Convergence::MaxIters, 2, gap));
    let refused = Err(protocol::pilot::PilotError::BadFormats);
    let lines = [
        record_of(&t, &d, &batch, Outcome::Gate(&read), cost),
        record_of(&t, &d, &batch, Outcome::Gate(&stuck), cost),
        record_of(&t, &d, &batch, Outcome::Gate(&refused), cost),
        stopped_record(&t, "killed", "run limit", Cost::default()),
    ];
    for (line, outcome) in lines.iter().zip(["read", "read", "refused", "killed"]) {
        assert!(
            line.contains(&format!("\"outcome\":\"{outcome}\"")),
            "{line}"
        );
        assert!(line.contains("\"truth\":{\"dtf\":") && line.contains("\"cost\":{"));
        assert_eq!(line.contains("\"readings\":"), outcome == "read");
    }
    assert!(
        lines[0].contains("\"rechecks\":\"DDNNNNNN\"")
            && lines[1].contains("\"rechecks\":\"IIIIIIII\"")
    );
    assert!(
        lines[0].contains("\"routes\":\"RRAAAAAA\"")
            && lines[0].contains("\"routes\":\"CCAAAAAA\"")
    );
}

/// The real path refuses a smoke batch at the gate; outside it, a real smoke fit is read.
#[test]
fn the_real_path_refuses_a_smoke_batch_and_reads_a_real_smoke_fit_outside_the_gate() {
    let t = task(Case::C2, 0);
    let line = run(&t, || None);
    assert!(line.contains("\"outcome\":\"refused\"") && line.contains("NotEnoughRespondents"));
    let d = t.case.design(t.phase);
    let batch = dif_batch(&d, t.seed());
    let fitted = latent_dif(&batch.anchors, &batch.x, &Formats::open(d.anchors, d.k), 1);
    let line = record_of(
        &t,
        &d,
        &batch,
        Outcome::Gate(&fitted.map_err(|_| protocol::pilot::PilotError::BadFormats)),
        || None,
    );
    assert!(line.contains("\"outcome\":\"read\"") && line.contains("\"source\":\"S1\""));
}

fn fake(t: &Task, out: &Path) -> std::io::Result<std::process::Child> {
    write_part(
        out,
        t,
        &stopped_record(t, "read", "stand-in", Cost::default()),
    )?;
    Command::new("true").spawn()
}

/// Each ended run is recorded once; a later session skips them and runs nothing again.
#[cfg(target_os = "linux")]
#[test]
fn ended_runs_are_recorded_once_and_never_rerun() {
    let out = scratch("resume");
    let all: Vec<Task> = CASES.into_iter().map(|c| task(c, 0)).collect();
    let first = supervise(&all, 3, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    assert_eq!(first.outcomes.get("read"), Some(&6));
    let again = supervise(&all, 3, &out, caps(60_000, 60_000), fake)
        .unwrap()
        .unwrap();
    assert_eq!((again.skipped, again.outcomes.len()), (6, 0));
    let records = fs::read_to_string(out.join("records.jsonl")).unwrap();
    assert_eq!(records.lines().count(), 6);
    assert!(fs::read_dir(out.join("parts")).unwrap().next().is_none());
}

/// A child that exits without a record is recorded as failed, with the design's truth.
#[cfg(target_os = "linux")]
#[test]
fn a_child_without_a_record_is_recorded_as_failed() {
    let out = scratch("failed");
    let all = [task(Case::C4, 0)];
    let s = supervise(&all, 1, &out, caps(60_000, 60_000), |_, _| {
        Command::new("false").spawn()
    });
    assert_eq!(s.unwrap().unwrap().outcomes.get("failed"), Some(&1));
    let records = fs::read_to_string(out.join("records.jsonl")).unwrap();
    assert!(records.contains("\"outcome\":\"failed\"") && records.contains("\"truth\":"));
}

/// The run limit kills the run, launches nothing more, and a stop refuses any later session.
#[cfg(target_os = "linux")]
#[test]
fn the_run_limit_kills_the_run_and_the_stop_refuses_a_resumption() {
    let out = scratch("run-limit");
    let all: Vec<Task> = CASES.into_iter().map(|c| task(c, 0)).collect();
    let sleep = |_: &Task, _: &Path| Command::new("sleep").arg("30").spawn();
    let s = supervise(&all, 1, &out, caps(300, 60_000), sleep)
        .unwrap()
        .unwrap();
    assert_eq!(s.outcomes.get("killed"), Some(&1));
    assert!(s.wall_ms < 10_000, "{s:?}");
    assert_eq!(s.stop.as_deref(), Some("run limit: smoke-C0-0"));
    let refused = supervise(&all, 1, &out, caps(300, 60_000), sleep).unwrap();
    assert_eq!(
        refused,
        Err(Refusal::Stopped("run limit: smoke-C0-0".into()))
    );
    let records = fs::read_to_string(out.join("records.jsonl")).unwrap();
    assert!(records.contains("\"outcome\":\"killed\",\"reason\":\"run limit\""));
}

/// The wall cap kills every running child, each recorded as killed.
#[cfg(target_os = "linux")]
#[test]
fn the_wall_cap_kills_every_running_child() {
    let out = scratch("wall");
    let all: Vec<Task> = CASES.into_iter().map(|c| task(c, 0)).collect();
    let sleep = |_: &Task, _: &Path| Command::new("sleep").arg("30").spawn();
    let s = supervise(&all, 2, &out, caps(60_000, 400), sleep)
        .unwrap()
        .unwrap();
    assert_eq!(
        (s.outcomes.get("killed"), s.stop.as_deref()),
        (Some(&2), Some("wall cap"))
    );
    let ledger = read_ledger(&out).unwrap();
    assert_eq!((ledger.ended.len(), ledger.unfinished.len()), (2, 0));
}

/// A run started and never ended, or caps already spent, refuse a session.
#[cfg(target_os = "linux")]
#[test]
fn an_unfinished_run_or_spent_caps_refuse_a_session() {
    let out = scratch("unfinished");
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("ledger.tsv"), "start\tsmoke-C0-0\n").unwrap();
    let all = [task(Case::C0, 0)];
    let s = supervise(&all, 1, &out, caps(60_000, 60_000), fake).unwrap();
    assert_eq!(s, Err(Refusal::Unfinished(vec!["smoke-C0-0".into()])));
    fs::write(out.join("ledger.tsv"), "session\t60000\t0\t0\t-\n").unwrap();
    let s = supervise(&all, 1, &out, caps(60_000, 60_000), fake).unwrap();
    assert_eq!(s, Err(Refusal::Spent));
}
