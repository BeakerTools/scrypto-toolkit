use std::collections::{BTreeMap, BTreeSet};

use radix_engine::types::{
    ComponentAddress, Decimal, Hash, HashMap, HashSet, IndexMap, IndexSet, NonFungibleGlobalId,
    NonFungibleLocalId, PackageAddress, PreciseDecimal, ResourceAddress,
};
use radix_engine::types::{Encode, ManifestCustomValueKind, ValueKind};
use radix_engine::types::{Encoder, ManifestEncoder};
use radix_engine_interface::blueprints::resource::OwnerRole;
use radix_engine_interface::count;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;
use transaction::prelude::Categorize;

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

impl<E: EnvRef + Clone> Environment<E> {
    fn to_encode<'a>(
        &self,
        test_engine: &TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    ) {
        match self {
            Environment::Account(address) => {
                let account = *test_engine.get_account(address.clone());
                (manifest_builder, Box::new(account))
            }
            Environment::Component(address) => {
                let component = test_engine.get_component(address.clone());
                (manifest_builder, Box::new(component))
            }
            Environment::Package(address) => {
                let package = test_engine.get_package(address.clone());
                (manifest_builder, Box::new(package))
            }
            Environment::FungibleBucket(resource, amount) => {
                let resource_address = test_engine.get_resource(resource.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address, amount),
                );
                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount: *amount,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            Environment::NonFungibleBucket(resource, ids) => {
                let resource_address = test_engine.get_resource(resource.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw_non_fungibles",
                    manifest_args!(resource_address, ids.clone()),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            Environment::FungibleProof(resource, amount) => {
                let resource_address = test_engine.get_resource(resource.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_amount",
                    manifest_args!(resource_address, amount),
                );
                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfAmount {
                        amount: *amount,
                        resource_address,
                    },
                );
                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
            Environment::NonFungibleProof(resource, ids) => {
                let resource_address = test_engine.get_resource(resource.clone());
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_non_fungibles",
                    manifest_args!(resource_address, ids.clone()),
                );
                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
            Environment::Resource(resource) => {
                let resource_address = test_engine.get_resource(resource.clone());
                (manifest_builder, Box::new(resource_address))
            }
        }
    }
}

impl<E: EnvRef + Clone> EnvironmentEncode for Environment<E> {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (manifest_builder, encoded) = self.to_encode(test_engine, manifest_builder, caller);
        encoder.encode(encoded.as_ref()).expect("Could not encode");
        manifest_builder
    }
}

impl<E: EnvRef + Clone> EnvironmentEncode for Vec<Environment<E>> {
    fn encode(
        &self,
        test_engine: &TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let mut manifest_builder = manifest_builder;

        encoder.write_value_kind(ValueKind::Array).expect("");
        let size = self.len();
        let mut encoded = Vec::new();
        for elem in self {
            let (mb, encode) = elem.to_encode(test_engine, manifest_builder, caller);
            manifest_builder = mb;
            encoded.push(encode);
        }

        let mut encoded = encoded.iter();
        match encoded.next() {
            None => {
                encoder.write_value_kind(ValueKind::I8).unwrap();
                encoder.write_size(size).expect("");
            }
            Some(elem) => {
                let encode = elem.as_ref();
                encode.encode_value_kind(encoder).expect("Error");
                encoder.write_size(size).expect("");
                encoder.encode_deeper_body(encode).expect("");
            }
        }

        for elem in encoded {
            encoder.encode_deeper_body(elem.as_ref()).expect("OK");
        }
        manifest_builder
    }
}
macro_rules! type_impl {
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

type_impl!(u8);
type_impl!(u16);
type_impl!(u32);
type_impl!(u64);
type_impl!(u128);
type_impl!(i8);
type_impl!(i16);
type_impl!(i32);
type_impl!(i64);
type_impl!(i128);
type_impl!(String);
type_impl!(ComponentAddress);
type_impl!(PackageAddress);
type_impl!(ResourceAddress);
type_impl!(NonFungibleGlobalId);
type_impl!(NonFungibleLocalId);
type_impl!(Hash);
type_impl!(Decimal);
type_impl!(PreciseDecimal);
type_impl!(OwnerRole);

macro_rules! collection_impl {
    ($type:ident) => {
        impl<
                T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>>
                    + Categorize<ManifestCustomValueKind>,
            > EnvironmentEncode for $type<T>
        {
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

    ($type:ident, $( $path:path ),*) => {
        impl<
                T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>>
                    + Categorize<ManifestCustomValueKind> $(+ $path)*,
            > EnvironmentEncode for $type<T>
        {
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

collection_impl!(Vec);
collection_impl!(BTreeSet);
collection_impl!(HashSet, Ord, std::hash::Hash);
collection_impl!(IndexSet, std::hash::Hash);

macro_rules!double_collection_impl {
    ($type:ident, $( $path:path ),*) => {
        impl<
        K: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + Categorize<ManifestCustomValueKind> $(+ $path)*,
        V: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + Categorize<ManifestCustomValueKind>,
            > EnvironmentEncode for $type<K, V>
        {
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

double_collection_impl!(BTreeMap,);
double_collection_impl!(HashMap, Ord, std::hash::Hash);
double_collection_impl!(IndexMap, std::hash::Hash, Eq, PartialEq);
