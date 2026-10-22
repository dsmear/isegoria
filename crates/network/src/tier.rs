//! The replicated set's tiers (`docs/04` §Who reads, `docs/01` D54, `docs/10` T77): each
//! writer's member and tier, the transport binding that grants the circle, and the pull
//! sync's answers filtered by the reader's access.

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

/// What a peer reads: the circle only through [`Tiers::access`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    member: Option<usize>,
}

impl Access {
    pub const PUBLIC: Access = Access { member: None };

    /// The member whose verified binding granted the circle.
    pub fn member(self) -> Option<usize> {
        self.member
    }
}

/// Member `member`'s signature binding the transport key `transport`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportBinding {
    pub member: usize,
    pub transport: [u8; 32],
    pub signature: Signature,
}

/// Each member's two writers: its consortium key on its public feed, its circle key on its
/// circle feed; no other writer.
pub struct Tiers {
    network_id: [u8; 32],
    consortium: Consortium,
    writers: WriterSet,
    places: BTreeMap<[u8; 32], (usize, Tier)>,
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
        Ok(Tiers {
            writers: WriterSet::new(network_id, &keys),
            network_id,
            consortium,
            places,
        })
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
        Access {
            member: verified.map(|b| b.member),
        }
    }

    /// Whether `access` reads `writer`: public always, circle with a binding, unknown never.
    pub fn readable(&self, writer: &[u8; 32], access: Access) -> bool {
        match self.tier(writer) {
            Some(Tier::Public) => true,
            Some(Tier::Circle) => access.member.is_some(),
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
