/// returns all keys in an `Iterator<Item = (Key, Value)>`.
pub fn keys<K, V>(source: impl IntoIterator<Item = (K, V)>) -> impl Iterator<Item = K> {
    source.into_iter().map(|(k, _)| k)
}

/// returns all values in an `Iterator<Item = (Key, Value)>`.
pub fn values<K, V>(source: impl IntoIterator<Item = (K, V)>) -> impl Iterator<Item = V> {
    source.into_iter().map(|(_, v)| v)
}
