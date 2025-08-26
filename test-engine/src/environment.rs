use crate::internal_prelude::*;
use crate::references::{ReferenceName, ResourceReference};
use crate::test_engine::TestEngine;
use std::vec::Vec;

pub trait ToValue {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue);
}

// ! Refs

// macro to implement ToValue of env reference types
macro_rules! env_to_value_impl {
    ($name:ident, $getter:ident) => {
        impl<N: ReferenceName + Clone> ToValue for $name<N> {
            fn to_value<'a>(
                &self,
                test_engine: &mut TestEngine,
                manifest_builder: ManifestBuilder,
                _caller: ComponentAddress,
            ) -> (ManifestBuilder, ManifestValue) {
                let address = test_engine.$getter(self.0.clone());
                (
                    manifest_builder,
                    Value::Custom {
                        value: ManifestCustomValue::Address((address.clone()).into()),
                    },
                )
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct EnvResource<N: ReferenceName + Clone>(pub N);
env_to_value_impl!(EnvResource, get_resource);

#[derive(Clone, Debug)]
pub struct EnvAccount<N: ReferenceName + Clone>(pub N);
env_to_value_impl!(EnvAccount, get_account);

#[derive(Clone, Debug)]
pub struct EnvComponent<N: ReferenceName + Clone>(pub N);
env_to_value_impl!(EnvComponent, get_component);

pub struct EnvPackage<N: ReferenceName + Clone>(pub N);
env_to_value_impl!(EnvPackage, get_package);

pub enum Environment<N: ReferenceName + Clone> {
    Account(N),
    Component(N),
    Package(N),
    Resource(N),
}

impl<N: ReferenceName + Clone> ToValue for Environment<N> {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        _caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let value = match self {
            Environment::Resource(resource) => {
                ManifestCustomValue::Address(test_engine.get_resource(resource.clone()).into())
            }
            Environment::Account(address) => {
                ManifestCustomValue::Address((*test_engine.get_account(address.clone())).into())
            }
            Environment::Component(address) => {
                ManifestCustomValue::Address(test_engine.get_component(address.clone()).into())
            }
            Environment::Package(address) => {
                ManifestCustomValue::Address(test_engine.get_package(address.clone()).into())
            }
        };

        (manifest_builder, Value::Custom { value })
    }
}

// ! Fungible

#[derive(Clone, Debug)]
pub enum Fungible<R: ResourceReference + Clone> {
    FromAccount(R, Decimal),
    FromWorkTop(R, Decimal),
    AllFromAccount(R),
    AllFromWorktop(R),
}

impl<R: ResourceReference + Clone> Fungible<R> {
    pub fn to_manifest_bucket<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestBucket) {
        match self {
            Fungible::FromAccount(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address, amount),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeFromWorktop(TakeFromWorktop {
                        resource_address,
                        amount,
                    }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
            Fungible::FromWorkTop(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let (manifest_builder, symbols) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeFromWorktop(TakeFromWorktop {
                        resource_address,
                        amount,
                    }),
                );

                (manifest_builder, symbols.new_bucket.unwrap())
            }
            Fungible::AllFromAccount(resource) => {
                let amount_owned = test_engine.current_balance(resource.clone());
                let resource_address = resource.address(test_engine);

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address, amount_owned),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeFromWorktop(TakeFromWorktop {
                        resource_address,
                        amount: amount_owned,
                    }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
            Fungible::AllFromWorktop(resource) => {
                let resource_address = resource.address(test_engine);

                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeAllFromWorktop(TakeAllFromWorktop { resource_address }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
        }
    }
}

impl<R: ResourceReference + Clone> ToValue for Fungible<R> {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let manifest_bucket = self.to_manifest_bucket(test_engine, manifest_builder, caller);

        (
            manifest_bucket.0,
            Value::Custom {
                value: ManifestCustomValue::Bucket(manifest_bucket.1),
            },
        )
    }
}

// ! Non Fungible

pub enum NonFungible<R: ResourceReference + Clone> {
    FromAccount(R, Vec<NonFungibleLocalId>),
    FromWorktop(R, Vec<NonFungibleLocalId>),
    AllFromAccount(R),
    AllFromWorktop(R),
}

impl<R: ResourceReference + Clone> NonFungible<R> {
    fn to_manifest_bucket<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestBucket) {
        match self {
            NonFungible::FromAccount(resource, ids) => {
                let resource_address = resource.address(test_engine);

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw_non_fungibles",
                    manifest_args!(resource_address, ids.clone()),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop(TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
            NonFungible::FromWorktop(resource, ids) => {
                let resource_address = resource.address(test_engine);
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop(TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }

            NonFungible::AllFromAccount(resource) => {
                let ids_owned = test_engine.current_ids_balance(resource.clone());
                let resource_address = resource.address(test_engine);

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw_non_fungibles",
                    manifest_args!(resource_address, ids_owned.clone()),
                );
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop(TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids_owned,
                    }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
            NonFungible::AllFromWorktop(resource) => {
                let resource_address = resource.address(test_engine);

                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeAllFromWorktop(TakeAllFromWorktop { resource_address }),
                );
                (manifest_builder, bucket.new_bucket.unwrap())
            }
        }
    }
}

impl<R: ResourceReference + Clone> ToValue for NonFungible<R> {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let manifest_bucket = self.to_manifest_bucket(test_engine, manifest_builder, caller);

        (
            manifest_bucket.0,
            Value::Custom {
                value: ManifestCustomValue::Bucket(manifest_bucket.1),
            },
        )
    }
}

// ! Proofs

pub enum ProofOf<R: ResourceReference + Clone> {
    FungibleFromAccount(R, Decimal),
    FungibleFromAuthZone(R, Decimal),
    NonFungibleFromAccount(R, Vec<NonFungibleLocalId>),
    NonFungibleFromAuthZone(R, Vec<NonFungibleLocalId>),
}

impl<R: ResourceReference + Clone> ToValue for ProofOf<R> {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let (manifest_builder, proof) = match self {
            ProofOf::FungibleFromAccount(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_amount",
                    manifest_args!(resource_address, amount),
                );

                manifest_builder
                    .add_instruction_advanced(InstructionV1::PopFromAuthZone(PopFromAuthZone))
            }

            ProofOf::FungibleFromAuthZone(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfAmount(
                        CreateProofFromAuthZoneOfAmount {
                            resource_address,
                            amount,
                        },
                    ),
                )
            }
            ProofOf::NonFungibleFromAccount(resource, ids) => {
                let resource_address = resource.address(test_engine);
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_non_fungibles",
                    manifest_args!(resource_address, ids.clone()),
                );

                manifest_builder
                    .add_instruction_advanced(InstructionV1::PopFromAuthZone(PopFromAuthZone))
            }
            ProofOf::NonFungibleFromAuthZone(resource, ids) => {
                let resource_address = resource.address(test_engine);
                manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfNonFungibles(
                        CreateProofFromAuthZoneOfNonFungibles {
                            resource_address,
                            ids: ids.clone(),
                        },
                    ),
                )
            }
        };

        (
            manifest_builder,
            Value::Custom {
                value: ManifestCustomValue::Proof(proof.new_proof.unwrap()),
            },
        )
    }
}

// ! Env Vec

pub struct EnvVec {
    value_kind: ManifestValueKind,
    elements: Vec<Box<dyn ToValue>>,
}

impl EnvVec {
    pub fn from_vec(value_kind: ManifestValueKind, elements: Vec<Box<dyn ToValue>>) -> Self {
        Self {
            value_kind,
            elements,
        }
    }

    pub fn new(value_kind: ManifestValueKind) -> Self {
        Self {
            value_kind,
            elements: Vec::new(),
        }
    }

    pub fn extend(&mut self, elements: EnvVec) {
        self.elements.extend(elements.elements);
    }

    pub fn push(&mut self, element: Box<dyn ToValue>) {
        self.elements.push(element);
    }

    pub fn pop(&mut self) -> Option<Box<dyn ToValue>> {
        self.elements.pop()
    }
}

impl Iterator for EnvVec {
    type Item = Box<dyn ToValue>;

    fn next(&mut self) -> Option<Self::Item> {
        self.elements.pop()
    }
}

impl ToValue for EnvVec {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let (manifest_builder, vec) = self.elements.iter().fold(
            (manifest_builder, Vec::new()),
            |(manifest_builder, mut vec), element| {
                let (manifest_builder, element_value) =
                    element.to_value(test_engine, manifest_builder, caller);

                vec.push(element_value);

                (manifest_builder, vec)
            },
        );

        let value_kind = if let Some(first) = vec.first() {
            get_value_kind(first)
        } else {
            self.value_kind
        };

        let value = Value::Array {
            element_value_kind: value_kind,
            elements: vec,
        };

        (manifest_builder, value)
    }
}

// ! Env Tuple

pub struct EnvTuple {
    elements: Vec<Box<dyn ToValue>>,
}

impl EnvTuple {
    pub fn from_vec(elements: Vec<Box<dyn ToValue>>) -> Self {
        Self { elements }
    }

    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

impl ToValue for EnvTuple {
    fn to_value<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let (manifest_builder, tuple) = self.elements.iter().fold(
            (manifest_builder, Vec::new()),
            |(manifest_builder, mut tuple), element| {
                let (manifest_builder, element_value) =
                    element.to_value(test_engine, manifest_builder, caller);
                tuple.push(element_value);
                (manifest_builder, tuple)
            },
        );

        let value = Value::Tuple { fields: tuple };

        (manifest_builder, value)
    }
}

// ! Env map

pub struct EnvMap {
    key_kind: ManifestValueKind,
    value_kind: ManifestValueKind,
    elements: Vec<(Box<dyn ToValue>, Box<dyn ToValue>)>,
}

impl EnvMap {
    pub fn from_vec(
        key_kind: ManifestValueKind,
        value_kind: ManifestValueKind,
        elements: Vec<(Box<dyn ToValue>, Box<dyn ToValue>)>,
    ) -> Self {
        Self {
            key_kind,
            value_kind,
            elements,
        }
    }

    pub fn new(key_kind: ManifestValueKind, value_kind: ManifestValueKind) -> Self {
        Self {
            key_kind,
            value_kind,
            elements: Vec::new(),
        }
    }
}

impl ToValue for EnvMap {
    fn to_value(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let (manifest_builder, entries) = self.elements.iter().fold(
            (manifest_builder, Vec::new()),
            |(manifest_builder, mut entries), (key, value)| {
                let (manifest_builder, key_value) =
                    key.to_value(test_engine, manifest_builder, caller);
                let (manifest_builder, value_value) =
                    value.to_value(test_engine, manifest_builder, caller);
                entries.push((key_value, value_value));
                (manifest_builder, entries)
            },
        );

        let (key_kind, value_kind) = if let Some(first) = entries.first() {
            (get_value_kind(&first.0), get_value_kind(&first.1))
        } else {
            (self.key_kind, self.value_kind)
        };

        let value = Value::Map {
            key_value_kind: key_kind,
            value_value_kind: value_kind,
            entries,
        };

        (manifest_builder, value)
    }
}

// ! Env Option

pub enum EnvOption {
    None,
    Some(Box<dyn ToValue>),
}

impl ToValue for EnvOption {
    fn to_value(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let (manifest_builder, value) = match self {
            EnvOption::Some(element) => {
                let (manifest_builder, inner_value) =
                    element.to_value(test_engine, manifest_builder, caller);

                (
                    manifest_builder,
                    Value::Enum {
                        discriminator: 1,
                        fields: vec![inner_value],
                    },
                )
            }
            EnvOption::None => (
                manifest_builder,
                Value::Enum {
                    discriminator: 0,
                    fields: vec![],
                },
            ),
        };

        (manifest_builder, value)
    }
}

// Other encoding types

impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + ?Sized> ToValue for T {
    fn to_value(
        &self,
        _test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        _caller: ComponentAddress,
    ) -> (ManifestBuilder, ManifestValue) {
        let mut buf = sbor::rust::vec::Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();

        encoder.encode(&self).unwrap();

        let value = manifest_decode(&buf).unwrap();

        (manifest_builder, value)
    }
}

fn get_value_kind(value: &ManifestValue) -> ManifestValueKind {
    match value {
        Value::Bool { .. } => ValueKind::Bool,
        Value::I8 { .. } => ValueKind::I8,
        Value::I16 { .. } => ValueKind::I16,
        Value::I32 { .. } => ValueKind::I32,
        Value::I64 { .. } => ValueKind::I64,
        Value::I128 { .. } => ValueKind::I128,
        Value::U8 { .. } => ValueKind::U8,
        Value::U16 { .. } => ValueKind::U16,
        Value::U32 { .. } => ValueKind::U32,
        Value::U64 { .. } => ValueKind::U64,
        Value::U128 { .. } => ValueKind::U128,
        Value::String { .. } => ValueKind::String,
        Value::Enum { .. } => ValueKind::Enum,
        Value::Array { .. } => ValueKind::Array,
        Value::Tuple { .. } => ValueKind::Tuple,
        Value::Map { .. } => ValueKind::Map,
        Value::Custom { value } => ValueKind::Custom(value.get_custom_value_kind()),
    }
}
