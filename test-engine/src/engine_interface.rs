use std::collections::BTreeMap;
use std::path::Path;

use radix_engine::prelude::btreeset;
use radix_engine::transaction::{CostingParameters, ExecutionConfig, TransactionReceipt};
use radix_engine::types::{
    ComponentAddress, Decimal, Encoder, Epoch, GlobalAddress, NonFungibleLocalId, ResourceAddress,
    Secp256k1PublicKey,
};
use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::prelude::{
    AddressBech32Decoder, ManifestAddressReservation, ManifestExpression, ScryptoDecode,
    RESOURCE_PACKAGE,
};
use radix_engine_common::to_manifest_value_and_unwrap;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::{
    BlueprintId, FromPublicKey, FungibleResourceManagerCreateWithInitialSupplyManifestInput,
    FungibleResourceRoles, MetadataValue, NonFungibleGlobalId, OwnerRole,
    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
    FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
};
use scrypto_unit::{DefaultTestRunner, TestRunnerBuilder};
use transaction::builder::ManifestBuilder;
use transaction::model::{InstructionV1, TransactionManifestV1};
use transaction::prelude::{
    DynamicGlobalAddress, PreAllocatedAddress, Secp256k1PrivateKey, TestTransaction,
};

use crate::account::Account;
use crate::manifest_args;

pub struct EngineInterface {
    test_runner: DefaultTestRunner,
}

impl EngineInterface {
    pub fn new() -> Self {
        Self {
            test_runner: TestRunnerBuilder::new().without_trace().build(),
        }
    }

    pub fn publish_package<P: AsRef<Path>>(&mut self, package_dir: P) -> TransactionReceipt {
        self.test_runner.try_publish_package(package_dir.as_ref())
    }

    pub fn publish_compiled_package(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
            .build();

        self.test_runner.execute_manifest(manifest, vec![])
    }

    pub fn new_account(&mut self) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        self.test_runner.new_account(false)
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let nonce = self.test_runner.next_transaction_nonce();
        let exec_config = ExecutionConfig::for_test_transaction().with_kernel_trace(with_trace);

        self.test_runner.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            CostingParameters::default(),
            exec_config,
        )
    }

    pub fn get_metadata(&mut self, address: GlobalAddress, key: &str) -> Option<MetadataValue> {
        self.test_runner.get_metadata(address, key)
    }

    pub fn nft_ids(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Vec<NonFungibleLocalId> {
        let account_vault = self
            .test_runner
            .get_component_vaults(account, resource_address);
        let account_vault = account_vault.first();
        account_vault.map_or(vec![], |vault_id| {
            match self.test_runner.inspect_non_fungible_vault(*vault_id) {
                None => vec![],
                Some((_amount, ids)) => ids.collect(),
            }
        })
    }

    pub fn balance(&mut self, account: ComponentAddress, resource: ResourceAddress) -> Decimal {
        self.test_runner.get_component_balance(account, resource)
    }

    pub fn new_fungible(
        &mut self,
        account: ComponentAddress,
        initial_amount: Decimal,
    ) -> ResourceAddress {
        self.test_runner
            .create_fungible_resource(initial_amount, 18, account)
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.test_runner.set_current_epoch(epoch);
    }

    pub fn get_epoch(&mut self) -> Epoch {
        self.test_runner.get_current_epoch()
    }

    pub fn create_pre_allocated_token(
        &mut self,
        address: &str,
        initial_supply: Decimal,
        network_definition: NetworkDefinition,
        default_account: &Account,
    ) -> ResourceAddress {
        let dec = AddressBech32Decoder::new(&network_definition);
        let mut pre_allocated_addresses: Vec<PreAllocatedAddress> = Vec::new();

        let resource_addr: GlobalAddress = GlobalAddress::try_from_bech32(&dec, address).unwrap();

        pre_allocated_addresses.push(
            (
                BlueprintId {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                resource_addr,
            )
                .into(),
        );

        let receipt = self
            .test_runner
            .execute_system_transaction_with_preallocated_addresses(
                vec![
                    InstructionV1::CallFunction {
                        package_address: RESOURCE_PACKAGE.into(),
                        blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                        function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                            .to_string(),
                        args: to_manifest_value_and_unwrap!(
                            &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                                owner_role: OwnerRole::None,
                                divisibility: 18,
                                track_total_supply: false,
                                metadata: Default::default(),
                                resource_roles: FungibleResourceRoles::default(),
                                initial_supply,
                                address_reservation: Some(ManifestAddressReservation(0)),
                            }
                        ),
                    },
                    InstructionV1::CallMethod {
                        address: DynamicGlobalAddress::Static(GlobalAddress::new_or_panic(
                            (*default_account.address()).into(),
                        )),
                        method_name: "deposit_batch".to_string(),
                        args: manifest_args!(ManifestExpression::EntireWorktop).into(),
                    },
                ],
                pre_allocated_addresses,
                btreeset!(NonFungibleGlobalId::from_public_key(
                    &default_account.public_key()
                )),
            );

        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn get_state<T: ScryptoDecode>(&self, component_address: ComponentAddress) -> T {
        self.test_runner.component_state(component_address)
    }
}
