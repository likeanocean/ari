// :: collection macros.

// a helper macro that returns the number of items passed to it.
#[macro_export]
macro_rules! _ari_item_count {
    (=a $($x: tt)*) => (
        ()
    );

    ($($items: expr),*) => (
        <[()]>::len(&[
            $(
                $crate::_ari_item_count![=a $items]
            ), *
        ])
    );
}

/// creates a `HashMap` containing the specified entries.
///
/// # examples.
///
/// ```
/// # use ari::hash_map;
///
/// let map = hash_map![
///     "key_1" => "value_1",
///     "key_2" => "value_2",
/// ];
///
/// assert_eq!(map["key_1"], "value_1");
/// assert_eq!(map["key_2"], "value_2");
/// ```
#[macro_export]
macro_rules! hash_map {
    ($($key: expr => $value: expr),* $(,)*) => ({
        let count      = $crate::_ari_item_count!($($key), *);
        let mut object = ::std::collections::HashMap::with_capacity(count);

        $(
            object.insert($key, $value);
        )*

        object
    });
}

/// creates a `BTreeMap` containing the specified entries.
///
/// # examples.
///
/// ```
/// # use ari::btree_map;
///
/// let map = btree_map![
///     "key_1" => "value_1",
///     "key_2" => "value_2",
/// ];
///
/// assert_eq!(map["key_1"], "value_1");
/// assert_eq!(map["key_2"], "value_2");
/// ```
#[macro_export]
macro_rules! btree_map {
    ($($key: expr => $value: expr),* $(,)*) => ({
        let mut object = ::std::collections::BTreeMap::new();

        $(
            object.insert($key, $value);
        )*

        object
    });
}

/// creates a `HashSet` containing the specified values.
///
/// # examples.
///
/// ```
/// # use ari::hash_set;
///
/// let map = hash_set![1, 2, 3, 4];
///
/// assert_eq!(map.contains(&1), true);
/// assert_eq!(map.contains(&2), true);
/// assert_eq!(map.contains(&3), true);
/// assert_eq!(map.contains(&4), true);
/// assert_eq!(map.contains(&5), false);
/// ```
#[macro_export]
macro_rules! hash_set {
    ($($key: expr),* $(,)*) => ({
        let count      = $crate::_ari_item_count!($($key), *);
        let mut object = ::std::collections::HashSet::with_capacity(count);

        $(
            object.insert($key);
        )*

        object
    });
}

/// creates a `BTreeSet` containing the specified values.
///
/// # examples.
///
/// ```
/// # use ari::btree_set;
///
/// let map = btree_set![1, 2, 3, 4];
///
/// assert_eq!(map.contains(&1), true);
/// assert_eq!(map.contains(&2), true);
/// assert_eq!(map.contains(&3), true);
/// assert_eq!(map.contains(&4), true);
/// assert_eq!(map.contains(&5), false);
/// ```
#[macro_export]
macro_rules! btree_set {
    ($($key: expr),* $(,)*) => ({
        let mut object = ::std::collections::BTreeSet::new();

        $(
            object.insert($key);
        )*

        object
    });
}
