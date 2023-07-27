#[macro_export]
macro_rules! env_args {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use sdt_test_engine::environment::EnvironmentEncode;
         let mut temp_vec: Vec<Box<dyn EnvironmentEncode>> = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
        temp_vec
    }};
}