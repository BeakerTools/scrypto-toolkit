use std::path::Path;
use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use radix_engine::types::{ComponentAddress, Decimal, GlobalAddress, HashMap, PackageAddress, ResourceAddress, XRD};
use radix_engine_interface::prelude::{MetadataValue, NonFungibleGlobalId};
use transaction::model::TransactionManifestV1;
use crate::account::Account;
use crate::calls::CallBuilder;
use crate::engine_interface::EngineInterface;
use crate::environment::EnvironmentEncode;
use crate::formatted_strings::ToFormatted;
use crate::receipt_traits::Outcome;

pub struct TestEngine {
    engine_interface: EngineInterface,
    accounts: HashMap<String, Account>,
    current_account: String,
    packages: HashMap<String, PackageAddress>,
    current_package: Option<String>,
    components: HashMap<String, ComponentAddress>,
    current_component: Option<String>,
    resources: HashMap<String, ResourceAddress>
}

impl TestEngine{

    pub fn new() -> Self {
        let mut engine_interface = EngineInterface::new();

        let default_account = Account::new(&mut engine_interface);
        let mut accounts = HashMap::new();
        accounts.insert("default".format(), default_account);

        let mut resources = HashMap::new();
        resources.insert("Radix".format(), XRD);
        resources.insert("XRD".format(), XRD);

        Self {
            engine_interface,
            accounts,
            current_account: "default".format(),
            packages: HashMap::new(),
            current_package: None,
            components: HashMap::new(),
            current_component: None,
            resources
        }
    }


    pub fn new_package<F: ToFormatted, P: AsRef<Path>>(&mut self, name: F, path: P) {
        match self.packages.get(&name.format()){
            Some(_) => {
                panic!("A package with name {} already exists", name.format());
            }
            None => {
                self.packages.insert(name.format(), self.engine_interface.publish_package(path));
                if self.current_package.is_none(){
                    self.current_package = Some(name.format());
                }
            }
        }
    }

    pub fn new_account<F: ToFormatted>(&mut self, name: F) {
        match self.accounts.get(&name.format()) {
            Some(_) => panic!("An account with name {} already exists", name.format()),
            None => self.accounts.insert(name.format(), Account::new(&mut self.engine_interface))
        };
    }

    pub fn new_component<F: ToFormatted>(&mut self, component_name: F, blueprint_name: &str, instantiation_function: &str, args: Vec<Box<dyn EnvironmentEncode>>) -> TransactionReceipt{
        match self.components.get(&component_name.format())
        {
            Some(_) => panic!("A component with name {} already exists", component_name.format()),
            None => {
                let caller = self.current_account().clone();
                let package = self.current_package().clone();
                let receipt = CallBuilder::from(self, caller)
                    .call_function( package, blueprint_name, instantiation_function, args)
                    .run();
                receipt.assert_is_success();

                if let TransactionResult::Commit(commit) = &receipt.transaction_result {
                    let component: ComponentAddress = commit.new_component_addresses().get(0).unwrap().clone();
                    self.components.insert(component_name.format(), component);

                    if self.current_component.is_none() {
                        self.current_component = Some(component_name.format())
                    };

                    self.update_resources_from_result(&commit);
                }
                else if let TransactionResult::Reject(reject) = &receipt.transaction_result {
                    panic!("{}", reject.error);
                }

                receipt
            }
        }
    }

    pub fn call_method(&mut self, method_name: &str, args: Vec<Box<dyn EnvironmentEncode>>) -> TransactionReceipt {
        let caller = self.current_account().clone();
        let component = self.current_component().clone();
        let receipt = CallBuilder::from(self, caller)
            .call_method(component, method_name, args)
            .run();
        if let TransactionResult::Commit(commit) = &receipt.transaction_result {
            self.update_resources_from_result(commit);
        }
        receipt
    }

    pub fn new_token<F: ToFormatted, G: TryInto<Decimal>>(&mut self, token_name: F, initial_distribution: G)
        where <G as TryInto<Decimal>>::Error: std::fmt::Debug
    {
        match self.resources.get(&token_name.format()){
            Some(_) => {
                panic!("Token with name {} already exists", token_name.format());
            }
            None => {
                let account = self.current_account().address().clone();
                let token_address = self.engine_interface.new_fungible(account, initial_distribution.try_into().unwrap());
                self.resources.insert(token_name.format(), token_address);
            }
        }
    }

    pub fn current_balance<F: ToFormatted>(&mut self, resource: F) -> Decimal {
        let account = self.current_account_address().clone();
        let resource = self.get_resource(resource);
        self.engine_interface.balance(account, resource)
    }

    pub fn balance_of<F: ToFormatted, G: ToFormatted>(&mut self, account: F, resource: F) -> Decimal {
        let account = self.get_account(account);
        let resource = self.get_resource(resource);
        self.engine_interface.balance(account.clone(), resource)
    }

    pub(crate) fn execute_call(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
        initial_proofs: Vec<NonFungibleGlobalId>
    ) -> TransactionReceipt {

        let receipt = self.engine_interface.execute_manifest(manifest, with_trace, initial_proofs);
        if let TransactionResult::Commit(commit_result) = &receipt.transaction_result {
            self.update_resources_from_result(commit_result);
        }
        receipt
    }

    pub fn get_package<F: ToFormatted>(&self, name: F) -> PackageAddress {
        match self.packages.get(&name.format()){
            None => panic!("There is no package with name {}", name.format()),
            Some(address) => address.clone()
        }
    }

    pub fn get_component<F: ToFormatted>(&self, name: F) -> ComponentAddress {
        match self.components.get(&name.format()){
            None => panic!("There is no component with name {}", name.format()),
            Some(address) => address.clone()
        }
    }

    pub fn get_account<F: ToFormatted>(&self, name: F) -> &ComponentAddress {
        match self.accounts.get(&name.format()){
            None => panic!("There is no account with name {}", name.format()),
            Some(account) => account.address()
        }
    }

    pub fn get_resource<F: ToFormatted>(&self, name: F) -> ResourceAddress {
        match self.resources.get(&name.format()) {
            None => panic!("There is no resource with name {}", name.format()),
            Some(address) => address.clone()
        }
    }

    pub fn current_package(&self) -> &PackageAddress {
        self.packages
            .get(self.current_package.as_ref().unwrap())
            .unwrap()
    }

    pub fn current_account_address(&self) -> &ComponentAddress {
        &self.accounts.get(&self.current_account).unwrap().address()
    }

    pub fn current_account(&self) -> &Account {
        &self.accounts.get(&self.current_account).unwrap()
    }

    pub fn current_component(&self) -> &ComponentAddress {
        self.components
            .get(self.current_component.as_ref().unwrap())
            .unwrap()
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

            match self.engine_interface.get_metadata(GlobalAddress::from(resource.clone()), "symbol")
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