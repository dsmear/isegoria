//! A consortium member's duties on its node (`docs/04` §Proposing and signing, §Changing view,
//! §The beacon on cuts, T74, T76): what to commit, reveal, propose, co-sign or change view.

use ed25519_dalek::Signature;
use network::beacon::BeaconOutcome;
use network::beacon::RoundId;
use network::consortium::{Consortium, Member};
use network::cut::{
    added, changed_view, collect_with, counts_own, forced, member_objects, promise, proposal,
    proposer_in, should_sign_in, sign_cut_in, signed_in, views_change, Collected, Cut,
    MemberObject,
};
use network::observer::{ObserveError, Observer};
use network::replica::Replica;
use network::rounds::Rounds;
use network::tier::Tiers;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

pub struct MemberRole {
    pub member: Member,
    pub index: usize,
    pub consortium: Consortium,
    /// How often the member looks for a duty.
    pub every: Duration,
    /// The cuts a proposer puts in an epoch before it closes it.
    pub cuts_per_epoch: u64,
    /// The looks a proposer waits for missing commits or reveals before proposing anyway.
    pub patience: u32,
    /// The looks without the cut collected before the member changes view; above `patience`.
    pub view_patience: u32,
}

/// A member's looks at the cut it works on, reset when the cut or its view changes, and the
/// cuts it collected.
#[derive(Clone, Default)]
pub struct Progress {
    /// Looks the proposer has waited for missing commits or reveals.
    pub waited: u32,
    /// Looks without the cut collected in the member's view.
    pub idle: u32,
    at: Option<(u64, u64)>,
    collector: Collector,
}

impl Progress {
    /// A member's progress, its collector fixed to `consortium` on `network_id`.
    pub fn new(consortium: &Consortium, network_id: [u8; 32]) -> Self {
        Progress {
            collector: Collector::new(consortium, network_id),
            ..Progress::default()
        }
    }

    /// Whether the duties halted at a conflict their collector found (`docs/04` §The duties'
    /// stop at a conflict); never cleared.
    pub fn halted(&self) -> bool {
        self.collector.conflict()
    }
}

/// The network id, member-set hash and threshold a collector counts its rounds under.
type Context = ([u8; 32], [u8; 32], usize);

fn context_of(consortium: &Consortium, network_id: [u8; 32]) -> Context {
    (
        network_id,
        consortium.member_set_hash(),
        consortium.threshold(),
    )
}

/// A consortium or network other than the one a collector was fixed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OtherContext;

/// A collected cut as a node reports it (`docs/04` §The cuts' views and exclusions in the
/// status): the view its signatures count in, and the members whose signatures it does not
/// count, increasing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectedCut {
    pub digest: [u8; 32],
    pub view: u64,
    pub excluded: Vec<usize>,
}

/// The cuts collected so far and the beacon's rounds they ran, extended from the last cut
/// rather than collected again from number 0 (`docs/04` §A node process); fixed to one
/// consortium and network, by [`Collector::new`] or at its first update.
#[derive(Clone, Default)]
pub struct Collector {
    context: Option<Context>,
    cuts: Vec<Cut>,
    collected: Vec<CollectedCut>,
    rounds: Rounds,
    conflict: bool,
}

impl Collector {
    pub fn new(consortium: &Consortium, network_id: [u8; 32]) -> Self {
        Collector {
            context: Some(context_of(consortium, network_id)),
            ..Collector::default()
        }
    }

    /// Extends the cuts from `replica`, a withholder's signature counted only where rule 6
    /// allows; two cuts collected for one number, also after an earlier conflict, keep the
    /// cuts before it and stop there.
    pub fn update(
        &mut self,
        replica: &Replica,
        consortium: &Consortium,
        network_id: [u8; 32],
    ) -> Result<&[Cut], OtherContext> {
        let context = context_of(consortium, network_id);
        if self.context.is_some_and(|c| c != context) {
            return Err(OtherContext);
        }
        self.context = Some(context);
        let others = recheck(
            replica,
            consortium,
            network_id,
            &self.cuts,
            &mut self.collected,
        );
        let rounds = &self.rounds;
        let excluded = |c: &Cut| rounds.excluded(c);
        if contested(replica, consortium, network_id, others, &excluded) {
            let mut fresh = Collector::new(consortium, network_id);
            fresh.extend(replica, consortium, network_id);
            fresh.conflict = true;
            *self = fresh;
        }
        self.extend(replica, consortium, network_id);
        Ok(&self.cuts)
    }

    /// Collects the next cuts from `replica` until one is pending or two conflict.
    fn extend(&mut self, replica: &Replica, consortium: &Consortium, network_id: [u8; 32]) {
        while !self.conflict {
            let number = self.cuts.len() as u64;
            let rounds = &self.rounds;
            let ready = collect_with(replica, consortium, network_id, number, &|c| {
                rounds.excluded(c)
            });
            let (cut, view) = match ready {
                Collected::Ready(cut, view, _) => (cut, view),
                Collected::Conflict(..) => {
                    self.conflict = true;
                    break;
                }
                Collected::Pending => break,
            };
            let Ok(order) = added(replica, self.cuts.last(), &cut) else {
                break;
            };
            let excluded = self.rounds.excluded(&cut);
            for id in order {
                let object = replica.get(&id).and_then(|(_, b)| MemberObject::decode(b));
                let own = object.filter(|o| {
                    consortium.keys().get(o.member()).map(|k| k.to_bytes()) == Some(id.writer)
                });
                if let Some(o) = own {
                    let last = self.cuts.last();
                    let _ = self.rounds.count(consortium, network_id, &o, &cut, last);
                }
            }
            let last = self.cuts.last();
            self.rounds.finish(consortium, network_id, &cut, last);
            self.collected.push(CollectedCut {
                digest: cut.digest(),
                view,
                excluded,
            });
            self.cuts.push(cut);
        }
    }

    pub fn cuts(&self) -> &[Cut] {
        &self.cuts
    }

    /// Each collected cut's digest, view and exclusions, from number 0.
    pub fn collected_cuts(&self) -> &[CollectedCut] {
        &self.collected
    }

    /// Whether the consortium signed two cuts for one number.
    pub fn conflict(&self) -> bool {
        self.conflict
    }

    /// The outcomes of the rounds the cuts finished, from epoch 0.
    pub fn beacons(&self) -> Vec<&BeaconOutcome> {
        (0..).map_while(|e| self.rounds.outcome(e)).collect()
    }
}

/// Lowers each collected cut's view to the lowest in which `replica` holds `t` of its counted
/// signatures, as [`collect_with`] picks it; returns the numbers signed for another cut.
fn recheck(
    replica: &Replica,
    consortium: &Consortium,
    network_id: [u8; 32],
    cuts: &[Cut],
    collected: &mut [CollectedCut],
) -> BTreeSet<u64> {
    type Group = (Cut, Vec<(usize, Signature)>);
    let mut others = BTreeSet::new();
    let mut lower: BTreeMap<(usize, u64), Group> = BTreeMap::new();
    for (_, o) in member_objects(replica, consortium) {
        let MemberObject::CutSignature {
            cut,
            view,
            member,
            signature,
            ..
        } = o
        else {
            continue;
        };
        let Some(k) = usize::try_from(cut.number).ok().filter(|k| *k < cuts.len()) else {
            continue;
        };
        if cuts[k] != cut {
            others.insert(cut.number);
        } else if view < collected[k].view {
            let group = lower.entry((k, view)).or_insert_with(|| (cut, Vec::new()));
            group.1.push((member, signature));
        }
    }
    for ((k, view), (cut, signatures)) in lower {
        let cp = cut.checkpoint_in(view, network_id, consortium.member_set_hash());
        let counted = consortium.verify_excluding(&cp, &signatures, &collected[k].excluded);
        if counted && view < collected[k].view {
            collected[k].view = view;
        }
    }
    others
}

/// Whether one of `numbers` now has two cuts each signed by `t` members.
fn contested(
    replica: &Replica,
    consortium: &Consortium,
    network_id: [u8; 32],
    numbers: BTreeSet<u64>,
    excluded: &dyn Fn(&Cut) -> Vec<usize>,
) -> bool {
    numbers.into_iter().any(|n| {
        let ready = collect_with(replica, consortium, network_id, n, excluded);
        matches!(ready, Collected::Conflict(..))
    })
}

/// The cuts a node without circle access follows (`docs/04` §Who reads), applied by an
/// [`Observer`] fixed to one tiered set; a conflict found later keeps the cuts before it.
#[derive(Clone)]
pub struct PublicCollector {
    observer: Observer,
    cuts: Vec<Cut>,
    collected: Vec<CollectedCut>,
    conflict: bool,
}

impl PublicCollector {
    pub fn new(tiers: &Tiers) -> Self {
        PublicCollector {
            observer: Observer::new(tiers),
            cuts: Vec::new(),
            collected: Vec::new(),
            conflict: false,
        }
    }

    /// Extends the cuts from `replica` as [`Collector::update`] does; other tiers, or a replica
    /// of another writer set, are refused before anything changes.
    pub fn update(&mut self, replica: &Replica, tiers: &Tiers) -> Result<&[Cut], ObserveError> {
        if !self.observer.follows(tiers) {
            return Err(ObserveError::Context);
        }
        if replica.writers() != tiers.writers() {
            return Err(ObserveError::Replica);
        }
        let (c, net) = (tiers.consortium(), tiers.network_id());
        let others = recheck(replica, c, net, &self.cuts, &mut self.collected);
        let observer = &self.observer;
        let excluded = |cut: &Cut| observer.excluded(cut);
        if contested(replica, c, net, others, &excluded) {
            let mut fresh = PublicCollector::new(tiers);
            fresh.extend(replica, tiers);
            fresh.conflict = true;
            *self = fresh;
        }
        self.extend(replica, tiers);
        Ok(&self.cuts)
    }

    /// As [`Collector`]'s extension, each cut applied by the observer.
    fn extend(&mut self, replica: &Replica, tiers: &Tiers) {
        let (c, net) = (tiers.consortium(), tiers.network_id());
        while !self.conflict {
            let number = self.cuts.len() as u64;
            let observer = &self.observer;
            let ready = collect_with(replica, c, net, number, &|cut| observer.excluded(cut));
            match ready {
                Collected::Ready(cut, view, sigs) => {
                    let excluded = self.observer.excluded(&cut);
                    if self
                        .observer
                        .apply_in_view(replica, tiers, &cut, view, &sigs)
                        .is_err()
                    {
                        break;
                    }
                    self.collected.push(CollectedCut {
                        digest: cut.digest(),
                        view,
                        excluded,
                    });
                    self.cuts.push(cut);
                }
                Collected::Conflict(..) => self.conflict = true,
                Collected::Pending => break,
            }
        }
    }

    pub fn cuts(&self) -> &[Cut] {
        &self.cuts
    }

    pub fn conflict(&self) -> bool {
        self.conflict
    }

    /// As [`Collector::collected_cuts`].
    pub fn collected_cuts(&self) -> &[CollectedCut] {
        &self.collected
    }

    pub fn beacons(&self) -> Vec<&BeaconOutcome> {
        (0..).map_while(|e| self.observer.beacon(e)).collect()
    }
}

/// The cuts collected from `replica`, from number 0, as far as each extends the last; a
/// withholder's signature counted only where rule 6 allows, as the ledger counts it.
pub fn collected(replica: &Replica, consortium: &Consortium, network_id: [u8; 32]) -> Vec<Cut> {
    let mut collector = Collector::new(consortium, network_id);
    collector
        .update(replica, consortium, network_id)
        .map(<[Cut]>::to_vec)
        .unwrap_or_default()
}

impl MemberRole {
    fn round(&self, network_id: [u8; 32], epoch: u64) -> RoundId {
        RoundId {
            network_id,
            member_set_hash: self.consortium.member_set_hash(),
            epoch,
        }
    }

    /// The objects this member publishes now; none once its cuts hold a conflict.
    pub fn duties(
        &self,
        replica: &Replica,
        network_id: [u8; 32],
        progress: &mut Progress,
    ) -> Vec<MemberObject> {
        let c = &self.consortium;
        let Ok(cuts) = progress
            .collector
            .update(replica, c, network_id)
            .map(<[Cut]>::to_vec)
        else {
            return Vec::new();
        };
        if progress.halted() {
            return Vec::new();
        }
        let last = cuts.last();
        let epoch = last.map_or(0, |l| l.epoch + u64::from(l.closes));
        let objects: Vec<(usize, MemberObject)> = member_objects(replica, c)
            .map(|(_, o)| (o.member(), o))
            .collect();
        let has = |who: usize, f: &dyn Fn(&MemberObject) -> bool| {
            objects.iter().any(|(m, o)| *m == who && f(o))
        };
        let committed = |who: usize, e: u64| {
            has(
                who,
                &|o| matches!(o, MemberObject::Commit(x) if x.round.epoch == e),
            )
        };
        let revealed = |who: usize, e: u64| {
            has(
                who,
                &|o| matches!(o, MemberObject::Reveal { epoch, .. } if *epoch == e),
            )
        };
        let me = self.index;
        let mut out = Vec::new();
        if !committed(me, epoch) {
            let (commit, _) = self.member.beacon_commit(self.round(network_id, epoch), me);
            out.push(MemberObject::Commit(commit));
        }
        let closed = last.filter(|l| l.closes).map(|l| l.epoch);
        if let Some(e) = closed.filter(|e| committed(me, *e) && !revealed(me, *e)) {
            let (_, reveal) = self.member.beacon_commit(self.round(network_id, e), me);
            out.push(MemberObject::Reveal { epoch: e, reveal });
        }
        let next = cuts.len() as u64;
        let view = promise(replica, c, me, next);
        if progress.at != Some((next, view)) {
            progress.at = Some((next, view));
            progress.waited = 0;
            progress.idle = 0;
        }
        let n = c.keys().len();
        let changes = views_change(c);
        let mut others: Vec<u64> = (0..n)
            .filter(|m| *m != me)
            .map(|m| promise(replica, c, m, next))
            .collect();
        others.sort_unstable_by(|a, b| b.cmp(a));
        let joined = c.threshold().checked_sub(2).and_then(|i| others.get(i));
        if let Some(&w) = joined.filter(|w| changes && **w > view) {
            out.push(self.change(next, w));
            return out;
        }

        let members = 0..n;
        let fresh = |waited: &mut u32| {
            let in_epoch = cuts.iter().filter(|x| x.epoch == epoch).count() as u64;
            let closes = in_epoch + 1 >= self.cuts_per_epoch;
            let reveals =
                closed.is_none_or(|e| members.clone().all(|m| !committed(m, e) || revealed(m, e)));
            let commits = !closes || members.clone().all(|m| committed(m, epoch));
            if reveals && commits || *waited >= self.patience {
                *waited = 0;
                Some(Cut::next(replica, last, epoch, closes))
            } else {
                *waited += 1;
                None
            }
        };
        let mine = proposer_in(c, next, view) == me && !signed_in(replica, c, me, next, view);
        if mine {
            let named = changed_view(replica, c, next, view);
            let cut = match view {
                0 => fresh(&mut progress.waited),
                _ if named.len() < c.threshold() => None,
                _ => match forced(replica, c, next, view, &named) {
                    Ok(Some(cut)) => counts_own(replica, c, me, last, &cut).then_some(cut),
                    Ok(None) => fresh(&mut progress.waited),
                    Err(_) => None,
                },
            };
            if let Some(cut) = cut {
                let named = if view == 0 { Vec::new() } else { named };
                out.push(sign_cut_in(
                    &self.member,
                    me,
                    network_id,
                    c,
                    &cut,
                    view,
                    named,
                ));
                return out;
            }
        }
        let mut views: Vec<u64> = objects
            .iter()
            .filter_map(|(_, o)| match o {
                MemberObject::CutSignature { cut, view: v, .. } if cut.number == next => Some(*v),
                _ => None,
            })
            .filter(|v| *v >= view)
            .collect();
        views.sort_unstable();
        views.dedup();
        let top = views
            .into_iter()
            .filter_map(|v| proposal(replica, c, next, v).map(|cut| (v, cut)))
            .rfind(|(v, cut)| should_sign_in(replica, c, me, last, cut, *v));
        if let Some((v, cut)) = top {
            out.push(sign_cut_in(
                &self.member,
                me,
                network_id,
                c,
                &cut,
                v,
                Vec::new(),
            ));
            return out;
        }
        progress.idle += 1;
        let with_me = 1 + others.iter().filter(|v| **v >= view).count();
        let held = view == 0 || with_me >= c.threshold();
        if changes && held && progress.idle > self.view_patience {
            out.push(self.change(next, view.saturating_add(1)));
        }
        out
    }

    fn change(&self, number: u64, view: u64) -> MemberObject {
        MemberObject::ViewChange {
            number,
            view,
            member: self.index,
        }
    }
}
