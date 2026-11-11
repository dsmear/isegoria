//! `isegoria-client`, a participant's tool: its credential by files, its events and read
//! requests sealed to a member (`docs/05` §Distributed integration, §Built for participants).

pub mod credential;
pub mod event;
pub mod read;

use identity::credential::ceremony::CombineError;
use identity::credential::IssuerPublic;
use network::genesis::Genesis;
use network::tier::Tiers;
use p2p::blind::{Answer, ClientError};
use protocol::answer_gateway::Inadmissible;
use protocol::answer_path::AnswerConfig;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// The named file could not be read or written; an output that exists is `AlreadyExists`.
    Io(&'static str, io::ErrorKind),
    /// The named file or argument is not its kind's one encoding.
    Malformed(&'static str),
    /// The credential does not verify under the issuer key given.
    Unverified,
    Combine(CombineError),
    /// The event is one a member's gateway would not log.
    Inadmissible(Inadmissible),
    /// The relay or key configuration is not the genesis's.
    Unlisted(&'static str),
    Channel(ClientError),
    /// The gateway's sealed answer is no receipt.
    NotLogged(Answer),
    /// The sealed reply does not open under the request's key, or answers another read.
    Reply,
    /// The gateway answered the read with no reply (a refused proof, another network).
    NoReply(Answer),
}

/// The bound on a credential, issuance or commit file: the largest is a few hundred bytes.
pub(crate) const SMALL: u64 = 4096;
/// The bound on a genesis, a question's file and an event's.
pub(crate) const LARGE: u64 = 1 << 20;

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bytes [`hex`] writes as `text`, and `None` for any other text.
pub(crate) fn unhex(text: &str) -> Option<Vec<u8>> {
    let digit = |c: &u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    };
    let pairs = text.as_bytes().chunks(2);
    pairs
        .map(|p| match p {
            [high, low] => Some((digit(high)? << 4) | digit(low)?),
            _ => None,
        })
        .collect()
}

pub(crate) fn random() -> Result<[u8; 32], Error> {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|_| Error::Io("randomness", io::ErrorKind::Other))?;
    Ok(seed)
}

fn create(path: &Path, private: bool, what: &'static str) -> Result<File, Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    if private {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(not(unix))]
    let _ = private;
    options.open(path).map_err(|e| Error::Io(what, e.kind()))
}

fn fill(mut file: File, path: &Path, bytes: &[u8], what: &'static str) -> Result<(), Error> {
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    written.map_err(|e| {
        let _ = fs::remove_file(path);
        Error::Io(what, e.kind())
    })
}

/// A file to write: its path, its bytes, whether it is 0600, and its name in errors.
pub struct Output<'a>(pub &'a Path, pub &'a [u8], pub bool, pub &'static str);

/// Writes each output to its path, which must not exist; if one cannot be made none is left.
pub(crate) fn write_new(outputs: &[Output]) -> Result<(), Error> {
    let remove = |made: &[&Output]| {
        for Output(path, ..) in made {
            let _ = fs::remove_file(path);
        }
    };
    let mut files = Vec::new();
    for out in outputs {
        match create(out.0, out.2, out.3) {
            Ok(file) => files.push((file, out)),
            Err(e) => {
                remove(&files.iter().map(|(_, o)| *o).collect::<Vec<_>>());
                return Err(e);
            }
        }
    }
    let made: Vec<&Output> = files.iter().map(|(_, o)| *o).collect();
    for (file, Output(path, bytes, _, what)) in files {
        fill(file, path, bytes, what).inspect_err(|_| remove(&made))?;
    }
    Ok(())
}

/// The bytes of `path`, refused as `Malformed` beyond `max`.
pub(crate) fn read(path: &Path, max: u64, what: &'static str) -> Result<Vec<u8>, Error> {
    let io = |e: io::Error| Error::Io(what, e.kind());
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(io)?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    match bytes.len() as u64 > max {
        true => Err(Error::Malformed(what)),
        false => Ok(bytes),
    }
}

pub(crate) fn decoded<T>(
    bytes: &[u8],
    decode: fn(&[u8]) -> Option<T>,
    what: &'static str,
) -> Result<T, Error> {
    decode(bytes).ok_or(Error::Malformed(what))
}

/// A network as its genesis document names it: the answer review v1 under D49–D51's
/// configuration, its id, its issuer key and its tiers (`docs/04` T88-P1).
pub struct Network {
    pub genesis: Genesis,
    pub id: [u8; 32],
    pub issuer: IssuerPublic,
    pub tiers: Tiers,
}

impl Network {
    /// The document whose bytes these are, checked as a node checks it.
    pub fn decode(bytes: &[u8]) -> Result<Network, Error> {
        let refused = Error::Malformed("genesis");
        let genesis = Genesis::decode(bytes).map_err(|_| Error::Malformed("genesis"))?;
        let config = AnswerConfig::decode(&genesis.answer_config);
        if config != Some(AnswerConfig::default()) {
            return Err(refused);
        }
        let issuer = IssuerPublic::decode(&genesis.issuer).ok_or(Error::Malformed("genesis"))?;
        let tiers = genesis.tiers().map_err(|_| Error::Malformed("genesis"))?;
        Ok(Network {
            id: genesis.id(),
            genesis,
            issuer,
            tiers,
        })
    }

    pub fn load(path: &Path) -> Result<Network, Error> {
        Network::decode(&read(path, LARGE, "genesis")?)
    }
}
