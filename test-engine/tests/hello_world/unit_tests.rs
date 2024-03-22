mod hello_word_tests {
    use radix_engine::types::Decimal;
    use test_engine::env_args;
    use test_engine::receipt_traits::Outcome;
    use test_engine::test_engine::TestEngine;

    #[test]
    fn test_free_token() {
        let mut test_engine = TestEngine::new();
        println!("1");
        test_engine.new_package("hello world", "tests/hello_world/package");
        println!("2");
        test_engine.new_component("hello_comp", "Hello", "instantiate_hello", env_args!());
        println!("3");
        let receipt = test_engine.call_method("free_token", env_args!());
        println!("4");
        receipt.assert_is_success();
        let amount_owned = test_engine.current_balance("Hello Token");
        assert_eq!(amount_owned, Decimal::one())
    }
}
