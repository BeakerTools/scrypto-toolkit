use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::{
    manifest_decode, ComponentAddress, Encoder, ManifestArgs, ManifestEncoder, ManifestValueKind,
    PackageAddress, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX,
};
use transaction::builder::ManifestBuilder;

use crate::account::Account;
use crate::environment::EnvironmentEncode;
use crate::test_engine::TestEngine;
use std::vec::Vec;
use transaction::prelude::TransactionManifestV1;

pub struct CallBuilder<'a> {
    caller: Account,
    manifest: Option<TransactionManifestV1>,
    test_engine: &'a mut TestEngine,
    with_trace: bool,
}

impl<'a> CallBuilder<'a> {
    pub fn from(test_env: &'a mut TestEngine, caller: Account) -> Self {
        Self {
            caller,
            manifest: None,
            test_engine: test_env,
            with_trace: false,
        }
    }

    pub fn run(self) -> TransactionReceipt {
        self.test_engine.execute_call(
            self.manifest.clone().unwrap(),
            self.with_trace.clone(),
            vec![self.caller.proof()],
        )
    }

    pub(crate) fn call_method(
        mut self,
        component: ComponentAddress,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest = ManifestBuilder::new().lock_fee_from_faucet();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest = arg.encode(
                &mut self.test_engine,
                manifest,
                &mut encoder,
                self.caller.address().clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let transaction = manifest
            .call_method(component, method_name, manifest_arg)
            .deposit_batch(self.caller.address().clone())
            .build();
        self.manifest = Some(transaction);
        self
    }

    pub(crate) fn call_function(
        mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {
        let mut manifest = ManifestBuilder::new().lock_fee_from_faucet();

        let mut buf = Vec::new();
        let mut encoder = ManifestEncoder::new(&mut buf, MANIFEST_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ManifestValueKind::Tuple).unwrap();
        encoder.write_size(args.len()).unwrap();
        for arg in args {
            manifest = arg.encode(
                &mut self.test_engine,
                manifest,
                &mut encoder,
                self.caller.address().clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let transaction = manifest
            .call_function(package_address, blueprint_name, function_name, manifest_arg)
            .deposit_batch(self.caller.address().clone())
            .build();
        self.manifest = Some(transaction);
        self
    }
}
