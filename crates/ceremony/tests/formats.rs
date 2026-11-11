//! The ceremony's formats (`docs/03` T19-P3; AT-ID-15, 16, 24): the procedure and the id, a
//! message's encoding, signature and seal, and the keys `--new-identity` makes.

mod support;

use ceremony::message::{self, Header, MessageError, HEADER, SEAL_INFO};
use ceremony::procedure::{Keys, Procedure, ProcedureError};
use ceremony::{hex, Error};
use ed25519_dalek::SigningKey;
use hpke::Deserializable;
use std::collections::HashSet;
use support::*;

type Kem = hpke::kem::X25519HkdfSha256;

fn fixed() -> Procedure {
    Procedure {
        threshold: 3,
        members: (0..4)
            .map(|i| Keys {
                consortium: [1 + i; 32],
                hpke: [11 + i; 32],
            })
            .collect(),
        signers: vec![0, 2, 3],
        slots: vec![1, 2, 3],
    }
}

fn fresh() -> Vec<[u8; 32]> {
    (0..4).map(|i| [21 + i; 32]).collect()
}

fn member(seed: u8) -> (SigningKey, message::HpkeSecret, [u8; 32]) {
    let (secret, public) = message::hpke_pair(&[seed + 100; 32]);
    (SigningKey::from_bytes(&[seed; 32]), secret, public)
}

fn header() -> Header {
    Header {
        id: [5; 32],
        step: 2,
        sender: 1,
        recipient: 0,
    }
}

// The slots' digest, the opening and the id match an independent oracle's (AT-ID-15).
#[test]
fn at_id_15_the_id_matches_its_known_answer() {
    let p = fixed();
    assert_eq!(
        hex(&p.slots_digest()),
        "50e841d03d2202cdd4e32cb191aaae1f5444ac464ee8db32b2e2646d2c4d3b7a"
    );
    assert_eq!(
        hex(&p.opening()),
        "042359976b239e46c9862a941afd5172aeeb4c6124b4799c3ca235767ca2756a"
    );
    assert_eq!(
        hex(&p.id(&fresh())),
        "2a4440e9378f18e0c61822d26646684b0ed7fbb173c18385794bb11626a13f69"
    );
}

// The id changes with each key, their order, the threshold, the slots, each fresh value (AT-ID-15).
#[test]
fn at_id_15_the_id_binds_every_input() {
    let p = fixed();
    let base = p.id(&fresh());
    let mut ids = HashSet::from([base, p.opening()]);
    for i in 0..4 {
        let (mut a, mut b) = (p.clone(), p.clone());
        a.members[i].consortium[0] ^= 1;
        b.members[i].hpke[0] ^= 1;
        let mut f = fresh();
        f[i][31] ^= 1;
        for id in [a.id(&fresh()), b.id(&fresh()), p.id(&f)] {
            assert!(ids.insert(id));
        }
    }
    let mut q = p.clone();
    q.members.swap(0, 1);
    assert!(ids.insert(q.id(&fresh())));
    let mut q = p.clone();
    q.threshold = 2;
    assert!(ids.insert(q.id(&fresh())));
    for slots in [vec![1, 2, 4], vec![1, 2], vec![1, 2, 3, 4]] {
        let mut q = p.clone();
        q.slots = slots;
        assert!(ids.insert(q.id(&fresh())));
    }
    let mut q = p.clone();
    q.signers = vec![0, 1, 3];
    assert_eq!(q.id(&fresh()), base);
}

// A procedure whose key is invalid or repeated, or whose shape cannot run, is refused (AT-ID-15).
#[test]
fn at_id_15_the_procedure_refuses_what_does_not_hold() {
    let c = Committee::new("procedure", &[1, 2]);
    let good = procedure_text(&c.printed, &[1, 2]);
    let p = Procedure::parse(&good).unwrap();
    assert_eq!(p.members.len(), 4);
    let bytes = p.encode();
    assert_eq!(Procedure::decode(&bytes), Some(p.clone()));
    assert!((0..bytes.len()).all(|n| Procedure::decode(&bytes[..n]).is_none()));
    let mut longer = bytes;
    longer.push(0);
    assert!(Procedure::decode(&longer).is_none());
    let (c0, h1) = (hex(&c.printed[0].consortium), hex(&c.printed[1].hpke));
    let c2 = hex(&c.printed[2].consortium);
    let refused = |from: &str, to: &str| Procedure::parse(&good.replacen(from, to, 1));
    let key = |field: &str| Err(ProcedureError::Key(field.into()));
    assert_eq!(refused(&h1, &"00".repeat(32)), key("member[1].hpke"));
    assert_eq!(refused(&h1, &"ff".repeat(32)), key("member[1].hpke"));
    assert_eq!(refused(&c0, "zz"), key("member[0].consortium"));
    let repeated = Err(ProcedureError::Repeated("member[2].consortium".into()));
    assert_eq!(refused(&c2, &c0), repeated);
    for (from, to) in [
        ("threshold = 3", "threshold = 5"),
        ("signers = [0, 2, 3]", "signers = [0, 2]"),
        ("signers = [0, 2, 3]", "signers = [3, 2, 0]"),
        ("slots = [1, 2]", "slots = [2, 1]"),
        ("slots = [1, 2]", "slots = []"),
    ] {
        assert_eq!(refused(from, to), Err(ProcedureError::Setup), "{to}");
    }
    let circle = format!("consortium = \"{c0}\"\ncircle = \"{c0}\"");
    let with_circle = refused(&format!("consortium = \"{c0}\""), &circle);
    assert!(matches!(with_circle, Err(ProcedureError::Parse(_))));
}

// The signed digest matches an independent oracle's, under the ceremony's own tag (AT-ID-16).
#[test]
fn at_id_16_the_signed_digest_matches_its_known_answer() {
    assert_eq!(
        hex(&message::signed_digest(&header(), b"payload")),
        "9f2e25febdd5f7a48ca6617a12ccddd3209193050717f8a693b2d1527d75eef7"
    );
    assert_eq!(header().encode()[..2], [1, 5]);
    assert_eq!(header().encode()[33..], [2, 1, 0]);
    assert_eq!(message::SIGNED, "isegoria/ceremony/message/v1");
}

// The file is header, enc, then signature and payload sealed under the header as aad (AT-ID-16).
#[test]
fn at_id_16_the_seal_is_base_mode_over_the_header() {
    let ((sender, _, _), (_, _, recipient)) = (member(1), member(0));
    let file = message::seal(&header(), b"payload", &sender, &recipient).unwrap();
    assert_eq!(file[..HEADER], header().encode());
    let enc = &file[HEADER..HEADER + 32];
    let len = u64::from_le_bytes(file[HEADER + 32..HEADER + 40].try_into().unwrap());
    let sealed = &file[HEADER + 40..];
    assert_eq!(len as usize, sealed.len());
    let (secret, _) = <Kem as hpke::Kem>::derive_keypair(&[100; 32]);
    let encapped = <Kem as hpke::Kem>::EncappedKey::from_bytes(enc).unwrap();
    let open = |info: &[u8], aad: &[u8]| {
        hpke::single_shot_open::<hpke::aead::AesGcm128, hpke::kdf::HkdfSha256, Kem>(
            &hpke::OpModeR::Base,
            &secret,
            &encapped,
            info,
            sealed,
            aad,
        )
    };
    let plain = open(b"isegoria/ceremony/seal/v1", &header().encode()).unwrap();
    assert_eq!(SEAL_INFO, b"isegoria/ceremony/seal/v1");
    assert_eq!(&plain[64..], b"payload");
    let signature = ed25519_dalek::Signature::from_bytes(plain[..64].try_into().unwrap());
    let digest = message::signed_digest(&header(), b"payload");
    assert!(sender
        .verifying_key()
        .verify_strict(&digest, &signature)
        .is_ok());
    assert!(open(b"another", &header().encode()).is_err());
    assert!(open(SEAL_INFO, &[0; HEADER]).is_err());
}

// A message opens only for its recipient, from another member, at a step, untampered (AT-ID-16).
#[test]
fn at_id_16_a_message_opens_only_as_sent() {
    let keys: Vec<_> = (0..3).map(member).collect();
    let members: Vec<[u8; 32]> = keys
        .iter()
        .map(|k| k.0.verifying_key().to_bytes())
        .collect();
    let seal = |h: &Header, by: usize| message::seal(h, b"payload", &keys[by].0, &keys[0].2);
    let open = |file: &[u8]| message::open(file, 0, &keys[0].1, &members);
    let file = seal(&header(), 1).unwrap();
    assert_eq!(open(&file), Ok((header(), b"payload".to_vec())));
    assert_eq!(
        message::open(&file, 2, &keys[2].1, &members),
        Err(MessageError::Recipient)
    );
    assert_eq!(
        message::open(&file, 0, &keys[2].1, &members),
        Err(MessageError::Seal)
    );
    assert!((0..file.len()).all(|n| open(&file[..n]) == Err(MessageError::Malformed)));
    let mut longer = file.clone();
    longer.push(0);
    assert_eq!(open(&longer), Err(MessageError::Malformed));
    for at in 0..file.len() {
        let mut flipped = file.clone();
        flipped[at] ^= 1;
        assert!(open(&flipped).is_err(), "byte {at}");
    }
    let claimed = Header {
        sender: 2,
        ..header()
    };
    assert_eq!(
        open(&seal(&claimed, 1).unwrap()),
        Err(MessageError::Signature)
    );
    for sender in [0, 3, 9] {
        let h = Header { sender, ..header() };
        assert_eq!(open(&seal(&h, 1).unwrap()), Err(MessageError::Sender));
    }
    for step in [0, 14] {
        let h = Header { step, ..header() };
        assert_eq!(open(&seal(&h, 1).unwrap()), Err(MessageError::Step));
    }
    let mut version = file;
    version[0] = 2;
    assert_eq!(open(&version), Err(MessageError::Version));
}

// The keys read from `--new-identity`'s files are those it printed; no other runs (AT-ID-24).
#[test]
fn at_id_24_the_keys_are_new_identitys() {
    let c = Committee::new("keys", &[1]);
    for (dir, printed) in c.identities.iter().zip(&c.printed) {
        let keys = ceremony::member_keys(dir).unwrap();
        assert_eq!(keys.public.consortium, printed.consortium);
        assert_eq!(keys.public.hpke, printed.hpke);
    }
    let stranger = c.root.join("stranger");
    node::new_identity(&stranger).unwrap();
    let dir = c.root.join("ceremony");
    assert_eq!(
        ceremony::init(&c.procedure, &stranger, &dir).unwrap_err(),
        Error::NotMember
    );
    assert!(!dir.exists());
    let missing = ceremony::init(&c.procedure, &c.root.join("none"), &dir).unwrap_err();
    assert!(matches!(missing, Error::Io("member.key", _)));
}
