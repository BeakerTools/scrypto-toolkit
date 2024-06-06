#[macro_export]
macro_rules! env_args {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use test_engine::prelude::*;

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
         use test_engine::prelude::*;

         let mut temp_vec: Vec<Box<dyn ToEncode>> = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
         EnvVec::from_vec(temp_vec)
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

#[macro_export]
macro_rules! nf_ids {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use test_engine::prelude::*;

         let mut temp_vec: Vec<NonFungibleLocalId> = vec![];
            $(
                temp_vec.push($x.to_id());
            )*
         temp_vec
    }};
}

#[macro_export]
macro_rules! none {
    () => {
        None::<u64>
    };
}

#[cfg(test)]
mod test_macros {
    use crate::prelude::*;

    macro_rules! integer_test {
        ($type_name: ident) => {
            let test = 12 as $type_name;
            assert_eq!(nf_ids!(test), vec![NonFungibleLocalId::integer(12 as u64)])
        };
    }

    #[test]
    fn test_nf_ids_int() {
        integer_test!(u8);
        integer_test!(u16);
        integer_test!(u32);
        integer_test!(u64);
        integer_test!(u128);
        integer_test!(i8);
        integer_test!(i16);
        integer_test!(i32);
        integer_test!(i64);
        integer_test!(i128);
    }

    #[test]
    fn test_nf_ids_from_string() {
        let str_1 = "#1#";
        let str_2 = "<SomeId>";
        let str_3 = "blabla";

        assert_eq!(nf_ids!(str_1), vec![NonFungibleLocalId::integer(1u64)]);
        assert_eq!(
            nf_ids!(str_2),
            vec![NonFungibleLocalId::string("SomeId").unwrap()]
        );
        assert_eq!(
            nf_ids!(str_3),
            vec![NonFungibleLocalId::string("blabla").unwrap()]
        )
    }
}
