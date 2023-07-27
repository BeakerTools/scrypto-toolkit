use radix_engine::types::{
    ComponentAddress, Decimal, Hash, NonFungibleGlobalId, NonFungibleLocalId, PackageAddress,
    PreciseDecimal, ResourceAddress,
};
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_interface::data::scrypto::scrypto_decode;
use radix_engine_interface::data::scrypto::ScryptoDecode;

pub trait FromInstruction {
    fn from(instructions: Vec<InstructionOutput>) -> Self;
}

macro_rules! from_return_impl {
    ($type:ident) => {
        impl FromInstruction for $type {
            fn from(mut instructions: Vec<InstructionOutput>) -> Self {
                if instructions.len() != 3 {
                    panic!("Could not parse method return into given type 1")
                }

                instructions.pop();
                let bytes = match instructions.pop().unwrap() {
                    InstructionOutput::None => {
                        panic!("The method does not return anything")
                    }
                    InstructionOutput::CallReturn(bytes) => bytes,
                };
                scrypto_decode::<$type>(&bytes)
                    .expect("Could not parse method return into given type 2")
            }
        }
    };
}

from_return_impl!(u8);
from_return_impl!(u16);
from_return_impl!(u32);
from_return_impl!(u64);
from_return_impl!(u128);
from_return_impl!(i8);
from_return_impl!(i16);
from_return_impl!(i32);
from_return_impl!(i64);
from_return_impl!(i128);
from_return_impl!(String);
from_return_impl!(ComponentAddress);
from_return_impl!(PackageAddress);
from_return_impl!(ResourceAddress);
from_return_impl!(NonFungibleGlobalId);
from_return_impl!(NonFungibleLocalId);
from_return_impl!(Hash);
from_return_impl!(Decimal);
from_return_impl!(PreciseDecimal);


macro_rules! from_return_tuple_impl {
    ( $( $idx:tt $type:ident )+ ) => {
        impl<$($type: FromInstruction + ScryptoDecode),+> FromInstruction for ($($type, )+){
            fn from(instructions: Vec<InstructionOutput>) -> Self{
                (
                    $(
                        match instructions.get($idx+1).clone().unwrap()
                        {
                            InstructionOutput::None => { panic!("The method does not return anything") }
                            InstructionOutput::CallReturn(bytes) => {
                                 scrypto_decode::<$type>(&bytes).expect("Could not parse method return into given type")
                            }
                        }

                    ,)*
                )
            }
        }
    }
}

from_return_tuple_impl! { 0 T0 }
from_return_tuple_impl! { 0 T0 1 T1 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 }
from_return_tuple_impl! { 0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19 }