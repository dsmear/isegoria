//! T19's ceremony for one member (`docs/03` §Committee setup for the trial): the libraries' key
//! generation, pairwise base OT and prepared signatures, one flight of payloads per call, and the
//! member's state between calls. A **bespoke** composition, unreviewed (`docs/CLAUDE.md`).

use super::*;
use ark_bls12_381::G2Affine;
use bbs_plus::threshold::multiplication_phase::Phase2Output;
use bbs_plus::threshold::threshold_bbs_plus::Phase1Output;
use oblivious_transfer_protocols::base_ot::simplest_ot::{
    Challenges, HashedKey, ReceiverPubKeys, Responses,
};
use oblivious_transfer_protocols::cointoss::Commitments;
use oblivious_transfer_protocols::ot_based_multiplication::base_ot_multi_party_pairwise::SenderPubKeyAndProof;
use oblivious_transfer_protocols::ot_based_multiplication::batch_mul_multi_party::{
    Message1, Message2,
};
use secret_sharing_and_dkg::common::{PublicKeyBase, Share};
use secret_sharing_and_dkg::frost_dkg::{Round1Msg, Round1State, Round2State};
use sha2::Digest as _;

mod issuance;

pub use issuance::{combine, slot_label, AnswerError, CombineError, SignatureShare, SlotRequest};

/// The ceremony's flights: 1 the fresh values, 2–12 the libraries', 13 the confirmation.
pub const FLIGHTS: u8 = 13;
const STATE_VERSION: u8 = 1;
const SALT: usize = 256;
const BASE_OT_LABEL: &[u8] = b"isegoria/ceremony/base-ot/v1";
const GADGET_LABEL: &[u8] = b"isegoria/bbs+/gadget/v1";
const KEYGEN_ECHO: &[u8] = b"isegoria/ceremony/echo/keygen/v1";
const PHASE1_ECHO: &[u8] = b"isegoria/ceremony/echo/phase1/v1";

type Commit = Phase1<Fr, SALT>;
type Mul = Phase2<Fr, KAPPA, STAT>;
type Ot = Participant<G1Affine>;
type Reveal = Vec<(Fr, [u8; SALT])>;
type Prepared = (Phase1Output<Fr>, Phase2Output<Fr>);

fn params() -> SignatureParamsG1<E> {
    SignatureParamsG1::<E>::new::<Sha256>(PARAMS_LABEL, MSG_COUNT)
}

fn base_ot_generator() -> G1Affine {
    PublicKeyBase::<G1Affine>::new::<Blake2b512>(BASE_OT_LABEL).0
}

fn gadget() -> Gadget {
    Gadget::new::<Blake2b512>(Ote {}, GADGET_LABEL)
}

/// The libraries' participant id of member index `i`.
fn pid(i: u16) -> ParticipantId {
    i + 1
}

fn encoded<T: CanonicalSerialize>(value: &T) -> Vec<u8> {
    let mut bytes = Vec::new();
    value
        .serialize_compressed(&mut bytes)
        .expect("serializing into memory cannot fail");
    bytes
}

/// The `T` that `bytes` are exactly the one compressed, validated encoding of.
fn strict<T: CanonicalSerialize + CanonicalDeserialize>(bytes: &[u8]) -> Option<T> {
    let mut rest = bytes;
    let value = T::deserialize_compressed(&mut rest).ok()?;
    (rest.is_empty() && encoded(&value) == bytes).then_some(value)
}

/// SHA-256 over the tag's length (8 bytes), the tag, then `value`'s compressed encoding.
fn echo_digest<T: CanonicalSerialize>(tag: &[u8], value: &T) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update((tag.len() as u64).to_le_bytes());
    h.update(tag);
    h.update(encoded(value));
    h.finalize().into()
}

fn put<T: CanonicalSerialize>(out: &mut Vec<u8>, value: &T) {
    let bytes = encoded(value);
    out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    out.extend_from_slice(&bytes);
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if n > self.0.len() {
            return None;
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Some(head)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.bytes(1)?[0])
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.bytes(4)?.try_into().ok()?))
    }

    fn take<T: CanonicalSerialize + CanonicalDeserialize>(&mut self) -> Option<T> {
        let len = u64::from_le_bytes(self.bytes(8)?.try_into().ok()?);
        strict(self.bytes(usize::try_from(len).ok()?)?)
    }

    fn end(self) -> Option<()> {
        self.0.is_empty().then_some(())
    }
}

/// The committee's shape: `members` in genesis order, `threshold`, the named signers and the
/// listed slots (`docs/03` T19-P1, P2, P5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setup {
    members: u16,
    threshold: u16,
    signers: Vec<u16>,
    slots: Vec<u32>,
}

impl Setup {
    /// `None` unless `2 <= threshold <= members <= 255`, `signers` are `threshold` member
    /// indices in increasing order, and `slots` a non-empty increasing list.
    pub fn new(members: u16, threshold: u16, signers: Vec<u16>, slots: Vec<u32>) -> Option<Self> {
        let increasing = |v: &[u32]| v.windows(2).all(|w| w[0] < w[1]);
        let signer_slots: Vec<u32> = signers.iter().map(|&s| u32::from(s)).collect();
        let ok = (2..=members).contains(&threshold)
            && members <= 255
            && signers.len() == usize::from(threshold)
            && increasing(&signer_slots)
            && signers.iter().all(|&s| s < members)
            && !slots.is_empty()
            && u32::try_from(slots.len()).is_ok()
            && increasing(&slots);
        ok.then_some(Setup {
            members,
            threshold,
            signers,
            slots,
        })
    }

    pub fn members(&self) -> u16 {
        self.members
    }

    pub fn threshold(&self) -> u16 {
        self.threshold
    }

    pub fn signers(&self) -> &[u16] {
        &self.signers
    }

    pub fn slots(&self) -> &[u32] {
        &self.slots
    }

    pub fn is_signer(&self, member: u16) -> bool {
        self.signers.contains(&member)
    }

    /// Whether member `from` sends member `to` a message in `flight` (`docs/03`, First step).
    pub fn sends(&self, flight: u8, from: u16, to: u16) -> bool {
        let signing = self.is_signer(from) && self.is_signer(to);
        from != to
            && from < self.members
            && to < self.members
            && match flight {
                1..=3 | 13 => true,
                4 | 6 | 8 => from < to,
                5 | 7 => from > to,
                9 | 10 => signing,
                11 => signing && from < to,
                12 => signing && from > to,
                _ => false,
            }
    }

    /// The members whose message member `me` needs in `flight`, in member order.
    pub fn senders(&self, flight: u8, me: u16) -> Vec<u16> {
        (0..self.members)
            .filter(|&j| self.sends(flight, j, me))
            .collect()
    }

    pub fn recipients(&self, flight: u8, me: u16) -> Vec<u16> {
        (0..self.members)
            .filter(|&j| self.sends(flight, me, j))
            .collect()
    }

    fn batch(&self) -> u32 {
        self.slots.len() as u32
    }

    fn ids(&self, members: impl Iterator<Item = u16>, me: u16) -> BTreeSet<ParticipantId> {
        members.filter(|&j| j != me).map(pid).collect()
    }
}

/// Why a flight is not taken: only [`Refusal::Missing`] leaves the ceremony able to go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Member `j`'s payload is missing: nothing changed, the call can be made again with it.
    Missing(u16),
    /// Member `j`'s payload was not expected, does not decode, or fails the library's check.
    Unexpected(u16),
    Decode(u16),
    Library(u16),
    /// The generated key is not a genesis issuer key, or an own step of the library failed.
    Key,
    Internal,
    Finished,
}

#[derive(Clone)]
struct KeyShare {
    share: Share<Fr>,
    key: G2Affine,
}

#[derive(Clone)]
enum Stage {
    Start,
    Contributed(Round1State<G2Affine>),
    Shared(Round2State<G2Affine>),
    Ot(KeyShare, Ot),
    Signing(KeyShare, BaseOTOutput, Commit, Commitments),
    Multiplying(KeyShare, BaseOTOutput, Phase1Output<Fr>, Mul),
    Waiting(KeyShare, BaseOTOutput),
    Prepared(KeyShare, BaseOTOutput, Option<Prepared>),
}

/// One member's private state: its key share, pairwise OT state and part of each prepared
/// signature, never another member's share.
#[derive(Clone)]
pub struct Party {
    setup: Setup,
    me: u16,
    id: [u8; 32],
    next: u8,
    stage: Stage,
    /// What the libraries assume broadcast, as received: key generation's digest, phase 1's.
    echo: Vec<[u8; 32]>,
}

impl fmt::Debug for Party {
    /// Redacted: the share and the libraries' secret state (PV-4, docs/08 §8.3).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Party")
            .field("me", &self.me)
            .field("next", &self.next)
            .finish_non_exhaustive()
    }
}

/// The outgoing payloads of a flight: recipient, bytes.
pub type Outgoing = Vec<(u16, Vec<u8>)>;

impl Party {
    /// Member `me` of `setup` in the ceremony `id`, before its flight 2.
    pub fn new(setup: Setup, me: u16, id: [u8; 32]) -> Option<Self> {
        (me < setup.members).then_some(Party {
            setup,
            me,
            id,
            next: 2,
            stage: Stage::Start,
            echo: Vec::new(),
        })
    }

    pub fn setup(&self) -> &Setup {
        &self.setup
    }

    pub fn me(&self) -> u16 {
        self.me
    }

    pub fn id(&self) -> [u8; 32] {
        self.id
    }

    /// The flight the next [`Party::advance`] sends; `FLIGHTS + 1` once prepared.
    pub fn next(&self) -> u8 {
        self.next
    }

    /// The committee's key once key generation has finished, in the genesis's form.
    pub fn issuer(&self) -> Option<IssuerPublic> {
        match &self.stage {
            Stage::Ot(k, _)
            | Stage::Signing(k, _, _, _)
            | Stage::Multiplying(k, _, _, _)
            | Stage::Waiting(k, _)
            | Stage::Prepared(k, _, _) => issuer_of(&k.key),
            _ => None,
        }
    }

    pub fn is_prepared(&self) -> bool {
        matches!(self.stage, Stage::Prepared(..))
    }

    /// The echo the confirmation carries (`docs/03`, *Files*): key generation's digest, then a
    /// signer's phase-1 digest, 32 bytes each, once the flights computing them are taken.
    pub fn echo(&self) -> Vec<u8> {
        self.echo.concat()
    }

    /// Whether `theirs`, member `j`'s echo, is this member's: key generation's digest, and phase
    /// 1's where both members sign; a non-signer's echo holds 32 bytes, a signer's 64.
    pub fn agrees(&self, j: u16, theirs: &[u8]) -> bool {
        let signer = self.setup.is_signer(j);
        let both = signer && self.setup.is_signer(self.me);
        theirs.len() == 32 * (1 + usize::from(signer))
            && self.echo.first().is_some_and(|k| theirs[..32] == k[..])
            && (!both || self.echo.get(1).is_some_and(|p| theirs[32..] == p[..]))
    }

    /// Takes the payloads of flight `next − 1`, by sender, and returns the party after flight
    /// `next` with that flight's payloads; flight 1 is the caller's, so flight 2 takes none.
    pub fn advance(&self, incoming: &BTreeMap<u16, Vec<u8>>) -> Result<(Party, Outgoing), Refusal> {
        if self.next > FLIGHTS {
            return Err(Refusal::Finished);
        }
        let from = match self.next {
            2 => Vec::new(),
            n => self.setup.senders(n - 1, self.me),
        };
        if let Some(&j) = from.iter().find(|j| !incoming.contains_key(j)) {
            return Err(Refusal::Missing(j));
        }
        if let Some(&j) = incoming.keys().find(|j| !from.contains(j)) {
            return Err(Refusal::Unexpected(j));
        }
        let mut echo = self.echo.clone();
        let (stage, out) = self.flight(incoming, &mut echo)?;
        let to = self.setup.recipients(self.next, self.me);
        let sent: Vec<u16> = out.iter().map(|(j, _)| *j).collect();
        if self.next < FLIGHTS && sent != to {
            return Err(Refusal::Internal);
        }
        let party = Party {
            next: self.next + 1,
            stage,
            echo,
            ..self.clone()
        };
        Ok((party, out))
    }

    fn flight(
        &self,
        incoming: &BTreeMap<u16, Vec<u8>>,
        echo: &mut Vec<[u8; 32]>,
    ) -> Result<(Stage, Outgoing), Refusal> {
        let s = &self.setup;
        let (me, t, n) = (pid(self.me), s.threshold, s.members);
        let g2 = params().g2;
        let to = s.recipients(self.next, self.me);
        let all = || s.ids(0..n, self.me);
        let cosigners = || s.ids(s.signers.iter().copied(), self.me);
        let mut out = Vec::new();
        let stage = match (self.next, self.stage.clone()) {
            (2, Stage::Start) => {
                let (state, msg) =
                    Round1State::<G2Affine>::start_with_random_secret::<_, Blake2b512>(
                        &mut OsRng, me, t, n, &self.id, &g2,
                    )
                    .map_err(internal)?;
                let bytes = encoded(&msg);
                out = to.iter().map(|&j| (j, bytes.clone())).collect();
                Stage::Contributed(state)
            }
            (3, Stage::Contributed(mut state)) => {
                each(incoming, |j, msg: Round1Msg<G2Affine>| {
                    if msg.sender_id != pid(j) || msg.comm_coeffs.0.len() != usize::from(t) {
                        return Err(());
                    }
                    state
                        .add_received_message::<Blake2b512>(msg, &self.id, &g2)
                        .map_err(|_| ())
                })?;
                if state.coeff_comms.len() != usize::from(n) {
                    return Err(Refusal::Internal);
                }
                echo.push(echo_digest(KEYGEN_ECHO, &state.coeff_comms));
                let (next, shares) = state.finish().map_err(internal)?;
                for &j in &to {
                    let share = shares.0.get(usize::from(j)).filter(|x| x.id == pid(j));
                    out.push((j, encoded(share.ok_or(Refusal::Internal)?)));
                }
                Stage::Shared(next)
            }
            (4, Stage::Shared(mut state)) => {
                each(incoming, |j, share: Share<Fr>| {
                    state.add_received_share(pid(j), share, &g2).map_err(|_| ())
                })?;
                if state.shares.len() != usize::from(n) {
                    return Err(Refusal::Internal);
                }
                let (share, _, key) = state.finish(&g2).map_err(internal)?;
                issuer_of(&key).ok_or(Refusal::Key)?;
                let count = Ote {}.num_base_ot();
                let generator = base_ot_generator();
                let (ot, keys) =
                    Ot::init::<_, Blake2b512>(&mut OsRng, me, all(), count, &generator)
                        .map_err(internal)?;
                for &j in &to {
                    out.push((j, encoded(keys.get(&pid(j)).ok_or(Refusal::Internal)?)));
                }
                Stage::Ot(KeyShare { share, key }, ot)
            }
            (5, Stage::Ot(k, mut ot)) => {
                let generator = base_ot_generator();
                each(incoming, |j, key: SenderPubKeyAndProof<G1Affine>| {
                    let reply = ot
                        .receive_sender_pubkey::<_, Blake2b512, Shake256, BASE_OT_KEY_SIZE>(
                            &mut OsRng,
                            pid(j),
                            key,
                            &generator,
                        );
                    reply.map(|r| out.push((j, encoded(&r)))).map_err(|_| ())
                })?;
                Stage::Ot(k, ot)
            }
            (6, Stage::Ot(k, mut ot)) => {
                each(incoming, |j, keys: ReceiverPubKeys<G1Affine>| {
                    let reply = ot
                        .receive_receiver_pubkey::<Blake2b512, Shake256, BASE_OT_KEY_SIZE>(
                            pid(j),
                            keys,
                        );
                    reply.map(|r| out.push((j, encoded(&r)))).map_err(|_| ())
                })?;
                Stage::Ot(k, ot)
            }
            (7, Stage::Ot(k, mut ot)) => {
                each(incoming, |j, challenges: Challenges| {
                    let reply = ot.receive_challenges::<Blake2b512>(pid(j), challenges);
                    reply.map(|r| out.push((j, encoded(&r)))).map_err(|_| ())
                })?;
                Stage::Ot(k, ot)
            }
            (8, Stage::Ot(k, mut ot)) => {
                each(incoming, |j, responses: Responses| {
                    let reply = ot.receive_responses(pid(j), responses);
                    reply.map(|r| out.push((j, encoded(&r)))).map_err(|_| ())
                })?;
                Stage::Ot(k, ot)
            }
            (9, Stage::Ot(k, mut ot)) => {
                each(incoming, |j, keys: Vec<(HashedKey, HashedKey)>| {
                    ot.receive_hashed_keys::<Blake2b512>(pid(j), keys)
                        .map_err(|_| ())
                })?;
                let (lower, higher) = (self.me, n - 1 - self.me);
                let complete = ot.receiver_keys.len() == usize::from(lower)
                    && ot.sender_keys.len() == usize::from(higher)
                    && ot.sender_challenger.is_empty()
                    && ot.receiver_responder.is_empty();
                if !complete {
                    return Err(Refusal::Internal);
                }
                let base = ot.finish();
                if !s.is_signer(self.me) {
                    Stage::Waiting(k, base)
                } else {
                    let (commit, comm, zero) = Commit::init_for_bbs_plus::<_, Blake2b512>(
                        &mut OsRng,
                        s.batch(),
                        me,
                        cosigners(),
                        self.id.to_vec(),
                    )
                    .map_err(internal)?;
                    for &j in &to {
                        let zero = zero.get(&pid(j)).ok_or(Refusal::Internal)?;
                        out.push((j, encoded(&(comm.clone(), zero.clone()))));
                    }
                    Stage::Signing(k, base, commit, comm)
                }
            }
            (10, Stage::Signing(k, base, mut commit, own)) => {
                each(incoming, |j, (comm, zero): (Commitments, Commitments)| {
                    commit
                        .receive_commitment(pid(j), comm, zero)
                        .map_err(|_| ())
                })?;
                let mut all = commit.commitment_protocol.other_commitments.clone();
                all.insert(me, own.clone());
                if all.len() != usize::from(t) {
                    return Err(Refusal::Internal);
                }
                echo.push(echo_digest(PHASE1_ECHO, &all));
                for &j in &to {
                    let shares = commit.get_comm_shares_and_salts();
                    let zero = commit
                        .get_comm_shares_and_salts_for_zero_sharing_protocol_with_other(&pid(j));
                    out.push((j, encoded(&(shares, zero))));
                }
                Stage::Signing(k, base, commit, own)
            }
            (11, Stage::Signing(k, base, mut commit, _)) => {
                each(incoming, |j, (shares, zero): (Reveal, Reveal)| {
                    commit
                        .receive_shares::<Blake2b512>(pid(j), shares, zero)
                        .map_err(|_| ())
                })?;
                if !commit.ready_to_compute_randomness_and_arguments_for_multiplication() {
                    return Err(Refusal::Internal);
                }
                let phase1 = commit
                    .finish_for_bbs_plus::<Blake2b512>(&k.share.share)
                    .map_err(internal)?;
                if !phase1_complete(&phase1, s.batch(), &cosigners()) {
                    return Err(Refusal::Internal);
                }
                let (mul, messages) = Mul::init::<_, Shake256>(
                    &mut OsRng,
                    me,
                    phase1.masked_signing_key_shares.clone(),
                    phase1.masked_rs.clone(),
                    base.clone(),
                    cosigners(),
                    Ote {},
                    &gadget(),
                )
                .map_err(internal)?;
                for &j in &to {
                    out.push((j, encoded(messages.get(&pid(j)).ok_or(Refusal::Internal)?)));
                }
                Stage::Multiplying(k, base, phase1, mul)
            }
            (12, Stage::Multiplying(k, base, phase1, mut mul)) => {
                let gadget = gadget();
                each(incoming, |j, m: Message1<Fr>| {
                    let reply = mul.receive_message1::<Blake2b512, Shake256>(pid(j), m, &gadget);
                    reply.map(|r| out.push((j, encoded(&r)))).map_err(|_| ())
                })?;
                Stage::Multiplying(k, base, phase1, mul)
            }
            (13, Stage::Multiplying(k, base, phase1, mut mul)) => {
                let gadget = gadget();
                each(incoming, |j, m: Message2<Fr>| {
                    mul.receive_message2::<Blake2b512>(pid(j), m, &gadget)
                        .map_err(|_| ())
                })?;
                let phase2 = mul.finish();
                if !phase2_complete(&phase2, s, self.me) {
                    return Err(Refusal::Internal);
                }
                Stage::Prepared(k, base, Some((phase1, phase2)))
            }
            (10..=12, Stage::Waiting(k, base)) => Stage::Waiting(k, base),
            (13, Stage::Waiting(k, base)) => Stage::Prepared(k, base, None),
            _ => return Err(Refusal::Internal),
        };
        Ok((stage, out))
    }

    /// Version 1, the next flight, the member, the id, the setup, the stage's objects, then the
    /// echo's count (1 byte) and digests.
    pub fn encode(&self) -> Vec<u8> {
        let s = &self.setup;
        let mut out = vec![STATE_VERSION, self.next];
        out.extend_from_slice(&self.me.to_le_bytes());
        out.extend_from_slice(&self.id);
        put(
            &mut out,
            &(s.members, s.threshold, s.signers.clone(), s.slots.clone()),
        );
        let key = |out: &mut Vec<u8>, k: &KeyShare| {
            put(out, &k.share);
            put(out, &k.key);
        };
        match &self.stage {
            Stage::Start => out.push(0),
            Stage::Contributed(state) => {
                out.push(1);
                put(&mut out, state);
            }
            Stage::Shared(state) => {
                out.push(2);
                put(&mut out, state);
            }
            Stage::Ot(k, ot) => {
                out.push(3);
                key(&mut out, k);
                put(&mut out, ot);
            }
            Stage::Signing(k, base, commit, own) => {
                out.push(4);
                key(&mut out, k);
                put(&mut out, base);
                put(&mut out, commit);
                put(&mut out, own);
            }
            Stage::Multiplying(k, base, phase1, mul) => {
                out.push(5);
                key(&mut out, k);
                put(&mut out, base);
                put(&mut out, phase1);
                put(&mut out, mul);
            }
            Stage::Waiting(k, base) => {
                out.push(6);
                key(&mut out, k);
                put(&mut out, base);
            }
            Stage::Prepared(k, base, prepared) => {
                out.push(if prepared.is_some() { 8 } else { 7 });
                key(&mut out, k);
                put(&mut out, base);
                if let Some(prepared) = prepared {
                    put(&mut out, prepared);
                }
            }
        }
        out.push(self.echo.len() as u8);
        for digest in &self.echo {
            out.extend_from_slice(digest);
        }
        out
    }

    /// `None` unless `bytes` are exactly one encoding of a state some flight can reach.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader(bytes);
        if r.u8()? != STATE_VERSION {
            return None;
        }
        let next = r.u8()?;
        let me = u16::from_le_bytes(r.bytes(2)?.try_into().ok()?);
        let id: [u8; 32] = r.bytes(32)?.try_into().ok()?;
        let (members, threshold, signers, slots): (u16, u16, Vec<u16>, Vec<u32>) = r.take()?;
        let setup = Setup::new(members, threshold, signers, slots)?;
        let key = |r: &mut Reader| {
            Some(KeyShare {
                share: r.take()?,
                key: r.take()?,
            })
        };
        let signer = setup.is_signer(me);
        let stage = match (r.u8()?, next) {
            (0, 2) => Stage::Start,
            (1, 3) => Stage::Contributed(r.take()?),
            (2, 4) => Stage::Shared(r.take()?),
            (3, 5..=9) => Stage::Ot(key(&mut r)?, r.take()?),
            (4, 10 | 11) if signer => Stage::Signing(key(&mut r)?, r.take()?, r.take()?, r.take()?),
            (5, 12 | 13) if signer => {
                Stage::Multiplying(key(&mut r)?, r.take()?, r.take()?, r.take()?)
            }
            (6, 10..=13) if !signer => Stage::Waiting(key(&mut r)?, r.take()?),
            (7, 14) if !signer => Stage::Prepared(key(&mut r)?, r.take()?, None),
            (8, 14) if signer => Stage::Prepared(key(&mut r)?, r.take()?, Some(r.take()?)),
            _ => return None,
        };
        let held = usize::from(next >= 4) + usize::from(signer && next >= 11);
        let mut echo = Vec::new();
        for _ in 0..r.u8()? {
            echo.push(r.bytes(32)?.try_into().ok()?);
        }
        r.end()?;
        if echo.len() != held {
            return None;
        }
        let party = Party::new(setup, me, id).map(|p| Party {
            next,
            stage,
            echo,
            ..p
        })?;
        (party.encode() == bytes).then_some(party)
    }

    fn prepared(&self) -> Option<&Prepared> {
        match &self.stage {
            Stage::Prepared(_, _, prepared) => prepared.as_ref(),
            _ => None,
        }
    }
}

/// Decodes each sender's payload as a `T` and hands it to `apply`, in member order.
fn each<T: CanonicalSerialize + CanonicalDeserialize>(
    incoming: &BTreeMap<u16, Vec<u8>>,
    mut apply: impl FnMut(u16, T) -> Result<(), ()>,
) -> Result<(), Refusal> {
    for (&j, bytes) in incoming {
        let value = strict(bytes).ok_or(Refusal::Decode(j))?;
        apply(j, value).map_err(|_| Refusal::Library(j))?;
    }
    Ok(())
}

fn phase1_complete(p: &Phase1Output<Fr>, batch: u32, others: &BTreeSet<ParticipantId>) -> bool {
    let len = batch as usize;
    p.batch_size == batch
        && [&p.r, &p.e, &p.s, &p.masked_signing_key_shares, &p.masked_rs]
            .iter()
            .all(|v| v.len() == len)
        && p.others.iter().copied().collect::<BTreeSet<_>>() == *others
        && p.others.len() == others.len()
}

fn phase2_complete(p: &Phase2Output<Fr>, s: &Setup, me: u16) -> bool {
    let len = s.batch() as usize;
    let ids = |below: bool| -> Vec<ParticipantId> {
        let side = s.signers.iter().filter(|&&j| (j < me) == below && j != me);
        side.map(|&j| pid(j)).collect()
    };
    let full = |m: &BTreeMap<ParticipantId, (Vec<Fr>, Vec<Fr>)>| {
        m.values().all(|(a, b)| a.len() == len && b.len() == len)
    };
    p.0.z_A.keys().copied().collect::<Vec<_>>() == ids(true)
        && p.0.z_B.keys().copied().collect::<Vec<_>>() == ids(false)
        && full(&p.0.z_A)
        && full(&p.0.z_B)
}

fn internal<X>(_: X) -> Refusal {
    Refusal::Internal
}

fn issuer_of(key: &G2Affine) -> Option<IssuerPublic> {
    let mut bytes = vec![ISSUER_ENCODING];
    key.serialize_compressed(&mut bytes).ok()?;
    IssuerPublic::decode(&bytes)
}

#[cfg(test)]
mod tests;
