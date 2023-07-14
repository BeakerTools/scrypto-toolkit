use std::path::Path;
use radix_engine::types::{ComponentAddress, PackageAddress};
use scrypto_unit::TestRunner;

pub struct EngineInterface {
    test_runner: TestRunner
}

impl EngineInterface {

    pub fn new() -> Self {
        Self {
            test_runner: TestRunner::builder().without_trace().build()
        }
    }

    pub fn publish_package<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        self.test_runner.compile_and_publish(package_dir)
    }

    pub fn new_account(&mut self) -> ComponentAddress {
        let (_,_, address) = self.test_runner.new_account(false);
        address
    }

}