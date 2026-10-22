//! A node's configuration and files (`docs/04` §A node process, T78): what refuses to start,
//! the transport key kept across starts, and the status file's one text per status.

use network::consortium::Member;
use node::{hex, parse, plan, render, Config, ConfigError};
use p2p::Status;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("node_config")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn key(seed: u8) -> String {
    hex(&Member::from_seed([seed; 32]).public().to_bytes())
}

/// A member's configuration with `edit` applied to its text.
fn load(name: &str, edit: impl Fn(String) -> String) -> Result<Config, ConfigError> {
    let dir = scratch(name);
    let key_file = dir.join("key");
    std::fs::write(&key_file, hex(&[1; 32])).unwrap();
    let text = format!(
        "network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/0\"]\n\
         writers = [\"{a}\"]\nkey_file = \"{}\"\n\n[consortium]\nmembers = [\"{a}\", \"{b}\"]\n\
         threshold = 2\n\n[member]\nindex = 0\nevery_ms = 100\ncuts_per_epoch = 2\n\
         patience = 5\nview_patience = 10\n",
        hex(&[7; 32]),
        dir.join("data").display(),
        key_file.display(),
        a = key(1),
        b = key(2),
    );
    let path = dir.join("node.toml");
    std::fs::write(&path, edit(text)).unwrap();
    Config::load(&path)
}

fn planned(name: &str, edit: impl Fn(String) -> String) -> Result<(), ConfigError> {
    plan(load(name, edit)?).map(|_| ())
}

/// AT-NET-21: a valid configuration plans; bad hex, a threshold outside `1..=n`, a repeated
/// member, a member without a key, a bad address and an unknown field refuse to start.
#[test]
fn at_net_21_what_refuses_to_start() {
    assert_eq!(planned("valid", |t| t), Ok(()));
    assert_eq!(
        planned("hex", |t| t.replace(&hex(&[7; 32]), "07")),
        Err(ConfigError::Hex("network_id"))
    );
    assert_eq!(
        planned("zero", |t| t.replace("threshold = 2", "threshold = 0")),
        Err(ConfigError::Threshold)
    );
    assert_eq!(
        planned("three", |t| t.replace("threshold = 2", "threshold = 3")),
        Err(ConfigError::Threshold)
    );
    assert_eq!(
        planned("twice", |t| t.replace(&key(2), &key(1))),
        Err(ConfigError::Key("consortium.members"))
    );
    assert_eq!(
        planned("keyless", |t| t
            .lines()
            .filter(|l| !l.starts_with("key_file"))
            .collect::<Vec<_>>()
            .join("\n")),
        Err(ConfigError::MemberWithoutKey)
    );
    assert!(matches!(
        planned("addr", |t| t.replace("/ip4/127.0.0.1/tcp/0", "nowhere")),
        Err(ConfigError::Address(_))
    ));
    assert!(matches!(
        planned("unknown", |t| t + "\ncolour = \"blue\"\n"),
        Err(ConfigError::Parse(_))
    ));
}

/// AT-NET-21: the transport key is made at the first start and kept, so the peer id stays.
#[test]
fn at_net_21_the_peer_id_survives_a_restart() {
    let first = plan(load("transport", |t| t).unwrap()).unwrap();
    let path = first.status.with_file_name("transport.key");
    let kept = std::fs::read_to_string(&path).unwrap();
    assert_eq!(kept.len(), 64);
    let config = load("transport-twin", |t| t).unwrap();
    std::fs::create_dir_all(&config.data_dir).unwrap();
    std::fs::write(config.data_dir.join("transport.key"), &kept).unwrap();
    let second = plan(config).unwrap();
    assert_eq!(first.transport.public(), second.transport.public());
}

/// AT-NET-21: a status reads back as written; other text is refused.
#[test]
fn at_net_21_the_status_file_has_one_text() {
    let status = Status {
        caught_up: true,
        circle_caught_up: true,
        peers: 3,
        bound: 2,
        equivocations: 1,
        conflict: false,
        cuts: vec![[1; 32], [2; 32]],
        beacons: vec![(Some([9; 32]), vec![]), (None, vec![0, 2])],
    };
    let text = render(&status);
    assert_eq!(parse(&text), Some(status.clone()));
    assert_eq!(parse(""), Some(Status::default()));
    for bad in [
        text.replace("cut 1", "cut 3"),
        text.replace("beacon 1", "beacon 0"),
        text.replace("[0,2]", "0,2"),
        text.replace("peers 3", "peers three"),
        text.replace("bound 2", "bound -2"),
        text.clone() + "spare line\n",
    ] {
        assert_eq!(parse(&bad), None, "{bad}");
    }
}

/// AT-NET-21: unset, the periodic pull and the status are once a second.
#[test]
fn at_net_21_the_default_periods() {
    let planned = plan(load("defaults", |t| t).unwrap()).unwrap();
    assert_eq!(planned.p2p.sync_every, std::time::Duration::from_secs(1));
    assert_eq!(planned.status_every, std::time::Duration::from_secs(1));
}

/// The member configuration made tiered: circle keys of seeds 11 and 12, writers all four.
fn tiered(circle_key: &std::path::Path) -> impl Fn(String) -> String + '_ {
    move |t: String| {
        let writers = format!(
            "[\"{}\", \"{}\", \"{}\", \"{}\"]",
            key(1),
            key(2),
            key(11),
            key(12)
        );
        t.replace(
            &format!("writers = [\"{}\"]", key(1)),
            &format!("writers = {writers}"),
        )
        .replace(
            "\n\n[consortium]",
            &format!(
                "\ncircle_key_file = \"{}\"\n\n[consortium]",
                circle_key.display()
            ),
        )
        .replace(
            "threshold = 2\n",
            &format!(
                "threshold = 2\ncircle = [\"{}\", \"{}\"]\n",
                key(11),
                key(12)
            ),
        )
    }
}

/// AT-NET-23: a tiered member plans both feeds; bad circle keys, writers or files refuse to start.
#[test]
fn at_net_23_a_tiered_configuration() {
    let dir = scratch("circle-keys");
    let (own, other) = (dir.join("own"), dir.join("other"));
    std::fs::write(&own, hex(&[11; 32])).unwrap();
    std::fs::write(&other, hex(&[12; 32])).unwrap();
    let member = plan(load("tiered", tiered(&own)).unwrap()).unwrap();
    assert!(member.p2p.tiers.is_some() && member.p2p.consortium.is_none());
    let circle = member.p2p.circle.as_ref().expect("a circle feed");
    assert!(circle.dir.ends_with("circle"));
    assert_eq!(hex(&circle.writer.public().to_bytes()), key(11));
    let reader = |t: String| {
        let t = tiered(&own)(t);
        let t = t.replace("\n\n[member]", "\n\n[unused]");
        let t = t.split("\n\n[unused]").next().unwrap().to_string() + "\n";
        t.lines()
            .filter(|l| !l.starts_with("key_file") && !l.starts_with("circle_key_file"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let read = plan(load("tiered-reader", reader).unwrap()).unwrap();
    assert!(read.p2p.tiers.is_some() && read.p2p.circle.is_none() && read.p2p.member.is_none());
    let edit = |name: &str, f: &dyn Fn(String) -> String| planned(name, |t| f(tiered(&own)(t)));
    assert_eq!(
        edit("one-circle", &|t| t.replace(
            &format!(", \"{}\"]\n\n[member]", key(12)),
            "]\n\n[member]"
        )),
        Err(ConfigError::Tiers(network::tier::TierError::Count))
    );
    let repeated = network::tier::TierError::Repeated {
        key: Member::from_seed([2; 32]).public().to_bytes(),
    };
    assert_eq!(
        edit("member-key", &|t| t.replace(
            &format!("\"{}\"]\n\n[member]", key(12)),
            &format!("\"{}\"]\n\n[member]", key(2))
        )),
        Err(ConfigError::Tiers(repeated))
    );
    assert_eq!(
        edit("writers", &|t| t.replace(
            &format!(", \"{}\"]\nkey_file", key(12)),
            "]\nkey_file"
        )),
        Err(ConfigError::Writers)
    );
    assert_eq!(
        planned("other-circle", tiered(&other)),
        Err(ConfigError::CircleKey)
    );
    assert_eq!(
        edit("no-circle-file", &|t| t
            .lines()
            .filter(|l| !l.starts_with("circle_key_file"))
            .collect::<Vec<_>>()
            .join("\n")),
        Err(ConfigError::CircleKey)
    );
    assert_eq!(
        planned("untiered-circle", |t| t.replace(
            "\n\n[consortium]",
            &format!("\ncircle_key_file = \"{}\"\n\n[consortium]", own.display())
        )),
        Err(ConfigError::CircleKey)
    );
}
