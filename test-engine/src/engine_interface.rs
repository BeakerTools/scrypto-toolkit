use crate::account::Account;
use crate::internal_prelude::*;
use std::collections::BTreeMap;
use std::path::Path;

pub struct EngineInterface {
    simulator: DefaultLedgerSimulator,
}

impl EngineInterface {
    pub fn new() -> Self {
        let test_runner_builder = LedgerSimulatorBuilder::new()
            .with_custom_genesis(CustomGenesis::default(
                Epoch::of(1),
                CustomGenesis::default_consensus_manager_config(),
            ))
            .without_kernel_trace()
            .build();

        Self {
            simulator: test_runner_builder,
        }
    }

    pub fn new_with_custom_genesis(genesis: CustomGenesis) -> Self {
        let test_runner_builder = LedgerSimulatorBuilder::new()
            .with_custom_genesis(genesis)
            .without_kernel_trace()
            .build();
        Self {
            simulator: test_runner_builder,
        }
    }

    pub fn with_simulator<F, R>(&mut self, action: F) -> R
    where
        F: FnOnce(&mut DefaultLedgerSimulator) -> R,
    {
        action(&mut self.simulator)
    }

    pub fn publish_package<P: AsRef<Path>>(&mut self, package_dir: P) -> TransactionReceipt {
        self.simulator.try_publish_package(package_dir.as_ref())
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

        self.simulator.execute_manifest(manifest, vec![])
    }

    pub fn new_account(&mut self) -> (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress) {
        self.simulator.new_account(false)
    }

    pub fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let nonce = self.simulator.next_transaction_nonce();
        let exec_config = ExecutionConfig::for_test_transaction().with_kernel_trace(with_trace);

        self.simulator.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs.into_iter().collect()),
            exec_config,
        )
    }

    pub fn get_metadata(&mut self, address: GlobalAddress, key: &str) -> Option<MetadataValue> {
        self.simulator.get_metadata(address, key)
    }

    pub fn nft_ids(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> Vec<NonFungibleLocalId> {
        let account_vault = self
            .simulator
            .get_component_vaults(account, resource_address);
        let account_vault = account_vault.first();
        account_vault.map_or(vec![], |vault_id| {
            match self.simulator.inspect_non_fungible_vault(*vault_id) {
                None => vec![],
                Some((_amount, ids)) => ids.collect(),
            }
        })
    }

    pub fn balance(&mut self, account: ComponentAddress, resource: ResourceAddress) -> Decimal {
        self.simulator.get_component_balance(account, resource)
    }

    pub fn fungible_vault_balance(&mut self, vault: NodeId) -> Decimal {
        self.simulator
            .inspect_vault_balance(vault)
            .unwrap_or(Decimal::ZERO)
    }

    pub fn non_fungible_vault_balance(&mut self, vault: NodeId) -> Vec<NonFungibleLocalId> {
        let tmp = self
            .simulator
            .inspect_non_fungible_vault(vault)
            .unwrap_or((
                Decimal::ZERO,
                Box::new(Vec::<NonFungibleLocalId>::new().into_iter()),
            ))
            .1;
        tmp.collect()
    }

    pub fn new_fungible(
        &mut self,
        account: ComponentAddress,
        initial_amount: Decimal,
        divisibility: u8,
    ) -> ResourceAddress {
        self.simulator
            .create_fungible_resource(initial_amount, divisibility, account)
    }

    pub fn new_non_fungible<T: ManifestEncode + NonFungibleData>(
        &mut self,
        id_type: NonFungibleIdType,
    ) -> ResourceAddress {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                id_type,
                false,
                NonFungibleResourceRoles::single_locked_rule(AccessRule::AllowAll),
                metadata!(),
                None::<Vec<(NonFungibleLocalId, T)>>,
            )
            .build();
        let receipt = self.execute_manifest(manifest, false, vec![]);
        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn mint_non_fungible<T: ManifestEncode>(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        id: NonFungibleLocalId,
        data: T,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(resource_address, vec![(id, data)])
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();

        self.execute_manifest(manifest, false, vec![]);
    }

    pub fn mint_ruid_non_fungible<T: ManifestEncode>(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        data: T,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_ruid_non_fungible(resource_address, vec![data])
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();

        self.execute_manifest(manifest, false, vec![]);
    }

    pub fn set_epoch(&mut self, epoch: Epoch) {
        self.simulator.set_current_epoch(epoch);
    }

    pub fn get_epoch(&mut self) -> Epoch {
        self.simulator.get_current_epoch()
    }

    pub fn advance_time(&mut self, time: u64) {
        let current_time = self
            .simulator
            .get_current_time(TimePrecisionV2::Second)
            .seconds_since_unix_epoch;

        self.simulator
            .advance_to_round_at_timestamp(Round::of(1), (current_time + (time as i64)) * 1000);
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

        let receipt = self.simulator.execute_system_transaction(
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
            btreeset!(NonFungibleGlobalId::from_public_key(
                &default_account.public_key()
            )),
            pre_allocated_addresses,
        );

        receipt.expect_commit(true).new_resource_addresses()[0]
    }

    pub fn get_state<T: ScryptoDecode>(&self, component_address: ComponentAddress) -> T {
        self.simulator.component_state(component_address)
    }

    pub fn get_kvs_entry<K: ScryptoEncode, V: ScryptoEncode + ScryptoDecode>(
        &self,
        kv_store_id: Own,
        key: &K,
    ) -> Option<V> {
        self.simulator.get_kv_store_entry(kv_store_id, key)
    }

    pub fn get_non_fungible_data<T: NonFungibleData>(
        &mut self,
        resource_address: ResourceAddress,
        id: NonFungibleLocalId,
    ) -> T {
        self.simulator.get_non_fungible_data(resource_address, id)
    }
}
