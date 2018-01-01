use std::cmp::{Ordering, PartialOrd};


/// compares and returns the minimum of two values.
///
/// returns the first argument if the comparison determines them to be equal.
///
/// # examples.
///
/// ```
/// assert_eq!(1, ari::cmp::partial_min(1, 2));
/// assert_eq!(2, ari::cmp::partial_min(2, 2));
/// ```
pub fn partial_min<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    match a.partial_cmp(&b) {
        Some(Ordering::Less) | Some(Ordering::Equal) => a,
        Some(Ordering::Greater) | None => b,
    }
}

/// compares and returns the maximum of two values.
///
/// returns the second argument if the comparison determines them to be equal.
///
/// # examples.
///
/// ```
/// assert_eq!(2, ari::cmp::partial_max(1, 2));
/// assert_eq!(2, ari::cmp::partial_max(2, 2));
/// ```
pub fn partial_max<T>(a: T, b: T) -> T
where
    T: PartialOrd,
{
    match a.partial_cmp(&b) {
        Some(Ordering::Greater) | Some(Ordering::Equal) => a,
        Some(Ordering::Less) | None => b,
    }
}


/// a comparison function for floating point numbers.
///
/// nan values are ordered at the end.
///
/// # examples.
///
/// ```
/// # use std::cmp::Ordering;
///
/// assert_eq!(Ordering::Less, ari::cmp::compare_floating(&1.0, &2.0));
/// assert_eq!(Ordering::Equal, ari::cmp::compare_floating(&2.0, &2.0));
/// ```
pub fn compare_floating<T>(a: &T, b: &T) -> Ordering
where
    T: Float + Copy + PartialOrd,
{
    match (a, b) {
        (x, y) if x.is_nan() && y.is_nan() => Ordering::Equal,
        (x, _) if x.is_nan() => Ordering::Greater,
        (_, y) if y.is_nan() => Ordering::Less,
        (_, _) => a.partial_cmp(b).unwrap(),
    }
}


pub trait Float {
    fn is_nan(self) -> bool;
}

impl Float for f32 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}

impl Float for f64 {
    fn is_nan(self) -> bool {
        self.is_nan()
    }
}
