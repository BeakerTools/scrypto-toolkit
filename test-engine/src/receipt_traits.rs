use crate::from_instruction::FromInstruction;
use crate::internal_prelude::*;

pub trait Outcome {
    fn assert_is_success(self) -> Self;
    fn assert_failed_with(self, error: &str) -> Self;
}

impl Outcome for TransactionReceipt {
    /// Asserts that the transaction succeeded.
    /// Panics if the transaction was rejected, aborted or failed.
    fn assert_is_success(self) -> Self {
        match &self.result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(_) => self,
                TransactionOutcome::Failure(failure) => {
                    panic!("Transaction failed with: {}", failure);
                }
            },
            TransactionResult::Reject(reject) => {
                panic!("Transaction rejected with: {}", reject.reason);
            }
            TransactionResult::Abort(abort) => {
                panic!("Transaction aborted with: {}", abort.reason);
            }
        }
    }

    /// Asserts that the transaction failed with a given message.
    /// Panics if the transaction succeeded or was rejected/aborted.
    ///
    /// # Arguments
    /// * `error` : Expected error message.
    fn assert_failed_with(self, error: &str) -> Self {
        match &self.result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(_) => {
                    panic!("Transaction succeeded !");
                }
                TransactionOutcome::Failure(failure) => {
                    if failure.to_string().contains(error) {
                        self
                    } else {
                        panic!(
                            "Transaction did not fail with expected error ! \n\
                                Error: {} \n\
                                Expected Error: {}",
                            failure, error
                        );
                    }
                }
            },
            TransactionResult::Reject(reject) => {
                panic!("Transaction rejected with: {}", reject.reason);
            }
            TransactionResult::Abort(abort) => {
                panic!("Transaction aborted with: {}", abort.reason);
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
        match &self.result {
            TransactionResult::Commit(commit) => match &commit.outcome {
                TransactionOutcome::Success(output) => T::from(output.clone()),
                TransactionOutcome::Failure(failure) => {
                    panic!("Transaction failed with: {}", failure);
                }
            },
            TransactionResult::Reject(reject) => {
                panic!("Transaction rejected with: {}", reject.reason);
            }
            TransactionResult::Abort(abort) => {
                panic!("Transaction abort with: {}", abort.reason);
            }
        }
    }
}
