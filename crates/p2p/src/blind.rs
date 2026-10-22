//! Blind relays (`docs/04` §Blind relays, `docs/10` T85): a person's event sealed to a member's
//! key with Oblivious HTTP, carried over libp2p through a relay, logged on the circle feed.

use crate::{read_limited, write_all};
use bhttp::{Message as Http, Mode, StatusCode};
use ed25519_dalek::Signature;
use futures::StreamExt;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, noise, tcp, yamux, Multiaddr, StreamProtocol};
use network::codec::{Reader, Writer};
use network::consortium::Member;
use network::replica::EntryId;
use network::tier::{Tier, Tiers};
use ohttp::hpke::{Aead, Kdf, Kem};
use ohttp::{ClientRequest, ClientResponse, KeyConfig, Server, SymmetricSuite};
use std::io::{self, Cursor};
use std::time::Duration;

pub const RELAY_PROTOCOL: &str = "/isegoria/ohttp/relay/1";
pub const GATEWAY_PROTOCOL: &str = "/isegoria/ohttp/gateway/1";
/// The longest encapsulated request, and the longest encapsulated reply.
pub const MAX_REQUEST: usize = 1 << 20;
pub const MAX_REPLY: usize = 64 << 10;
/// The `Carried` envelope around a sealed reply: version, kind and the field's length.
const ENVELOPE: usize = 10;
const CONFIG_LEN: usize = 41;
const SUITE: [u8; 6] = [0, 4, 0, 1, 0, 1];

/// Member `member`'s OHTTP key configuration (RFC 9458 §3), signed by its consortium key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyAnnouncement {
    pub member: usize,
    pub config: Vec<u8>,
    pub signature: Signature,
}

impl KeyAnnouncement {
    /// Version 1, the index (8 bytes), the configuration as a field, the signature (64).
    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u8(1)
            .u64(self.member as u64)
            .field(&self.config)
            .fixed(&self.signature.to_bytes())
            .finish()
    }

    /// `None` unless `bytes` are exactly one encoding.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let member = usize::try_from(r.u64().ok()?).ok()?;
        let config = r.field().ok()?.to_vec();
        let signature = Signature::from_bytes(&r.fixed().ok()?);
        r.finish().ok()?;
        Some(KeyAnnouncement {
            member,
            config,
            signature,
        })
    }

    /// Whether this is member `member`'s configuration under `tiers`: signed by its consortium
    /// key for this network and member set, its index the key id, X25519 with this suite only.
    pub fn verify(&self, tiers: &Tiers) -> bool {
        let c = &self.config;
        c.len() == CONFIG_LEN
            && u8::try_from(self.member).is_ok_and(|id| c[0] == id)
            && c[1..3] == [0, 0x20]
            && c[CONFIG_LEN - SUITE.len()..] == SUITE
            && KeyConfig::decode(c).is_ok()
            && tiers.key_signed(self.member, c, &self.signature)
    }
}

/// A member gateway's key: its HPKE pair from the 32 bytes of `hpke_key_file` (RFC 9180
/// `DeriveKeyPair`), and its signed configuration.
pub struct GatewayKey {
    server: Server,
    announcement: KeyAnnouncement,
}

impl GatewayKey {
    /// `None` for an index beyond one byte or a key the HPKE library refuses.
    pub fn new(tiers: &Tiers, member: &Member, index: usize, ikm: &[u8; 32]) -> Option<Self> {
        let suite = SymmetricSuite::new(Kdf::HkdfSha256, Aead::Aes128Gcm);
        let id = u8::try_from(index).ok()?;
        let config = KeyConfig::derive(id, Kem::X25519Sha256, vec![suite], ikm).ok()?;
        let encoded = config.encode().ok()?;
        let signature = tiers.sign_key(member, index, &encoded);
        Some(GatewayKey {
            server: Server::new(config).ok()?,
            announcement: KeyAnnouncement {
                member: index,
                config: encoded,
                signature,
            },
        })
    }

    pub fn announcement(&self) -> &KeyAnnouncement {
        &self.announcement
    }
}

/// A gateway's admission check, supplied by its embedding: which bytes it would log.
pub type Admission = Box<dyn Fn(&[u8]) -> bool + Send>;

/// A member node's gateway: its key, the relays' transport keys it answers, its admission.
pub struct Gateway {
    pub key: GatewayKey,
    pub relays: Vec<[u8; 32]>,
    pub admit: Admission,
}

/// What a relay or a gateway answers: a sealed reply, or a refusal in the clear.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Carried {
    Sealed(Vec<u8>),
    Refused(Refusal),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    NotARelay = 1,
    UnknownKey = 2,
    Undecryptable = 3,
    Unreachable = 4,
}

impl Carried {
    /// Version 1, then kind 1 and the sealed reply as a field, or kind 2 and the refusal's code.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(1);
        match self {
            Carried::Sealed(bytes) => w.u8(1).field(bytes),
            Carried::Refused(r) => w.u8(2).u8(*r as u8),
        };
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let carried = match r.u8().ok()? {
            1 => Carried::Sealed(r.field().ok()?.to_vec()),
            2 => Carried::Refused(match r.u8().ok()? {
                1 => Refusal::NotARelay,
                2 => Refusal::UnknownKey,
                3 => Refusal::Undecryptable,
                4 => Refusal::Unreachable,
                _ => return None,
            }),
            _ => return None,
        };
        r.finish().ok()?;
        Some(carried)
    }
}

/// A gateway's sealed answer; a receipt attests the circle entry, not the protocol's verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Logged(EntryId),
    OtherNetwork,
    NotThisRequest,
    NotAdmitted,
    RetryLater,
    Failed,
}

impl Answer {
    fn status(self) -> u16 {
        match self {
            Answer::Logged(_) => 200,
            Answer::OtherNetwork => 421,
            Answer::NotThisRequest => 400,
            Answer::NotAdmitted => 403,
            Answer::RetryLater => 503,
            Answer::Failed => 500,
        }
    }

    /// The Binary HTTP response: its status, and for a receipt the entry's 72 bytes.
    pub(crate) fn to_http(self) -> Option<Vec<u8>> {
        let mut m = Http::response(StatusCode::try_from(self.status()).ok()?);
        if let Answer::Logged(id) = self {
            let mut body = id.writer.to_vec();
            body.extend_from_slice(&id.seq.to_le_bytes());
            body.extend_from_slice(&id.hash);
            m.write_content(body);
        }
        let mut out = Vec::new();
        m.write_bhttp(Mode::KnownLength, &mut out).ok()?;
        Some(out)
    }

    fn from_http(bytes: &[u8]) -> Option<Self> {
        let m = Http::read_bhttp(&mut Cursor::new(bytes)).ok()?;
        let answer = match m.control().status()?.code() {
            200 => {
                let b = m.content();
                (b.len() == 72).then_some(())?;
                Answer::Logged(EntryId {
                    writer: b[..32].try_into().ok()?,
                    seq: u64::from_le_bytes(b[32..40].try_into().ok()?),
                    hash: b[40..].try_into().ok()?,
                })
            }
            421 => Answer::OtherNetwork,
            400 => Answer::NotThisRequest,
            403 => Answer::NotAdmitted,
            503 => Answer::RetryLater,
            500 => Answer::Failed,
            _ => return None,
        };
        Some(answer)
    }
}

/// The path a person's event is posted to on network `network_id`.
pub fn event_path(network_id: &[u8; 32]) -> Vec<u8> {
    let hex: String = network_id.iter().map(|b| format!("{b:02x}")).collect();
    format!("/isegoria/v1/{hex}/event").into_bytes()
}

/// What an inner request asks: the event's bytes for this network, or why it is not that.
pub(crate) fn read_request(bytes: &[u8], network_id: &[u8; 32]) -> Result<Vec<u8>, Answer> {
    let m = Http::read_bhttp(&mut Cursor::new(bytes)).map_err(|_| Answer::NotThisRequest)?;
    let c = m.control();
    if c.method() != Some(b"POST") || c.scheme() != Some(b"isegoria") {
        return Err(Answer::NotThisRequest);
    }
    let path = c.path().ok_or(Answer::NotThisRequest)?;
    if path == event_path(network_id) {
        return Ok(m.content().to_vec());
    }
    let other = path
        .strip_prefix(b"/isegoria/v1/")
        .and_then(|p| p.strip_suffix(b"/event"))
        .is_some_and(|hex| hex.len() == 64 && hex.iter().all(u8::is_ascii_hexdigit));
    Err(if other {
        Answer::OtherNetwork
    } else {
        Answer::NotThisRequest
    })
}

/// The gateway's answer to an encapsulated request, sealed in that request's context.
pub(crate) fn open_request(
    key: &GatewayKey,
    request: &[u8],
) -> Result<(Vec<u8>, ohttp::ServerResponse), Refusal> {
    if request.first() != key.announcement.config.first() {
        return Err(Refusal::UnknownKey);
    }
    key.server
        .decapsulate(request)
        .map_err(|_| Refusal::Undecryptable)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientError {
    /// The key configuration is not its member's under the consortium the client holds.
    Unverified,
    TooLarge,
    Seal,
    Transport(String),
    Refused(Refusal),
    /// The reply does not open in this request's context: altered, or another request's.
    Open,
    /// It opened, but is no answer this channel gives, or a receipt naming another feed.
    Malformed,
}

/// A person's sealed event, and what it needs to open the reply.
pub struct Sealed {
    pub request: Vec<u8>,
    response: ClientResponse,
    circle: [u8; 32],
}

/// Seals `event` to `key` once `key` verifies under `tiers`, the configuration the client
/// already holds; never on a relay's word.
pub fn seal(tiers: &Tiers, key: &KeyAnnouncement, event: &[u8]) -> Result<Sealed, ClientError> {
    let circle = tiers.key(key.member, Tier::Circle);
    let circle = circle
        .filter(|_| key.verify(tiers))
        .ok_or(ClientError::Unverified)?;
    let mut m = Http::request(
        b"POST".to_vec(),
        b"isegoria".to_vec(),
        Vec::new(),
        event_path(&tiers.network_id()),
    );
    m.write_content(event);
    let mut inner = Vec::new();
    m.write_bhttp(Mode::KnownLength, &mut inner)
        .map_err(|_| ClientError::Seal)?;
    let client = ClientRequest::from_encoded_config(&key.config).map_err(|_| ClientError::Seal)?;
    let (request, response) = client.encapsulate(&inner).map_err(|_| ClientError::Seal)?;
    if request.len() > MAX_REQUEST {
        return Err(ClientError::TooLarge);
    }
    Ok(Sealed {
        request,
        response,
        circle,
    })
}

impl Sealed {
    /// The gateway's answer, once the reply opens in this request's context.
    pub fn open(self, carried: &Carried) -> Result<Answer, ClientError> {
        let sealed = match carried {
            Carried::Refused(r) => return Err(ClientError::Refused(*r)),
            Carried::Sealed(bytes) if bytes.len() > MAX_REPLY => return Err(ClientError::TooLarge),
            Carried::Sealed(bytes) => bytes,
        };
        let plain = self
            .response
            .decapsulate(sealed)
            .map_err(|_| ClientError::Open)?;
        match Answer::from_http(&plain) {
            Some(Answer::Logged(id)) if id.writer != self.circle => Err(ClientError::Malformed),
            Some(answer) => Ok(answer),
            None => Err(ClientError::Malformed),
        }
    }
}

/// The ohttp protocols' codec: requests and replies refused beyond their limits as read.
#[derive(Clone)]
pub(crate) struct Limited;

#[async_trait::async_trait]
impl request_response::Codec for Limited {
    type Protocol = StreamProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        read_limited(io, MAX_REQUEST as u64).await
    }

    async fn read_response<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Vec<u8>>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        let reply = read_limited(io, (MAX_REPLY + ENVELOPE) as u64).await?;
        match reply.is_empty() {
            true => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "closed unanswered",
            )),
            false => Ok(reply),
        }
    }

    async fn write_request<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        write_all(io, r).await
    }

    async fn write_response<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        r: Vec<u8>,
    ) -> io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        write_all(io, r).await
    }
}

pub(crate) fn behaviour(protocol: &'static str) -> request_response::Behaviour<Limited> {
    let protocols = [(StreamProtocol::new(protocol), ProtocolSupport::Full)];
    request_response::Behaviour::with_codec(Limited, protocols, Default::default())
}

/// Sends `request` to the relay at `relay` from a new transport identity, and returns what
/// the relay answers.
pub async fn submit(relay: Multiaddr, request: Vec<u8>) -> Result<Carried, ClientError> {
    let transport = |e: String| ClientError::Transport(e);
    let mut swarm =
        libp2p::SwarmBuilder::with_existing_identity(identity::Keypair::generate_ed25519())
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| transport(e.to_string()))?
            .with_behaviour(|_| behaviour(RELAY_PROTOCOL))
            .map_err(|e| transport(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(30)))
            .build();
    swarm.dial(relay).map_err(|e| transport(e.to_string()))?;
    let mut request = Some(request);
    let deadline = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => return Err(transport("no reply".into())),
            event = swarm.select_next_some() => match event {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    if let Some(r) = request.take() {
                        swarm.behaviour_mut().send_request(&peer_id, r);
                    }
                }
                SwarmEvent::OutgoingConnectionError { error, .. } => {
                    return Err(transport(error.to_string()));
                }
                SwarmEvent::Behaviour(request_response::Event::Message {
                    message: request_response::Message::Response { response, .. },
                    ..
                }) => {
                    return Carried::decode(&response).ok_or(ClientError::Malformed);
                }
                SwarmEvent::Behaviour(request_response::Event::OutboundFailure { error, .. }) => {
                    return Err(transport(error.to_string()));
                }
                _ => {}
            },
        }
    }
}
