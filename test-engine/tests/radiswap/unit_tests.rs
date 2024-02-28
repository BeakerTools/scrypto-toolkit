mod radiswap_tests {
    use radix_engine::types::dec;
    use radix_engine_interface::blueprints::resource::OwnerRole;
    use test_engine::environment::Environment;
    use test_engine::test_engine::TestEngine;
    use test_engine::{env_args, global_package};

    global_package!(RADISWAP_PACKAGE, "tests/radiswap/package");

    fn initialize() -> TestEngine {
        let mut test_engine = TestEngine::with_package("radiswap package", &RADISWAP_PACKAGE);
        test_engine.new_token("usd", dec!(100000));
        test_engine.new_token("btc", dec!(100));
        test_engine.new_component(
            "radiswap",
            "Radiswap",
            "new",
            env_args!(
                OwnerRole::None,
                Environment::Resource("usd"),
                Environment::Resource("btc")
            ),
        );
        test_engine
    }

    #[test]
    fn test_add_liquidity() {
        let mut test_engine = initialize();
        test_engine.call_method(
            "add_liquidity",
            env_args!(
                Environment::FungibleBucket("usd", dec!(1000)),
                Environment::FungibleBucket("btc", dec!(1))
            ),
        );
        let usd_amount = test_engine.current_balance("usd");
        let btc_amount = test_engine.current_balance("btc");
        assert_eq!(usd_amount, dec!(99000));
        assert_eq!(btc_amount, dec!(99));
    }

    #[test]
    fn test_swap() {
        let mut test_engine = initialize();
        test_engine.call_method(
            "add_liquidity",
            env_args!(
                Environment::FungibleBucket("usd", dec!(1000)),
                Environment::FungibleBucket("btc", dec!(1))
            ),
        );
        test_engine.call_method(
            "swap",
            env_args!(Environment::FungibleBucket("usd", dec!(1000))),
        );
        let usd_amount = test_engine.current_balance("usd");
        let btc_amount = test_engine.current_balance("btc");
        assert_eq!(usd_amount, dec!(98000));
        assert_eq!(btc_amount, dec!("99.5"));
    }
}
