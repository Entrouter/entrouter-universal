// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Guardian
//
//  Watches a value through named pipeline layers.
//  If any layer mutates it, you know EXACTLY which one.
//
//  Usage:
//    let g = Guardian::new("my_token_value");
//    g.checkpoint("http_layer");
//    g.checkpoint("json_layer");
//    g.checkpoint("redis_layer");
//    g.checkpoint("postgres_layer");
//    g.assert_intact(); // panics if anything touched it
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use crate::{encode_str, decode_str, fingerprint_str, UniversalError};

#[derive(Debug, Clone)]
pub struct LayerRecord {
    pub layer:     String,
    pub encoded:   String,
    pub fingerprint: String,
    pub intact:    bool,
}

#[derive(Debug)]
pub struct Guardian {
    original:            String,
    original_fingerprint: String,
    encoded:             String,
    pub layers:          Vec<LayerRecord>,
}

impl Guardian {
    /// Create a new Guardian for a value.
    /// Encodes and fingerprints at the entry point.
    pub fn new(input: &str) -> Self {
        Self {
            original:             input.to_string(),
            original_fingerprint: fingerprint_str(input),
            encoded:              encode_str(input),
            layers:               Vec::new(),
        }
    }

    /// Record a checkpoint at a named layer.
    /// Pass the encoded string as it exists AT THAT LAYER.
    /// If a layer mutated it, this will record the mutation.
    pub fn checkpoint(&mut self, layer_name: &str, current_encoded: &str) {
        let decoded = decode_str(current_encoded).unwrap_or_default();
        let fp = fingerprint_str(&decoded);
        let intact = fp == self.original_fingerprint;

        self.layers.push(LayerRecord {
            layer:       layer_name.to_string(),
            encoded:     current_encoded.to_string(),
            fingerprint: fp,
            intact,
        });
    }

    /// Get the encoded string to pass through the pipeline.
    pub fn encoded(&self) -> &str {
        &self.encoded
    }

    /// Get the original fingerprint to compare against.
    pub fn original_fingerprint(&self) -> &str {
        &self.original_fingerprint
    }

    /// Find which layer first broke integrity.
    pub fn first_violation(&self) -> Option<&LayerRecord> {
        self.layers.iter().find(|l| !l.intact)
    }

    /// True if all checkpoints passed.
    pub fn is_intact(&self) -> bool {
        self.layers.iter().all(|l| l.intact)
    }

    /// Assert intact — useful in tests and debug builds.
    pub fn assert_intact(&self) {
        if let Some(violation) = self.first_violation() {
            panic!(
                "Entrouter Universal: integrity violation at layer '{}'\nExpected fingerprint: {}\nGot: {}",
                violation.layer,
                self.original_fingerprint,
                violation.fingerprint
            );
        }
    }

    /// Print a full pipeline report.
    pub fn report(&self) -> String {
        let mut out = String::new();
        out.push_str("━━━━ Entrouter Universal Pipeline Report ━━━━\n");
        out.push_str(&format!("Original fingerprint: {}\n", self.original_fingerprint));
        out.push_str(&format!("Overall intact: {}\n\n", self.is_intact()));

        for (i, layer) in self.layers.iter().enumerate() {
            let status = if layer.intact { "✅" } else { "❌ VIOLATED" };
            out.push_str(&format!(
                "  Layer {}: {} — {}\n",
                i + 1, layer.layer, status
            ));
            if !layer.intact {
                out.push_str(&format!(
                    "    Expected: {}\n    Got:      {}\n",
                    self.original_fingerprint, layer.fingerprint
                ));
            }
        }

        out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        out
    }
}

// ── Verify Result ─────────────────────────────────────────

pub use crate::verify::VerifyResult;
