//! The state a member's node computes from its collected cuts, kept outside `p2p` (`docs/05`
//! §Where the state is computed, `docs/01` D60), and the spool of its operator's requests.

use crate::member::Collector;
use ed25519_dalek::Signature;
use network::cut::Cut;
use network::replica::Replica;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// The most bytes a spooled request may hold; a larger file is left in place, unread.
pub const MAX_SPOOLED: u64 = 4 << 10;

/// The protocol state a member's node derives from the cuts it collects (`docs/05` D60).
pub trait Hook: Send {
    /// Applies the next collected cut, its `signatures` given in `view`: `Ok(Some(d))` reports
    /// the state digest `d` after a closing cut; an error, a name without spaces, stops progress.
    fn apply(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        view: u64,
        signatures: &[(usize, Signature)],
    ) -> Result<Option<[u8; 32]>, String>;

    /// Whether the member may sign `cut`, proposed after the last cut applied.
    fn may_sign(&self, cut: &Cut) -> bool;

    /// Whether the cut after the last one applied closes its epoch.
    fn closes(&self) -> bool;

    /// Whether the member's gateway logs a person's `event` (`docs/05` *Gateway admission*); by
    /// default nothing.
    fn admit(&self, _event: &[u8]) -> bool {
        false
    }

    /// The sealed reply to a person's read `request`, from the cuts applied and the node's
    /// `replica`; `None` is a refusal, the default (`docs/05` T79-P7).
    fn read(&self, _replica: &Replica, _request: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

/// A member's hook and the directory its operator's requests are spooled in.
pub struct Hooked {
    pub hook: Box<dyn Hook>,
    pub spool: PathBuf,
}

/// How far a hook has followed a node's collected cuts: the digests it reported, by cut number,
/// and the cut it refused, after which it applies nothing more.
#[derive(Clone, Debug, Default)]
pub struct Follower {
    applied: usize,
    digests: Vec<(u64, [u8; 32])>,
    refused: Option<(u64, String)>,
}

impl Follower {
    /// Applies to `hook` each cut `collector` holds past those applied, until one is refused.
    pub fn follow(&mut self, hook: &mut dyn Hook, collector: &Collector, replica: &Replica) {
        while self.refused.is_none() {
            let Some((cut, view, signatures)) = collector.signed(self.applied) else {
                return;
            };
            match hook.apply(replica, cut, view, signatures) {
                Ok(digest) => {
                    self.digests.extend(digest.map(|d| (cut.number, d)));
                    self.applied += 1;
                }
                Err(reason) => self.refused = Some((cut.number, reason)),
            }
        }
    }

    /// The cuts applied, from number 0.
    pub fn applied(&self) -> usize {
        self.applied
    }

    pub fn digests(&self) -> &[(u64, [u8; 32])] {
        &self.digests
    }

    pub fn refused(&self) -> Option<&(u64, String)> {
        self.refused.as_ref()
    }
}

/// The requests waiting in `spool`, by name: regular files whose name starts with no `.`.
pub fn pending(spool: &Path) -> io::Result<Vec<(String, Vec<u8>)>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(spool) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if name.starts_with('.') || !entry.file_type()?.is_file() {
            continue;
        }
        let mut bytes = Vec::new();
        let file = std::fs::File::open(entry.path())?;
        file.take(MAX_SPOOLED + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 <= MAX_SPOOLED {
            out.push((name, bytes));
        }
    }
    out.sort();
    Ok(out)
}

/// Moves request `name` to `spool/done/`, where [`executed`] lists it.
pub fn done(spool: &Path, name: &str) -> io::Result<()> {
    let dir = spool.join("done");
    std::fs::create_dir_all(&dir)?;
    std::fs::rename(spool.join(name), dir.join(name))
}

/// The executed requests by name, each with its bytes.
pub fn executed(spool: &Path) -> io::Result<Vec<(String, Vec<u8>)>> {
    pending(&spool.join("done"))
}
