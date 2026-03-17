// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Entrouter Universal - Compression
//
//  Gzip before Base64. Transparent to the consumer.
//  Large payloads shrink before encoding - smaller wire size,
//  same integrity guarantees.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use std::io::{Write, Read};
use crate::UniversalError;

/// Maximum decompressed size (16 MiB) to guard against gzip bombs.
const MAX_DECOMPRESS_SIZE: usize = 16 * 1024 * 1024;

/// Gzip compress bytes
pub fn compress(input: &[u8]) -> Result<Vec<u8>, UniversalError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(input)
        .map_err(|e| UniversalError::CompressError(e.to_string()))?;
    encoder.finish()
        .map_err(|e| UniversalError::CompressError(e.to_string()))
}

/// Gzip decompress bytes with a size guard against gzip bombs.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, UniversalError> {
    let mut decoder = GzDecoder::new(input);
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = decoder.read(&mut buf)
            .map_err(|e| UniversalError::CompressError(e.to_string()))?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        if out.len() > MAX_DECOMPRESS_SIZE {
            return Err(UniversalError::CompressError(
                format!("decompressed size exceeds {} byte limit", MAX_DECOMPRESS_SIZE),
            ));
        }
    }
    Ok(out)
}
