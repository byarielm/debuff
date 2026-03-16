use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use p256::ecdsa::{signature::Signer, Signature, SigningKey};

/// A test keypair for creating service auth JWTs.
pub struct TestKeypair {
    signing_key: SigningKey,
}

impl TestKeypair {
    /// Generate a random ES256 keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
        Self { signing_key }
    }

    /// Return the public key in multibase encoding with the P-256 multicodec prefix (0x80, 0x24).
    pub fn public_key_multibase(&self) -> String {
        use p256::ecdsa::VerifyingKey;
        let verifying_key = VerifyingKey::from(&self.signing_key);
        let compressed = verifying_key.to_encoded_point(true);
        let compressed_bytes = compressed.as_bytes();

        // P-256 multicodec prefix: 0x80 0x24
        let mut prefixed = vec![0x80, 0x24];
        prefixed.extend_from_slice(compressed_bytes);

        // Multibase base58btc encoding (prefix 'z')
        multibase::encode(multibase::Base::Base58Btc, &prefixed)
    }

    /// Build a valid service auth JWT for testing.
    pub fn build_service_jwt(&self, iss: &str, aud: &str) -> String {
        let header = serde_json::json!({
            "alg": "ES256",
            "typ": "JWT"
        });

        let exp = chrono::Utc::now().timestamp() as u64 + 300; // 5 minutes
        let payload = serde_json::json!({
            "iss": iss,
            "aud": aud,
            "exp": exp,
            "lxm": "com.atproto.moderation.createReport"
        });

        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header).unwrap());
        let payload_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap());

        let message = format!("{header_b64}.{payload_b64}");
        let signature: Signature = self.signing_key.sign(message.as_bytes());
        let sig_b64 = URL_SAFE_NO_PAD.encode(signature.to_bytes());

        format!("{message}.{sig_b64}")
    }
}

/// Mount a wiremock handler that returns a DID document for the given DID,
/// with the test keypair registered as the `#atproto` verification method.
pub async fn mock_did_document(
    mock_server: &wiremock::MockServer,
    did: &str,
    keypair: &TestKeypair,
) {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, ResponseTemplate};

    let did_doc = serde_json::json!({
        "@context": [
            "https://www.w3.org/ns/did/v1",
            "https://w3id.org/security/multikey/v1"
        ],
        "id": did,
        "verificationMethod": [
            {
                "id": format!("{did}#atproto"),
                "type": "Multikey",
                "controller": did,
                "publicKeyMultibase": keypair.public_key_multibase()
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path(format!("/{did}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&did_doc))
        .mount(mock_server)
        .await;
}
