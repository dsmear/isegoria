//! What a person's client accepts from a relay (`docs/04` §Blind relays): arbitrary `Carried`
//! bytes, any plaintext sealed in its request's context, and every gateway answer. No panic;
//! one encoding; a foreign reply never opens; an answer reads back as sealed.
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use network::consortium::{Consortium, Member};
use network::replica::{EntryId, FeedWriter};
use network::tier::{Tier, Tiers};
use ohttp::hpke::{Aead, Kdf, Kem};
use ohttp::{KeyConfig, Server, SymmetricSuite};
use p2p::blind::{seal, Answer, Carried, ClientError, GatewayKey, KeyAnnouncement};
use p2p::fuzz::answer_http;
use std::sync::OnceLock;

const NET: [u8; 32] = [3; 32];
const IKM: [u8; 32] = [7; 32];

struct Fixture {
    tiers: Tiers,
    announcement: KeyAnnouncement,
    server: Server,
    circle: [u8; 32],
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let members: Vec<Member> = (1..=3).map(|i| Member::from_seed([i; 32])).collect();
        let circle: Vec<_> = (4..=6)
            .map(|i| FeedWriter::from_seed(NET, [i; 32]).public())
            .collect();
        let keys = members.iter().map(|m| m.public()).collect();
        let tiers = Tiers::new(NET, Consortium::new(keys, 2), &circle).expect("a tiered set");
        let key = GatewayKey::new(&tiers, &members[1], 1, &IKM).expect("a gateway key");
        let suite = SymmetricSuite::new(Kdf::HkdfSha256, Aead::Aes128Gcm);
        let config = KeyConfig::derive(1, Kem::X25519Sha256, vec![suite], &IKM).expect("derives");
        Fixture {
            circle: tiers.key(1, Tier::Circle).expect("member 1's circle key"),
            announcement: key.announcement().clone(),
            server: Server::new(config).expect("a server"),
            tiers,
        }
    })
}

#[derive(Arbitrary, Debug)]
enum Reply {
    Carried(Vec<u8>),
    Plain(Vec<u8>),
    Answer {
        kind: u8,
        own_circle: bool,
        writer: [u8; 32],
        seq: u64,
        hash: [u8; 32],
    },
}

fuzz_target!(|input: (Vec<u8>, Reply)| {
    let (event, reply) = input;
    let f = fixture();
    let sealed = seal(&f.tiers, &f.announcement, &event).expect("a verified key seals");
    let sealed_in_context = |plain: &[u8]| {
        let (_, response) = f
            .server
            .decapsulate(&sealed.request)
            .expect("the key opens it");
        Carried::Sealed(response.encapsulate(plain).expect("a reply seals"))
    };
    match reply {
        Reply::Carried(bytes) => {
            if let Some(carried) = Carried::decode(&bytes) {
                assert_eq!(carried.encode(), bytes, "decoded from another encoding");
                assert!(sealed.open(&carried).is_err(), "a foreign reply opened");
            }
        }
        Reply::Plain(plain) => {
            let carried = sealed_in_context(&plain);
            if let Ok(Answer::Logged(id)) = sealed.open(&carried) {
                assert_eq!(id.writer, f.circle, "a receipt naming another feed");
            }
        }
        Reply::Answer {
            kind,
            own_circle,
            writer,
            seq,
            hash,
        } => {
            let writer = if own_circle { f.circle } else { writer };
            let answer = match kind % 6 {
                0 => Answer::Logged(EntryId { writer, seq, hash }),
                1 => Answer::OtherNetwork,
                2 => Answer::NotThisRequest,
                3 => Answer::NotAdmitted,
                4 => Answer::RetryLater,
                _ => Answer::Failed,
            };
            let carried = sealed_in_context(&answer_http(answer).expect("an answer encodes"));
            let expected = match answer {
                Answer::Logged(id) if id.writer != f.circle => Err(ClientError::Malformed),
                _ => Ok(answer),
            };
            assert_eq!(sealed.open(&carried), expected);
        }
    }
});
