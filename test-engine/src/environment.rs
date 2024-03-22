use radix_engine::prelude::ValueKind;
use radix_engine::types::{ComponentAddress, Decimal, NonFungibleLocalId};
use radix_engine::types::{Encode, ManifestCustomValueKind};
use radix_engine::types::{Encoder, ManifestEncoder};
use radix_engine_interface::count;
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
            } /*
              Environment::Vec(items) => {
                  let mut ret = vec![];
                  let mut mb = manifest_builder;
                  for item in items {
                      let (new_mb, new_item) = item.to_encode(test_engine, mb, caller);
                      mb = new_mb;
                      ret.push(new_item);
                  }
                  (mb, Box::new(ret))
              }*/
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
        match self {
            /*
            Environment::Vec(elements) => {
                let mut manifest_builder = manifest_builder;

                encoder.write_value_kind(ValueKind::Array).expect("");
                let size = elements.len();
                let mut encoded = Vec::new();
                for elem in elements {
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
            }*/
            _ => {
                let (manifest_builder, encoded) =
                    self.to_encode(test_engine, manifest_builder, caller);
                encoder.encode(encoded.as_ref()).expect("Could not encode");
                manifest_builder
            }
        }
    }
}

pub struct EnvVec<E: EnvRef + Clone> {
    elements: Vec<Environment<E>>,
}

impl<E: EnvRef + Clone> EnvVec<E> {
    pub fn from_vec(elements: Vec<Environment<E>>) -> Self {
        Self { elements }
    }
}

impl<E: EnvRef + Clone> EnvironmentEncode for EnvVec<E> {
    fn encode(
        &self,
        test_engine: &TestEngine,
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

impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>>> EnvironmentEncode for T {
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
