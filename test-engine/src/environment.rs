use radix_engine::types::{
    ComponentAddress, Decimal, Hash, NonFungibleGlobalId, NonFungibleLocalId, PackageAddress,
    PreciseDecimal, ResourceAddress,
};
use radix_engine::types::{Encoder, ManifestEncoder};
use radix_engine_interface::blueprints::resource::OwnerRole;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;

use crate::environment_reference::EnvRef;
use crate::manifest_args;
use crate::test_engine::TestEngine;

pub trait EnvironmentEncode {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder;
}

pub enum Environment<E: EnvRef + Clone> {
    Account(E),
    Component(E),
    Package(E),
    FungibleBucket(E, Decimal),
    NonFungibleBucket(E, Vec<NonFungibleLocalId>),
    FungibleProof(E, Decimal),
    NonFungibleProof(E, Vec<NonFungibleLocalId>),
    Resource(E),
}

impl<E: EnvRef + Clone> EnvironmentEncode for Environment<E> {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        match self {
            Environment::Account(name) => {
                let account_address = test_engine.get_account(name.clone());
                encoder.encode(&account_address).unwrap();
                manifest_builder
            }
            Environment::Component(name) => {
                let component_address = test_engine.get_component(name.clone());
                encoder.encode(&component_address).unwrap();
                manifest_builder
            }
            Environment::Package(name) => {
                let package_address = test_engine.get_package(name.clone());
                encoder.encode(&package_address).unwrap();
                manifest_builder
            }
            Environment::FungibleBucket(resource_name, amount) => {
                let resource_address = test_engine.get_resource(resource_name.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address.clone(), amount),
                );
                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount: amount.clone(),
                    });
                encoder.encode(&(bucket.new_bucket.unwrap())).unwrap();
                manifest_builder
            }
            Environment::NonFungibleBucket(resource_name, ids) => {
                let resource_address = test_engine.get_resource(resource_name.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw_by_ids",
                    manifest_args!(resource_address.clone(), ids.clone()),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                encoder.encode(&(bucket.new_bucket.unwrap())).unwrap();
                manifest_builder
            }
            Environment::FungibleProof(resource_name, amount) => {
                let resource_address = test_engine.get_resource(resource_name.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_by_amount",
                    manifest_args!(resource_address.clone(), amount),
                );
                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfAmount {
                        amount: amount.clone(),
                        resource_address,
                    },
                );
                encoder.encode(&(proof.new_proof.unwrap())).unwrap();
                manifest_builder
            }
            Environment::NonFungibleProof(resource_name, ids) => {
                let resource_address = test_engine.get_resource(resource_name.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_by_ids",
                    manifest_args!(resource_address.clone(), ids.clone()),
                );
                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                encoder.encode(&(proof.new_proof.unwrap())).unwrap();
                manifest_builder
            }
            Environment::Resource(resource_name) => {
                let resource_address = test_engine.get_resource(resource_name.clone());
                encoder.encode(&resource_address).unwrap();
                manifest_builder
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
                manifest_builder: ManifestBuilder,
                encoder: &mut ManifestEncoder,
                _caller: ComponentAddress,
            ) -> ManifestBuilder {
                encoder.encode(&self).unwrap();
                manifest_builder
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
env_encode_impl!(OwnerRole);
