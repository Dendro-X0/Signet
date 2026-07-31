use sha2::{Digest, Sha256};

/// SHA-256 fingerprint of DER certificate bytes, colon-separated uppercase hex.
pub fn fingerprint_sha256_colon(der: &[u8]) -> String {
    let digest = Sha256::digest(der);
    let hex = hex::encode_upper(digest);
    hex.as_bytes()
        .chunks(2)
        .map(|c| std::str::from_utf8(c).unwrap_or("??"))
        .collect::<Vec<_>>()
        .join(":")
}

pub fn fingerprint_from_cert_pem(pem: &str) -> anyhow::Result<String> {
    let der = pem_to_der(pem)?;
    Ok(fingerprint_sha256_colon(&der))
}

pub fn pem_to_der(pem: &str) -> anyhow::Result<Vec<u8>> {
    let (_, der) = x509_parser::pem::parse_x509_pem(pem.as_bytes())
        .map_err(|e| anyhow::anyhow!("invalid certificate PEM: {e}"))?;
    Ok(der.contents.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_format_length() {
        let fp = fingerprint_sha256_colon(&[0u8; 32]);
        // 32-byte digest → 64 hex chars → 32 pairs with 31 colons
        assert_eq!(fp.matches(':').count(), 31);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit() || c == ':'));
    }
}
