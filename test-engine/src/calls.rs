use std::path::Path;
use std::vec::Vec;

use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::{ComponentAddress, Decimal, Encoder, FAUCET, manifest_decode, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX, ManifestArgs, ManifestEncoder, ManifestExpression, ManifestValueKind, NetworkDefinition, PackageAddress};
use transaction::builder::ManifestBuilder;
use transaction::manifest::decompiler::ManifestObjectNames;
use transaction::manifest::dumper::dump_manifest_to_file_system;
use transaction::prelude::{dec, DynamicGlobalAddress, ResolvableArguments, TransactionManifestV1};

use crate::account::Account;
use crate::environment::EnvironmentEncode;
use crate::environment_reference::EnvRef;
use crate::manifest_args;
use crate::test_engine::TestEngine;

pub struct CallBuilder<'a> {
    caller: Account,
    manifest: TransactionManifestV1,
    fee_payer: ComponentAddress,
    fee_locked: Decimal,
    deposit_destination: ComponentAddress,
    test_engine: &'a mut TestEngine,
    output_manifest: Option<(dyn AsRef<Path>, String)>,
    object_names: ManifestObjectNames,
    with_trace: bool,
}

impl<'a> CallBuilder<'a> {
    pub fn execute(mut self) -> TransactionReceipt {
        self.write_lock();
        self.write_deposit();
        self.output_manifest();

        self.test_engine
            .execute_call(self.manifest, self.with_trace, vec![self.caller.proof()])
    }

    /// Deposits the batch to the given account.
    ///
    /// # Arguments
    /// * `account`: reference name of the account to which deposit the batch.
    pub fn deposit_batch<E: EnvRef>(mut self, account: E) -> Self{
        self.deposit_destination = self.test_engine.get_account(account).clone();
        self
    }

    /// Locks fees.
    ///
    /// # Arguments
    /// * `locker`: reference name of the component that will pay the fees.
    /// * `amount`: amount of fees to lock.
    pub fn lock_fee<E: EnvRef>(mut self, locker: E, amount: Decimal) -> Self{
        self.fee_payer = self.test_engine.get_component_ref(locker);
        self.fee_locked = amount;
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, path:P, name: impl ToString) -> Self{
        self.output_manifest = Some((path, name.to_string()));
        self
    }

    /// Displays trace or not.
    ///
    /// # Arguments
    /// * `trace`:
    pub fn with_trace(mut self, trace: bool) -> Self{
        self.with_trace = trace;
        self
    }

    pub(crate) fn call_method(
        test_engine: &'a mut TestEngine,
        caller: Account,
        component: ComponentAddress,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest = ManifestBuilder::new();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest = arg.encode(
                &test_engine,
                manifest,
                &mut encoder,
                caller.address().clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest = manifest
            .call_method(component, method_name, manifest_arg);

        let object_names = manifest.object_names();

        Self {
            caller,
            manifest: manifest.build(),
            test_engine,
            fee_payer: FAUCET,
            fee_locked: dec!(5000),
            deposit_destination: caller.address().clone(),
            output_manifest: None,
            object_names,
            with_trace: false,
        }
    }

    pub(crate) fn call_function(
        test_engine: &'a mut TestEngine,
        caller: Account,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest = ManifestBuilder::new();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest = arg.encode(
                &test_engine,
                manifest,
                &mut encoder,
                caller.address().clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest = manifest
            .call_function(package_address, blueprint_name, function_name, manifest_arg);

        let object_names = manifest.object_names();

        Self {
            caller,
            manifest: manifest.build(),
            test_engine,
            fee_payer: FAUCET,
            fee_locked: dec!(5000),
            deposit_destination: caller.address().clone(),
            output_manifest: None,
            object_names,
            with_trace: false,
        }
    }

    fn write_lock(&mut self) {
        self.manifest.instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.fee_payer.clone()),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(self.fee_locked).resolve(),
            },
        );
    }

    fn write_deposit(&mut self) {
        self.manifest
            .instructions
            .push(transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.caller.address().clone()),
                method_name: "deposit_batch".to_string(),
                args: manifest_args!(ManifestExpression::EntireWorktop).resolve(),
            });
    }

    fn output_manifest(&self) {
        match &self.output_manifest{
            None => {},
            Some((path, name)) => {
                match dump_manifest_to_file_system(&self.manifest, self.object_names.clone(), path, Some(name),  &NetworkDefinition::kisharnet()) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("Error when outputting manifest: {:?}", error);
                    }
                }
            }
        }
    }
}
