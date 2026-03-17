#[derive(Debug, Clone, PartialEq)]
pub struct VerifyResult {
    pub intact:      bool,
    pub decoded:     Vec<u8>,
    pub fingerprint: String,
}

impl std::fmt::Display for VerifyResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.intact {
            write!(f, "Intact (fp: {}...)", &self.fingerprint[..16])
        } else {
            write!(f, "Violated")
        }
    }
}
