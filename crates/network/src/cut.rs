//! Cuts (`docs/04` §Cuts, `docs/08` §10.3, T74): the consortium-signed marks that fix which
//! replicated entries count and in what order they are applied.

use crate::codec::{Reader, Writer};
use crate::consortium::Checkpoint;
use crate::hash::tagged;
use crate::replica::{EntryId, Replica};

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
}

impl Cut {
    /// A cut marking every writer's feed in `replica` as it stands.
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
        Cut { number, marks }
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
            &[&self.number.to_le_bytes(), &w.finish()],
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
            .field(&marks.finish())
            .finish()
    }

    pub fn decode(bytes: &[u8]) -> Option<Cut> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != 1 {
            return None;
        }
        let number = r.u64().ok()?;
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
        let cut = Cut { number, marks };
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
