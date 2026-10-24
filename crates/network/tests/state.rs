use network::state::*;
use sha2::{Digest, Sha256};

fn h(domain: &str, fields: &[&[u8]]) -> Hash {
    let mut h = Sha256::new();
    h.update((domain.len() as u64).to_le_bytes());
    h.update(domain.as_bytes());
    for field in fields {
        h.update((field.len() as u64).to_le_bytes());
        h.update(field);
    }
    h.finalize().into()
}

fn context() -> Context {
    Context {
        lineage: [1; 32],
        network_id: [2; 32],
        epoch: 7,
    }
}
fn person(byte: u8) -> CallerAuthenticatedNym {
    CallerAuthenticatedNym::from_verified_caller([byte; 32], Role::Propose)
}
fn records(payload: &[u8]) -> Records {
    Records::new(Role::Propose, payload.to_vec())
}
fn populated() -> State {
    let mut state = State::new(context());
    state
        .insert(&person(4), records(b"negative record"), [5; 32])
        .unwrap();
    state
        .insert(&person(6), records(b"another record"), [7; 32])
        .unwrap();
    state
}
fn statement(state: &State) -> RootStatement {
    state.statement([8; 32], 9, [10; 32], [11; 32])
}

// An opening binds every secret byte, path and context to the root; AT-T86-1..3.
#[test]
fn openings_bind_bytes_and_context_and_separate_forks() {
    let state = populated();
    let opening = state.opening(&person(4)).unwrap();
    let root = state.root();
    assert!(opening.verify(&context(), Role::Propose, &root));
    for i in 0..opening.encode().len() {
        let mut bytes = opening.encode();
        bytes[i] ^= 1;
        if let Ok(other) = Opening::decode(&bytes) {
            assert!(!other.verify(&context(), Role::Propose, &root));
        }
    }
    let mut wrong = root;
    wrong[0] ^= 1;
    assert!(!opening.verify(&context(), Role::Propose, &wrong));
    for field in 0..3 {
        let mut ctx = context();
        match field {
            0 => ctx.lineage[0] ^= 1,
            1 => ctx.network_id[0] ^= 1,
            _ => ctx.epoch += 1,
        }
        assert!(!opening.verify(&ctx, Role::Propose, &root));
    }
    assert!(!opening.verify(&context(), Role::Judge, &root));
    let mut fork_context = context();
    fork_context.network_id = [99; 32];
    assert_eq!(
        position(&context().lineage, Role::Propose, &[4; 32]),
        position(&fork_context.lineage, Role::Propose, &[4; 32])
    );
    assert_ne!(salt(&[5; 32], &context()), salt(&[5; 32], &fork_context));
    let fork = state
        .fork_with_member(&statement(&state), fork_context.network_id)
        .unwrap();
    assert_ne!(fork.root(), root);
    assert!(!opening.verify(&fork_context, Role::Propose, &fork.root()));
}

// Public paths cannot substitute another caller or erase an occupied nym; AT-T86-4.
#[test]
fn absence_and_authenticated_boundary() {
    let state = populated();
    let absent = state.path(position(&context().lineage, Role::Propose, &[3; 32]));
    assert!(absent.verify_absence(&state.root()));
    assert!(verify_new(&context(), &state.root(), Some(&person(3)), &absent).is_ok());
    assert!(verify_new(&context(), &state.root(), Some(&person(4)), &absent).is_err());
    assert!(verify_new(&context(), &state.root(), None, &absent).is_err());
    let opening = state.opening(&person(4)).unwrap();
    assert!(verify_reentry(&context(), &state.root(), None, &opening).is_err());
    assert!(verify_reentry(&context(), &state.root(), Some(&person(6)), &opening).is_err());
    assert!(verify_reentry(&context(), &state.root(), Some(&person(4)), &opening).is_ok());
    assert!(!opening.path.verify_absence(&state.root()));
}

// Live leaves re-salt; frozen leaves reopen once and retain later records; AT-T86-5..7.
#[test]
fn frozen_reopening_is_consumed_before_next_root_and_origins_survive_forks() {
    let mut parent = populated();
    let old = parent.opening(&person(4)).unwrap();
    parent.advance_epoch(8).unwrap();
    assert_ne!(parent.root(), populated().root());
    let current = parent.opening(&person(4)).unwrap();
    let trusted = statement(&parent);
    let mut fork = State::fork_without_member(&trusted, &parent.shape(), [12; 32]).unwrap();
    let frozen_root = fork.root();
    fork.advance_epoch(9).unwrap();
    assert_eq!(frozen_root, fork.root());
    assert!(fork.reopen(Some(&person(4)), &old, [5; 32]).is_err());
    assert!(fork.reopen(Some(&person(4)), &current, [99; 32]).is_err());
    fork.reopen(Some(&person(4)), &current, [5; 32]).unwrap();
    fork.set_records(&person(4), records(b"newer negative record"))
        .unwrap();
    assert!(fork.reopen(Some(&person(4)), &current, [5; 32]).is_err());
    let mut second = fork.fork_with_member(&statement(&fork), [13; 32]).unwrap();
    second.advance_epoch(10).unwrap();
    assert!(second.opening(&person(4)).unwrap().records.payload() == b"newer negative record");
    let mut first_origin_opening = parent.opening(&person(6)).unwrap();
    first_origin_opening.path = second.path(first_origin_opening.path.position);
    second
        .reopen(Some(&person(6)), &first_origin_opening, [7; 32])
        .unwrap();
    second.advance_epoch(11).unwrap();
    assert!(second.opening(&person(6)).is_some());
    let second_statement = statement(&second);
    let mut third =
        State::fork_without_member(&second_statement, &second.shape(), [14; 32]).unwrap();
    let second_origin = second.opening(&person(4)).unwrap();
    third
        .reopen(Some(&person(4)), &second_origin, [5; 32])
        .unwrap();
    assert!(third.reopen(Some(&person(4)), &current, [5; 32]).is_err());
}

// Reconstruction authenticates metadata through shape digest as well as root; AT-T86-7,9.
#[test]
fn shape_and_statement_have_one_encoding_and_both_commitments_are_checked() {
    let state = populated();
    let shape = state.shape();
    let trusted = statement(&state);
    assert_eq!(Shape::decode(&shape.encode()).unwrap(), shape);
    assert_eq!(RootStatement::decode(&trusted.encode()).unwrap(), trusted);
    assert_eq!(shape.root(), state.root());
    let mut bytes = shape.encode();
    bytes.push(0);
    assert!(Shape::decode(&bytes).is_err());
    let entries = shape.entries();
    assert!(Shape::new(vec![entries[0], entries[0]]).is_err());
    assert!(Shape::new(vec![entries[1], entries[0]]).is_err());
    let mut changed = entries.to_vec();
    changed[0].origin = Origin::Frozen {
        network_id: [55; 32],
        epoch: 0,
    };
    let altered = Shape::new(changed).unwrap();
    assert_eq!(shape.root(), altered.root());
    assert_ne!(shape.digest(), altered.digest());
    assert!(State::fork_without_member(&trusted, &altered, [12; 32]).is_err());
    for i in 1..trusted.encode().len() {
        let mut encoded = trusted.encode();
        encoded[i] ^= 1;
        let changed = RootStatement::decode(&encoded).unwrap();
        assert_ne!(changed.digest(), trusted.digest());
    }
}

// Two extreme positions pin MSB order and bottom-up bitmap siblings; AT-T86-8.
#[test]
fn hand_computed_tree_and_bitmap() {
    let mut empties = vec![h("isegoria/state/empty/v1", &[])];
    for height in 0..256 {
        empties.push(h(
            "isegoria/state/node/v1",
            &[&empties[height], &empties[height]],
        ));
    }
    assert_eq!(empty_hash(0), Some(empties[0]));
    assert_eq!(empty_hash(256), Some(empties[256]));
    let a = [0; 32];
    let mut b = [0; 32];
    b[0] = 128;
    let mut c = [0; 32];
    c[31] = 1;
    let entries = vec![
        ShapeEntry {
            position: a,
            leaf_hash: [20; 32],
            origin: Origin::Live,
        },
        ShapeEntry {
            position: c,
            leaf_hash: [21; 32],
            origin: Origin::Live,
        },
        ShapeEntry {
            position: b,
            leaf_hash: [22; 32],
            origin: Origin::Live,
        },
    ];
    let shape = Shape::new(entries).unwrap();
    let mut left = h("isegoria/state/node/v1", &[&[20; 32], &[21; 32]]);
    let mut right = [22; 32];
    for (height, empty) in empties.iter().enumerate().take(255) {
        if height > 0 {
            left = h("isegoria/state/node/v1", &[&left, empty]);
        }
        right = h("isegoria/state/node/v1", &[&right, empty]);
    }
    let expected = h("isegoria/state/node/v1", &[&left, &right]);
    assert_eq!(shape.root(), expected);
    let path = shape.path(a);
    assert_eq!(path.bitmap()[0], 128);
    assert_eq!(path.bitmap()[31], 1);
    assert_eq!(path.siblings(), &[[21; 32], right]);
    assert!(path.verify(&[20; 32], &expected));
    let mut encoded = path.encode();
    encoded[65..97].copy_from_slice(&empties[0]);
    assert!(Path::decode(&encoded).is_err());
    let mut encoded = path.encode();
    encoded.pop();
    assert!(Path::decode(&encoded).is_err());
    let mut encoded = path.encode();
    encoded.extend_from_slice(&[0; 32]);
    assert!(Path::decode(&encoded).is_err());
    let mut encoded = path.encode();
    encoded[65..97].copy_from_slice(&right);
    encoded[97..129].copy_from_slice(&[21; 32]);
    assert!(!Path::decode(&encoded).unwrap().verify(&[20; 32], &expected));
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// Independent SHA-256 vectors pin field boundaries, epoch endianness and tags; AT-T86-3,8.
#[test]
fn state_known_answers() {
    let pos = position(&context().lineage, Role::Propose, &[4; 32]);
    let salt = salt(&[5; 32], &context());
    let commit = commitment(&context(), &records(b"negative record"), &salt);
    assert_eq!(
        hex(&empty_hash(0).unwrap()),
        "ca08c3391f88b19b529ec83d1e5481c65ce620b2e614321dc6c9f2a1b85e18da"
    );
    assert_eq!(
        hex(&empty_hash(256).unwrap()),
        "964913e8c4db001d4e22a9bd7611069e0f4815151c305075f2adc6c2ecae7c2d"
    );
    assert_eq!(
        hex(&pos),
        "4079e40ed02116ddd47f182586bb5dd0b64a612c3c38824578ebfa2b94ac403f"
    );
    assert_eq!(
        hex(&salt),
        "52e0338429e634a2c2e0adfbd813c0f6d12ee57842cce47da519ebdc529c80c1"
    );
    assert_eq!(
        hex(&commit),
        "97e11912933c9d2f12f09a28758e5a9aab0af525b905a7d3499c1c2b6bfad5d7"
    );
    assert_eq!(
        hex(&leaf_hash(&pos, &commit)),
        "54de9c6f89ac3b94a787aeb46be2d3e578b411800045d266b8d4b874898c8300"
    );
    assert_eq!(empty_hash(257), None);
}

// Shape changes and missing member data cannot pass an anchored reconstruction; AT-T86-7.
#[test]
fn reconstruction_requires_complete_authentic_shape_and_private_state() {
    let state = populated();
    let shape = state.shape();
    let trusted = statement(&state);
    let mut entries = shape.entries().to_vec();
    entries.pop();
    assert!(Shape::new(entries).unwrap().check(&trusted).is_err());
    let mut entries = shape.entries().to_vec();
    entries[0].leaf_hash[0] ^= 1;
    let changed = Shape::new(entries).unwrap();
    assert_ne!(changed.root(), shape.root());
    assert!(changed.check(&trusted).is_err());
    let mut entries = shape.entries().to_vec();
    entries.push(ShapeEntry {
        position: [255; 32],
        leaf_hash: [200; 32],
        origin: Origin::Live,
    });
    assert!(Shape::new(entries).unwrap().check(&trusted).is_err());
    assert!(Shape::new(vec![ShapeEntry {
        position: [0; 32],
        leaf_hash: empty_hash(0).unwrap(),
        origin: Origin::Live
    }])
    .is_err());
    let mut recovered = State::fork_without_member(&trusted, &shape, [12; 32]).unwrap();
    assert!(recovered
        .insert(&person(4), records(b"default"), [5; 32])
        .is_err());
    assert!(recovered
        .set_records(&person(4), records(b"default"))
        .is_err());
    let before = statement(&recovered);
    let opening = state.opening(&person(4)).unwrap();
    recovered
        .reopen(Some(&person(4)), &opening, [5; 32])
        .unwrap();
    assert_eq!(
        recovered
            .shape()
            .entries()
            .iter()
            .filter(|e| e.origin == Origin::Live)
            .count(),
        1
    );
    assert!(recovered.fork_with_member(&before, [13; 32]).is_err());
    recovered
        .set_records(&person(4), records(b"later"))
        .unwrap();
    assert!(recovered
        .fork_with_member(&statement(&recovered), [13; 32])
        .is_ok());
}

// Versions, lengths, role tags, duplicate positions and trailing data are refused; AT-T86-9.
#[test]
fn canonical_codecs_reject_truncation_and_extensions() {
    let state = populated();
    type Decoder = fn(&[u8]) -> bool;
    let values: Vec<(Vec<u8>, Decoder)> = vec![
        (state.shape().encode(), |b| Shape::decode(b).is_ok()),
        (state.path([0; 32]).encode(), |b| Path::decode(b).is_ok()),
        (statement(&state).encode(), |b| {
            RootStatement::decode(b).is_ok()
        }),
        (state.opening(&person(4)).unwrap().encode(), |b| {
            Opening::decode(b).is_ok()
        }),
        (records(b"data").encode(), |b| Records::decode(b).is_ok()),
    ];
    for (bytes, decode) in values {
        for end in 0..bytes.len() {
            assert!(!decode(&bytes[..end]));
        }
        let mut changed = bytes.clone();
        changed.push(0);
        assert!(!decode(&changed));
        let mut changed = bytes;
        changed[0] = 2;
        assert!(!decode(&changed));
    }
    assert!(Shape::decode(&[1, 255, 255, 255, 255, 255, 255, 255, 255]).is_err());
    let mut invalid_role = records(b"data").encode();
    invalid_role[1] = 3;
    assert!(Records::decode(&invalid_role).is_err());
    let mut malformed_shape = state.shape().encode();
    malformed_shape[73] = 2;
    assert!(Shape::decode(&malformed_shape).is_err());
}

use proptest::prelude::*;
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // Arbitrary opaque records retain membership and deny occupied absence; AT-T86-1,4,5.
    #[test]
    fn arbitrary_records_and_epochs(payload in prop::collection::vec(any::<u8>(), 0..512), key in any::<Hash>(), epoch in 0u64..u64::MAX) {
        let mut ctx = context(); ctx.epoch = epoch;
        let mut state = State::new(ctx);
        state.insert(&person(4), records(&payload), key).unwrap();
        let opening = state.opening(&person(4)).unwrap();
        prop_assert!(opening.verify(&ctx, Role::Propose, &state.root()));
        prop_assert!(!opening.path.verify_absence(&state.root()));
        let previous = state.root();
        state.advance_epoch(epoch + 1).unwrap();
        prop_assert_ne!(state.root(), previous);
        let new_opening = state.opening(&person(4)).unwrap();
        prop_assert!(new_opening.verify(&state.context(), Role::Propose, &state.root()));
    }

    // Hostile bytes never panic and accepted inputs retain their exact encoding; AT-T86-9.
    #[test]
    fn hostile_codecs_are_canonical(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
        if let Ok(value) = Shape::decode(&bytes) { prop_assert_eq!(&value.encode(), &bytes); }
        if let Ok(value) = Path::decode(&bytes) { prop_assert_eq!(&value.encode(), &bytes); }
        if let Ok(value) = Records::decode(&bytes) { prop_assert_eq!(&value.encode(), &bytes); }
        if let Ok(value) = Opening::decode(&bytes) { prop_assert_eq!(&value.encode(), &bytes); }
        if let Ok(value) = RootStatement::decode(&bytes) { prop_assert_eq!(&value.encode(), &bytes); }
    }
}
