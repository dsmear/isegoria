use super::*;
use futures::io::Cursor;
use libp2p::request_response::Codec;
use network::archive::{Layout, Manifest};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, Mark};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use std::sync::atomic::{AtomicU64, Ordering};

const NET: [u8; 32] = [93; 32];
struct Fixture {
    tiers: Tiers,
    members: Vec<Member>,
    target: Cut,
    replica: Replica,
}
impl Fixture {
    fn new() -> Self {
        let members: Vec<_> = (1..=3).map(|n| Member::from_seed([n; 32])).collect();
        let tiers = Self::tiers(2);
        let mut replica = Replica::new(tiers.writers().clone());
        let mut marks = Vec::new();
        for (seed, object) in [
            (1, b"public archive".as_slice()),
            (11, b"private sentinel".as_slice()),
        ] {
            let entry = FeedWriter::from_seed(NET, [seed; 32])
                .sign(TransparencyLog::new().append(cid(object)));
            marks.push(Mark {
                writer: entry.writer,
                len: 1,
                head: entry.entry.hash,
            });
            replica.insert(entry, object.to_vec()).unwrap();
        }
        marks.sort_by_key(|m| m.writer);
        Self {
            tiers,
            members,
            target: Cut {
                number: 0,
                epoch: 0,
                closes: false,
                marks,
            },
            replica,
        }
    }
    fn tiers(threshold: usize) -> Tiers {
        Tiers::new(
            NET,
            Consortium::new(
                (1..=3)
                    .map(|n| Member::from_seed([n; 32]).public())
                    .collect(),
                threshold,
            ),
            &(11..=13)
                .map(|n| FeedWriter::from_seed(NET, [n; 32]).public())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }
    fn context(&self) -> Context<'_> {
        Context {
            tiers: &self.tiers,
            lineage: [94; 32],
            previous: None,
            target: &self.target,
            view: 0,
            excluded: &[],
        }
    }
    fn sign(&self, manifest: Manifest) -> SignedManifest {
        let cp = manifest.checkpoint();
        SignedManifest {
            manifest,
            signatures: self
                .members
                .iter()
                .take(2)
                .enumerate()
                .map(|(i, m)| (i, m.sign(&cp)))
                .collect(),
        }
    }
    fn archive(&self) -> (Segment, SignedManifest, Vec<Vec<u8>>) {
        let ctx = self.context();
        let cp = self
            .target
            .checkpoint_in(0, NET, self.tiers.consortium().member_set_hash());
        let signatures = self
            .members
            .iter()
            .take(2)
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        let segment = Segment::build(&ctx, &self.replica, signatures, None).unwrap();
        let built = BuiltArchive::build(
            &ctx,
            &segment,
            Layout {
                data_shards: 3,
                parity_shards: 2,
            },
        )
        .unwrap();
        (segment, self.sign(built.manifest), built.shards)
    }
    fn config(&self, dir: Option<PathBuf>) -> crate::Config {
        crate::Config {
            writers: self.tiers.writers().clone(),
            sync_every: Duration::from_secs(3600),
            own: None,
            circle: None,
            store: dir,
            member: None,
            consortium: None,
            tiers: Some(Self::tiers(2)),
            relay: None,
            gateway: None,
        }
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "isegoria-archive-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// Shard streams enforce exact requests and inclusive response limits (AT-NET-27).
#[tokio::test]
async fn wire_bounds() {
    let mut codec = ShardCodec;
    let protocol = protocol(&NET);
    let req = request([1; 32], 4);
    assert_eq!(
        codec
            .read_request(&protocol, &mut Cursor::new(req.clone()))
            .await
            .unwrap(),
        req
    );
    for bad in [
        req[..40].to_vec(),
        [req.as_slice(), &[0]].concat(),
        vec![2; 41],
    ] {
        assert!(codec
            .read_request(&protocol, &mut Cursor::new(bad))
            .await
            .is_err());
    }
    assert_eq!(
        codec
            .read_response(&protocol, &mut Cursor::new(vec![0; REPLY_LIMIT as usize]))
            .await
            .unwrap()
            .len(),
        REPLY_LIMIT as usize
    );
    assert!(codec
        .read_response(
            &protocol,
            &mut Cursor::new(vec![0; REPLY_LIMIT as usize + 1])
        )
        .await
        .is_err());
}

// Echo, exact width, and signed hash are all required for a shard (AT-NET-27).
#[test]
fn reply_authentication() {
    let f = Fixture::new();
    let (_, signed, shards) = f.archive();
    let valid = reply(signed.manifest.digest(), 0, &shards[0]);
    assert_eq!(checked(&signed, 0, &valid), Some(shards[0].clone()));
    for offset in [0, 1, 2, 34, 42] {
        let mut bad = valid.clone();
        bad[offset] ^= 1;
        assert!(checked(&signed, 0, &bad).is_none());
    }
    assert!(checked(&signed, 1, &valid).is_none());
    assert!(checked(&signed, 0, &valid[..valid.len() - 1]).is_none());
    assert!(checked(&signed, 0, &[valid.as_slice(), &[0]].concat()).is_none());
}

// Admission rechecks full semantics, selected indices, and exact configuration (AT-NET-27).
#[test]
fn admission_and_certificate_scope() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let ctx = f.context();
    for indices in [&[][..], &[0, 0], &[5]] {
        assert!(matches!(
            Prepared::new(&ctx, &signed, &segment, indices),
            Err(InstallError::Indices)
        ));
    }
    let mut store = Store::open(&f.tiers, None).unwrap();
    let lower = Fixture::tiers(1);
    let lower_ctx = Context {
        tiers: &lower,
        ..ctx
    };
    let mut one = signed.clone();
    one.signatures.truncate(1);
    let prepared = Prepared::new(&lower_ctx, &one, &segment, &[0]).unwrap();
    assert_eq!(
        store.install(&f.tiers, prepared),
        Err(InstallError::Configuration)
    );
    assert!(store.records.is_empty());
    let mut changed = segment.clone();
    changed.lineage = [9; 32];
    assert!(Prepared::new(&ctx, &signed, &changed, &[0]).is_err());
    let mut stale_ctx = ctx;
    stale_ctx.lineage = [9; 32];
    assert!(signed.verify(&stale_ctx).is_err());
    assert!(signed.verify_certificate(&f.tiers).is_ok());
    assert!(one.verify_certificate(&f.tiers).is_err());
    assert!(one.verify_certificate(&lower).is_ok());
    let mut private = segment.clone();
    private.entries.extend(
        f.replica
            .ids()
            .filter_map(|id| f.replica.get(&id))
            .filter(|(e, _)| f.tiers.tier(&e.writer) == Some(network::tier::Tier::Circle))
            .map(|(e, b)| (e.clone(), b.to_vec())),
    );
    private.entries.sort_by_key(|(entry, _)| entry.id());
    let public_bytes = segment.encode().unwrap();
    assert!(public_bytes
        .windows(b"public archive".len())
        .any(|w| w == b"public archive"));
    assert!(!public_bytes
        .windows(b"private sentinel".len())
        .any(|w| w == b"private sentinel"));
    let bytes = private.encode().unwrap();
    let encoded = network::erasure::encode(&bytes, 3, 2);
    let mut w = Writer::new();
    w.u8(1)
        .fixed(&NET)
        .fixed(&ctx.lineage)
        .fixed(&f.tiers.consortium().member_set_hash())
        .u8(0)
        .u64(0)
        .fixed(&f.target.digest())
        .u64(0)
        .fixed(&cid(&bytes).0)
        .u64(bytes.len() as u64)
        .u64(3)
        .u64(2)
        .u64(5);
    for hash in &encoded.manifest {
        w.fixed(hash);
    }
    let forged = f.sign(Manifest::decode(&w.finish()).unwrap());
    assert!(forged.verify(&ctx).is_ok());
    assert!(Prepared::new(&ctx, &forged, &private, &[0]).is_err());
    assert!(store.records.is_empty());
}

// Persistent selected sets survive restart, reject bitrot/torn files, and repair (AT-NET-27).
#[test]
fn disk_restart_damage_and_canonical_order() {
    let f = Fixture::new();
    let (segment, signed, shards) = f.archive();
    let dir = Temp::new();
    let mut store = Store::open(&f.tiers, Some(dir.0.clone())).unwrap();
    store
        .install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[4, 0]).unwrap(),
        )
        .unwrap();
    let digest = signed.manifest.digest();
    let path = dir.0.join(format!("{}.arc", hex(&digest)));
    let bytes = fs::read(&path).unwrap();
    assert_eq!(
        bytes.len(),
        49 + signed.encode().unwrap().len() + 32 + 2 * shards[0].len()
    );
    let record = Record::decode(&bytes, &f.tiers, digest).unwrap();
    assert_eq!(
        record.shards.keys().copied().collect::<Vec<_>>(),
        vec![0, 4]
    );
    drop(store);
    let store = Store::open(&f.tiers, Some(dir.0.clone())).unwrap();
    assert_eq!(
        store.serve(&f.tiers, &request(digest, 0)),
        reply(digest, 0, &shards[0])
    );
    assert_eq!(store.serve(&f.tiers, &request(digest, 1)), [1, 0]);
    let mut corrupt = bytes.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    fs::write(&path, corrupt).unwrap();
    assert_eq!(store.serve(&f.tiers, &request(digest, 0)), [1, 0]);
    fs::write(&path, &bytes[..bytes.len() - 1]).unwrap();
    drop(store);
    let mut store = Store::open(&f.tiers, Some(dir.0.clone())).unwrap();
    assert_eq!(store.serve(&f.tiers, &request(digest, 0)), [1, 0]);
    store
        .install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[2]).unwrap(),
        )
        .unwrap();
    assert_eq!(store.serve(&f.tiers, &request(digest, 0)), [1, 0]);
    assert_eq!(
        store.serve(&f.tiers, &request(digest, 2)),
        reply(digest, 2, &shards[2])
    );
    assert!(matches!(
        Store::open(&Fixture::tiers(1), Some(dir.0.clone())),
        Err(InstallError::Configuration)
    ));
    fs::write(dir.0.join("config"), [1]).unwrap();
    assert!(matches!(
        Store::open(&f.tiers, Some(dir.0.clone())),
        Err(InstallError::Configuration)
    ));
    let noheader = Temp::new();
    fs::write(noheader.0.join("leftover.tmp"), [0]).unwrap();
    assert!(matches!(
        Store::open(&f.tiers, Some(noheader.0.clone())),
        Err(InstallError::Configuration)
    ));
    let mut unordered = Writer::new();
    unordered
        .u8(1)
        .fixed(&f.tiers.context())
        .field(&signed.encode().unwrap())
        .u64(2)
        .u64(4)
        .field(&shards[4])
        .u64(0)
        .field(&shards[0]);
    assert!(Record::decode(&unordered.finish(), &f.tiers, digest).is_none());
    assert!(Record::decode(&bytes, &f.tiers, [0; 32]).is_none());
    assert!(Record::decode(&[bytes.as_slice(), &[0]].concat(), &f.tiers, digest).is_none());
}

// Quotas count damaged archives, temporary entries, and actual bytes (AT-NET-27).
#[test]
fn conservative_quotas() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let dir = Temp::new();
    let mut store = Store::open(&f.tiers, Some(dir.0.clone())).unwrap();
    for n in 0..16 {
        fs::write(dir.0.join(format!("bad{n}.arc")), [0]).unwrap();
    }
    assert_eq!(store.usage(&f.tiers).unwrap().shards, 4096);
    assert_eq!(
        store.install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[0]).unwrap()
        ),
        Err(InstallError::Quota)
    );
    let damaged_path = dir
        .0
        .join(format!("{}.arc", hex(&signed.manifest.digest())));
    fs::rename(dir.0.join("bad0.arc"), &damaged_path).unwrap();
    store
        .install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[0]).unwrap(),
        )
        .unwrap();
    assert_ne!(
        store.serve(&f.tiers, &request(signed.manifest.digest(), 0)),
        [1, 0]
    );
    let committed = fs::read(&damaged_path).unwrap();
    let huge = File::create(dir.0.join("full.tmp")).unwrap();
    huge.set_len(MAX_BYTES - store.usage(&f.tiers).unwrap().bytes)
        .unwrap();
    assert_eq!(
        store.install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[1]).unwrap()
        ),
        Err(InstallError::Quota)
    );
    assert_eq!(fs::read(&damaged_path).unwrap(), committed);
    fs::remove_file(dir.0.join("full.tmp")).unwrap();
    fs::remove_file(&damaged_path).unwrap();
    for n in 1..16 {
        fs::remove_file(dir.0.join(format!("bad{n}.arc"))).unwrap();
    }
    for n in 0..511 {
        fs::write(dir.0.join(format!("{n}.tmp")), []).unwrap();
    }
    assert_eq!(
        store.install(
            &f.tiers,
            Prepared::new(&f.context(), &signed, &segment, &[0]).unwrap()
        ),
        Err(InstallError::Quota)
    );
    for n in 0..511 {
        fs::remove_file(dir.0.join(format!("{n}.tmp"))).unwrap();
    }
    let huge = File::create(dir.0.join("large.tmp")).unwrap();
    huge.set_len(MAX_BYTES).unwrap();
    assert!(matches!(store.usage(&f.tiers), Err(InstallError::Quota)));
    assert!(matches!(
        Store::open(&f.tiers, Some(dir.0.clone())),
        Err(InstallError::Quota)
    ));
    assert!(Usage {
        archives: 257,
        ..Usage::default()
    }
    .check()
    .is_err());
    assert_eq!(MAX_RECORD, (64 << 20) + MAX_MANIFEST + 49 + 4096);
}

// Five assigned-only real holders recover after two stop without a replica (AT-NET-27).
#[tokio::test]
async fn five_holders_two_lost_recover_and_reinstall() {
    let f = Fixture::new();
    let (segment, signed, shards) = f.archive();
    let mut nodes = Vec::new();
    let mut addresses = Vec::new();
    for i in 0..5 {
        let node =
            crate::Handle::spawn(identity::Keypair::generate_ed25519(), f.config(None)).unwrap();
        node.install_archive(&f.context(), &signed, &segment, &[i])
            .await
            .unwrap();
        let addr = node
            .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
            .await
            .unwrap();
        addresses.push((node.peer_id(), addr));
        nodes.push(Some(node));
    }
    nodes[1].take();
    nodes[4].take();
    tokio::task::yield_now().await;
    let holders = addresses
        .iter()
        .enumerate()
        .map(|(index, (peer, address))| Holder {
            index,
            peer: *peer,
            address: address.clone(),
        })
        .collect();
    let recovered = fetch(&f.context(), &signed, holders, Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(recovered.segment, segment);
    assert_eq!(
        recovered.repairs,
        vec![(1, shards[1].clone()), (4, shards[4].clone())]
    );
    let repaired =
        crate::Handle::spawn(identity::Keypair::generate_ed25519(), f.config(None)).unwrap();
    repaired
        .install_archive(&f.context(), &signed, &recovered.segment, &[1, 4])
        .await
        .unwrap();
    let addr = repaired
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let holders = [
        (0, addresses[0].0, addresses[0].1.clone()),
        (1, repaired.peer_id(), addr.clone()),
        (4, repaired.peer_id(), addr),
    ]
    .into_iter()
    .map(|(index, peer, address)| Holder {
        index,
        peer,
        address,
    })
    .collect();
    assert_eq!(
        fetch(&f.context(), &signed, holders, Duration::from_secs(5))
            .await
            .unwrap()
            .segment,
        segment
    );
    let lower = Fixture::tiers(1);
    let ctx = Context {
        tiers: &lower,
        ..f.context()
    };
    let mut one = signed.clone();
    one.signatures.truncate(1);
    assert_eq!(
        repaired.install_archive(&ctx, &one, &segment, &[0]).await,
        Err(InstallError::Configuration)
    );
    assert!(repaired.replica().await.ids().next().is_none());
}

struct Hostile {
    requests: std::sync::Arc<AtomicU64>,
    peer: PeerId,
    address: Multiaddr,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Hostile {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn hostile(answer: Option<Vec<u8>>) -> Hostile {
    let mut swarm =
        libp2p::SwarmBuilder::with_existing_identity(identity::Keypair::generate_ed25519())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .unwrap()
            .with_behaviour(|_| Client {
                archive: behaviour(&NET),
            })
            .unwrap()
            .build();
    let peer = *swarm.local_peer_id();
    swarm
        .listen_on("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .unwrap();
    let address = loop {
        if let SwarmEvent::NewListenAddr { address, .. } = swarm.select_next_some().await {
            break address;
        }
    };
    let requests = std::sync::Arc::new(AtomicU64::new(0));
    let counter = requests.clone();
    let task = tokio::spawn(async move {
        let mut silent = Vec::new();
        loop {
            if let SwarmEvent::Behaviour(ClientEvent::Archive(request_response::Event::Message {
                message: request_response::Message::Request { channel, .. },
                ..
            })) = swarm.select_next_some().await
            {
                counter.fetch_add(1, Ordering::Relaxed);
                if let Some(bytes) = &answer {
                    let _ = swarm
                        .behaviour_mut()
                        .archive
                        .send_response(channel, bytes.clone());
                } else {
                    silent.push(channel);
                }
            }
        }
    });
    Hostile {
        requests,
        peer,
        address,
        task,
    }
}

// Hostile providers and offline peers never supply k; silent I/O respects deadline (AT-NET-27).
#[tokio::test]
async fn hostile_offline_deadline_and_input_bounds() {
    let f = Fixture::new();
    let (_, signed, shards) = f.archive();
    let bad = hostile(Some(reply([0; 32], 0, &shards[0]))).await;
    let holders = vec![Holder {
        index: 0,
        peer: bad.peer,
        address: bad.address.clone(),
    }];
    assert!(matches!(
        fetch(&f.context(), &signed, holders, Duration::from_secs(2)).await,
        Err(FetchError::Unavailable)
    ));
    let silent = hostile(None).await;
    let start = Instant::now();
    assert!(matches!(
        fetch(
            &f.context(),
            &signed,
            vec![Holder {
                index: 0,
                peer: silent.peer,
                address: silent.address.clone()
            }],
            Duration::from_millis(100)
        )
        .await,
        Err(FetchError::Timeout)
    ));
    assert!(start.elapsed() < Duration::from_secs(1));
    let offline = vec![Holder {
        index: 0,
        peer: PeerId::random(),
        address: "/ip4/127.0.0.1/tcp/1".parse().unwrap(),
    }];
    assert!(matches!(
        fetch(&f.context(), &signed, offline, Duration::from_secs(2)).await,
        Err(FetchError::Unavailable)
    ));
    let duplicate = vec![
        Holder {
            index: 0,
            peer: bad.peer,
            address: bad.address.clone(),
        },
        Holder {
            index: 0,
            peer: bad.peer,
            address: bad.address.clone(),
        },
    ];
    assert!(matches!(
        fetch(&f.context(), &signed, duplicate, Duration::from_secs(1)).await,
        Err(FetchError::Holders)
    ));
    assert!(matches!(
        fetch(&f.context(), &signed, vec![], Duration::ZERO).await,
        Err(FetchError::Timeout)
    ));
}

// A real tiered handle serves only durable assignments before and after restart (AT-NET-27).
#[tokio::test]
async fn handle_persistence_restart_transfer() {
    let f = Fixture::new();
    let (segment, signed, shards) = f.archive();
    let dir = Temp::new();
    let key = identity::Keypair::generate_ed25519();
    let node = crate::Handle::spawn(key.clone(), f.config(Some(dir.0.clone()))).unwrap();
    node.install_archive(&f.context(), &signed, &segment, &[0, 2, 4])
        .await
        .unwrap();
    let addr = node
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let holders = [0, 2, 4]
        .into_iter()
        .map(|index| Holder {
            index,
            peer: node.peer_id(),
            address: addr.clone(),
        })
        .collect();
    assert_eq!(
        fetch(&f.context(), &signed, holders, Duration::from_secs(3))
            .await
            .unwrap()
            .segment,
        segment
    );
    drop(node);
    tokio::task::yield_now().await;
    let node = crate::Handle::spawn(key, f.config(Some(dir.0.clone()))).unwrap();
    let addr = node
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let holders = [0, 2, 4]
        .into_iter()
        .map(|index| Holder {
            index,
            peer: node.peer_id(),
            address: addr.clone(),
        })
        .collect();
    assert_eq!(
        fetch(&f.context(), &signed, holders, Duration::from_secs(3))
            .await
            .unwrap()
            .segment,
        segment
    );
    let path = dir
        .0
        .join("archive")
        .join(format!("{}.arc", hex(&signed.manifest.digest())));
    let mut bytes = fs::read(&path).unwrap();
    *bytes.last_mut().unwrap() ^= 1;
    fs::write(&path, bytes).unwrap();
    let holders = [0, 2, 4]
        .into_iter()
        .map(|index| Holder {
            index,
            peer: node.peer_id(),
            address: addr.clone(),
        })
        .collect();
    assert!(matches!(
        fetch(&f.context(), &signed, holders, Duration::from_secs(3)).await,
        Err(FetchError::Unavailable)
    ));
    node.install_archive(&f.context(), &signed, &segment, &[0, 2, 4])
        .await
        .unwrap();
    let holders = [0, 2, 4]
        .into_iter()
        .map(|index| Holder {
            index,
            peer: node.peer_id(),
            address: addr.clone(),
        })
        .collect();
    let recovered = fetch(&f.context(), &signed, holders, Duration::from_secs(3))
        .await
        .unwrap();
    assert_eq!(
        recovered.repairs,
        vec![(1, shards[1].clone()), (3, shards[3].clone())]
    );
}

// With exactly k possible replies, a hostile shard cannot complete recovery (AT-NET-27).
#[tokio::test]
async fn hostile_reply_cannot_complete_quorum() {
    let f = Fixture::new();
    let (segment, signed, shards) = f.archive();
    let honest =
        crate::Handle::spawn(identity::Keypair::generate_ed25519(), f.config(None)).unwrap();
    honest
        .install_archive(&f.context(), &signed, &segment, &[1, 2])
        .await
        .unwrap();
    let addr = honest
        .listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    let correct = reply(signed.manifest.digest(), 0, &shards[0]);
    let mut answers = vec![(true, correct.clone())];
    for offset in [2, 34, 42] {
        let mut bad = correct.clone();
        bad[offset] ^= 1;
        answers.push((false, bad));
    }
    answers.push((false, correct[..correct.len() - 1].to_vec()));
    for (valid, answer) in answers {
        let provider = hostile(Some(answer)).await;
        let holders = vec![
            Holder {
                index: 0,
                peer: provider.peer,
                address: provider.address.clone(),
            },
            Holder {
                index: 1,
                peer: honest.peer_id(),
                address: addr.clone(),
            },
            Holder {
                index: 2,
                peer: honest.peer_id(),
                address: addr.clone(),
            },
        ];
        let result = fetch(&f.context(), &signed, holders, Duration::from_secs(3)).await;
        assert_eq!(result.is_ok(), valid);
        if !valid {
            assert!(matches!(result, Err(FetchError::Unavailable)));
        }
        assert_eq!(provider.requests.load(Ordering::Relaxed), 1);
    }
    let provider = hostile(Some(correct)).await;
    let mut invalid = signed.clone();
    invalid.signatures.clear();
    let holders = vec![Holder {
        index: 0,
        peer: provider.peer,
        address: provider.address.clone(),
    }];
    assert!(matches!(
        fetch(&f.context(), &invalid, holders, Duration::from_secs(1)).await,
        Err(FetchError::Archive(_))
    ));
    assert_eq!(provider.requests.load(Ordering::Relaxed), 0);
    let mut address = Multiaddr::empty();
    for _ in 0..17 {
        address.push(libp2p::multiaddr::Protocol::Tcp(1));
    }
    assert!(matches!(
        fetch(
            &f.context(),
            &signed,
            vec![Holder {
                index: 0,
                peer: provider.peer,
                address
            }],
            Duration::from_secs(1)
        )
        .await,
        Err(FetchError::Holders)
    ));
    let address = Multiaddr::from(libp2p::multiaddr::Protocol::Dns(std::borrow::Cow::Owned(
        "x".repeat(1024),
    )));
    assert!(matches!(
        fetch(
            &f.context(),
            &signed,
            vec![Holder {
                index: 0,
                peer: provider.peer,
                address
            }],
            Duration::from_secs(1)
        )
        .await,
        Err(FetchError::Holders)
    ));
}
