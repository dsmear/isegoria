//! T19's ceremony run once in process for the ceremony targets (`docs/03` T19-P1, P2): four
//! members, threshold 3, the named signers and slots; each member before each flight, and after.
#![allow(dead_code)]

use identity::credential::ceremony::{Party, Setup, FLIGHTS};
use std::collections::BTreeMap;

pub const SIGNERS: [u16; 3] = [0, 2, 3];
pub const SLOTS: [u32; 2] = [5, 9];

/// A member's state before a flight, and the payloads by sender that flight takes.
pub struct Step {
    pub party: Party,
    pub incoming: BTreeMap<u16, Vec<u8>>,
}

pub struct Ceremony {
    pub steps: Vec<Step>,
    pub prepared: Vec<Party>,
}

pub fn run() -> Ceremony {
    let setup = Setup::new(4, 3, SIGNERS.to_vec(), SLOTS.to_vec()).expect("a valid setup");
    let mut parties: Vec<Party> = (0..4)
        .map(|i| Party::new(setup.clone(), i, [7; 32]).expect("a member"))
        .collect();
    let mut inbox: BTreeMap<u16, BTreeMap<u16, Vec<u8>>> = BTreeMap::new();
    let mut steps = Vec::new();
    for _ in 2..=FLIGHTS {
        let mut next: BTreeMap<u16, BTreeMap<u16, Vec<u8>>> = BTreeMap::new();
        for party in parties.iter_mut() {
            let incoming = inbox.remove(&party.me()).unwrap_or_default();
            let (after, out) = party.advance(&incoming).expect("a genuine flight");
            for (to, bytes) in out {
                next.entry(to).or_default().insert(party.me(), bytes);
            }
            let before = std::mem::replace(party, after);
            steps.push(Step {
                party: before,
                incoming,
            });
        }
        inbox = next;
    }
    Ceremony {
        steps,
        prepared: parties,
    }
}
