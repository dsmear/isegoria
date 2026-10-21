//! The epoch's beacon run on cuts (`docs/04` §The beacon on cuts, `docs/08` PROTO-015,
//! T74 step 2): commits and reveals as members' objects, their deadlines the cuts.

use identity::credential::{Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::Role;
use network::beacon::{BeaconReveal, BeaconRound, RoundError, RoundId};
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{EntryId, FeedWriter, Replica, WriterSet};
use protocol::deposit::{deposit_context, Draft};
use protocol::events::NodeEvent;
use protocol::ledger::{CutReport, Ledger, LedgerError, Refusal};

const NET: [u8; 32] = [8; 32];
const SEEDS: [u8; 4] = [10, 11, 12, 20];

fn members() -> Vec<Member> {
    SEEDS[..3]
        .iter()
        .map(|s| Member::from_seed([*s; 32]))
        .collect()
}

fn consortium() -> Consortium {
    Consortium::new(members().iter().map(|m| m.public()).collect(), 2)
}

fn issuer() -> Issuer {
    Issuer::new([1u8; 32])
}

fn deposit(issuer: &Issuer, epoch: u64) -> NodeEvent {
    let holder = Credential::from_secret([2; 32]);
    let (req, pending) = holder.request_issuance(&Label([2; 32]), &issuer.public());
    let credential = pending.finalize(issuer.issue(&req).unwrap());
    let draft = Draft {
        item: b"item".to_vec(),
        primary_source: b"source".to_vec(),
    };
    let proof = prove(
        &credential,
        &issuer.public(),
        Role::Propose,
        &deposit_context(draft.content_id(), epoch),
    );
    NodeEvent::Deposit {
        epoch,
        quota: 1,
        draft,
        proof,
    }
}

/// Members 0–2 and a relay (3), each with its feed; cuts proposed over all of it.
struct Net {
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replica: Replica,
    ledger: Ledger,
}

impl Net {
    fn new() -> Net {
        let writers: Vec<FeedWriter> = SEEDS
            .iter()
            .map(|s| FeedWriter::from_seed(NET, [*s; 32]))
            .collect();
        let keys: Vec<_> = writers.iter().map(|w| w.public()).collect();
        Net {
            replica: Replica::new(WriterSet::new(NET, &keys)),
            logs: (0..4).map(|_| TransparencyLog::new()).collect(),
            writers,
            ledger: Ledger::new(NET, consortium(), issuer().public()),
        }
    }

    fn publish(&mut self, w: usize, object: Vec<u8>) -> EntryId {
        let entry = self.logs[w].append(cid(&object)).clone();
        let signed = self.writers[w].sign(&entry);
        let id = signed.id();
        self.replica.insert(signed, object).unwrap();
        id
    }

    fn member(&mut self, w: usize, object: MemberObject) -> EntryId {
        self.publish(w, object.encode())
    }

    /// Proposes the next cut, signed by `by`, and applies it if the ledger takes it.
    fn cut_by(&mut self, epoch: u64, closes: bool, by: &[usize]) -> Result<CutReport, LedgerError> {
        let cut = Cut::next(&self.replica, self.ledger.last(), epoch, closes);
        let cp = cut.checkpoint(NET, consortium().member_set_hash());
        let sigs: Vec<_> = by.iter().map(|&i| (i, members()[i].sign(&cp))).collect();
        self.ledger.apply(&self.replica, &cut, &sigs)
    }

    /// Epoch 0's round with every member committing and `revealing` revealing, run on cuts
    /// 0 (closing epoch 0) and 1 (the round finishing there).
    fn round_with(&mut self, revealing: &[usize]) {
        for (i, m) in members().iter().enumerate() {
            self.member(i, MemberObject::Commit(m.beacon_commit(round(0), i).0));
        }
        self.cut(0, true);
        for &i in revealing {
            let reveal = members()[i].beacon_commit(round(0), i).1;
            self.member(i, MemberObject::Reveal { epoch: 0, reveal });
        }
        self.cut(1, false);
    }

    /// Proposes, signs with two members and applies the next cut.
    fn cut(&mut self, epoch: u64, closes: bool) -> CutReport {
        let cut = Cut::next(&self.replica, self.ledger.last(), epoch, closes);
        let c = consortium();
        let cp = cut.checkpoint(NET, c.member_set_hash());
        let sigs: Vec<_> = members()[..2]
            .iter()
            .enumerate()
            .map(|(i, m)| (i, m.sign(&cp)))
            .collect();
        self.ledger.apply(&self.replica, &cut, &sigs).unwrap()
    }
}

fn round(epoch: u64) -> RoundId {
    RoundId {
        network_id: NET,
        member_set_hash: consortium().member_set_hash(),
        epoch,
    }
}

/// AT-PRO-14: commits count in the cuts of their epoch up to the closing one, reveals in
/// the cut after it; the outcome is the in-process round's; everything else is refused.
#[test]
fn at_pro_14_the_beacon_runs_on_cuts() {
    let issuer = issuer();
    let mut net = Net::new();
    let pairs: Vec<_> = members()
        .iter()
        .enumerate()
        .map(|(i, m)| m.beacon_commit(round(0), i))
        .collect();
    let c0 = net.member(0, MemberObject::Commit(pairs[0].0));
    let c1 = net.member(1, MemberObject::Commit(pairs[1].0));
    let early = net.member(
        0,
        MemberObject::Reveal {
            epoch: 0,
            reveal: pairs[0].1,
        },
    );
    let on_relay = net.member(3, MemberObject::Commit(pairs[2].0));
    let late_deposit = net.publish(3, deposit(&issuer, 1).encode());
    let report = net.cut(0, false);
    assert_eq!(report.applied.len(), 2);
    for (id, why) in [
        (early, Refusal::OutOfWindow),
        (on_relay, Refusal::NotAuthorized),
        (late_deposit, Refusal::WrongEpoch),
    ] {
        assert!(report.refused.contains(&(id, why)), "{id:?}");
    }
    assert!(report.applied.contains(&c0) && report.applied.contains(&c1));

    let c2 = net.member(2, MemberObject::Commit(pairs[2].0));
    let again = net.member(0, MemberObject::Commit(pairs[0].0));
    let deposit0 = net.publish(3, deposit(&issuer, 0).encode());
    let report = net.cut(0, true);
    assert!(report.applied.contains(&c2) && report.applied.contains(&deposit0));
    assert!(report
        .refused
        .contains(&(again, Refusal::Beacon(RoundError::DuplicateCommit))));
    assert_eq!(net.ledger.beacon(0), None);

    let r0 = net.member(
        0,
        MemberObject::Reveal {
            epoch: 0,
            reveal: pairs[0].1,
        },
    );
    let r1 = net.member(
        1,
        MemberObject::Reveal {
            epoch: 0,
            reveal: pairs[1].1,
        },
    );
    let wrong = BeaconReveal {
        member: 2,
        secret: [0; 32],
    };
    let bad = net.member(
        2,
        MemberObject::Reveal {
            epoch: 0,
            reveal: wrong,
        },
    );
    let stale = net.member(
        2,
        MemberObject::Commit(members()[2].beacon_commit(round(0), 2).0),
    );
    let report = net.cut(1, false);
    assert!(report.applied.contains(&r0) && report.applied.contains(&r1));
    assert!(report
        .refused
        .contains(&(bad, Refusal::Beacon(RoundError::RevealMismatch))));
    assert!(report.refused.contains(&(stale, Refusal::OutOfWindow)));

    let c = consortium();
    let mut direct = BeaconRound::open(&c, NET, 0);
    for (commit, _) in &pairs {
        direct.commit(commit).unwrap();
    }
    direct.close_commits().unwrap();
    direct.close_deposits().unwrap();
    for (_, reveal) in &pairs[..2] {
        direct.reveal(reveal).unwrap();
    }
    let outcome = direct.finish().unwrap();
    assert_eq!(net.ledger.beacon(0), Some(&outcome));
    assert!(outcome.value().is_some());
    assert_eq!(outcome.withheld(), &[2]);
    assert_eq!(net.ledger.beacon(1), None);
}

/// AT-PRO-14: with fewer than `t` reveals the round ends with no value, its withholders
/// recorded; with no commit at all it ends empty.
#[test]
fn at_pro_14_too_few_reveals_give_no_beacon() {
    let mut net = Net::new();
    let pairs: Vec<_> = members()
        .iter()
        .enumerate()
        .map(|(i, m)| m.beacon_commit(round(0), i))
        .collect();
    for (i, (commit, _)) in pairs.iter().enumerate() {
        net.member(i, MemberObject::Commit(*commit));
    }
    net.cut(0, true);
    net.member(
        1,
        MemberObject::Reveal {
            epoch: 0,
            reveal: pairs[1].1,
        },
    );
    net.cut(1, true);
    let outcome = net.ledger.beacon(0).unwrap();
    assert_eq!(outcome.value(), None);
    assert_eq!(outcome.revealed(), &[1]);
    assert_eq!(outcome.withheld(), &[0, 2]);
    net.cut(2, false);
    let empty = net.ledger.beacon(1).unwrap();
    assert_eq!(
        (empty.value(), empty.revealed(), empty.withheld()),
        (None, &[][..], &[][..])
    );
}

/// AT-PRO-15: a member that withheld its reveal counts toward no cut of the next epoch after
/// its round finished, its closing included; from the epoch after, it counts again.
#[test]
fn at_pro_15_a_withholder_does_not_sign_its_epochs_next_cuts() {
    let mut net = Net::new();
    net.round_with(&[0, 1]);
    let outcome = net.ledger.beacon(0).unwrap();
    assert!(outcome.value().is_some());
    assert_eq!(outcome.withheld(), [2]);
    let probe = |number, epoch| Cut {
        number,
        epoch,
        closes: false,
        marks: Vec::new(),
    };
    assert_eq!(net.ledger.excluded(&probe(2, 1)), vec![2]);
    assert!(net.ledger.excluded(&probe(2, 2)).is_empty());
    assert_eq!(net.cut_by(1, false, &[0, 2]), Err(LedgerError::Unsigned));
    assert!(net.cut_by(1, false, &[0, 1]).is_ok());
    assert_eq!(net.cut_by(1, true, &[1, 2]), Err(LedgerError::Unsigned));
    assert!(net.cut_by(1, true, &[0, 1]).is_ok());
    assert!(
        net.cut_by(2, false, &[0, 2]).is_ok(),
        "the next epoch counts it again"
    );
}

/// AT-PRO-15: a round with fewer than `t` reveals yields no beacon and publishes nothing, so
/// no signature is excluded.
#[test]
fn at_pro_15_no_beacon_excludes_no_one() {
    let mut net = Net::new();
    net.round_with(&[0]);
    assert!(net.ledger.beacon(0).unwrap().value().is_none());
    assert!(net.cut_by(1, false, &[0, 2]).is_ok());
}

// AT-PRO-15/C6: view signatures and withholder exclusions preserve the author clock atomically.
#[test]
fn view_changes_and_withholders_preserve_the_author_clock() {
    let mut net = Net::new();
    net.round_with(&[0, 1]);
    assert_eq!(net.ledger.state().results().closing_count(), 1);
    assert!(net.ledger.state().results().author_point().is_none());
    let cut = Cut::next(&net.replica, net.ledger.last(), 1, true);
    let cp = cut.checkpoint_in(3, NET, consortium().member_set_hash());
    let sign = |by: &[usize]| -> Vec<_> { by.iter().map(|&i| (i, members()[i].sign(&cp))).collect() };
    let before = net.ledger.last().cloned();
    assert_eq!(
        net.ledger.apply_in_view(&net.replica, &cut, 3, &sign(&[0, 2])),
        Err(LedgerError::Unsigned)
    );
    assert_eq!(net.ledger.last(), before.as_ref());
    assert_eq!(net.ledger.state().results().closing_count(), 1);
    assert!(net.ledger.state().results().author_point().is_none());
    net.ledger
        .apply_in_view(&net.replica, &cut, 3, &sign(&[0, 1]))
        .unwrap();
    assert_eq!(net.ledger.state().results().closing_count(), 2);
    assert!(net.ledger.state().results().author_point().is_none());
    assert!(net
        .ledger
        .apply_in_view(&net.replica, &cut, 3, &sign(&[0, 1]))
        .is_err());
    assert_eq!(net.ledger.state().results().closing_count(), 2);
    assert!(net.cut_by(2, false, &[0, 2]).is_ok());
    assert_eq!(net.ledger.state().results().closing_count(), 2);
}
