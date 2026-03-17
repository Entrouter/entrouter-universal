// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Envelope
//
//  The container that travels through the pipeline.
//  Wraps your data at the entry point, verifies at exit.
//
//  Structure:
//  {
//    "d": "<base64 encoded data>",   ← opaque to every layer
//    "f": "<sha256 fingerprint>",    ← travels alongside
//    "v": 1                          ← version for future changes
//  }
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use serde::{Deserialize, Serialize};
use crate::{encode_str, decode_str, fingerprint_str, UniversalError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Base64 encoded data — opaque to every layer
    pub d: String,
    /// SHA-256 fingerprint of the original raw input
    pub f: String,
    /// Version
    pub v: u8,
}

impl Envelope {
    /// Wrap data at the entry point.
    /// Call this once. Pass the Envelope through everything.
    pub fn wrap(input: &str) -> Self {
        Self {
            d: encode_str(input),
            f: fingerprint_str(input),
            v: 1,
        }
    }

    /// Unwrap and verify at the exit point.
    /// If anything mutated the data in transit, this returns an error.
    pub fn unwrap_verified(&self) -> Result<String, UniversalError> {
        let decoded = decode_str(&self.d)?;
        let actual_fp = fingerprint_str(&decoded);

        if actual_fp != self.f {
            return Err(UniversalError::IntegrityViolation {
                expected: self.f.clone(),
                actual: actual_fp,
            });
        }

        Ok(decoded)
    }

    /// Unwrap without verification — use when you trust the source
    /// but still need the decoded value.
    pub fn unwrap_raw(&self) -> Result<String, UniversalError> {
        decode_str(&self.d)
    }

    /// Check if the envelope is intact without consuming it.
    pub fn is_intact(&self) -> bool {
        self.unwrap_verified().is_ok()
    }

    /// Get the fingerprint — useful for logging/debugging.
    pub fn fingerprint(&self) -> &str {
        &self.f
    }

    /// Serialize to a JSON string safe to store anywhere.
    pub fn to_json(&self) -> Result<String, UniversalError> {
        serde_json::to_string(self)
            .map_err(|e| UniversalError::MalformedEnvelope(e.to_string()))
    }

    /// Deserialize from a JSON string.
    pub fn from_json(s: &str) -> Result<Self, UniversalError> {
        serde_json::from_str(s)
            .map_err(|e| UniversalError::MalformedEnvelope(e.to_string()))
    }
}
