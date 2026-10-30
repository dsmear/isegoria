//! Manual holder installation (`docs/04` §Initial archive installation on a holder).

#[allow(dead_code)]
#[path = "support/holder.rs"]
mod holder;
#[cfg(unix)]
#[path = "support/process.rs"]
mod process;
#[allow(dead_code)]
#[path = "support/production.rs"]
mod support;
use holder::*;
use network::archive::{Layout, SignedManifest};
use network::codec::Reader;
use node::install::{install, Outcome};
use node::production::{self, Error};
use p2p::archive::{InstallError, Installed};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use support::*;

const SEEDS: [u8; 3] = [41, 42, 43];
const PRINT: &str = "--transport-public-key";
const RFC8032_SEED: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
const RFC8032_PUBLIC: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

fn certified(f: &Fixture, name: &str, layout: Layout) -> (PathBuf, PathBuf) {
    let mut input = f.input();
    input.data_shards = layout.data_shards;
    input.parity_shards = layout.parity_shards;
    let bundle = f.root.join(name);
    production::prepare(&f.node(0), &input, &bundle).unwrap();
    let shares: Vec<_> = (0..2)
        .map(|i| {
            let share = f.root.join(format!("{name}-share-{i}.bin"));
            production::sign(&f.node(i), &input, &bundle.join("manifest.bin"), &share).unwrap();
            share
        })
        .collect();
    let certificate = f.root.join(format!("{name}-certificate.bin"));
    let manifest = bundle.join("manifest.bin");
    production::combine(&f.node(2), &input, &manifest, &shares, &certificate).unwrap();
    (bundle, certificate)
}

fn record(f: &Fixture, name: &str, certificate: &Path) -> PathBuf {
    let digest = SignedManifest::decode(&fs::read(certificate).unwrap())
        .unwrap()
        .manifest
        .digest();
    f.root
        .join(name)
        .join("replica/archive")
        .join(format!("{}.arc", node::hex(&digest)))
}

/// The retained indices of a holder record, in its documented encoding.
fn indices(path: &Path) -> Vec<u64> {
    let bytes = fs::read(path).unwrap();
    let mut r = Reader::new(&bytes);
    assert_eq!(r.u8().unwrap(), 1);
    r.fixed::<32>().unwrap();
    r.field().unwrap();
    let count = r.u64().unwrap();
    let out = (0..count)
        .map(|_| {
            let index = r.u64().unwrap();
            r.field().unwrap();
            index
        })
        .collect();
    r.finish().unwrap();
    out
}

fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap())
        .map(|e| {
            (
                e.file_name().to_string_lossy().into(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

#[cfg(unix)]
fn identity(path: &Path) -> u64 {
    std::os::unix::fs::MetadataExt::ino(&fs::metadata(path).unwrap())
}

#[cfg(not(unix))]
fn identity(_: &Path) -> u64 {
    0
}

/// The index that `roster` assigns to its first seed's transport key.
fn assigned(f: &Fixture, certificate: &Path, roster: &[u8]) -> usize {
    let signed = SignedManifest::decode(&fs::read(certificate).unwrap()).unwrap();
    let keys: Vec<[u8; 32]> = roster
        .iter()
        .map(|s| {
            network::consortium::Member::from_seed([*s; 32])
                .public()
                .to_bytes()
        })
        .collect();
    let verified = signed.verify(&f.context()).unwrap();
    let placement = verified.placement(&f.context(), &keys).unwrap();
    placement.iter().position(|k| *k == keys[0]).unwrap()
}

// Context, certificate, marker, content and identity faults leave the store untouched (AT-NET-33).
#[test]
fn refused_inputs_never_open_the_store() {
    let f = Fixture::new("install-refusals", false);
    let (bundle, certificate) = certified(
        &f,
        "bundle",
        Layout {
            data_shards: 2,
            parity_shards: 1,
        },
    );
    let (other, other_certificate) = certified(
        &f,
        "other",
        Layout {
            data_shards: 1,
            parity_shards: 1,
        },
    );
    let share = f.root.join("bundle-share-0.bin");
    let text = holder(
        &f,
        "holder",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &job(&f, &certificate, &SEEDS),
    );
    let dir = f.root.join("holder");
    let untouched = || {
        let names: Vec<_> = snapshot(&dir).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["transport.key".to_string()]);
    };
    let cases = [
        ("view", text.replace("view = 4", "view = 5")),
        (
            "lineage",
            text.replace(&node::hex(&LINEAGE), &node::hex(&[0; 32])),
        ),
        (
            "predecessor",
            text.replace(
                &format!(
                    "previous_cut_file = \"{}\"\n",
                    f.root.join("previous.cut").display()
                ),
                "",
            ),
        ),
        (
            "target",
            text.replace(
                &f.root.join("target.cut").display().to_string(),
                &f.root.join("previous.cut").display().to_string(),
            ),
        ),
        (
            "excluded-signer",
            text.replace("excluded = []", "excluded = [1]"),
        ),
        (
            "one-signature",
            text.replace(
                &certificate.display().to_string(),
                &share.display().to_string(),
            ),
        ),
        ("threshold", text.replace("threshold = 2", "threshold = 3")),
        (
            "other-certificate",
            text.replace(
                &certificate.display().to_string(),
                &other_certificate.display().to_string(),
            ),
        ),
        ("unassigned", text.replace(&public(41), &public(44))),
        (
            "no-maintenance",
            text.split("\n[archive_maintenance]")
                .next()
                .unwrap()
                .to_string(),
        ),
    ];
    for (name, text) in &cases {
        assert!(install(&config(text), &bundle).is_err(), "{name}");
        untouched();
    }
    let valid = config(&text);
    assert!(install(&valid, &other).is_err());
    let marker = fs::read_to_string(bundle.join("COMPLETE")).unwrap();
    let hex = marker.trim_end().rsplit(' ').next().unwrap();
    let segment = fs::read(bundle.join("segment.bin")).unwrap();
    let mut altered = f.segment.clone();
    altered.entries[0].1[0] ^= 1;
    let altered = altered.encode().unwrap();
    let index = assigned(&f, &certificate, &SEEDS);
    let shard_path = bundle.join(format!("shard-{index:03}.bin"));
    let shard = fs::read(&shard_path).unwrap();
    let mut flipped = shard.clone();
    flipped[0] ^= 1;
    let edits: Vec<(&str, PathBuf, Option<Vec<u8>>)> = vec![
        ("no-marker", bundle.join("COMPLETE"), None),
        (
            "upper-marker",
            bundle.join("COMPLETE"),
            Some(marker.replace(hex, &hex.to_uppercase()).into_bytes()),
        ),
        (
            "open-marker",
            bundle.join("COMPLETE"),
            Some(marker.trim_end().as_bytes().to_vec()),
        ),
        ("segment", bundle.join("segment.bin"), Some(altered)),
        (
            "segment-codec",
            bundle.join("segment.bin"),
            Some([segment.clone(), vec![0]].concat()),
        ),
        ("no-segment", bundle.join("segment.bin"), None),
        ("shard", shard_path.clone(), Some(flipped)),
        ("short-shard", shard_path.clone(), Some(shard[1..].to_vec())),
        ("no-shard", shard_path.clone(), None),
    ];
    for (name, path, bytes) in edits {
        let saved = fs::read(&path).unwrap();
        match bytes {
            Some(bytes) => fs::write(&path, bytes).unwrap(),
            None => fs::remove_file(&path).unwrap(),
        }
        assert!(install(&valid, &bundle).is_err(), "{name}");
        untouched();
        fs::write(&path, saved).unwrap();
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .open(bundle.join("segment.bin"))
        .unwrap();
    file.set_len(network::archive::MAX_SEGMENT as u64 + 1)
        .unwrap();
    assert!(matches!(
        install(&valid, &bundle),
        Err(Error::Config(node::ConfigError::Bounds("archive.segment")))
    ));
    untouched();
    fs::write(bundle.join("segment.bin"), &segment).unwrap();
    fs::remove_file(dir.join("transport.key")).unwrap();
    assert!(install(&valid, &bundle).is_err());
    assert!(fs::read_dir(&dir).unwrap().next().is_none());
    fs::remove_dir(&dir).unwrap();
    assert!(install(&valid, &bundle).is_err());
    assert!(!dir.exists());
}

// Repeats write nothing, other records survive, full or foreign stores refuse (AT-NET-33).
#[test]
fn repeated_installs_preserve_existing_records_and_limits() {
    let f = Fixture::new("install-preserve", false);
    let (a, a_certificate) = certified(
        &f,
        "a",
        Layout {
            data_shards: 2,
            parity_shards: 1,
        },
    );
    let (b, b_certificate) = certified(
        &f,
        "b",
        Layout {
            data_shards: 1,
            parity_shards: 1,
        },
    );
    let first_index = assigned(&f, &a_certificate, &SEEDS);
    let moved = (44..=60)
        .map(|other| vec![41, 42, other])
        .find(|roster| assigned(&f, &a_certificate, roster) != first_index)
        .unwrap();
    let jobs = |roster: &[u8]| job(&f, &a_certificate, roster) + &job(&f, &b_certificate, &SEEDS);
    let first = config(&holder(
        &f,
        "holder",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &jobs(&SEEDS),
    ));
    let Outcome {
        index, installed, ..
    } = install(&first, &a).unwrap();
    assert_eq!((index, installed), (first_index, Installed::Written));
    assert_eq!(install(&first, &b).unwrap().installed, Installed::Written);
    let store = f.root.join("holder/replica/archive");
    let (a_record, b_record) = (
        record(&f, "holder", &a_certificate),
        record(&f, "holder", &b_certificate),
    );
    assert_eq!(indices(&a_record), vec![index as u64]);
    let before = (snapshot(&store), identity(&a_record));
    assert_eq!(install(&first, &a).unwrap().installed, Installed::Present);
    assert_eq!((snapshot(&store), identity(&a_record)), before);
    let b_bytes = fs::read(&b_record).unwrap();
    let second = config(&holder(
        &f,
        "holder",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &jobs(&moved),
    ));
    let outcome = install(&second, &a).unwrap();
    assert_eq!(outcome.installed, Installed::Written);
    let mut both = vec![index as u64, outcome.index as u64];
    both.sort();
    assert_eq!(indices(&a_record), both);
    assert_eq!(fs::read(&b_record).unwrap(), b_bytes);
    let mut damaged = fs::read(&a_record).unwrap();
    *damaged.last_mut().unwrap() ^= 1;
    fs::write(&a_record, damaged).unwrap();
    assert_eq!(install(&second, &a).unwrap().installed, Installed::Written);
    assert_eq!(indices(&a_record), vec![outcome.index as u64]);
    assert_eq!(fs::read(&b_record).unwrap(), b_bytes);
    for n in 0..510 {
        fs::write(store.join(format!(".junk-{n}.tmp")), b"").unwrap();
    }
    let full = snapshot(&store);
    assert!(matches!(
        install(&first, &a),
        Err(Error::Install(InstallError::Quota))
    ));
    assert_eq!(snapshot(&store), full);
    let foreign = config(&holder(
        &f,
        "foreign",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &jobs(&SEEDS),
    ));
    let bound = f.root.join("foreign/replica/archive");
    fs::create_dir_all(&bound).unwrap();
    fs::write(bound.join("config"), [1; 33]).unwrap();
    assert!(matches!(
        install(&foreign, &a),
        Err(Error::Install(InstallError::Configuration))
    ));
    assert_eq!(snapshot(&bound), vec![("config".to_string(), vec![1; 33])]);
}

fn prepared(f: &Fixture) -> (PathBuf, PathBuf) {
    let layout = Layout {
        data_shards: 2,
        parity_shards: 1,
    };
    certified(f, "bundle", layout)
}

// The data-directory lock is free only once the storage task has ended (AT-NET-33).
#[cfg(unix)]
#[test]
fn lock_lasts_until_the_storage_task_ends() {
    let f = Fixture::new("install-lock", false);
    let (bundle, certificate) = prepared(&f);
    for case in ["stop", "error", "cancel"] {
        let listen = match case {
            "error" => "/ip4/127.0.0.1/udp/0/quic-v1",
            _ => "/ip4/127.0.0.1/tcp/0",
        };
        let text = holder(&f, case, 41, listen, &job(&f, &certificate, &SEEDS));
        let plan = node::plan(config(&text)).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            match case {
                "stop" => node::run(plan, std::future::ready(())).await.unwrap(),
                "error" => assert!(node::run(plan, std::future::ready(())).await.is_err()),
                _ => {
                    let run = node::run(plan, std::future::pending());
                    tokio::pin!(run);
                    tokio::select! {
                        biased;
                        _ = &mut run => unreachable!("run ended without a stop"),
                        _ = std::future::ready(()) => {}
                    }
                }
            }
        });
        let alive = runtime.metrics().num_alive_tasks();
        let during = install(&config(&text), &bundle);
        let locked = matches!(during, Err(Error::Invalid("data directory in use")));
        assert_eq!(
            locked,
            alive > 0,
            "{case}: {alive} task(s) alive, {during:?}"
        );
        assert_eq!(alive, usize::from(case == "cancel"), "{case}");
        drop(runtime);
        assert!(install(&config(&text), &bundle).is_ok(), "{case}");
    }
}

// Transport keys are read within a bound, never rewritten, and refusals write nothing (AT-NET-33).
#[test]
fn transport_key_is_bounded_and_left_unchanged() {
    let f = Fixture::new("install-key", false);
    let (bundle, certificate) = prepared(&f);
    let text = holder(
        &f,
        "holder",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &job(&f, &certificate, &SEEDS),
    );
    let dir = f.root.join("holder");
    let key = dir.join("transport.key");
    let valid = fs::read(&key).unwrap();
    let padded = |n| [valid.clone(), vec![b' '; n]].concat();
    for (bytes, expected) in [
        (padded(65), node::ConfigError::Bounds("transport.key")),
        (vec![b'z'; 64], node::ConfigError::Hex("transport.key")),
        (
            valid[..63].to_vec(),
            node::ConfigError::Hex("transport.key"),
        ),
    ] {
        fs::write(&key, &bytes).unwrap();
        for result in [
            install(&config(&text), &bundle).map(|_| ()),
            node::plan(config(&text)).map(|_| ()).map_err(Error::Config),
        ] {
            assert!(
                matches!(&result, Err(Error::Config(e)) if *e == expected),
                "{result:?}"
            );
            assert_eq!(
                snapshot(&dir),
                vec![("transport.key".to_string(), bytes.clone())]
            );
        }
    }
    fs::write(&key, padded(64)).unwrap();
    assert_eq!(
        install(&config(&text), &bundle).unwrap().installed,
        Installed::Written
    );
    assert_eq!(fs::read(&key).unwrap(), padded(64));
}

// A FIFO in place of the transport key is refused without opening it (AT-NET-33).
#[cfg(unix)]
#[test]
fn fifo_transport_key_is_refused_without_blocking() {
    let f = Fixture::new("install-fifo", false);
    let (bundle, certificate) = prepared(&f);
    let text = holder(
        &f,
        "holder",
        41,
        "/ip4/127.0.0.1/tcp/0",
        &job(&f, &certificate, &SEEDS),
    );
    let dir = f.root.join("holder");
    let key = dir.join("transport.key");
    fs::remove_file(&key).unwrap();
    let made = std::process::Command::new("mkfifo")
        .arg(&key)
        .status()
        .unwrap();
    assert!(made.success());
    let bounds = node::ConfigError::Bounds("transport.key");
    assert!(matches!(install(&config(&text), &bundle), Err(Error::Config(e)) if e == bounds));
    assert!(matches!(node::plan(config(&text)), Err(e) if e == bounds));
    let names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("transport.key")]);
    assert!(std::os::unix::fs::FileTypeExt::is_fifo(
        &fs::symlink_metadata(&key).unwrap().file_type()
    ));
}

// The printed key is RFC 8032's public key and a roster entry; nothing is written (AT-NET-33).
#[test]
fn transport_public_key_prints_the_roster_entry() {
    let f = Fixture::new("public-key", false);
    let (bundle, certificate) = prepared(&f);
    let path = bare(&f, "vector", 41);
    let dir = f.root.join("vector");
    fs::write(dir.join("transport.key"), RFC8032_SEED).unwrap();
    let before = tree(&dir);
    assert_eq!(before.len(), 1);
    let out = cli(&[Path::new(PRINT), &path]);
    assert!(out.status.success());
    assert_eq!(out.stdout, format!("{RFC8032_PUBLIC}\n").into_bytes());
    assert!(out.stderr.is_empty());
    assert_eq!(tree(&dir), before);
    let names: Vec<_> = (0..SEEDS.len()).map(|i| format!("holder-{i}")).collect();
    let keys: Vec<_> = SEEDS
        .iter()
        .zip(&names)
        .map(|(seed, name)| {
            let line = ok(&[Path::new(PRINT), &bare(&f, name, *seed)]);
            assert_eq!(line, format!("{}\n", public(*seed)));
            line.trim_end().to_string()
        })
        .collect();
    let jobs = roster_job(&f, &certificate, &keys);
    let mut indices: Vec<_> = SEEDS
        .iter()
        .zip(&names)
        .map(|(seed, name)| {
            let text = holder(&f, name, *seed, "/ip4/127.0.0.1/tcp/0", &jobs);
            install(&config(&text), &bundle).unwrap().index
        })
        .collect();
    indices.sort();
    assert_eq!(indices, vec![0, 1, 2]);
}

// Missing, malformed, oversized or non-regular keys exit 2 and write nothing (AT-NET-33).
#[test]
fn transport_public_key_refusals_write_nothing() {
    let f = Fixture::new("public-key-refusals", false);
    let path = bare(&f, "holder", 41);
    let dir = f.root.join("holder");
    let key = dir.join("transport.key");
    let seed = fs::read(&key).unwrap();
    let refused = |expected: &str| {
        let out = cli(&[Path::new(PRINT), &path]);
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert_eq!(out.status.code(), Some(2), "{stderr}");
        assert!(out.stdout.is_empty());
        assert_eq!(
            stderr,
            format!("transport key {}: {expected}\n", key.display())
        );
    };
    let (bounds, hex) = ("Bounds(\"transport.key\")", "Hex(\"transport.key\")");
    for (bytes, expected) in [
        ([seed.clone(), vec![b' '; 65]].concat(), bounds),
        (vec![b'z'; 64], hex),
        (seed[..63].to_vec(), hex),
        (vec![0xff; 64], hex),
    ] {
        fs::write(&key, &bytes).unwrap();
        let before = tree(&dir);
        refused(expected);
        assert_eq!(tree(&dir), before);
    }
    fs::remove_file(&key).unwrap();
    refused("Read(NotFound)");
    assert!(tree(&dir).is_empty());
    fs::create_dir(&key).unwrap();
    refused(bounds);
    assert_eq!(tree(&dir), vec![(key.clone(), None)]);
    fs::remove_dir(&key).unwrap();
    #[cfg(unix)]
    {
        assert!(Command::new("mkfifo").arg(&key).status().unwrap().success());
        refused(bounds);
        assert_eq!(tree(&dir), vec![(key.clone(), None)]);
        fs::remove_file(&key).unwrap();
    }
    fs::remove_dir(&dir).unwrap();
    refused("Read(NotFound)");
    assert!(!dir.exists());
    let out = cli(&[Path::new(PRINT), &f.root.join("absent.toml")]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(out.stderr, b"configuration: Read(NotFound)\n");
}

#[cfg(unix)]
mod processes {
    use super::process::{address, until, Process};
    use super::*;
    use libp2p::{Multiaddr, PeerId};
    use p2p::archive::{fetch, Holder};

    async fn served(f: &Fixture, signed: &SignedManifest, holders: Vec<(usize, &str)>) {
        let holders = holders
            .into_iter()
            .map(|(index, address)| {
                let (address, peer) = address.split_once("/p2p/").unwrap();
                Holder {
                    index,
                    peer: peer.parse::<PeerId>().unwrap(),
                    address: address.parse::<Multiaddr>().unwrap(),
                }
            })
            .collect();
        let recovered = fetch(&f.context(), signed, holders, Duration::from_secs(10)).await;
        assert_eq!(recovered.unwrap().segment, f.segment);
    }

    // Prepared, co-signed and installed shards are served, kept healthy and exclusive (AT-NET-33).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn installed_shards_are_served_audited_and_exclusive() {
        let f = Fixture::new("install-workflow", false);
        let input = f.root.join("production.toml");
        fs::write(&input, format!("lineage = \"{}\"\nprevious_cut_file = \"{}\"\ntarget_cut_file = \"{}\"\nview = 4\nexcluded = []\nsegment_file = \"{}\"\ndata_shards = 2\nparity_shards = 1\n", node::hex(&LINEAGE), f.root.join("previous.cut").display(), f.root.join("target.cut").display(), f.root.join("segment.bin").display())).unwrap();
        let members: Vec<_> = (0..3)
            .map(|i| {
                let path = f.root.join(format!("member-{i}.toml"));
                fs::write(&path, f.node_text(i)).unwrap();
                path
            })
            .collect();
        let bundle = f.root.join("bundle");
        let digest = ok(&[Path::new("--archive-prepare"), &members[0], &input, &bundle]);
        let manifest = bundle.join("manifest.bin");
        let shares = [f.root.join("a.share"), f.root.join("b.share")];
        for (member, share) in members.iter().zip(&shares) {
            assert_eq!(
                ok(&[
                    Path::new("--archive-sign"),
                    member,
                    &input,
                    &manifest,
                    share
                ]),
                digest
            );
        }
        let certificate = f.root.join("certificate.bin");
        let combine = [
            Path::new("--archive-combine"),
            &members[2],
            &input,
            &manifest,
            &certificate,
        ];
        assert_eq!(
            ok(&[&combine[..], &[shares[1].as_path(), shares[0].as_path()]].concat()),
            digest
        );
        let signed = SignedManifest::decode(&fs::read(&certificate).unwrap()).unwrap();
        let addresses: Vec<_> = SEEDS.iter().map(|s| address(*s)).collect();
        let configs: Vec<_> = SEEDS
            .iter()
            .zip(&addresses)
            .enumerate()
            .map(|(i, (seed, address))| {
                let listen = address.split("/p2p/").next().unwrap();
                let name = format!("holder-{i}");
                let text = holder(&f, &name, *seed, listen, &job(&f, &certificate, &SEEDS));
                let path = f.root.join(format!("{name}.toml"));
                fs::write(&path, text).unwrap();
                path
            })
            .collect();
        let install = Path::new("--archive-install");
        let mut indices = Vec::new();
        for config in &configs {
            let out = ok(&[install, config, &bundle]);
            let (printed, state) = out.split_once('\n').unwrap();
            assert_eq!(format!("{printed}\n"), digest);
            let index: usize = state
                .strip_prefix("shard ")
                .unwrap()
                .strip_suffix(" installed\n")
                .unwrap()
                .parse()
                .unwrap();
            indices.push(index);
        }
        let mut sorted = indices.clone();
        sorted.sort();
        assert_eq!(sorted, vec![0, 1, 2]);
        let first = record(&f, "holder-0", &certificate);
        let bytes = fs::read(&first).unwrap();
        assert!(ok(&[install, &configs[0], &bundle])
            .ends_with(&format!("shard {} present\n", indices[0])));
        assert_eq!(fs::read(&first).unwrap(), bytes);
        let mut nodes: Vec<_> = configs.iter().map(|c| Process::start(c)).collect();
        for (i, index) in indices.iter().enumerate() {
            let status = f.root.join(format!("holder-{i}/archive-status"));
            until(&status, |s| {
                s.contains(&format!("Ok(Healthy {{ index: {index} }})"))
            })
            .await;
        }
        served(
            &f,
            &signed,
            vec![
                (indices[0], addresses[0].as_str()),
                (indices[1], addresses[1].as_str()),
            ],
        )
        .await;
        let refused = cli(&[install, &configs[1], &bundle]);
        assert_eq!(refused.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&refused.stderr).contains("data directory in use"));
        let status = Process::start(&configs[1])
            .exited(Duration::from_secs(10))
            .await;
        assert_eq!(status.code(), Some(1));
        nodes[0].terminate(Duration::from_secs(5)).await;
        served(
            &f,
            &signed,
            vec![
                (indices[1], addresses[1].as_str()),
                (indices[2], addresses[2].as_str()),
            ],
        )
        .await;
        let status = f.root.join("holder-0/archive-status");
        fs::write(&status, "stale").unwrap();
        nodes[0] = Process::start(&configs[0]);
        until(&status, |s| {
            s.contains(&format!("Ok(Healthy {{ index: {} }})", indices[0]))
        })
        .await;
        served(
            &f,
            &signed,
            vec![
                (indices[0], addresses[0].as_str()),
                (indices[2], addresses[2].as_str()),
            ],
        )
        .await;
        assert_eq!(fs::read(&first).unwrap(), bytes);
        for node in &mut nodes {
            node.terminate(Duration::from_secs(5)).await;
        }
    }

    // The key is printed beside a running node, which keeps its key and lock (AT-NET-33).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn transport_public_key_reads_beside_a_running_node() {
        let f = Fixture::new("public-key-running", false);
        let path = bare(&f, "holder", 41);
        let key = f.root.join("holder/transport.key");
        let seed = fs::read(&key).unwrap();
        let mut node = Process::start(&path);
        until(&f.root.join("holder/status"), |s| !s.is_empty()).await;
        assert_eq!(ok(&[Path::new(PRINT), &path]), format!("{}\n", public(41)));
        assert_eq!(fs::read(&key).unwrap(), seed);
        let second = Process::start(&path).exited(Duration::from_secs(10)).await;
        assert_eq!(second.code(), Some(1));
        node.terminate(Duration::from_secs(5)).await;
    }
}
