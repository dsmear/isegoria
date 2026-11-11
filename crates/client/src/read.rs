//! A person's private reads (`docs/05` T79-P7, §Built for participants): one request per read
//! under one role with a fresh response key, and the reply opened only by that key.

use crate::{hex, Error, Network};
use identity::credential::AnonymousCredential;
use identity::nym::Role;
use libp2p::Multiaddr;
use network::cid::cid;
use p2p::blind::KeyAnnouncement;
use protocol::answer_events::AnswerEvent;
use protocol::answer_read::{
    Assigned, Authored, EventStatus, Phase, Read, ReadRequest, Reply, ReviewPhase,
};
use protocol::answer_rule::Verdict;
use protocol::answer_seal::ResponseKey;

/// What a person asks: an event's status by its bytes, a reviewer's questions from a number,
/// or an author's records.
pub enum Ask {
    Status(Vec<u8>),
    Reviewer { from: u64 },
    Author,
}

/// The role a read is proved under: an event's own, `Judge` for a reviewer, `Propose` for an
/// author; `None` for bytes that are no event a person sends.
pub fn role(ask: &Ask) -> Option<Role> {
    Some(match ask {
        Ask::Status(bytes) => match AnswerEvent::decode(bytes)? {
            AnswerEvent::JoinPool { .. }
            | AnswerEvent::Commit { .. }
            | AnswerEvent::Reveal { .. } => Role::Judge,
            AnswerEvent::Deposit { .. } => Role::Propose,
            _ => return None,
        },
        Ask::Reviewer { .. } => Role::Judge,
        Ask::Author => Role::Propose,
    })
}

/// What opens one read's reply: its response key and the read it answers.
pub struct Opener {
    key: ResponseKey,
    read: Read,
}

/// The encoded request for `ask`, proved with `cred` under its role on `net` once `cred`
/// verifies there (else `Unverified`), and its opener; each with a new response key.
pub fn request(
    net: &Network,
    cred: &AnonymousCredential,
    ask: &Ask,
) -> Result<(Vec<u8>, Opener), Error> {
    let role = role(ask).ok_or(Error::Malformed("event"))?;
    let read = match ask {
        Ask::Status(bytes) => Read::Status { event: cid(bytes) },
        Ask::Reviewer { from } => Read::Reviewer { from: *from },
        Ask::Author => Read::Author,
    };
    if !cred.verify(&net.issuer) {
        return Err(Error::Unverified);
    }
    let key = ResponseKey::generate();
    let request = ReadRequest::new(read, key.public(), cred, role, &net.issuer, &net.id);
    Ok((request.encode(), Opener { key, read }))
}

impl Opener {
    pub fn response_key(&self) -> [u8; 32] {
        self.key.public()
    }

    /// The reply `sealed` holds, once it opens under this key and answers this read's kind.
    pub fn open(&self, sealed: &[u8]) -> Result<Reply, Error> {
        let reply = (self.key.open(sealed))
            .and_then(|plain| Reply::decode(&plain))
            .ok_or(Error::Reply)?;
        let answers = matches!(
            (&reply, self.read),
            (Reply::Status(_), Read::Status { .. })
                | (Reply::Reviewer(_), Read::Reviewer { .. })
                | (Reply::Author(_), Read::Author)
        );
        answers.then_some(reply).ok_or(Error::Reply)
    }
}

/// NOT WIRED: the read's sealed reply, `request` sealed to `key` and sent through `relay`; it
/// waits for `p2p::blind::{read_path, seal_read}` (`docs/05` T79-P7) and refuses until then.
pub async fn send_read(
    net: &Network,
    key: &KeyAnnouncement,
    relay: Multiaddr,
    request: &[u8],
) -> Result<Vec<u8>, Error> {
    let _ = (net, key, relay, request);
    Err(Error::Unwired("read path not wired yet"))
}

/// One read: its request sent through `relay` to `key`'s member, and the reply opened.
pub async fn read(
    net: &Network,
    cred: &AnonymousCredential,
    key: &KeyAnnouncement,
    relay: Multiaddr,
    ask: &Ask,
) -> Result<Reply, Error> {
    let (request, opener) = request(net, cred, ask)?;
    opener.open(&send_read(net, key, relay, &request).await?)
}

fn text(bytes: &[u8]) -> String {
    format!("{:?}", String::from_utf8_lossy(bytes))
}

fn review_phase(phase: ReviewPhase) -> &'static str {
    match phase {
        ReviewPhase::CommitsOpen => "commits-open",
        ReviewPhase::RevealsOpen => "reveals-open",
        ReviewPhase::Closed => "closed",
    }
}

fn verdict(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::InsufficientParticipation => "insufficient-participation",
        Verdict::Accepted => "accepted",
        Verdict::Discarded => "discarded",
        Verdict::Divided => "divided",
    }
}

fn assigned(a: &Assigned) -> String {
    let mut out = format!(
        "question {}\nphase {}\ncommitted {}\nrevealed {}\ncontext {}\nstem {}\n",
        a.question,
        review_phase(a.phase),
        a.committed,
        a.revealed,
        text(&a.context),
        text(&a.stem)
    );
    for (i, alternative) in a.alternatives.iter().enumerate() {
        out += &format!("alternative {i} {}\n", text(alternative));
    }
    out
}

fn authored(a: &Authored) -> String {
    let mut out = format!("reputation {}\nquota {}\n", a.reputation, a.quota);
    for (q, phase) in &a.questions {
        let phase = match phase {
            Phase::Pending => "pending".to_string(),
            Phase::Unavailable => "unavailable".to_string(),
            Phase::Committing => "committing".to_string(),
            Phase::Revealing => "revealing".to_string(),
            Phase::Closed {
                verdict: v,
                observation: None,
            } => format!("closed {}", verdict(*v)),
            Phase::Closed {
                verdict: v,
                observation: Some(o),
            } => format!(
                "closed {} observation {} {} {} {}",
                verdict(*v),
                if o.success { "success" } else { "failure" },
                o.weight,
                o.origin.epoch,
                o.origin.closings
            ),
        };
        out += &format!("question {q} {phase}\n");
    }
    out
}

/// A reply as printed, one fact per line; texts quoted and escaped, keys in hex.
pub fn render(reply: &Reply) -> String {
    match reply {
        Reply::Status(logged) => {
            let mut out = format!("entries {}\n", logged.len());
            for l in logged {
                let status = match l.status {
                    EventStatus::Pending => "pending".to_string(),
                    EventStatus::Applied { question: None } => "applied".to_string(),
                    EventStatus::Applied { question: Some(q) } => format!("applied question {q}"),
                    EventStatus::Refused(r) => format!("refused {r:?}"),
                };
                out += &format!("entry {} {} {status}\n", hex(&l.writer), l.seq);
            }
            out
        }
        Reply::Reviewer(None) => "question none\n".to_string(),
        Reply::Reviewer(Some(a)) => assigned(a),
        Reply::Author(a) => authored(a),
    }
}
