//! A gateway's answer to opened requests on `…/read` and `…/event` (`docs/04` §Blind relays) with
//! a fake hook: arbitrary, crafted and client-sealed requests. No panic; only this network's POST
//! reaches the hook, with its body; each answer as the hook gave, a 500 past `MAX_REPLY`.
#![no_main]

use arbitrary::Arbitrary;
use bhttp::{Message, Mode};
use ed25519_dalek::Signature;
use libfuzzer_sys::fuzz_target;
use network::consortium::{Consortium, Member};
use network::cut::Cut;
use network::replica::{FeedWriter, Replica};
use network::tier::Tiers;
use p2p::blind::{
    event_path, read_path, seal, seal_read, Answer, Carried, GatewayKey, ReadAnswer, MAX_REPLY,
};
use p2p::fuzz::{gateway_answer, gateway_respond, read_answer_http, SEAL_OVERHEAD};
use p2p::hook::Hook;
use std::cell::RefCell;
use std::io::Cursor;
use std::sync::OnceLock;

const NET: [u8; 32] = [0xab; 32];

struct Fixture {
    tiers: Tiers,
    key: GatewayKey,
    replica: Replica,
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
        let key = GatewayKey::new(&tiers, &members[0], 0, &[7; 32]).expect("a gateway key");
        let replica = Replica::new(tiers.writers().clone());
        Fixture {
            tiers,
            key,
            replica,
        }
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Call {
    Admit(Vec<u8>),
    Read(Vec<u8>),
}

/// A hook that admits as told, replies to every read with `reply`, and records its calls.
struct Fake {
    admits: bool,
    reply: Option<Vec<u8>>,
    calls: RefCell<Vec<Call>>,
}

impl Hook for Fake {
    fn apply(
        &mut self,
        _: &Replica,
        _: &Cut,
        _: u64,
        _: &[(usize, Signature)],
    ) -> Result<Option<[u8; 32]>, String> {
        Err("unused".into())
    }

    fn may_sign(&self, _: &Cut) -> bool {
        false
    }

    fn closes(&self) -> bool {
        false
    }

    fn admit(&self, event: &[u8]) -> bool {
        self.calls.borrow_mut().push(Call::Admit(event.to_vec()));
        self.admits
    }

    fn read(&self, _: &Replica, request: &[u8]) -> Option<Vec<u8>> {
        self.calls.borrow_mut().push(Call::Read(request.to_vec()));
        self.reply.clone()
    }
}

#[derive(Arbitrary, Debug)]
enum Part {
    Expected,
    Raw(Vec<u8>),
}

#[derive(Arbitrary, Debug)]
enum Network {
    Ours,
    OursUpper,
    Other([u8; 32]),
    Raw(Vec<u8>),
}

#[derive(Arbitrary, Debug)]
enum Suffix {
    Event,
    Read,
    Raw(Vec<u8>),
}

#[derive(Arbitrary, Debug)]
enum Input {
    Inner(Vec<u8>),
    Crafted {
        method: Part,
        scheme: Part,
        prefix: Part,
        network: Network,
        suffix: Suffix,
        body: Vec<u8>,
    },
    Sealed {
        read: bool,
        body: Vec<u8>,
    },
    Request(Vec<u8>),
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The status and content of a gateway's Binary HTTP response.
fn status(response: &[u8]) -> (u16, Vec<u8>) {
    let m = Message::read_bhttp(&mut Cursor::new(response)).expect("a response reads");
    let code = m.control().status().expect("a response").code();
    (code, m.content().to_vec())
}

/// Whether `path` names an event or a read on some network, as 64 lower-case hex digits.
fn some_network(path: &[u8]) -> bool {
    let Some(rest) = path.strip_prefix(b"/isegoria/v1/") else {
        return false;
    };
    let id = rest
        .strip_suffix(b"/event")
        .or_else(|| rest.strip_suffix(b"/read"));
    id.is_some_and(|id| id.len() == 64 && id.iter().all(|b| b"0123456789abcdef".contains(b)))
}

/// The answer to opened `inner` checked against the request it parses as: the hook is called
/// once exactly for this network's POST, with its body; otherwise 400, or 421 for another network.
fn checked(inner: &[u8], fake: &Fake) -> (u16, Vec<u8>) {
    let f = fixture();
    let response = gateway_respond(inner, &NET, fake, &f.replica).expect("every answer encodes");
    let (code, content) = status(&response);
    let calls = fake.calls.borrow().clone();
    let parsed = Message::read_bhttp(&mut Cursor::new(inner)).ok();
    let posted = parsed.as_ref().filter(|m| {
        m.control().method() == Some(b"POST") && m.control().scheme() == Some(b"isegoria")
    });
    let path = posted.and_then(|m| m.control().path());
    let body = posted.map(|m| m.content().to_vec()).unwrap_or_default();
    match path {
        Some(p) if p == read_path(&NET) => {
            assert_eq!(
                calls,
                [Call::Read(body)],
                "a read not given to the hook as sent"
            );
            match &fake.reply {
                Some(reply) => assert_eq!((code, &content), (200, reply), "another reply"),
                None => assert_eq!(code, 403, "a refused read not answered 403"),
            }
        }
        Some(p) if p == event_path(&NET) => {
            assert_eq!(
                calls,
                [Call::Admit(body)],
                "an event not given to the hook as sent"
            );
            assert_eq!(
                code,
                if fake.admits { 503 } else { 403 },
                "another event answer"
            );
        }
        Some(p) if some_network(p) => {
            assert!(
                calls.is_empty(),
                "another network's request reached the hook"
            );
            assert_eq!(code, 421, "another network's request not answered 421");
        }
        _ => {
            assert!(
                calls.is_empty(),
                "a request not this network's reached the hook"
            );
            assert_eq!(code, 400, "a malformed request not answered 400");
        }
    }
    (code, content)
}

fn crafted(
    method: Part,
    scheme: Part,
    prefix: Part,
    network: Network,
    suffix: Suffix,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let or = |part: Part, expected: &[u8]| match part {
        Part::Expected => expected.to_vec(),
        Part::Raw(bytes) => bytes,
    };
    let network = match network {
        Network::Ours => hex(&NET).into_bytes(),
        Network::OursUpper => hex(&NET).to_uppercase().into_bytes(),
        Network::Other(id) => hex(&id).into_bytes(),
        Network::Raw(bytes) => bytes,
    };
    let suffix = match suffix {
        Suffix::Event => b"/event".to_vec(),
        Suffix::Read => b"/read".to_vec(),
        Suffix::Raw(bytes) => bytes,
    };
    let path = [or(prefix, b"/isegoria/v1/"), network, suffix].concat();
    (or(method, b"POST"), or(scheme, b"isegoria"), path)
}

/// A client's sealed event or read through the gateway: opened as the hook answered it.
fn sealed(read: bool, body: &[u8], fake: &Fake) {
    let f = fixture();
    let announcement = f.key.announcement();
    let seal_it = || match read {
        true => seal_read(&f.tiers, announcement, body),
        false => seal(&f.tiers, announcement, body),
    };
    let Ok(sealed) = seal_it() else {
        return;
    };
    let carried = gateway_answer(&f.key, &sealed.request, &NET, fake, &f.replica);
    let called = fake.calls.borrow_mut().drain(..).collect::<Vec<_>>();
    if read {
        assert_eq!(
            called,
            [Call::Read(body.to_vec())],
            "a sealed read not given as sent"
        );
        assert!(
            matches!(&carried, Carried::Sealed(b) if b.len() <= MAX_REPLY),
            "a read's answer not sealed within MAX_REPLY"
        );
        let too_large = |reply: &[u8]| {
            let http = read_answer_http(&ReadAnswer::Reply(reply.to_vec())).expect("it encodes");
            http.len() + SEAL_OVERHEAD > MAX_REPLY
        };
        let expected = match &fake.reply {
            Some(reply) if too_large(reply) => Ok(ReadAnswer::Answer(Answer::Failed)),
            Some(reply) => Ok(ReadAnswer::Reply(reply.clone())),
            None => Ok(ReadAnswer::Answer(Answer::NotAdmitted)),
        };
        assert_eq!(
            sealed.open_read(&carried),
            expected,
            "a read opened otherwise"
        );
    } else {
        assert_eq!(
            called,
            [Call::Admit(body.to_vec())],
            "a sealed event not given as sent"
        );
        let expected = if fake.admits {
            Answer::RetryLater
        } else {
            Answer::NotAdmitted
        };
        assert_eq!(
            sealed.open(&carried),
            Ok(expected),
            "an event opened otherwise"
        );
    }
    let Ok(again) = seal_it() else {
        return;
    };
    let elsewhere = gateway_answer(&f.key, &again.request, &[0xcd; 32], fake, &f.replica);
    assert!(
        fake.calls.borrow().is_empty(),
        "another network's request reached the hook"
    );
    let answer = match read {
        true => again.open_read(&elsewhere),
        false => again.open(&elsewhere).map(ReadAnswer::Answer),
    };
    assert_eq!(answer, Ok(ReadAnswer::Answer(Answer::OtherNetwork)));
}

fuzz_target!(|input: (bool, Option<Vec<u8>>, Input)| {
    let (admits, reply, input) = input;
    let f = fixture();
    let fake = Fake {
        admits,
        reply,
        calls: RefCell::new(Vec::new()),
    };
    match input {
        Input::Inner(bytes) => {
            checked(&bytes, &fake);
        }
        Input::Crafted {
            method,
            scheme,
            prefix,
            network,
            suffix,
            body,
        } => {
            let canonical = matches!(
                (&method, &scheme, &prefix, &suffix),
                (
                    Part::Expected,
                    Part::Expected,
                    Part::Expected,
                    Suffix::Event | Suffix::Read
                )
            );
            let ours = match &network {
                Network::Ours => Some(true),
                Network::Other(id) => Some(*id == NET),
                Network::OursUpper | Network::Raw(_) => None,
            };
            let upper = matches!(network, Network::OursUpper);
            let (method, scheme, path) = crafted(method, scheme, prefix, network, suffix);
            let mut m = Message::request(method, scheme, Vec::new(), path);
            m.write_content(&body);
            let mut inner = Vec::new();
            if m.write_bhttp(Mode::KnownLength, &mut inner).is_err() {
                return;
            }
            let (code, _) = checked(&inner, &fake);
            match (canonical, ours) {
                (true, Some(true)) => assert!(!fake.calls.borrow().is_empty(), "ours unheard"),
                (true, Some(false)) => assert_eq!(code, 421, "another network's not 421"),
                (true, None) if upper => assert_eq!(code, 400, "our id in upper case not 400"),
                _ => {}
            }
        }
        Input::Sealed { read, body } => sealed(read, &body, &fake),
        Input::Request(bytes) => {
            let carried = gateway_answer(&f.key, &bytes, &NET, &fake, &f.replica);
            if !fake.calls.borrow().is_empty() {
                assert!(
                    matches!(carried, Carried::Sealed(_)),
                    "a refused request heard"
                );
            }
        }
    }
});
