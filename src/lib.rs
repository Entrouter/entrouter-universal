// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Entrouter Universal
//
//  Pipeline integrity guardian.
//  What goes in, comes out identical.
//
//  The problem: HTTP → JSON → Rust → Redis → Postgres
//  Each layer has its own escaping rules. Each one thinks
//  it's being helpful. By the time your data reaches the
//  destination, it's been mangled by 5 different opinions.
//
//  The solution: Base64 at entry. Opaque string through
//  every layer. Decode at destination. Compare. Done.
//
//  No special characters. Nothing to escape. Nothing to
//  double-escape. Every layer just moves a string it
//  cannot touch.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub mod envelope;
pub mod guardian;
pub mod verify;

pub use envelope::Envelope;
pub use guardian::Guardian;
pub use verify::VerifyResult;

// ── Errors ────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum UniversalError {
    #[error("Integrity violation: data was mutated in transit. Expected {expected}, got {actual}")]
    IntegrityViolation { expected: String, actual: String },

    #[error("Decode error: {0}")]
    DecodeError(String),

    #[error("Envelope malformed: {0}")]
    MalformedEnvelope(String),
}

// ── Core encode/decode ────────────────────────────────────

/// Encode raw bytes to a Base64 string safe to pass through any layer.
/// HTTP ✅ JSON ✅ Rust ✅ Redis ✅ Postgres ✅
/// Nothing to escape. Nothing to mangle.
pub fn encode(input: &[u8]) -> String {
    STANDARD.encode(input)
}

/// Decode a Base64 string back to raw bytes.
pub fn decode(input: &str) -> Result<Vec<u8>, UniversalError> {
    STANDARD
        .decode(input)
        .map_err(|e| UniversalError::DecodeError(e.to_string()))
}

/// Encode a string slice — convenience wrapper.
pub fn encode_str(input: &str) -> String {
    encode(input.as_bytes())
}

/// Decode back to a UTF-8 string.
pub fn decode_str(input: &str) -> Result<String, UniversalError> {
    let bytes = decode(input)?;
    String::from_utf8(bytes).map_err(|e| UniversalError::DecodeError(e.to_string()))
}

// ── Fingerprint ───────────────────────────────────────────

/// SHA-256 fingerprint of the raw input.
/// Travels alongside the encoded data through every layer.
/// On arrival: decode → hash → compare fingerprint. If they
/// match, nothing touched it. If they don't, something did.
pub fn fingerprint(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

/// Fingerprint a string slice — convenience wrapper.
pub fn fingerprint_str(input: &str) -> String {
    fingerprint(input.as_bytes())
}

// ── Verify ────────────────────────────────────────────────

/// Verify that a Base64-encoded string, when decoded, matches
/// the original fingerprint. This is the exit point check.
pub fn verify(encoded: &str, original_fingerprint: &str) -> Result<VerifyResult, UniversalError> {
    let decoded = decode(encoded)?;
    let actual_fingerprint = fingerprint(&decoded);

    if actual_fingerprint == original_fingerprint {
        Ok(VerifyResult {
            intact: true,
            decoded,
            fingerprint: actual_fingerprint,
        })
    } else {
        Err(UniversalError::IntegrityViolation {
            expected: original_fingerprint.to_string(),
            actual: actual_fingerprint,
        })
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_simple() {
        let original = "hello world";
        let encoded = encode_str(original);
        let decoded = decode_str(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn round_trip_special_chars() {
        let original = r#"hello "world" it's a test with \backslashes\ and "quotes" and
newlines"#;
        let encoded = encode_str(original);
        let decoded = decode_str(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn round_trip_json_payload() {
        let original = r#"{"user":"john","token":"abc\"def","data":{"nested":"val\\ue"}}"#;
        let encoded = encode_str(original);
        let decoded = decode_str(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn round_trip_sql_injection_attempt() {
        let original = "'; DROP TABLE users; --";
        let encoded = encode_str(original);
        let decoded = decode_str(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn fingerprint_matches_after_round_trip() {
        let original = "entrouter universal test payload";
        let fp = fingerprint_str(original);
        let encoded = encode_str(original);
        let result = verify(&encoded, &fp).unwrap();
        assert!(result.intact);
    }

    #[test]
    fn fingerprint_detects_mutation() {
        let original = "original payload";
        let fp = fingerprint_str(original);
        // Simulate a layer mutating the data
        let mutated = encode_str("mutated payload");
        let result = verify(&mutated, &fp);
        assert!(result.is_err());
    }

    #[test]
    fn no_special_chars_in_encoded() {
        let nasty = r#"{"key":"val\"ue","arr":[1,2,3],"nested":{"a":"b\\c"}}"#;
        let encoded = encode_str(nasty);
        // Base64 only contains A-Z, a-z, 0-9, +, /, =
        // None of these trigger escaping in HTTP, JSON, Redis, or Postgres
        assert!(encoded.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn envelope_full_pipeline_simulation() {
        use crate::envelope::Envelope;

        let original = r#"winner_token: abc"123"\n\t special stuff"#;

        // Entry point — wrap it
        let env = Envelope::wrap(original);

        // Simulate passing through HTTP → JSON → Redis → Postgres
        // by serialising to JSON string and back (the worst offender)
        let as_json = serde_json::to_string(&env).unwrap();
        let from_json: Envelope = serde_json::from_str(&as_json).unwrap();

        // Exit point — unwrap and verify
        let result = from_json.unwrap_verified().unwrap();
        assert_eq!(result, original);
    }
}
