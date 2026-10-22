//! A tiered network's transport (`docs/04` §Transport, a network with tiers; `docs/10` T77):
//! the binding exchanged on each connection, each peer's verified access, what reaches it.

use crate::member::PublicCollector;
use libp2p::{identity, PeerId};
use network::codec::{Reader, Writer};
use network::replica::{EntryId, Replica, SignedEntry, Summary, MAX_RESPONSE};
use network::tier::{Access, Tier, Tiers, TransportBinding};
use std::collections::{HashMap, HashSet};

const BINDING_LEN: usize = 105;

/// The bind protocol's messages: a hello on each new connection, and the reply to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bind {
    Hello(Option<TransportBinding>),
    /// Whether the replier granted the hello's sender the circle, and its own binding.
    Reply {
        granted: bool,
        binding: Option<TransportBinding>,
    },
}

fn write_binding(w: &mut Writer, binding: Option<&TransportBinding>) {
    match binding {
        None => w.u8(0),
        Some(b) => w.u8(1).fixed(&b.encode()),
    };
}

fn read_binding(r: &mut Reader) -> Option<Option<TransportBinding>> {
    match r.u8().ok()? {
        0 => Some(None),
        1 => Some(Some(TransportBinding::decode(
            &r.fixed::<BINDING_LEN>().ok()?,
        )?)),
        _ => None,
    }
}

impl Bind {
    /// Version 1, kind 1 (hello) or 2 (reply, then granted as 0 or 1), then 0 for no binding
    /// or 1 and its 105 bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(1);
        match self {
            Bind::Hello(binding) => {
                w.u8(1);
                write_binding(&mut w, binding.as_ref());
            }
            Bind::Reply { granted, binding } => {
                w.u8(2).u8(u8::from(*granted));
                write_binding(&mut w, binding.as_ref());
            }
        }
        w.finish()
    }

    /// `None` unless `bytes` are exactly one message.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let bind = match r.u8().ok()? {
            1 => Bind::Hello(read_binding(&mut r)?),
            2 => {
                let granted = match r.u8().ok()? {
                    0 => false,
                    1 => true,
                    _ => return None,
                };
                let binding = read_binding(&mut r)?;
                Bind::Reply { granted, binding }
            }
            _ => return None,
        };
        r.finish().ok()?;
        Some(bind)
    }
}

/// The ed25519 key `peer`'s id inlines, the one Noise authenticated; `None` for another type.
pub fn transport_key(peer: &PeerId) -> Option<[u8; 32]> {
    let hash = peer.as_ref();
    if hash.code() != 0 {
        return None;
    }
    let key = identity::PublicKey::try_decode_protobuf(hash.digest()).ok()?;
    if key.to_peer_id() != *peer {
        return None;
    }
    Some(key.try_into_ed25519().ok()?.to_bytes())
}

/// A tiered node's view of its peers: whose binding verified, and who granted it the circle.
pub(crate) struct Tiered {
    pub(crate) tiers: Tiers,
    pub(crate) binding: Option<TransportBinding>,
    pub(crate) public: PublicCollector,
    accesses: HashMap<PeerId, Access>,
    granted_by: HashSet<PeerId>,
}

impl Tiered {
    pub(crate) fn new(tiers: Tiers, binding: Option<TransportBinding>) -> Self {
        Tiered {
            public: PublicCollector::new(&tiers),
            tiers,
            binding,
            accesses: HashMap::new(),
            granted_by: HashSet::new(),
        }
    }

    pub(crate) fn access(&self, peer: &PeerId) -> Access {
        self.accesses.get(peer).copied().unwrap_or(Access::PUBLIC)
    }

    /// Sets `peer`'s access from `binding`, checked against its authenticated key; whether it
    /// reads the circle.
    pub(crate) fn verify(&mut self, peer: &PeerId, binding: Option<&TransportBinding>) -> bool {
        let access = transport_key(peer).map_or(Access::PUBLIC, |k| self.tiers.access(&k, binding));
        let member = self.tiers.member(access).is_some();
        if member {
            self.accesses.insert(*peer, access);
        } else {
            self.accesses.remove(peer);
            self.granted_by.remove(peer);
        }
        member
    }

    /// Records that `peer`, a verified member, granted this node the circle.
    pub(crate) fn granted(&mut self, peer: PeerId) {
        if self.accesses.contains_key(&peer) {
            self.granted_by.insert(peer);
        }
    }

    pub(crate) fn forget(&mut self, peer: &PeerId) {
        self.accesses.remove(peer);
        self.granted_by.remove(peer);
    }

    /// Whether a pull asked of `peer` now answers for the circle feed's catch-up.
    pub(crate) fn circle_pull(&self, peer: &PeerId) -> bool {
        self.granted_by.contains(peer) && self.accesses.contains_key(peer)
    }

    /// The summary to send `peer`, and the answers to its Summary and Want.
    pub(crate) fn summary(&self, replica: &Replica, peer: &PeerId) -> Summary {
        self.tiers.serve(replica, self.access(peer)).summary()
    }

    pub(crate) fn have_for(&self, replica: &Replica, peer: &PeerId, s: &Summary) -> Vec<EntryId> {
        self.tiers.serve(replica, self.access(peer)).have_for(s)
    }

    pub(crate) fn entries_for(
        &self,
        replica: &Replica,
        peer: &PeerId,
        ids: &[EntryId],
    ) -> Vec<(SignedEntry, Vec<u8>)> {
        let served = self.tiers.serve(replica, self.access(peer));
        served.entries_for(ids, MAX_RESPONSE)
    }

    pub(crate) fn reads(&self, peer: &PeerId, writer: &[u8; 32]) -> bool {
        self.tiers.readable(writer, self.access(peer))
    }

    pub(crate) fn is_public(&self, id: &EntryId) -> bool {
        self.tiers.tier(&id.writer) == Some(Tier::Public)
    }

    /// Whether `peer`'s binding verified.
    pub(crate) fn is_bound(&self, peer: &PeerId) -> bool {
        self.accesses.contains_key(peer)
    }

    /// The connected peers whose binding verified.
    pub(crate) fn bound(&self) -> Vec<PeerId> {
        self.accesses.keys().copied().collect()
    }
}
