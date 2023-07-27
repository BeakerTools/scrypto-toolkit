mod radiswap_tests{
    use radix_engine::types::dec;
    use radix_engine_interface::blueprints::resource::OwnerRole;
    use sdt_test_engine::env_args;
    use sdt_test_engine::environment::Environment;
    use sdt_test_engine::test_engine::TestEngine;

    fn initialize() -> TestEngine {
        let mut test_engine = TestEngine::new();
        test_engine.new_token("usd", dec!(100000));
        test_engine.new_token("btc", dec!(100));
        test_engine.new_package("radiswap package", "tests/hello_world/package");
        test_engine.new_component("radiswap", "Radiswap", "instantiate_pool", env_args!(OwnerRole::None, Environment::Resource("usd"), Environment::Resource("btc")));
        test_engine
    }

    #[test]
    fn test_add_liquidity(){
       let mut test_engine = initialize();
        let receipt = test_engine.call_method("add_liquidity", env_args!(Environment::FungibleBucket("usd", dec!(1000)), Environment::FungibleBucket("btc", dec!(1))));
    }

}