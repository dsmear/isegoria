//! A person's events under their roles in the network's prefixed contexts, and their sending
//! sealed to a member through a listed relay (`docs/05` T79-P1, T79-P2, T79-P7; `docs/04` T85).

use crate::{decoded, hex, random, read, unhex, write_new, Error, Network, Output, LARGE, SMALL};
use identity::credential::AnonymousCredential;
use identity::nullifier::{prove, NullifierProof};
use identity::nym::{Nym, Role};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use network::cid::{cid, Cid};
use p2p::blind::{seal, submit, Answer, KeyAnnouncement, Sealed};
use protocol::answer::{commit_answer, Answer as Choice, Question};
use protocol::answer_events::AnswerEvent;
use protocol::answer_gateway::admit;
use protocol::answer_path::{commit_context, network_context, POOL_CONTEXT};
use protocol::deposit::deposit_context;
use serde::Deserialize;
use std::path::Path;

type Proved = Result<NullifierProof, Error>;

/// A proof under `role` in `net`'s prefixed `offline` context, `Unverified` for a credential
/// that does not verify under `net`'s issuer key: none is ever proved from.
fn proof(net: &Network, cred: &AnonymousCredential, role: Role, offline: &[u8]) -> Proved {
    if !cred.verify(&net.issuer) {
        return Err(Error::Unverified);
    }
    Ok(prove(
        cred,
        &net.issuer,
        role,
        &network_context(&net.id, offline),
    ))
}

/// The person's one nym under `role`, the id every proof under it carries (invariant 5).
pub fn nym(net: &Network, cred: &AnonymousCredential, role: Role) -> Result<Nym, Error> {
    Ok(proof(net, cred, role, POOL_CONTEXT)?.id())
}

pub fn join(net: &Network, cred: &AnonymousCredential) -> Result<Vec<u8>, Error> {
    let proof = proof(net, cred, Role::Judge, POOL_CONTEXT)?;
    Ok(AnswerEvent::JoinPool { proof }.encode())
}

/// A deposit of `question` in the open `epoch`, proved under `Propose`.
pub fn deposit(
    net: &Network,
    cred: &AnonymousCredential,
    epoch: u64,
    question: Question,
) -> Result<Vec<u8>, Error> {
    let offline = deposit_context(question.content_id(), epoch);
    let proof = proof(net, cred, Role::Propose, &offline)?;
    let event = AnswerEvent::Deposit {
        epoch,
        question,
        proof,
    };
    Ok(event.encode())
}

/// What a reveal opens: the question's number, the answer committed and its nonce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Commitment {
    pub question: u64,
    pub answer: Choice,
    pub nonce: [u8; 32],
}

const COMMITMENT_TAG: &[u8] = b"isegoria/client/commit/v1";

impl Commitment {
    /// Private: the tag, the question (8 bytes, little-endian), the answer's code, the nonce.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = COMMITMENT_TAG.to_vec();
        out.extend_from_slice(&self.question.to_le_bytes());
        out.extend_from_slice(&self.answer.code());
        out.extend_from_slice(&self.nonce);
        out
    }

    /// `None` unless `bytes` are exactly one commitment's encoding.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(COMMITMENT_TAG)?;
        let (question, rest) = rest.split_first_chunk::<8>()?;
        let (code, nonce) = rest.split_first_chunk::<2>()?;
        Some(Commitment {
            question: u64::from_le_bytes(*question),
            answer: Choice::from_code(*code)?,
            nonce: nonce.try_into().ok()?,
        })
    }
}

/// A commit of `answer` to `question` under a fresh nonce, proved under `Judge`.
pub fn commit(
    net: &Network,
    cred: &AnonymousCredential,
    question: u64,
    answer: Choice,
) -> Result<(Vec<u8>, Commitment), Error> {
    let opened = Commitment {
        question,
        answer,
        nonce: random()?,
    };
    let commitment = commit_answer(
        answer,
        &opened.nonce,
        nym(net, cred, Role::Judge)?,
        question,
    );
    let proof = proof(
        net,
        cred,
        Role::Judge,
        &commit_context(question, &commitment),
    )?;
    let event = AnswerEvent::Commit {
        question,
        commitment,
        proof,
    };
    Ok((event.encode(), opened))
}

pub fn reveal(
    net: &Network,
    cred: &AnonymousCredential,
    opened: &Commitment,
) -> Result<Vec<u8>, Error> {
    let event = AnswerEvent::Reveal {
        question: opened.question,
        reviewer: nym(net, cred, Role::Judge)?,
        answer: opened.answer,
        nonce: opened.nonce,
    };
    Ok(event.encode())
}

/// `Ok` for an event a member's gateway on `net` would log (`answer_gateway::admit`).
pub fn admitted(net: &Network, event: &[u8]) -> Result<(), Error> {
    admit(event, &net.id, &net.issuer).map_err(Error::Inadmissible)
}

/// Writes an admitted `event` to `path`, a new 0600 file, with `also`; returns its content id.
pub fn write_event(
    net: &Network,
    event: &[u8],
    path: &Path,
    also: Option<Output>,
) -> Result<Cid, Error> {
    admitted(net, event)?;
    let mut outputs = vec![Output(path, event, true, "event")];
    outputs.extend(also);
    write_new(&outputs)?;
    Ok(cid(event))
}

pub fn read_commitment(path: &Path) -> Result<Commitment, Error> {
    decoded(
        &read(path, SMALL, "commitment")?,
        Commitment::decode,
        "commitment",
    )
}

pub fn read_event(path: &Path) -> Result<Vec<u8>, Error> {
    read(path, LARGE, "event")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuestionFile {
    context: String,
    stem: String,
    alternatives: Vec<String>,
    key: u8,
}

/// A question's TOML file: `context`, `stem`, `alternatives` and `key`, the index of the
/// alternative the author states as correct, nothing else; the texts' UTF-8 bytes.
pub fn question_file(path: &Path) -> Result<Question, Error> {
    let text = String::from_utf8(read(path, LARGE, "question")?);
    let file: QuestionFile = text
        .ok()
        .and_then(|t| toml::from_str(&t).ok())
        .ok_or(Error::Malformed("question"))?;
    Ok(Question {
        context: file.context.into_bytes(),
        stem: file.stem.into_bytes(),
        alternatives: file
            .alternatives
            .into_iter()
            .map(String::into_bytes)
            .collect(),
        key: file.key,
    })
}

/// An answer as typed: an alternative's index `0`–`255`, `none` or `dont-know`.
pub fn answer_of(text: &str) -> Option<Choice> {
    match text {
        "none" => Some(Choice::NoneCorrect),
        "dont-know" => Some(Choice::DontKnow),
        _ => (text.parse::<u8>().ok())
            .filter(|i| i.to_string() == text)
            .map(Choice::Alternative),
    }
}

/// A member's key configuration in hex, as `isegoria-node --announce` prints it, once it
/// verifies under the genesis's tiers with that member's HPKE key.
pub fn announcement(net: &Network, text: &str) -> Result<KeyAnnouncement, Error> {
    let a = unhex(text)
        .and_then(|b| KeyAnnouncement::decode(&b))
        .ok_or(Error::Malformed("announcement"))?;
    let listed = net.genesis.members.get(a.member).map(|m| m.hpke);
    match a.verify(&net.tiers) && a.hpke_key().is_some() && a.hpke_key() == listed {
        true => Ok(a),
        false => Err(Error::Unlisted("announcement")),
    }
}

/// A relay's address, ending in `/p2p/` and the peer id of a transport key the genesis lists.
pub fn relay(net: &Network, text: &str) -> Result<Multiaddr, Error> {
    let address: Multiaddr = text.parse().map_err(|_| Error::Malformed("relay"))?;
    let Some(Protocol::P2p(peer)) = address.iter().last() else {
        return Err(Error::Malformed("relay"));
    };
    let listed = net.genesis.relays.iter().any(|key| {
        libp2p::identity::ed25519::PublicKey::try_from_bytes(key)
            .map(|k| PeerId::from_public_key(&k.into()) == peer)
            .unwrap_or(false)
    });
    match listed {
        true => Ok(address),
        false => Err(Error::Unlisted("relay")),
    }
}

/// `event` sealed to `key`, once it is admissible on `net`.
pub fn sealed(net: &Network, key: &KeyAnnouncement, event: &[u8]) -> Result<Sealed, Error> {
    admitted(net, event)?;
    seal(&net.tiers, key, event).map_err(Error::Channel)
}

/// Sends `event` sealed to `key` through `relay` from a new transport key; the receipt, or
/// `NotLogged` with the gateway's other answer.
pub async fn send(
    net: &Network,
    key: &KeyAnnouncement,
    relay: Multiaddr,
    event: &[u8],
) -> Result<Answer, Error> {
    let sealed = sealed(net, key, event)?;
    let carried = submit(relay, sealed.request.clone())
        .await
        .map_err(Error::Channel)?;
    match sealed.open(&carried).map_err(Error::Channel)? {
        Answer::Logged(id) => Ok(Answer::Logged(id)),
        other => Err(Error::NotLogged(other)),
    }
}

/// A receipt as printed: `logged <writer> <seq> <hash>`, the circle entry in hex.
pub fn render(answer: &Answer) -> String {
    match answer {
        Answer::Logged(id) => format!("logged {} {} {}\n", hex(&id.writer), id.seq, hex(&id.hash)),
        other => format!("{other:?}\n"),
    }
}
