use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub(crate) struct Checksum(String);

impl Checksum {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Checksum {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> Checksum {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(7 + digest.len() * 2);
    encoded.push_str("sha256:");
    append_lower_hex(&mut encoded, &digest);
    Checksum(encoded)
}

fn append_lower_hex(target: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        target.push(HEX[(byte >> 4) as usize] as char);
        target.push(HEX[(byte & 0x0f) as usize] as char);
    }
}

#[cfg(test)]
mod tests {
    use super::sha256;

    #[test]
    fn matches_known_vector() {
        assert_eq!(
            sha256(b"abc").as_str(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
