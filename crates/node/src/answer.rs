//! The answer review in a member's node (`docs/05` §Where the state is computed, `docs/01`
//! D60): the hook applying each collected cut to a ledger, and the operators' closure requests.

use crate::genesis::Document;
use crate::{tiered, Config, ConfigError};
use ed25519_dalek::Signature;
use network::cut::Cut;
use network::replica::Replica;
use p2p::hook::Hook;
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{AnswerLedger, AnswerLedgerError};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The ledger of one network, rebuilt from cut 0 at each start, never stored.
pub struct AnswerHook {
    ledger: AnswerLedger,
}

impl AnswerHook {
    /// A fresh ledger under `document`'s tiers, issuer and configuration.
    pub fn new(document: &Document) -> Option<Self> {
        let tiers = document.genesis.tiers().ok()?;
        let config = document.answer_config;
        let ledger = AnswerLedger::new(tiers, document.issuer.clone(), config)?;
        Some(AnswerHook { ledger })
    }

    pub fn ledger(&self) -> &AnswerLedger {
        &self.ledger
    }
}

/// A refused cut's name in the status.
fn reason(error: AnswerLedgerError) -> String {
    match error {
        AnswerLedgerError::Replica => "replica",
        AnswerLedgerError::OutOfSequence { .. } => "out-of-sequence",
        AnswerLedgerError::Unsigned => "unsigned",
        AnswerLedgerError::Checks(_) => "checks",
        AnswerLedgerError::Cut(_) => "cut",
    }
    .to_string()
}

impl Hook for AnswerHook {
    fn apply(
        &mut self,
        replica: &Replica,
        cut: &Cut,
        view: u64,
        signatures: &[(usize, Signature)],
    ) -> Result<Option<[u8; 32]>, String> {
        let applied = self.ledger.apply_in_view(replica, cut, view, signatures);
        applied.map_err(reason)?;
        Ok(cut.closes.then(|| self.ledger.digest()))
    }

    fn may_sign(&self, cut: &Cut) -> bool {
        self.ledger.check(cut).is_ok()
    }

    fn closes(&self) -> bool {
        self.ledger.next().is_some_and(|next| next.closes)
    }
}

/// The event an operator command names: `close-commits <question>`, `close <question>` or
/// `close-epoch <epoch>` (`docs/05` T79-P5).
pub fn event(args: &[String]) -> Option<AnswerEvent> {
    let [kind, number] = args else {
        return None;
    };
    let digits = !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit());
    let n: u64 = number.parse().ok().filter(|_| digits)?;
    match kind.as_str() {
        "close-commits" => Some(AnswerEvent::CloseCommits { question: n }),
        "close" => Some(AnswerEvent::Close { question: n }),
        "close-epoch" => Some(AnswerEvent::RequestCloseEpoch { epoch: n }),
        _ => None,
    }
}

/// The spool a member's node reads its operator's requests from.
pub fn spool(data_dir: &Path) -> PathBuf {
    data_dir.join("requests")
}

/// Spools the request `args` names for the running member `config` configures, returning its
/// file's name; writes nothing else, and refuses a node that is no tiered member.
pub fn request(config: &Config, args: &[String]) -> Result<String, ConfigError> {
    let event = event(args).ok_or(ConfigError::Request)?;
    tiered(config)?.ok_or(ConfigError::Genesis(crate::genesis::Error::Missing))?;
    if config.member.is_none() || config.key_file.is_none() {
        return Err(ConfigError::MemberWithoutKey);
    }
    let read = |e: std::io::Error| ConfigError::Read(e.kind());
    let dir = spool(&config.data_dir);
    if !dir.is_dir() {
        std::fs::create_dir(&dir).map_err(read)?;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let label = args.join("-");
    let name = (0..)
        .map(|i| format!("{:020}-{label}", nanos + i))
        .find(|name| !dir.join(name).exists() && !dir.join("done").join(name).exists())
        .ok_or(ConfigError::Request)?;
    let staged = dir.join(format!(".{name}"));
    std::fs::write(&staged, event.encode()).map_err(read)?;
    std::fs::rename(&staged, dir.join(&name)).map_err(read)?;
    Ok(name)
}
