use std::path::Path;

use radix_engine::transaction::{ExecutionConfig, FeeReserveConfig, TransactionReceipt};
use radix_engine::types::{
    ComponentAddress, Decimal, GlobalAddress, PackageAddress, ResourceAddress, Secp256k1PublicKey,
};
use radix_engine_interface::prelude::{MetadataValue, NonFungibleGlobalId};
use scrypto_unit::TestRunner;
use transaction::model::TransactionManifestV1;
use transaction::prelude::{Secp256k1PrivateKey, TestTransaction};

pub struct EngineInterface {
    test_runner: TestRunner,
}

impl EngineInterface {
    pub fn new() -> Self {
        Self {
            test_runner: TestRunner::builder().without_trace().build(),
        }
    }

    pub fn publish_package<P: AsRef<Path>>(&mut self, package_dir: P) -> PackageAddress {
        self.test_runner.compile_and_publish(package_dir)
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
            FeeReserveConfig::default(),
            exec_config,
        )
    }

    pub fn get_metadata(&mut self, address: GlobalAddress, key: &str) -> Option<MetadataValue> {
        self.test_runner.get_metadata(address, key)
    }

    pub fn balance(&mut self, account: ComponentAddress, resource: ResourceAddress) -> Decimal {
        match self.test_runner.account_balance(account, resource) {
            None => Decimal::zero(),
            Some(amount) => amount,
        }
    }

    pub fn new_fungible(
        &mut self,
        account: ComponentAddress,
        initial_amount: Decimal,
    ) -> ResourceAddress {
        self.test_runner
            .create_fungible_resource(initial_amount, 18, account)
    }
}
