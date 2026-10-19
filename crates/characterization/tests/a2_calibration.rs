//! The frozen calibration a validation is bound to (`docs/19` §13.21): `calibration_line` on
//! constructed directories, the declared one accepted and every departure refused.

use characterization::probe::{config_lines, tasks, Phase};
use characterization::probe_calibration::{calibration_line, rank, sha256_hex};
use characterization::probe_jobs::Plan;
use std::fs;
use std::path::PathBuf;

const CANDIDATE: &str = "candidate\ttree\tlock\tlatent\tclean\n";

fn config(caps: &str, phase: Phase) -> String {
    format!("{CANDIDATE}caps\t{caps}\n{}", config_lines(phase))
}

fn line(t: f64) -> String {
    format!("{:016x}\t{t:?}", t.to_bits())
}

struct Cal {
    dir: PathBuf,
    cases: [(u64, u64, f64); 2],
    lost: Vec<(&'static str, u64)>,
    t: f64,
    rule: &'static str,
}

impl Cal {
    fn new(name: &str) -> Cal {
        let dir = std::env::temp_dir().join(format!("a2-cal-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let plan = Plan::new(
            &config("7200000\t1800000\t600000\t2097152", Phase::IndCal),
            Phase::IndCal,
            100,
            tasks(Phase::IndCal, 100),
        );
        fs::write(dir.join("plan-ind-cal.tsv"), plan.text).unwrap();
        fs::write(dir.join("records.jsonl"), "{}\n").unwrap();
        fs::write(dir.join("ledger.tsv"), "ledger\ta2-probe/v2\n").unwrap();
        Cal {
            dir,
            cases: [(100, 96, 0.25), (98, 95, 0.5)],
            lost: vec![("C1", 2)],
            t: 0.5,
            rule: "rule\torder-statistic\talpha\t1/20\tstrict",
        }
    }

    /// Writes `threshold.tsv` naming the files as they now are.
    fn freeze(&self) -> &Cal {
        let mut text = format!("schema\ta2-ind/v1\n{}\n", self.rule);
        for (case, (n, k, t)) in ["C0", "C1"].iter().zip(self.cases) {
            text += &format!(
                "case\t{case}\tplanned\t100\tevaluable\t{n}\tk\t{k}\tt\t{}\n",
                line(t)
            );
        }
        for (case, count) in &self.lost {
            text += &format!("not-evaluable\t{case}\tgate refused\t{count}\n");
        }
        text += &format!("threshold\t{}\n", line(self.t));
        for name in ["records.jsonl", "plan-ind-cal.tsv", "ledger.tsv"] {
            text += &format!(
                "file\t{name}\t{}\n",
                sha256_hex(&fs::read(self.dir.join(name)).unwrap())
            );
        }
        fs::write(self.dir.join("threshold.tsv"), text).unwrap();
        self
    }

    fn bind(&self, config: &str) -> Result<String, String> {
        calibration_line(&self.dir, config)
    }
}

/// The rank is `⌈19 (n + 1) / 20⌉`, finite from 19 evaluable values on (§13.21).
#[test]
fn the_rank_is_the_declared_order_statistic_and_needs_nineteen_values() {
    assert_eq!(
        (rank(100), rank(98), rank(19), rank(18), rank(0)),
        (Some(96), Some(95), Some(19), None, None)
    );
}

/// The declared calibration binds a validation of the same configuration, caps apart (§13.21).
#[test]
fn the_declared_calibration_binds_a_validation_whatever_its_caps() {
    let cal = Cal::new("valid");
    cal.freeze();
    let digest = sha256_hex(&fs::read(cal.dir.join("threshold.tsv")).unwrap());
    let want = format!("calibration\t{digest}\t{:016x}\n", 0.5f64.to_bits());
    assert_eq!(
        cal.bind(&config("14400000\t3600000\t600000\t2097152", Phase::IndVal)),
        Ok(want)
    );
}

/// A changed file, a threshold out of form, another plan or configuration: refused (§13.21).
#[test]
fn a_threshold_out_of_form_or_bound_to_other_data_is_refused() {
    let val = config("14400000\t3600000\t600000\t2097152", Phase::IndVal);
    let refused = |cal: &Cal, why: &str| {
        let got = cal.bind(&val);
        assert!(
            got.as_ref().is_err_and(|e| e.contains(why)),
            "{why}: {got:?}"
        );
    };
    let cal = Cal::new("refused");
    cal.freeze();
    fs::write(cal.dir.join("records.jsonl"), "{}\n{}\n").unwrap();
    refused(&cal, "records.jsonl is not the file the threshold names");
    let mut cal = Cal::new("refused");
    cal.t = 0.25;
    refused(cal.freeze(), "not the larger");
    cal.t = 0.5;
    cal.cases[0].1 = 95;
    refused(
        cal.freeze(),
        "rank 95 is not the declared one for 100 values",
    );
    cal.cases = [(100, 96, 0.25), (18, 19, 0.5)];
    cal.lost = vec![("C1", 82)];
    refused(
        cal.freeze(),
        "rank 19 is not the declared one for 18 values",
    );
    cal.cases = [(100, 96, 0.25), (98, 95, 0.5)];
    cal.lost = vec![("C1", 1)];
    refused(cal.freeze(), "do not sum");
    cal.lost = vec![("C1", 2)];
    cal.rule = "rule\torder-statistic\talpha\t1/10\tstrict";
    refused(cal.freeze(), "schema and rule");
    let cal = Cal::new("refused");
    let text = fs::read_to_string(cal.freeze().dir.join("threshold.tsv")).unwrap();
    fs::write(
        cal.dir.join("threshold.tsv"),
        text.replace("\t0.5\n", "\t0.50001\n"),
    )
    .unwrap();
    refused(&cal, "is not the double");
    for plan in [
        Plan::new(
            &config("1\t1\t1\t1", Phase::IndCal),
            Phase::IndCal,
            1,
            tasks(Phase::IndCal, 1),
        ),
        Plan::new(
            &config("1\t1\t1\t1", Phase::IndCal),
            Phase::IndCal,
            100,
            tasks(Phase::IndCal, 100),
        ),
    ] {
        let cal = Cal::new("refused");
        let text = plan.text.replace("\t14817136108943293105\n", "\t1\n");
        fs::write(cal.dir.join("plan-ind-cal.tsv"), text).unwrap();
        refused(cal.freeze(), "not the declared calibration plan");
    }
    let cal = Cal::new("refused");
    cal.freeze();
    assert!(cal.bind(&val.replacen("\ttree\t", "\tother\t", 1)).is_err());
    assert!(cal.bind(&val.replacen("C3\t3000", "C3\t2999", 1)).is_err());
    fs::remove_file(cal.dir.join("threshold.tsv")).unwrap();
    assert!(cal
        .bind(&val)
        .is_err_and(|e| e.starts_with("threshold.tsv:")));
}
