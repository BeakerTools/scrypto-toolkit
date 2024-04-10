use std::collections::hash_map::Entry;
use std::path::Path;

use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use radix_engine::types::{
    dec, ComponentAddress, Decimal, GlobalAddress, HashMap, NonFungibleLocalId, PackageAddress,
    ResourceAddress, FAUCET, XRD,
};
use radix_engine_common::prelude::{Own, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::prelude::{MetadataValue, NonFungibleGlobalId};
use transaction::model::TransactionManifestV1;
use transaction::prelude::NetworkDefinition;

use crate::account::Account;
use crate::calls::CallBuilder;
use crate::engine_interface::EngineInterface;
use crate::environment::EnvironmentEncode;
use crate::environment_reference::{EntityRef, EnvRef, ResourceRef};
use crate::receipt_traits::Outcome;

pub struct TestEngine {
    engine_interface: EngineInterface,
    accounts: HashMap<String, Account>,
    current_account: String,
    packages: HashMap<String, PackageAddress>,
    current_package: Option<String>,
    components: HashMap<String, ComponentAddress>,
    current_component: Option<String>,
    resources: HashMap<String, ResourceAddress>,
}

impl TestEngine {
    /// Returns a new TestEngine.
    pub fn new() -> Self {
        let mut engine_interface = EngineInterface::new();

        let default_account = Account::new(&mut engine_interface);
        let mut accounts = HashMap::new();
        accounts.insert("default".format(), default_account);

        let mut resources = HashMap::new();
        resources.insert("Radix".format(), XRD);
        resources.insert("XRD".format(), XRD);

        let mut components = HashMap::new();
        components.insert("faucet".format(), FAUCET);

        Self {
            engine_interface,
            accounts,
            current_account: "default".format(),
            packages: HashMap::new(),
            current_package: None,
            components,
            current_component: None,
            resources,
        }
    }

    /// Returns a new TestEngine with an initial global package.
    ///
    /// # Arguments
    /// * `name`: name that will be used to reference the package.
    /// * `package`: compiled package data.
    pub fn with_package<E: EnvRef>(name: E, package: &(Vec<u8>, PackageDefinition)) -> Self {
        let mut test_engine = Self::new();
        test_engine.add_global_package(name, package);

        test_engine
    }

    /// Creates a new package from given path with a reference name.
    ///
    /// # Arguments
    /// * `name`: name that will be used to reference the package.
    /// * `path`: path of the package.
    pub fn new_package<E: EnvRef, P: AsRef<Path>>(&mut self, name: E, path: P) {
        match self.packages.get(&name.format()) {
            Some(_) => {
                panic!("A package with name {} already exists", name.format());
            }
            None => {
                let receipt = self.engine_interface.publish_package(path);
                self.create_package(name, receipt);
            }
        }
    }

    /// Adds a global package to the TestEngine.
    ///
    /// # Arguments
    /// * `name`: name that will be used to reference the package.
    /// * `package`: compiled package data.
    pub fn add_global_package<E: EnvRef>(
        &mut self,
        name: E,
        package: &(Vec<u8>, PackageDefinition),
    ) {
        match self.packages.get(&name.format()) {
            Some(_) => {
                panic!("A package with name {} already exists", name.format());
            }
            None => {
                let receipt = self
                    .engine_interface
                    .publish_compiled_package(package.0.clone(), package.1.clone());
                self.create_package(name, receipt);
            }
        }
    }

    /// Creates a new account with a reference name.
    ///
    /// # Arguments
    /// * `name`: name that will be used to reference the account.
    pub fn new_account<E: EnvRef>(&mut self, name: E) {
        match self.accounts.get(&name.format()) {
            Some(_) => panic!("An account with name {} already exists", name.format()),
            None => self
                .accounts
                .insert(name.format(), Account::new(&mut self.engine_interface)),
        };
    }

    /// Instantiates a new component of the current package with a reference name.
    ///
    /// # Arguments
    /// * `component_name`: name that will be used to reference the component.
    /// * `blueprint_name`: name of the blueprint.
    /// * `instantiation_function`: name of the function that instantiates the component.
    /// * `args`: environment arguments to instantiate the component.
    pub fn new_component<E: EnvRef>(
        &mut self,
        component_name: E,
        blueprint_name: &str,
        instantiation_function: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        self.create_component(
            component_name,
            blueprint_name,
            instantiation_function,
            args,
            None::<E>,
        )
    }

    pub fn new_component_with_badge<E: EnvRef, R: ResourceRef>(
        &mut self,
        component_name: E,
        blueprint_name: &str,
        instantiation_function: &str,
        badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        self.create_component(
            component_name,
            blueprint_name,
            instantiation_function,
            args,
            Some(badge),
        )
    }

    /// Calls a method of the current component.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    pub fn call_method(
        &mut self,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        let caller = self.current_account().clone();
        let component = *self.current_component();
        CallBuilder::call_method(self, caller, component, method_name, args).execute()
    }

    /// Creates a call builder for a method call.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    pub fn custom_method_call(
        &mut self,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> CallBuilder {
        let caller = self.current_account().clone();
        let component = *self.current_component();
        CallBuilder::call_method(self, caller, component, method_name, args)
    }

    /// Calls a method of the current component with a given admin badge.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `admin_badge`: reference name or address of the resource to use as an admin badge.
    /// * `args`: environment arguments to call the method.
    pub fn call_method_with_badge<R: ResourceRef>(
        &mut self,
        method_name: &str,
        admin_badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt {
        self.custom_method_call(method_name, args)
            .with_badge(admin_badge)
            .execute()
    }

    /// Calls faucet with the current account.
    pub fn call_faucet(&mut self) {
        let caller = self.current_account().clone();
        CallBuilder::call_method(self, caller, FAUCET, "free", vec![])
            .lock_fee("faucet", dec!(10))
            .execute();
    }

    /// Creates a new token with an initial_distribution and a reference name.
    ///
    /// # Arguments
    /// * `token_name`: name that will be used to reference the token.
    /// * `initial_distribution`: initial distribution of the token.
    pub fn new_token<E: EnvRef, D: TryInto<Decimal>>(
        &mut self,
        token_name: E,
        initial_distribution: D,
    ) where
        <D as TryInto<Decimal>>::Error: std::fmt::Debug,
    {
        match self.resources.get(&token_name.format()) {
            Some(_) => {
                panic!("Token with name {} already exists", token_name.format());
            }
            None => {
                let account = *self.current_account().address();
                let token_address = self
                    .engine_interface
                    .new_fungible(account, initial_distribution.try_into().unwrap());
                self.resources.insert(token_name.format(), token_address);
            }
        }
    }

    /// Creates a new token with a given resource address.
    ///
    /// # Arguments
    /// * `token_name`: name that will be used to reference the token.
    /// * `initial_distribution`: initial distribution of the token.
    /// * `resource_address`: address of the resource.
    /// * `network`: network on which the resource has the given address.
    pub fn add_token<E: EnvRef, D: TryInto<Decimal>>(
        &mut self,
        token_name: E,
        initial_supply: D,
        resource_address: &str,
        network: NetworkDefinition,
    ) where
        <D as TryInto<Decimal>>::Error: std::fmt::Debug,
    {
        match self.resources.get(&token_name.format()) {
            Some(_) => {
                panic!("Token with name {} already exists", token_name.format());
            }
            None => {
                let account = self.current_account().clone();
                let token_address = self.engine_interface.create_pre_allocated_token(
                    resource_address,
                    initial_supply.try_into().unwrap(),
                    network,
                    &account,
                );
                self.resources.insert(token_name.format(), token_address);
            }
        }
    }

    /// Returns the balance of the current account in the given resource.
    ///
    /// # Arguments
    /// * `resource`: reference name or address of the resource.
    pub fn current_balance<R: ResourceRef>(&mut self, resource: R) -> Decimal {
        let account = *self.current_account_address();
        let resource = resource.address(self);
        self.engine_interface.balance(account, resource)
    }

    /// Returns the balance of the given entity in the given resource.
    ///
    /// # Arguments
    /// * `entity`: reference name or address of the entity.
    /// * `resource`: reference name or address of the resource.
    pub fn balance_of<E: EntityRef, R: ResourceRef>(&mut self, entity: E, resource: R) -> Decimal {
        let entity = entity.address(self);
        let resource = resource.address(self);
        self.engine_interface.balance(entity, resource)
    }

    /// Returns the IDs of the given non-fungible resource owned by the current account.
    ///
    /// # Arguments
    /// * `resource`: reference name or address of the non-fungible resource.
    pub fn current_ids_balance<R: ResourceRef>(&mut self, resource: R) -> Vec<NonFungibleLocalId> {
        let account = *self.current_account_address();
        let resource = resource.address(self);
        self.engine_interface.nft_ids(account, resource)
    }

    /// Returns the IDs of the given non-fungible resource owned by the given account.
    ///
    /// # Arguments
    /// * `account`: reference name of the account.
    /// * `resource`: reference name or address of the resource.
    pub fn ids_balance_of<E: EntityRef, R: ResourceRef>(
        &mut self,
        entity: E,
        resource: R,
    ) -> Vec<NonFungibleLocalId> {
        let entity = entity.address(self);
        let resource = resource.address(self);
        self.engine_interface.nft_ids(entity, resource)
    }

    /// Moves to next epoch.
    pub fn next_epoch(&mut self) {
        let epoch = self.engine_interface.get_epoch();
        self.engine_interface.set_epoch(epoch.next().unwrap());
    }

    /// Advances epochs by the given amount.
    ///
    /// # Arguments
    /// * `epochs`: amount of epochs to jump to.
    pub fn jump_epochs(&mut self, epochs: u64) {
        let epoch = self.engine_interface.get_epoch();
        self.engine_interface
            .set_epoch(epoch.after(epochs).unwrap());
    }

    pub fn advance_time(&mut self, time: u64) {
        self.engine_interface.advance_time(time);
    }

    /// Jumps back epochs by the given amount.
    ///
    /// # Arguments
    /// * `epochs`: amount of epochs to jump back to.
    pub fn jump_back_epochs(&mut self, mut epochs: u64) {
        let epoch = self.engine_interface.get_epoch();
        while epochs != 0 {
            epoch.previous();
            epochs -= 1;
        }
        self.engine_interface.set_epoch(epoch)
    }

    /// Returns the [`PackageAddress`] of the given pacresourcekage.
    ///
    /// # Arguments
    /// * `name`: reference name of the package.
    pub fn get_package<E: EnvRef>(&self, name: E) -> PackageAddress {
        match self.packages.get(&name.format()) {
            None => panic!("There is no package with name {}", name.format()),
            Some(address) => *address,
        }
    }

    /// Returns the [`ComponentAddress`] of the given component.
    ///
    /// # Arguments
    /// * `name`: reference name of the component.
    pub fn get_component<E: EnvRef>(&self, name: E) -> ComponentAddress {
        match self.components.get(&name.format()) {
            None => panic!("There is no component with name {}", name.format()),
            Some(address) => *address,
        }
    }

    /// Returns the [`ComponentAddress`] of the given account.
    ///
    /// # Arguments
    /// * `name`: reference name of the account.
    pub fn get_account<E: EnvRef>(&self, name: E) -> &ComponentAddress {
        match self.accounts.get(&name.format()) {
            None => panic!("There is no account with name {}", name.format()),
            Some(account) => account.address(),
        }
    }

    /// Sets the current account.
    ///
    /// # Arguments
    /// * `name`: reference name of the account.
    pub fn set_current_account<E: EnvRef>(&mut self, name: E) {
        self.current_account = name.format();
        self.get_account(name);
    }

    /// Sets the current component
    ///
    /// # Arguments
    /// * `name`: reference name of the component.
    pub fn set_current_component<E: EnvRef>(&mut self, name: E) {
        self.current_component = Some(name.format());
        self.get_component(name);
    }

    /// Sets the current package.
    ///
    /// # Arguments
    /// * `name`: reference name of the account.
    pub fn set_current_package<E: EnvRef>(&mut self, name: E) {
        self.current_package = Some(name.format());
        self.get_package(name);
    }

    /// Returns the [`ResourceAddress`] of the given resource.
    ///
    /// # Arguments
    /// * `name`: reference name of the resource.
    pub fn get_resource<E: EnvRef>(&self, name: E) -> ResourceAddress {
        match self.resources.get(&name.format()) {
            None => panic!("There is no resource with name {}", name.format()),
            Some(resource) => *resource,
        }
    }

    /// Returns the [`PackageAddress`] of the current package.
    pub fn current_package(&self) -> &PackageAddress {
        self.packages
            .get(self.current_package.as_ref().unwrap())
            .unwrap()
    }

    /// Returns the [`ComponentAddress`] of the current account.
    pub fn current_account_address(&self) -> &ComponentAddress {
        self.accounts.get(&self.current_account).unwrap().address()
    }

    /// Returns the [`ComponentAddress`] of the current component.
    pub fn current_component(&self) -> &ComponentAddress {
        self.components
            .get(self.current_component.as_ref().unwrap())
            .unwrap()
    }

    /// Returns the state of the current component.
    pub fn current_component_state<T: ScryptoDecode>(&self) -> T {
        self.engine_interface.get_state(*self.current_component())
    }

    /// Returns the state of the given component.
    ///
    /// # Arguments
    /// * `component`: commponent reference or address for which to get the state.
    pub fn get_component_state<T: ScryptoDecode, E: EntityRef>(&self, component: E) -> T {
        self.engine_interface.get_state(component.address(self))
    }

    /// Returns the value of a KeyValueStore at a given key.
    ///
    /// # Arguments
    /// * `kv_store_id`: id of the KeyValueStore.
    /// * `key`: key of the value to get.
    pub fn get_kvs_value_at<K: ScryptoEncode, V: ScryptoEncode + ScryptoDecode>(
        &self,
        kv_store_id: Own,
        key: &K,
    ) -> Option<V> {
        self.engine_interface.get_kvs_entry(kv_store_id, key)
    }

    pub(crate) fn get_component_ref<E: EnvRef>(&self, name: E) -> ComponentAddress {
        let name = name.format();
        match self.accounts.get(&name) {
            None => match self.components.get(&name) {
                None => {
                    panic!(
                        "There is no environment reference with name {}",
                        name.format()
                    )
                }
                Some(component) => *component,
            },
            Some(account) => *account.address(),
        }
    }

    pub(crate) fn current_account(&self) -> &Account {
        self.accounts.get(&self.current_account).unwrap()
    }

    pub(crate) fn execute_call(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
        initial_proofs: Vec<NonFungibleGlobalId>,
        with_update: bool,
    ) -> TransactionReceipt {
        let receipt = self
            .engine_interface
            .execute_manifest(manifest, with_trace, initial_proofs);
        if with_update {
            if let TransactionResult::Commit(commit_result) = &receipt.result {
                self.update_data_from_result(commit_result);
            }
        }
        receipt
    }

    pub(crate) fn network(&self) -> NetworkDefinition {
        NetworkDefinition::simulator()
    }

    pub(crate) fn ids_owned_at_address(
        &mut self,
        resource: ResourceAddress,
    ) -> Vec<NonFungibleLocalId> {
        let account = *self.current_account().address();
        self.engine_interface.nft_ids(account, resource)
    }

    pub(crate) fn update_data_from_result(&mut self, result: &CommitResult) {
        for component in result.new_component_addresses() {
            if let Some(name) = self.get_metadata_value_of("name", (*component).into()) {
                self.insert_component(name, *component)
            }
        }

        self.update_resources_from_result(result);
    }

    pub(crate) fn get_entity<E: EnvRef>(&self, entity: E) -> ComponentAddress {
        match self.accounts.get(&entity.format()) {
            Some(account) => *(account.address()),
            None => match self.components.get(&entity.format()) {
                Some(component) => *component,
                None => {
                    panic!("There is no component with name {}!", entity.format())
                }
            },
        }
    }

    fn create_component<E: EnvRef, R: ResourceRef>(
        &mut self,
        component_name: E,
        blueprint_name: &str,
        instantiation_function: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
        opt_badge: Option<R>,
    ) -> TransactionReceipt {
        let caller = self.current_account().clone();
        let package = *self.current_package();
        let mut partial_call = CallBuilder::call_function(
            self,
            caller,
            package,
            blueprint_name,
            instantiation_function,
            args,
        );

        if let Some(badge) = opt_badge {
            partial_call = partial_call.with_badge(badge)
        }

        let receipt = partial_call.execute_no_update();

        let mut receipt = receipt.assert_is_success();

        if let TransactionResult::Commit(ref mut commit) = &mut receipt.result {
            let mut components: Vec<&ComponentAddress> =
                commit.new_component_addresses().iter().collect();
            if let Some(component) = components.first() {
                self.components.insert(component_name.format(), **component);
                components.remove(0);
            }

            if self.current_component.is_none() {
                self.current_component = Some(component_name.format())
            };

            self.update_resources_from_result(commit);
        } else if let TransactionResult::Reject(reject) = &receipt.result {
            panic!("{}", reject.reason);
        }

        receipt
    }

    fn create_package<E: EnvRef>(&mut self, name: E, receipt: TransactionReceipt) {
        match receipt.result {
            TransactionResult::Commit(commit) => {
                self.packages
                    .insert(name.format(), commit.new_package_addresses()[0]);
                if self.current_package.is_none() {
                    self.current_package = Some(name.format());
                }
            }
            TransactionResult::Reject(reject) => {
                panic!(
                    "Could not publish package {}. Transaction was rejected with error: {}",
                    name.format(),
                    reject.reason
                );
            }
            TransactionResult::Abort(abort) => {
                panic!(
                    "Could not publish package {}. Transaction was aborted with error: {}",
                    name.format(),
                    abort.reason
                );
            }
        }
    }

    fn update_resources_from_result(&mut self, result: &CommitResult) {
        // Update tracked resources
        for resource in result.new_resource_addresses() {
            if let Some(name) = self.get_metadata_value_of("name", (*resource).into()) {
                self.insert_resource(name, *resource);
            }
            if let Some(name) = self.get_metadata_value_of("symbol", (*resource).into()) {
                self.try_insert_resource(name, *resource);
            }
        }
    }

    fn get_metadata_value_of(&mut self, metadata: &str, address: GlobalAddress) -> Option<String> {
        if let Some(MetadataValue::String(value)) =
            self.engine_interface.get_metadata(address, metadata)
        {
            Some(value)
        } else {
            None
        }
    }

    fn insert_resource(&mut self, name: String, resource_address: ResourceAddress) {
        if let Entry::Vacant(e) = self.resources.entry(name.format()) {
            e.insert(resource_address);
        } else {
            panic!("Token with name {} already exists", name.format());
        }
    }

    fn try_insert_resource(&mut self, name: String, resource_address: ResourceAddress) {
        if let Entry::Vacant(e) = self.resources.entry(name.format()) {
            e.insert(resource_address);
        }
    }

    fn insert_component(&mut self, name: String, component_address: ComponentAddress) {
        if let Entry::Vacant(e) = self.components.entry(name.format()) {
            e.insert(component_address);
        } else {
            panic!("Component with name {} already exists", name.format());
        }
    }
}
impl Default for TestEngine {
    fn default() -> Self {
        Self::new()
    }
}
