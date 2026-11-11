//! Fuzzing entry points, built only under `cfg(fuzzing)`: the readers of other machines' bytes
//! that the public interface does not reach (`docs/12` §4).

use crate::blind::{self, Answer, Carried, GatewayKey, ReadAnswer, Refusal, Respond};
use crate::hook::Hook;
use crate::{archive, discovery};
use network::replica::Replica;
use network::tier::Tiers;

/// A gateway's decapsulation of a relayed request: the inner request, or the refusal.
pub fn gateway_open(key: &GatewayKey, request: &[u8]) -> Result<Vec<u8>, Refusal> {
    blind::open_request(key, request).map(|(inner, _)| inner)
}

/// An inner request's body, an event's or a read's, or the gateway's answer refusing it.
pub fn gateway_read(inner: &[u8], network_id: &[u8; 32]) -> Result<Vec<u8>, Answer> {
    blind::read_request(inner, network_id).map(|(_, body)| body)
}

/// The Binary HTTP response a gateway seals for `answer`.
pub fn answer_http(answer: Answer) -> Option<Vec<u8>> {
    answer.to_http()
}

/// What OHTTP adds to a gateway's reply when it seals it.
pub const SEAL_OVERHEAD: usize = blind::SEAL_OVERHEAD;

/// The Binary HTTP response a gateway seals for a read's `answer`.
pub fn read_answer_http(answer: &ReadAnswer) -> Option<Vec<u8>> {
    answer.to_http()
}

/// A gateway with `hook` whose circle feed has not caught up: it logs nothing, and answers an
/// admitted event "retry later".
struct Unlogged<'a> {
    hook: &'a dyn Hook,
    replica: &'a Replica,
}

impl Respond for Unlogged<'_> {
    fn read_answer(&mut self, request: &[u8]) -> ReadAnswer {
        blind::read_answer(Some(self.hook), self.replica, request)
    }

    fn event_answer(&mut self, event: Vec<u8>) -> Answer {
        match self.hook.admit(&event) {
            true => Answer::RetryLater,
            false => Answer::NotAdmitted,
        }
    }
}

/// The Binary HTTP response to opened request `inner` of such a gateway on `network_id`.
pub fn gateway_respond(
    inner: &[u8],
    network_id: &[u8; 32],
    hook: &dyn Hook,
    replica: &Replica,
) -> Option<Vec<u8>> {
    blind::respond(inner, network_id, &mut Unlogged { hook, replica })
}

/// Such a gateway's reply to a listed relay's `request`, as the node carries it back.
pub fn gateway_answer(
    key: &GatewayKey,
    request: &[u8],
    network_id: &[u8; 32],
    hook: &dyn Hook,
    replica: &Replica,
) -> Carried {
    match blind::open_request(key, request) {
        Err(refusal) => Carried::Refused(refusal),
        Ok((inner, response)) => {
            let reply = gateway_respond(&inner, network_id, hook, replica);
            blind::carried(response, reply)
        }
    }
}

pub fn serve_object(tiers: &Tiers, replica: &Replica, request: &[u8]) -> Vec<u8> {
    discovery::serve(tiers, replica, request)
}

/// A tiered node's answer to a shard request from an empty archive store held in memory.
pub fn serve_shard(tiers: &Tiers, request: &[u8]) -> Option<Vec<u8>> {
    let store = archive::Store::open(tiers, None).ok()?;
    Some(store.serve(tiers, request))
}
