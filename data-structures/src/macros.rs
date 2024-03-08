/// A macro for convenient creation of `BigVec` instances.
///
/// This macro provides a convenient way to create instances of `BigVec`.
///
/// # Examples
///
/// Creating an empty `BigVec`:
///
/// ```no_code
/// let empty_vec = big_vec!();
/// ```
///
/// Creating a `BigVec` with elements:
///
/// ```no_code
/// let vec_with_elements = big_vec![1, 2, 3, 4, 5];
/// ```
#[macro_export]
macro_rules! big_vec {
    () => {
        $crate::big_vec::BigVec::new();
    };
    ($($x:expr),* $(,)?) => {
        $crate::big_vec::BigVec::from(vec![$($x),*])
    };
}
