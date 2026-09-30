//! A run is a function of its task, the records do not depend on the number of workers, and
//! an interrupted run resumes without losing or repeating a record (`docs/13` §2).

use characterization::grid::{tasks, Grid, Study, Task, STUDIES};
use characterization::record::{header, read, Record};
use characterization::run::run;
use characterization::runner::{execute, Options};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("characterization-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn options(out: &Path, jobs: usize) -> Options {
    Options {
        jobs,
        out: out.to_path_buf(),
        progress: false,
    }
}

/// Every record of a directory, its time cleared, in a canonical order.
fn records(out: &Path) -> Vec<String> {
    let mut all: Vec<String> = STUDIES
        .iter()
        .flat_map(|&s| read(out, s).unwrap())
        .map(|r| Record { elapsed_ms: 0, ..r }.line())
        .collect();
    all.sort();
    all
}

/// The quicker studies of the smoke grid, one of each record kind but the DTF one.
fn smoke() -> Vec<Task> {
    let quick = [
        Study::DifPower,
        Study::BridgingSweep,
        Study::BridgingCapture,
    ];
    tasks(&quick, Grid::Smoke, Some(2), None)
}

/// The same task gives the same record, time aside, and records parse back to themselves.
#[test]
fn a_run_is_a_function_of_its_task() {
    let kinds = [
        Study::DifPower,
        Study::DtfError,
        Study::BridgingSweep,
        Study::BridgingCapture,
        Study::FloorPower,
        Study::BridgingExtra,
    ];
    for task in tasks(&kinds, Grid::Smoke, Some(1), None) {
        let first = Record::new(&task, 0, run(&task));
        let again = Record::new(&task, 0, run(&task));
        assert_eq!(first.line(), again.line(), "{}", task.cell.key());
        let parsed = Record::parse(task.study, &first.line()).unwrap();
        assert_eq!(parsed.line(), first.line());
    }
}

/// One worker and three workers write the same records.
#[test]
fn the_records_do_not_depend_on_the_number_of_workers() {
    let (one, three) = (scratch("one"), scratch("three"));
    let report = execute(&smoke(), &options(&one, 1)).unwrap();
    assert_eq!(
        (report.ran, report.skipped, report.failed),
        (smoke().len(), 0, 0)
    );
    execute(&smoke(), &options(&three, 3)).unwrap();
    assert_eq!(records(&one), records(&three));
    assert_eq!(records(&one).len(), smoke().len());
    let _ = (fs::remove_dir_all(one), fs::remove_dir_all(three));
}

/// A run cut in the middle of a line resumes, drops the torn line and repeats nothing.
#[test]
fn an_interrupted_run_resumes_without_losing_or_repeating_a_record() {
    let (whole, cut) = (scratch("whole"), scratch("cut"));
    execute(&smoke(), &options(&whole, 2)).unwrap();
    let half: Vec<Task> = smoke().into_iter().take(smoke().len() / 2).collect();
    execute(&half, &options(&cut, 2)).unwrap();
    let torn = cut.join(Study::DifPower.name()).join("records.csv");
    fs::OpenOptions::new()
        .append(true)
        .open(&torn)
        .unwrap()
        .write_all(b"dif-power,n=400 a=10 k=4,1,123,4")
        .unwrap();
    let report = execute(&smoke(), &options(&cut, 2)).unwrap();
    assert_eq!(report.skipped, half.len());
    assert_eq!(records(&whole), records(&cut));
    let again = execute(&smoke(), &options(&cut, 2)).unwrap();
    assert_eq!((again.ran, again.skipped), (0, smoke().len()));
    let _ = (fs::remove_dir_all(whole), fs::remove_dir_all(cut));
}

/// Records written under another schema are refused, never mixed with new ones.
#[test]
fn records_of_another_schema_are_refused() {
    let out = scratch("schema");
    let dir = out.join(Study::DifNull.name());
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("records.csv"), "study,cell,something,else\n").unwrap();
    let only = tasks(&[Study::DifNull], Grid::Smoke, Some(1), None);
    assert!(execute(&only, &options(&out, 1)).is_err());
    assert!(header(Study::DifNull).starts_with("study,cell,replicate,seed,elapsed_ms,"));
    let _ = fs::remove_dir_all(out);
}

/// One record per kind is pinned, time aside: a change to what a study measures shows here.
#[test]
fn a_record_of_each_kind_is_pinned() {
    let pins = [
        (
            Study::DifPower,
            "2d38bd1d3bd49967dcf0f5506a68e3144699ea3623d1b1d7843f1035520bd994",
        ),
        (
            Study::DtfError,
            "a90418aef1d564a130318bb81a96f54e49e4b54f96ba11cad15dbabf8a7b8b3b",
        ),
        (
            Study::BridgingSweep,
            "e026c1759e184f67d495dbc3dc3d0aaec2687cd90858932cebc7e18f700cec5a",
        ),
        (
            Study::BridgingCapture,
            "a54dc4c4a4f453d8727c8618def851b67d72d62978c113d470901df99e451778",
        ),
        (
            Study::DifTwoAxes,
            "cdb16cef55d897094aae79448996a266a5e08fbd32f496ded217a550459b71a8",
        ),
        (
            Study::FloorPower,
            "4a45a1cfa8239ca42d195cd1bf35c9ee8969b53dfb9a6e0869cfb8b07280320b",
        ),
        (
            Study::FloorMisspec,
            "4eaebc9204692250ab53288e4205cf47069154f9eecd73ec2c45fe2ef687d781",
        ),
        (
            Study::FloorDtf,
            "72d6135a1ca48e03fc747563fc002eda58ed3572f1c9c588c0ad2084efa1f85f",
        ),
        (
            Study::BridgingLambda,
            "bed457a39859bb0b08b68122e7e7893334ff2d0b589f1d47e9899f42ff6f7782",
        ),
        (
            Study::BridgingExtra,
            "63f785c0941bf4d49c4a6dec35a56a0db6e4570989438d07be56553a50a2cf17",
        ),
    ];
    for (study, pin) in pins {
        let task = tasks(&[study], Grid::Smoke, Some(1), None).remove(0);
        let line = Record::new(&task, 0, run(&task)).line();
        let digest: String = Sha256::digest(line.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(digest, pin, "{}: {line}", study.name());
    }
}
