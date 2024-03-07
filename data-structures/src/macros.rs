#[macro_export]
macro_rules! big_vec {
    () => {
        $crate::big_vec::BigVec::new()
    };
    ($($x:expr),* $(,)?) => {
        let tmp_vec = [$($x),*];
        $crate::big_vec::BigVec::from(tmp_vec)
    };
}
