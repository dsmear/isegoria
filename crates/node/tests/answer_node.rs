//! A member's node running the answer review (`docs/05` §Where the state is computed, `docs/01`
//! D60, `docs/10` T79): the hook, the operators' requests, the state digest, withheld reveals.

#[path = "support/genesis.rs"]
mod genesis;

use identity::credential::{AnonymousCredential, Credential, Issuer};
use identity::enrollment::Label;
use identity::nullifier::prove;
use identity::nym::Role;
use libp2p::Multiaddr;
use network::beacon::RoundId;
use network::cid::cid;
use network::consortium::{Consortium, Member};
use network::cut::{sign_cut, Cut, MemberObject};
use network::log::TransparencyLog;
use network::replica::{FeedWriter, Replica, SignedEntry};
use node::answer::AnswerHook;
use node::{hex, ConfigError};
use p2p::hook::{executed, pending, Follower, Hook};
use p2p::member::{CollectedCut, Collector, Faults, MemberRole, Progress};
use p2p::{Handle, Status};
use protocol::answer::Question;
use protocol::answer_events::AnswerEvent;
use protocol::answer_ledger::{AnswerLedger, Refusal};
use protocol::answer_path::{network_context, Refused, Round, POOL_CONTEXT};
use protocol::deposit::deposit_context;
use protocol::lottery;
use protocol::randomness::{Beacon, ANSWER_DRAW};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const N: usize = 3;
const T: usize = 2;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("node_answer")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn member_seed(i: usize) -> [u8; 32] {
    [1 + i as u8; 32]
}

fn circle_seed(i: usize) -> [u8; 32] {
    [11 + i as u8; 32]
}

fn transport_seed(i: usize) -> [u8; 32] {
    [31 + i as u8; 32]
}

fn port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn list(keys: impl IntoIterator<Item = [u8; 32]>) -> String {
    let quoted: Vec<String> = keys
        .into_iter()
        .map(|k| format!("\"{}\"", hex(&k)))
        .collect();
    format!("[{}]", quoted.join(","))
}

/// Three members, `t` signing (two by default), under the test issuer; each member's files in
/// `root/m<i>`.
struct Fixture {
    root: PathBuf,
    genesis: network::genesis::Genesis,
    ports: Vec<u16>,
    t: usize,
}

impl Fixture {
    fn new(name: &str) -> Fixture {
        Fixture::with(name, T)
    }

    fn with(name: &str, t: usize) -> Fixture {
        let root = scratch(name);
        let members = (0..N)
            .map(|i| genesis::seeded(1 + i as u8, 11 + i as u8, 21 + i as u8))
            .collect();
        let g = genesis::genesis(t, members, vec![]);
        genesis::write(&root.join("genesis.bin"), &g);
        for i in 0..N {
            let dir = root.join(format!("m{i}"));
            std::fs::create_dir_all(&dir).unwrap();
            for (name, seed) in [
                ("member.key", member_seed(i)),
                ("circle.key", circle_seed(i)),
                ("transport.key", transport_seed(i)),
            ] {
                std::fs::write(dir.join(name), hex(&seed)).unwrap();
            }
            let (public, circle) = (genesis::key(1 + i as u8), genesis::key(11 + i as u8));
            p2p::record::create(&dir.join("identity"), public, circle).unwrap();
        }
        let ports = (0..N).map(|_| port()).collect();
        Fixture {
            root,
            genesis: g,
            ports,
            t,
        }
    }

    fn net(&self) -> [u8; 32] {
        self.genesis.id()
    }

    fn dir(&self, i: usize) -> PathBuf {
        self.root.join(format!("m{i}"))
    }

    fn address(&self, i: usize) -> Multiaddr {
        let pair = libp2p::identity::Keypair::ed25519_from_bytes(transport_seed(i)).unwrap();
        let peer = pair.public().to_peer_id();
        format!("/ip4/127.0.0.1/tcp/{}/p2p/{peer}", self.ports[i])
            .parse()
            .unwrap()
    }

    /// Member `i`'s TOML, with `member` the `[member]` table's extra lines.
    fn text(&self, i: usize, member: &str) -> String {
        let g = &self.genesis;
        let consortium: Vec<_> = g.members.iter().map(|m| m.consortium).collect();
        let circle: Vec<_> = g.members.iter().map(|m| m.circle).collect();
        let writers = consortium.iter().chain(&circle).copied();
        let dir = self.dir(i);
        let peers: Vec<String> = (0..N)
            .filter(|j| *j != i)
            .map(|j| format!("\"{}\"", self.address(j)))
            .collect();
        genesis::lines(&self.root.join("genesis.bin"), g)
            + &format!(
                "network_id = \"{}\"\ndata_dir = \"{}\"\nlisten = [\"/ip4/127.0.0.1/tcp/{}\"]\n\
                 peers = [{}]\nwriters = {}\nkey_file = \"{}\"\ncircle_key_file = \"{}\"\n\
                 sync_every_ms = 100\n[consortium]\nmembers = {}\nthreshold = {}\ncircle = {}\n\
                 [member]\nindex = {i}\nevery_ms = 100\npatience = 10\nview_patience = 20\n{member}",
                hex(&g.id()),
                dir.display(),
                self.ports[i],
                peers.join(","),
                list(writers),
                dir.join("member.key").display(),
                dir.join("circle.key").display(),
                list(consortium.clone()),
                self.t,
                list(circle.clone()),
            )
    }

    fn config(&self, i: usize) -> node::Config {
        let path = self.root.join(format!("m{i}.toml"));
        std::fs::write(&path, self.text(i, "")).unwrap();
        node::Config::load(&path).unwrap()
    }

    fn document(&self) -> node::genesis::Document {
        node::genesis::load(&self.root.join("genesis.bin")).unwrap()
    }

    fn hook(&self) -> AnswerHook {
        AnswerHook::new(&self.document()).unwrap()
    }

    fn consortium(&self) -> Consortium {
        let tiers = self.genesis.tiers().unwrap();
        Consortium::new(tiers.consortium().keys().to_vec(), self.t)
    }

    /// Member `i`'s node, planned from its TOML, listening.
    async fn start(&self, i: usize) -> Handle {
        self.start_with(i, Faults::default()).await
    }

    /// As [`Self::start`], its duties injecting `faults`.
    async fn start_with(&self, i: usize, faults: Faults) -> Handle {
        let mut plan = node::plan(self.config(i)).unwrap();
        assert!(plan.p2p.hook.is_some(), "a tiered member runs the hook");
        plan.p2p.member.as_mut().unwrap().faults = faults;
        let handle = Handle::spawn(plan.transport, plan.p2p).unwrap();
        for addr in plan.listen {
            handle.listen(addr).await.unwrap();
        }
        handle
    }

    async fn dial(&self, handle: &Handle, i: usize) {
        for j in (0..N).filter(|j| *j != i) {
            let _ = handle.dial(self.address(j)).await;
        }
    }

    /// As [`until`], member `i` dialing the others again while it lacks one.
    async fn until(&self, handle: &Handle, i: usize, ok: impl Fn(&Status) -> bool) -> Status {
        let end = Instant::now() + Duration::from_secs(40);
        loop {
            let status = handle.status().await;
            if ok(&status) {
                return status;
            }
            assert!(
                Instant::now() < end,
                "member {i}: {}",
                node::render(&status)
            );
            if status.peers < N - 1 {
                self.dial(handle, i).await;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

async fn until(handle: &Handle, what: &str, ok: impl Fn(&Status) -> bool) -> Status {
    let end = Instant::now() + Duration::from_secs(40);
    loop {
        let status = handle.status().await;
        if ok(&status) {
            return status;
        }
        assert!(Instant::now() < end, "{what}: {}", node::render(&status));
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

struct Person {
    cred: AnonymousCredential,
}

fn person(issuer: &Issuer, secret: u8) -> Person {
    let holder = Credential::from_secret([secret; 32]);
    let (req, pending) = holder.request_issuance(&Label([secret; 32]), &issuer.public());
    Person {
        cred: pending.finalize(issuer.issue(&req).unwrap()),
    }
}

fn issuer() -> Issuer {
    Issuer::new([77; 32])
}

fn join(net: &[u8; 32], p: &Person) -> Vec<u8> {
    let ctx = network_context(net, POOL_CONTEXT);
    let proof = prove(&p.cred, &issuer().public(), Role::Judge, &ctx);
    AnswerEvent::JoinPool { proof }.encode()
}

fn question() -> Question {
    Question {
        context: b"civic education, 2026".to_vec(),
        stem: b"which?".to_vec(),
        alternatives: vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()],
        key: 0,
    }
}

fn deposit(net: &[u8; 32], p: &Person) -> Vec<u8> {
    let q = question();
    let ctx = network_context(net, &deposit_context(q.content_id(), 0));
    let proof = prove(&p.cred, &issuer().public(), Role::Propose, &ctx);
    AnswerEvent::Deposit {
        epoch: 0,
        question: q,
        proof,
    }
    .encode()
}

/// The collected cuts of `replica`, each applied to a fresh ledger: the digest after each
/// closing cut, and the ledger.
fn replayed(f: &Fixture, replica: &Replica) -> (Vec<(u64, [u8; 32])>, AnswerHook) {
    let c = f.consortium();
    let mut collector = Collector::new(&c, f.net());
    collector.update(replica, &c, f.net()).unwrap();
    let mut hook = f.hook();
    let mut follower = Follower::default();
    follower.follow(&mut hook, &collector, replica);
    assert!(follower.refused().is_none());
    (follower.digests().to_vec(), hook)
}

/// The objects member `i`'s circle feed holds in `replica`, in feed order.
fn circle_objects(f: &Fixture, replica: &Replica, i: usize) -> Vec<Vec<u8>> {
    let writer = genesis::key(11 + i as u8);
    let mut ids: Vec<_> = replica.ids().filter(|id| id.writer == writer).collect();
    ids.sort_by_key(|id| id.seq);
    let _ = f;
    ids.iter()
        .map(|id| replica.get(id).unwrap().1.to_vec())
        .collect()
}

// Every member reports one digest per closing cut, a replay's; a restarted one rebuilds them,
// its closure requests logged as their events once (AT-T79-17, AT-T79-20, AT-T79-21).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_t79_17_a_restarted_member_rebuilds_the_same_digests() {
    let mut f = Fixture::new("restart");
    let net = f.net();
    let mut nodes = Vec::new();
    for i in 0..N {
        nodes.push(f.start(i).await);
    }
    for (i, node) in nodes.iter().enumerate() {
        f.dial(node, i).await;
    }
    for node in &nodes {
        until(node, "started", |s| {
            s.caught_up && s.circle_caught_up && s.cuts.len() >= 2
        })
        .await;
    }
    let issuer = issuer();
    let people: Vec<Person> = (100..102).map(|s| person(&issuer, s)).collect();
    let author = person(&issuer, 1);
    for event in [
        join(&net, &people[0]),
        join(&net, &people[1]),
        deposit(&net, &author),
    ] {
        nodes[0].publish_circle(event).await.unwrap();
    }
    let marked = nodes[0].status().await.cuts.len() + 3;
    until(&nodes[0], "persons' events in a cut", |s| {
        s.cuts.len() >= marked
    })
    .await;
    let requests = [
        (0, ["close-commits", "0"]),
        (0, ["close-epoch", "0"]),
        (1, ["close-epoch", "0"]),
        (2, ["close-epoch", "0"]),
    ];
    for (i, args) in requests {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        node::answer::request(&f.config(i), &args).unwrap();
    }
    let mut statuses = Vec::new();
    for node in &nodes {
        statuses.push(
            until(node, "a closing cut and its round", |s| {
                !s.digests.is_empty()
                    && !s.beacons.is_empty()
                    && s.cuts.len() > s.digests[0].0 as usize + 2
            })
            .await,
        );
    }
    let first = statuses[0].digests[0];
    for s in &statuses {
        assert_eq!(s.digests[0], first, "{}", node::render(s));
        assert!(s.refused_cut.is_none() && !s.duties_halted);
    }
    assert_eq!(statuses[0].requests.len(), 2);
    assert_eq!(statuses[1].requests.len(), 1);
    let spool = f.dir(0).join("requests");
    assert!(pending(&spool).unwrap().is_empty());
    assert_eq!(executed(&spool).unwrap().len(), 2);

    let replica = nodes[0].replica().await;
    let (digests, hook) = replayed(&f, &replica);
    assert_eq!(digests[0], first, "the status's digest is the replay's");
    let cuts = p2p::member::collected(&replica, &f.consortium(), net);
    let closing: Vec<u64> = cuts.iter().filter(|c| c.closes).map(|c| c.number).collect();
    let numbers: Vec<u64> = digests.iter().map(|d| d.0).collect();
    assert_eq!(numbers, closing, "one digest per closing cut");
    let ledger: &AnswerLedger = hook.ledger();
    let entry = ledger.state().entry(0).unwrap();
    assert_eq!(entry.round, Round::Unavailable { available: 2 });
    let refused: Vec<Refusal> = ledger.refused().iter().map(|r| r.1).collect();
    assert!(refused.contains(&Refusal::Path(Refused::WrongRound)));
    let logged = circle_objects(&f, &replica, 0);
    assert!(logged.contains(&AnswerEvent::CloseCommits { question: 0 }.encode()));
    assert!(logged.contains(&AnswerEvent::RequestCloseEpoch { epoch: 0 }.encode()));

    let before = nodes[0].status().await;
    let length = circle_objects(&f, &replica, 0).len();
    nodes.remove(0).finished().await;
    f.ports[0] = port();
    let restarted = f.start(0).await;
    f.dial(&restarted, 0).await;
    let after = f
        .until(&restarted, 0, |s| {
            s.circle_caught_up && s.cuts.len() > before.cuts.len()
        })
        .await;
    assert_eq!(after.digests[..before.digests.len()], before.digests[..]);
    assert_eq!(after.requests, before.requests);
    let replica = restarted.replica().await;
    assert_eq!(
        circle_objects(&f, &replica, 0).len(),
        length,
        "nothing logged again"
    );
    restarted.finished().await;
    for node in nodes {
        node.finished().await;
    }
}

// A request repeated by its operator is logged again; one left pending after it was logged is
// not, and a restart keeps each request's entry (AT-T79-33).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_t79_33_a_repeated_request_is_logged_again() {
    let mut f = Fixture::new("repeat");
    let mut nodes = Vec::new();
    for i in 0..N {
        nodes.push(f.start(i).await);
    }
    for (i, node) in nodes.iter().enumerate() {
        f.dial(node, i).await;
    }
    let started = |s: &Status| s.caught_up && s.circle_caught_up;
    f.until(&nodes[0], 0, started).await;
    let close = AnswerEvent::CloseCommits { question: 0 }.encode();
    let args: Vec<String> = ["close-commits", "0"]
        .iter()
        .map(|a| a.to_string())
        .collect();
    for _ in 0..2 {
        node::answer::request(&f.config(0), &args).unwrap();
    }
    let both = f.until(&nodes[0], 0, |s| s.requests.len() == 2).await;
    assert_ne!(both.requests[0].1, both.requests[1].1, "two entries");
    let replica = nodes[0].replica().await;
    let count = |f: &Fixture, r: &Replica| {
        circle_objects(f, r, 0)
            .iter()
            .filter(|o| **o == close)
            .count()
    };
    assert_eq!(count(&f, &replica), 2);

    nodes.remove(0).finished().await;
    let spool = f.dir(0).join("requests");
    let (last, _) = executed(&spool).unwrap().pop().unwrap();
    std::fs::rename(spool.join("done").join(&last), spool.join(&last)).unwrap();
    f.ports[0] = port();
    let restarted = f.start(0).await;
    f.dial(&restarted, 0).await;
    let after = f
        .until(&restarted, 0, |s| started(s) && s.requests.len() == 2)
        .await;
    assert!(pending(&spool).unwrap().is_empty());
    assert_eq!(
        after.requests, both.requests,
        "the pending one maps to its entry"
    );
    assert_eq!(
        count(&f, &restarted.replica().await),
        2,
        "nothing logged again"
    );
    restarted.finished().await;
    for node in nodes {
        node.finished().await;
    }
}

/// A replica of the fixture's network and each writer's log, for the members' duties.
struct Feeds {
    replica: Replica,
    logs: std::collections::BTreeMap<[u8; 32], TransparencyLog>,
    net: [u8; 32],
}

impl Feeds {
    fn new(f: &Fixture) -> Feeds {
        let tiers = f.genesis.tiers().unwrap();
        Feeds {
            replica: Replica::new(tiers.writers().clone()),
            logs: Default::default(),
            net: f.net(),
        }
    }

    fn entry(&mut self, seed: [u8; 32], object: Vec<u8>) -> SignedEntry {
        let writer = FeedWriter::from_seed(self.net, seed);
        let log = self.logs.entry(writer.public().to_bytes()).or_default();
        let signed = writer.sign(log.append(cid(&object)));
        self.replica.insert(signed.clone(), object).unwrap();
        signed
    }

    fn sign(&mut self, f: &Fixture, cut: &Cut, by: &[usize]) -> Vec<(SignedEntry, Vec<u8>)> {
        let c = f.consortium();
        let mut out = Vec::new();
        for &m in by {
            let member = Member::from_seed(member_seed(m));
            let object = sign_cut(&member, m, self.net, &c, cut).encode();
            out.push((self.entry(member_seed(m), object.clone()), object));
        }
        out
    }
}

fn role(f: &Fixture, index: usize) -> MemberRole {
    MemberRole {
        member: Member::from_seed(member_seed(index)),
        index,
        consortium: f.consortium(),
        every: Duration::from_millis(100),
        cuts_per_epoch: 1,
        patience: 0,
        view_patience: 1000,
        faults: Faults::default(),
    }
}

fn signed(out: &[MemberObject], number: u64) -> Option<&Cut> {
    out.iter().find_map(|o| match o {
        MemberObject::CutSignature { cut, .. } if cut.number == number => Some(cut),
        _ => None,
    })
}

/// `replica`'s cuts applied to `hook`.
fn followed(f: &Fixture, replica: &Replica, hook: &mut AnswerHook) -> Follower {
    let c = f.consortium();
    let mut collector = Collector::new(&c, f.net());
    collector.update(replica, &c, f.net()).unwrap();
    let mut follower = Follower::default();
    follower.follow(hook, &collector, replica);
    follower
}

// A member co-signs no proposal the members' checks refuse, which it signs without the hook;
// a proposal they accept it signs (AT-T79-18).
#[test]
fn at_t79_18_a_member_does_not_co_sign_a_failing_proposal() {
    let f = Fixture::new("co-sign");
    let me = role(&f, 1);
    let proposals = [(0, true, false), (1, false, false), (0, false, true)];
    for (epoch, closes, accepted) in proposals {
        let mut feeds = Feeds::new(&f);
        let cut = Cut::next(&feeds.replica, None, epoch, closes);
        feeds.sign(&f, &cut, &[0]);
        let mut hook = f.hook();
        followed(&f, &feeds.replica, &mut hook);
        assert_eq!(hook.may_sign(&cut), accepted);
        let mut progress = Progress::new(&f.consortium(), f.net());
        let out = me.duties_with(&feeds.replica, f.net(), &mut progress, Some(&hook));
        assert_eq!(signed(&out, 0).is_some(), accepted, "{epoch} {closes}");
        let mut progress = Progress::new(&f.consortium(), f.net());
        let out = me.duties(&feeds.replica, f.net(), &mut progress);
        assert!(
            signed(&out, 0).is_some(),
            "without the hook, {epoch} {closes}"
        );
    }
}

// A proposer closes an epoch after `t` members' requests in its cuts, never by a count of cuts
// (AT-T79-22).
#[test]
fn at_t79_22_a_proposer_closes_on_t_requests_only() {
    let f = Fixture::new("closes");
    let mut feeds = Feeds::new(&f);
    let mut hook = f.hook();
    followed(&f, &feeds.replica, &mut hook);
    let mut progress = Progress::new(&f.consortium(), f.net());
    let out = role(&f, 0).duties_with(&feeds.replica, f.net(), &mut progress, Some(&hook));
    let cut0 = signed(&out, 0).expect("a proposal").clone();
    assert!(!cut0.closes, "cuts_per_epoch 1 is not used");
    let without = role(&f, 0).duties(&feeds.replica, f.net(), &mut Progress::default());
    assert!(signed(&without, 0).unwrap().closes);
    feeds.sign(&f, &cut0, &[0, 1]);
    let request = AnswerEvent::RequestCloseEpoch { epoch: 0 }.encode();
    feeds.entry(circle_seed(0), request.clone());
    feeds.entry(circle_seed(2), request);
    let cut1 = Cut::next(&feeds.replica, Some(&cut0), 0, false);
    feeds.sign(&f, &cut1, &[1, 2]);
    let mut hook = f.hook();
    followed(&f, &feeds.replica, &mut hook);
    assert!(hook.closes());
    let mut progress = Progress::new(&f.consortium(), f.net());
    let out = role(&f, 2).duties_with(&feeds.replica, f.net(), &mut progress, Some(&hook));
    let cut2 = signed(&out, 2).expect("a proposal");
    assert!(cut2.closes && cut2.epoch == 0);
}

// A collected cut the checks refuse is not applied: the ledger stays fresh, progress stops
// there, and the node reports it, also after a restart (AT-T79-19).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn at_t79_19_a_collected_cut_failing_the_checks_is_not_applied() {
    let f = Fixture::new("refused");
    let mut feeds = Feeds::new(&f);
    let cut0 = Cut::next(&feeds.replica, None, 0, true);
    let colluding = feeds.sign(&f, &cut0, &[0, 1]);
    let mut hook = f.hook();
    let follower = followed(&f, &feeds.replica, &mut hook);
    assert_eq!(follower.applied(), 0);
    assert_eq!(follower.refused().map(|r| r.0), Some(0));
    assert_eq!(hook.ledger().digest(), f.hook().ledger().digest());
    assert!(hook.ledger().last().is_none());

    for _ in 0..2 {
        let node = f.start(2).await;
        for (entry, object) in &colluding {
            let _ = node.insert(entry.clone(), object.clone()).await;
        }
        let status = until(&node, "the refused cut", |s| s.refused_cut.is_some()).await;
        let (number, reason) = status.refused_cut.clone().unwrap();
        assert_eq!((number, reason.as_str()), (0, "checks"));
        assert!(status.duties_halted && status.digests.is_empty());
        assert_eq!(status.cuts.len(), 1, "the cut is kept as evidence");
        node.finished().await;
    }
}

// The status carries each closing cut's digest, the refused cut and the executed requests,
// and reads back; malformed lines are refused (AT-T79-20).
#[test]
fn at_t79_20_the_status_carries_the_digests() {
    let status = Status {
        cuts: vec![CollectedCut {
            digest: [1; 32],
            view: 0,
            excluded: vec![],
        }],
        digests: vec![(3, [0xab; 32]), (9, [0xcd; 32])],
        refused_cut: Some((11, "checks".to_string())),
        requests: vec![("00000000000000000001-close-epoch-0".to_string(), 4)],
        ..Status::default()
    };
    let text = node::render(&status);
    let ab = hex(&[0xab; 32]);
    assert!(text.contains(&format!("digest 3 {ab}\n")), "{text}");
    assert!(text.contains("refused_cut 11 checks\n"), "{text}");
    assert!(text.contains("request 00000000000000000001-close-epoch-0 4\n"));
    assert_eq!(node::parse(&text), Some(status.clone()));
    let none = node::render(&Status::default());
    assert!(none.contains("refused_cut none\n"));
    assert_eq!(node::parse(&none), Some(Status::default()));
    for bad in [
        text.replace("digest 3", "digest three"),
        text.replace(&format!("digest 3 {ab}"), &format!("digest 3 {}", &ab[2..])),
        text.replace(
            &format!("digest 9 {}", hex(&[0xcd; 32])),
            &format!("digest 2 {ab}"),
        ),
        text.replace("refused_cut 11 checks", "refused_cut 11"),
        text.replace("refused_cut 11 checks", "refused_cut eleven checks"),
        text.replace("close-epoch-0 4", "close-epoch-0"),
        text.replace("close-epoch-0 4", "close-epoch-0 four"),
    ] {
        assert_eq!(node::parse(&bad), None, "{bad}");
    }
}

// Each operator command spools its event under a new name; other words, other numbers and
// nodes that are no tiered member are refused, writing nothing (AT-T79-21).
#[test]
fn at_t79_21_operator_commands_spool_their_events() {
    let f = Fixture::new("commands");
    let config = f.config(1);
    let spool = f.dir(1).join("requests");
    let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let commands = [
        (
            ["close-commits", "7"],
            AnswerEvent::CloseCommits { question: 7 },
        ),
        (["close", "7"], AnswerEvent::Close { question: 7 }),
        (
            ["close-epoch", "2"],
            AnswerEvent::RequestCloseEpoch { epoch: 2 },
        ),
    ];
    for (args, event) in &commands {
        let name = node::answer::request(&config, &words(args)).unwrap();
        assert!(name.ends_with(&args.join("-")), "{name}");
        let bytes = std::fs::read(spool.join(&name)).unwrap();
        assert_eq!(bytes, event.encode());
    }
    let names: Vec<String> = pending(&spool).unwrap().into_iter().map(|p| p.0).collect();
    assert_eq!(names.len(), 3);
    for bad in [
        &["close-epoch"][..],
        &["close-epoch", "two"],
        &["close-epoch", "-1"],
        &["close-epoch", "2", "3"],
        &["draw", "0"],
        &["join", "0"],
        &[],
    ] {
        let refused = node::answer::request(&config, &words(bad));
        assert_eq!(refused, Err(ConfigError::Request), "{bad:?}");
    }
    assert_eq!(pending(&spool).unwrap().len(), 3);
    let path = f.root.join("plain.toml");
    let reader = f.text(1, "").replace("[member]", "[reader]");
    let reader = reader.split("[reader]").next().unwrap().to_string();
    std::fs::write(&path, reader).unwrap();
    let reader = node::Config::load(&path).unwrap();
    let refused = node::answer::request(&reader, &words(&["close-epoch", "0"]));
    assert_eq!(refused, Err(ConfigError::MemberWithoutKey));
    let other = f
        .text(1, "")
        .replace(&f.dir(1).display().to_string(), "/nonexistent/m1");
    std::fs::write(&path, other).unwrap();
    let missing = node::Config::load(&path).unwrap();
    let refused = node::answer::request(&missing, &words(&["close-epoch", "0"]));
    assert!(matches!(refused, Err(ConfigError::Read(_))), "{refused:?}");
}

// `--request` prints the spooled request's name; arguments that do not hold exit 2 (AT-T79-21).
#[test]
fn at_t79_21_the_request_flag() {
    let f = Fixture::new("flag");
    let path = f.root.join("m0.toml");
    std::fs::write(&path, f.text(0, "")).unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_isegoria-node"))
            .arg("--request")
            .arg(&path)
            .args(args)
            .output()
            .unwrap()
    };
    let out = run(&["close-epoch", "0"]);
    assert!(out.status.success(), "{out:?}");
    let name = String::from_utf8(out.stdout).unwrap();
    let name = name.trim_end();
    assert!(f.dir(0).join("requests").join(name).is_file(), "{name}");
    for bad in [&["close-epoch"][..], &["open", "0"]] {
        assert_eq!(run(bad).status.code(), Some(2));
    }
}

// A tiered member runs the hook and configures no `cuts_per_epoch`; a member of a network
// without a genesis needs one (AT-T79-22).
#[test]
fn at_t79_22_the_hook_replaces_cuts_per_epoch() {
    let f = Fixture::new("config");
    let path = f.root.join("m0.toml");
    std::fs::write(&path, f.text(0, "cuts_per_epoch = 2\n")).unwrap();
    let config = node::Config::load(&path).unwrap();
    assert!(matches!(node::plan(config), Err(ConfigError::CutsPerEpoch)));
    assert!(node::plan(f.config(0)).unwrap().p2p.hook.is_some());
}

// A member's duties withhold its reveals of the epochs its faults name and are otherwise the
// same; a planned member has none, and no configuration key sets one (AT-T79-30).
#[test]
fn at_t79_30_the_switch_withholds_reveals_only() {
    let f = Fixture::new("switch");
    let mut feeds = Feeds::new(&f);
    let round = RoundId {
        network_id: f.net(),
        member_set_hash: f.consortium().member_set_hash(),
        epoch: 0,
    };
    let (commit, _) = Member::from_seed(member_seed(2)).beacon_commit(round, 2);
    feeds.entry(member_seed(2), MemberObject::Commit(commit).encode());
    let cut0 = Cut::next(&feeds.replica, None, 0, true);
    feeds.sign(&f, &cut0, &[0, 1]);
    let duties = |faults: Faults| {
        let me = MemberRole {
            faults,
            ..role(&f, 2)
        };
        let mut progress = Progress::new(&f.consortium(), f.net());
        me.duties(&feeds.replica, f.net(), &mut progress)
    };
    let is_reveal = |o: &MemberObject| matches!(o, MemberObject::Reveal { epoch: 0, .. });
    let honest = duties(Faults::default());
    assert!(honest.iter().any(is_reveal));
    let kept: Vec<MemberObject> = honest.iter().filter(|o| !is_reveal(o)).cloned().collect();
    assert!(kept.iter().any(|o| matches!(o, MemberObject::Commit(_))));
    assert_eq!(duties(Faults::withholding_reveals([0])), kept);
    assert_eq!(duties(Faults::withholding_reveals([1])), honest);

    let planned = node::plan(f.config(2)).unwrap().p2p.member.unwrap();
    assert_eq!(planned.faults, Faults::default());
    let path = f.root.join("faulty.toml");
    for key in ["faults = [0]", "withhold_reveals = [0]"] {
        std::fs::write(&path, f.text(2, &format!("{key}\n"))).unwrap();
        let refused = node::Config::load(&path);
        assert!(matches!(refused, Err(ConfigError::Parse(_))), "{key}");
    }
}

fn request(f: &Fixture, i: usize, args: [&str; 2]) {
    let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    node::answer::request(&f.config(i), &args).unwrap();
}

/// The ledger `handle`'s replica gives, replayed from cut 0.
async fn ledger(f: &Fixture, handle: &Handle) -> AnswerHook {
    replayed(f, &handle.replica().await).1
}

/// Until the ledger of `handle`'s replica holds `ok`.
async fn until_ledger(f: &Fixture, handle: &Handle, ok: impl Fn(&AnswerLedger) -> bool) {
    let end = Instant::now() + Duration::from_secs(40);
    while !ok(ledger(f, handle).await.ledger()) {
        assert!(Instant::now() < end, "the ledger");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Why `ledger` refused the entry of member `i`'s circle feed holding `event`.
fn refused(ledger: &AnswerLedger, replica: &Replica, i: usize, event: &AnswerEvent) -> Refusal {
    let bytes = event.encode();
    let circle = genesis::key(11 + i as u8);
    let found = ledger.refused().iter().find(|(id, _)| {
        id.writer == circle && replica.get(id).is_some_and(|(_, b)| b[..] == bytes[..])
    });
    found.map(|r| r.1).expect("a refusal")
}

/// Question 0's group in `ledger`, checked to be the one epoch 0's pool draws under round
/// `epoch`'s value (`docs/05` T79-P3).
fn drawn_under(ledger: &AnswerLedger, epoch: u64) -> Vec<identity::nym::Nym> {
    let state = ledger.state();
    let Round::Committing { group, .. } = &state.entry(0).unwrap().round else {
        panic!("{:?}", state.entry(0).unwrap().round);
    };
    let beacon = Beacon::from_outcome(ledger.beacon(epoch).unwrap()).unwrap();
    let size = state.config().rule.group() as usize;
    let seed = beacon.seed(ANSWER_DRAW, 0);
    assert_eq!(*group, lottery::admit(&state.pool()[..12], size, seed, 0));
    group.clone()
}

/// Three members under threshold `t`, member 2 withholding its reveals of epoch 0; twelve
/// reviewers join and a question is deposited, then members `0..t` request closing epoch 0.
async fn withheld(name: &str, t: usize) -> (Fixture, Vec<Handle>, Vec<Status>) {
    let f = Fixture::with(name, t);
    let net = f.net();
    let mut nodes = Vec::new();
    for i in 0..N {
        let faults = match i {
            2 => Faults::withholding_reveals([0]),
            _ => Faults::default(),
        };
        nodes.push(f.start_with(i, faults).await);
    }
    for (i, node) in nodes.iter().enumerate() {
        f.dial(node, i).await;
    }
    for (i, node) in nodes.iter().enumerate() {
        f.until(node, i, |s| {
            s.caught_up && s.circle_caught_up && s.cuts.len() >= 2
        })
        .await;
    }
    let issuer = issuer();
    let mut events: Vec<Vec<u8>> = (100..112)
        .map(|s| join(&net, &person(&issuer, s)))
        .collect();
    events.push(deposit(&net, &person(&issuer, 1)));
    for event in events {
        nodes[0].publish_circle(event).await.unwrap();
    }
    until_ledger(&f, &nodes[0], |l| {
        l.state().pool().len() == 12 && l.state().questions() == 1
    })
    .await;
    for i in 0..t {
        request(&f, i, ["close-epoch", "0"]);
    }
    let mut statuses = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        statuses.push(f.until(node, i, |s| !s.beacons.is_empty()).await);
    }
    (f, nodes, statuses)
}

// With fewer than `t` reveals round 0 has no value and excludes no one; its question stays
// pending, a closure of it refused, until round 1's value draws it; members agree (AT-T79-31).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_t79_31_a_withheld_beacon_waits_for_the_next_value() {
    let (f, nodes, statuses) = withheld("no-beacon", N).await;
    for s in &statuses {
        assert_eq!(s.beacons[0], (None, vec![2]), "{}", node::render(s));
        assert_eq!(s.digests[0], statuses[0].digests[0]);
    }
    let hook = ledger(&f, &nodes[0]).await;
    let state = hook.ledger().state();
    assert_eq!(state.entry(0).unwrap().round, Round::Pending);
    assert_eq!((state.epoch(), state.pool().len()), (1, 12));

    request(&f, 0, ["close-commits", "0"]);
    for i in 0..N {
        request(&f, i, ["close-epoch", "1"]);
    }
    let mut after = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        after.push(f.until(node, i, |s| s.beacons.len() >= 2).await);
    }
    let mut groups = Vec::new();
    for (s, node) in after.iter().zip(&nodes) {
        assert!(s.beacons[1].0.is_some() && s.beacons[1].1.is_empty());
        assert_eq!(s.beacons[..2], after[0].beacons[..2]);
        assert_eq!(s.digests[..2], after[0].digests[..2]);
        assert!(
            s.cuts.iter().all(|c| c.excluded.is_empty()),
            "no one is excluded"
        );
        assert!(s.refused_cut.is_none() && !s.duties_halted);
        let replica = node.replica().await;
        let (_, hook) = replayed(&f, &replica);
        groups.push(drawn_under(hook.ledger(), 1));
        let close = AnswerEvent::CloseCommits { question: 0 };
        let refusal = refused(hook.ledger(), &replica, 0, &close);
        assert_eq!(refusal, Refusal::Path(Refused::WrongRound));
    }
    assert!(groups.iter().all(|g| *g == groups[0]));
    for node in nodes {
        node.finished().await;
    }
}

// With `t` reveals round 0 has a value that draws the question, and its withholder's signature
// counts in no cut of epoch 1 after it, while its closure request does (AT-T79-32).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn at_t79_32_a_withholder_is_excluded_and_its_requests_count() {
    let (f, nodes, statuses) = withheld("withholder", T).await;
    let value = statuses[0].beacons[0].0.expect("round 0's value");
    for s in &statuses {
        assert_eq!(s.beacons[0], (Some(value), vec![2]), "{}", node::render(s));
        assert_eq!(s.digests[0], statuses[0].digests[0]);
    }
    let finished = statuses[0].digests[0].0 as usize + 1;

    request(&f, 1, ["close-epoch", "2"]);
    until_ledger(&f, &nodes[1], |l| {
        l.refused().iter().any(|r| r.1 == Refusal::WrongEpoch)
    })
    .await;
    request(&f, 2, ["close-epoch", "1"]);
    request(&f, 0, ["close-epoch", "1"]);
    let mut after = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        after.push(
            f.until(node, i, |s| {
                s.digests.len() >= 2 && s.cuts.len() > s.digests[1].0 as usize + 2
            })
            .await,
        );
    }
    let closing = after[0].digests[1].0 as usize;
    let early = AnswerEvent::RequestCloseEpoch { epoch: 2 };
    let mut groups = Vec::new();
    for (s, node) in after.iter().zip(&nodes) {
        assert_eq!(s.digests[..2], after[0].digests[..2], "{}", node::render(s));
        assert_eq!(s.beacons[..2], after[0].beacons[..2]);
        assert!(s.beacons[1].0.is_some() && s.beacons[1].1.is_empty());
        for (k, cut) in s.cuts.iter().enumerate() {
            let excluded = if k > finished && k <= closing {
                vec![2]
            } else {
                vec![]
            };
            assert_eq!(cut.excluded, excluded, "cut {k}");
        }
        let replica = node.replica().await;
        let (_, hook) = replayed(&f, &replica);
        groups.push(drawn_under(hook.ledger(), 0));
        assert_eq!(hook.ledger().state().epoch(), 2);
        assert_eq!(
            refused(hook.ledger(), &replica, 1, &early),
            Refusal::WrongEpoch
        );
    }
    assert!(groups.iter().all(|g| *g == groups[0]));
    assert_eq!(
        after[2].requests.len(),
        1,
        "the withholder's request is logged"
    );
    for node in nodes {
        node.finished().await;
    }
}
