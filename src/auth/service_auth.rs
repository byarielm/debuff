use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use p256::ecdsa::{Signature as P256Signature, VerifyingKey as P256Key, signature::Verifier};
use serde::Deserialize;

use crate::AppState;
use crate::error::AppError;

/// Authenticated ATProto user identity extracted from a service auth JWT.
///
/// Used for XRPC endpoints that receive proxied requests from PDSes
/// (e.g. createReport). The JWT is signed by the reporter's signing key
/// and validated by resolving their DID document.
#[derive(Debug, Clone)]
pub struct ServiceAuth {
    /// The authenticated user's DID (from `iss`).
    pub did: String,
}

// ---------------------------------------------------------------------------
// JWT types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
    #[serde(default)]
    typ: Option<String>,
}

#[derive(Deserialize)]
struct JwtPayload {
    iss: String,
    aud: String,
    exp: u64,
    #[serde(default)]
    lxm: Option<String>,
}

// ---------------------------------------------------------------------------
// DID document types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DidDocument {
    #[serde(default)]
    verification_method: Vec<VerificationMethod>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerificationMethod {
    id: String,
    #[serde(rename = "type")]
    method_type: String,
    #[serde(default)]
    public_key_multibase: Option<String>,
}

// ---------------------------------------------------------------------------
// Extractor
// ---------------------------------------------------------------------------

impl FromRequestParts<AppState> for ServiceAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let payload = verify_service_jwt(token, state, false).await?;

        Ok(ServiceAuth { did: payload.iss })
    }
}

// ---------------------------------------------------------------------------
// JWT verification
// ---------------------------------------------------------------------------

fn verify_service_jwt<'a>(
    token: &'a str,
    state: &'a AppState,
    is_retry: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<JwtPayload, AppError>> + Send + 'a>>
{
    Box::pin(async move {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized);
        }

        let header_bytes = URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| AppError::Unauthorized)?;
        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| AppError::Unauthorized)?;
        let sig_bytes = URL_SAFE_NO_PAD
            .decode(parts[2])
            .map_err(|_| AppError::Unauthorized)?;

        let header: JwtHeader =
            serde_json::from_slice(&header_bytes).map_err(|_| AppError::Unauthorized)?;
        let payload: JwtPayload =
            serde_json::from_slice(&payload_bytes).map_err(|_| AppError::Unauthorized)?;

        // Reject forbidden typ values.
        if let Some(ref typ) = header.typ {
            let t = typ.to_lowercase();
            if t == "at+jwt" || t == "refresh+jwt" || t == "dpop+jwt" {
                return Err(AppError::Unauthorized);
            }
        }

        // Only support ES256 and ES256K.
        if header.alg != "ES256" && header.alg != "ES256K" {
            tracing::warn!(alg = %header.alg, "unsupported JWT algorithm");
            return Err(AppError::Unauthorized);
        }

        // Check expiration.
        let now = chrono::Utc::now().timestamp() as u64;
        if now > payload.exp {
            tracing::warn!(exp = payload.exp, now = now, "service auth JWT expired");
            return Err(AppError::Unauthorized);
        }

        // Check audience matches our labeler DID.
        let expected_aud = &state.config.labeler.did;
        // The aud may include a fragment like #atproto_labeler, so strip it.
        let aud_did = payload.aud.split('#').next().unwrap_or(&payload.aud);
        if aud_did != expected_aud {
            tracing::warn!(
                expected = %expected_aud,
                got = %payload.aud,
                "service auth JWT audience mismatch"
            );
            return Err(AppError::Unauthorized);
        }

        // Check lxm if present.
        if let Some(ref lxm) = payload.lxm
            && lxm != "com.atproto.moderation.createReport"
        {
            tracing::warn!(lxm = %lxm, "service auth JWT lxm mismatch");
            return Err(AppError::Unauthorized);
        }

        // Resolve the issuer's DID document to get their signing key.
        let signing_key = resolve_signing_key(&payload.iss, state).await?;

        // Verify signature: message is the UTF-8 bytes of "header.payload".
        let msg = format!("{}.{}", parts[0], parts[1]);

        let valid = match header.alg.as_str() {
            "ES256" => verify_es256(msg.as_bytes(), &sig_bytes, &signing_key),
            "ES256K" => verify_es256k(msg.as_bytes(), &sig_bytes, &signing_key),
            _ => false,
        };

        if !valid && !is_retry {
            // Key may have been rotated — retry with a fresh DID resolution.
            tracing::debug!(iss = %payload.iss, "signature failed, retrying with fresh DID doc");
            return verify_service_jwt(token, state, true).await;
        }
        if !valid {
            tracing::warn!(iss = %payload.iss, "service auth JWT signature verification failed");
            return Err(AppError::Unauthorized);
        }

        Ok(payload)
    })
}

// ---------------------------------------------------------------------------
// DID resolution
// ---------------------------------------------------------------------------

async fn resolve_signing_key(did: &str, state: &AppState) -> Result<Vec<u8>, AppError> {
    let url = if did.starts_with("did:plc:") {
        format!(
            "{}/{did}",
            state.config.labeler.plc_url.trim_end_matches('/')
        )
    } else if did.starts_with("did:web:") {
        let domain = did.strip_prefix("did:web:").unwrap();
        let domain = domain.replace(':', "/");
        format!("https://{domain}/.well-known/did.json")
    } else {
        return Err(AppError::BadRequest(format!(
            "unsupported DID method: {did}"
        )));
    };

    let resp = state
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("DID resolution failed for {did}: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "DID resolution returned {} for {did}",
            resp.status()
        )));
    }

    let doc: DidDocument = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("invalid DID document for {did}: {e}")))?;

    // Find the #atproto verification method.
    let vm = doc
        .verification_method
        .iter()
        .find(|vm| vm.id == format!("{did}#atproto") || vm.id == "#atproto")
        .ok_or_else(|| {
            AppError::Internal(format!(
                "no #atproto verification method in DID doc for {did}"
            ))
        })?;

    let multibase = vm.public_key_multibase.as_deref().ok_or_else(|| {
        AppError::Internal(format!("no publicKeyMultibase on #atproto key for {did}"))
    })?;

    decode_multibase_key(multibase, &vm.method_type)
}

/// Decode a multibase-encoded public key from a DID document.
///
/// Returns the raw compressed public key bytes (33 bytes for both P-256 and secp256k1).
fn decode_multibase_key(multibase_str: &str, method_type: &str) -> Result<Vec<u8>, AppError> {
    let (_, decoded) = multibase::decode(multibase_str)
        .map_err(|e| AppError::Internal(format!("multibase decode failed: {e}")))?;

    // Multikey format: 2-byte multicodec prefix + compressed public key.
    // P-256:     0x80 0x24 (varint for 0x1200)
    // secp256k1: 0xe7 0x01 (varint for 0xe7)
    match method_type {
        "Multikey" => {
            if decoded.len() < 2 {
                return Err(AppError::Internal("multikey too short".into()));
            }
            // Strip the 2-byte multicodec prefix.
            Ok(decoded[2..].to_vec())
        }
        "EcdsaSecp256r1VerificationKey2019" | "EcdsaSecp256k1VerificationKey2019" => {
            // Raw compressed key (33 bytes).
            Ok(decoded)
        }
        other => Err(AppError::Internal(format!(
            "unsupported verification method type: {other}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Signature verification
// ---------------------------------------------------------------------------

fn verify_es256(msg: &[u8], sig_bytes: &[u8], key_bytes: &[u8]) -> bool {
    let Ok(verifying_key) = P256Key::from_sec1_bytes(key_bytes) else {
        tracing::warn!("failed to parse P-256 public key");
        return false;
    };

    // Try standard (low-S) signature first, then allow high-S (malleable).
    if let Ok(sig) = P256Signature::from_bytes(sig_bytes.into())
        && verifying_key.verify(msg, &sig).is_ok()
    {
        return true;
    }

    // Try normalizing (the p256 crate may reject high-S).
    if let Ok(sig) = P256Signature::from_bytes(sig_bytes.into())
        && let Some(normalized) = sig.normalize_s()
        && verifying_key.verify(msg, &normalized).is_ok()
    {
        return true;
    }

    false
}

fn verify_es256k(msg: &[u8], sig_bytes: &[u8], key_bytes: &[u8]) -> bool {
    use k256::ecdsa::{Signature as K256Signature, VerifyingKey as K256Key, signature::Verifier};

    let Ok(verifying_key) = K256Key::from_sec1_bytes(key_bytes) else {
        tracing::warn!("failed to parse secp256k1 public key");
        return false;
    };

    if let Ok(sig) = K256Signature::from_bytes(sig_bytes.into())
        && verifying_key.verify(msg, &sig).is_ok()
    {
        return true;
    }

    if let Ok(sig) = K256Signature::from_bytes(sig_bytes.into())
        && let Some(normalized) = sig.normalize_s()
        && verifying_key.verify(msg, &normalized).is_ok()
    {
        return true;
    }

    false
}
