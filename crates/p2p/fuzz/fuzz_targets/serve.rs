//! What a tiered node serves of discovery and archives (`docs/04` §Public object discovery,
//! §Archive holders and transfer): arbitrary object and shard requests, read and answered.
//! No panic; a request reads exactly when well formed; only a held public object is served.
#![no_main]

use arbitrary::Arbitrary;
use futures::executor::block_on;
use futures::io::Cursor;
use libfuzzer_sys::fuzz_target;
use libp2p::request_response::Codec;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use network::tier::Tiers;
use p2p::archive::{self, ShardCodec};
use p2p::discovery::{object_protocol, ObjectCodec};
use p2p::fuzz::{serve_object, serve_shard};
use std::sync::OnceLock;

const NET: [u8; 32] = [1; 32];

struct Fixture {
    tiers: Tiers,
    replica: Replica,
    public: Vec<u8>,
    circle: Vec<u8>,
}

fn request(object: &[u8]) -> Vec<u8> {
    let mut r = vec![1];
    r.extend(cid(object).0);
    r
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
        let mut replica = Replica::new(tiers.writers().clone());
        for (seed, object) in [(1, b"public".to_vec()), (4, b"circle".to_vec())] {
            let mut log = TransparencyLog::new();
            let entry = log.append(cid(&object)).clone();
            let signed = FeedWriter::from_seed(NET, [seed; 32]).sign(&entry);
            replica.insert(signed, object).expect("a valid entry");
        }
        Fixture {
            tiers,
            replica,
            public: request(b"public"),
            circle: request(b"circle"),
        }
    })
}

#[derive(Arbitrary, Debug)]
enum Input {
    Object(Vec<u8>),
    HeldPublic,
    HeldCircle,
    Shard(Vec<u8>),
}

fuzz_target!(|input: Input| {
    let f = fixture();
    let bytes = match input {
        Input::Shard(bytes) => {
            let read = block_on(
                ShardCodec.read_request(&archive::protocol(&NET), &mut Cursor::new(bytes.clone())),
            );
            assert_eq!(read.is_ok(), bytes.len() == 41 && bytes[0] == 1);
            let served = serve_shard(&f.tiers, &bytes);
            assert_eq!(served, Some(vec![1, 0]), "an empty store serves nothing");
            return;
        }
        Input::Object(bytes) => bytes,
        Input::HeldPublic => f.public.clone(),
        Input::HeldCircle => f.circle.clone(),
    };
    let read =
        block_on(ObjectCodec.read_request(&object_protocol(&NET), &mut Cursor::new(bytes.clone())));
    assert_eq!(read.is_ok(), bytes.len() == 33 && bytes[0] == 1);
    let reply = serve_object(&f.tiers, &f.replica, &bytes);
    assert_eq!(
        reply != [1, 0],
        bytes == f.public,
        "only the held public object"
    );
});
