// ── Verify Result ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// True if the data arrived intact
    pub intact:      bool,
    /// The decoded raw bytes
    pub decoded:     Vec<u8>,
    /// The fingerprint that was verified
    pub fingerprint: String,
}
