//! The answer review in a member's node (`docs/05` §Where the state is computed, `docs/01`
//! D60): the hook applying each collected cut to a ledger, admitting and answering persons for
//! the gateway, and the operators' closure requests.

use crate::genesis::Document;
use crate::{tiered, Config, ConfigError};
use ed25519_dalek::Signature;
use network::cut::Cut;
use network::replica::{EntryId, Replica};
use p2p::hook::Hook;
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{AnswerLedger, AnswerLedgerError, Refusal};
use protocol::answer_path::{Refused, Round};
use protocol::{answer_gateway, answer_read};
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

    fn admit(&self, event: &[u8]) -> bool {
        let network = self.ledger.tiers().network_id();
        answer_gateway::admit(event, &network, self.ledger.issuer()).is_ok()
    }

    fn read(&self, replica: &Replica, request: &[u8]) -> Option<Vec<u8>> {
        answer_read::answer(&self.ledger, replica, request)
    }

    fn status(&self, replica: &Replica, requests: &[EntryId]) -> Vec<String> {
        let path = self.ledger.state();
        let mut lines = vec![format!("epoch {}", path.epoch())];
        for q in 0..path.questions() {
            if let Some(entry) = path.entry(q) {
                lines.push(format!("question {q} {}", phase(&entry.round)));
            }
        }
        for id in requests {
            let refused = self.ledger.refused().iter().find(|(r, _)| r == id);
            let outcome = match refused {
                _ if !answer_read::counted(&self.ledger, replica, id) => "pending".to_string(),
                Some((_, refusal)) => format!("refused {}", refusal_name(refusal)),
                None => "counted".to_string(),
            };
            lines.push(format!("request_state {} {outcome}", id.seq));
        }
        lines
    }
}

const PHASES: [&str; 5] = [
    "pending",
    "unavailable",
    "committing",
    "revealing",
    "closed",
];

/// A question's round in the status.
fn phase(round: &Round) -> &'static str {
    PHASES[match round {
        Round::Pending => 0,
        Round::Unavailable { .. } => 1,
        Round::Committing { .. } => 2,
        Round::Revealing { .. } => 3,
        Round::Closed(_) => 4,
    }]
}

/// A refused entry's reason in the status, without its details.
fn refusal_name(refusal: &Refusal) -> &'static str {
    match refusal {
        Refusal::NotAnEvent => "not-an-event",
        Refusal::NotAuthorized => "not-authorized",
        Refusal::PublicFeed => "public-feed",
        Refusal::Derived => "derived",
        Refusal::WrongEpoch => "wrong-epoch",
        Refusal::Repeated => "repeated",
        Refusal::TooLarge => "too-large",
        Refusal::OutOfWindow => "out-of-window",
        Refusal::Beacon(_) => "beacon",
        Refusal::Path(refused) => match refused {
            Refused::WrongEpoch => "wrong-event-epoch",
            Refused::Malformed(_) => "malformed",
            Refused::Unproven(_) => "unproven",
            Refused::AlreadyInPool => "already-in-pool",
            Refused::Duplicate { .. } => "duplicate",
            Refused::OverQuota { .. } => "over-quota",
            Refused::UnknownQuestion => "unknown-question",
            Refused::BeaconPending => "beacon-pending",
            Refused::WrongRound => "wrong-round",
            Refused::NotDrawn => "not-drawn",
            Refused::AlreadyCommitted => "already-committed",
            Refused::NoCommit => "no-commit",
            Refused::AlreadyRevealed => "already-revealed",
            Refused::RevealMismatch => "reveal-mismatch",
            Refused::AnswerOutOfRange => "answer-out-of-range",
            Refused::Overflow => "overflow",
            Refused::NetworkOnly => "network-only",
        },
    }
}

/// Whether `line` is one of the status lines [`AnswerHook`] writes (`docs/05` §Built in the node).
pub fn status_line(line: &str) -> bool {
    let n = |t: &str| t.parse::<u64>().is_ok_and(|v| v.to_string() == t);
    let word = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_lowercase() || b == b'-');
    match line.split(' ').collect::<Vec<_>>()[..] {
        ["epoch", e] => n(e),
        ["question", q, phase] => n(q) && PHASES.contains(&phase),
        ["request_state", seq, "pending" | "counted"] => n(seq),
        ["request_state", seq, "refused", reason] => n(seq) && word(reason),
        _ => false,
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
        crate::sync_dir(&config.data_dir).map_err(read)?;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let label = args.join("-");
    let name = (0..)
        .map(|i| format!("{:020}-{label}", nanos + i))
        .find(|name| !dir.join(name).exists() && !dir.join("done").join(name).exists())
        .ok_or(ConfigError::Request)?;
    crate::production::publish_file(&dir.join(&name), &event.encode()).map_err(read)?;
    Ok(name)
}
