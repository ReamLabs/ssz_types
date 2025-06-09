use ethereum_hashing::{hash_fixed, ZERO_HASHES, ZERO_HASHES_MAX_INDEX};
use tree_hash::{Hash256, MerkleHasher, TreeHash, TreeHashType};
use typenum::Unsigned;

/// A helper function providing common functionality between the `TreeHash` implementations for
/// `FixedVector` and `VariableList`.
pub fn vec_tree_hash_root<T, N>(vec: &[T]) -> Hash256
where
    T: TreeHash,
    N: Unsigned,
{
    match T::tree_hash_type() {
        TreeHashType::Basic => {
            let mut hasher = MerkleHasher::with_leaves(
                (N::to_usize() + T::tree_hash_packing_factor() - 1) / T::tree_hash_packing_factor(),
            );

            for item in vec {
                hasher
                    .write(&item.tree_hash_packed_encoding())
                    .expect("ssz_types variable vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz_types variable vec should not have a remaining buffer")
        }
        TreeHashType::Container | TreeHashType::List | TreeHashType::Vector => {
            let mut hasher = MerkleHasher::with_leaves(N::to_usize());

            for item in vec {
                hasher
                    .write(item.tree_hash_root().as_slice())
                    .expect("ssz_types vec should not contain more elements than max");
            }

            hasher
                .finish()
                .expect("ssz_types vec should not have a remaining buffer")
        }
    }
}

pub fn extend_root(original_root: Hash256, from_height: usize, to_height: usize) -> Hash256 {
    assert!(to_height <= ZERO_HASHES_MAX_INDEX,
            "ZERO_HASHES_MAX_INDEX does not support up to the target height");

    assert!(from_height < to_height,
            "Target height must be greater than original height");

    let mut current_root = original_root;

    for level in from_height..to_height {
        let empty_sibling = Hash256::from_slice(&ZERO_HASHES[level]);
        current_root = combine_hashes(&current_root, &empty_sibling);
    }

    current_root
}

fn combine_hashes(left: &Hash256, right: &Hash256) -> Hash256 {
    let mut combined = [0u8; 64];

    combined[..32].copy_from_slice(left.as_slice());
    combined[32..].copy_from_slice(right.as_slice());

    Hash256::from_slice(&hash_fixed(&combined))
}

#[cfg(test)]
mod test {
    use crate::tree_hash::*;

    fn extend_and_compare_root(items: &[u8], from_height: u32, to_height: u32) {
        let original_root = tree_hash::merkle_root(items, usize::pow(2, from_height));
        let extended_root = extend_root(original_root, from_height as usize, to_height as usize);

        let expected_root = tree_hash::merkle_root(items, usize::pow(2, to_height));
        assert_eq!(extended_root, expected_root);
    }

    #[test]
    fn extending_root() {
        extend_and_compare_root(&[], 1, 2);
        extend_and_compare_root(&[100; 4], 4, 8);
        extend_and_compare_root(&[255; 2000000], 29, 40);
    }
}
