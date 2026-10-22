//! The replicated set's tiers (`docs/04` §Who reads, `docs/01` D54, `docs/10` T77): each
//! writer's member and tier, the transport binding that grants the circle, and the pull
//! sync's answers filtered by the reader's access.

use crate::codec::{Reader, Writer};
use crate::consortium::{Consortium, Member};
use crate::hash::tagged;
use crate::replica::{EntryId, Replica, SignedEntry, Summary, WriterSet};
use ed25519_dalek::{Signature, Signer, VerifyingKey};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Public,
    Circle,
}

/// Why a tiered writer set does not hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierError {
    /// Not exactly one circle key per member.
    Count,
    /// A circle key equal to a member key or to another circle key.
    Repeated { key: [u8; 32] },
}

/// What a peer reads: the circle only as [`Tiers::access`] verified it, under that same set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    member: Option<usize>,
    context: [u8; 32],
}

impl Access {
    pub const PUBLIC: Access = Access {
        member: None,
        context: [0; 32],
    };
}

/// Member `member`'s signature binding the transport key `transport`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportBinding {
    pub member: usize,
    pub transport: [u8; 32],
    pub signature: Signature,
}

impl TransportBinding {
    /// The 105-byte encoding: version 1, the index, the transport key, the signature.
    pub fn encode(&self) -> Vec<u8> {
        Writer::new()
            .u8(1)
            .u64(self.member as u64)
            .fixed(&self.transport)
            .fixed(&self.signature.to_bytes())
            .finish()
    }

    /// `None` unless `bytes` are exactly one encoding.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let binding = TransportBinding {
            member: usize::try_from(r.u64().ok()?).ok()?,
            transport: r.fixed().ok()?,
            signature: Signature::from_bytes(&r.fixed().ok()?),
        };
        r.finish().ok()?;
        Some(binding)
    }
}

/// Each member's two writers: its consortium key on its public feed, its circle key on its
/// circle feed; no other writer.
pub struct Tiers {
    network_id: [u8; 32],
    consortium: Consortium,
    writers: WriterSet,
    places: BTreeMap<[u8; 32], (usize, Tier)>,
    context: [u8; 32],
}

impl Tiers {
    /// `circle[i]` is member `i`'s circle key.
    pub fn new(
        network_id: [u8; 32],
        consortium: Consortium,
        circle: &[VerifyingKey],
    ) -> Result<Self, TierError> {
        if circle.len() != consortium.keys().len() {
            return Err(TierError::Count);
        }
        let mut places = BTreeMap::new();
        for (i, key) in consortium.keys().iter().enumerate() {
            places.insert(key.to_bytes(), (i, Tier::Public));
        }
        for (i, key) in circle.iter().enumerate() {
            if places.insert(key.to_bytes(), (i, Tier::Circle)).is_some() {
                return Err(TierError::Repeated {
                    key: key.to_bytes(),
                });
            }
        }
        let keys: Vec<VerifyingKey> = consortium.keys().iter().chain(circle).copied().collect();
        let threshold = (consortium.threshold() as u64).to_le_bytes();
        let mut fields: Vec<&[u8]> = vec![&network_id, &threshold];
        let member_set = consortium.member_set_hash();
        fields.push(&member_set);
        let circle: Vec<[u8; 32]> = circle.iter().map(|k| k.to_bytes()).collect();
        fields.extend(circle.iter().map(|k| k.as_slice()));
        Ok(Tiers {
            writers: WriterSet::new(network_id, &keys),
            context: tagged("isegoria/tiers/context/v1", &fields),
            network_id,
            consortium,
            places,
        })
    }

    /// The digest of the whole configuration: an [`Access`] and an observer hold to it.
    pub(crate) fn context(&self) -> [u8; 32] {
        self.context
    }

    pub fn network_id(&self) -> [u8; 32] {
        self.network_id
    }

    pub fn consortium(&self) -> &Consortium {
        &self.consortium
    }

    pub fn writers(&self) -> &WriterSet {
        &self.writers
    }

    /// The member and tier of `writer`'s feed; `None` for a key outside the set.
    pub fn place(&self, writer: &[u8; 32]) -> Option<(usize, Tier)> {
        self.places.get(writer).copied()
    }

    pub fn tier(&self, writer: &[u8; 32]) -> Option<Tier> {
        self.place(writer).map(|(_, tier)| tier)
    }

    fn message(&self, index: usize, transport: &[u8; 32]) -> [u8; 32] {
        tagged(
            "isegoria/transport/bind/v1",
            &[
                &self.network_id,
                &self.consortium.member_set_hash(),
                &(index as u64).to_le_bytes(),
                transport,
            ],
        )
    }

    /// Member `member`'s writer key on `tier`.
    pub fn key(&self, member: usize, tier: Tier) -> Option<[u8; 32]> {
        let mut found = self.places.iter().filter(|(_, p)| **p == (member, tier));
        found.next().map(|(key, _)| *key)
    }

    fn key_message(&self, index: usize, config: &[u8]) -> [u8; 32] {
        tagged(
            "isegoria/ohttp/key/v1",
            &[
                &self.network_id,
                &self.consortium.member_set_hash(),
                &(index as u64).to_le_bytes(),
                config,
            ],
        )
    }

    /// Member `index`'s signature over its OHTTP key configuration (`docs/04` §Blind relays).
    pub fn sign_key(&self, member: &Member, index: usize, config: &[u8]) -> Signature {
        member.key.sign(&self.key_message(index, config))
    }

    /// Whether `signature` is member `index`'s over `config`, on this network and member set.
    pub fn key_signed(&self, index: usize, config: &[u8], signature: &Signature) -> bool {
        let message = self.key_message(index, config);
        let key = self.consortium.keys().get(index);
        key.is_some_and(|k| k.verify_strict(&message, signature).is_ok())
    }

    /// Member `index`'s binding of `transport`, signed with `member`'s key.
    pub fn bind(&self, member: &Member, index: usize, transport: [u8; 32]) -> TransportBinding {
        TransportBinding {
            member: index,
            transport,
            signature: member.key.sign(&self.message(index, &transport)),
        }
    }

    /// The circle for a peer whose transport key is `transport`, if `binding` verifies.
    pub fn access(&self, transport: &[u8; 32], binding: Option<&TransportBinding>) -> Access {
        let verified = binding.filter(|b| {
            let message = self.message(b.member, &b.transport);
            b.transport == *transport
                && self
                    .consortium
                    .keys()
                    .get(b.member)
                    .is_some_and(|k| k.verify_strict(&message, &b.signature).is_ok())
        });
        verified.map_or(Access::PUBLIC, |b| Access {
            member: Some(b.member),
            context: self.context,
        })
    }

    /// The member whose binding `access` verified, if this tiered set verified it.
    pub fn member(&self, access: Access) -> Option<usize> {
        access.member.filter(|_| access.context == self.context)
    }

    /// Whether `access` reads `writer`: public always, circle with a binding, unknown never.
    pub fn readable(&self, writer: &[u8; 32], access: Access) -> bool {
        match self.tier(writer) {
            Some(Tier::Public) => true,
            Some(Tier::Circle) => self.member(access).is_some(),
            None => false,
        }
    }

    pub fn serve<'a>(&'a self, replica: &'a Replica, access: Access) -> Served<'a> {
        Served {
            tiers: self,
            replica,
            access,
        }
    }
}

/// A replica's pull-sync answers to a reader with `access`.
pub struct Served<'a> {
    tiers: &'a Tiers,
    replica: &'a Replica,
    access: Access,
}

impl Served<'_> {
    /// The summary to send this reader.
    pub fn summary(&self) -> Summary {
        let mut summary = self.replica.summary();
        summary.0.retain(|w| self.readable(&w.writer));
        summary
    }

    /// As [`Replica::have_for`], over the entries this reader may read.
    pub fn have_for(&self, peer: &Summary) -> Vec<EntryId> {
        let mut have = self.replica.have_for(peer);
        have.retain(|id| self.readable(&id.writer));
        have
    }

    /// As [`Replica::entries_for`], over the entries this reader may read.
    pub fn entries_for(&self, wanted: &[EntryId], cap: usize) -> Vec<(SignedEntry, Vec<u8>)> {
        let wanted: Vec<EntryId> = wanted
            .iter()
            .filter(|id| self.readable(&id.writer))
            .copied()
            .collect();
        self.replica.entries_for(&wanted, cap)
    }

    fn readable(&self, writer: &[u8; 32]) -> bool {
        self.tiers.readable(writer, self.access)
    }
}
