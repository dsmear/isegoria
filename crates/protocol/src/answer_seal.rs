//! The inner seal of a read's reply (`docs/05` T79-P7, §Built for participants): RFC 9180 base
//! mode, single shot, DHKEM(X25519, HKDF-SHA256), HKDF-SHA256, AES-128-GCM; a bespoke composition.

use hpke::aead::AesGcm128;
use hpke::kdf::HkdfSha256;
use hpke::kem::X25519HkdfSha256;
use hpke::{Deserializable, Kem, OpModeR, OpModeS, Serializable};

type Suite = X25519HkdfSha256;
type PublicKey = <Suite as Kem>::PublicKey;
type PrivateKey = <Suite as Kem>::PrivateKey;
type EncappedKey = <Suite as Kem>::EncappedKey;

/// The seal's `info`; its `aad` is empty.
pub const INFO: &[u8] = b"isegoria/answer-review/read-reply/v1";
/// A sealed reply's bytes beyond the reply: the encapsulated key (32) and the tag (16).
pub const OVERHEAD: usize = ENC + 16;
const ENC: usize = 32;

/// A requester's response key pair, fresh for each read request.
pub struct ResponseKey {
    secret: PrivateKey,
    public: [u8; 32],
}

impl ResponseKey {
    /// A fresh pair from the operating system's randomness.
    pub fn generate() -> Self {
        let (secret, public) = Suite::gen_keypair();
        Self::pair(secret, &public)
    }

    /// RFC 9180 `DeriveKeyPair(ikm)`.
    pub fn derive(ikm: &[u8; 32]) -> Self {
        let (secret, public) = Suite::derive_keypair(ikm);
        Self::pair(secret, &public)
    }

    fn pair(secret: PrivateKey, public: &PublicKey) -> Self {
        let mut bytes = [0; 32];
        public.write_exact(&mut bytes);
        ResponseKey {
            secret,
            public: bytes,
        }
    }

    pub fn public(&self) -> [u8; 32] {
        self.public
    }

    /// The reply [`seal`] sealed to this key; `None` if the bytes do not open under it.
    pub fn open(&self, sealed: &[u8]) -> Option<Vec<u8>> {
        open_with(&self.secret, INFO, b"", sealed)
    }
}

/// `reply` sealed to `response_key`: the encapsulated key, then the ciphertext with its tag;
/// `None` for a key the KEM refuses.
pub fn seal(response_key: &[u8; 32], reply: &[u8]) -> Option<Vec<u8>> {
    let key = PublicKey::from_bytes(response_key).ok()?;
    let mode = OpModeS::Base;
    let sealed =
        hpke::single_shot_seal::<AesGcm128, HkdfSha256, Suite>(&mode, &key, INFO, reply, b"");
    sealed.ok().map(|(enc, ct)| framed(&enc, ct))
}

fn framed(enc: &EncappedKey, ct: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0; ENC];
    enc.write_exact(&mut out);
    out.extend(ct);
    out
}

fn open_with(secret: &PrivateKey, info: &[u8], aad: &[u8], sealed: &[u8]) -> Option<Vec<u8>> {
    if sealed.len() < OVERHEAD {
        return None;
    }
    let (enc, ct) = sealed.split_at(ENC);
    let enc = EncappedKey::from_bytes(enc).ok()?;
    let mode = OpModeR::Base;
    hpke::single_shot_open::<AesGcm128, HkdfSha256, Suite>(&mode, secret, &enc, info, ct, aad).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hpke::rand_core::utils::next_word_via_fill;
    use hpke::rand_core::{Infallible, TryCryptoRng, TryRng};

    /// Randomness given in advance: the ephemeral key's `ikmE`.
    struct Fixed<'a>(&'a [u8]);

    impl TryRng for Fixed<'_> {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Infallible> {
            next_word_via_fill(self)
        }

        fn try_next_u64(&mut self) -> Result<u64, Infallible> {
            next_word_via_fill(self)
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
            let (taken, rest) = self.0.split_at(dst.len());
            dst.copy_from_slice(taken);
            self.0 = rest;
            Ok(())
        }
    }

    impl TryCryptoRng for Fixed<'_> {}

    fn seal_with(key: &[u8; 32], info: &[u8], aad: &[u8], pt: &[u8], ikm_e: &[u8]) -> Vec<u8> {
        let key = PublicKey::from_bytes(key).unwrap();
        let mode = OpModeS::Base;
        let mut rng = Fixed(ikm_e);
        let (enc, ct) = hpke::single_shot_seal_with_rng::<AesGcm128, HkdfSha256, Suite>(
            &mode, &key, info, pt, aad, &mut rng,
        )
        .unwrap();
        framed(&enc, ct)
    }

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn bytes32(s: &str) -> [u8; 32] {
        hex(s).try_into().unwrap()
    }

    const IKM_E: &str = "7268600d403fce431561aef583ee1613527cff655c1343f29812e66706df3234";
    const IKM_R: &str = "6db9df30aa07dd42ee5e8181afdb977e538f5e1fec8a06223f33f7013e525037";
    const PK_R: &str = "3948cfe0ad1ddb695d780e59077195da6c56506b027329794ab02bca80815c4d";
    const ENC_E: &str = "37fda3567bdbd628e88668c3c8d7e97d1d1253b6d4ea6d44c150f741f1bf4431";

    // AT-T79-29: RFC 9180 A.1.1's base-mode vector: the derived key, `enc` and the first message.
    #[test]
    fn the_rfc_9180_vector() {
        let receiver = ResponseKey::derive(&bytes32(IKM_R));
        assert_eq!(receiver.public(), bytes32(PK_R));
        let info = hex("4f6465206f6e2061204772656369616e2055726e");
        let aad = hex("436f756e742d30");
        let pt = hex("4265617574792069732074727574682c20747275746820626561757479");
        let ct ="f938558b5d72f1a23810b4be2ab4f84331acc02fc97babc53a52ae8218a355a96d8770ac83d07bea87e13c512a";
        let sealed = seal_with(&receiver.public(), &info, &aad, &pt, &hex(IKM_E));
        assert_eq!(sealed, [hex(ENC_E), hex(ct)].concat());
        assert_eq!(open_with(&receiver.secret, &info, &aad, &sealed), Some(pt));
    }

    // AT-T79-29: the reply seal's own vector (an independent RFC 9180 reference), opened by `open`.
    #[test]
    fn the_reply_seal_vector() {
        let receiver = ResponseKey::derive(&bytes32(IKM_R));
        let reply = b"isegoria: a sealed read reply";
        let sealed = seal_with(&receiver.public(), INFO, b"", reply, &hex(IKM_E));
        let expected = "8547a122f9d590a8840b79becba07c76da9bc4e7a843fb54720be84540415310178684e1c15674596f83d9b29f";
        assert_eq!(sealed, [hex(ENC_E), hex(expected)].concat());
        assert_eq!(sealed.len(), reply.len() + OVERHEAD);
        assert_eq!(receiver.open(&sealed).as_deref(), Some(&reply[..]));
    }

    // AT-T79-29: a fresh seal opens only under its key, unaltered and whole; two seals differ.
    #[test]
    fn a_seal_opens_only_whole_under_its_key() {
        let (receiver, other) = (ResponseKey::generate(), ResponseKey::generate());
        let reply = b"a reply";
        let sealed = seal(&receiver.public(), reply).unwrap();
        assert_eq!(receiver.open(&sealed).as_deref(), Some(&reply[..]));
        assert_ne!(seal(&receiver.public(), reply).unwrap(), sealed);
        assert_eq!(other.open(&sealed), None);
        for i in 0..sealed.len() {
            let mut altered = sealed.clone();
            altered[i] ^= 1;
            assert_eq!(receiver.open(&altered), None, "byte {i}");
        }
        assert_eq!(receiver.open(&sealed[..sealed.len() - 1]), None);
        assert_eq!(receiver.open(&[sealed.clone(), vec![0]].concat()), None);
        assert_eq!(receiver.open(&sealed[..OVERHEAD - 1]), None);
        let empty = seal(&receiver.public(), b"").unwrap();
        assert_eq!(empty.len(), OVERHEAD);
        assert_eq!(receiver.open(&empty), Some(Vec::new()));
        assert_eq!(seal(&[0; 32], reply), None);
    }
}
