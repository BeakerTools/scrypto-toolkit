use crate::from_instruction::FromInstruction;
use radix_engine::transaction::{TransactionOutcome, TransactionReceipt, TransactionResult};

pub trait Outcome {
    fn assert_is_success(&self);
}

impl Outcome for TransactionReceipt {

    /// Asserts that the transaction succeeded.
    /// Panics if the transaction was rejected, aborted or failed.
    fn assert_is_success(&self) {
        match &self.transaction_result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(_) => {},
                TransactionOutcome::Failure(failure) => {
                    panic!("Transaction failed with: {}", failure);
                }
            },
            TransactionResult::Reject(reject) => {
                panic!("Transaction rejected with: {}", reject.error);
            }
            TransactionResult::Abort(abort) => {
                panic!("Transaction abort with: {}", abort.reason);
            }
        }
    }
}

pub trait GetReturn<T> {
    fn get_return(&self) -> T;
}

impl<T> GetReturn<T> for TransactionReceipt
where
    T: FromInstruction,
{
    /// Returns the method's return from a receipt.
    fn get_return(&self) -> T {
        match &self.transaction_result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(output) => T::from(output.clone()),
                TransactionOutcome::Failure(failure) => {
                    panic!("Transaction failed with: {}", failure);
                }
            },
            TransactionResult::Reject(reject) => {
                panic!("Transaction rejected with: {}", reject.error);
            }
            TransactionResult::Abort(abort) => {
                panic!("Transaction abort with: {}", abort.reason);
            }
        }
    }
}
