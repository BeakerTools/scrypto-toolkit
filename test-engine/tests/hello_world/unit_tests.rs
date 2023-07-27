mod hello_word_tests {
    use radix_engine::types::Decimal;
    use sdt_test_engine::env_args;
    use sdt_test_engine::test_engine::TestEngine;
    use sdt_test_engine::receipt_traits::Outcome;

    #[test]
    fn test_free_token(){
        let mut test_engine = TestEngine::new();
        test_engine.new_package("hello world", "tests/hello_world/package");
        test_engine.new_component("hello_comp", "Hello", "instantiate_hello", env_args!());
        let receipt = test_engine.call_method("free_token", env_args!());
        receipt.assert_is_success();
        let amount_owned = test_engine.current_balance("Hello Token");
        assert_eq!(amount_owned, Decimal::one())
    }
}