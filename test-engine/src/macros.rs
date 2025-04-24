#[macro_export]
macro_rules! env_args {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
         use test_engine::prelude::*;

         let mut temp_vec: Vec<Box<dyn ToValue>> = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
        temp_vec
    }};

}

#[macro_export]
macro_rules! env_tuple{
    () => (
        EnvTuple::new()
    );

    ($( $x:expr ),*) => {{
        use test_engine::prelude::*;

        let mut temp_vec: Vec<Box<dyn ToValue>> = vec![];
        $(
            temp_vec.push(Box::new($x));
        )*
        EnvTuple::from_vec(temp_vec)
    }};
}

#[macro_export]
macro_rules! env_vec {
    (@empty $kind:expr) => {{
        use test_engine::prelude::*;

        EnvVec::from_vec($kind,vec![])
    }};

    ($x0:expr $(, $x:expr)* $(,)?) => {{
        use test_engine::prelude::*;

        let mut temp_vec: Vec<Box<dyn ToValue>> = vec![Box::new($x0)];
        $(
            temp_vec.push(Box::new($x));
        )*
        EnvVec::from_vec(ManifestValueKind::Bool, temp_vec)
    }};
}

#[macro_export]
macro_rules! some {
    ($x:expr) => {{
        use test_engine::prelude::*;

        EnvOption::Some(Box::new($x))
    }};
}

#[macro_export]
macro_rules! none {
    () => {
        EnvOption::None
    };
}

#[macro_export]
macro_rules! env_map {
    (@empty $key_kind:expr , $value_kind:expr) => {{
        use test_engine::prelude::*;

        EnvMap::new($key_kind, $value_kind)
    }};

    ($($key:expr => $value:expr),*) => {{
        use test_engine::prelude::*;

        let mut temp_vec:Vec<(Box<dyn ToValue>, Box<dyn ToValue>)> = vec![];
        $(
            temp_vec.push((Box::new($key), Box::new($value)));
        )*
        EnvMap::from_vec(ManifestValueKind::Bool, ManifestValueKind::Bool, temp_vec)
    }};
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
macro_rules! global_package_from_binary {
    ($name:ident, $release_path:expr, $package_name:expr) => {
        lazy_static! {
            static ref $name: (Vec<u8>, PackageDefinition) = {
                let package_test_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
                let release_dir = package_test_dir.join($release_path);
                let package_name = if $package_name == "" {
                    env!("CARGO_PKG_NAME")
                } else {
                    $package_name
                };

                let wasm_path = release_dir.join(format!("{}.wasm", package_name));

                let definition_path = release_dir.join(format!("{}.rpd", package_name));

                let code = std::fs::read(&wasm_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read built WASM from path {:?} - {:?}",
                        &wasm_path, err
                    )
                });
                let definition = std::fs::read(&definition_path).unwrap_or_else(|err| {
                    panic!(
                        "Failed to read package definition from path {:?} - {:?}",
                        &definition_path, err
                    )
                });
                let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                    panic!(
                        "Failed to parse package definition from path {:?} - {:?}",
                        &definition_path, err
                    )
                });
                (code, definition)
            };
        }
    };
}

#[macro_export]
macro_rules! global_package_advanced {
    ($name:ident, $release_path:expr, $package_name:expr, $code_path:expr) => {
        lazy_static! {
            static ref $name: (Vec<u8>, PackageDefinition) = {
                let package_test_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();

                let package_dir = package_test_dir.join($code_path);

                let code_dir = package_dir.join("src");

                let release_dir = package_test_dir
                    .join($release_path)
                    .join("target")
                    .join("wasm32-unknown-unknown")
                    .join("release");

                let package_name = if $package_name == "" {
                    env!("CARGO_PKG_NAME")
                } else {
                    $package_name
                };

                let wasm_path = release_dir.join(format!("{}.wasm", package_name));

                let definition_path = release_dir.join(format!("{}.rpd", package_name));

                let code_modification_time =
                    get_last_modified_time(&code_dir).unwrap_or(SystemTime::UNIX_EPOCH);

                let wasm_modification_time =
                    get_last_modified_time(&wasm_path).unwrap_or(SystemTime::UNIX_EPOCH);

                if code_modification_time >= wasm_modification_time {
                    PackagePublishingSource::CompileAndPublishFromSource(
                        package_dir,
                        CompileProfile::Standard,
                    )
                    .code_and_definition()
                } else {
                    let code = std::fs::read(&wasm_path).unwrap_or_else(|err| {
                        panic!(
                            "Failed to read built WASM from path {:?} - {:?}",
                            &wasm_path, err
                        )
                    });
                    let definition = std::fs::read(&definition_path).unwrap_or_else(|err| {
                        panic!(
                            "Failed to read package definition from path {:?} - {:?}",
                            &definition_path, err
                        )
                    });
                    let definition = manifest_decode(&definition).unwrap_or_else(|err| {
                        panic!(
                            "Failed to parse package definition from path {:?} - {:?}",
                            &definition_path, err
                        )
                    });
                    (code, definition)
                }
            };
        }
    };

    ($name:ident) => {
        global_package_advanced!($name, "", "", "");
    };

    ($name:ident, $release_path:expr) => {
        global_package_advanced!($name, $release_path, "", "");
    };

    ($name:ident, $release_path:expr, $package_name:expr) => {
        global_package_advanced!($name, $release_path, $package_name, "");
    };
}
