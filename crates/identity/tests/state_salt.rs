use identity::credential::{Credential, Issuer};
use identity::enrollment::Label;
use identity::nym::Role;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// Canonical scalar bytes and textual role tags pin the holder key; AT-T86-3,9.
#[test]
fn holder_salt_key_known_answers_survive_issuance() {
    let holder = Credential::from_secret([255; 32]);
    let issuer = Issuer::new([1; 32]);
    let (request, pending) = holder.request_issuance(&Label([3; 32]), &issuer.public());
    let issued = pending.finalize(issuer.issue(&request).unwrap());
    for (role, expected) in [
        (
            Role::Propose,
            "c05b6cb7ad3279f7685900c115dbd9379e08d770d7b55e8f84fd11cf70d85a93",
        ),
        (
            Role::Judge,
            "3c3dfb3fc5d0d026eb82c6b12ad57544d0aa908f45c642731cc6aa00ef30495c",
        ),
        (
            Role::Respond,
            "9f4636596738004e371f2301fafc16881dd1e0dce5e69cde9b88192ee6c571b1",
        ),
    ] {
        assert_eq!(hex(&holder.state_salt_key(role)), expected);
        assert_eq!(issued.state_salt_key(role), holder.state_salt_key(role));
    }
}

// Seeds representing the same credential scalar cannot rotate the salt key; AT-T86-3.
#[test]
fn equivalent_seed_bytes_keep_the_same_key() {
    let mut one = [0; 32];
    one[0] = 1;
    let modulus_plus_one = [
        2, 0, 0, 0, 255, 255, 255, 255, 254, 91, 254, 255, 2, 164, 189, 83, 5, 216, 161, 9, 8, 216,
        57, 51, 72, 125, 157, 41, 83, 167, 237, 115,
    ];
    let a = Credential::from_secret(one);
    let b = Credential::from_secret(modulus_plus_one);
    for role in [Role::Propose, Role::Judge, Role::Respond] {
        assert_eq!(a.state_salt_key(role), b.state_salt_key(role));
    }
}
