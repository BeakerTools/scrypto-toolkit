mod nft_marketplace_tests {
    use radix_engine_interface::dec;
    use std::env::args;
    use test_engine::env_args;
    use test_engine::environment::Environment;
    use test_engine::test_engine::TestEngine;

    fn bootstrap() -> TestEngine {
        let mut test_engine = TestEngine::new();
        test_engine.new_package("nft marketplace", "tests/nft_marketplace/package");
        test_engine.new_component("bootstrap", "Bootstrap", "bootstrap", env_args!());
        test_engine
    }

    #[test]
    fn test_bootstrap() {
        let mut test_engine = bootstrap();
        let cars_owned = test_engine.current_balance("cars nft");
        let phones_owned = test_engine.current_balance("phones nft");
        let laptop_owned = test_engine.current_balance("laptops nft");
        assert_eq!(cars_owned, dec!(4));
        assert_eq!(phones_owned, dec!(4));
        assert_eq!(laptop_owned, dec!(4));
    }

    fn init_dutch_auction(test_engine: &mut TestEngine) {
        let car_id = test_engine.current_ids_balance("cars nft").pop();
        test_engine.new_component(
            "dutch_auction",
            "DutchAuction",
            "instantiate_dutch_auction",
            env_args![
                vec![Environment::NonFungibleBucket(
                    "cars nft",
                    vec![car_id.unwrap()]
                )],
                Environment::Resource("xrd"),
                dec!(10),
                dec!(1000),
                dec!(500),
                14
            ],
        );
    }

    #[test]
    fn test_init_dutch_auction() {
        let mut test_engine = bootstrap();
        init_dutch_auction(&mut test_engine);
        assert_eq!(test_engine.current_balance("Ownership badge"), dec!(1));
    }

    fn new_buyer(test_engine: &mut TestEngine, name: &str) {
        test_engine.new_account(name.clone());
        test_engine.set_current_account(name);
        test_engine.call_faucet();
    }
}
