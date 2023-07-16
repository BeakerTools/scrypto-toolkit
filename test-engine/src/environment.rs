use crate::test_engine::TestEngine;
use radix_engine::types::{
    ComponentAddress, Decimal, Hash, NonFungibleGlobalId, NonFungibleLocalId, PackageAddress,
    PreciseDecimal, ResourceAddress,
};
use radix_engine::types::{Encoder, ManifestEncoder};
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;
use transaction::prelude::manifest_args;

pub trait EnvironmentEncode {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: &mut ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    );
}

pub enum Environment<F: Formattable + Clone> {
    Account(F),
    Component(F),
    Package(F),
    FungibleBucket(F, Decimal),
    NonFungibleBucket(F, Vec<NonFungibleLocalId>),
    FungibleProof(F, Decimal),
    NonFungibleProof(F, Vec<NonFungibleLocalId>),
}

impl<F: Formattable + Clone> EnvironmentEncode for Environment<F> {
    fn encode(&self, test_engine: &TestEngine, manifest_builder: &mut ManifestBuilder, encoder: &mut ManifestEncoder, caller: ComponentAddress) {
        match self {
            Environment::Account(name) => {
                let account_address = test_engine.get_account(name.clone());
                encoder.encode(&account_address).unwrap();
            }
            Environment::Component(name) => {
                let component_address = test_engine.get_component(name.clone());
                encoder.encode(&component_address).unwrap();
            }
            Environment::Package(name) => {
                let package_address = test_engine.get_package(name.clone());
                encoder.encode(&package_address).unwrap();
            }
            Environment::FungibleBucket(resource_name, amount) => {
                let resource_address = test_engine.get_fungible(resource_name.clone());
                manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address.clone(), amount),
                );
                let (_, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount: amount.clone(),
                    });
                encoder.encode(&(bucket.unwrap())).unwrap();
            }
            Environment::NonFungibleBucket(resource_name, ids) => {
                let resource_address = test_engine.get_fungible(resource_name.clone());
                manifest_builder.call_method(
                    caller,
                    "withdraw_by_ids",
                    manifest_args!(resource_address.clone(), set_ids.clone()),
                );
                let (_, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    });
                encoder.encode(&(bucket.unwrap())).unwrap();
            }
            Environment::FungibleProof(resource_name, amount) => {
                let resource_address = test_engine.get_fungible(resource_name.clone());
                manifest_builder.call_method(
                    caller,
                    "create_proof_by_amount",
                    manifest_args!(resource_address.clone(), amount),
                );
                let (_, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfAmount {
                        amount: amount.clone(),
                        resource_address,
                    },
                );
                encoder.encode(&(proof.unwrap())).unwrap();
            }
            Environment::NonFungibleProof(resource_name, ids) => {
                let resource_address = test_engine.get_fungible(resource_name.clone());
                manifest_builder.call_method(
                    caller,
                    "create_proof_by_ids",
                    manifest_args!(resource_address.clone(), set_ids.clone()),
                );
                let (_, proof) =
                    manifest_builder.add_instruction_advanced(InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                        resource_address,
                        ids: ids.clone(),
                    });
                encoder.encode(&(proof.unwrap())).unwrap();
            }
        }
    }
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
