//! Test: impl Iterator - impl Trait in return position
//!
//! Iron must transpile impl Trait return types correctly.

pub fn numbers() -> impl Iterator<Item = i32> {
    std::array::IntoIter::new([1, 2, 3])
}

pub fn evens(n: i32) -> impl Iterator<Item = i32> {
    (n..).step_by(2)
}

pub fn chars() -> impl Iterator<Item = char> {
    "hello".chars()
}

pub fn take_five<T>(iter: T) -> impl Iterator<Item = T::Item>
where
    T: Iterator,
{
    iter.take(5)
}
