//! A stopped node's replica written as a node stores it, with the inputs that pin its cuts.

use super::support::{Fixture, NET, PRIVATE, PUBLIC};
use network::cid::cid;
use network::cut::{Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{DurableReplica, FeedWriter};
use node::production;
use std::fs;
use std::path::Path;

/// Writes `dir/replica` as a node stores it: the fixture's public and circle entries, then
/// each `(cut, view, signers)` signature as a member object on its signer's feed.
pub fn replica(f: &Fixture, dir: &Path, signed: &[(&Cut, u64, &[usize])]) {
    let mut replica =
        DurableReplica::open(&dir.join("replica"), f.tiers.writers().clone()).unwrap();
    let mut feeds: Vec<_> = [1, 2, 3, 11]
        .into_iter()
        .map(|seed| {
            (
                FeedWriter::from_seed(NET, [seed; 32]),
                TransparencyLog::new(),
            )
        })
        .collect();
    let mut put = |feed: usize, object: Vec<u8>| {
        let (writer, log) = &mut feeds[feed];
        let entry = writer.sign(log.append(cid(&object)));
        replica.insert(entry, object).unwrap();
    };
    put(0, b"trusted prefix".to_vec());
    put(0, PUBLIC.to_vec());
    put(3, PRIVATE.to_vec());
    let set = f.tiers.consortium().member_set_hash();
    for (cut, view, signers) in signed {
        for &member in *signers {
            let object = MemberObject::CutSignature {
                cut: (*cut).clone(),
                view: *view,
                member,
                signature: f.members[member].sign(&cut.checkpoint_in(*view, NET, set)),
                named: Vec::new(),
            };
            put(member, object.encode());
        }
    }
}

/// Member 0's configuration, with `dir` as its data directory.
pub fn member(f: &Fixture, dir: &Path) -> String {
    f.node_text(0).replace(
        &f.root.join("node-state").display().to_string(),
        &dir.display().to_string(),
    )
}

/// The production input pinning `target` after `previous`, publishing at `segment`.
pub fn input(
    f: &Fixture,
    previous: Option<&Cut>,
    target: &Cut,
    view: u64,
    segment: &str,
) -> production::Config {
    let write = |suffix: &str, cut: &Cut| {
        let path = f.root.join(format!("{segment}.{suffix}"));
        fs::write(&path, cut.encode()).unwrap();
        path
    };
    production::Config {
        previous_cut_file: previous.map(|cut| write("previous", cut)),
        target_cut_file: write("target", target),
        view,
        segment_file: f.root.join(segment),
        ..f.input()
    }
}

/// `input` as a `production.toml`.
pub fn text(input: &production::Config) -> String {
    let path = |p: &Path| format!("\"{}\"", p.display());
    let mut text = format!(
        "lineage = \"{}\"\ntarget_cut_file = {}\nview = {}\nexcluded = {:?}\nsegment_file = {}\n\
         data_shards = {}\nparity_shards = {}\n",
        input.lineage,
        path(&input.target_cut_file),
        input.view,
        input.excluded,
        path(&input.segment_file),
        input.data_shards,
        input.parity_shards
    );
    for (key, value) in [
        ("previous_cut_file", &input.previous_cut_file),
        ("root_statement_file", &input.root_statement_file),
        ("shape_file", &input.shape_file),
    ] {
        if let Some(value) = value {
            text += &format!("{key} = {}\n", path(value));
        }
    }
    text
}

/// Cut 0 over the fixture target's marks.
pub fn first(f: &Fixture) -> Cut {
    Cut {
        number: 0,
        epoch: 0,
        closes: false,
        marks: f.target.marks.clone(),
    }
}
