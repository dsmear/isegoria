//! The ceremony over files, in-process (`docs/03` T19-P2–P6; AT-ID-17–29): four members'
//! directories, their messages carried from each `out/` to the others, then issuance per slot.

mod support;

use ceremony::message::{self, Header, MessageError};
use ceremony::{Error, Refusal, FINISH};
use identity::credential::ceremony::{combine, AnswerError, SignatureShare, SlotRequest};
use identity::credential::{Credential, IssuerPublic};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use support::*;

const SLOTS: [u32; 3] = [1, 2, 3];

fn request(root: &Path, name: &str, secret: u8, slot: u32, issuer: &IssuerPublic) -> PathBuf {
    let holder = Credential::from_secret([secret; 32]);
    let (request, _) = SlotRequest::new(&holder, slot, issuer);
    let path = root.join(name);
    std::fs::write(&path, request.encode()).unwrap();
    path
}

/// Re-seals `file` (from `from` to `to`) with its payload changed by `edit`, signed by `from`.
fn forged(
    c: &Committee,
    file: &Path,
    from: usize,
    to: usize,
    edit: impl Fn(&mut Vec<u8>),
) -> Vec<u8> {
    let keys = ceremony::member_keys(&c.identities[to]).unwrap();
    let members: Vec<[u8; 32]> = c.printed.iter().map(|p| p.consortium).collect();
    let bytes = std::fs::read(file).unwrap();
    let (header, mut payload) = message::open(&bytes, to as u8, &keys.hpke, &members).unwrap();
    edit(&mut payload);
    let signer = ceremony::member_keys(&c.identities[from]).unwrap().signing;
    message::seal(&header, &payload, &signer, &c.printed[to].hpke).unwrap()
}

fn write(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

// The 14 steps over files give all members one key, which `--genesis-assemble` takes (AT-ID-17).
#[test]
fn at_id_17_the_ceremony_over_files_yields_the_genesis_issuer_key() {
    let c = Committee::new("full", &SLOTS);
    let reports = c.run("a", FINISH);
    let issuer = reports[0].issuer.clone().unwrap();
    assert!(reports.iter().all(|r| r.issuer.as_ref() == Some(&issuer)));
    assert!(reports
        .iter()
        .all(|r| r.id == reports[0].id && r.sent.is_empty()));
    assert!(reports[0]
        .render()
        .contains(&format!("issuer {}", ceremony::hex(&issuer))));
    let key = IssuerPublic::decode(&issuer).unwrap();
    let mut text = format!(
        "threshold = 3\nfamily = \"answer-review/v1\"\nissuer = \"{}\"\nrelays = []\n",
        ceremony::hex(&issuer)
    );
    for p in &c.printed {
        let [a, b, h] = [p.consortium, p.circle, p.hpke].map(|k| ceremony::hex(&k));
        text += &format!("\n[[member]]\nconsortium = \"{a}\"\ncircle = \"{b}\"\nhpke = \"{h}\"\n");
    }
    let input = write(&c.root, "genesis.toml", text.as_bytes());
    let document = node::genesis::assemble(&input, &c.root.join("genesis.bin")).unwrap();
    assert_eq!(document.issuer.encode(), issuer);
    for i in 0..4 {
        let summary = ceremony::status(&c.dir("a", i)).unwrap();
        assert_eq!(summary.phase, "done");
        assert_eq!(summary.answered, SIGNERS.contains(&i).then_some(0));
        #[cfg(unix)]
        {
            assert_eq!(mode(&c.dir("a", i)), 0o700);
            assert_eq!(mode(&c.dir("a", i).join("state")), 0o600);
            assert_eq!(mode(&c.inbox("a", 13, (i + 1) % 4)[0]), 0o600);
        }
    }
    let to_signers = request(&c.root, "request", 9, 2, &key);
    let mut shares = Vec::new();
    for &i in &SIGNERS {
        let share = c.root.join(format!("share-{i}"));
        assert_eq!(ceremony::answer(&c.dir("a", i), &to_signers, &share), Ok(2));
        shares.push(SignatureShare::decode(&std::fs::read(share).unwrap()).unwrap());
    }
    assert_eq!(
        shares.iter().map(|s| s.member()).collect::<Vec<_>>(),
        [0, 2, 3]
    );
    assert_eq!(
        ceremony::step(&c.dir("a", 0), &c.identities[0], &[]).unwrap_err(),
        Error::Finished
    );
}

// A participant's request, answered by the three signers, combines into a credential (AT-ID-22).
#[test]
fn at_id_22_the_participant_combines_the_three_shares() {
    let c = Committee::new("participant", &SLOTS);
    let issuer = c.run("a", FINISH)[0].issuer.clone().unwrap();
    let key = IssuerPublic::decode(&issuer).unwrap();
    let holder = Credential::from_secret([11; 32]);
    let (request, pending) = SlotRequest::new(&holder, 3, &key);
    let path = write(&c.root, "request", &request.encode());
    let shares: Vec<SignatureShare> = SIGNERS
        .iter()
        .map(|&i| {
            let share = c.root.join(format!("share-{i}"));
            ceremony::answer(&c.dir("a", i), &path, &share).unwrap();
            SignatureShare::decode(&std::fs::read(share).unwrap()).unwrap()
        })
        .collect();
    let credential = combine(pending.clone(), &shares, &key).unwrap();
    assert!(credential.verify(&key));
    assert!(combine(pending, &shares[..2], &key).is_err());
}

// A slot yields one share, durably: the same request gets it again, any other none (AT-ID-21).
#[test]
fn at_id_21_a_slot_is_answered_once() {
    let c = Committee::new("once", &SLOTS);
    let issuer = c.run("a", FINISH)[0].issuer.clone().unwrap();
    let key = IssuerPublic::decode(&issuer).unwrap();
    let (signer, other) = (c.dir("a", 0), c.dir("a", 2));
    let first = request(&c.root, "first", 1, 1, &key);
    let second = request(&c.root, "second", 2, 1, &key);
    let share = |n: &str| c.root.join(n);
    assert_eq!(ceremony::answer(&signer, &first, &share("s1")), Ok(1));
    let sent = std::fs::read(share("s1")).unwrap();
    assert_eq!(ceremony::answer(&signer, &first, &share("s2")), Ok(1));
    assert_eq!(std::fs::read(share("s2")).unwrap(), sent);
    assert_eq!(
        ceremony::answer(&signer, &second, &share("s3")),
        Err(Error::AlreadyAnswered(1))
    );
    let entry = signer.join("answered").join("1");
    let kept = std::fs::read(&entry).unwrap();
    let digest = Sha256::digest(std::fs::read(&first).unwrap());
    assert_eq!((&kept[..32], &kept[32..]), (&digest[..], &sent[..]));
    #[cfg(unix)]
    assert_eq!(mode(&entry), 0o600);
    assert!(!share("s3").exists());
    assert_eq!(ceremony::answer(&other, &second, &share("s4")), Ok(1));
    let outside = request(&c.root, "outside", 3, 7, &key);
    let refused = Err(Error::Answer(AnswerError::NotListed));
    assert_eq!(ceremony::answer(&signer, &outside, &share("s5")), refused);
    assert!(!signer.join("answered").join("7").exists());
    let mut relabelled = std::fs::read(request(&c.root, "r3", 4, 2, &key)).unwrap();
    relabelled[1..5].copy_from_slice(&3u32.to_le_bytes());
    let relabelled = write(&c.root, "relabelled", &relabelled);
    let proof = Err(Error::Answer(AnswerError::Proof));
    assert_eq!(ceremony::answer(&signer, &relabelled, &share("s6")), proof);
    let three = request(&c.root, "three", 5, 3, &key);
    assert_eq!(ceremony::answer(&signer, &three, &share("s7")), Ok(3));
    assert_eq!(
        ceremony::answer(&signer, &three, &share("s1")),
        Err(Error::Exists)
    );
    let garbage = write(&c.root, "garbage", b"not a request");
    assert_eq!(
        ceremony::answer(&signer, &garbage, &share("s8")),
        Err(Error::Request)
    );
    assert_eq!(
        ceremony::answer(&c.dir("a", 1), &three, &share("s9")),
        Err(Error::NotSigner)
    );
    assert_eq!(ceremony::status(&signer).unwrap().answered, Some(2));
    std::fs::remove_file(share("s1")).unwrap();
    assert_eq!(ceremony::answer(&signer, &first, &share("s1")), Ok(1));
    assert_eq!(std::fs::read(share("s1")).unwrap(), sent);
    let refused = Err(Error::AlreadyAnswered(1));
    assert_eq!(ceremony::answer(&signer, &second, &share("s3")), refused);
    let mut truncated = kept.clone();
    truncated.pop();
    std::fs::write(&entry, truncated).unwrap();
    let broken = Err(Error::Io("record", std::io::ErrorKind::InvalidData));
    assert_eq!(ceremony::answer(&signer, &first, &share("s11")), broken);
    assert!(!share("s11").exists());
    std::fs::rename(signer.join("answered"), c.root.join("moved")).unwrap();
    let two = request(&c.root, "two", 6, 2, &key);
    assert_eq!(
        ceremony::answer(&signer, &two, &share("s10")),
        Err(Error::Lost)
    );
}

// A message failing its seal, signature, recipient, id, step or library check aborts (AT-ID-18).
#[test]
fn at_id_18_a_refused_message_aborts_the_member() {
    let c = Committee::new("refused", &SLOTS);
    c.run("a", 2);
    c.init("b");
    for i in 0..4 {
        c.step_one("b", 2, i).unwrap();
    }
    let inbox = c.inbox("a", 2, 0);
    let from = |j: usize| inbox[j - 1].clone();
    let case = |name: &str, bad: PathBuf| {
        let dir = c.root.join(name);
        copy_dir(&c.dir("a", 0), &dir);
        let mut files = inbox.clone();
        files.push(bad.clone());
        files.retain(|f| f != &from(1) || bad == from(1));
        let error = ceremony::step(&dir, &c.identities[0], &files).unwrap_err();
        assert_eq!(ceremony::status(&dir).unwrap().phase, "aborted", "{name}");
        assert_eq!(
            ceremony::step(&dir, &c.identities[0], &inbox),
            Err(Error::Aborted)
        );
        error
    };
    let mut sealed = std::fs::read(from(1)).unwrap();
    let last = sealed.len() - 1;
    sealed[last] ^= 1;
    let tampered = write(&c.root, "tampered", &sealed);
    let seal = Error::Refused(Some(tampered.clone()), Refusal::Message(MessageError::Seal));
    assert_eq!(case("seal", tampered), seal);
    let elsewhere = c.inbox("a", 2, 1)[0].clone();
    let recipient = Refusal::Message(MessageError::Recipient);
    assert_eq!(
        case("recipient", elsewhere.clone()),
        Error::Refused(Some(elsewhere), recipient)
    );
    let keys = ceremony::member_keys(&c.identities[1]).unwrap();
    let members: Vec<[u8; 32]> = c.printed.iter().map(|p| p.consortium).collect();
    let own = ceremony::member_keys(&c.identities[0]).unwrap();
    let (h, payload) =
        message::open(&std::fs::read(from(2)).unwrap(), 0, &own.hpke, &members).unwrap();
    let claimed = message::seal(
        &Header { sender: 2, ..h },
        &payload,
        &keys.signing,
        &c.printed[0].hpke,
    );
    let claimed = write(&c.root, "claimed", &claimed.unwrap());
    let signature = Refusal::Message(MessageError::Signature);
    assert_eq!(
        case("signature", claimed.clone()),
        Error::Refused(Some(claimed), signature)
    );
    let foreign = c.inbox("b", 2, 0)[0].clone();
    assert_eq!(
        case("foreign", foreign.clone()),
        Error::Refused(Some(foreign), Refusal::Id)
    );
    let altered = forged(&c, &from(1), 1, 0, |p| p[0] ^= 1);
    let altered = write(&c.root, "altered", &altered);
    let refused = case("library", altered);
    assert!(
        matches!(refused, Error::Refused(Some(_), Refusal::Payload(_))),
        "{refused:?}"
    );
}

// A later step's file waits for that step; another file of an earlier step aborts (AT-ID-26).
#[test]
fn at_id_26_a_later_steps_file_waits_for_its_step() {
    let c = Committee::new("later", &SLOTS);
    c.run("a", 2);
    c.step_one("a", 3, 1).unwrap();
    let (dir, ahead) = (c.dir("a", 0), c.dir("a", 1).join("out").join("03-1-0.msg"));
    let mut files = c.inbox("a", 2, 0);
    files.insert(0, ahead.clone());
    let report = ceremony::step(&dir, &c.identities[0], &files).unwrap();
    assert_eq!((report.step, &report.later), (3, &vec![ahead.clone()]));
    assert!(report
        .render()
        .contains(&format!("later {}\n", ahead.display())));
    assert_eq!(ceremony::status(&dir).unwrap().phase, "running");
    for i in [2, 3] {
        c.step_one("a", 3, i).unwrap();
    }
    let inbox = c.inbox("a", 3, 0);
    assert!(inbox.contains(&ahead));
    let four = ceremony::step(&dir, &c.identities[0], &inbox).unwrap();
    assert_eq!((four.step, four.later), (4, vec![]));
    let earlier = forged(&c, &c.inbox("a", 2, 0)[0], 1, 0, |p| p[0] ^= 1);
    let earlier = write(&c.root, "earlier", &earlier);
    let refused = ceremony::step(&dir, &c.identities[0], &[earlier.clone()]);
    assert_eq!(
        refused,
        Err(Error::Refused(Some(earlier), Refusal::Step(2)))
    );
    assert_eq!(ceremony::status(&dir).unwrap().phase, "aborted");
}

// A step lacking a message changes nothing; repeated and taken files are skipped (AT-ID-19).
#[test]
fn at_id_19_a_step_needs_every_expected_message() {
    let c = Committee::new("complete", &SLOTS);
    c.init("a");
    let dir = c.dir("a", 0);
    assert_eq!(
        ceremony::step(&dir, &c.identities[0], &[]),
        Err(Error::Incomplete(vec![1, 2, 3]))
    );
    c.step_all("a", 2);
    let state = std::fs::read(dir.join("state")).unwrap();
    let inbox = c.inbox("a", 2, 0);
    let two = &inbox[..2];
    assert_eq!(
        ceremony::step(&dir, &c.identities[0], two),
        Err(Error::Incomplete(vec![3]))
    );
    assert_eq!(std::fs::read(dir.join("state")).unwrap(), state);
    let summary = ceremony::status(&dir).unwrap();
    assert_eq!((summary.step, summary.expects), (3, vec![1, 2, 3]));
    let mut files = inbox.clone();
    files.push(inbox[0].clone());
    files.extend(c.inbox("a", 1, 0));
    assert_eq!(
        ceremony::step(&dir, &c.identities[0], &files).unwrap().step,
        3
    );
    let wrong = ceremony::step(&dir, &c.identities[1], &[]).unwrap_err();
    assert_eq!(wrong, Error::NotMember);
    std::fs::write(dir.join("state"), b"not a state").unwrap();
    assert_eq!(ceremony::status(&dir).unwrap_err(), Error::State);
}

// A confirmation with another key aborts the member that takes it; the others finish (AT-ID-20).
#[test]
fn at_id_20_a_different_confirmation_aborts() {
    let c = Committee::new("confirm", &SLOTS);
    c.run("a", FINISH - 1);
    let copy = c.root.join("copy");
    copy_dir(&c.dir("a", 0), &copy);
    let inbox = c.inbox("a", FINISH - 1, 0);
    let other = forged(&c, &inbox[0], 1, 0, |p| p[10] ^= 1);
    let other = write(&c.root, "other", &other);
    let files = vec![other.clone(), inbox[1].clone(), inbox[2].clone()];
    let refused = ceremony::step(&copy, &c.identities[0], &files).unwrap_err();
    assert_eq!(
        refused,
        Error::Refused(Some(other), Refusal::Confirmation(1))
    );
    let list = forged(&c, &inbox[1], 2, 0, |p| p[97 + 32 + 3] ^= 1);
    let list = write(&c.root, "list", &list);
    let copy2 = c.root.join("copy2");
    copy_dir(&c.dir("a", 0), &copy2);
    let files = vec![inbox[0].clone(), list.clone(), inbox[2].clone()];
    let refused = ceremony::step(&copy2, &c.identities[0], &files).unwrap_err();
    assert_eq!(
        refused,
        Error::Refused(Some(list), Refusal::Confirmation(2))
    );
    let reports = c.step_all("a", FINISH);
    assert!(reports.iter().all(|r| r.issuer.is_some()));
}

// A step run again after losing its files writes the same bytes; an older state refuses (AT-ID-25).
#[test]
fn at_id_25_a_step_run_again_writes_the_same_files() {
    let c = Committee::new("again", &SLOTS);
    c.run("a", 10);
    let (dir, older) = (c.dir("a", 0), c.root.join("older"));
    copy_dir(&dir, &older);
    let first = c.step_one("a", 11, 0).unwrap();
    assert_eq!(first.sent.len(), 2);
    let sent: Vec<Vec<u8>> = first
        .sent
        .iter()
        .map(|f| std::fs::read(f).unwrap())
        .collect();
    let state = std::fs::read(dir.join("state")).unwrap();
    for file in &first.sent {
        std::fs::remove_file(file).unwrap();
    }
    assert_eq!(c.step_one("a", 11, 0).unwrap(), first);
    let again: Vec<Vec<u8>> = first
        .sent
        .iter()
        .map(|f| std::fs::read(f).unwrap())
        .collect();
    assert_eq!(again, sent);
    assert_eq!(std::fs::read(dir.join("state")).unwrap(), state);
    for file in &first.sent {
        std::fs::copy(file, older.join("out").join(file.file_name().unwrap())).unwrap();
    }
    let before = std::fs::read(older.join("state")).unwrap();
    let refused = ceremony::step(&older, &c.identities[0], &c.inbox("a", 10, 0));
    assert_eq!(
        refused,
        Err(Error::Written(older.join("out").join("11-0-2.msg")))
    );
    assert_eq!(std::fs::read(older.join("state")).unwrap(), before);
    let kept: Vec<Vec<u8>> = ["11-0-2.msg", "11-0-3.msg"]
        .map(|n| std::fs::read(older.join("out").join(n)).unwrap())
        .to_vec();
    assert_eq!(kept, sent);
    let opened = &c.init("b")[0].sent[0];
    let bytes = std::fs::read(opened).unwrap();
    std::fs::remove_file(opened).unwrap();
    let incomplete = ceremony::step(&c.dir("b", 0), &c.identities[0], &[]);
    assert_eq!(incomplete, Err(Error::Incomplete(vec![1, 2, 3])));
    assert_eq!(std::fs::read(opened).unwrap(), bytes);
}

// A confirmation echoing other commitments aborts the member that takes it (AT-ID-28).
#[test]
fn at_id_28_a_confirmation_echoing_other_commitments_aborts() {
    let c = Committee::new("echo", &SLOTS);
    c.run("a", FINISH - 1);
    let inbox = c.inbox("a", FINISH - 1, 0);
    for (at, from, file) in [(140, 1, 0), (140, 2, 1), (170, 3, 2)] {
        let copy = c.root.join(format!("copy-{at}-{from}"));
        copy_dir(&c.dir("a", 0), &copy);
        let other = forged(&c, &inbox[file], from, 0, |p| p[at] ^= 1);
        let other = write(&c.root, &format!("other-{at}-{from}"), &other);
        let mut files = inbox.clone();
        files[file] = other.clone();
        let refused = ceremony::step(&copy, &c.identities[0], &files).unwrap_err();
        let why = Refusal::Confirmation(from as u8);
        assert_eq!(refused, Error::Refused(Some(other), why));
    }
    let reports = c.step_all("a", FINISH);
    assert!(reports.iter().all(|r| r.issuer.is_some()));
}

// A step-1 file of an aborted ceremony is refused by a new ceremony's number (AT-ID-29).
#[test]
fn at_id_29_a_step_one_file_of_an_earlier_ceremony_is_refused() {
    let c = Committee::new("number", &SLOTS);
    c.init("a");
    ceremony::abort(&c.dir("a", 1)).unwrap();
    let text = std::fs::read_to_string(&c.procedure).unwrap();
    let next = write(
        &c.root,
        "next.toml",
        text.replacen("ceremony = 1", "ceremony = 2", 1).as_bytes(),
    );
    std::fs::create_dir_all(c.root.join("b")).unwrap();
    for i in 0..4 {
        ceremony::init(&next, &c.identities[i], &c.dir("b", i)).unwrap();
    }
    let stale = c.inbox("a", 1, 1)[0].clone();
    assert!(stale.ends_with("01-0-1.msg"));
    let mut files = c.inbox("b", 1, 1);
    files.retain(|f| !f.ends_with("01-0-1.msg"));
    files.push(stale.clone());
    let refused = ceremony::step(&c.dir("b", 1), &c.identities[1], &files).unwrap_err();
    assert_eq!(refused, Error::Refused(Some(stale), Refusal::Id));
    let honest = ceremony::step(&c.dir("b", 2), &c.identities[2], &c.inbox("b", 1, 2));
    assert_eq!(honest.unwrap().step, 2);
}

// An aborted member runs nothing more and holds no party; a new ceremony has a new id (AT-ID-23).
#[test]
fn at_id_23_an_aborted_ceremony_is_not_reused() {
    let c = Committee::new("abort", &SLOTS);
    let a = c.run("a", 2);
    let b = c.run("b", 2);
    assert_ne!(a[0].id, b[0].id);
    let dir = c.dir("a", 0);
    let before = std::fs::metadata(dir.join("state")).unwrap().len();
    ceremony::abort(&dir).unwrap();
    assert!(std::fs::metadata(dir.join("state")).unwrap().len() < before);
    let summary = ceremony::status(&dir).unwrap();
    assert_eq!((summary.phase, summary.issuer), ("aborted", None));
    let inbox = c.inbox("a", 2, 0);
    assert_eq!(
        ceremony::step(&dir, &c.identities[0], &inbox),
        Err(Error::Aborted)
    );
    let any = c.root.join("any");
    assert_eq!(ceremony::answer(&dir, &any, &any), Err(Error::Aborted));
    let again = ceremony::init(&c.procedure, &c.identities[0], &dir).unwrap_err();
    assert_eq!(
        again,
        Error::Io("ceremony directory", std::io::ErrorKind::AlreadyExists)
    );
    let stale = c.inbox("a", 2, 1)[0].clone();
    let mut files = c.inbox("b", 2, 1);
    files.retain(|f| !f.ends_with("02-0-1.msg"));
    files.push(stale.clone());
    let refused = ceremony::step(&c.dir("b", 1), &c.identities[1], &files).unwrap_err();
    assert_eq!(refused, Error::Refused(Some(stale), Refusal::Id));
}
