use anchor_lang::prelude::*;
use solana_program::keccak;

/// Verify a Merkle proof
pub fn verify_merkle_proof(
    proof: &[[u8; 32]],
    root: &[u8; 32],
    leaf: [u8; 32],
) -> bool {
    let mut computed_hash = leaf;

    for proof_element in proof.iter() {
        computed_hash = if computed_hash <= *proof_element {
            hash_pair(computed_hash, *proof_element)
        } else {
            hash_pair(*proof_element, computed_hash)
        };
    }

    computed_hash == *root
}

/// Hash two leaf/node hashes together
fn hash_pair(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    let mut combined = [0u8; 64];
    combined[..32].copy_from_slice(&a);
    combined[32..].copy_from_slice(&b);
    keccak::hash(&combined).to_bytes()
}

/// Generate a leaf hash from a pubkey
pub fn get_leaf_hash(address: &Pubkey, amount: Option<u64>) -> [u8; 32] {
    if let Some(amt) = amount {
        // If amount is specified, include it in the leaf hash (for whitelist_limited case)
        let mut data = Vec::with_capacity(40);
        data.extend_from_slice(address.as_ref());
        data.extend_from_slice(&amt.to_le_bytes());
        keccak::hash(&data).to_bytes()
    } else {
        // Simple address-only hash
        keccak::hash(address.as_ref()).to_bytes()
    }
}
