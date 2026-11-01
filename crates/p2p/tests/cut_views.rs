//! The view and exclusions each collected cut is reported with (`docs/04` §The cuts' views and
//! exclusions in the status, AT-NET-39): both collectors, against `collect_with`.

use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{collect_with, sign_cut_in, views_change, Collected, Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica};
use network::tier::{Tier, Tiers};
use p2p::member::{CollectedCut, Collector, PublicCollector};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

const NET: [u8; 32] = [39; 32];

fn member(i: usize) -> Member {
    Member::from_seed([100 + i as u8; 32])
}

/// Members' public feeds `0..n` and circle feeds `n..2n` in one full replica.
struct Net {
    n: usize,
    t: usize,
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replica: Replica,
}

impl Net {
    fn new(n: usize, t: usize) -> Net {
        let tiers = Tiers::new(NET, consortium(n, t), &circle(n)).unwrap();
        let mut net = Net {
            n,
            t,
            writers: (0..2 * n).map(writer).collect(),
            logs: (0..2 * n).map(|_| TransparencyLog::new()).collect(),
            replica: Replica::new(tiers.writers().clone()),
        };
        net.draft(0);
        net
    }

    fn consortium(&self) -> Consortium {
        consortium(self.n, self.t)
    }

    fn tiers(&self) -> Tiers {
        Tiers::new(NET, self.consortium(), &circle(self.n)).unwrap()
    }

    fn publish(&mut self, w: usize, bytes: Vec<u8>) {
        let entry = self.logs[w].append(cid(&bytes)).clone();
        let signed = self.writers[w].sign(&entry);
        self.replica.insert(signed, bytes).unwrap();
    }

    /// A circle entry of member `m`, which a reader never holds.
    fn draft(&mut self, m: usize) {
        let k = self.logs[self.n + m].len();
        self.publish(self.n + m, format!("draft-{m}-{k}").into_bytes());
    }

    fn sign(&mut self, cut: &Cut, view: u64, by: &[usize]) {
        for &m in by {
            let o = sign_cut_in(
                &member(m),
                m,
                NET,
                &self.consortium(),
                cut,
                view,
                Vec::new(),
            );
            self.publish(m, o.encode());
        }
    }

    fn round(&self, epoch: u64) -> RoundId {
        RoundId {
            network_id: NET,
            member_set_hash: self.consortium().member_set_hash(),
            epoch,
        }
    }

    fn commit(&mut self, m: usize, epoch: u64) {
        let (commit, _) = member(m).beacon_commit(self.round(epoch), m);
        self.publish(m, MemberObject::Commit(commit).encode());
    }

    fn reveal(&mut self, m: usize, epoch: u64) {
        let (_, reveal) = member(m).beacon_commit(self.round(epoch), m);
        self.publish(m, MemberObject::Reveal { epoch, reveal }.encode());
    }

    /// What a reader without circle access holds.
    fn public(&self) -> Replica {
        let tiers = self.tiers();
        let mut out = Replica::new(tiers.writers().clone());
        for id in self.replica.ids() {
            if tiers.tier(&id.writer) == Some(Tier::Public) {
                let (e, o) = self.replica.get(&id).unwrap();
                out.insert(e.clone(), o.to_vec()).unwrap();
            }
        }
        out
    }
}

fn consortium(n: usize, t: usize) -> Consortium {
    Consortium::new((0..n).map(|i| member(i).public()).collect(), t)
}

/// Feed `w`: member `w`'s public feed below `n`, member `w − n`'s circle feed from `n`.
fn writer(w: usize) -> FeedWriter {
    FeedWriter::from_seed(NET, [100 + w as u8; 32])
}

fn circle(n: usize) -> Vec<ed25519_dalek::VerifyingKey> {
    (n..2 * n).map(|w| writer(w).public()).collect()
}

/// A member's and a reader's collector, extended together.
struct Both {
    member: Collector,
    reader: PublicCollector,
}

impl Both {
    fn new(net: &Net) -> Both {
        Both {
            member: Collector::new(&net.consortium(), NET),
            reader: PublicCollector::new(&net.tiers()),
        }
    }

    /// Both reports after an update from `net`, which must agree; the member's cuts with them.
    fn update(&mut self, net: &Net) -> (Vec<Cut>, Vec<CollectedCut>) {
        let cuts = self
            .member
            .update(&net.replica, &net.consortium(), NET)
            .unwrap()
            .to_vec();
        let read = self.reader.update(&net.public(), &net.tiers()).unwrap();
        assert_eq!(read, &cuts[..], "the reader follows the member's cuts");
        let reported = self.member.collected_cuts().to_vec();
        assert_eq!(
            self.reader.collected_cuts(),
            &reported[..],
            "the reader reports alike"
        );
        assert_eq!(reported.len(), cuts.len());
        assert_eq!(self.member.conflict(), self.reader.conflict());
        (cuts, reported)
    }
}

/// Each reported cut is the one `collect_with` gives for its number with its exclusions, in
/// its view.
fn oracle(net: &Net, cuts: &[Cut], reported: &[CollectedCut]) {
    for (k, (cut, r)) in cuts.iter().zip(reported).enumerate() {
        assert_eq!(r.digest, cut.digest());
        let excluded = r.excluded.clone();
        let got = collect_with(&net.replica, &net.consortium(), NET, k as u64, &move |_| {
            excluded.clone()
        });
        match got {
            Collected::Ready(c, view, _) => {
                assert_eq!((c.digest(), view), (cut.digest(), r.view), "cut {k}")
            }
            other => panic!("cut {k}: {other:?}"),
        }
    }
}

fn views(reported: &[CollectedCut]) -> Vec<u64> {
    reported.iter().map(|r| r.view).collect()
}

fn exclusions(reported: &[CollectedCut]) -> Vec<Vec<usize>> {
    reported.iter().map(|r| r.excluded.clone()).collect()
}

// Cuts collected in views 0, 1 and 2 each report their own view (AT-NET-39).
#[test]
fn at_net_39_each_cut_reports_its_view() {
    for (n, t) in [(3, 2), (4, 3)] {
        let mut net = Net::new(n, t);
        assert!(views_change(&net.consortium()));
        let all: Vec<usize> = (0..t).collect();
        let mut both = Both::new(&net);
        let mut last: Option<Cut> = None;
        for view in 0..3u64 {
            net.draft(view as usize % n);
            let cut = Cut::next(&net.replica, last.as_ref(), 0, false);
            for lower in 0..view {
                net.sign(&cut, lower, &all[..t - 1]);
            }
            net.sign(&cut, view, &all);
            let (cuts, reported) = both.update(&net);
            assert_eq!(cuts.last(), Some(&cut));
            oracle(&net, &cuts, &reported);
            last = Some(cut);
        }
        let (_, reported) = both.update(&net);
        assert_eq!(views(&reported), [0, 1, 2], "n = {n}, t = {t}");
        assert_eq!(exclusions(&reported), vec![Vec::<usize>::new(); 3]);
    }
}

/// `n = 3`, `t = 2`: epoch 0 closes at cut 1, members 0 and 1 reveal, 2 withholds; cut 2
/// finishes the round with a value. Returns the net and the cuts 0–2.
fn withheld() -> (Net, Vec<Cut>) {
    let mut net = Net::new(3, 2);
    for m in 0..3 {
        net.commit(m, 0);
    }
    let first = Cut::next(&net.replica, None, 0, false);
    net.sign(&first, 0, &[0, 1]);
    let closing = Cut::next(&net.replica, Some(&first), 0, true);
    net.sign(&closing, 0, &[0, 1, 2]);
    net.reveal(0, 0);
    net.reveal(1, 0);
    let ending = Cut::next(&net.replica, Some(&closing), 1, false);
    net.sign(&ending, 0, &[0, 1, 2]);
    (net, vec![first, closing, ending])
}

// Rule 6's withholder is reported excluded exactly where collection excludes it (AT-NET-39).
#[test]
fn at_net_39_cuts_report_the_withholders_rule_6_excludes() {
    let (mut net, mut cuts) = withheld();
    let mut both = Both::new(&net);
    let (_, reported) = both.update(&net);
    assert_eq!(reported.len(), 3);
    let value = both.member.beacons()[0].clone();
    assert!(value.value().is_some() && value.withheld() == [2]);
    let third = Cut::next(&net.replica, cuts.last(), 1, false);
    net.sign(&third, 0, &[0, 2]);
    let without = collect_with(&net.replica, &net.consortium(), NET, 3, &|_| vec![]);
    assert!(matches!(without, Collected::Ready(ref c, 0, _) if *c == third));
    assert_eq!(
        both.update(&net).0.len(),
        3,
        "the withholder's signature does not count"
    );
    net.sign(&third, 0, &[1]);
    cuts.push(third);
    let closing = Cut::next(&net.replica, cuts.last(), 1, true);
    net.sign(&closing, 0, &[2, 1, 0]);
    cuts.push(closing);
    let next = Cut::next(&net.replica, cuts.last(), 2, false);
    net.sign(&next, 0, &[0, 2]);
    cuts.push(next);
    let (collected, reported) = both.update(&net);
    assert_eq!(collected, cuts);
    let none = Vec::<usize>::new();
    let expected = [&none, &none, &none, &vec![2], &vec![2], &none];
    assert_eq!(exclusions(&reported).iter().collect::<Vec<_>>(), expected);
    oracle(&net, &collected, &reported);
}

// A lower view completed after collection is reported, by no excluded signer (AT-NET-39).
#[test]
fn at_net_39_a_lower_view_completed_later_is_reported() {
    let mut net = Net::new(4, 3);
    let cut = Cut::next(&net.replica, None, 0, false);
    net.sign(&cut, 2, &[0, 1, 2]);
    net.sign(&cut, 1, &[1, 2]);
    net.sign(&cut, 0, &[0, 1]);
    let mut both = Both::new(&net);
    assert_eq!(views(&both.update(&net).1), [2]);
    net.sign(&cut, 1, &[3]);
    assert_eq!(views(&both.update(&net).1), [1]);
    net.sign(&cut, 0, &[3]);
    let (cuts, reported) = both.update(&net);
    assert_eq!(views(&reported), [0]);
    oracle(&net, &cuts, &reported);
    assert_eq!(Both::new(&net).update(&net).1, reported);

    let (mut net, mut cuts) = withheld();
    let third = Cut::next(&net.replica, cuts.last(), 1, false);
    net.sign(&third, 1, &[0, 1]);
    cuts.push(third.clone());
    let mut both = Both::new(&net);
    assert_eq!(views(&both.update(&net).1), [0, 0, 0, 1]);
    net.sign(&third, 0, &[0, 2]);
    let (collected, reported) = both.update(&net);
    assert_eq!(
        views(&reported),
        [0, 0, 0, 1],
        "member 2's view-0 signature does not count"
    );
    oracle(&net, &collected, &reported);
    net.sign(&third, 0, &[1]);
    assert_eq!(views(&both.update(&net).1), [0, 0, 0, 0]);
}

// At a conflict the cuts before it keep their views and exclusions (AT-NET-39).
#[test]
fn at_net_39_a_conflict_keeps_the_cuts_before_it() {
    let (mut net, mut cuts) = withheld();
    let third = Cut::next(&net.replica, cuts.last(), 1, false);
    net.sign(&third, 1, &[0, 1]);
    cuts.push(third);
    let mut both = Both::new(&net);
    let (_, before) = both.update(&net);
    assert_eq!(views(&before), [0, 0, 0, 1]);
    assert_eq!(before[3].excluded, [2]);
    net.draft(1);
    let fourth = Cut::next(&net.replica, cuts.last(), 1, false);
    net.sign(&fourth, 0, &[0, 1]);
    net.draft(2);
    let rival = Cut::next(&net.replica, cuts.last(), 1, false);
    assert_ne!(rival, fourth);
    net.sign(&rival, 1, &[0, 1]);
    let (_, held) = both.update(&net);
    assert!(both.member.conflict() && both.reader.conflict());
    assert_eq!(held, before);

    let (mut net, cuts) = withheld();
    let mut both = Both::new(&net);
    let (_, before) = both.update(&net);
    net.draft(2);
    let rival = Cut::next(&net.replica, Some(&cuts[1]), 1, false);
    assert_ne!(rival, cuts[2]);
    net.sign(&rival, 0, &[1, 2]);
    let (_, held) = both.update(&net);
    assert!(both.member.conflict() && both.reader.conflict());
    assert_eq!(held, before[..2]);
}

// A second conflict at an earlier number takes both collectors back to it (AT-NET-39).
#[test]
fn at_net_39_a_second_conflict_takes_the_cuts_back() {
    for earlier in [1usize, 0] {
        let mut net = Net::new(3, 2);
        let mut cuts: Vec<Cut> = Vec::new();
        for m in 0..3 {
            net.draft(m);
            let cut = Cut::next(&net.replica, cuts.last(), 0, false);
            net.sign(&cut, 0, &[0, 1]);
            cuts.push(cut);
        }
        let mut both = Both::new(&net);
        assert_eq!(both.update(&net).0, cuts);
        for (m, by) in [(1, [0, 1]), (2, [1, 2])] {
            net.draft(m);
            let next = Cut::next(&net.replica, cuts.last(), 0, false);
            net.sign(&next, 0, &by);
        }
        assert_eq!(both.update(&net).0, cuts);
        assert!(both.member.conflict());
        net.draft(2);
        let rival = Cut::next(
            &net.replica,
            earlier.checked_sub(1).map(|p| &cuts[p]),
            0,
            false,
        );
        assert_ne!(rival, cuts[earlier]);
        net.sign(&rival, 0, &[0, 2]);
        let (kept, reported) = both.update(&net);
        assert!(both.member.conflict());
        assert_eq!(kept, cuts[..earlier], "second conflict at {earlier}");
        let mut fresh = Both::new(&net);
        assert_eq!(fresh.update(&net), (kept.clone(), reported.clone()));
        assert!(fresh.member.conflict());
        oracle(&net, &kept, &reported);
        assert_eq!(both.update(&net), (kept, reported), "looked at again");
    }
}

/// When a generated signature is given: never, as its cut is built, or after every cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Given {
    Never,
    Now,
    Late,
}

/// One cut of a generated schedule: whether it closes its epoch, the members committing at an
/// epoch's start and revealing after its close, each member's signature per view, an update,
/// and a rival cut for the number `back` before this one with its view-0 signers.
#[derive(Clone, Debug)]
struct Planned {
    closes: bool,
    commits: Vec<bool>,
    reveals: Vec<bool>,
    signers: Vec<Vec<Given>>,
    update: bool,
    rival: Option<(usize, Vec<bool>)>,
}

fn planned(n: usize) -> impl Strategy<Value = Planned> {
    let mask = move |p: f64| proptest::collection::vec(proptest::bool::weighted(p), n);
    let given = prop_oneof![2 => Just(Given::Never), 2 => Just(Given::Now), 1 => Just(Given::Late)];
    (
        proptest::bool::weighted(0.4),
        mask(0.9),
        mask(0.7),
        proptest::collection::vec(proptest::collection::vec(given, n), 3),
        proptest::bool::weighted(0.7),
        proptest::option::weighted(0.06, (0..8usize, mask(0.7))),
    )
        .prop_map(
            |(closes, commits, reveals, signers, update, rival)| Planned {
                closes,
                commits,
                reveals,
                signers,
                update,
                rival,
            },
        )
}

fn schedule() -> impl Strategy<Value = (usize, usize, Vec<Planned>)> {
    prop_oneof![Just((3, 2)), Just((4, 3)), Just((5, 3))].prop_flat_map(|(n, t)| {
        (
            Just(n),
            Just(t),
            proptest::collection::vec(planned(n), 3..10),
        )
    })
}

/// A cut after `prev` signed in view 0 by `by`, unlike any cut before it: a circle draft first.
fn rival(net: &mut Net, prev: Option<&Cut>, epoch: u64, closes: bool, by: &[usize]) -> Cut {
    net.draft(0);
    let cut = Cut::next(&net.replica, prev, epoch, closes);
    net.sign(&cut, 0, by);
    cut
}

/// Builds `plan` into `net`, `both` updated where planned and the late signatures given last;
/// returns the cuts built and the rivals signed.
fn build(net: &mut Net, both: &mut Both, plan: &[Planned]) -> (Vec<Cut>, Vec<Cut>) {
    let n = net.n;
    let (mut built, mut late, mut rivals) = (Vec::<Cut>::new(), Vec::new(), Vec::new());
    for (k, p) in plan.iter().enumerate() {
        let last = built.last();
        let epoch = last.map_or(0, |l| l.epoch + u64::from(l.closes));
        if last.is_none_or(|l| l.closes) {
            if let Some(e) = last.map(|l| l.epoch) {
                for m in (0..n).filter(|m| p.reveals[*m]) {
                    net.reveal(m, e);
                }
            }
            for m in (0..n).filter(|m| p.commits[*m]) {
                net.commit(m, epoch);
            }
        }
        net.draft(k % n);
        let cut = Cut::next(&net.replica, built.last(), epoch, p.closes);
        for (view, given) in p.signers.iter().enumerate() {
            for (m, when) in given.iter().enumerate() {
                match when {
                    Given::Now => net.sign(&cut, view as u64, &[m]),
                    Given::Late => late.push((cut.clone(), view as u64, m)),
                    Given::Never => {}
                }
            }
        }
        built.push(cut);
        if let Some((back, signers)) = &p.rival {
            let j = k - back % (k + 1);
            let by: Vec<usize> = (0..n).filter(|m| signers[*m]).collect();
            let (prev, like) = (j.checked_sub(1).map(|i| &built[i]), &built[j]);
            rivals.push(rival(net, prev, like.epoch, like.closes, &by));
        }
        if p.update {
            both.update(net);
        }
    }
    for (cut, view, m) in late {
        net.sign(&cut, view, &[m]);
    }
    (built, rivals)
}

/// After an update, each collected cut was signed among `signed`, is `collect_with`'s, and a
/// fresh collector reports alike; returns the cuts and the report.
fn agrees(
    net: &Net,
    both: &mut Both,
    signed: &[&[Cut]],
) -> Result<(Vec<Cut>, Vec<CollectedCut>), TestCaseError> {
    let (cuts, reported) = both.update(net);
    for (k, cut) in cuts.iter().enumerate() {
        prop_assert!(
            signed.iter().any(|s| s.contains(cut)),
            "cut {} never signed",
            k
        );
    }
    oracle(net, &cuts, &reported);
    let mut fresh = Both::new(net);
    prop_assert_eq!(fresh.update(net), (cuts.clone(), reported.clone()));
    prop_assert_eq!(fresh.member.conflict(), both.member.conflict());
    Ok((cuts, reported))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// AT-NET-39: generated schedules and rivals report `collect_with`'s cuts, as a fresh one does.
    #[test]
    fn at_net_39_reports_follow_collect_with((n, t, plan) in schedule()) {
        let mut net = Net::new(n, t);
        let mut both = Both::new(&net);
        let (built, rivals) = build(&mut net, &mut both, &plan);
        agrees(&net, &mut both, &[&built, &rivals])?;
    }

    /// AT-NET-39: after a conflict at the next number, one at a collected number reports as from 0.
    #[test]
    fn at_net_39_a_second_conflict_reports_as_from_zero(
        (n, t, plan) in schedule(),
        back in 0..8usize,
    ) {
        let mut net = Net::new(n, t);
        let mut both = Both::new(&net);
        let (built, mut rivals) = build(&mut net, &mut both, &plan);
        let (cuts, _) = agrees(&net, &mut both, &[&built, &rivals])?;
        let all: Vec<usize> = (0..n).collect();
        let last = cuts.last();
        let epoch = last.map_or(0, |l| l.epoch + u64::from(l.closes));
        for _ in 0..2 {
            rivals.push(rival(&mut net, last, epoch, false, &all));
        }
        prop_assert_eq!(agrees(&net, &mut both, &[&built, &rivals])?.0, cuts.clone());
        prop_assert!(both.member.conflict());
        if let Some(j) = back.checked_rem(cuts.len()) {
            let (prev, like) = (j.checked_sub(1).map(|i| &cuts[i]), &cuts[j]);
            rivals.push(rival(&mut net, prev, like.epoch, like.closes, &all));
            let (second, _) = agrees(&net, &mut both, &[&built, &rivals])?;
            prop_assert!(both.member.conflict());
            prop_assert_eq!(&second[..], &cuts[..j]);
        }
    }
}
