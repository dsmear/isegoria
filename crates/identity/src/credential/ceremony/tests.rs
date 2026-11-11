use super::*;
use ark_ec::CurveGroup;

const N: u16 = 4;
const T: u16 = 3;
const SIGNERS: [u16; 3] = [0, 2, 3];
const SLOTS: [u32; 2] = [5, 9];
const ID: [u8; 32] = [7; 32];

type Inbox = BTreeMap<u16, BTreeMap<u16, Vec<u8>>>;

fn setup() -> Setup {
    Setup::new(N, T, SIGNERS.to_vec(), SLOTS.to_vec()).unwrap()
}

fn parties(id: [u8; 32]) -> Vec<Party> {
    (0..N)
        .map(|i| Party::new(setup(), i, id).unwrap())
        .collect()
}

fn flight(parties: &mut [Party], inbox: &mut Inbox) -> usize {
    let mut next = Inbox::new();
    let mut count = 0;
    for p in parties.iter_mut() {
        let incoming = inbox.remove(&p.me()).unwrap_or_default();
        let (after, out) = p.advance(&incoming).unwrap();
        *p = Party::decode(&after.encode()).expect("the state round-trips");
        for (to, bytes) in out {
            count += 1;
            next.entry(to).or_default().insert(p.me(), bytes);
        }
    }
    *inbox = next;
    count
}

fn run_to(parties: &mut [Party], last: u8) -> (Inbox, Vec<usize>) {
    let mut inbox = Inbox::new();
    let counts = (2..=last).map(|_| flight(parties, &mut inbox)).collect();
    (inbox, counts)
}

fn prepared() -> Vec<Party> {
    let mut p = parties(ID);
    run_to(&mut p, FLIGHTS);
    p
}

fn issue(parties: &[Party], by: &[u16], slot: u32) -> Result<AnonymousCredential, CombineError> {
    let issuer = parties[0].issuer().unwrap();
    let holder = Credential::from_secret([3; 32]);
    let (request, pending) = SlotRequest::new(&holder, slot, &issuer);
    let request = SlotRequest::decode(&request.encode()).unwrap();
    let shares: Vec<SignatureShare> = by
        .iter()
        .map(|&i| parties[usize::from(i)].answer(&request).unwrap())
        .map(|s| SignatureShare::decode(&s.encode()).unwrap())
        .collect();
    combine(pending, &shares, &issuer)
}

// Four parties, re-decoded at each flight, agree on a genesis key 3 signers issue under (AT-ID-17).
#[test]
fn at_id_17_four_parties_prepare_a_genesis_key_that_issues() {
    let p = prepared();
    let issuer = p[0].issuer().unwrap();
    assert!(p.iter().all(|q| q.is_prepared() && q.next() == FLIGHTS + 1));
    assert!(p
        .iter()
        .all(|q| q.issuer().unwrap().encode() == issuer.encode()));
    assert_eq!(
        IssuerPublic::decode(&issuer.encode()).unwrap().encode(),
        issuer.encode()
    );
    for slot in SLOTS {
        let credential = issue(&p, &SIGNERS, slot).unwrap();
        assert!(credential.verify(&issuer));
        assert!(!credential.verify(&Issuer::new([1; 32]).public()));
    }
    assert_eq!(
        issue(&p, &SIGNERS[..2], 5).unwrap_err(),
        CombineError::Signature
    );
}

// Each flight carries the spike's messages: 11 library flights between the parties (AT-ID-19).
#[test]
fn at_id_19_the_flights_carry_the_spikes_messages() {
    let (_, counts) = run_to(&mut parties(ID), FLIGHTS);
    assert_eq!(counts, [12, 12, 6, 6, 6, 6, 6, 6, 6, 3, 3, 0]);
    let s = setup();
    assert_eq!(s.senders(1, 0), [1, 2, 3]);
    assert_eq!(s.senders(13, 1), [0, 2, 3]);
    assert!((9..=12).all(|f| s.senders(f, 1).is_empty() && s.recipients(f, 1).is_empty()));
}

// A flight lacking an expected payload changes nothing; an unexpected one is refused (AT-ID-19).
#[test]
fn at_id_19_a_flight_needs_every_expected_payload() {
    let mut p = parties(ID);
    let (mut inbox, _) = run_to(&mut p, 2);
    let mut incoming = inbox.remove(&0).unwrap();
    let missing = incoming.remove(&2).unwrap();
    assert_eq!(p[0].advance(&incoming).unwrap_err(), Refusal::Missing(2));
    incoming.insert(2, missing);
    let before = p[0].encode();
    assert!(p[0].advance(&incoming).is_ok());
    assert_eq!(p[0].encode(), before);
    let mut q = parties(ID);
    let (mut later, _) = run_to(&mut q, 4);
    let mut to_three = later.remove(&3).unwrap();
    to_three.insert(3, Vec::new());
    assert_eq!(q[3].advance(&to_three).unwrap_err(), Refusal::Unexpected(3));
    let early = BTreeMap::from([(1, Vec::new())]);
    assert_eq!(q[0].advance(&early).unwrap_err(), Refusal::Unexpected(1));
}

// A payload altered, of another ceremony or claiming another sender is refused (AT-ID-18).
#[test]
fn at_id_18_a_payload_that_fails_is_refused() {
    let mut p = parties(ID);
    let (inbox, _) = run_to(&mut p, 2);
    let honest = inbox[&0].clone();
    for at in [0, 9, honest[&1].len() - 1] {
        let mut bad = honest.clone();
        bad.get_mut(&1).unwrap()[at] ^= 1;
        assert!(matches!(
            p[0].advance(&bad),
            Err(Refusal::Decode(1) | Refusal::Library(1))
        ));
    }
    let mut other = parties([8; 32]);
    let (foreign, _) = run_to(&mut other, 2);
    let mut mixed = honest.clone();
    mixed.insert(1, foreign[&0][&1].clone());
    assert_eq!(p[0].advance(&mixed).unwrap_err(), Refusal::Library(1));
    let mut swapped = honest.clone();
    swapped.insert(1, honest[&2].clone());
    assert_eq!(p[0].advance(&swapped).unwrap_err(), Refusal::Library(1));
    let mut short = honest;
    short.get_mut(&3).unwrap().push(0);
    assert_eq!(p[0].advance(&short).unwrap_err(), Refusal::Decode(3));
}

// A refused share or base-OT reply is refused by sender, later in the ceremony too (AT-ID-18).
#[test]
fn at_id_18_a_later_payload_that_fails_is_refused() {
    for last in [3, 5, 10] {
        let mut p = parties(ID);
        let (inbox, _) = run_to(&mut p, last);
        let (&to, incoming) = inbox.iter().next().unwrap();
        let (&from, bytes) = incoming.iter().next().unwrap();
        let mut bad = incoming.clone();
        let flipped = bytes.len() / 2;
        bad.get_mut(&from).unwrap()[flipped] ^= 0x40;
        let refused = p[usize::from(to)].advance(&bad).unwrap_err();
        assert!(
            refused == Refusal::Decode(from) || refused == Refusal::Library(from),
            "flight {last}: {refused:?}"
        );
    }
}

// A contribution committing to no coefficient, or not to `t`, is refused, never a panic (AT-ID-27).
#[test]
fn at_id_27_a_contribution_of_another_degree_is_refused() {
    let mut p = parties(ID);
    let (inbox, _) = run_to(&mut p, 2);
    for len in [0, 1, usize::from(T) - 1, usize::from(T) + 1] {
        let mut bad = inbox[&0].clone();
        let mut msg: Round1Msg<G2Affine> = strict(&bad[&1]).unwrap();
        msg.comm_coeffs.0.resize(len, msg.comm_coeffs.0[0]);
        bad.insert(1, encoded(&msg));
        assert_eq!(
            p[0].advance(&bad).unwrap_err(),
            Refusal::Library(1),
            "{len}"
        );
    }
}

// Other commitments sent to one member pass the library but not the echoes (AT-ID-28).
#[test]
fn at_id_28_an_equivocated_contribution_shows_in_the_echoes() {
    let honest = prepared();
    for q in &honest {
        assert!((0..N).all(|j| q.agrees(j, &honest[usize::from(j)].echo())));
    }
    let mut p = parties(ID);
    let (mut inbox, _) = run_to(&mut p, 2);
    let delta = Fr::from(7u64);
    let mut msg: Round1Msg<G2Affine> = strict(&inbox[&0][&3]).unwrap();
    msg.comm_coeffs.0[1] = (params().g2 * delta + msg.comm_coeffs.0[1]).into_affine();
    inbox.get_mut(&0).unwrap().insert(3, encoded(&msg));
    flight(&mut p, &mut inbox);
    let mut share: Share<Fr> = strict(&inbox[&0][&3]).unwrap();
    share.share += delta;
    inbox.get_mut(&0).unwrap().insert(3, encoded(&share));
    for _ in 4..=FLIGHTS {
        flight(&mut p, &mut inbox);
    }
    let issuer = p[0].issuer().unwrap().encode();
    assert!(p
        .iter()
        .all(|q| q.is_prepared() && q.issuer().unwrap().encode() == issuer));
    assert_eq!(issue(&p, &SIGNERS, 5).unwrap_err(), CombineError::Signature);
    for j in 1..N {
        assert!(!p[0].agrees(j, &p[usize::from(j)].echo()));
        assert!(!p[usize::from(j)].agrees(0, &p[0].echo()));
    }
    assert!(p[2].agrees(3, &p[3].echo()) && p[1].agrees(2, &p[2].echo()));
    let mut other = p[3].echo();
    assert_eq!((other.len(), p[1].echo().len()), (64, 32));
    other[40] ^= 1;
    assert!(!p[2].agrees(3, &other) && p[1].agrees(3, &other));
}

// The state has one encoding: truncations, a trailing byte, another version refused (AT-ID-19).
#[test]
fn at_id_19_the_state_decodes_strictly() {
    let mut p = parties(ID);
    run_to(&mut p, 2);
    let bytes = p[1].encode();
    assert!(Party::decode(&bytes).is_some());
    assert!((0..bytes.len()).all(|n| Party::decode(&bytes[..n]).is_none()));
    let mut longer = bytes.clone();
    longer.push(0);
    assert!(Party::decode(&longer).is_none());
    let mut version = bytes;
    version[0] = 2;
    assert!(Party::decode(&version).is_none());
    assert!(format!("{:?}", p[1]).starts_with("Party"));
}

// Only a signer, a listed slot, a proof; a request gets one share, also restarted (AT-ID-21).
#[test]
fn at_id_21_a_member_answers_only_a_listed_slot_with_a_proof() {
    let p = prepared();
    let issuer = p[0].issuer().unwrap();
    let holder = Credential::from_secret([4; 32]);
    let (outside, _) = SlotRequest::new(&holder, 6, &issuer);
    assert_eq!(p[0].answer(&outside).unwrap_err(), AnswerError::NotListed);
    let (listed, _) = SlotRequest::new(&holder, 9, &issuer);
    assert_eq!(p[1].answer(&listed).unwrap_err(), AnswerError::NotPrepared);
    let (other, _) = SlotRequest::new(&holder, 5, &issuer);
    let mut forged = listed.clone();
    forged.request.commitment = other.request.commitment;
    assert_eq!(p[0].answer(&forged).unwrap_err(), AnswerError::Proof);
    let mut bytes = listed.encode();
    bytes[1..5].copy_from_slice(&5u32.to_le_bytes());
    let relabelled = SlotRequest::decode(&bytes).unwrap();
    assert_eq!(p[0].answer(&relabelled).unwrap_err(), AnswerError::Proof);
    let share = p[0].answer(&listed).unwrap().encode();
    let restarted = Party::decode(&p[0].encode()).unwrap();
    assert_eq!(restarted.answer(&listed).unwrap().encode(), share);
    let (again, _) = SlotRequest::new(&holder, 9, &issuer);
    assert_ne!(restarted.answer(&again).unwrap().encode(), share);
}

// Shares combine only from distinct members of one slot, the request's (AT-ID-22).
#[test]
fn at_id_22_shares_combine_only_for_one_slot_from_distinct_members() {
    let p = prepared();
    let issuer = p[0].issuer().unwrap();
    let holder = Credential::from_secret([5; 32]);
    let (request, pending) = SlotRequest::new(&holder, 5, &issuer);
    let (other, _) = SlotRequest::new(&holder, 9, &issuer);
    let share = |i: usize, r: &SlotRequest| p[i].answer(r).unwrap();
    let twice = [share(0, &request), share(0, &request), share(2, &request)];
    assert_eq!(
        combine(pending.clone(), &twice, &issuer).unwrap_err(),
        CombineError::Shares
    );
    let mixed = [share(0, &request), share(2, &request), share(3, &other)];
    assert_eq!(
        combine(pending.clone(), &mixed, &issuer).unwrap_err(),
        CombineError::Slot
    );
    let wrong = [share(0, &other), share(2, &other), share(3, &other)];
    assert_eq!(
        combine(pending.clone(), &wrong, &issuer).unwrap_err(),
        CombineError::Slot
    );
    assert_eq!(
        combine(pending.clone(), &[], &issuer).unwrap_err(),
        CombineError::Shares
    );
    let good = [share(0, &request), share(2, &request), share(3, &request)];
    assert_eq!(good.iter().map(|s| s.member()).collect::<Vec<_>>(), SIGNERS);
    let single = Issuer::new([1; 32]).public();
    assert_eq!(
        combine(pending.clone(), &good, &single).unwrap_err(),
        CombineError::Signature
    );
    assert!(combine(pending, &good, &issuer).unwrap().verify(&issuer));
}

// Requests and shares have one encoding: truncations, trailing bytes, versions refused (AT-ID-22).
#[test]
fn at_id_22_requests_and_shares_decode_strictly() {
    let p = prepared();
    let issuer = p[0].issuer().unwrap();
    let (request, _) = SlotRequest::new(&Credential::from_secret([6; 32]), 9, &issuer);
    let share = p[2].answer(&request).unwrap().encode();
    let request = request.encode();
    assert_eq!(SlotRequest::decode(&request).unwrap().encode(), request);
    assert_eq!(SignatureShare::decode(&share).unwrap().encode(), share);
    for bytes in [&request, &share] {
        assert!((0..bytes.len()).all(|n| {
            SlotRequest::decode(&bytes[..n]).is_none()
                && SignatureShare::decode(&bytes[..n]).is_none()
        }));
        let (mut longer, mut version) = (bytes.clone(), bytes.clone());
        longer.push(0);
        version[0] = 2;
        for b in [longer, version] {
            assert!(SlotRequest::decode(&b).is_none() && SignatureShare::decode(&b).is_none());
        }
    }
    assert_eq!(slot_label(9).0[..5], [9, 0, 0, 0, 0]);
}

// The setup refuses a threshold, signer list or slot list the committee cannot run (AT-ID-19).
#[test]
fn at_id_19_the_setup_refuses_what_cannot_run() {
    assert!(Setup::new(4, 3, vec![0, 2, 3], vec![1]).is_some());
    assert!(Setup::new(4, 1, vec![0], vec![1]).is_none());
    assert!(Setup::new(4, 5, vec![0, 1, 2, 3, 4], vec![1]).is_none());
    assert!(Setup::new(4, 3, vec![0, 2], vec![1]).is_none());
    assert!(Setup::new(4, 3, vec![2, 0, 3], vec![1]).is_none());
    assert!(Setup::new(4, 3, vec![0, 2, 4], vec![1]).is_none());
    assert!(Setup::new(4, 3, vec![0, 2, 3], vec![]).is_none());
    assert!(Setup::new(4, 3, vec![0, 2, 3], vec![2, 1]).is_none());
    assert!(Setup::new(4, 3, vec![0, 2, 3], vec![1, 1]).is_none());
    assert!(Setup::new(256, 3, vec![0, 2, 3], vec![1]).is_none());
    assert!(Party::new(setup(), 4, ID).is_none());
}
