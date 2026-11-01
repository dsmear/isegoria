//! A tiered network's transport (`docs/04` §Transport, a network with tiers; `docs/10` T77):
//! the binding exchanged on each connection, each peer's verified access, what reaches it.

use crate::member::PublicCollector;
use libp2p::{identity, PeerId};
use network::codec::{Reader, Writer};
use network::replica::{EntryId, Replica, SignedEntry, Summary, MAX_RESPONSE};
use network::tier::{Access, Tier, Tiers, TransportBinding};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

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

/// A tiered node's view of its peers: whose binding verified, and who granted it the circle,
/// each verification and grant numbered so that one withdrawn and given again is a new one.
pub(crate) struct Tiered {
    pub(crate) tiers: Tiers,
    pub(crate) binding: Option<TransportBinding>,
    pub(crate) public: PublicCollector,
    accesses: HashMap<PeerId, (Access, u64)>,
    grants: HashMap<PeerId, u64>,
    next: u64,
}

/// What a pull was asked under: the peer's member, binding number and grant.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Asked {
    pub(crate) member: usize,
    binding: u64,
    pub(crate) grant: Option<u64>,
}

/// Other members a start check needs (`docs/04` §Increments A and B, B 8).
pub(crate) const OTHER_MEMBERS: usize = 2;

/// A governed feed's start check (`docs/04` §Increments A and B, B 1–3).
#[derive(Default)]
pub(crate) struct Check {
    rounds: HashMap<PeerId, Asked>,
    done: Option<Vec<usize>>,
}

impl Check {
    /// Counts `peer`'s round if it still `stands`; whether it completed the check.
    pub(crate) fn count(&mut self, t: &Tiered, peer: PeerId, asked: Asked, circle: bool) -> bool {
        if self.done.is_some() {
            return false;
        }
        self.rounds.insert(peer, asked);
        self.rounds.retain(|p, a| t.stands(p, a, circle));
        let members = self.members(t, circle);
        let complete = members.len() >= OTHER_MEMBERS;
        if complete {
            self.done = Some(members);
        }
        complete
    }

    /// The other members whose rounds hold, sorted; once complete, those that completed it.
    pub(crate) fn members(&self, t: &Tiered, circle: bool) -> Vec<usize> {
        if let Some(done) = &self.done {
            return done.clone();
        }
        let standing = self.rounds.iter().filter(|(p, a)| t.stands(p, a, circle));
        let mut members: Vec<usize> = standing.map(|(_, a)| a.member).collect();
        members.sort_unstable();
        members.dedup();
        members
    }
}

impl Tiered {
    pub(crate) fn new(tiers: Tiers, binding: Option<TransportBinding>) -> Self {
        Tiered {
            public: PublicCollector::new(&tiers),
            tiers,
            binding,
            accesses: HashMap::new(),
            grants: HashMap::new(),
            next: 0,
        }
    }

    pub(crate) fn access(&self, peer: &PeerId) -> Access {
        self.accesses.get(peer).map_or(Access::PUBLIC, |(a, _)| *a)
    }

    /// Sets `peer`'s access from `binding`, checked against its authenticated key; whether it
    /// reads the circle.
    pub(crate) fn verify(&mut self, peer: &PeerId, binding: Option<&TransportBinding>) -> bool {
        let access = transport_key(peer).map_or(Access::PUBLIC, |k| self.tiers.access(&k, binding));
        let member = self.tiers.member(access).is_some();
        if !member {
            self.forget(peer);
        } else if self.accesses.get(peer).is_none_or(|(a, _)| *a != access) {
            self.next += 1;
            self.accesses.insert(*peer, (access, self.next));
        }
        member
    }

    /// Records `peer`'s reply: a grant if `granted` and its binding verified, kept under its
    /// number while it lasts; any other reply withdraws the grant.
    pub(crate) fn granted(&mut self, peer: PeerId, granted: bool) {
        if !granted || !self.accesses.contains_key(&peer) {
            self.grants.remove(&peer);
        } else if let Entry::Vacant(new) = self.grants.entry(peer) {
            self.next += 1;
            new.insert(self.next);
        }
    }

    pub(crate) fn forget(&mut self, peer: &PeerId) {
        self.accesses.remove(peer);
        self.grants.remove(peer);
    }

    /// The grant a pull asked of `peer` now is made under, if any.
    pub(crate) fn grant(&self, peer: &PeerId) -> Option<u64> {
        let grant = self.grants.get(peer).copied();
        grant.filter(|_| self.accesses.contains_key(peer))
    }

    /// Whether `grant` is still `peer`'s: not once withdrawn, even if granted again since.
    pub(crate) fn holds(&self, peer: &PeerId, grant: u64) -> bool {
        self.grant(peer) == Some(grant)
    }

    /// What a pull asked of `peer` now is asked under; `None` unless its binding verified.
    pub(crate) fn asked(&self, peer: &PeerId) -> Option<Asked> {
        let (access, binding) = self.accesses.get(peer)?;
        Some(Asked {
            member: self.tiers.member(*access)?,
            binding: *binding,
            grant: self.grant(peer),
        })
    }

    /// Whether `asked`'s binding, and for the circle its grant, are still `peer`'s (`holds`).
    pub(crate) fn stands(&self, peer: &PeerId, asked: &Asked, circle: bool) -> bool {
        let bound = self
            .accesses
            .get(peer)
            .is_some_and(|(_, n)| *n == asked.binding);
        bound && (!circle || asked.grant.is_some_and(|g| self.holds(peer, g)))
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

    /// A connected peer whose binding verified as member `member`'s.
    pub(crate) fn peer_of(&self, member: usize) -> Option<PeerId> {
        let mut bound = self.accesses.iter();
        let found = bound.find(|(_, (a, _))| self.tiers.member(*a) == Some(member));
        found.map(|(peer, _)| *peer)
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
