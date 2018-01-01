use num_traits::{Float, One, Zero};
use std::ops::{Add, Mul};


/// peforms a linear interpolation between `a` and `b`.
pub fn lerp<T, U>(a: T, b: T, amount: U) -> T
where
    T: Add<Output = T> + Mul<U, Output = T>,
    U: Float + One,
{
    a * (U::one() - amount) + b * amount
}

/// peforms a linear interpolation between `a` and `b`, returning a value that is bounded to `[a, b]`.
pub fn bounded_lerp<T, U>(a: T, b: T, amount: U) -> T
where
    T: Add<Output = T> + Mul<U, Output = T>,
    U: Float + Zero + One,
{
    lerp(a, b, match amount {
        amount if amount < U::zero() => U::zero(),
        amount if amount > U::one() => U::one(),
        amount => amount,
    })
}
