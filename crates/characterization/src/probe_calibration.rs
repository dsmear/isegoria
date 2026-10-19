//! The frozen calibration an `ind-val` run is bound to (`docs/19` §13.21): its `threshold.tsv`
//! checked against the calibration's plan, records, ledger and configuration before any run.

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
pub const DATA: [&str; 3] = ["records.jsonl", "plan-ind-cal.tsv", "ledger.tsv"];

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

/// `threshold.tsv` in the declared form: `t`'s bits, and the files it names with their digests.
fn parse(text: &str) -> Result<(String, Vec<(String, String)>), String> {
    let rows: Vec<Vec<&str>> = text.lines().map(|l| l.split('\t').collect()).collect();
    let mut rows = rows.iter().map(Vec::as_slice).peekable();
    if rows.next().map(|r| r.join("\t")).as_deref() != Some(THRESHOLD_SCHEMA)
        || rows.next().map(|r| r.join("\t")).as_deref() != Some(RULE)
    {
        return Err("not the declared schema and rule".into());
    }
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
    for name in DATA {
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

/// The `calibration` line binding a validation whose configuration is `config` to the threshold
/// frozen in `dir`, or why `dir` is not the declared calibration for it (`docs/19` §13.21).
pub fn calibration_line(dir: &Path, config: &str) -> Result<String, String> {
    let read = |name: &str| fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"));
    let threshold = read(THRESHOLD)?;
    let text = String::from_utf8(threshold.clone()).map_err(|_| "threshold.tsv is not text")?;
    let (bits, files) = parse(&text).map_err(|e| format!("threshold.tsv: {e}"))?;
    for (name, digest) in &files {
        if sha256_hex(&read(name)?) != *digest {
            return Err(format!("{name} is not the file the threshold names"));
        }
    }
    let plan = String::from_utf8(read(DATA[1])?).map_err(|_| "the plan is not text")?;
    let own = Plan::config(&plan);
    let declared = Plan::new(
        "",
        Phase::IndCal,
        REPLICATES,
        tasks(Phase::IndCal, REPLICATES),
    );
    if plan[own.len()..] != declared.text {
        return Err("plan-ind-cal.tsv is not the declared calibration plan".into());
    }
    if kept(own) != kept(config) {
        return Err("the calibration ran another configuration than this validation".into());
    }
    Ok(format!("calibration\t{}\t{bits}\n", sha256_hex(&threshold)))
}
