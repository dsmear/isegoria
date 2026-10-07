//! The view change under random schedules (`docs/04` §Changing view, `docs/08` PROTO-015,
//! T76): members on their own replicas, partial deliveries and crashes.

use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{collect, Collected};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, WriterSet};
use p2p::member::{collected, MemberRole, Progress};
use proptest::prelude::*;
use std::time::Duration;

const NET: [u8; 32] = [5; 32];
const RELAY: u8 = 99;

fn seed(i: usize) -> [u8; 32] {
    [70 + i as u8; 32]
}

fn consortium(n: usize, t: usize) -> Consortium {
    let keys = (0..n)
        .map(|i| Member::from_seed(seed(i)).public())
        .collect();
    Consortium::new(keys, t)
}

#[derive(Clone, Debug)]
enum Step {
    /// A member looks at its duties.
    Tick(usize),
    /// A member's replica takes every entry of writer `writer` (the relay at `n`) another
    /// member's replica holds.
    Deliver {
        from: usize,
        to: usize,
        writer: usize,
    },
    /// The relay logs a new entry into a member's replica.
    Relay(usize),
    /// A member stops for good.
    Crash(usize),
}

struct Model {
    n: usize,
    t: usize,
    roles: Vec<MemberRole>,
    writers: Vec<FeedWriter>,
    logs: Vec<TransparencyLog>,
    replicas: Vec<Replica>,
    progress: Vec<Progress>,
    alive: Vec<bool>,
}

impl Model {
    fn new(n: usize, t: usize, cuts_per_epoch: u64) -> Model {
        let mut writers: Vec<FeedWriter> = (0..n)
            .map(|i| FeedWriter::from_seed(NET, seed(i)))
            .collect();
        writers.push(FeedWriter::from_seed(NET, [RELAY; 32]));
        let keys: Vec<_> = writers.iter().map(|w| w.public()).collect();
        let set = WriterSet::new(NET, &keys);
        Model {
            n,
            t,
            roles: (0..n)
                .map(|i| MemberRole {
                    member: Member::from_seed(seed(i)),
                    index: i,
                    consortium: consortium(n, t),
                    every: Duration::from_millis(1),
                    cuts_per_epoch,
                    patience: 2,
                    view_patience: 4,
                })
                .collect(),
            writers,
            logs: (0..=n).map(|_| TransparencyLog::new()).collect(),
            replicas: (0..n).map(|_| Replica::new(set.clone())).collect(),
            progress: vec![Progress::default(); n],
            alive: vec![true; n],
        }
    }

    fn write(&mut self, writer: usize, into: usize, bytes: Vec<u8>) {
        let entry = self.logs[writer].append(cid(&bytes)).clone();
        let signed = self.writers[writer].sign(&entry);
        self.replicas[into].insert(signed, bytes).unwrap();
    }

    fn step(&mut self, step: &Step) {
        match *step {
            Step::Tick(i) if self.alive[i % self.n] => {
                let i = i % self.n;
                let out = self.roles[i].duties(&self.replicas[i], NET, &mut self.progress[i]);
                for o in out {
                    self.write(i, i, o.encode());
                }
            }
            Step::Deliver { from, to, writer } => {
                let (from, to, writer) = (from % self.n, to % self.n, writer % (self.n + 1));
                if from == to || !self.alive[from] {
                    return;
                }
                let key = self.writers[writer].public().to_bytes();
                let ids: Vec<_> = self.replicas[from]
                    .ids()
                    .filter(|id| id.writer == key)
                    .collect();
                for id in ids {
                    let (e, o) = self.replicas[from].get(&id).unwrap();
                    let (e, o) = (e.clone(), o.to_vec());
                    self.replicas[to].insert(e, o).unwrap();
                }
            }
            Step::Relay(i) => {
                let n = self.n;
                let k = self.logs[n].len();
                self.write(n, i % n, format!("relayed-{k}").into_bytes());
            }
            Step::Crash(i) => {
                let dead = self.alive.iter().filter(|a| !**a).count();
                if dead < self.n - self.t {
                    self.alive[i % self.n] = false;
                }
            }
            Step::Tick(_) => {}
        }
    }

    /// Every entry any replica holds.
    fn union(&self) -> Replica {
        let mut all = Replica::new(self.replicas[0].writers().clone());
        for r in &self.replicas {
            for id in r.ids().collect::<Vec<_>>() {
                let (e, o) = r.get(&id).unwrap();
                let _ = all.insert(e.clone(), o.to_vec());
            }
        }
        all
    }

    /// No number has two cuts collected over everything the members ever signed (a conflict,
    /// once there, stays), and every member's cuts are a prefix of the union's.
    fn check_safe(&self) -> usize {
        let all = self.union();
        let c = consortium(self.n, self.t);
        for k in 0.. {
            match collect(&all, &c, NET, k) {
                Collected::Conflict(a, b) => panic!("two cuts collected for {k}: {a:?} {b:?}"),
                Collected::Pending => break,
                Collected::Ready(..) => {}
            }
        }
        let cuts = collected(&all, &c, NET);
        for r in &self.replicas {
            let mine = collected(r, &c, NET);
            assert_eq!(mine[..], cuts[..mine.len()], "a member's cuts diverge");
        }
        cuts.len()
    }

    /// Full delivery between live members, every live member ticking, `rounds` times.
    fn heal(&mut self, rounds: usize) {
        for _ in 0..rounds {
            for from in 0..self.n {
                for to in 0..self.n {
                    if self.alive[to] {
                        for w in 0..=self.n {
                            self.step(&Step::Deliver {
                                from,
                                to,
                                writer: w,
                            });
                        }
                    }
                }
            }
            for i in 0..self.n {
                self.step(&Step::Tick(i));
            }
        }
    }
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        6 => (0..5usize).prop_map(Step::Tick),
        6 => (0..5usize, 0..5usize, 0..6usize)
            .prop_map(|(from, to, writer)| Step::Deliver { from, to, writer }),
        1 => (0..5usize).prop_map(Step::Relay),
        1 => (0..5usize).prop_map(Step::Crash),
    ];
    proptest::collection::vec(step, 20..120)
}

fn sizes() -> impl Strategy<Value = (usize, usize)> {
    prop_oneof![Just((3, 2)), Just((4, 3)), Just((5, 3))]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// AT-NET-19: under any schedule of looks, partial deliveries and crashes, closing cuts
    /// and beacon messages included, no number has two cuts collected.
    #[test]
    fn at_net_19_no_two_cuts_for_one_number((n, t) in sizes(), schedule in steps()) {
        let mut m = Model::new(n, t, 2);
        for s in &schedule {
            m.step(s);
        }
        m.check_safe();
        m.heal(30);
        m.check_safe();
    }

    /// AT-NET-19: after any schedule with at most `n − t` crashes, once the live members
    /// reach each other the cuts go on (no cut closes, so no beacon message is waited for).
    #[test]
    fn at_net_19_the_live_members_keep_cutting((n, t) in sizes(), schedule in steps()) {
        let mut m = Model::new(n, t, u64::MAX);
        for s in &schedule {
            m.step(s);
        }
        let before = m.check_safe();
        m.step(&Step::Relay(0));
        m.heal(60);
        prop_assert!(m.check_safe() >= before + 2, "the cuts stalled at {}", before);
    }
}
