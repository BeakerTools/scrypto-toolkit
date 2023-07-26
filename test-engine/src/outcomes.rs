use radix_engine::transaction::{TransactionOutcome, TransactionReceipt, TransactionResult};

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