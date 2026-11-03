//! A tiered network's genesis in a node (`docs/04` §Operating contract, T88-P1): the document
//! loaded whole, its family's parts validated, the TOML's repeated values compared with it.

use crate::archive::{read_bounded, MAX_CONFIG};
use crate::{bytes_of, hex, keys, unhex, Config, ConfigError};
use identity::credential::IssuerPublic;
use network::genesis::{Genesis, GenesisError, VERSION};
use protocol::answer_path::AnswerConfig;
use std::path::Path;

/// The TOML's name for the event family, the answer review v1.
pub const FAMILY: &str = "answer-review/v1";

/// Why a genesis does not start a node.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// A tiered configuration, or one repeating a genesis's values, that names none.
    Missing,
    Decode(GenesisError),
    /// The document's configuration is not D49–D51's, which the family pins, in its encoding;
    /// or its issuer key is not one canonical encoding.
    AnswerConfig,
    Issuer,
    /// The TOML value that does not repeat the document's.
    Differs(&'static str),
}

/// A checked document, its family's parts decoded.
pub struct Document {
    pub genesis: Genesis,
    pub digest: [u8; 32],
    pub id: [u8; 32],
    pub answer_config: AnswerConfig,
    pub issuer: IssuerPublic,
}

fn refused(error: Error) -> ConfigError {
    ConfigError::Genesis(error)
}

/// The document at `path`; reads nothing else and writes nothing.
pub fn load(path: &Path) -> Result<Document, ConfigError> {
    let bytes = read_bounded(path, MAX_CONFIG, "genesis")?;
    let genesis = Genesis::decode(&bytes).map_err(|e| refused(Error::Decode(e)))?;
    let answer_config = AnswerConfig::decode(&genesis.answer_config)
        .filter(|config| *config == AnswerConfig::default())
        .ok_or(refused(Error::AnswerConfig))?;
    let issuer = IssuerPublic::decode(&genesis.issuer).ok_or(refused(Error::Issuer))?;
    Ok(Document {
        digest: genesis.digest(),
        id: genesis.id(),
        genesis,
        answer_config,
        issuer,
    })
}

/// What `--genesis-digest` prints: the digest and the network id, then the document.
pub fn render(document: &Document) -> String {
    let g = &document.genesis;
    let (digest, id) = (hex(&document.digest), hex(&document.id));
    let mut out = format!("digest {digest}\nnetwork_id {id}\nversion {VERSION}\n");
    out += &format!("threshold {}\n", g.threshold);
    for (i, m) in g.members.iter().enumerate() {
        let keys = [m.consortium, m.circle, m.hpke].map(|k| hex(&k));
        out += &format!("member {i} {}\n", keys.join(" "));
    }
    let c = &document.answer_config;
    let ((num, den), (low, high, q, new)) = (c.rule.threshold(), c.quotas.parts());
    let (group, quorum, weight) = (c.rule.group(), c.rule.quorum(), c.divided_weight);
    out += &format!("family {FAMILY}\nanswer_config group {group} quorum {quorum} ");
    out += &format!("threshold {num}/{den} divided_weight {weight} low {low} high {high} ");
    out += &format!("quotas {} {} {} new_author {new} ", q[0], q[1], q[2]);
    out += &format!("half_life_closings {}\n", c.author.half_life_closings());
    out += &format!("issuer {}\n", hex(&g.issuer));
    for relay in &g.relays {
        out += &format!("relay {}\n", hex(relay));
    }
    out
}

fn listed(list: &[String], field: &'static str) -> Result<Vec<[u8; 32]>, ConfigError> {
    Ok(keys(list, field)?.iter().map(|k| k.to_bytes()).collect())
}

/// Refuses `config` unless each value it repeats is the document's; a gateway's relays as a set.
pub(crate) fn compare(config: &Config, document: &Document) -> Result<(), ConfigError> {
    let g = &document.genesis;
    let check = |same: bool, field| same.then_some(()).ok_or(refused(Error::Differs(field)));
    let id = unhex(&config.network_id, "network_id")?;
    check(id == document.id, "network_id")?;
    check(
        config.consortium.threshold == g.threshold,
        "consortium.threshold",
    )?;
    let members: Vec<_> = g.members.iter().map(|m| m.consortium).collect();
    let field = "consortium.members";
    check(listed(&config.consortium.members, field)? == members, field)?;
    let circle: Vec<_> = g.members.iter().map(|m| m.circle).collect();
    let field = "consortium.circle";
    check(listed(&config.consortium.circle, field)? == circle, field)?;
    check(config.family.as_deref() == Some(FAMILY), "family")?;
    let repeats = |text: &Option<String>, bytes: &[u8]| {
        text.as_deref().and_then(bytes_of).as_deref() == Some(bytes)
    };
    check(
        repeats(&config.answer_config, &g.answer_config),
        "answer_config",
    )?;
    check(repeats(&config.issuer, &g.issuer), "issuer")?;
    if config.hpke_key_file.is_some() {
        let relays = config.relays.iter().map(|k| unhex(k, "relays"));
        let mut relays = relays.collect::<Result<Vec<_>, _>>()?;
        relays.sort();
        check(relays == g.relays, "relays")?;
    }
    Ok(())
}
