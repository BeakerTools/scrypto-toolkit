use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::scrypto_decode;
use radix_engine_interface::data::scrypto::ScryptoDecode;

pub trait FromInstruction {
    fn from(instructions: Vec<InstructionOutput>) -> Self;
}

impl<T: ScryptoDecode> FromInstruction for T {
    fn from(mut instructions: Vec<InstructionOutput>) -> Self {
        instructions.pop();
        let bytes = match instructions.pop().unwrap() {
            InstructionOutput::None => {
                panic!("The method does not return anything")
            }
            InstructionOutput::CallReturn(bytes) => bytes,
        };
        scrypto_decode::<T>(&bytes).expect("Could not parse method return into given type 2")
    }
}
