#[macro_export]
macro_rules! env_args {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use test_engine::environment::EnvironmentEncode;
         let mut temp_vec: Vec<Box<dyn EnvironmentEncode>> = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
        temp_vec
    }};
}

#[macro_export]
macro_rules! env_vec {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use test_engine::environment::EnvVec;
         EnvVec::from_vec(vec![$($x,)*])
    }};
}

#[macro_export]
macro_rules! global_package {
    ($name:ident, $path:expr) => {
        use test_engine::prelude::*;

        lazy_static! {
            static ref $name: (Vec<u8>, PackageDefinition) =
                { PackagePublishingSource::from($path).code_and_definition() };
        }
    };
}
