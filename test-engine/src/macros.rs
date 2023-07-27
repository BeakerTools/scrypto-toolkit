#[macro_export]
macro_rules! env_args {
    () => {{
        let ret: Vec<Decimal> = Vec::new();
        ret
    }};

     ($( $x:expr ),*) => {{
        let mut temp_vec = vec![];
            $(
                temp_vec.push($x);
            )*
        temp_vec
    }};
}