#[macro_export]
macro_rules! env_args {
     ($( $x:expr ),*) => {{
        let mut temp_vec = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
        temp_vec
    }};
}