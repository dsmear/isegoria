use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

const PUBLIC: [u8; 32] = [1; 32];
const CIRCLE: [u8; 32] = [2; 32];

fn scratch(name: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("record-{name}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn every_record() -> Vec<Record> {
    let states = [State::Unused, State::Opening, State::Written];
    let mut stops = vec![None];
    for t in [Tier::Public, Tier::Circle] {
        stops.extend(
            [
                Stop::Lost(t),
                Stop::Inconsistent(t),
                Stop::SetAside(t),
                Stop::OwnEntry(t),
            ]
            .map(Some),
        );
    }
    let mut out = Vec::new();
    for a in states {
        for b in states {
            for stop in &stops {
                out.push(Record {
                    states: [a, b],
                    stop: *stop,
                    ..Record::new(PUBLIC, CIRCLE)
                });
            }
        }
    }
    out
}

fn fail_before(_: &Path, _: &Record) -> io::Result<()> {
    Err(io::Error::other("before the rename"))
}

fn fail_after(path: &Path, record: &Record) -> io::Result<()> {
    store(path, record)?;
    Err(io::Error::other("after the rename"))
}

// Every record round-trips; any byte changed, cut or added is unreadable, never fresh (AT-NET-36).
#[test]
fn records_have_one_encoding() {
    for record in every_record() {
        let bytes = record.encode();
        assert_eq!(bytes.len(), LEN);
        assert_eq!(Record::decode(&bytes), Some(record.clone()));
        for i in 0..LEN {
            for flip in [1u8, 0x80] {
                let mut bad = bytes.clone();
                bad[i] ^= flip;
                assert_eq!(Record::decode(&bad), None, "byte {i}");
            }
        }
        assert_eq!(Record::decode(&bytes[..LEN - 1]), None);
        assert_eq!(Record::decode(&[bytes.as_slice(), &[0]].concat()), None);
    }
}

// Values outside the states, the stop kinds or the feeds are refused with a valid sum (AT-NET-36).
#[test]
fn unknown_values_are_refused_even_summed() {
    let resum = |mut body: Vec<u8>| {
        body.truncate(BODY);
        let sum = cid(&body);
        body.extend_from_slice(&sum.0);
        body
    };
    let base = Record::new(PUBLIC, CIRCLE).encode();
    for (at, value) in [(80, 3), (81, 9), (82, 5), (83, 2), (83, 1)] {
        let mut bad = base.clone();
        bad[at] = value;
        assert_eq!(Record::decode(&resum(bad)), None, "{at} {value}");
    }
}

// Missing, unreadable, other keys and a kept stop are each read for what they are (AT-NET-36).
#[test]
fn opening_a_record() {
    let dir = scratch("open");
    let path = dir.join("identity");
    let open = |circle| Guard::open(Some(&path), PUBLIC, circle);
    assert_eq!(read(&path), Found::Missing);
    assert_eq!(open(Some(CIRCLE)).unwrap().stop(), Some(Stop::Missing));
    assert_eq!(
        Guard::open(None, PUBLIC, None).unwrap().stop(),
        Some(Stop::Missing)
    );
    assert!(!path.exists());
    create(&path, PUBLIC, CIRCLE).unwrap();
    let guard = open(Some(CIRCLE)).unwrap();
    assert_eq!(guard.stop(), None);
    assert_eq!(guard.state(Tier::Circle), State::Unused);
    assert!(open(None).unwrap().stop().is_none());
    assert!(open(Some([3; 32])).is_err());
    assert!(Guard::open(Some(&path), [3; 32], Some(CIRCLE)).is_err());
    let kept = Record {
        stop: Some(Stop::Lost(Tier::Circle)),
        ..Record::new(PUBLIC, CIRCLE)
    };
    store(&path, &kept).unwrap();
    assert_eq!(
        open(Some(CIRCLE)).unwrap().stop(),
        Some(Stop::Lost(Tier::Circle))
    );
    let mut torn = kept.encode();
    torn[90] ^= 1;
    std::fs::write(&path, &torn).unwrap();
    assert_eq!(open(Some(CIRCLE)).unwrap().stop(), Some(Stop::Unreadable));
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert_eq!(read(&path), Found::Unreadable);
}

/// `read` on its own thread: a read still blocked after 10 s fails the test, its FIFO unblocked.
fn read_within(path: &Path) -> Found {
    let (tx, rx) = std::sync::mpsc::channel();
    let owned = path.to_path_buf();
    std::thread::spawn(move || tx.send(read(&owned)));
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .unwrap_or_else(|_| {
            let writer = path.to_path_buf();
            std::thread::spawn(move || OpenOptions::new().write(true).open(writer));
            panic!("reading {} blocked", path.display())
        })
}

// A FIFO, a directory or an oversized file is unreadable at once and left as found (AT-NET-36).
#[test]
fn only_a_regular_file_is_read() {
    let dir = scratch("kinds");
    let path = dir.join("identity");
    let record = Record::new(PUBLIC, CIRCLE).encode();
    std::fs::write(&path, &record).unwrap();
    assert_eq!(
        read_within(&path),
        Found::Present(Record::new(PUBLIC, CIRCLE))
    );
    let over = [record.as_slice(), &[0]].concat();
    std::fs::write(&path, &over).unwrap();
    assert_eq!(read_within(&path), Found::Unreadable);
    assert_eq!(std::fs::read(&path).unwrap(), over);
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    assert_eq!(read_within(&path), Found::Unreadable);
    #[cfg(unix)]
    {
        std::fs::remove_dir(&path).unwrap();
        let made = std::process::Command::new("mkfifo").arg(&path).status();
        assert!(made.unwrap().success());
        assert_eq!(read_within(&path), Found::Unreadable);
        let mut guard = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
        assert_eq!(guard.halt(Stop::Lost(Tier::Public)), Stop::Unreadable);
        let kind = std::fs::symlink_metadata(&path).unwrap().file_type();
        assert!(std::os::unix::fs::FileTypeExt::is_fifo(&kind));
    }
    let names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["identity"]);
}

// A new record is never written over an existing file (AT-NET-36).
#[test]
fn creation_never_overwrites() {
    let dir = scratch("create");
    let path = dir.join("identity");
    std::fs::write(&path, b"kept").unwrap();
    assert!(create(&path, PUBLIC, CIRCLE).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"kept");
}

// A transition failing before or after its rename stops; the disk holds one state (AT-NET-36).
#[test]
fn failed_transitions_stop_the_member() {
    for (fault, on_disk) in [
        (
            fail_before as fn(&Path, &Record) -> io::Result<()>,
            State::Unused,
        ),
        (fail_after, State::Opening),
    ] {
        let dir = scratch("advance");
        let path = dir.join("identity");
        create(&path, PUBLIC, CIRCLE).unwrap();
        let mut guard = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
        guard.write = fault;
        assert_eq!(
            guard.advance(Tier::Public, State::Opening),
            Err(Stop::Failed)
        );
        assert_eq!(guard.stop(), Some(Stop::Failed));
        guard.write = store;
        assert_eq!(
            guard.advance(Tier::Public, State::Opening),
            Err(Stop::Failed)
        );
        let Found::Present(record) = read(&path) else {
            panic!("a readable record")
        };
        assert_eq!(record.state(Tier::Public), on_disk);
        assert_eq!(record.stop, None);
        let reopened = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
        assert_eq!(reopened.stop(), None);
        assert_eq!(reopened.state(Tier::Public), on_disk);
    }
    for (fault, on_disk) in [
        (
            fail_before as fn(&Path, &Record) -> io::Result<()>,
            State::Opening,
        ),
        (fail_after, State::Written),
    ] {
        let dir = scratch("written");
        let path = dir.join("identity");
        create(&path, PUBLIC, CIRCLE).unwrap();
        let mut guard = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
        guard.advance(Tier::Circle, State::Opening).unwrap();
        guard.write = fault;
        assert_eq!(
            guard.advance(Tier::Circle, State::Written),
            Err(Stop::Failed)
        );
        let Found::Present(record) = read(&path) else {
            panic!("a readable record")
        };
        assert_eq!(record.state(Tier::Circle), on_disk);
        assert_eq!(record.state(Tier::Public), State::Unused);
    }
}

// A durable stop is kept in the record; one that cannot be written still stops (AT-NET-36).
#[test]
fn stops_are_kept_or_held_in_memory() {
    let dir = scratch("halt");
    let path = dir.join("identity");
    create(&path, PUBLIC, CIRCLE).unwrap();
    let mut guard = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
    guard.write = fail_before;
    assert_eq!(
        guard.halt(Stop::OwnEntry(Tier::Public)),
        Stop::OwnEntry(Tier::Public)
    );
    assert_eq!(read(&path), Found::Present(Record::new(PUBLIC, CIRCLE)));
    assert_eq!(
        guard.advance(Tier::Public, State::Opening),
        Err(Stop::OwnEntry(Tier::Public))
    );
    guard.write = store;
    assert_eq!(
        guard.halt(Stop::Lost(Tier::Circle)),
        Stop::OwnEntry(Tier::Public)
    );
    let Found::Present(record) = read(&path) else {
        panic!("a readable record")
    };
    assert_eq!(record.stop, Some(Stop::Lost(Tier::Circle)));
    let mut failed = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
    assert_eq!(failed.halt(Stop::Failed), Stop::Lost(Tier::Circle));
    std::fs::write(&path, b"unreadable").unwrap();
    let mut unreadable = Guard::open(Some(&path), PUBLIC, Some(CIRCLE)).unwrap();
    assert_eq!(
        unreadable.halt(Stop::SetAside(Tier::Public)),
        Stop::Unreadable
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"unreadable");
}

// The status words of every stop read back, and nothing else does (AT-NET-36).
#[test]
fn stop_names_read_back() {
    let mut all = vec![Stop::Missing, Stop::Unreadable, Stop::Failed];
    for t in [Tier::Public, Tier::Circle] {
        all.extend([
            Stop::Lost(t),
            Stop::Inconsistent(t),
            Stop::SetAside(t),
            Stop::OwnEntry(t),
        ]);
    }
    for stop in all {
        assert_eq!(Stop::parse(&stop.name()), Some(stop));
    }
    for bad in [
        "",
        "lost",
        "lost both",
        "failed public",
        "own-entry  circle",
        "none",
    ] {
        assert_eq!(Stop::parse(bad), None, "{bad}");
    }
}
