//! T19's first step, a spike: the library's key generation, pairwise base OT and prepared
//! signatures for four members, threshold 3, each message carried in-process as its bytes
//! (P4's files stand in; nothing signed or sealed). `docs/03` §Committee setup for the trial.

use super::*;
use crate::nullifier;
use ark_bls12_381::G2Affine;
use bbs_plus::threshold::multiplication_phase::Phase2Output;
use bbs_plus::threshold::threshold_bbs_plus::Phase1Output;
use secret_sharing_and_dkg::common::{PublicKeyBase, Share};
use secret_sharing_and_dkg::frost_dkg::Round1State;

const N: u16 = 4;
const T: u16 = 3;
const SIGNERS: [u16; 3] = [1, 3, 4];
const SLOTS: u32 = 2;
const PARTICIPANT: u16 = 0;
const SPIKE_ID: &[u8] = b"isegoria/spike/t19/v1";

type Log = Vec<(&'static str, u16, u16)>;
type Prepared = BTreeMap<u16, (Phase1Output<Fr>, Phase2Output<Fr>)>;

fn params() -> SignatureParamsG1<E> {
    SignatureParamsG1::<E>::new::<Sha256>(PARAMS_LABEL, MSG_COUNT)
}

fn carry<M: CanonicalSerialize + CanonicalDeserialize>(
    log: &mut Log,
    round: &'static str,
    from: u16,
    to: u16,
    message: &M,
) -> M {
    log.push((round, from, to));
    let mut bytes = Vec::new();
    message.serialize_compressed(&mut bytes).unwrap();
    let mut rest = &bytes[..];
    let decoded = M::deserialize_compressed(&mut rest).unwrap();
    assert!(rest.is_empty());
    decoded
}

fn others(all: &[u16], me: u16) -> BTreeSet<ParticipantId> {
    all.iter().copied().filter(|&j| j != me).collect()
}

fn keygen(log: &mut Log, ids: &[u16]) -> BTreeMap<u16, (Share<Fr>, G2Affine)> {
    let g2 = params().g2;
    let mut round1 = BTreeMap::new();
    let mut sent = BTreeMap::new();
    for &i in ids {
        let (state, msg) = Round1State::<G2Affine>::start_with_random_secret::<_, Blake2b512>(
            &mut OsRng, i, T, N, SPIKE_ID, &g2,
        )
        .unwrap();
        round1.insert(i, state);
        sent.insert(i, msg);
    }
    for &i in ids {
        for j in others(ids, i) {
            let msg = carry(log, "dkg-1", j, i, &sent[&j]);
            let state = round1.get_mut(&i).unwrap();
            state
                .add_received_message::<Blake2b512>(msg, SPIKE_ID, &g2)
                .unwrap();
        }
    }
    let (mut round2, mut dealt) = (BTreeMap::new(), BTreeMap::new());
    for (i, state) in round1 {
        let (next, shares) = state.finish().unwrap();
        round2.insert(i, next);
        dealt.insert(i, shares);
    }
    for &i in ids {
        for j in others(ids, i) {
            let share = carry(log, "dkg-2", j, i, &dealt[&j].0[i as usize - 1]);
            round2
                .get_mut(&i)
                .unwrap()
                .add_received_share(j, share, &g2)
                .unwrap();
        }
    }
    let mut out = BTreeMap::new();
    for (i, state) in round2 {
        let (share, _, key) = state.finish(&g2).unwrap();
        out.insert(i, (share, key));
    }
    out
}

fn base_ot(log: &mut Log) -> BTreeMap<u16, BaseOTOutput> {
    let ids: Vec<u16> = (1..=N).collect();
    let b = PublicKeyBase::<G1Affine>::new::<Blake2b512>(b"isegoria/spike/t19/base-ot").0;
    let count = Ote {}.num_base_ot();
    let mut party = BTreeMap::new();
    let mut pks = BTreeMap::new();
    for &i in &ids {
        let (p, pk) =
            Participant::init::<_, Blake2b512>(&mut OsRng, i, others(&ids, i), count, &b).unwrap();
        party.insert(i, p);
        pks.insert(i, pk);
    }
    let pairs: Vec<(u16, u16)> = (1..=N)
        .flat_map(|s| (s + 1..=N).map(move |r| (s, r)))
        .collect();
    let mut flight = BTreeMap::new();
    for &(s, r) in &pairs {
        let pk = carry(log, "ot-1", s, r, &pks[&s][&r]);
        let p = party.get_mut(&r).unwrap();
        let reply = p.receive_sender_pubkey::<_, Blake2b512, Shake256, BASE_OT_KEY_SIZE>(
            &mut OsRng, s, pk, &b,
        );
        flight.insert((s, r), reply.unwrap());
    }
    let mut challenges = BTreeMap::new();
    for &(s, r) in &pairs {
        let pk = carry(log, "ot-2", r, s, &flight[&(s, r)]);
        let p = party.get_mut(&s).unwrap();
        let chal = p.receive_receiver_pubkey::<Blake2b512, Shake256, BASE_OT_KEY_SIZE>(r, pk);
        challenges.insert((s, r), chal.unwrap());
    }
    let mut responses = BTreeMap::new();
    for &(s, r) in &pairs {
        let chal = carry(log, "ot-3", s, r, &challenges[&(s, r)]);
        let p = party.get_mut(&r).unwrap();
        responses.insert((s, r), p.receive_challenges::<Blake2b512>(s, chal).unwrap());
    }
    let mut hashed = BTreeMap::new();
    for &(s, r) in &pairs {
        let resp = carry(log, "ot-4", r, s, &responses[&(s, r)]);
        let p = party.get_mut(&s).unwrap();
        hashed.insert((s, r), p.receive_responses(r, resp).unwrap());
    }
    for &(s, r) in &pairs {
        let keys = carry(log, "ot-5", s, r, &hashed[&(s, r)]);
        let p = party.get_mut(&r).unwrap();
        p.receive_hashed_keys::<Blake2b512>(s, keys).unwrap();
    }
    party.into_iter().map(|(i, p)| (i, p.finish())).collect()
}

fn prepare(
    log: &mut Log,
    shares: &BTreeMap<u16, (Share<Fr>, G2Affine)>,
    ots: &BTreeMap<u16, BaseOTOutput>,
) -> Prepared {
    let mut round1 = BTreeMap::new();
    let (mut comms, mut zeros) = (BTreeMap::new(), BTreeMap::new());
    for &i in &SIGNERS {
        let (state, comm, zero) = Phase1::<Fr, 256>::init_for_bbs_plus::<_, Blake2b512>(
            &mut OsRng,
            SLOTS,
            i,
            others(&SIGNERS, i),
            SPIKE_ID.to_vec(),
        )
        .unwrap();
        round1.insert(i, state);
        comms.insert(i, comm);
        zeros.insert(i, zero);
    }
    for &i in &SIGNERS {
        for j in others(&SIGNERS, i) {
            let (c, z) = carry(
                log,
                "sign-1",
                j,
                i,
                &(comms[&j].clone(), zeros[&j][&i].clone()),
            );
            round1
                .get_mut(&i)
                .unwrap()
                .receive_commitment(j, c, z)
                .unwrap();
        }
    }
    for &i in &SIGNERS {
        for j in others(&SIGNERS, i) {
            let sender = &round1[&j];
            let reveal = (
                sender.get_comm_shares_and_salts(),
                sender.get_comm_shares_and_salts_for_zero_sharing_protocol_with_other(&i),
            );
            let (c, z) = carry(log, "sign-2", j, i, &reveal);
            let state = round1.get_mut(&i).unwrap();
            state.receive_shares::<Blake2b512>(j, c, z).unwrap();
        }
    }
    let phase1: BTreeMap<u16, Phase1Output<Fr>> = round1
        .into_iter()
        .map(|(i, s)| {
            (
                i,
                s.finish_for_bbs_plus::<Blake2b512>(&shares[&i].0.share)
                    .unwrap(),
            )
        })
        .collect();

    let ote = Ote {};
    let gadget = Gadget::new::<Blake2b512>(ote, b"isegoria/bbs+/gadget/v1");
    let (mut round2, mut first) = (BTreeMap::new(), Vec::new());
    for &i in &SIGNERS {
        let (state, msgs) = Phase2::init::<_, Shake256>(
            &mut OsRng,
            i,
            phase1[&i].masked_signing_key_shares.clone(),
            phase1[&i].masked_rs.clone(),
            ots[&i].clone(),
            others(&SIGNERS, i),
            ote,
            &gadget,
        )
        .unwrap();
        round2.insert(i, state);
        first.extend(msgs.into_iter().map(|(to, m)| (i, to, m)));
    }
    let mut second = Vec::new();
    for (from, to, m) in first {
        let m = carry(log, "mul-1", from, to, &m);
        let state = round2.get_mut(&to).unwrap();
        let reply = state.receive_message1::<Blake2b512, Shake256>(from, m, &gadget);
        second.push((to, from, reply.unwrap()));
    }
    for (from, to, m) in second {
        let m = carry(log, "mul-2", from, to, &m);
        let state = round2.get_mut(&to).unwrap();
        state
            .receive_message2::<Blake2b512>(from, m, &gadget)
            .unwrap();
    }
    let finish = |(i, s): (u16, Phase2<Fr, KAPPA, STAT>)| (i, (phase1[&i].clone(), s.finish()));
    round2.into_iter().map(finish).collect()
}

fn slot_label(slot: u32) -> Label {
    let mut label = [0u8; 32];
    label[..4].copy_from_slice(&slot.to_le_bytes());
    Label(label)
}

fn issue(
    log: &mut Log,
    prepared: &Prepared,
    signers: &[u16],
    slot: u32,
    issuer: &IssuerPublic,
) -> AnonymousCredential {
    let mut secret = [0u8; 32];
    rand_core::RngCore::fill_bytes(&mut OsRng, &mut secret);
    let holder = Credential::from_secret(secret);
    let (request, pending) = holder.request_issuance(&slot_label(slot), issuer);
    let label = label_scalar(&request.label);
    let mut shares = Vec::new();
    for &i in signers {
        log.push(("issue-request", PARTICIPANT, i));
        verify_request(&params(), &request).unwrap();
        let uncommitted = BTreeMap::from([(IDX_LABEL, &label)]);
        let (p1, p2) = &prepared[&i];
        let share = BBSPlusSignatureShare::new_with_committed_messages(
            &request.commitment,
            uncommitted,
            slot as usize,
            p1,
            p2,
            &params(),
        )
        .unwrap();
        shares.push(carry(log, "issue-share", i, PARTICIPANT, &share));
    }
    let signature = BBSPlusSignatureShare::aggregate(shares).unwrap();
    pending.finalize(BlindSignature(signature))
}

struct Ceremony {
    log: Log,
    keys: BTreeMap<u16, (Share<Fr>, G2Affine)>,
    prepared: Prepared,
}

fn ceremony() -> Ceremony {
    let mut log = Vec::new();
    let keys = keygen(&mut log, &[1, 2, 3, 4]);
    let ots = base_ot(&mut log);
    let prepared = prepare(&mut log, &keys, &ots);
    Ceremony {
        log,
        keys,
        prepared,
    }
}

// The four agree on a key the genesis decodes; 3 prepared shares issue under it (T19-P2, P5).
#[test]
fn the_ceremonys_key_is_a_genesis_issuer_key() {
    let mut c = ceremony();
    let key = c.keys[&1].1;
    assert!(c.keys.values().all(|(_, k)| *k == key));
    let mut bytes = vec![ISSUER_ENCODING];
    key.serialize_compressed(&mut bytes).unwrap();
    let issuer = IssuerPublic::decode(&bytes).expect("the genesis's decoder accepts it");
    assert_eq!(issuer.encode(), bytes);
    assert_eq!(issuer.public_key().0, key);
    let single = Issuer::new([1; 32]).public();
    for slot in 0..SLOTS {
        let credential = issue(&mut c.log, &c.prepared, &SIGNERS, slot, &issuer);
        assert!(credential.verify(&issuer));
        assert!(!credential.verify(&single));
        let proof = nullifier::prove(&credential, &issuer, Role::Judge, b"ctx");
        assert!(nullifier::verify(&proof, &issuer, b"ctx"));
        assert!(!nullifier::verify(&proof, &single, b"ctx"));
    }
    let two = issue(&mut c.log, &c.prepared, &SIGNERS[..2], 0, &issuer);
    assert!(!two.verify(&issuer));
}

// The library's flights between members, in order, with their messages (T19-P2, P3).
#[test]
fn the_ceremony_takes_eleven_rounds_between_members() {
    let log = ceremony().log;
    let mut rounds: Vec<(&str, usize)> = Vec::new();
    for (round, _, _) in &log {
        match rounds.last_mut() {
            Some((r, n)) if r == round => *n += 1,
            _ => rounds.push((round, 1)),
        }
    }
    let expected = [
        ("dkg-1", 12),
        ("dkg-2", 12),
        ("ot-1", 6),
        ("ot-2", 6),
        ("ot-3", 6),
        ("ot-4", 6),
        ("ot-5", 6),
        ("sign-1", 6),
        ("sign-2", 6),
        ("mul-1", 3),
        ("mul-2", 3),
    ];
    assert_eq!(rounds, expected);
    let lower_first = |r: &str| matches!(r, "ot-1" | "ot-3" | "ot-5" | "mul-1");
    for (round, from, to) in &log {
        if round.starts_with("ot-") || round.starts_with("mul-") {
            assert_eq!(from < to, lower_first(round), "{round} {from}->{to}");
        }
    }
    let signing = log.iter().position(|(r, _, _)| *r == "sign-1").unwrap();
    assert!(log[signing..]
        .iter()
        .all(|(_, from, to)| *from != 2 && *to != 2));
}

// The library finishes a key generation missing a member's contribution (T19-P2).
#[test]
fn keygen_without_a_member_still_finishes() {
    let keys = keygen(&mut Vec::new(), &[1, 2, 3]);
    let key = keys[&1].1;
    assert!(keys.values().all(|(_, k)| *k == key));
    let mut bytes = vec![ISSUER_ENCODING];
    key.serialize_compressed(&mut bytes).unwrap();
    assert!(IssuerPublic::decode(&bytes).is_some());
}
