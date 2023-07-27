use radix_engine::transaction::{TransactionOutcome, TransactionReceipt, TransactionResult};
use crate::from_instruction::FromInstruction;

pub trait Outcome {
    fn assert_is_success(&self);
}

impl Outcome for TransactionReceipt {
    fn assert_is_success(&self) {
        if let TransactionResult::Reject(reject) = &self.transaction_result {
            panic!("{}", reject.error);
        }
        else if let TransactionResult::Commit(commit) = &self.transaction_result {
            if let TransactionOutcome::Failure(failure) = &commit.outcome {
                panic!("{}", failure)
            }
        }
    }
}

pub trait GetReturn<T> {
    fn get_return(&self) -> T;
}

impl<T> GetReturn<T> for TransactionReceipt
where T: FromInstruction {
    fn get_return(&self) -> T {
        match &self.transaction_result
        {
            TransactionResult::Commit(commit) =>
                {
                    match &commit.outcome{
                        TransactionOutcome::Success(output) => {
                            T::from(output.clone())
                        }
                        TransactionOutcome::Failure(failure) =>
                            {
                                panic!("Transaction failed with: {}", failure);
                            }
                    }
                }
            TransactionResult::Reject(reject) =>
                {
                    panic!("Transaction rejected with: {}", reject.error);
                }
            TransactionResult::Abort(abort) =>
                {
                    panic!("Transaction abort with: {}", abort.reason);
                }
        }
    }
}