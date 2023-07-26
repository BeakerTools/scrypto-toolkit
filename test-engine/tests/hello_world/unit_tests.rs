mod hello_word_tests {
    use radix_engine::transaction::{TransactionOutcome, TransactionResult};
    use sdt_test_engine::env_args;
    use sdt_test_engine::test_engine::TestEngine;

    #[test]
    fn test_free_token(){
        let mut test_engine = TestEngine::new();
        test_engine.new_package("hello world", "tests/hello_world/package");
        test_engine.new_component("hello_comp", "Hello", "instantiate_hello", env_args!());
        let receipt = test_engine.call_method("free_token", env_args!());

        if let TransactionResult::Reject(reject) = &receipt.transaction_result {
            panic!("{}", reject.error);
        }
        else if let TransactionResult::Commit(commit) = &receipt.transaction_result {
            if let TransactionOutcome::Failure(failure) = &commit.outcome {
                panic!("{}", failure)
            }
        }

    }
}