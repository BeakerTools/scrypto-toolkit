use std::path::Path;
use radix_engine::transaction::{CommitResult, TransactionReceipt, TransactionResult};
use radix_engine::types::{ComponentAddress, GlobalAddress, HashMap, PackageAddress, ResourceAddress};
use radix_engine_interface::prelude::MetadataValue;
use transaction::model::TransactionManifestV1;
use crate::calls::CallBuilder;
use crate::engine_interface::EngineInterface;
use crate::environment::EnvironmentEncode;
use crate::formatted_strings::ToFormatted;

pub struct TestEngine {
    engine_interface: EngineInterface,
    accounts: HashMap<String, ComponentAddress>,
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
        let default_account = engine_interface.new_account();
        let mut accounts = HashMap::new();
        accounts.insert("default".format(), default_account);

        Self {
            engine_interface,
            accounts,
            current_account: "default".format(),
            packages: HashMap::new(),
            current_package: None,
            components: HashMap::new(),
            current_component: None,
            resources: HashMap::new()
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
            None => self.accounts.insert(name.format(), self.engine_interface.new_account())
        };
    }

    pub fn new_component<F: ToFormatted>(&mut self, component_name: F, blueprint_name: &str, instantiation_function: &str, args: Vec<Box<dyn EnvironmentEncode>>) -> CallBuilder{
        match self.components.get(&component_name.format())
        {
            Some(_) => panic!("A component with name {} already exists", component_name.format()),
            None => {
                let caller = self.current_account().clone();
                let package = self.current_package().clone();
                CallBuilder::from(self, caller)
                    .call_function( package, blueprint_name, instantiation_function, args)
            }
        }
    }

    pub(crate) fn execute_call(
        &mut self,
        manifest: TransactionManifestV1,
        with_trace: bool,
    ) -> TransactionReceipt {

        let receipt = self.engine_interface.execute_manifest(manifest, with_trace);
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

    pub fn get_account<F: ToFormatted>(&self, name: F) -> ComponentAddress {
        match self.accounts.get(&name.format()){
            None => panic!("There is no account with name {}", name.format()),
            Some(address) => address.clone()
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

    pub fn current_account(&self) -> &ComponentAddress {
        self.accounts.get(&self.current_account).unwrap()
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