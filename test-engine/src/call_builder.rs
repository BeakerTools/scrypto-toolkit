use std::collections::BTreeSet;
use std::vec::Vec;

use crate::account::Account;
use crate::environment::{EnvironmentEncode, Fungible, NonFungible};
use crate::internal_prelude::*;
use crate::method_call::SimpleMethodCaller;
use crate::references::{ComponentReference, GlobalReference, ReferenceName, ResourceReference};
use crate::test_engine::TestEngine;
use crate::to_id::ToId;

struct TransactionManifestData {
    transaction_manifest: TransactionManifestV1,
    object_names: ManifestObjectNames,
}

pub struct CallBuilder<'a> {
    caller: Account,
    manifest_builder: ManifestBuilder,
    fee_payer: ComponentAddress,
    fee_locked: Decimal,
    test_engine: &'a mut TestEngine,
    output_manifest: Option<(String, String)>,
    admin_badge: Vec<(ResourceAddress, Option<BTreeSet<NonFungibleLocalId>>)>,
    with_trace: bool,
    with_tx_fee: bool,
    log_title: Option<&'a str>,
    deposit_destination: ComponentAddress,
    manifest_data: Option<TransactionManifestData>,
}

impl<'a> CallBuilder<'a> {
    pub fn new(test_engine: &'a mut TestEngine) -> Self {
        let caller = test_engine.current_account().clone();

        Self {
            deposit_destination: *caller.address(),
            caller,
            manifest_builder: ManifestBuilder::new(),
            fee_payer: FAUCET,
            fee_locked: dec!(5000),
            test_engine,
            output_manifest: None,
            admin_badge: vec![],
            with_trace: false,
            manifest_data: None,
            with_tx_fee: false,
            log_title: None,
        }
    }

    pub fn with_test_engine<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut TestEngine) -> R,
    {
        f(self.test_engine)
    }

    pub fn with_manifest_builder<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ManifestBuilder) -> ManifestBuilder,
    {
        self.manifest_builder = f(self.manifest_builder);

        self
    }

    /// Creates a call builder for a method call of the current component and skip the transaction execution.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    pub fn call(self, method_name: &str, args: Vec<Box<dyn EnvironmentEncode>>) -> Self {
        let component = *self.test_engine.current_component();
        self.call_method_internal(component, method_name, args)
    }

    /// Creates a call builder for a method call of the given component and skip the transaction execution.
    ///
    /// # Arguments
    /// * `entity_name`: reference name or address of the entity to call the method on.
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    pub fn call_from<G: GlobalReference>(
        self,
        entity_name: G,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let component = entity_name.address(self.test_engine);
        self.call_method_internal(component, method_name, args)
    }

    /// Sets the current component.
    ///
    /// # Arguments
    /// * `name`: reference name of the component.
    pub fn set_current_component<E: ReferenceName>(self, name: E) -> Self {
        self.test_engine.set_current_component(name);
        self
    }

    /// Executes the call.
    pub fn execute(mut self) -> TransactionReceipt {
        self.manifest_data = Some(TransactionManifestData {
            object_names: self.manifest_builder.object_names().clone(),
            transaction_manifest: self.manifest_builder.build(),
        });

        self.manifest_builder = ManifestBuilder::new();

        self.write_lock();
        self.write_deposit();
        self.write_badge();
        self.output_manifest();

        let receipt = self.test_engine.execute_call(
            self.manifest_data.unwrap().transaction_manifest,
            self.with_trace,
            vec![self.caller.proof()],
            true,
        );

        if let Some(title) = self.log_title {
            Self::output_log_title(title);
        }

        Self::output_logs(&receipt);

        if self.with_tx_fee {
            Self::output_tx_fee(&receipt);
        }

        receipt
    }

    pub fn execute_and_expect_success(self) -> CommitResult {
        self.execute().expect_commit_success().clone()
    }

    pub fn execute_and_expect_failure(self) -> CommitResult {
        self.execute().expect_commit_failure().clone()
    }

    /// Deposits the batch to the given account.
    ///
    /// # Arguments
    /// * `account`: reference name of the account to which deposit the batch.
    pub fn deposit_batch<E: ReferenceName>(mut self, account: E) -> Self {
        self.deposit_destination = *self.test_engine.get_account(account);
        self
    }

    /// Locks fees.
    ///
    /// # Arguments
    /// * `locker`: reference name of the component that will pay the fees.
    /// * `amount`: amount of fees to lock.
    pub fn lock_fee<E: ComponentReference, D: TryInto<Decimal>>(
        mut self,
        locker: E,
        amount: D,
    ) -> Self
    where
        <D as TryInto<Decimal>>::Error: std::fmt::Debug,
    {
        self.fee_payer = locker.address(self.test_engine);
        self.fee_locked = amount.try_into().unwrap();
        self
    }

    /// Transfers fungible resources form the current account to the given recipient.
    ///
    /// # Arguments
    /// * `recipient`: resources to transfer to.
    /// * `resource`: reference name of the resource to transfer.
    /// * `amount`: amount to transfer.
    pub fn transfer<
        E: ReferenceName,
        R: ReferenceName + Clone + 'static,
        // D: TryInto<Decimal> + Clone + 'static,
    >(
        self,
        recipient: E,
        resource: R,
        amount: Decimal,
    ) -> Self
// where
    //     <D as TryInto<Decimal>>::Error: std::fmt::Debug,
    {
        self.call_from(
            recipient,
            "try_deposit_or_abort",
            vec![
                Box::new(Fungible::FromAccount(resource.clone(), amount)),
                Box::new(None::<u64>),
            ],
        )
    }

    /// Transfers non-fungible resources form the current account to the given recipient.
    ///
    /// # Arguments
    /// * `recipient`: resources to transfer to.
    /// * `resource`: reference name of the resource to transfer.
    /// * `ids`: ids to transfer.
    pub fn transfer_non_fungibles<E: ReferenceName, R: ReferenceName + Clone + 'static, T: ToId>(
        self,
        recipient: E,
        resource: R,
        ids: Vec<T>,
    ) -> Self {
        self.call_from(
            recipient,
            "try_deposit_or_abort",
            vec![
                Box::new(NonFungible::FromAccount(
                    resource,
                    ids.into_iter().map(|id| id.to_id()).collect(),
                )),
                Box::new(None::<u64>),
            ],
        )
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
    pub fn with_badge<R: ResourceReference>(mut self, badge: R) -> Self {
        let resource = badge.address(self.test_engine);
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

        self.admin_badge.push((resource, ids_tree));
        self
    }

    /// Withdraws resource from an account
    pub fn withdraw<R: ResourceReference>(mut self, resource: R, amount: Decimal) -> Self {
        let account = self.test_engine.current_account().address();
        let resource_address = resource.address(self.test_engine);
        self.manifest_builder = self.manifest_builder.call_method(
            *account,
            "withdraw",
            manifest_args!(resource_address, amount),
        );

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

    /// Displays tx fee or not.
    ///
    /// # Arguments
    /// * `trace`:
    pub fn with_log_tx_fee(mut self) -> Self {
        self.with_tx_fee = true;
        self
    }

    pub fn with_log_title(mut self, title: &'a str) -> Self {
        self.log_title = Some(title);
        self
    }

    pub(crate) fn call_method_internal(
        mut self,
        component: impl ResolvableGlobalAddress,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest_builder = self.manifest_builder;

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest_builder = arg.encode(
                self.test_engine,
                manifest_builder,
                &mut encoder,
                *self.caller.address(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest_builder = manifest_builder.call_method(component, method_name, manifest_arg);

        self.manifest_builder = manifest_builder;

        self
    }

    pub(crate) fn execute_no_update(mut self) -> TransactionReceipt {
        self.manifest_data = Some(TransactionManifestData {
            object_names: self.manifest_builder.object_names().clone(),
            transaction_manifest: self.manifest_builder.build(),
        });

        self.manifest_builder = ManifestBuilder::new();

        self.write_lock();
        self.write_deposit();
        self.write_badge();
        self.output_manifest();

        let receipt = self.test_engine.execute_call(
            self.manifest_data.unwrap().transaction_manifest,
            self.with_trace,
            vec![self.caller.proof()],
            false,
        );

        if let Some(title) = self.log_title {
            Self::output_log_title(title);
        }

        Self::output_logs(&receipt);

        if self.with_tx_fee {
            Self::output_tx_fee(&receipt);
        }

        receipt
    }

    pub(crate) fn call_function_internal(
        mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest_builder = self.manifest_builder;

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest_builder = arg.encode(
                self.test_engine,
                manifest_builder,
                &mut encoder,
                *self.caller.address(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let manifest_builder = manifest_builder.call_function(
            package_address,
            blueprint_name,
            function_name,
            manifest_arg,
        );

        self.manifest_builder = manifest_builder;

        self
    }

    fn write_lock(&mut self) {
        let manifest = &mut self.manifest_data.as_mut().unwrap().transaction_manifest;

        manifest.instructions.insert(
            0,
            InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.fee_payer),
                method_name: "lock_fee".to_string(),
                args: manifest_args!(self.fee_locked).resolve(),
            },
        );
    }

    fn write_deposit(&mut self) {
        let manifest = &mut self.manifest_data.as_mut().unwrap().transaction_manifest;

        manifest.instructions.push(InstructionV1::CallMethod {
            address: DynamicGlobalAddress::from(*self.caller.address()),
            method_name: "deposit_batch".to_string(),
            args: manifest_args!(ManifestExpression::EntireWorktop).resolve(),
        });
    }
    fn write_badge(&mut self) {
        let manifest = &mut self.manifest_data.as_mut().unwrap().transaction_manifest;
        for (badge, opt_ids) in &self.admin_badge {
            if badge.is_fungible() {
                manifest.instructions.insert(
                    1,
                    InstructionV1::CallMethod {
                        address: DynamicGlobalAddress::from(*self.caller.address()),
                        method_name: "create_proof_of_amount".to_string(),
                        args: manifest_args!(badge, Decimal::one()).resolve(),
                    },
                )
            } else {
                manifest.instructions.insert(
                    1,
                    InstructionV1::CallMethod {
                        address: DynamicGlobalAddress::from(*self.caller.address()),
                        method_name: "create_proof_of_non_fungibles".to_string(),
                        args: manifest_args!(badge, opt_ids.clone().unwrap()).resolve(),
                    },
                );
            }
        }
    }

    fn output_manifest(&mut self) {
        let manifest = self.manifest_data.as_mut().unwrap();

        match &self.output_manifest {
            None => {}
            Some((path, name)) => {
                match dump_manifest_to_file_system(
                    manifest.object_names.clone(),
                    &manifest.transaction_manifest,
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

    fn output_log_title(title: &str) {
        println!("\nTX: {title}");
    }

    fn output_tx_fee(receipt: &TransactionReceipt) {
        println!("Transaction fees:{:?}", receipt.fee_summary.total_cost());
    }
}

impl SimpleMethodCaller for CallBuilder<'_> {
    fn call_method(
        self,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        let component = *self.test_engine.current_component();
        self.call_method_internal(component, method_name, args)
            .execute()
    }

    fn call_method_from<G: GlobalReference>(
        self,
        entity_name: G,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        let component = entity_name.address(self.test_engine);
        self.call_method_internal(component, method_name, args)
            .execute()
    }

    fn call_method_with_badge<R: ResourceReference>(
        self,
        method_name: &str,
        admin_badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        let component = *self.test_engine.current_component();
        self.call_method_internal(component, method_name, args)
            .with_badge(admin_badge)
            .execute()
    }
}
