//! The frozen calibration a validation run is bound to (`docs/19` §§13.21, 13.29): its
//! `threshold.tsv` checked against the calibration's plan, records, ledger and configuration.

use crate::probe::{tasks, Phase};
use crate::probe_jobs::Plan;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const THRESHOLD: &str = "threshold.tsv";
pub const THRESHOLD_SCHEMA: &str = "schema\ta2-ind/v1";
pub const RULE: &str = "rule\torder-statistic\talpha\t1/20\tstrict";
pub const REPLICATES: u32 = 100;
pub const NEUTRAL: [&str; 2] = ["C0", "C1"];

/// The calibration's files the threshold names by SHA-256, in its order.
pub fn data(cal: Phase) -> [String; 3] {
    [
        "records.jsonl".into(),
        format!("plan-{}.tsv", cal.name()),
        "ledger.tsv".into(),
    ]
}

/// The lines of `cal`'s `threshold.tsv` before its cases: `ind-cal`'s schema v1 and rule, or
/// schema v2 naming the stage and its `N`, then the rule.
pub fn header(cal: Phase) -> String {
    match cal {
        Phase::IndCal => format!("{THRESHOLD_SCHEMA}\n{RULE}\n"),
        _ => format!(
            "schema\ta2-ind/v2\nstage\t{}\tn\t{}\n{RULE}\n",
            cal.name(),
            cal.scale().0
        ),
    }
}

/// The stage and `N` a `threshold.tsv` declares: `ind-cal` at 3,000 by schema v1, else v2's.
fn declared(lines: &[&str]) -> Option<(String, String)> {
    match lines {
        [THRESHOLD_SCHEMA, ..] => Some(("ind-cal".into(), "3000".into())),
        ["schema\ta2-ind/v2", stage, ..] => match stage.split('\t').collect::<Vec<_>>()[..] {
            ["stage", name, "n", n] => Some((name.into(), n.into())),
            _ => None,
        },
        _ => None,
    }
}

/// The rank `⌈19 (n + 1) / 20⌉` of a case's threshold among its `n` evaluable values, if `≤ n`.
pub fn rank(n: u64) -> Option<u64> {
    let k = (19 * (n + 1)).div_ceil(20);
    (k <= n).then_some(k)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A double written as its 64 bits in hexadecimal and its decimal, which must agree.
fn double(bits: &str, decimal: &str) -> Result<f64, String> {
    let v = u64::from_str_radix(bits, 16)
        .ok()
        .filter(|_| bits.len() == 16)
        .map(f64::from_bits)
        .filter(|v| v.is_finite())
        .ok_or(format!("{bits:?} is not a finite double's bits"))?;
    if decimal.parse::<f64>().ok() != Some(v) {
        return Err(format!("{decimal:?} is not the double {bits}"));
    }
    Ok(v)
}

/// `cal`'s `threshold.tsv` in the declared form: `t`'s bits, and the files it names with their
/// digests; another stage or size named by its header is refused as such.
fn parse(text: &str, cal: Phase) -> Result<(String, Vec<(String, String)>), String> {
    let lines: Vec<&str> = text.lines().collect();
    let head = header(cal);
    let head: Vec<&str> = head.lines().collect();
    if lines.len() < head.len() || lines[..head.len()] != head[..] {
        let want = (cal.name().to_string(), cal.scale().0.to_string());
        return Err(match declared(&lines) {
            Some(got) if got != want => format!(
                "a calibration of {} at N = {}, not of {} at N = {}",
                got.0, got.1, want.0, want.1
            ),
            _ => "not the declared schema and rule".into(),
        });
    }
    let rows: Vec<Vec<&str>> = lines[head.len()..]
        .iter()
        .map(|l| l.split('\t').collect())
        .collect();
    let mut rows = rows.iter().map(Vec::as_slice).peekable();
    let mut cases = Vec::new();
    for case in NEUTRAL {
        let Some(["case", c, "planned", p, "evaluable", n, "k", k, "t", bits, dec]) = rows.next()
        else {
            return Err(format!("no case line for {case}"));
        };
        let (n, k) = (n.parse::<u64>(), k.parse::<u64>());
        let (Ok(n), Ok(k), true) = (n, k, *c == case && *p == REPLICATES.to_string()) else {
            return Err(format!("{case}'s line is not in the declared form"));
        };
        if n > u64::from(REPLICATES) || rank(n) != Some(k) {
            return Err(format!(
                "{case}: rank {k} is not the declared one for {n} values"
            ));
        }
        cases.push((n, double(bits, dec)?));
    }
    let mut lost = [0u64; 2];
    while let Some(&["not-evaluable", c, why, count]) = rows.peek() {
        let i = NEUTRAL
            .iter()
            .position(|x| x == c)
            .ok_or("an unknown case")?;
        lost[i] += count
            .parse::<u64>()
            .map_err(|_| "a count is not a number")?;
        if why.is_empty() {
            return Err("a not-evaluable line without its reason".into());
        }
        rows.next();
    }
    if (0..2).any(|i| cases[i].0 + lost[i] != u64::from(REPLICATES)) {
        return Err("a case's counts do not sum to its planned runs".into());
    }
    let Some(["threshold", bits, dec]) = rows.next() else {
        return Err("no threshold line".into());
    };
    let larger = if cases[0].1 >= cases[1].1 {
        cases[0].1
    } else {
        cases[1].1
    };
    if double(bits, dec)?.to_bits() != larger.to_bits() {
        return Err("the threshold is not the larger of the cases'".into());
    }
    let mut files = Vec::new();
    for name in data(cal) {
        match rows.next() {
            Some(&["file", f, digest]) if f == name => files.push((f.into(), digest.into())),
            _ => return Err(format!("no digest line for {name}")),
        }
    }
    if rows.next().is_some() {
        return Err("lines after the digests".into());
    }
    Ok((bits.to_string(), files))
}

fn kept(config: &str) -> Vec<&str> {
    config
        .lines()
        .filter(|l| !l.starts_with("caps\t") && !l.starts_with("calibration\t"))
        .collect()
}

/// Whether `dir` holds `cal`'s plan and no other: a calibration of another stage or size is
/// refused by its plan's name before any content is read.
pub fn held(dir: &Path, cal: Phase) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("the calibration: {e}"))?;
    let mut plans: Vec<String> = entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| name.starts_with("plan-") && name.ends_with(".tsv"))
        .collect();
    plans.sort();
    if plans != [data(cal)[1].clone()] {
        return Err(format!(
            "the calibration is not {}'s at N = {}: its plans are {plans:?}",
            cal.name(),
            cal.scale().0
        ));
    }
    Ok(())
}

/// The `calibration` line binding `validation`, whose configuration is `config`, to the threshold
/// frozen in `dir`, or why `dir` is not its own calibration (`docs/19` §§13.21, 13.29).
pub fn calibration_line(dir: &Path, validation: Phase, config: &str) -> Result<String, String> {
    let cal = validation
        .calibration()
        .ok_or(format!("{} is not a validation", validation.name()))?;
    held(dir, cal)?;
    let read = |name: &str| fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    let threshold = read(THRESHOLD)?;
    let text = String::from_utf8(threshold.clone()).map_err(|_| "threshold.tsv is not text")?;
    let (bits, files) = parse(&text, cal).map_err(|e| format!("threshold.tsv: {e}"))?;
    for (name, digest) in &files {
        if sha256_hex(&read(name)?) != *digest {
            return Err(format!("{name} is not the file the threshold names"));
        }
    }
    let name = &data(cal)[1];
    let plan = String::from_utf8(read(name)?).map_err(|_| "the plan is not text")?;
    let own = Plan::config(&plan);
    let declared = Plan::new("", cal, REPLICATES, tasks(cal, REPLICATES));
    if plan[own.len()..] != declared.text {
        return Err(format!("{name} is not the declared calibration plan"));
    }
    if kept(own) != kept(config) {
        return Err("the calibration ran another configuration than this validation".into());
    }
    Ok(format!("calibration\t{}\t{bits}\n", sha256_hex(&threshold)))
}
