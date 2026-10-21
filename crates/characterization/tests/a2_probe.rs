//! The A2 probe's harness (`docs/19` §13.11): seeds, routing, draws and forms on constructed
//! readings; records of every outcome; the supervisor's stops on stand-in children.

use characterization::generate::dif_batch;
use characterization::grid::engine_seed;
use characterization::probe::{
    assess, config_lines, diagnosed_record, gate, generating_dtf, probe_seed, raw_gap, record_of,
    route, run, stopped_record, tasks, Case, Cost, Draw, Items, Outcome, Phase, Removal, Route,
    Task, PHASES,
};
use identity::nym::Nym;
use protocol::admission::NullifierSet;
use protocol::pilot::{PilotError, Templates};
use protocol::revalidation::{latent_batch, Recheck};
use scoring::dtf::{ClassCurves, DTF_MAX};
use scoring::latent::{
    latent_dif, latent_dif_candidates, Ability, Formats, LatentDif, LatentParams,
};
use scoring::Convergence;

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

fn task(case: Case, replicate: u32) -> Task {
    Task {
        phase: Phase::Smoke,
        case,
        replicate,
        n: None,
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

/// The Simpson DTF meets the method-B values of the archive's `sim/a2_probe_reference.json`.
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

/// A returned selection over `DTF_MAX` is the blocking defect; a high true DTF is not one.
#[test]
fn only_a_selection_over_the_tolerance_is_a_defect() {
    let mut rechecks = [Recheck::NoDif; 8];
    rechecks[3..5].fill(Recheck::Dif);
    let c = curves([0.0, 0.0, 0.0, 0.9, -0.9, 0.0, 0.0, 0.0]);
    let mut r = assess(&rechecks, Some(&c), true, 7);
    assert!(!r.defect());
    r.cost = Some(DTF_MAX + 1e-9);
    assert!(r.defect());
    r.draw = Draw::NotNeeded;
    assert!(!r.defect());
    let high = generating_dtf(&reference(Case::C2), &(0..8).collect::<Vec<_>>());
    assert!(high > 0.5);
}

/// `candidates` refits confirm's batches of C0, C3 and C5 under its own keys (`docs/19` §13.14).
#[test]
fn the_candidates_phase_reuses_confirm_s_seeds_under_its_own_keys() {
    let all = tasks(Phase::Candidates, 8);
    assert_eq!(all.len(), 24);
    for t in &all {
        assert!(matches!(t.case, Case::C0 | Case::C3 | Case::C5));
        let twin = Task {
            phase: Phase::Confirm,
            ..*t
        };
        assert_eq!(t.seed(), twin.seed());
        assert_ne!(t.key(), twin.key());
        assert_eq!(t.case.design(t.phase), twin.case.design(twin.phase));
    }
}

fn respondents(n: usize) -> NullifierSet {
    let mut set = NullifierSet::new();
    for i in 0..n {
        let mut id = [0u8; 32];
        id[..8].copy_from_slice(&(i as u64).to_le_bytes());
        set.spend(Nym(id)).unwrap();
    }
    set
}

/// The diagnostic path's gate refuses as `latent_batch` does, error for error, before any fit.
#[test]
fn the_diagnostic_gate_refuses_as_the_production_gate() {
    let d = Case::C3.design(Phase::Confirm);
    let batch = dif_batch(&d, 7);
    let (a, x) = (&batch.anchors, &batch.x);
    let coin: Vec<Vec<f64>> = (0..d.n)
        .map(|i| {
            (0..d.anchors)
                .map(|j| ((i * 7 + j * 13) % 2) as f64)
                .collect()
        })
        .collect();
    let open = Formats::open(d.anchors, d.k);
    let none = Templates::none(d.anchors, d.k);
    let one: Vec<Vec<f64>> = x.iter().map(|r| r[..1].to_vec()).collect();
    type Refused<'a> = (&'a str, usize, &'a [Vec<f64>], &'a [Vec<f64>], Formats);
    let cases: Vec<Refused> = vec![
        ("respondents", 2_999, a, x, open.clone()),
        ("rows", 3_001, a, x, open.clone()),
        ("anchor rows", d.n, &a[..d.n - 1], x, open.clone()),
        ("formats", d.n, a, x, Formats::open(d.anchors + 1, d.k)),
        ("one item", d.n, a, &one, Formats::open(d.anchors, 1)),
        ("anchors", d.n, &coin, x, open.clone()),
    ];
    for (name, n, anchors, responses, formats) in cases {
        let templates = Templates::none(formats.anchors.len(), formats.items.len());
        let ours = gate(&respondents(n), anchors, responses, &formats, &templates);
        let theirs = latent_batch(&respondents(n), anchors, responses, &formats, &templates, 1);
        let theirs = theirs
            .err()
            .unwrap_or_else(|| panic!("{name}: latent_batch admitted"));
        assert_eq!(
            format!("{ours:?}"),
            format!("{:?}", Err::<(), PilotError>(theirs)),
            "{name}"
        );
    }
    assert!(gate(&respondents(d.n), a, x, &open, &none).is_ok());
}

/// The diagnostic record holds the ordinary record of the same fit, then each candidate; one
/// real smoke fit outside the gate.
#[test]
fn the_diagnostic_record_extends_the_ordinary_record_of_the_same_fit() {
    let t = task(Case::C2, 0);
    let d = t.case.design(t.phase);
    let batch = dif_batch(&d, t.seed());
    let lp = LatentParams {
        seed: 1,
        ..LatentParams::default()
    };
    let formats = Formats::open(d.anchors, d.k);
    let diagnosed = latent_dif_candidates(&batch.anchors, &batch.x, &formats, &lp)
        .map_err(|_| PilotError::BadFormats);
    let fit = diagnosed.as_ref().map(|(f, _)| f.clone()).map_err(|e| *e);
    let line = diagnosed_record(&t, &d, &batch, &diagnosed, || None);
    let base = record_of(&t, &d, &batch, Outcome::Gate(&fit), || None);
    let (i, j) = (
        line.find(",\"candidates\":[").unwrap(),
        line.rfind(",\"cost\":").unwrap(),
    );
    assert_eq!(format!("{}{}", &line[..i], &line[j..]), base);
    assert_eq!(line.matches("\"verdict\":\"selected\"").count(), 1);
    assert!(line.contains("\"gain\":0.0,\"penalty\":0.0,\"delta_bic\":0.0"));
    assert!(line.contains("\"dtf_basis\":"));
}

/// `dtf-check` (`docs/19` §13.18): C0 and C3 at 16 replicates, paired by seed under their own
/// keys, every seed new — none of smoke's, explore's, confirm's or `candidates`'.
#[test]
fn the_dtf_check_phase_pairs_new_seeds_disjoint_from_every_earlier_phase() {
    let all = tasks(Phase::DtfCheck, 16);
    assert_eq!(all.len(), 32);
    assert!(all.iter().all(|t| matches!(t.case, Case::C0 | Case::C3)));
    assert!(all
        .chunks(2)
        .all(|p| p[0].seed() == p[1].seed() && p[0].case != p[1].case));
    assert!(all.iter().all(|t| t.key().starts_with("dtf-check-")));
    let mut ours: Vec<u64> = all.iter().map(Task::seed).collect();
    ours.dedup();
    assert_eq!(ours.len(), 16);
    let mut earlier: Vec<u64> = (0..64)
        .flat_map(|r| [Phase::Smoke, Phase::Explore, Phase::Confirm].map(|p| probe_seed(p, r)))
        .collect();
    earlier.extend(tasks(Phase::Candidates, 8).iter().map(Task::seed));
    assert!(ours.iter().all(|s| !earlier.contains(s)));
    assert!(all.iter().all(|t| t.phase.diagnostic()));
}

/// `ind-cal` and `ind-val` (`docs/19` §13.21): their 200 and 400 runs, each with a seed of its own
/// — across stages and cases — none of an earlier phase's; known values the oracle reproduces.
#[test]
fn the_indicator_stages_seed_every_run_apart_from_each_other_and_every_earlier_phase() {
    let cal = tasks(Phase::IndCal, 100);
    let val = tasks(Phase::IndVal, 100);
    assert_eq!((cal.len(), val.len()), (200, 400));
    assert!(cal.iter().all(|t| matches!(t.case, Case::C0 | Case::C1)));
    for case in [Case::C0, Case::C1, Case::C3, Case::C5] {
        assert_eq!(val.iter().filter(|t| t.case == case).count(), 100);
    }
    assert!(cal.iter().all(|t| t.key().starts_with("ind-cal-")));
    assert!(val.iter().all(|t| t.key().starts_with("ind-val-")));
    let mut ours: Vec<u64> = cal.iter().chain(&val).map(Task::seed).collect();
    ours.sort_unstable();
    ours.dedup();
    assert_eq!(ours.len(), 600);
    let mut earlier: Vec<u64> = (0..64)
        .flat_map(|r| [Phase::Smoke, Phase::Explore, Phase::Confirm].map(|p| probe_seed(p, r)))
        .collect();
    earlier.extend(tasks(Phase::Candidates, 8).iter().map(Task::seed));
    earlier.extend(tasks(Phase::DtfCheck, 16).iter().map(Task::seed));
    assert!(ours.iter().all(|s| !earlier.contains(s)));
    assert_eq!(cal[0].seed(), 14817136108943293105);
    assert_eq!(val[399].seed(), 12356750456368884976);
    assert!(cal.iter().chain(&val).all(|t| t.phase.diagnostic()));
    assert!(val.iter().all(|t| t.case.design(t.phase).n == 3000));
}

/// `size` (`docs/19` §13.26): 64 runs, `n` in every key, one seed per case and replicate shared
/// by the two sizes alone, none of an earlier phase's.
#[test]
fn the_size_study_plans_64_explicit_runs_seeded_apart_but_shared_across_sizes() {
    let all = tasks(Phase::Size, 16);
    assert_eq!(all.len(), 64);
    assert_eq!(
        (all[0].key(), all[63].key()),
        (
            "size-n3000-C0-0".to_string(),
            "size-n6000-C3-15".to_string()
        )
    );
    for t in &all {
        assert!(matches!(t.case, Case::C0 | Case::C3) && t.phase.diagnostic());
        assert_eq!(t.design().n as u32, t.n.unwrap());
        assert_eq!(t.design().anchors, 60);
        let twin = all
            .iter()
            .find(|u| u.case == t.case && u.replicate == t.replicate && u.n != t.n)
            .unwrap();
        assert_eq!(t.seed(), twin.seed());
    }
    let mut ours: Vec<u64> = all.iter().map(Task::seed).collect();
    ours.sort_unstable();
    ours.dedup();
    assert_eq!(ours.len(), 32);
    let mut earlier: Vec<u64> = (0..64)
        .flat_map(|r| [Phase::Smoke, Phase::Explore, Phase::Confirm].map(|p| probe_seed(p, r)))
        .collect();
    for (phase, r) in [
        (Phase::Candidates, 8),
        (Phase::DtfCheck, 16),
        (Phase::IndCal, 100),
        (Phase::IndVal, 100),
    ] {
        earlier.extend(tasks(phase, r).iter().map(Task::seed));
    }
    assert!(ours.iter().all(|s| !earlier.contains(s)));
}

/// At one seed the 6,000 batch nests the 3,000 one: items, truth and the first 3,000 rows (§13.26).
#[test]
fn the_larger_size_nests_the_smaller_one_s_items_and_respondents() {
    for (case, replicate) in [(Case::C0, 0), (Case::C3, 0), (Case::C3, 15)] {
        let at = |n| Task {
            phase: Phase::Size,
            case,
            replicate,
            n: Some(n),
        };
        let (small, large) = (at(3000), at(6000));
        let (ds, dl) = (small.design(), large.design());
        let (a, b) = (dif_batch(&ds, small.seed()), dif_batch(&dl, large.seed()));
        assert_eq!(
            (&a.a, &a.b, &a.signs, &a.floors),
            (&b.a, &b.b, &b.signs, &b.floors)
        );
        assert_eq!((a.x.len(), b.x.len()), (3000, 6000));
        assert_eq!(a.anchors[..], b.anchors[..3000]);
        assert_eq!(a.x[..], b.x[..3000]);
        assert_eq!(a.z[..], b.z[..3000]);
        assert_ne!(b.x[3000..], b.x[..3000]);
        let all: Vec<usize> = (0..8).collect();
        let truth = |d, batch| generating_dtf(&Items::of(d, batch), &all);
        assert_eq!(truth(&ds, &a), truth(&dl, &b));
        assert_eq!(engine_seed(small.seed()), engine_seed(large.seed()));
    }
}

/// `n` belongs to `size` alone, at 3,000 or 6,000; every other phase runs without one (§13.26).
#[test]
fn only_size_takes_an_n_and_only_its_two_sizes() {
    for n in [None, Some(0), Some(1), Some(3000), Some(4000), Some(6000)] {
        assert_eq!(
            Phase::Size.takes(n),
            matches!(n, Some(3000 | 6000)),
            "size {n:?}"
        );
        for phase in [Phase::Smoke, Phase::Explore, Phase::DtfCheck, Phase::IndVal] {
            assert_eq!(phase.takes(n), n.is_none(), "{} {n:?}", phase.name());
        }
    }
}

/// `ind-cal6` and `ind-val6` (`docs/19` §13.29): 200 and 400 runs at 6,000, without `n`, each
/// seeded apart from the other stage, every case and every earlier phase; known values.
#[test]
fn the_stages_at_6000_seed_every_run_apart_from_each_other_and_every_earlier_phase() {
    let cal = tasks(Phase::IndCal6, 100);
    let val = tasks(Phase::IndVal6, 100);
    assert_eq!((cal.len(), val.len()), (200, 400));
    assert!(cal.iter().all(|t| matches!(t.case, Case::C0 | Case::C1)));
    for case in [Case::C0, Case::C1, Case::C3, Case::C5] {
        assert_eq!(val.iter().filter(|t| t.case == case).count(), 100);
    }
    assert_eq!(
        (cal[0].key(), val[399].key()),
        ("ind-cal6-C0-0".to_string(), "ind-val6-C5-99".to_string())
    );
    for t in cal.iter().chain(&val) {
        assert!(t.n.is_none() && t.phase.diagnostic());
        assert_eq!(
            (t.design().n, t.design().anchors, t.design().k),
            (6000, 60, 8)
        );
        let mut same = t.case.design(Phase::IndVal);
        same.n = 6000;
        assert_eq!(t.design(), same);
    }
    let mut ours: Vec<u64> = cal.iter().chain(&val).map(Task::seed).collect();
    ours.sort_unstable();
    ours.dedup();
    assert_eq!(ours.len(), 600);
    let mut earlier: Vec<u64> = (0..64)
        .flat_map(|r| [Phase::Smoke, Phase::Explore, Phase::Confirm].map(|p| probe_seed(p, r)))
        .collect();
    for (phase, r) in [
        (Phase::Candidates, 8),
        (Phase::DtfCheck, 16),
        (Phase::IndCal, 100),
        (Phase::IndVal, 100),
        (Phase::Size, 16),
    ] {
        earlier.extend(tasks(phase, r).iter().map(Task::seed));
    }
    assert!(ours.iter().all(|s| !earlier.contains(s)));
    assert_eq!(cal[0].seed(), 4800293454221026440);
    assert_eq!(val[399].seed(), 4099950399348182921);
}

/// Each phase's scale, name and calibration: only the two new stages at 6,000, their
/// configuration alike and every design at 6,000; the earlier phases unchanged (§13.29).
#[test]
fn only_the_new_stages_run_at_6000_and_each_validation_names_its_own_calibration() {
    for phase in PHASES {
        assert_eq!(Phase::parse(phase.name()), Some(phase));
        let at6000 = matches!(phase, Phase::IndCal6 | Phase::IndVal6);
        let n = if at6000 {
            6000
        } else if phase == Phase::Smoke {
            400
        } else {
            3000
        };
        assert_eq!(phase.scale().0, n, "{}", phase.name());
        assert_eq!(phase.takes(None), phase != Phase::Size);
        assert!(!phase.takes(Some(6000)) || phase == Phase::Size);
        let cal = match phase {
            Phase::IndVal => Some(Phase::IndCal),
            Phase::IndVal6 => Some(Phase::IndCal6),
            _ => None,
        };
        assert_eq!(phase.calibration(), cal, "{}", phase.name());
    }
    let six = config_lines(Phase::IndCal6);
    assert_eq!(six, config_lines(Phase::IndVal6));
    assert_eq!(six.matches("\t6000\t60\t8\t").count(), 6);
    assert_eq!(
        six.replace("\t6000\t", "\t3000\t"),
        config_lines(Phase::IndCal)
    );
    assert!(config_lines(Phase::IndVal).contains("design\tC0\t3000\t60\t8\tc0\t0.0\t0.0\t0.5\n"));
}
