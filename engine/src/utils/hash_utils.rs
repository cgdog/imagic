use std::hash::{DefaultHasher, Hash, Hasher};

pub struct HashUtils {

}

impl HashUtils {
    pub fn combine_hashes_fast(hashes: &[u64]) -> u64 {
        hashes.iter().fold(0, |acc, &hash| acc ^ hash)
    }

    pub fn combine_hashes(hashes: &[u64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for &hash in hashes {
            hash.hash(&mut hasher);
        }
        hasher.finish()
    }
}