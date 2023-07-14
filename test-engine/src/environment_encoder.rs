use crate::test_engine::TestEngine;
use radix_engine::types::{
    ComponentAddress, Decimal, Hash, NonFungibleGlobalId, NonFungibleLocalId, PackageAddress,
    PreciseDecimal, ResourceAddress,
};
use radix_engine::types::{Encoder, ManifestEncoder};
use transaction::builder::ManifestBuilder;

pub trait EnvironmentEncode {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: &mut ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    );
}

macro_rules! env_encode_impl {
    ($type:ident) => {
        impl EnvironmentEncode for $type {
            fn encode(
                &self,
                _test_engine: &TestEngine,
                _manifest_builder: &mut ManifestBuilder,
                encoder: &mut ManifestEncoder,
                _caller: ComponentAddress,
            ) {
                encoder.encode(&self).unwrap();
            }
        }
    };
}

env_encode_impl!(u8);
env_encode_impl!(u16);
env_encode_impl!(u32);
env_encode_impl!(u64);
env_encode_impl!(u128);
env_encode_impl!(i8);
env_encode_impl!(i16);
env_encode_impl!(i32);
env_encode_impl!(i64);
env_encode_impl!(i128);
env_encode_impl!(String);
env_encode_impl!(ComponentAddress);
env_encode_impl!(PackageAddress);
env_encode_impl!(ResourceAddress);
env_encode_impl!(NonFungibleGlobalId);
env_encode_impl!(NonFungibleLocalId);
env_encode_impl!(Hash);
env_encode_impl!(Decimal);
env_encode_impl!(PreciseDecimal);
