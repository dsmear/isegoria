//! Cuts (`docs/04` §Cuts, `docs/08` §10.3, T74): the consortium-signed marks that fix which
//! replicated entries count and in what order they are applied.

use crate::beacon::{BeaconCommit, BeaconReveal, RoundId};
use crate::codec::{Reader, Writer};
use crate::consortium::{Checkpoint, Consortium, Member};
use crate::hash::tagged;
use crate::replica::{EntryId, Replica};
use ed25519_dalek::Signature;
use std::collections::BTreeMap;

const MARK_LEN: usize = 72;

/// A writer's feed as a cut counts it: `len` entries ending with `head`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    pub writer: [u8; 32],
    pub len: u64,
    pub head: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cut {
    pub number: u64,
    pub epoch: u64,
    /// Whether this cut closes its epoch's deposits.
    pub closes: bool,
    /// Sorted by writer, no writer twice, no length 0.
    pub marks: Vec<Mark>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutError {
    /// Marks unsorted, repeated or empty.
    Malformed,
    /// The cut's number is not the one after the last applied.
    NotNext { expected: u64 },
    /// It drops, shortens or rewrites a writer the previous cut counted.
    Retracts { writer: [u8; 32] },
    /// The replica lacks an entry of this writer's chain: wait for sync.
    Missing { writer: [u8; 32] },
    /// An epoch before the last cut's, or not after an epoch the last cut closed.
    Epoch,
}

impl Cut {
    /// Cut `number` of epoch 0, not closing, marking every writer's feed as it stands.
    pub fn of(replica: &Replica, number: u64) -> Cut {
        let marks = replica
            .summary()
            .0
            .into_iter()
            .filter(|w| w.feed_len > 0)
            .map(|w| Mark {
                writer: w.writer,
                len: w.feed_len,
                head: w.head,
            })
            .collect();
        Cut {
            number,
            epoch: 0,
            closes: false,
            marks,
        }
    }

    /// The cut after `prev` a proposer puts forward: each writer's feed where it extends
    /// `prev`'s mark, `prev`'s mark where it does not.
    pub fn next(replica: &Replica, prev: Option<&Cut>, epoch: u64, closes: bool) -> Cut {
        let now = Cut::of(replica, 0).marks;
        let mut marks: BTreeMap<[u8; 32], Mark> = now.iter().map(|m| (m.writer, *m)).collect();
        for old in prev.map_or(&[][..], |p| &p.marks[..]) {
            let extends = marks.get(&old.writer).is_some_and(|m| {
                m.len >= old.len
                    && replica.chain(&m.writer, m.len, m.head).and_then(|c| {
                        let at = usize::try_from(old.len.checked_sub(1)?).ok()?;
                        c.get(at).map(|e| e.entry.hash)
                    }) == Some(old.head)
            });
            if !extends {
                marks.insert(old.writer, *old);
            }
        }
        Cut {
            number: prev.map_or(0, |p| p.number + 1),
            epoch,
            closes,
            marks: marks.into_values().collect(),
        }
    }

    fn write_marks(&self, w: &mut Writer) {
        for m in &self.marks {
            w.fixed(&m.writer).u64(m.len).fixed(&m.head);
        }
    }

    pub fn digest(&self) -> [u8; 32] {
        let mut w = Writer::new();
        self.write_marks(&mut w);
        tagged(
            "isegoria/cut/v1",
            &[
                &self.number.to_le_bytes(),
                &self.epoch.to_le_bytes(),
                &[u8::from(self.closes)],
                &w.finish(),
            ],
        )
    }

    /// The checkpoint the consortium signs for this cut: height its number, head its digest.
    pub fn checkpoint(&self, network_id: [u8; 32], member_set_hash: [u8; 32]) -> Checkpoint {
        Checkpoint::new(network_id, member_set_hash, self.number, self.digest())
    }

    pub fn is_well_formed(&self) -> bool {
        self.marks.iter().all(|m| m.len > 0)
            && self.marks.windows(2).all(|p| p[0].writer < p[1].writer)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut marks = Writer::new();
        self.write_marks(&mut marks);
        Writer::new()
            .u8(1)
            .u64(self.number)
            .u64(self.epoch)
            .u8(u8::from(self.closes))
            .field(&marks.finish())
            .finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Cut> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let number = r.u64().ok()?;
        let epoch = r.u64().ok()?;
        let closes = match r.u8().ok()? {
            0 => false,
            1 => true,
            _ => return None,
        };
        let field = r.field().ok()?;
        r.finish().ok()?;
        if field.len() % MARK_LEN != 0 {
            return None;
        }
        let mut f = Reader::new(field);
        let marks = (0..field.len() / MARK_LEN)
            .map(|_| {
                Some(Mark {
                    writer: f.fixed().ok()?,
                    len: f.u64().ok()?,
                    head: f.fixed().ok()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let cut = Cut {
            number,
            epoch,
            closes,
            marks,
        };
        cut.is_well_formed().then_some(cut)
    }
}

/// The entries `next` adds after `prev` (`None` before cut 0), in the order they apply.
pub fn added(replica: &Replica, prev: Option<&Cut>, next: &Cut) -> Result<Vec<EntryId>, CutError> {
    if !next.is_well_formed() {
        return Err(CutError::Malformed);
    }
    let expected = prev.map_or(0, |p| p.number + 1);
    if next.number != expected {
        return Err(CutError::NotNext { expected });
    }
    if prev.is_some_and(|p| next.epoch < p.epoch || (p.closes && next.epoch == p.epoch)) {
        return Err(CutError::Epoch);
    }
    let mut segments = Vec::new();
    for m in &next.marks {
        let chain = replica
            .chain(&m.writer, m.len, m.head)
            .ok_or(CutError::Missing { writer: m.writer })?;
        let from = prev
            .and_then(|p| p.marks.iter().find(|o| o.writer == m.writer))
            .map_or(Ok(0), |o| {
                let through = o
                    .len
                    .checked_sub(1)
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| chain.get(i))
                    .is_some_and(|e| e.entry.hash == o.head);
                if through {
                    Ok(o.len)
                } else {
                    Err(CutError::Retracts { writer: m.writer })
                }
            })?;
        let rank = tagged(
            "isegoria/cut/order/v1",
            &[&prev.map_or([0; 32], Cut::digest), &m.writer],
        );
        let ids: Vec<EntryId> = chain.iter().skip(from as usize).map(|e| e.id()).collect();
        segments.push((rank, ids));
    }
    if let Some(p) = prev {
        if let Some(gone) = p
            .marks
            .iter()
            .find(|o| next.marks.iter().all(|m| m.writer != o.writer))
        {
            return Err(CutError::Retracts {
                writer: gone.writer,
            });
        }
    }
    segments.sort();
    let rounds = segments.iter().map(|(_, ids)| ids.len()).max().unwrap_or(0);
    Ok((0..rounds)
        .flat_map(|i| {
            segments
                .iter()
                .filter_map(move |(_, ids)| ids.get(i).copied())
        })
        .collect())
}

const MEMBER_TAG: u8 = 0xC0;

/// A consortium member's own message, carried as an object on its feed (`docs/04`
/// §Members' objects).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemberObject {
    CutSignature {
        cut: Cut,
        member: usize,
        signature: Signature,
    },
    Commit(BeaconCommit),
    Reveal {
        epoch: u64,
        reveal: BeaconReveal,
    },
}

impl MemberObject {
    /// The member whose feed alone may carry this object.
    pub fn member(&self) -> usize {
        match self {
            MemberObject::CutSignature { member, .. } => *member,
            MemberObject::Commit(c) => c.member,
            MemberObject::Reveal { reveal, .. } => reveal.member,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(MEMBER_TAG);
        match self {
            MemberObject::CutSignature {
                cut,
                member,
                signature,
            } => {
                w.u8(1)
                    .field(&cut.encode())
                    .u64(*member as u64)
                    .fixed(&signature.to_bytes());
            }
            MemberObject::Commit(c) => {
                w.u8(2)
                    .fixed(&c.round.network_id)
                    .fixed(&c.round.member_set_hash)
                    .u64(c.round.epoch)
                    .u64(c.member as u64)
                    .fixed(&c.commitment)
                    .fixed(&c.signature.to_bytes());
            }
            MemberObject::Reveal { epoch, reveal } => {
                w.u8(3)
                    .u64(*epoch)
                    .u64(reveal.member as u64)
                    .fixed(&reveal.secret);
            }
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != MEMBER_TAG {
            return None;
        }
        let index = |r: &mut Reader| usize::try_from(r.u64().ok()?).ok();
        let object = match r.u8().ok()? {
            1 => MemberObject::CutSignature {
                cut: Cut::decode(r.field().ok()?)?,
                member: index(&mut r)?,
                signature: Signature::from_bytes(&r.fixed().ok()?),
            },
            2 => MemberObject::Commit(BeaconCommit {
                round: RoundId {
                    network_id: r.fixed().ok()?,
                    member_set_hash: r.fixed().ok()?,
                    epoch: r.u64().ok()?,
                },
                member: index(&mut r)?,
                commitment: r.fixed().ok()?,
                signature: Signature::from_bytes(&r.fixed().ok()?),
            }),
            3 => MemberObject::Reveal {
                epoch: r.u64().ok()?,
                reveal: BeaconReveal {
                    member: index(&mut r)?,
                    secret: r.fixed().ok()?,
                },
            },
            _ => return None,
        };
        r.finish().ok()?;
        Some(object)
    }
}

/// The member whose turn it is to propose cut `number`.
pub fn proposer(consortium: &Consortium, number: u64) -> usize {
    (number % consortium.keys().len() as u64) as usize
}

/// A member's signature over `cut`, as the object it publishes.
pub fn sign_cut(
    member: &Member,
    index: usize,
    network_id: [u8; 32],
    consortium: &Consortium,
    cut: &Cut,
) -> MemberObject {
    MemberObject::CutSignature {
        cut: cut.clone(),
        member: index,
        signature: member.sign(&cut.checkpoint(network_id, consortium.member_set_hash())),
    }
}

/// The member objects in `replica`, each on the feed of the member it names.
pub fn member_objects<'a>(
    replica: &'a Replica,
    consortium: &'a Consortium,
) -> impl Iterator<Item = (EntryId, MemberObject)> + 'a {
    replica.ids().filter_map(move |id| {
        let (_, bytes) = replica.get(&id)?;
        let object = MemberObject::decode(bytes)?;
        let key = consortium.keys().get(object.member())?;
        (key.as_bytes() == &id.writer).then_some((id, object))
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Collected {
    /// Fewer than `t` members signed any one cut with this number.
    Pending,
    Ready(Cut, Vec<(usize, Signature)>),
    /// Two cuts with one number, each signed by `t` members: the consortium equivocated.
    Conflict(Cut, Cut),
}

/// The cut numbered `number` that `t` members signed on their feeds.
pub fn collect(
    replica: &Replica,
    consortium: &Consortium,
    network_id: [u8; 32],
    number: u64,
) -> Collected {
    let mut by_digest: BTreeMap<[u8; 32], (Cut, BTreeMap<usize, Signature>)> = BTreeMap::new();
    for (_, object) in member_objects(replica, consortium) {
        let MemberObject::CutSignature {
            cut,
            member,
            signature,
        } = object
        else {
            continue;
        };
        let cp = cut.checkpoint(network_id, consortium.member_set_hash());
        if cut.number != number || !consortium.signed_by(&cp, member, &signature) {
            continue;
        }
        by_digest
            .entry(cut.digest())
            .or_insert_with(|| (cut, BTreeMap::new()))
            .1
            .insert(member, signature);
    }
    let mut ready = by_digest
        .into_values()
        .filter(|(_, sigs)| sigs.len() >= consortium.threshold());
    match (ready.next(), ready.next()) {
        (None, _) => Collected::Pending,
        (Some((cut, sigs)), None) => Collected::Ready(cut, sigs.into_iter().collect()),
        (Some((a, _)), Some((b, _))) => Collected::Conflict(a, b),
    }
}

/// Whether member `index` co-signs `cut` after `last` (`docs/04` §Proposing and signing):
/// the proposer signed it, it extends `last`, every entry is held, and it counts the
/// member's own commit (closing cut) or reveal (the cut after) if published.
pub fn should_sign(
    replica: &Replica,
    consortium: &Consortium,
    index: usize,
    last: Option<&Cut>,
    cut: &Cut,
) -> bool {
    let by_proposer = member_objects(replica, consortium).any(|(_, o)| {
        matches!(&o, MemberObject::CutSignature { cut: c, member, .. }
            if c == cut && *member == proposer(consortium, cut.number))
    });
    if !by_proposer || added(replica, last, cut).is_err() {
        return false;
    }
    let Some(key) = consortium.keys().get(index) else {
        return false;
    };
    let own = key.to_bytes();
    let counted = |seq: u64| cut.marks.iter().any(|m| m.writer == own && m.len > seq);
    let reveal_of = last.filter(|l| l.closes).map(|l| l.epoch);
    member_objects(replica, consortium).all(|(id, o)| {
        let needed = match &o {
            MemberObject::Commit(c) => cut.closes && c.round.epoch == cut.epoch,
            MemberObject::Reveal { epoch, .. } => reveal_of == Some(*epoch),
            MemberObject::CutSignature { .. } => false,
        };
        id.writer != own || !needed || counted(id.seq)
    })
}
