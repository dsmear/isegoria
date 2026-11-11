//! Issuance from the prepared signatures (`docs/03` T19-P5): a participant's request for one
//! listed slot, each named member's signature share, and the participant's combination.

use super::*;
use ark_ff::Zero;

const REQUEST_VERSION: u8 = 1;
const SHARE_VERSION: u8 = 1;

/// The label of slot `slot`: its number's four bytes, little-endian, then zeros (invariant 1).
pub fn slot_label(slot: u32) -> Label {
    let mut label = [0u8; 32];
    label[..4].copy_from_slice(&slot.to_le_bytes());
    Label(label)
}

/// A participant's request for one slot: the blind-issuance request bound to its label.
#[derive(Clone, Debug)]
pub struct SlotRequest {
    slot: u32,
    pub(super) request: IssuanceRequest,
}

impl SlotRequest {
    /// The request for `slot` under `issuer`, and what the participant keeps to finish it.
    pub fn new(
        credential: &Credential,
        slot: u32,
        issuer: &IssuerPublic,
    ) -> (Self, PendingIssuance) {
        let (request, pending) = credential.request_issuance(&slot_label(slot), issuer);
        (SlotRequest { slot, request }, pending)
    }

    pub fn slot(&self) -> u32 {
        self.slot
    }

    /// Version 1, the slot (4 bytes), then the commitment, the proof's `t` and its response.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![REQUEST_VERSION];
        out.extend_from_slice(&self.slot.to_le_bytes());
        put(&mut out, &self.request.commitment);
        put(&mut out, &self.request.t);
        put(&mut out, &self.request.response);
        out
    }

    /// `None` unless `bytes` are exactly one encoding; the proof is not checked here.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader(bytes);
        if r.u8()? != REQUEST_VERSION {
            return None;
        }
        let slot = r.u32()?;
        let request = IssuanceRequest {
            commitment: r.take()?,
            label: slot_label(slot),
            t: r.take()?,
            response: r.take()?,
        };
        r.end()?;
        Some(SlotRequest { slot, request })
    }
}

/// One named member's share of the signature for one slot.
#[derive(Clone, Debug, PartialEq)]
pub struct SignatureShare {
    slot: u32,
    share: BBSPlusSignatureShare<E>,
}

impl SignatureShare {
    pub fn slot(&self) -> u32 {
        self.slot
    }

    /// The member index of the share's signer.
    pub fn member(&self) -> u16 {
        self.share.id - 1
    }

    /// Version 1, the slot (4 bytes), then the library's share.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = vec![SHARE_VERSION];
        out.extend_from_slice(&self.slot.to_le_bytes());
        put(&mut out, &self.share);
        out
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader(bytes);
        if r.u8()? != SHARE_VERSION {
            return None;
        }
        let slot = r.u32()?;
        let share: BBSPlusSignatureShare<E> = r.take()?;
        r.end()?;
        (share.id >= 1).then_some(SignatureShare { slot, share })
    }
}

/// Why a member does not answer a request; none of them spends the slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnswerError {
    /// This member holds no prepared signature: not a named signer, or not prepared yet.
    NotPrepared,
    NotListed,
    /// The request's proof of knowledge does not verify.
    Proof,
    Library,
}

impl Party {
    /// This member's share for `request`'s slot from its prepared signature, the same for the same
    /// request; the caller records the slot and the share before it leaves (`docs/03` T19-P5).
    pub fn answer(&self, request: &SlotRequest) -> Result<SignatureShare, AnswerError> {
        let (phase1, phase2) = self.prepared().ok_or(AnswerError::NotPrepared)?;
        let index = self
            .setup
            .slots
            .binary_search(&request.slot)
            .map_err(|_| AnswerError::NotListed)?;
        let params = params();
        verify_request(&params, &request.request).map_err(|_| AnswerError::Proof)?;
        let label = label_scalar(&request.request.label);
        let share = BBSPlusSignatureShare::new_with_committed_messages(
            &request.request.commitment,
            BTreeMap::from([(IDX_LABEL, &label)]),
            index,
            phase1,
            phase2,
            &params,
        )
        .map_err(|_| AnswerError::Library)?;
        Ok(SignatureShare {
            slot: request.slot,
            share,
        })
    }
}

/// Why a participant's shares make no credential.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombineError {
    /// No share, or two from one member.
    Shares,
    /// The shares are for different slots, or not the request's.
    Slot,
    /// They aggregate to no signature that verifies under the issuer's key.
    Signature,
}

/// The credential `shares` of one slot by distinct members make, verified under `issuer`.
pub fn combine(
    pending: PendingIssuance,
    shares: &[SignatureShare],
    issuer: &IssuerPublic,
) -> Result<AnonymousCredential, CombineError> {
    let slot = shares.first().ok_or(CombineError::Shares)?.slot;
    let ids: BTreeSet<ParticipantId> = shares.iter().map(|s| s.share.id).collect();
    if ids.len() != shares.len() {
        return Err(CombineError::Shares);
    }
    let same = shares.iter().all(|s| s.slot == slot);
    if !same || label_scalar(&slot_label(slot)) != pending.label_scalar {
        return Err(CombineError::Slot);
    }
    let sum = shares.iter().fold(Fr::zero(), |sum, s| sum + s.share.u);
    if sum.is_zero() {
        return Err(CombineError::Signature);
    }
    let parts = shares.iter().map(|s| s.share.clone()).collect();
    let signature = BBSPlusSignatureShare::aggregate(parts).map_err(|_| CombineError::Signature)?;
    let credential = pending.finalize(BlindSignature(signature));
    if credential.verify(issuer) {
        Ok(credential)
    } else {
        Err(CombineError::Signature)
    }
}
