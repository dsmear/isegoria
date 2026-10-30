//! A node as an operating-system process (`docs/04` §A node process, `docs/10` T78): its
//! configuration, its files, its status, and the loop that runs it until it is stopped.

pub mod archive;
pub mod install;
pub mod production;

use ed25519_dalek::VerifyingKey;
use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use network::consortium::{Consortium, Member};
use network::replica::{FeedWriter, WriterSet};
use network::tier::{TierError, Tiers};
use p2p::blind::{Gateway, GatewayKey, KeyAnnouncement};
use p2p::member::MemberRole;
use p2p::{Config as P2p, Handle, Own, StartError, Status};
use serde::Deserialize;
use std::collections::HashSet;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub network_id: String,
    pub data_dir: PathBuf,
    pub listen: Vec<String>,
    #[serde(default)]
    pub peers: Vec<String>,
    pub writers: Vec<String>,
    pub consortium: ConsortiumConfig,
    /// The writer's 32-byte seed in hex; a member's key is its writer key.
    pub key_file: Option<PathBuf>,
    /// A tiered member's circle seed in hex (`docs/04` §A node process).
    pub circle_key_file: Option<PathBuf>,
    /// A tiered member's HPKE seed in hex: its gateway (`docs/04` §Blind relays).
    pub hpke_key_file: Option<PathBuf>,
    /// The relays' transport keys a tiered member's gateway answers.
    #[serde(default)]
    pub relays: Vec<String>,
    /// Whether this tiered non-member forwards persons' requests as a relay.
    #[serde(default)]
    pub relay: bool,
    pub member: Option<MemberConfig>,
    pub archive_maintenance: Option<archive::Config>,
    #[serde(default = "default_sync")]
    pub sync_every_ms: u64,
    #[serde(default = "default_status")]
    pub status_every_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsortiumConfig {
    pub members: Vec<String>,
    pub threshold: usize,
    /// The members' circle keys in member order: a tiered network when not empty.
    #[serde(default)]
    pub circle: Vec<String>,
    /// The members' signed OHTTP key configurations in member order, in hex.
    #[serde(default)]
    pub hpke: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberConfig {
    pub index: usize,
    pub every_ms: u64,
    pub cuts_per_epoch: u64,
    pub patience: u32,
    pub view_patience: u32,
}

fn default_sync() -> u64 {
    1000
}

fn default_status() -> u64 {
    1000
}

/// Why a configuration does not start a node.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    Read(io::ErrorKind),
    Bounds(&'static str),
    Archive(&'static str),
    ArchiveContext(network::archive::Error),
    Parse(String),
    /// A field that is not 64 hex digits: the field's name.
    Hex(&'static str),
    /// A key that is no ed25519 public key, or repeated among the members.
    Key(&'static str),
    /// A threshold outside `1..=n`.
    Threshold,
    Address(String),
    MemberWithoutKey,
    Tiers(TierError),
    /// `writers` is not exactly the tiered set's members' and circle keys.
    Writers,
    /// A tiered member without its circle key, or a circle key file elsewhere or of another key.
    CircleKey,
    /// A relay or gateway setting that does not hold: the field.
    Relays(&'static str),
}

impl Config {
    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        let bytes = archive::read_bounded(path, archive::MAX_CONFIG, "config")?;
        let text = String::from_utf8(bytes).map_err(|e| ConfigError::Parse(e.to_string()))?;
        toml::from_str(&text).map_err(|e| ConfigError::Parse(e.to_string()))
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str, field: &'static str) -> Result<[u8; 32], ConfigError> {
    let text = text.trim();
    if text.len() != 64 || !text.is_ascii() {
        return Err(ConfigError::Hex(field));
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b =
            u8::from_str_radix(&text[2 * i..2 * i + 2], 16).map_err(|_| ConfigError::Hex(field))?;
    }
    Ok(out)
}

fn keys(list: &[String], field: &'static str) -> Result<Vec<VerifyingKey>, ConfigError> {
    list.iter()
        .map(|k| VerifyingKey::from_bytes(&unhex(k, field)?).map_err(|_| ConfigError::Key(field)))
        .collect()
}

fn addresses(list: &[String]) -> Result<Vec<Multiaddr>, ConfigError> {
    list.iter()
        .map(|a| a.parse().map_err(|_| ConfigError::Address(a.clone())))
        .collect()
}

/// The node's existing libp2p key; none is made.
fn stored_transport_key(path: &Path) -> Result<Keypair, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.kind()))?;
    Keypair::ed25519_from_bytes(unhex(&text, "transport.key")?)
        .map_err(|_| ConfigError::Key("transport.key"))
}

/// The node's libp2p key: its seed kept in `path`, made at the first start.
fn transport_key(path: &Path) -> Result<Keypair, ConfigError> {
    let read = |e: io::Error| ConfigError::Read(e.kind());
    if path.exists() {
        return stored_transport_key(path);
    }
    let key = Keypair::generate_ed25519();
    let pair = key
        .clone()
        .try_into_ed25519()
        .map_err(|_| ConfigError::Key("transport.key"))?;
    write_private(path, &hex(pair.secret().as_ref())).map_err(read)?;
    Ok(key)
}

#[cfg(unix)]
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(text.as_bytes())?;
    f.sync_all()
}

#[cfg(not(unix))]
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    std::fs::write(path, text)
}

/// An exclusive advisory lock on a data directory's `lock` file, released when dropped.
struct DataLock {
    _file: std::fs::File,
}

/// The lock, or `None` while another process holds it; enforced on Unix only (`docs/04`).
fn lock(path: &Path) -> io::Result<Option<DataLock>> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: `flock` on a descriptor this function owns.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let e = io::Error::last_os_error();
            return match e.raw_os_error() {
                Some(libc::EWOULDBLOCK) => Ok(None),
                _ => Err(e),
            };
        }
    }
    Ok(Some(DataLock { _file: file }))
}

/// What a checked configuration starts.
pub struct Plan {
    pub transport: Keypair,
    pub p2p: P2p,
    pub listen: Vec<Multiaddr>,
    pub peers: Vec<Multiaddr>,
    pub status: PathBuf,
    pub status_every: Duration,
    pub archive_maintenance: Option<archive::Plan>,
}

/// Checks `config` and prepares its directory; nothing runs yet.
pub fn plan(config: Config) -> Result<Plan, ConfigError> {
    let network_id = unhex(&config.network_id, "network_id")?;
    let writers = keys(&config.writers, "writers")?;
    let members = keys(&config.consortium.members, "consortium.members")?;
    let distinct: HashSet<[u8; 32]> = members.iter().map(|k| k.to_bytes()).collect();
    if distinct.len() != members.len() {
        return Err(ConfigError::Key("consortium.members"));
    }
    let t = config.consortium.threshold;
    if !(1..=members.len()).contains(&t) {
        return Err(ConfigError::Threshold);
    }
    let seed_of = |path: &Option<PathBuf>, field| match path {
        Some(path) => {
            let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.kind()))?;
            unhex(&text, field).map(Some)
        }
        None => Ok(None),
    };
    let seed = seed_of(&config.key_file, "key_file")?;
    let circle_seed = seed_of(&config.circle_key_file, "circle_key_file")?;
    let circle = keys(&config.consortium.circle, "consortium.circle")?;
    let tiers = tiers_of(&config)?;
    let member = match (config.member.as_ref(), seed) {
        (None, _) => None,
        (Some(_), None) => return Err(ConfigError::MemberWithoutKey),
        (Some(m), Some(seed)) => Some(MemberRole {
            member: Member::from_seed(seed),
            index: m.index,
            consortium: Consortium::new(members.clone(), t),
            every: Duration::from_millis(m.every_ms),
            cuts_per_epoch: m.cuts_per_epoch,
            patience: m.patience,
            view_patience: m.view_patience,
        }),
    };
    let own_circle = |index: usize| {
        let seed = circle_seed.ok_or(ConfigError::CircleKey)?;
        let writer = FeedWriter::from_seed(network_id, seed);
        if circle.get(index) != Some(&writer.public()) {
            return Err(ConfigError::CircleKey);
        }
        Ok(Own {
            writer,
            dir: config.data_dir.join("circle"),
        })
    };
    let circle_own = match (&tiers, &member) {
        (Some(_), Some(role)) => Some(own_circle(role.index)?),
        _ if circle_seed.is_some() => return Err(ConfigError::CircleKey),
        _ => None,
    };
    let (relay, gateway) = blind_plan(&config, tiers.as_ref(), member.as_ref())?;
    let archive_maintenance = config
        .archive_maintenance
        .as_ref()
        .map(|a| archive::plan(a, tiers_of(&config)?))
        .transpose()?;
    let listen = addresses(&config.listen)?;
    let peers = addresses(&config.peers)?;
    std::fs::create_dir_all(&config.data_dir).map_err(|e| ConfigError::Read(e.kind()))?;
    let transport = transport_key(&config.data_dir.join("transport.key"))?;
    let p2p = P2p {
        writers: WriterSet::new(network_id, &writers),
        sync_every: Duration::from_millis(config.sync_every_ms),
        own: seed.map(|seed| Own {
            writer: FeedWriter::from_seed(network_id, seed),
            dir: config.data_dir.join("own"),
        }),
        circle: circle_own,
        store: Some(config.data_dir.join("replica")),
        consortium: (member.is_none() && tiers.is_none()).then(|| Consortium::new(members, t)),
        member,
        tiers,
        relay,
        gateway,
    };
    Ok(Plan {
        transport,
        p2p,
        listen,
        peers,
        archive_maintenance,
        status: config.data_dir.join("status"),
        status_every: Duration::from_millis(config.status_every_ms),
    })
}

/// The tiered set a configuration declares, if any: its circle keys and `writers` checked.
fn tiers_of(config: &Config) -> Result<Option<Tiers>, ConfigError> {
    let circle = keys(&config.consortium.circle, "consortium.circle")?;
    if circle.is_empty() {
        return Ok(None);
    }
    let network_id = unhex(&config.network_id, "network_id")?;
    let members = keys(&config.consortium.members, "consortium.members")?;
    let t = config.consortium.threshold;
    if !(1..=members.len()).contains(&t) {
        return Err(ConfigError::Threshold);
    }
    let tiers =
        Tiers::new(network_id, Consortium::new(members, t), &circle).map_err(ConfigError::Tiers)?;
    let writers = keys(&config.writers, "writers")?;
    if WriterSet::new(network_id, &writers) != *tiers.writers() {
        return Err(ConfigError::Writers);
    }
    Ok(Some(tiers))
}

fn bytes_of(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if text.len() % 2 != 0 || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// The members' key configurations, each verified as its position's member's.
fn announcements(tiers: &Tiers, list: &[String]) -> Result<Vec<KeyAnnouncement>, ConfigError> {
    list.iter()
        .enumerate()
        .map(|(i, text)| {
            let a = bytes_of(text).and_then(|b| KeyAnnouncement::decode(&b));
            a.filter(|a| a.member == i && a.verify(tiers))
                .ok_or(ConfigError::Relays("consortium.hpke"))
        })
        .collect()
}

/// A member's gateway key from its `hpke_key_file`.
fn gateway_key(tiers: &Tiers, role: &MemberRole, path: &Path) -> Result<GatewayKey, ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.kind()))?;
    let ikm = unhex(&text, "hpke_key_file")?;
    GatewayKey::new(tiers, &role.member, role.index, &ikm)
        .ok_or(ConfigError::Relays("hpke_key_file"))
}

/// The relay's routes and the member's gateway a configuration asks for; the gateway admits
/// nothing, since no admission rule exists before `docs/10` T79.
fn blind_plan(
    config: &Config,
    tiers: Option<&Tiers>,
    member: Option<&MemberRole>,
) -> Result<(Option<Vec<KeyAnnouncement>>, Option<Gateway>), ConfigError> {
    let asked = config.relay
        || config.hpke_key_file.is_some()
        || !config.relays.is_empty()
        || !config.consortium.hpke.is_empty();
    let Some(tiers) = tiers.filter(|_| asked) else {
        return match asked {
            true => Err(ConfigError::Relays("consortium.circle")),
            false => Ok((None, None)),
        };
    };
    let routes = announcements(tiers, &config.consortium.hpke)?;
    let relay = match config.relay {
        true if member.is_none()
            && config.key_file.is_none()
            && routes.len() == tiers.consortium().keys().len() =>
        {
            Some(routes.clone())
        }
        true => return Err(ConfigError::Relays("relay")),
        false => None,
    };
    let gateway = match (&config.hpke_key_file, member) {
        (None, _) if !config.relays.is_empty() => return Err(ConfigError::Relays("relays")),
        (None, _) => None,
        (Some(_), None) => return Err(ConfigError::Relays("hpke_key_file")),
        (Some(path), Some(role)) => {
            let key = gateway_key(tiers, role, path)?;
            if routes.get(role.index) != Some(key.announcement()) {
                return Err(ConfigError::Relays("consortium.hpke"));
            }
            let relays = config
                .relays
                .iter()
                .map(|k| unhex(k, "relays"))
                .collect::<Result<Vec<_>, _>>()?;
            let admit: p2p::blind::Admission = Box::new(|_| false);
            Some(Gateway { key, relays, admit })
        }
    };
    Ok((relay, gateway))
}

/// A member's signed OHTTP key configuration in hex, for the shared `consortium.hpke`.
pub fn announcement(config: &Config) -> Result<String, ConfigError> {
    let tiers = tiers_of(config)?.ok_or(ConfigError::Relays("consortium.circle"))?;
    let (Some(m), Some(path)) = (&config.member, &config.key_file) else {
        return Err(ConfigError::MemberWithoutKey);
    };
    let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read(e.kind()))?;
    let role = MemberRole {
        member: Member::from_seed(unhex(&text, "key_file")?),
        index: m.index,
        consortium: Consortium::new(
            tiers.consortium().keys().to_vec(),
            tiers.consortium().threshold(),
        ),
        every: Duration::from_millis(m.every_ms),
        cuts_per_epoch: m.cuts_per_epoch,
        patience: m.patience,
        view_patience: m.view_patience,
    };
    let path = config
        .hpke_key_file
        .as_ref()
        .ok_or(ConfigError::Relays("hpke_key_file"))?;
    Ok(hex(&gateway_key(&tiers, &role, path)?
        .announcement()
        .encode()))
}

/// The status file's text: one line per fact, cuts and rounds in order.
pub fn render(status: &Status) -> String {
    let mut out = format!(
        "caught_up {}\ncircle_caught_up {}\npeers {}\nbound {}\nequivocations {}\nconflict {}\n",
        status.caught_up,
        status.circle_caught_up,
        status.peers,
        status.bound,
        status.equivocations,
        status.conflict
    );
    for (n, digest) in status.cuts.iter().enumerate() {
        out += &format!("cut {n} {}\n", hex(digest));
    }
    for (e, (value, withheld)) in status.beacons.iter().enumerate() {
        let value = value.map_or("none".to_string(), |v| hex(&v));
        let withheld: Vec<String> = withheld.iter().map(|m| m.to_string()).collect();
        out += &format!("beacon {e} {value} [{}]\n", withheld.join(","));
    }
    out
}

/// The status a status file holds, `None` if the text is not one [`render`] writes.
pub fn parse(text: &str) -> Option<Status> {
    let mut status = Status::default();
    for line in text.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        match fields[..] {
            ["caught_up", v] => status.caught_up = v.parse().ok()?,
            ["circle_caught_up", v] => status.circle_caught_up = v.parse().ok()?,
            ["peers", v] => status.peers = v.parse().ok()?,
            ["bound", v] => status.bound = v.parse().ok()?,
            ["equivocations", v] => status.equivocations = v.parse().ok()?,
            ["conflict", v] => status.conflict = v.parse().ok()?,
            ["cut", n, d] => {
                (n.parse::<usize>().ok()? == status.cuts.len()).then_some(())?;
                status.cuts.push(unhex(d, "cut").ok()?);
            }
            ["beacon", e, v, w] => {
                (e.parse::<usize>().ok()? == status.beacons.len()).then_some(())?;
                let value = if v == "none" {
                    None
                } else {
                    Some(unhex(v, "beacon").ok()?)
                };
                let list = w.strip_prefix('[')?.strip_suffix(']')?;
                let withheld = list
                    .split(',')
                    .filter(|m| !m.is_empty())
                    .map(|m| m.parse().ok())
                    .collect::<Option<Vec<usize>>>()?;
                status.beacons.push((value, withheld));
            }
            _ => return None,
        }
    }
    Some(status)
}

/// Writes the status beside its file, then renames it over it.
pub fn write_status(path: &Path, status: &Status) -> io::Result<()> {
    let next = path.with_extension("next");
    std::fs::write(&next, render(status))?;
    std::fs::rename(next, path)
}

async fn dial(handle: &Handle, peers: &[Multiaddr]) {
    for addr in peers {
        let _ = handle.dial(addr.clone()).await;
    }
}

/// Runs the node until `stop` completes: listens, dials its peers, dials them again
/// whenever it has none, and writes its status on each period.
pub async fn run(plan: Plan, stop: impl Future<Output = ()>) -> Result<(), StartError> {
    let _lock = lock(&plan.status.with_file_name("lock"))
        .map_err(|e| StartError::Transport(format!("data directory lock: {e}")))?
        .ok_or_else(|| StartError::Transport("data directory in use".to_string()))?;
    let archive_path = plan.status.with_file_name("archive-status");
    archive::write_status(&archive_path, plan.archive_maintenance.is_some(), None)
        .map_err(|e| StartError::Transport(format!("archive maintenance status: {e}")))?;
    let mut handle = Handle::spawn(plan.transport, plan.p2p)?;
    for path in handle.set_aside() {
        eprintln!("set aside: {}", path.display());
    }
    for addr in plan.listen {
        let bound = handle.listen(addr).await.map_err(StartError::Transport)?;
        eprintln!("listening on {bound}/p2p/{}", handle.peer_id());
    }
    dial(&handle, &plan.peers).await;
    let monitor = handle.monitor();
    let (stop_archive, stopped_archive) = tokio::sync::oneshot::channel();
    let (reports, mut latest) = tokio::sync::watch::channel(None);
    let jobs = plan.archive_maintenance.as_ref().map(archive::Plan::jobs);
    let runner = async {
        if let (Some(archive), Some(jobs)) = (&plan.archive_maintenance, &jobs) {
            handle
                .run_archive_maintenance(jobs, archive.schedule, stopped_archive, reports)
                .await
        } else {
            std::future::pending().await
        }
    };
    tokio::pin!(runner);
    let mut tick = tokio::time::interval(plan.status_every);
    tokio::pin!(stop);
    let result = loop {
        tokio::select! {
            _ = &mut stop => {
                let _ = stop_archive.send(());
                break if plan.archive_maintenance.is_some() { runner.await } else { Ok(()) };
            },
            result = &mut runner => break result,
            changed = latest.changed(), if plan.archive_maintenance.is_some() => {
                if changed.is_ok() {
                    if let Err(e) = archive::write_status(&archive_path, true, latest.borrow().as_ref()) {
                        eprintln!("archive maintenance status not written: {e}");
                    }
                }
            },
            _ = tick.tick() => {
                let status = monitor.status().await;
                if let Err(e) = write_status(&plan.status, &status) {
                    eprintln!("status not written: {e}");
                }
                if status.peers == 0 {
                    for address in &plan.peers {
                        let _ = monitor.dial(address.clone()).await;
                    }
                }
            }
        }
    };
    let _ = write_status(&plan.status, &monitor.status().await);
    if let Err(e) = archive::write_status(
        &archive_path,
        plan.archive_maintenance.is_some(),
        latest.borrow().as_ref(),
    ) {
        eprintln!("archive maintenance status not written: {e}");
    }
    result.map_err(|e| StartError::Transport(format!("local archive maintenance: {e:?}")))
}
