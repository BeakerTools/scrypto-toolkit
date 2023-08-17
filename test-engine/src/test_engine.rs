use std::path::Path;

use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use radix_engine::types::{
    dec, ComponentAddress, Decimal, GlobalAddress, HashMap, NonFungibleLocalId, PackageAddress,
    ResourceAddress, FAUCET, XRD,
};
use radix_engine_interface::prelude::{MetadataValue, NonFungibleGlobalId};
use transaction::model::TransactionManifestV1;

use crate::account::Account;
use crate::calls::CallBuilder;
use crate::engine_interface::EngineInterface;
use crate::environment::EnvironmentEncode;
use crate::environment_reference::EnvRef;
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
                self.packages
                    .insert(name.format(), self.engine_interface.publish_package(path));
                if self.current_package.is_none() {
                    self.current_package = Some(name.format());
                }
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
        match self.components.get(&component_name.format()) {
            Some(_) => panic!(
                "A component with name {} already exists",
                component_name.format()
            ),
            None => {
                let caller = self.current_account().clone();
                let package = self.current_package().clone();
                let receipt = CallBuilder::call_function(
                    self,
                    caller,
                    package,
                    blueprint_name,
                    instantiation_function,
                    args,
                )
                .execute();
                let receipt = receipt.assert_is_success();

                if let TransactionResult::Commit(commit) = &receipt.transaction_result {
                    match commit.new_component_addresses().get(0) {
                        None => {}
                        Some(component) => {
                            self.components
                                .insert(component_name.format(), component.clone());
                        }
                    }

                    if self.current_component.is_none() {
                        self.current_component = Some(component_name.format())
                    };

                    self.update_resources_from_result(&commit);
                } else if let TransactionResult::Reject(reject) = &receipt.transaction_result {
                    panic!("{}", reject.error);
                }

                receipt
            }
        }
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
        let component = self.current_component().clone();
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
        let component = self.current_component().clone();
        CallBuilder::call_method(self, caller, component, method_name, args)
    }

    /// Calls a method of the current component with a given admin badge.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `admin_badge`: reference name of the resource to use as an admin badge.
    /// * `args`: environment arguments to call the method.
    pub fn call_method_with_badge<E: EnvRef>(
        &mut self,
        method_name: &str,
        admin_badge: E,
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
                let account = self.current_account().address().clone();
                let token_address = self
                    .engine_interface
                    .new_fungible(account, initial_distribution.try_into().unwrap());
                self.resources.insert(token_name.format(), token_address);
            }
        }
    }

    /// Returns the balance of the current account in the given resource.
    ///
    /// # Arguments
    /// * `resource`: reference name of the resource.
    pub fn current_balance<E: EnvRef>(&mut self, resource: E) -> Decimal {
        let account = self.current_account_address().clone();
        let resource = self.get_resource(resource);
        self.engine_interface.balance(account, resource)
    }

    /// Returns the balance of the given account in the given resource.
    ///
    /// # Arguments
    /// * `account`: reference name of the account.
    /// * `resource`: reference name of the resource.
    pub fn balance_of<E: EnvRef, G: EnvRef>(&mut self, account: E, resource: E) -> Decimal {
        let account = self.get_account(account);
        let resource = self.get_resource(resource);
        self.engine_interface.balance(account.clone(), resource)
    }

    /// Returns the IDs of the given non-fungible resource owned by the current account.
    ///
    /// # Arguments
    /// * `resource`: reference name of the non-fungible resource.
    pub fn current_ids_balance<E: EnvRef>(&mut self, resource: E) -> Vec<NonFungibleLocalId> {
        let account = self.current_account_address().clone();
        let resource = self.get_resource(resource);
        self.engine_interface.nft_ids(account, resource)
    }

    /// Returns the IDs of the given non-fungible resource owned by the given account.
    ///
    /// # Arguments
    /// * `account`: reference name of the account.
    /// * `resource`: reference name of the resource.
    pub fn ids_balance_of<E: EnvRef, G: EnvRef>(
        &mut self,
        account: E,
        resource: E,
    ) -> Vec<NonFungibleLocalId> {
        let account = self.get_account(account);
        let resource = self.get_resource(resource);
        self.engine_interface.nft_ids(account.clone(), resource)
    }

    /// Moves to next epoch.
    pub fn next_epoch(&mut self) {
        let epoch = self.engine_interface.get_epoch();
        self.engine_interface.set_epoch(epoch.next());
    }

    /// Advances epochs by the given amount.
    ///
    /// # Arguments
    /// * `epochs`: amount of epochs to jump to.
    pub fn jump_epochs(&mut self, epochs: u64) {
        let epoch = self.engine_interface.get_epoch();
        self.engine_interface.set_epoch(epoch.after(epochs));
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
            Some(address) => address.clone(),
        }
    }

    /// Returns the [`ComponentAddress`] of the given component.
    ///
    /// # Arguments
    /// * `name`: reference name of the component.
    pub fn get_component<E: EnvRef>(&self, name: E) -> ComponentAddress {
        match self.components.get(&name.format()) {
            None => panic!("There is no component with name {}", name.format()),
            Some(address) => address.clone(),
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
            Some(resource) => resource.clone(),
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
        &self.accounts.get(&self.current_account).unwrap().address()
    }

    /// Returns the [`ComponentAddress`] of the current component.
    pub fn current_component(&self) -> &ComponentAddress {
        self.components
            .get(self.current_component.as_ref().unwrap())
            .unwrap()
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
                Some(component) => component.clone(),
            },
            Some(account) => account.address().clone(),
        }
    }

    pub(crate) fn current_account(&self) -> &Account {
        &self.accounts.get(&self.current_account).unwrap()
    }

    pub(crate) fn execute_call(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> TransactionReceipt {
        let receipt = self
            .engine_interface
            .execute_manifest(manifest, with_trace, initial_proofs);
        if let TransactionResult::Commit(commit_result) = &receipt.transaction_result {
            self.update_resources_from_result(commit_result);
        }
        receipt
    }

    pub(crate) fn ids_owned_at_address(
        &mut self,
        resource: ResourceAddress,
    ) -> Vec<NonFungibleLocalId> {
        let account = self.current_account().address().clone();
        let ids = self.engine_interface.nft_ids(account, resource);
        ids
    }

    fn update_resources_from_result(&mut self, result: &CommitResult) {
        // Update tracked resources
        for resource in result.new_resource_addresses() {
            match self
                .engine_interface
                .get_metadata(GlobalAddress::from(resource.clone()), "name")
            {
                None => {}
                Some(entry) => match entry {
                    MetadataValue::String(name) => {
                        self.resources.insert(name.format(), resource.clone());
                    }
                    _ => {}
                },
            }

            match self
                .engine_interface
                .get_metadata(GlobalAddress::from(resource.clone()), "symbol")
            {
                None => {}
                Some(entry) => match entry {
                    MetadataValue::String(name) => {
                        self.resources.insert(name.format(), resource.clone());
                    }
                    _ => {}
                },
            }
        }
    }
}
