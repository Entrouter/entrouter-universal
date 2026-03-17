#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub intact:      bool,
    pub decoded:     Vec<u8>,
    pub fingerprint: String,
}
