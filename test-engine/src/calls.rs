use std::collections::BTreeSet;
use std::vec::Vec;

use radix_engine::transaction::{TransactionReceipt, TransactionResult};
use radix_engine::types::{
    manifest_decode, ComponentAddress, Decimal, Encoder, ManifestArgs, ManifestEncoder,
    ManifestExpression, ManifestValueKind, NonFungibleLocalId, PackageAddress, ResourceAddress,
    FAUCET, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX,
};
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
    output_manifest: Option<(String, String)>,
    admin_badge: Option<(ResourceAddress, Option<BTreeSet<NonFungibleLocalId>>)>,
    object_names: ManifestObjectNames,
    with_trace: bool,
}

impl<'a> CallBuilder<'a> {
    pub fn execute(mut self) -> TransactionReceipt {
        self.write_lock();
        self.write_deposit();
        self.write_badge();
        self.output_manifest();

        let receipt = self.test_engine.execute_call(
            self.manifest,
            self.with_trace,
            vec![self.caller.proof()],
            true,
        );

        Self::output_logs(&receipt);

        receipt
    }

    pub(crate) fn execute_no_update(mut self) -> TransactionReceipt {
        self.write_lock();
        self.write_deposit();
        self.write_badge();
        self.output_manifest();

        let receipt = self.test_engine.execute_call(
            self.manifest,
            self.with_trace,
            vec![self.caller.proof()],
            false,
        );

        Self::output_logs(&receipt);

        receipt
    }

    /// Deposits the batch to the given account.
    ///
    /// # Arguments
    /// * `account`: reference name of the account to which deposit the batch.
    pub fn deposit_batch<E: EnvRef>(mut self, account: E) -> Self {
        self.deposit_destination = *self.test_engine.get_account(account);
        self
    }

    /// Locks fees.
    ///
    /// # Arguments
    /// * `locker`: reference name of the component that will pay the fees.
    /// * `amount`: amount of fees to lock.
    pub fn lock_fee<E: EnvRef, D: TryInto<Decimal>>(mut self, locker: E, amount: D) -> Self
    where
        <D as TryInto<Decimal>>::Error: std::fmt::Debug,
    {
        self.fee_payer = self.test_engine.get_component_ref(locker);
        self.fee_locked = amount.try_into().unwrap();
        self
    }

    /// Outputs the manifest to the given path.
    ///
    /// # Arguments
    /// * `path`: path where to output the manifest.
    /// * `name`: name of the outputted file.
    pub fn output(mut self, path: impl ToString, name: impl ToString) -> Self {
        self.output_manifest = Some((path.to_string(), name.to_string()));
        self
    }

    /// Calls the method with the given admin badge.
    ///
    /// # Arguments
    /// * `badge_name` : reference name of the resource used as admin badge.
    pub fn with_badge<E: EnvRef>(mut self, badge_name: E) -> Self {
        let resource = self.test_engine.get_resource(badge_name);
        let ids_tree: Option<BTreeSet<NonFungibleLocalId>> = if resource.is_fungible() {
            None
        } else {
            Some(
                self.test_engine
                    .ids_owned_at_address(resource)
                    .into_iter()
                    .collect(),
            )
        };

        self.admin_badge = Some((resource, ids_tree));
        self
    }

    /// Displays trace or not.
    ///
    /// # Arguments
    /// * `trace`:
    pub fn with_trace(mut self, trace: bool) -> Self {
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
            manifest = arg.encode(test_engine, manifest, &mut encoder, *caller.address());
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest = manifest.call_method(component, method_name, manifest_arg);

        let object_names = manifest.object_names().clone();
        let deposit_destination = *caller.address();
        let transaction_manifest = manifest.build();

        Self {
            caller,
            manifest: transaction_manifest,
            test_engine,
            fee_payer: FAUCET,
            fee_locked: dec!(5000),
            deposit_destination,
            output_manifest: None,
            object_names,
            admin_badge: None,
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
            manifest = arg.encode(test_engine, manifest, &mut encoder, *caller.address());
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest =
            manifest.call_function(package_address, blueprint_name, function_name, manifest_arg);

        let object_names = manifest.object_names();
        let deposit_destination = *caller.address();

        Self {
            caller,
            manifest: manifest.build(),
            test_engine,
            fee_payer: FAUCET,
            fee_locked: dec!(5000),
            deposit_destination,
            output_manifest: None,
            object_names,
            admin_badge: None,
            with_trace: false,
        }
    }

    fn write_lock(&mut self) {
        self.manifest.instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.fee_payer),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(self.fee_locked).resolve(),
            },
        );
    }

    fn write_deposit(&mut self) {
        self.manifest
            .instructions
            .push(transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(*self.caller.address()),
                method_name: "deposit_batch".to_string(),
                args: manifest_args!(ManifestExpression::EntireWorktop).resolve(),
            });
    }
    fn write_badge(&mut self) {
        match &self.admin_badge {
            None => {}
            Some((badge, opt_ids)) => {
                if badge.is_fungible() {
                    self.manifest.instructions.insert(
                        1,
                        transaction::model::InstructionV1::CallMethod {
                            address: DynamicGlobalAddress::from(*self.caller.address()),
                            method_name: "create_proof_of_amount".to_string(),
                            args: manifest_args!(badge, Decimal::one()).resolve(),
                        },
                    )
                } else {
                    self.manifest.instructions.insert(
                        1,
                        transaction::model::InstructionV1::CallMethod {
                            address: DynamicGlobalAddress::from(*self.caller.address()),
                            method_name: "create_proof_of_non_fungibles".to_string(),
                            args: manifest_args!(badge, opt_ids.clone().unwrap()).resolve(),
                        },
                    );
                }
            }
        }
    }

    fn output_manifest(&self) {
        match &self.output_manifest {
            None => {}
            Some((path, name)) => {
                match dump_manifest_to_file_system(
                    self.object_names.clone(),
                    &self.manifest,
                    path,
                    Some(name),
                    &self.test_engine.network(),
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("Error when outputting manifest: {:?}", error);
                    }
                }
            }
        }
    }

    fn output_logs(receipt: &TransactionReceipt) {
        if let TransactionResult::Commit(commit_result) = &receipt.result {
            if !commit_result.application_logs.is_empty() {
                println!("\nApplication logs:");
                for (level, message) in &commit_result.application_logs {
                    println!("| [{level}]: {message}")
                }
            }
        }
    }
}
