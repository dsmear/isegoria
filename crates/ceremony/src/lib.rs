//! The issuing committee's ceremony tool (`docs/03` §Committee setup for the trial, T19-P4): one
//! member's directory, one step per invocation reading the files received and writing the files
//! to send, then issuance from the prepared signatures (P5) and abort (P6).

pub mod message;
pub mod procedure;

use ed25519_dalek::SigningKey;
use identity::credential::ceremony::{AnswerError, Party, Refusal as Flight, SlotRequest, FLIGHTS};
use message::{Header, HpkeSecret, MessageError};
use network::codec::{Reader, Writer};
use procedure::{Keys, Procedure, ProcedureError};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// The longest message, request or state file the tool reads.
pub const MAX_FILE: u64 = 64 << 20;
/// The step that takes the confirmations and ends the ceremony.
pub const FINISH: u8 = FLIGHTS + 1;
const MAX_INPUT: u64 = 1 << 20;
const STATE_VERSION: u8 = 1;
const STATE: &str = "state";
const OUT: &str = "out";
const ANSWERED: &str = "answered";

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub(crate) fn unhex(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// A file or directory, named by its role, could not be read or written.
    Io(&'static str, io::ErrorKind),
    Procedure(ProcedureError),
    /// The identity directory's keys are no member's of the procedure, or not this member's.
    NotMember,
    /// The state file is not one encoding, or the libraries failed on this member's own step.
    State,
    Internal,
    /// Another invocation holds the directory.
    Busy,
    Aborted,
    Finished,
    NotFinished,
    /// The step lacks these members' messages: nothing changed, run it again with them.
    Incomplete(Vec<u8>),
    /// A message (by its file) or the step was refused: the ceremony is aborted for this member.
    Refused(Option<PathBuf>, Refusal),
    NotSigner,
    /// The record of the slots answered is missing: this member's issuance has ended.
    Lost,
    Request,
    AlreadyAnswered(u32),
    Answer(AnswerError),
    /// The share's output file already exists.
    Exists,
}

/// Why a message or a step aborts the ceremony (`docs/03` T19-P3, P6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Message(MessageError),
    /// Of another ceremony, or of a step other than the one this step takes.
    Id,
    Step(u8),
    /// A second, different message from one sender in one step, or one the step does not take.
    Twice(u8),
    Unexpected(u8),
    /// The payload does not decode, or fails the library's check, or the key is no issuer key.
    Payload(Flight),
    /// A member's confirmation differs from this member's: another key or list.
    Confirmation(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Running,
    Done,
    Aborted,
}

#[derive(Clone)]
struct State {
    phase: Phase,
    me: u8,
    step: u8,
    procedure: Procedure,
    fresh: [u8; 32],
    id: Option<[u8; 32]>,
    party: Option<Party>,
    accepted: Vec<[u8; 32]>,
}

impl State {
    fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u8(STATE_VERSION)
            .u8(self.phase as u8)
            .u8(self.me)
            .u8(self.step)
            .field(&self.procedure.encode())
            .fixed(&self.fresh);
        match self.id {
            Some(id) => w.u8(1).fixed(&id),
            None => w.u8(0),
        };
        w.field(&self.party.as_ref().map(Party::encode).unwrap_or_default());
        w.u64(self.accepted.len() as u64);
        for digest in &self.accepted {
            w.fixed(digest);
        }
        w.finish()
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        let mut r = Reader::new(bytes);
        if r.u8().ok()? != STATE_VERSION {
            return None;
        }
        let phase = match r.u8().ok()? {
            0 => Phase::Running,
            1 => Phase::Done,
            2 => Phase::Aborted,
            _ => return None,
        };
        let (me, step) = (r.u8().ok()?, r.u8().ok()?);
        let procedure = Procedure::decode(r.field().ok()?)?;
        let fresh = r.fixed().ok()?;
        let id = match r.u8().ok()? {
            0 => None,
            1 => Some(r.fixed().ok()?),
            _ => return None,
        };
        let party = match r.field().ok()? {
            [] => None,
            party => Some(Party::decode(party)?),
        };
        let mut accepted = Vec::new();
        for _ in 0..r.u64().ok()?.min(bytes.len() as u64) {
            accepted.push(r.fixed().ok()?);
        }
        r.finish().ok()?;
        let held = party.as_ref().map(|p| (p.me(), p.id(), p.next()));
        let consistent = usize::from(me) < procedure.members.len()
            && match (phase, held) {
                (Phase::Aborted, None) => true,
                (Phase::Running, None) => step == 2 && id.is_none(),
                (Phase::Running, Some(h)) => (3..=FINISH).contains(&step) && h.2 == step,
                (Phase::Done, Some(h)) => step == FINISH + 1 && h.2 == FINISH,
                _ => false,
            }
            && held.is_none_or(|(m, i, _)| u8::try_from(m) == Ok(me) && Some(i) == id);
        let state = State {
            phase,
            me,
            step,
            procedure,
            fresh,
            id,
            party,
            accepted,
        };
        (consistent && state.encode() == bytes).then_some(state)
    }
}

/// A member's secrets from its `--new-identity` directory (`member.key`, `hpke.key`) and the
/// public keys they make, as the procedure lists them.
pub struct MemberKeys {
    pub signing: SigningKey,
    pub hpke: HpkeSecret,
    pub public: Keys,
}

pub fn member_keys(identity: &Path) -> Result<MemberKeys, Error> {
    let seed = |name: &'static str| {
        let bytes = read_bounded(&identity.join(name), name, 1024)?;
        let text = std::str::from_utf8(&bytes).ok();
        text.and_then(unhex)
            .ok_or(Error::Io(name, io::ErrorKind::InvalidData))
    };
    let signing = SigningKey::from_bytes(&seed("member.key")?);
    let (hpke, key) = message::hpke_pair(&seed("hpke.key")?);
    let public = Keys {
        consortium: signing.verifying_key().to_bytes(),
        hpke: key,
    };
    Ok(MemberKeys {
        signing,
        hpke,
        public,
    })
}

fn io(role: &'static str) -> impl Fn(io::Error) -> Error {
    move |e| Error::Io(role, e.kind())
}

fn read_bounded(path: &Path, role: &'static str, max: u64) -> Result<Vec<u8>, Error> {
    let file = std::fs::File::open(path).map_err(io(role))?;
    let mut bytes = Vec::new();
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(io(role))?;
    if bytes.len() as u64 > max {
        return Err(Error::Io(role, io::ErrorKind::InvalidData));
    }
    Ok(bytes)
}

#[cfg(unix)]
fn create_private(path: &Path) -> io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_private(path: &Path) -> io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

#[cfg(unix)]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().mode(0o700).create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &Path) -> io::Result<()> {
    std::fs::create_dir(dir)
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    std::fs::File::open(dir)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = dir;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = create_private(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Replaces `dir/name` whole: a new private file beside it, synced, renamed over it.
fn replace(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let staged = dir.join(format!("{name}.new"));
    match std::fs::remove_file(&staged) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    write_new(&staged, bytes)?;
    std::fs::rename(&staged, dir.join(name))?;
    sync_dir(dir)
}

/// An exclusive advisory lock on the directory's `lock` file, released when dropped.
struct Lock {
    _file: std::fs::File,
}

fn lock(dir: &Path) -> Result<Lock, Error> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("lock"))
        .map_err(io("lock"))?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        // SAFETY: `flock` on a descriptor this function owns.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let e = io::Error::last_os_error();
            return Err(match e.raw_os_error() {
                Some(libc::EWOULDBLOCK) => Error::Busy,
                _ => Error::Io("lock", e.kind()),
            });
        }
    }
    Ok(Lock { _file: file })
}

fn save(dir: &Path, state: &State) -> Result<(), Error> {
    replace(dir, STATE, &state.encode()).map_err(io("state"))
}

fn load(dir: &Path) -> Result<State, Error> {
    let bytes = read_bounded(&dir.join(STATE), "state", MAX_FILE)?;
    State::decode(&bytes).ok_or(Error::State)
}

/// Aborts the ceremony for this member: its party, share included, is dropped from the state.
fn refuse(dir: &Path, mut state: State, file: Option<&Path>, why: Refusal) -> Error {
    state.phase = Phase::Aborted;
    state.party = None;
    match save(dir, &state) {
        Ok(()) => Error::Refused(file.map(Path::to_path_buf), why),
        Err(e) => e,
    }
}

/// What step `step` sent, by file, and what it learned.
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    pub step: u8,
    pub id: Option<[u8; 32]>,
    pub sent: Vec<PathBuf>,
    pub issuer: Option<Vec<u8>>,
}

impl Report {
    /// `step <n>`, `ceremony <id>` once known, a `sent <file>` line per file, then after the last
    /// step `done` and `issuer <hex>`, the value `--genesis-assemble` takes.
    pub fn render(&self) -> String {
        let mut out = format!("step {}\n", self.step);
        if let Some(id) = self.id {
            out += &format!("ceremony {}\n", hex(&id));
        }
        for path in &self.sent {
            out += &format!("sent {}\n", path.display());
        }
        if let Some(issuer) = &self.issuer {
            out += &format!("done\nissuer {}\n", hex(issuer));
        }
        out
    }
}

fn send(
    dir: &Path,
    state: &State,
    keys: &MemberKeys,
    step: u8,
    id: [u8; 32],
    outgoing: &[(u16, Vec<u8>)],
) -> Result<Vec<PathBuf>, Error> {
    let out = dir.join(OUT);
    let mut sent = Vec::new();
    for (to, payload) in outgoing {
        let to = u8::try_from(*to).map_err(|_| Error::Internal)?;
        let header = Header {
            id,
            step,
            sender: state.me,
            recipient: to,
        };
        let recipient = &state.procedure.members[usize::from(to)].hpke;
        let bytes = message::seal(&header, payload, &keys.signing, recipient);
        let bytes = bytes.ok_or(Error::Procedure(ProcedureError::Key(format!(
            "member[{to}].hpke"
        ))))?;
        let name = format!("{step:02}-{}-{to}.msg", state.me);
        replace(&out, &name, &bytes).map_err(io("message"))?;
        sent.push(out.join(name));
    }
    Ok(sent)
}

/// The confirmation (step 13): the issuer key's 97 bytes, the slots' digest, the signers.
fn confirmation(party: &Party, procedure: &Procedure) -> Result<Vec<u8>, Error> {
    let mut out = party.issuer().ok_or(Error::Internal)?.encode();
    out.extend_from_slice(&procedure.slots_digest());
    out.push(u8::try_from(procedure.signers.len()).map_err(|_| Error::Internal)?);
    for &s in &procedure.signers {
        out.push(u8::try_from(s).map_err(|_| Error::Internal)?);
    }
    Ok(out)
}

/// Step 1: a new directory `dir` for the member whose keys are in `identity`, its fresh value
/// drawn from the system's randomness and sent to every other member.
pub fn init(procedure: &Path, identity: &Path, dir: &Path) -> Result<Report, Error> {
    let text = read_bounded(procedure, "procedure", MAX_INPUT)?;
    let text = String::from_utf8(text)
        .map_err(|e| Error::Procedure(ProcedureError::Parse(e.to_string())))?;
    let procedure = Procedure::parse(&text).map_err(Error::Procedure)?;
    let setup = procedure.setup().ok_or(Error::Internal)?;
    let keys = member_keys(identity)?;
    let me = procedure.member(&keys.public).ok_or(Error::NotMember)?;
    create_private_dir(dir).map_err(io("ceremony directory"))?;
    create_private_dir(&dir.join(OUT)).map_err(io("out"))?;
    let _lock = lock(dir)?;
    let mut fresh = [0u8; 32];
    getrandom::getrandom(&mut fresh).map_err(|_| Error::Io("randomness", io::ErrorKind::Other))?;
    let outgoing: Vec<_> = (setup.recipients(1, u16::from(me)).into_iter())
        .map(|j| (j, fresh.to_vec()))
        .collect();
    let opening = procedure.opening();
    let state = State {
        phase: Phase::Running,
        me,
        step: 2,
        procedure,
        fresh,
        id: None,
        party: None,
        accepted: Vec::new(),
    };
    let sent = send(dir, &state, &keys, 1, opening, &outgoing)?;
    save(dir, &state)?;
    if let Some(parent) = dir.parent().filter(|p| !p.as_os_str().is_empty()) {
        sync_dir(parent).map_err(io("ceremony directory"))?;
    }
    Ok(Report {
        step: 1,
        id: None,
        sent,
        issuer: None,
    })
}

/// Steps 2 to 14: takes the previous step's messages from `files`, each to this member, and
/// writes this step's into `out/`; a byte-identical file already taken is skipped.
pub fn step(dir: &Path, identity: &Path, files: &[PathBuf]) -> Result<Report, Error> {
    let _lock = lock(dir)?;
    let state = load(dir)?;
    match state.phase {
        Phase::Aborted => return Err(Error::Aborted),
        Phase::Done => return Err(Error::Finished),
        Phase::Running => {}
    }
    let keys = member_keys(identity)?;
    if state.procedure.members[usize::from(state.me)] != keys.public {
        return Err(Error::NotMember);
    }
    let setup = state.procedure.setup().ok_or(Error::State)?;
    let me = u16::from(state.me);
    let flight = state.step - 1;
    let id = match flight {
        1 => state.procedure.opening(),
        _ => state.id.ok_or(Error::State)?,
    };
    let members = state.procedure.consortium_keys();
    let (mut received, mut paths, mut digests) = (BTreeMap::new(), BTreeMap::new(), Vec::new());
    for path in files {
        let bytes = read_bounded(path, "message", MAX_FILE)?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        if state.accepted.contains(&digest) || digests.contains(&digest) {
            continue;
        }
        let refused = |state, why| Err(refuse(dir, state, Some(path), why));
        let (header, payload) = match message::open(&bytes, state.me, &keys.hpke, &members) {
            Ok(opened) => opened,
            Err(e) => return refused(state, Refusal::Message(e)),
        };
        let sender = u16::from(header.sender);
        if header.id != id {
            return refused(state, Refusal::Id);
        }
        if header.step != flight {
            return refused(state, Refusal::Step(header.step));
        }
        if !setup.sends(flight, sender, me) {
            return refused(state, Refusal::Unexpected(header.sender));
        }
        if received.insert(sender, payload).is_some() {
            return refused(state, Refusal::Twice(header.sender));
        }
        paths.insert(sender, path.clone());
        digests.push(digest);
    }
    let senders = setup.senders(flight, me).into_iter();
    let missing: Vec<u8> = senders
        .filter(|j| !received.contains_key(j))
        .map(|j| j as u8)
        .collect();
    if !missing.is_empty() {
        return Err(Error::Incomplete(missing));
    }
    let mut next = state.clone();
    let mut issuer = None;
    let outgoing = match state.step {
        2 => {
            let mut fresh = Vec::new();
            for j in 0..setup.members() {
                match received.get(&j).map(|v| v.as_slice().try_into()) {
                    None => fresh.push(state.fresh),
                    Some(Ok(value)) => fresh.push(value),
                    Some(Err(_)) => {
                        let why = Refusal::Payload(Flight::Decode(j));
                        return Err(refuse(dir, state, paths.get(&j).map(PathBuf::as_path), why));
                    }
                }
            }
            let id = state.procedure.id(&fresh);
            let party = Party::new(setup.clone(), me, id).ok_or(Error::Internal)?;
            let (party, out) = party
                .advance(&BTreeMap::new())
                .map_err(|_| Error::Internal)?;
            next.id = Some(id);
            next.party = Some(party);
            out
        }
        3..=FLIGHTS => {
            let party = state.party.as_ref().ok_or(Error::State)?;
            match party.advance(&received) {
                Ok((party, mut out)) => {
                    if state.step == FLIGHTS {
                        let confirm = confirmation(&party, &state.procedure)?;
                        let to = setup.recipients(FLIGHTS, me).into_iter();
                        out = to.map(|j| (j, confirm.clone())).collect();
                    }
                    next.party = Some(party);
                    out
                }
                Err(Flight::Missing(_) | Flight::Finished | Flight::Internal) => {
                    return Err(Error::Internal)
                }
                Err(e) => {
                    let blamed = match e {
                        Flight::Unexpected(j) | Flight::Decode(j) | Flight::Library(j) => {
                            paths.get(&j).map(PathBuf::as_path)
                        }
                        _ => None,
                    };
                    return Err(refuse(dir, state, blamed, Refusal::Payload(e)));
                }
            }
        }
        FINISH => {
            let party = state.party.as_ref().ok_or(Error::State)?;
            let own = confirmation(party, &state.procedure)?;
            if let Some((&j, _)) = received.iter().find(|(_, payload)| **payload != own) {
                let file = paths.get(&j).map(PathBuf::as_path);
                return Err(refuse(dir, state, file, Refusal::Confirmation(j as u8)));
            }
            if setup.is_signer(me) {
                match create_private_dir(&dir.join(ANSWERED)) {
                    Err(e) if e.kind() != io::ErrorKind::AlreadyExists => {
                        return Err(Error::Io("record", e.kind()))
                    }
                    _ => sync_dir(dir).map_err(io("record"))?,
                }
            }
            next.phase = Phase::Done;
            issuer = party.issuer().map(|i| i.encode());
            Vec::new()
        }
        _ => return Err(Error::State),
    };
    let sent = send(
        dir,
        &next,
        &keys,
        state.step,
        next.id.ok_or(Error::State)?,
        &outgoing,
    )?;
    next.step += 1;
    next.accepted.extend(digests);
    save(dir, &next)?;
    Ok(Report {
        step: state.step,
        id: next.id,
        sent,
        issuer,
    })
}

/// A member directory's state, as `status` prints it.
#[derive(Debug)]
pub struct Summary {
    pub member: u8,
    pub phase: &'static str,
    pub step: u8,
    pub id: Option<[u8; 32]>,
    /// The members whose messages the next step needs.
    pub expects: Vec<u8>,
    pub issuer: Option<Vec<u8>>,
    pub answered: Option<usize>,
}

impl Summary {
    /// `member`, `phase` (`running`, `done`, `aborted`), `next <step>` and `expects <members>`
    /// while running, `ceremony <id>`, `issuer <hex>` and a signer's `answered <count>`.
    pub fn render(&self) -> String {
        let mut out = format!("member {}\nphase {}\n", self.member, self.phase);
        if self.phase == "running" {
            let list: Vec<String> = self.expects.iter().map(u8::to_string).collect();
            out += &format!("next {}\nexpects {}\n", self.step, list.join(" "));
        }
        if let Some(id) = self.id {
            out += &format!("ceremony {}\n", hex(&id));
        }
        if let Some(issuer) = &self.issuer {
            out += &format!("issuer {}\n", hex(issuer));
        }
        if let Some(count) = self.answered {
            out += &format!("answered {count}\n");
        }
        out
    }
}

pub fn status(dir: &Path) -> Result<Summary, Error> {
    let _lock = lock(dir)?;
    let state = load(dir)?;
    let setup = state.procedure.setup().ok_or(Error::State)?;
    let (me, done) = (u16::from(state.me), state.phase == Phase::Done);
    let expects = match state.phase {
        Phase::Running => setup.senders(state.step - 1, me),
        _ => Vec::new(),
    };
    let answered = match done && setup.is_signer(me) {
        true => Some(std::fs::read_dir(dir.join(ANSWERED)).map_or(0, |d| d.count())),
        false => None,
    };
    Ok(Summary {
        member: state.me,
        phase: match state.phase {
            Phase::Running => "running",
            Phase::Done => "done",
            Phase::Aborted => "aborted",
        },
        step: state.step,
        id: state.id,
        expects: expects.into_iter().map(|j| j as u8).collect(),
        issuer: state
            .party
            .as_ref()
            .and_then(Party::issuer)
            .map(|i| i.encode()),
        answered,
    })
}

/// Issuance (`docs/03` T19-P5): this signer's share for the request in `request`, written to
/// `share`, a new file, after the slot is recorded as answered in `answered/<slot>`.
pub fn answer(dir: &Path, request: &Path, share: &Path) -> Result<u32, Error> {
    let _lock = lock(dir)?;
    let state = load(dir)?;
    match state.phase {
        Phase::Aborted => return Err(Error::Aborted),
        Phase::Running => return Err(Error::NotFinished),
        Phase::Done => {}
    }
    let setup = state.procedure.setup().ok_or(Error::State)?;
    if !setup.is_signer(u16::from(state.me)) {
        return Err(Error::NotSigner);
    }
    let record = dir.join(ANSWERED);
    if !record.is_dir() {
        return Err(Error::Lost);
    }
    if share.symlink_metadata().is_ok() {
        return Err(Error::Exists);
    }
    let bytes = read_bounded(request, "request", MAX_INPUT)?;
    let request = SlotRequest::decode(&bytes).ok_or(Error::Request)?;
    let slot = request.slot();
    let entry = record.join(slot.to_string());
    if entry.symlink_metadata().is_ok() {
        return Err(Error::AlreadyAnswered(slot));
    }
    let party = state.party.as_ref().ok_or(Error::State)?;
    let value = party.answer(&request).map_err(Error::Answer)?;
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    match write_new(&entry, &digest) {
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return Err(Error::AlreadyAnswered(slot))
        }
        written => written.map_err(io("record"))?,
    }
    sync_dir(&record).map_err(io("record"))?;
    write_new(share, &value.encode()).map_err(io("share"))?;
    Ok(slot)
}

/// The operators' decision (`docs/03` T19-P6): the ceremony ends for this member, its share
/// and state dropped; nothing of it is used again.
pub fn abort(dir: &Path) -> Result<(), Error> {
    let _lock = lock(dir)?;
    let mut state = load(dir)?;
    state.phase = Phase::Aborted;
    state.party = None;
    save(dir, &state)
}
