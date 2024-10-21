use crate::internal_prelude::*;
use crate::references::{ReferenceName, ResourceReference};
use crate::test_engine::TestEngine;

pub trait ToEncode {
    fn to_encode<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    );
}

pub trait EnvironmentEncode {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder;
}

pub enum Environment<N: ReferenceName + Clone> {
    Account(N),
    Component(N),
    Package(N),
    Resource(N),
}

impl<N: ReferenceName + Clone> ToEncode for Environment<N> {
    fn to_encode<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        _caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    ) {
        match self {
            Environment::Resource(resource) => {
                let resource_address = test_engine.get_resource(resource.clone());
                (manifest_builder, Box::new(resource_address))
            }
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
        }
    }
}

impl<N: ReferenceName + Clone> EnvironmentEncode for Environment<N> {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (manifest_builder, encoded) = self.to_encode(test_engine, manifest_builder, caller);
        encoder.encode(encoded.as_ref()).expect("Could not encode");
        manifest_builder
    }
}

// Fungible

pub enum Fungible<R: ResourceReference + Clone> {
    FromAccount(R, Decimal),
    FromWorkTop(R, Decimal),
    AllFromAccount(R),
    AllFromWorktop(R),
}

impl<R: ResourceReference + Clone> ToEncode for Fungible<R> {
    fn to_encode<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    ) {
        match self {
            Fungible::FromAccount(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address, amount),
                );
                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            Fungible::FromWorkTop(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            Fungible::AllFromAccount(resource) => {
                let amount_owned = test_engine.current_balance(resource.clone());
                let resource_address = resource.address(test_engine);

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "withdraw",
                    manifest_args!(resource_address, amount_owned),
                );
                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeFromWorktop {
                        resource_address,
                        amount: amount_owned,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            Fungible::AllFromWorktop(resource) => {
                let resource_address = resource.address(test_engine);

                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeAllFromWorktop {
                        resource_address,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
        }
    }
}

impl<R: ResourceReference + Clone> EnvironmentEncode for Fungible<R> {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (manifest_builder, encoded) = self.to_encode(test_engine, manifest_builder, caller);
        encoder.encode(encoded.as_ref()).expect("Could not encode");
        manifest_builder
    }
}

// Non Fungible

pub enum NonFungible<R: ResourceReference + Clone> {
    FromAccount(R, Vec<NonFungibleLocalId>),
    FromWorktop(R, Vec<NonFungibleLocalId>),
    AllFromAccount(R),
    AllFromWorktop(R),
}

impl<R: ResourceReference + Clone> ToEncode for NonFungible<R> {
    fn to_encode<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    ) {
        match self {
            NonFungible::FromAccount(resource, ids) => {
                let resource_address = resource.address(test_engine);

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
            NonFungible::FromWorktop(resource, ids) => {
                let resource_address = resource.address(test_engine);
                let (manifest_builder, bucket) = manifest_builder.add_instruction_advanced(
                    InstructionV1::TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
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
                    InstructionV1::TakeNonFungiblesFromWorktop {
                        resource_address,
                        ids: ids_owned,
                    },
                );
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
            NonFungible::AllFromWorktop(resource) => {
                let resource_address = resource.address(test_engine);

                let (manifest_builder, bucket) =
                    manifest_builder.add_instruction_advanced(InstructionV1::TakeAllFromWorktop {
                        resource_address,
                    });
                (manifest_builder, Box::new(bucket.new_bucket.unwrap()))
            }
        }
    }
}

impl<R: ResourceReference + Clone> EnvironmentEncode for NonFungible<R> {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (manifest_builder, encoded) = self.to_encode(test_engine, manifest_builder, caller);
        encoder.encode(encoded.as_ref()).expect("Could not encode");
        manifest_builder
    }
}

// Proofs

pub enum ProofOf<R: ResourceReference + Clone> {
    FungibleFromAccount(R, Decimal),
    FungibleFromAuthZone(R, Decimal),
    NonFungibleFromAccount(R, Vec<NonFungibleLocalId>),
    NonFungibleFromAuthZone(R, Vec<NonFungibleLocalId>),
}

impl<R: ResourceReference + Clone> ToEncode for ProofOf<R> {
    fn to_encode<'a>(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        caller: ComponentAddress,
    ) -> (
        ManifestBuilder,
        Box<dyn Encode<ManifestCustomValueKind, ManifestEncoder<'a>>>,
    ) {
        match self {
            ProofOf::FungibleFromAccount(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_amount",
                    manifest_args!(resource_address, amount),
                );

                let (manifest_builder, proof) =
                    manifest_builder.add_instruction_advanced(InstructionV1::PopFromAuthZone);

                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
            ProofOf::FungibleFromAuthZone(resource, amount) => {
                let resource_address = resource.address(test_engine);
                let amount = *amount;

                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfAmount {
                        resource_address,
                        amount,
                    },
                );
                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
            ProofOf::NonFungibleFromAccount(resource, ids) => {
                let resource_address = resource.address(test_engine);
                let manifest_builder = manifest_builder.call_method(
                    caller,
                    "create_proof_of_non_fungibles",
                    manifest_args!(resource_address, ids.clone()),
                );

                let (manifest_builder, proof) =
                    manifest_builder.add_instruction_advanced(InstructionV1::PopFromAuthZone);

                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
            ProofOf::NonFungibleFromAuthZone(resource, ids) => {
                let resource_address = resource.address(test_engine);
                let (manifest_builder, proof) = manifest_builder.add_instruction_advanced(
                    InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                        resource_address,
                        ids: ids.clone(),
                    },
                );
                (manifest_builder, Box::new(proof.new_proof.unwrap()))
            }
        }
    }
}

impl<R: ResourceReference + Clone> EnvironmentEncode for ProofOf<R> {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (manifest_builder, encoded) = self.to_encode(test_engine, manifest_builder, caller);
        encoder.encode(encoded.as_ref()).expect("Could not encode");
        manifest_builder
    }
}

// Env Vec

pub struct EnvVec {
    elements: Vec<Box<dyn ToEncode>>,
}

impl EnvVec {
    pub fn from_vec(elements: Vec<Box<dyn ToEncode>>) -> Self {
        Self { elements }
    }
}

impl EnvironmentEncode for EnvVec {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let mut manifest_builder = manifest_builder;

        encoder.write_value_kind(ValueKind::Array).expect("");
        let size = self.elements.len();
        let mut encoded = Vec::new();
        for elem in &self.elements {
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

pub struct EnvSome {
    element: Box<dyn ToEncode>,
}

impl EnvSome {
    pub fn new(element: Box<dyn ToEncode>) -> Self {
        Self { element }
    }
}

impl EnvironmentEncode for EnvSome {
    fn encode(
        &self,
        test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        caller: ComponentAddress,
    ) -> ManifestBuilder {
        let (mb, encode) = self
            .element
            .to_encode(test_engine, manifest_builder, caller);

        encoder.write_value_kind(ValueKind::Enum).expect("");
        encoder.write_discriminator(OPTION_VARIANT_SOME).expect("");
        encoder.write_size(1).expect("");
        encoder.encode(encode.as_ref()).expect("");

        mb
    }
}

impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>>> EnvironmentEncode for T {
    fn encode(
        &self,
        _test_engine: &mut TestEngine,
        manifest_builder: ManifestBuilder,
        encoder: &mut ManifestEncoder,
        _caller: ComponentAddress,
    ) -> ManifestBuilder {
        encoder.encode(&self).unwrap();
        manifest_builder
    }
}
