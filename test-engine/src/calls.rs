use radix_engine::transaction::{TransactionReceipt};
use radix_engine::types::{dec, manifest_decode, ComponentAddress, Decimal, Encoder, ManifestEncoder, ManifestValueKind, PackageAddress, ManifestArgs, MANIFEST_SBOR_V1_MAX_DEPTH, MANIFEST_SBOR_V1_PAYLOAD_PREFIX, ManifestExpression, ManifestValue};
use radix_engine_interface::constants::FAUCET_COMPONENT;
use radix_engine_interface::count;
use transaction::builder::ManifestBuilder;

use std::vec::Vec;
use transaction::prelude::{DynamicGlobalAddress, TransactionManifestV1};
use crate::environment::EnvironmentEncode;
use crate::manifest_args;
use crate::test_engine::TestEngine;

#[derive(Clone)]
pub enum Outcome {
    /// States that transaction should
    Success,

    /// States that an assertion is expected to fail with a given message
    AssertionFailed(String),

    /// States that another error should happen
    OtherError(String),
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        match self {
            Outcome::Success => true,
            _ => false,
        }
    }
}

pub struct CallBuilder<'a> {
    caller: ComponentAddress,
    manifest: Option<TransactionManifestV1>,
    fee_locked: Decimal,
    fee_payer: ComponentAddress,
    test_engine: &'a mut TestEngine,
    with_trace: bool,
}

impl<'a> CallBuilder<'a> {
    pub fn from(test_env: &'a mut TestEngine, caller: ComponentAddress) -> Self {
        Self {
            caller: caller.clone() ,
            manifest: None,
            fee_locked: dec!(10),
            fee_payer: caller,
            test_engine: test_env,
            with_trace: false,
        }
    }

    pub fn lock(mut self, amount: Decimal) -> Self {
        self.fee_locked = amount;
        self
    }

    pub fn faucet_pays_fees(mut self) -> Self {
        self.fee_payer = FAUCET_COMPONENT;
        self
    }

    pub fn run(mut self) -> TransactionReceipt {
        self.build();
        self.test_engine
            .execute_call(self.manifest.clone().unwrap(), self.with_trace.clone())
    }

    pub fn with_trace(mut self, trace: bool) -> Self {
        self.with_trace = trace;
        self
    }

    fn build(&mut self) {
        self.lock_fee();
        self.deposit_batch();
    }

    fn lock_fee(&mut self) {
        self.manifest.as_mut().unwrap().instructions.insert(
            0,
            transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.fee_payer),
                method_name: "lock_fee".to_string(),
                args: ManifestValue::from(manifest_args!(self.fee_locked)),
            },
        );
    }

    fn deposit_batch(&mut self) {
        self.manifest.as_mut().unwrap().instructions.push(
            transaction::model::InstructionV1::CallMethod {
                address: DynamicGlobalAddress::from(self.caller.clone()),
                method_name: "deposit_batch".to_string(),
                args: ManifestValue::from(manifest_args!(ManifestExpression::EntireWorktop))
            }
        );
    }

    pub(crate) fn call_method(
        mut self,
        component: ComponentAddress,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> Self {

        let mut manifest = ManifestBuilder::new();

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
                self.caller.clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let transaction = manifest
            .call_method(component, method_name, manifest_arg)
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
        let mut manifest = ManifestBuilder::new();

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
                self.caller.clone(),
            );
        }

        let value = manifest_decode(&buf).unwrap();
        let manifest_arg = ManifestArgs::new_from_tuple_or_panic(value);

        let transaction = manifest
            .call_function(package_address, blueprint_name, function_name, manifest_arg)
            .build();
        self.manifest = Some(transaction);
        self
    }
}