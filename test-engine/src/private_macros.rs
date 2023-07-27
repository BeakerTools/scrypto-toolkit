#[macro_export]
macro_rules! manifest_args {
    ($($args: expr),*) => {{
        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder.write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX).unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size(count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        let value = manifest_decode(&buf).unwrap();
        ManifestArgs::new_from_tuple_or_panic(value)
    }};
}
