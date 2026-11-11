//! The tool as processes (`docs/03` T19-P4; AT-ID-24): a whole ceremony, an answer and an abort
//! through `isegoria-ceremony`, one invocation per step and member.

mod support;

use ceremony::FINISH;
use identity::credential::ceremony::SlotRequest;
use identity::credential::{Credential, IssuerPublic};
use std::ffi::OsString;
use std::process::{Command, Output};
use support::*;

fn tool(args: Vec<OsString>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_isegoria-ceremony"))
        .args(args)
        .output()
        .unwrap()
}

fn args(words: &[&dyn AsRef<std::ffi::OsStr>]) -> Vec<OsString> {
    words.iter().map(|w| w.as_ref().to_os_string()).collect()
}

fn text(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

// One invocation per step and member runs the ceremony; other commands exit as stated (AT-ID-24).
#[test]
fn at_id_24_the_tool_runs_the_ceremony_as_processes() {
    let c = Committee::new("processes", &[1, 2]);
    std::fs::create_dir_all(c.root.join("p")).unwrap();
    for i in 0..4 {
        let out = tool(args(&[
            &"init",
            &c.procedure,
            &c.identities[i],
            &c.dir("p", i),
        ]));
        assert!(out.status.success(), "{out:?}");
        assert!(text(&out).starts_with("step 1\n"));
        assert_eq!(text(&out).matches("\nsent ").count(), 3);
    }
    let mut issuers = Vec::new();
    for step in 2..=FINISH {
        for i in 0..4 {
            let mut words = args(&[&"step", &c.dir("p", i), &c.identities[i]]);
            words.extend(c.inbox("p", step - 1, i).into_iter().map(OsString::from));
            let out = tool(words);
            assert!(out.status.success(), "step {step} member {i}: {out:?}");
            assert!(text(&out).starts_with(&format!("step {step}\nceremony ")));
            if step == FINISH {
                let line = text(&out)
                    .lines()
                    .find(|l| l.starts_with("issuer "))
                    .unwrap()
                    .to_string();
                issuers.push(line);
            }
        }
    }
    assert!(issuers.iter().all(|line| *line == issuers[0]));
    let status = tool(args(&[&"status", &c.dir("p", 0)]));
    assert!(text(&status).contains("phase done\n") && text(&status).contains("answered 0\n"));
    let again = tool(args(&[&"step", &c.dir("p", 0), &c.identities[0]]));
    assert_eq!(again.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&again.stderr).contains("Finished"));
    let issuer = issuers[0].trim_start_matches("issuer ");
    let bytes: Vec<u8> = (0..issuer.len() / 2)
        .map(|i| u8::from_str_radix(&issuer[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    let key = IssuerPublic::decode(&bytes).unwrap();
    let (request, _) = SlotRequest::new(&Credential::from_secret([1; 32]), 2, &key);
    let path = c.root.join("request");
    std::fs::write(&path, request.encode()).unwrap();
    let share = c.root.join("share");
    let answered = tool(args(&[&"answer", &c.dir("p", 0), &path, &share]));
    assert!(text(&answered).starts_with("answered 2\n"), "{answered:?}");
    let again = c.root.join("again");
    let twice = tool(args(&[&"answer", &c.dir("p", 0), &path, &again]));
    assert!(text(&twice).starts_with("answered 2\n"), "{twice:?}");
    assert_eq!(
        std::fs::read(&again).unwrap(),
        std::fs::read(&share).unwrap()
    );
    let (request, _) = SlotRequest::new(&Credential::from_secret([2; 32]), 2, &key);
    let path = c.root.join("another");
    std::fs::write(&path, request.encode()).unwrap();
    let other = c.root.join("other");
    let refused = tool(args(&[&"answer", &c.dir("p", 0), &path, &other]));
    assert_eq!(refused.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("AlreadyAnswered(2)"));
    let aborted = tool(args(&[&"abort", &c.dir("p", 1)]));
    assert_eq!(text(&aborted), "aborted\n");
    let status = tool(args(&[&"status", &c.dir("p", 1)]));
    assert!(text(&status).contains("phase aborted\n"));
    let unknown = tool(args(&[&"frobnicate"]));
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown.stderr).starts_with("usage"));
}
