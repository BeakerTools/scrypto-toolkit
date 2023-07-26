use std::collections::BTreeSet;
use std::path::Path;
use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig, TransactionReceipt};
use radix_engine::types::{ComponentAddress, GlobalAddress, PackageAddress};
use radix_engine_interface::prelude::{MetadataValue, NonFungibleGlobalId};
use scrypto_unit::TestRunner;
use transaction::model::TransactionManifestV1;
use transaction::prelude::{manifest_args, TestTransaction};

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

    pub fn execute_manifest(&mut self, manifest: TransactionManifestV1, with_trace: bool) -> TransactionReceipt {
        let nonce = self.test_runner.next_transaction_nonce();
        let exec_config = ExecutionConfig::for_test_transaction().with_kernel_trace(with_trace);
        let initial_proofs = BTreeSet::new();

        self.test_runner.execute_transaction(
            TestTransaction::new_from_nonce(manifest, nonce)
                .prepare()
                .expect("expected transaction to be preparable")
                .get_executable(initial_proofs),
            FeeReserveConfig::default(),
            exec_config,
        )
    }

    pub fn get_metadata(&mut self, address: GlobalAddress, key: &str) -> Option<MetadataValue> {
        self.test_runner.get_metadata(address, key)
    }
}