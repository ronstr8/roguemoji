//! Serde helpers for game state whose in-memory form doesn't fit JSON as is.

/// A map written as a list of `[key, value]` pairs, since JSON object keys
/// must be strings. Sorted by key, so a save's text doesn't depend on hashing.
pub mod pairs {
    use std::collections::HashMap;
    use std::hash::Hash;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<K, V, S>(map: &HashMap<K, V>, s: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize + Ord,
        V: Serialize,
        S: Serializer,
    {
        let mut list: Vec<(&K, &V)> = map.iter().collect();
        list.sort_by(|a, b| a.0.cmp(b.0));
        list.serialize(s)
    }

    pub fn deserialize<'de, K, V, D>(d: D) -> Result<HashMap<K, V>, D::Error>
    where
        K: Deserialize<'de> + Eq + Hash,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Ok(Vec::<(K, V)>::deserialize(d)?.into_iter().collect())
    }
}
