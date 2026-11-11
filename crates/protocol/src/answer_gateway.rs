//! Gateway admission (`docs/05` §Distributed integration, D60): whether a member's gateway logs
//! a person's bytes, decided without state; cut order alone decides what the state accepts.

use crate::admission::{admit as proven, Unproven};
use crate::answer::Malformed;
use crate::answer_events::AnswerEvent;
use crate::answer_ledger::fits;
use crate::answer_path::{commit_context, network_context, POOL_CONTEXT};
use crate::deposit::deposit_context;
use identity::credential::IssuerPublic;
use identity::nym::Role;

/// Why a gateway does not log bytes a person sent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inadmissible {
    NotAnEvent,
    /// `CloseCommits`, `Close` or `RequestCloseEpoch`: members' requests, never a person's.
    MemberOnly,
    /// `Draw` or `CloseEpoch`: derived at each cut's end, never logged.
    Derived,
    /// A deposit whose question encodes past `answer_ledger::MAX_QUESTION`.
    TooLarge,
    Malformed(Malformed),
    Unproven(Unproven),
}

/// `Ok` for a person's `JoinPool`, `Deposit` or `Commit` proved under `issuer` in network
/// `network_id`'s context, a deposit's question well formed and within the bound, or a `Reveal`.
pub fn admit(
    event: &[u8],
    network_id: &[u8; 32],
    issuer: &IssuerPublic,
) -> Result<(), Inadmissible> {
    let event = AnswerEvent::decode(event).ok_or(Inadmissible::NotAnEvent)?;
    let context = |offline: &[u8]| network_context(network_id, offline);
    let (proof, role, context) = match &event {
        AnswerEvent::JoinPool { proof } => (proof, Role::Judge, context(POOL_CONTEXT)),
        AnswerEvent::Deposit {
            epoch,
            question,
            proof,
        } => {
            if !fits(question) {
                return Err(Inadmissible::TooLarge);
            }
            question.check().map_err(Inadmissible::Malformed)?;
            let offline = deposit_context(question.content_id(), *epoch);
            (proof, Role::Propose, context(&offline))
        }
        AnswerEvent::Commit {
            question,
            commitment,
            proof,
        } => (
            proof,
            Role::Judge,
            context(&commit_context(*question, commitment)),
        ),
        AnswerEvent::Reveal { .. } => return Ok(()),
        AnswerEvent::CloseCommits { .. }
        | AnswerEvent::Close { .. }
        | AnswerEvent::RequestCloseEpoch { .. } => return Err(Inadmissible::MemberOnly),
        AnswerEvent::Draw { .. } | AnswerEvent::CloseEpoch { .. } => {
            return Err(Inadmissible::Derived)
        }
    };
    proven(proof, issuer, role, &context)
        .map(|_| ())
        .map_err(Inadmissible::Unproven)
}
