//! What a person's client accepts from a relay for a read (`docs/05` T79-P7): arbitrary `Carried`
//! bytes, any plaintext sealed in its request's context, every gateway answer and reply. No panic;
//! a foreign reply never opens; a read never opens to a receipt; a reply reads back as sealed.
#![no_main]

use arbitrary::Arbitrary;
use bhttp::Message;
use libfuzzer_sys::fuzz_target;
use network::consortium::{Consortium, Member};
use network::replica::{EntryId, FeedWriter};
use network::tier::Tiers;
use ohttp::hpke::{Aead, Kdf, Kem};
use ohttp::{KeyConfig, Server, SymmetricSuite};
use p2p::blind::{
    seal, seal_read, Answer, Carried, ClientError, GatewayKey, KeyAnnouncement, ReadAnswer, Sealed,
    MAX_REPLY,
};
use p2p::fuzz::{answer_http, read_answer_http};
use std::io::Cursor;
use std::sync::OnceLock;

const NET: [u8; 32] = [4; 32];
const IKM: [u8; 32] = [8; 32];

struct Fixture {
    tiers: Tiers,
    announcement: KeyAnnouncement,
    server: Server,
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
        let key = GatewayKey::new(&tiers, &members[2], 2, &IKM).expect("a gateway key");
        let suite = SymmetricSuite::new(Kdf::HkdfSha256, Aead::Aes128Gcm);
        let config = KeyConfig::derive(2, Kem::X25519Sha256, vec![suite], &IKM).expect("derives");
        Fixture {
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
        writer: [u8; 32],
        seq: u64,
        hash: [u8; 32],
    },
    Sealed(Vec<u8>),
}

/// `plain` sealed as the gateway seals its reply to `sealed`'s request.
fn in_context(sealed: &Sealed, plain: &[u8]) -> Carried {
    let f = fixture();
    let (_, response) = (f.server)
        .decapsulate(&sealed.request)
        .expect("the key opens it");
    Carried::Sealed(response.encapsulate(plain).expect("a reply seals"))
}

fuzz_target!(|input: (Vec<u8>, Reply)| {
    let (request, reply) = input;
    let f = fixture();
    let Ok(sealed) = seal_read(&f.tiers, &f.announcement, &request) else {
        return;
    };
    let event = seal(&f.tiers, &f.announcement, &request).expect("a verified key seals");
    let carried = match reply {
        Reply::Carried(bytes) => {
            let Some(carried) = Carried::decode(&bytes) else {
                return;
            };
            assert_eq!(carried.encode(), bytes, "decoded from another encoding");
            assert!(
                sealed.open_read(&carried).is_err(),
                "a foreign reply opened"
            );
            carried
        }
        Reply::Plain(plain) => {
            let carried = in_context(&sealed, &plain);
            match sealed.open_read(&carried) {
                Ok(ReadAnswer::Answer(Answer::Logged(_))) => panic!("a read opened to a receipt"),
                Ok(ReadAnswer::Reply(body)) => {
                    let m = Message::read_bhttp(&mut Cursor::new(&plain)).expect("it reads");
                    let code = m.control().status().map(|s| s.code());
                    assert_eq!((code, m.content()), (Some(200), &body[..]), "another reply");
                }
                _ => {}
            }
            carried
        }
        Reply::Answer {
            kind,
            writer,
            seq,
            hash,
        } => {
            let id = EntryId { writer, seq, hash };
            let answer = match kind % 6 {
                0 => Answer::Logged(id),
                1 => Answer::OtherNetwork,
                2 => Answer::NotThisRequest,
                3 => Answer::NotAdmitted,
                4 => Answer::RetryLater,
                _ => Answer::Failed,
            };
            let carried = in_context(&sealed, &answer_http(answer).expect("an answer encodes"));
            let expected = match answer {
                Answer::Logged(_) => {
                    ReadAnswer::Reply([&writer[..], &seq.to_le_bytes(), &hash].concat())
                }
                other => ReadAnswer::Answer(other),
            };
            assert_eq!(
                sealed.open_read(&carried),
                Ok(expected),
                "an answer opened otherwise"
            );
            carried
        }
        Reply::Sealed(bytes) => {
            let http = read_answer_http(&ReadAnswer::Reply(bytes.clone())).expect("it encodes");
            let carried = in_context(&sealed, &http);
            let expected = match &carried {
                Carried::Sealed(b) if b.len() > MAX_REPLY => Err(ClientError::TooLarge),
                _ => Ok(ReadAnswer::Reply(bytes)),
            };
            assert_eq!(
                sealed.open_read(&carried),
                expected,
                "a reply opened otherwise"
            );
            carried
        }
    };
    assert_eq!(
        event.open_read(&carried),
        Err(ClientError::Malformed),
        "an event's reply opened as a read's"
    );
});
