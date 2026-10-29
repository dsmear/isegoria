use super::*;

pub(super) fn keys() -> Vec<identity::Keypair> {
    (40..45)
        .map(|seed| identity::Keypair::ed25519_from_bytes([seed; 32]).unwrap())
        .collect()
}
pub(super) fn transport_roster(keys: &[identity::Keypair]) -> Vec<[u8; 32]> {
    keys.iter()
        .map(|k| crate::tiered::transport_key(&k.public().to_peer_id()).unwrap())
        .collect()
}
pub(super) fn assigned(
    f: &Fixture,
    signed: &SignedManifest,
    roster: &[[u8; 32]],
    local: [u8; 32],
) -> usize {
    signed
        .verify(&f.context())
        .unwrap()
        .placement(&f.context(), roster)
        .unwrap()
        .iter()
        .position(|key| *key == local)
        .unwrap()
}
pub(super) fn path(dir: &Temp, signed: &SignedManifest) -> PathBuf {
    dir.0
        .join("archive")
        .join(format!("{}.arc", hex(&signed.manifest.digest())))
}
async fn connect(node: &crate::Handle, bootstrap: &DiscoveryFixture) {
    node.listen("/ip4/127.0.0.1/tcp/0".parse().unwrap())
        .await
        .unwrap();
    node.dial(bootstrap.address.clone()).await.unwrap();
}
pub(super) async fn source(
    f: &Fixture,
    signed: &SignedManifest,
    segment: &Segment,
    bootstrap: &DiscoveryFixture,
) -> crate::Handle {
    let node = crate::Handle::spawn(identity::Keypair::generate_ed25519(), f.config(None)).unwrap();
    node.install_archive(&f.context(), signed, segment, &[0, 1, 2, 3, 4])
        .await
        .unwrap();
    connect(&node, bootstrap).await;
    wait_advertisements(
        bootstrap,
        &(0..5)
            .map(|i| (provider_key(&signed.manifest.digest(), i), node.peer_id()))
            .collect::<Vec<_>>(),
    )
    .await;
    node
}

// Roster permutation selects distinct authenticated transport keys deterministically (AT-NET-29).
#[test]
fn roster_uses_transport_keys() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let keys = keys();
    let mut roster = transport_roster(&keys);
    let first = signed
        .verify(&f.context())
        .unwrap()
        .placement(&f.context(), &roster)
        .unwrap();
    roster.reverse();
    assert_eq!(
        first,
        signed
            .verify(&f.context())
            .unwrap()
            .placement(&f.context(), &roster)
            .unwrap()
    );
    assert_eq!(first.iter().collect::<BTreeSet<_>>().len(), 5);
    for key in keys {
        assert!(first.contains(&crate::tiered::transport_key(&key.public().to_peer_id()).unwrap()));
    }
}

// Assigned holders adopt shards, repair loss/bitrot, and outlive the source (AT-NET-29).
#[tokio::test]
async fn adoption_restart_loss_and_bitrot() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let bootstrap = discovery_fixture(151, NET, None, vec![], None).await;
    let source = source(&f, &signed, &segment, &bootstrap).await;
    let keys = keys();
    let roster = transport_roster(&keys);
    let dirs: Vec<_> = (0..5).map(|_| Temp::new()).collect();
    let mut nodes = Vec::new();
    let mut expected = Vec::new();
    for (i, key) in keys.iter().enumerate() {
        let mut node =
            crate::Handle::spawn(key.clone(), f.config(Some(dirs[i].0.clone()))).unwrap();
        connect(&node, &bootstrap).await;
        let index = assigned(&f, &signed, &roster, roster[i]);
        assert_eq!(
            node.maintain_archive(
                &f.context(),
                &signed,
                &roster,
                vec![bootstrap.address.clone()],
                Duration::from_secs(10)
            )
            .await,
            Ok(Maintenance::Repaired { index })
        );
        assert_eq!(
            read_record(&path(&dirs[i], &signed), &f.tiers, signed.manifest.digest())
                .unwrap()
                .shards
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            vec![index]
        );
        assert!(node.replica().await.is_empty());
        expected.push((
            provider_key(&signed.manifest.digest(), index),
            node.peer_id(),
        ));
        nodes.push(Some(node));
    }
    wait_advertisements(&bootstrap, &expected).await;
    drop(source);
    for (i, corrupt) in [(0, false), (1, true)] {
        if corrupt {
            nodes[i]
                .as_ref()
                .unwrap()
                .install_archive(&f.context(), &signed, &segment, &[0, 1, 2, 3, 4])
                .await
                .unwrap();
        }
        drop(nodes[i].take());
        tokio::task::yield_now().await;
        if corrupt {
            let mut bytes = fs::read(path(&dirs[i], &signed)).unwrap();
            *bytes.last_mut().unwrap() ^= 1;
            fs::write(path(&dirs[i], &signed), bytes).unwrap();
        } else {
            fs::remove_file(path(&dirs[i], &signed)).unwrap();
        }
        let mut node =
            crate::Handle::spawn(keys[i].clone(), f.config(Some(dirs[i].0.clone()))).unwrap();
        connect(&node, &bootstrap).await;
        let index = assigned(&f, &signed, &roster, roster[i]);
        assert_eq!(
            node.maintain_archive(
                &f.context(),
                &signed,
                &roster,
                vec![bootstrap.address.clone()],
                Duration::from_secs(10)
            )
            .await,
            Ok(Maintenance::Repaired { index })
        );
        assert_eq!(
            read_record(&path(&dirs[i], &signed), &f.tiers, signed.manifest.digest())
                .unwrap()
                .shards
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            vec![index]
        );
        nodes[i] = Some(node);
    }
    drop(nodes[3].take());
    drop(nodes[4].take());
    assert_eq!(
        fetch_discovered(
            &f.context(),
            &signed,
            vec![bootstrap.address.clone()],
            Duration::from_secs(10)
        )
        .await
        .unwrap()
        .segment,
        segment
    );
}

// Healthy and unassigned calls do no discovery, rewrite, or implicit eviction (AT-NET-29).
#[tokio::test]
async fn healthy_and_unassigned_are_read_only() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let keys = keys();
    let roster = transport_roster(&keys);
    let dir = Temp::new();
    let bootstrap = discovery_fixture(152, NET, None, vec![], None).await;
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let index = assigned(&f, &signed, &roster, roster[0]);
    node.install_archive(&f.context(), &signed, &segment, &[index])
        .await
        .unwrap();
    let bytes = fs::read(path(&dir, &signed)).unwrap();
    let modified = fs::metadata(path(&dir, &signed))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(2)
        )
        .await,
        Ok(Maintenance::Healthy { index })
    );
    let absent: Vec<_> = (70..75)
        .map(|s| identity::Keypair::ed25519_from_bytes([s; 32]).unwrap())
        .collect();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &transport_roster(&absent),
            vec![bootstrap.address.clone()],
            Duration::from_secs(2)
        )
        .await,
        Ok(Maintenance::Unassigned)
    );
    assert_eq!(fs::read(path(&dir, &signed)).unwrap(), bytes);
    assert_eq!(
        fs::metadata(path(&dir, &signed))
            .unwrap()
            .modified()
            .unwrap(),
        modified
    );
    let lower = Fixture::tiers(1);
    let mut context = f.context();
    context.tiers = &lower;
    for candidate_roster in [&roster, &transport_roster(&absent)] {
        assert_eq!(
            node.maintain_archive(
                &context,
                &signed,
                candidate_roster,
                vec![],
                Duration::from_secs(2)
            )
            .await,
            Err(MaintainError::Install(InstallError::Configuration))
        );
    }
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![],
            Duration::from_secs(2)
        )
        .await,
        Ok(Maintenance::Healthy { index })
    );
    assert_eq!(bootstrap.contacts.load(Ordering::Relaxed), 0);
}

// Caller roster limits and key encodings are checked before files or contacts (AT-NET-29).
#[tokio::test]
async fn invalid_rosters_are_rejected() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let keys = keys();
    let good = transport_roster(&keys);
    let dir = Temp::new();
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let bootstrap = discovery_fixture(153, NET, None, vec![], None).await;
    let invalid = (0u64..)
        .map(|i| {
            let mut key = [0; 32];
            key[..8].copy_from_slice(&i.to_le_bytes());
            key
        })
        .find(|key| ed25519_dalek::VerifyingKey::from_bytes(key).is_err())
        .unwrap();
    let mut bad_encoding = good.clone();
    bad_encoding[0] = invalid;
    for bad in [
        vec![],
        good[..4].to_vec(),
        vec![good[0]; 5],
        vec![good[0]; MAX_ROSTER + 1],
        bad_encoding,
    ] {
        assert_eq!(
            node.maintain_archive(
                &f.context(),
                &signed,
                &bad,
                vec![bootstrap.address.clone()],
                Duration::from_secs(1)
            )
            .await,
            Err(MaintainError::Roster)
        );
    }
    assert!(!path(&dir, &signed).exists());
    assert_eq!(bootstrap.contacts.load(Ordering::Relaxed), 0);
}

// Manifest/context, node quorum, and non-Ed25519 identities fail before discovery (AT-NET-29).
#[tokio::test]
async fn trust_configuration_and_identity_preflight() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let keys = keys();
    let roster = transport_roster(&keys);
    let dir = Temp::new();
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let bootstrap = discovery_fixture(154, NET, None, vec![], None).await;
    let mut invalid = signed.clone();
    invalid.signatures.clear();
    assert!(matches!(
        node.maintain_archive(
            &f.context(),
            &invalid,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(1)
        )
        .await,
        Err(MaintainError::Archive(_))
    ));
    let mut context = f.context();
    context.lineage = [0; 32];
    assert!(matches!(
        node.maintain_archive(
            &context,
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(1)
        )
        .await,
        Err(MaintainError::Archive(_))
    ));
    let lower = Fixture::tiers(1);
    let mut context = f.context();
    context.tiers = &lower;
    assert_eq!(
        node.maintain_archive(
            &context,
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(1)
        )
        .await,
        Err(MaintainError::Install(InstallError::Configuration))
    );
    node.peer_id = PeerId::random();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(1)
        )
        .await,
        Err(MaintainError::Identity)
    );
    assert!(!path(&dir, &signed).exists());
    assert_eq!(bootstrap.contacts.load(Ordering::Relaxed), 0);
}

// A missing assignment repair preserves every previously authenticated selected index (AT-NET-29).
#[tokio::test]
async fn repair_preserves_extra_indices() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let keys = keys();
    let roster = transport_roster(&keys);
    let index = assigned(&f, &signed, &roster, roster[0]);
    let old = (index + 1) % 5;
    let dir = Temp::new();
    let bootstrap = discovery_fixture(155, NET, None, vec![], None).await;
    let _source = source(&f, &signed, &segment, &bootstrap).await;
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    node.install_archive(&f.context(), &signed, &segment, &[old])
        .await
        .unwrap();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_secs(10)
        )
        .await,
        Ok(Maintenance::Repaired { index })
    );
    let record = read_record(&path(&dir, &signed), &f.tiers, signed.manifest.digest()).unwrap();
    assert_eq!(record.shards.len(), 2);
    assert!(record.shards.contains_key(&old) && record.shards.contains_key(&index));
}

// Audit revalidates live disk data rather than trusting startup advertisements (AT-NET-29).
#[tokio::test]
async fn live_bitrot_is_repaired() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let keys = keys();
    let roster = transport_roster(&keys);
    let index = assigned(&f, &signed, &roster, roster[0]);
    let dir = Temp::new();
    let bootstrap = discovery_fixture(156, NET, None, vec![], None).await;
    let _source = source(&f, &signed, &segment, &bootstrap).await;
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    node.install_archive(&f.context(), &signed, &segment, &[0, 1, 2, 3, 4])
        .await
        .unwrap();
    for corrupt in [true, false] {
        if corrupt {
            let mut bytes = fs::read(path(&dir, &signed)).unwrap();
            *bytes.last_mut().unwrap() ^= 1;
            fs::write(path(&dir, &signed), bytes).unwrap();
        } else {
            fs::remove_file(path(&dir, &signed)).unwrap();
        }
        assert_eq!(
            node.maintain_archive(
                &f.context(),
                &signed,
                &roster,
                vec![bootstrap.address.clone()],
                Duration::from_secs(10)
            )
            .await,
            Ok(Maintenance::Repaired { index })
        );
        assert_eq!(
            read_record(&path(&dir, &signed), &f.tiers, signed.manifest.digest())
                .unwrap()
                .shards
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4]
        );
    }
}

// Fewer than k authentic shards and an expired deadline preserve the committed record (AT-NET-29).
#[tokio::test]
async fn unavailable_and_deadline_preserve_data() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let keys = keys();
    let roster = transport_roster(&keys);
    let index = assigned(&f, &signed, &roster, roster[0]);
    let dir = Temp::new();
    let bootstrap = discovery_fixture(157, NET, None, vec![], None).await;
    let mut node = crate::Handle::spawn(keys[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    node.install_archive(&f.context(), &signed, &segment, &[(index + 1) % 5])
        .await
        .unwrap();
    let bytes = fs::read(path(&dir, &signed)).unwrap();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::ZERO
        )
        .await,
        Err(MaintainError::Timeout)
    );
    assert_eq!(bootstrap.contacts.load(Ordering::Relaxed), 0);
    let holder =
        crate::Handle::spawn(identity::Keypair::generate_ed25519(), f.config(None)).unwrap();
    holder
        .install_archive(&f.context(), &signed, &segment, &[0, 1])
        .await
        .unwrap();
    connect(&holder, &bootstrap).await;
    wait_advertisements(
        &bootstrap,
        &(0..2)
            .map(|i| (provider_key(&signed.manifest.digest(), i), holder.peer_id()))
            .collect::<Vec<_>>(),
    )
    .await;
    assert!(matches!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![bootstrap.address.clone()],
            Duration::from_millis(300)
        )
        .await,
        Err(MaintainError::Timeout | MaintainError::Fetch(FetchError::Unavailable))
    ));
    assert_eq!(fs::read(path(&dir, &signed)).unwrap(), bytes);
    assert!(bootstrap
        .advertised
        .lock()
        .unwrap()
        .values()
        .all(|peers| !peers.contains(&node.peer_id())));
}

// Commit acceptance rejects queued expiration without persisting or advertising (AT-NET-29).
#[tokio::test]
async fn expired_commit_is_rejected() {
    let f = Fixture::new();
    let (segment, signed, _) = f.archive();
    let dir = Temp::new();
    let node = crate::Handle::spawn(keys()[0].clone(), f.config(Some(dir.0.clone()))).unwrap();
    let prepared = Prepared::new(&f.context(), &signed, &segment, &[0]).unwrap();
    let deadline = Deadline {
        started: Instant::now(),
        budget: Duration::ZERO,
    };
    assert_eq!(
        node.ask(|tx| crate::Command::MaintainArchive(prepared, deadline, tx))
            .await,
        Err(MaintainError::Timeout)
    );
    assert!(!path(&dir, &signed).exists());
}

// A maximum-size valid transport roster remains usable for a nonassigned node (AT-NET-29).
#[tokio::test]
async fn maximum_valid_roster() {
    let f = Fixture::new();
    let (_, signed, _) = f.archive();
    let roster: Vec<_> = (0u32..MAX_ROSTER as u32)
        .map(|i| {
            let mut seed = [0; 32];
            seed[..4].copy_from_slice(&i.to_le_bytes());
            crate::tiered::transport_key(
                &identity::Keypair::ed25519_from_bytes(seed)
                    .unwrap()
                    .public()
                    .to_peer_id(),
            )
            .unwrap()
        })
        .collect();
    let placement = signed
        .verify(&f.context())
        .unwrap()
        .placement(&f.context(), &roster)
        .unwrap();
    let local = roster
        .iter()
        .position(|key| !placement.contains(key))
        .unwrap() as u32;
    let mut seed = [0; 32];
    seed[..4].copy_from_slice(&local.to_le_bytes());
    let mut node = crate::Handle::spawn(
        identity::Keypair::ed25519_from_bytes(seed).unwrap(),
        f.config(None),
    )
    .unwrap();
    assert_eq!(
        node.maintain_archive(
            &f.context(),
            &signed,
            &roster,
            vec![],
            Duration::from_secs(5)
        )
        .await,
        Ok(Maintenance::Unassigned)
    );
}
