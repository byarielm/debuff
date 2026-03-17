use p256::ecdsa::{SigningKey, Signature, signature::Signer};
use sha2::{Sha256, Digest};
use serde::Serialize;

/// Represents a label before signing (all fields except sig)
#[derive(Serialize)]
pub struct UnsignedLabel {
    pub ver: i32,
    pub src: String,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub val: String,
    #[serde(skip_serializing_if = "is_false")]
    pub neg: bool,
    pub cts: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

pub struct LabelSigner {
    key: SigningKey,
}

impl LabelSigner {
    /// Create a signer from a PEM-encoded PKCS#8 private key
    pub fn from_pem(pem: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        use p256::pkcs8::DecodePrivateKey;
        let key = SigningKey::from_pkcs8_pem(pem)?;
        Ok(Self { key })
    }

    /// Create a signer from raw key bytes (32 bytes)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let key = SigningKey::from_bytes(bytes.into())?;
        Ok(Self { key })
    }

    /// Generate a new random signing key
    pub fn generate() -> Self {
        let key = SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
        Self { key }
    }

    /// Sign a label following ATProto spec:
    /// 1. Encode as DAG-CBOR (deterministic)
    /// 2. SHA-256 hash the bytes
    /// 3. Sign the hash with P-256 ECDSA
    pub fn sign_label(&self, label: &UnsignedLabel) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let cbor_bytes = serde_ipld_dagcbor::to_vec(label)?;
        let hash = Sha256::digest(&cbor_bytes);
        let signature: Signature = self.key.sign(&hash);
        Ok(signature.to_bytes().to_vec())
    }

    /// Export the signing key as a PEM-encoded PKCS#8 string.
    pub fn to_pem(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use p256::pkcs8::EncodePrivateKey;
        let pem = self.key.to_pkcs8_pem(p256::pkcs8::LineEnding::LF)?;
        Ok(pem.to_string())
    }

    /// Get the public key as a did:key multibase string (base58btc, P-256 multicodec prefix 0x1200).
    pub fn public_key_multibase(&self) -> String {
        use p256::ecdsa::VerifyingKey;
        let verifying_key = VerifyingKey::from(&self.key);
        let compressed = verifying_key.to_encoded_point(true);
        let compressed_bytes = compressed.as_bytes();

        // P-256 multicodec prefix: 0x1200 as varint = [0x80, 0x24]
        let mut prefixed = vec![0x80, 0x24];
        prefixed.extend_from_slice(compressed_bytes);

        multibase::encode(multibase::Base::Base58Btc, &prefixed)
    }

    /// Get the public key bytes (for registering in DID document)
    pub fn public_key_bytes(&self) -> Vec<u8> {
        use p256::ecdsa::VerifyingKey;
        let verifying_key = VerifyingKey::from(&self.key);
        verifying_key.to_encoded_point(true).as_bytes().to_vec()
    }
}
