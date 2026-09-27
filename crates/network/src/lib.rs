//! Storage and network layer (`docs/04-storage-network.md`): content addressing, Merkle
//! trees, the transparency log, checkpoints, replication and erasure coding are real; the
//! DHT and live OpenTimestamps anchoring are plug points (the transport is the `p2p` crate).

pub mod anchoring;
pub mod beacon;
pub mod cid;
pub mod codec;
pub mod consortium;
pub mod cut;
pub mod erasure;
pub mod log;
pub mod merkle;
pub mod replica;
pub mod store;

mod hash;

/// Infrastructural node roles (`docs/04`), distinct from the three person actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// pseudonymous participant
    Person,
    /// always-on consortium signer holding a full copy
    Signer,
    /// light client: a slice of data + signature verification
    Light,
}
