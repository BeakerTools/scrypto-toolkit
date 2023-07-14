use std::path::Path;
use radix_engine::types::{ComponentAddress, HashMap, PackageAddress};
use crate::blueprint::Blueprint;
use crate::engine_interface::EngineInterface;
use crate::environment_encoder::EnvironmentEncode;
use crate::formatted_strings::ToFormatted;

pub struct TestEngine {
    engine_interface: EngineInterface,
    accounts: HashMap<String, ComponentAddress>,
    current_account: String,
    packages: HashMap<String, PackageAddress>,
    current_package: Option<String>,
    components: HashMap<String, ComponentAddress>,
    current_component: Option<String>

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
            current_component: None
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

    pub fn new_component<F: ToFormatted, B: Blueprint>(&mut self, component_name: F, blueprint: B, args: Vec<Box<dyn EnvironmentEncode>>) {
        match self.components.get(&component_name.format())
        {
            Some(_) => panic!("A component with name {} already exists", component_name.format()),
            None => {
                let package_address = self.current_package().clone();
                let current_account = self.current_account().clone();

            }
        }
    }

    pub fn get_package<F: ToFormatted>(&self, name: F) -> PackageAddress {
        match self.packages.get(&name.format()){
            None => panic!("There is no package with name {}", name.format()),
            Some(address) => address.clone()
        }
    }

    pub fn get_account<F: ToFormatted>(&self, name: F) -> ComponentAddress {
        match self.accounts.get(&name.format()){
            None => panic!("There is no account with name {}", name.format()),
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
}