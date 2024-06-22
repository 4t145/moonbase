use std::hash::{DefaultHasher, Hasher};

/// shortcut to hash by [`DefaultHasher`]
pub fn hash<T: std::hash::Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
