use om9_core::retained::fingerprint;
use sha2::{Digest, Sha256};

#[test]
fn fingerprint_matches_hash_of_serialized_json_across_buffer_boundaries() {
    for value in [String::new(), "é\\\"".repeat(1400), "x".repeat(8192)] {
        let bytes = serde_json::to_vec(&value).unwrap();
        let expected: [u8; 32] = Sha256::digest(bytes).into();
        assert_eq!(fingerprint(&value).unwrap(), expected);
    }
}
