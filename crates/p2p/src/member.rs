//! A consortium member's duties on its node (`docs/04` §Proposing and signing, §The beacon
//! on cuts, T74): what to commit, reveal, propose or co-sign, read from the replica.

use network::beacon::RoundId;
use network::consortium::{Consortium, Member};
use network::cut::{
    added, collect, member_objects, proposer, should_sign, sign_cut, Collected, Cut, MemberObject,
};
use network::replica::Replica;
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
}

/// The cuts collected from `replica`, from number 0, as far as each extends the last.
pub fn collected(replica: &Replica, consortium: &Consortium, network_id: [u8; 32]) -> Vec<Cut> {
    let mut cuts: Vec<Cut> = Vec::new();
    while let Collected::Ready(cut, _) = collect(replica, consortium, network_id, cuts.len() as u64)
    {
        if added(replica, cuts.last(), &cut).is_err() {
            break;
        }
        cuts.push(cut);
    }
    cuts
}

impl MemberRole {
    fn round(&self, network_id: [u8; 32], epoch: u64) -> RoundId {
        RoundId {
            network_id,
            member_set_hash: self.consortium.member_set_hash(),
            epoch,
        }
    }

    /// The objects this member publishes now; `waited` counts a proposer's looks without
    /// proposing.
    pub fn duties(
        &self,
        replica: &Replica,
        network_id: [u8; 32],
        waited: &mut u32,
    ) -> Vec<MemberObject> {
        let c = &self.consortium;
        let cuts = collected(replica, c, network_id);
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
        let signed = |who: usize| {
            has(
                who,
                &|o| matches!(o, MemberObject::CutSignature { cut, .. } if cut.number == next),
            )
        };
        if signed(me) {
            return out;
        }
        let members = 0..c.keys().len();
        if proposer(c, next) == me {
            let in_epoch = cuts.iter().filter(|x| x.epoch == epoch).count() as u64;
            let closes = in_epoch + 1 >= self.cuts_per_epoch;
            let reveals =
                closed.is_none_or(|e| members.clone().all(|m| !committed(m, e) || revealed(m, e)));
            let commits = !closes || members.clone().all(|m| committed(m, epoch));
            let ready = reveals && commits;
            if ready || *waited >= self.patience {
                *waited = 0;
                let cut = Cut::next(replica, last, epoch, closes);
                out.push(sign_cut(&self.member, me, network_id, c, &cut));
            } else {
                *waited += 1;
            }
            return out;
        }
        let proposal = objects.iter().find_map(|(m, o)| match o {
            MemberObject::CutSignature { cut, .. }
                if *m == proposer(c, next) && cut.number == next =>
            {
                Some(cut)
            }
            _ => None,
        });
        if let Some(cut) = proposal.filter(|cut| should_sign(replica, c, me, last, cut)) {
            out.push(sign_cut(&self.member, me, network_id, c, cut));
        }
        out
    }
}
