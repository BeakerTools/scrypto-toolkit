mod nft_marketplace_tests {
    use radix_engine_interface::dec;

    use test_engine::environment::Environment;
    use test_engine::receipt_traits::Outcome;
    use test_engine::test_engine::TestEngine;
    use test_engine::{env_args, env_vec, global_package};

    global_package!(NFT_MARKETPLACE_PACKAGE, "tests/nft_marketplace/package");

    fn bootstrap() -> TestEngine {
        let mut test_engine = TestEngine::with_package("nft marketplace", &NFT_MARKETPLACE_PACKAGE);
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

    fn init_dutch_auction() -> TestEngine {
        let mut test_engine = bootstrap();
        let car_id = test_engine.current_ids_balance("cars nft").pop();
        test_engine.new_component(
            "dutch_auction",
            "DutchAuction",
            "instantiate_dutch_auction",
            env_args![
                env_vec![Environment::NonFungibleBucket(
                    "cars nft",
                    vec![car_id.unwrap()]
                )],
                Environment::Resource("xrd"),
                dec!(10),
                dec!(5),
                10u64
            ],
        );
        test_engine.set_current_component("dutch auction");
        test_engine
    }

    #[test]
    fn test_init_dutch_auction() {
        let mut test_engine = init_dutch_auction();
        assert_eq!(test_engine.current_balance("Ownership badge"), dec!(1));
    }

    fn new_buyer(test_engine: &mut TestEngine, name: &str) {
        test_engine.new_account(name);
        test_engine.set_current_account(name);
        test_engine.call_faucet();
    }

    #[test]
    fn test_buy_dutch_auction() {
        let mut test_engine = init_dutch_auction();
        new_buyer(&mut test_engine, "buyer");
        let amount_owned_before = test_engine.current_balance("xrd");
        test_engine
            .call_method(
                "buy",
                env_args![Environment::FungibleBucket("xrd", dec!(10))],
            )
            .assert_is_success();
        let amount_owned_after = test_engine.current_balance("radix");
        assert_eq!(amount_owned_before - amount_owned_after, dec!(10));
        assert_eq!(test_engine.current_balance("cars nft"), dec!(1));
    }

    #[test]
    fn test_buy_after_epochs_dutch_auction() {
        let mut test_engine = init_dutch_auction();
        new_buyer(&mut test_engine, "buyer");
        test_engine.jump_epochs(5);
        let amount_owned_before = test_engine.current_balance("xrd");
        test_engine
            .custom_method_call(
                "buy",
                env_args![Environment::FungibleBucket("xrd", dec!(10))],
            )
            .output("tests/nft_marketplace/package/manifests/", "buy")
            .execute();
        let amount_owned_after = test_engine.current_balance("radix");
        assert_eq!(amount_owned_before - amount_owned_after, dec!("7.5"));
        assert_eq!(test_engine.current_balance("cars nft"), dec!(1));
    }

    #[test]
    fn test_buy_after_epochs_not_enough_fails_dutch_auction() {
        let mut test_engine = init_dutch_auction();
        new_buyer(&mut test_engine, "buyer");
        test_engine.jump_epochs(3);
        test_engine.call_method("buy", env_args![
            Environment::FungibleBucket("xrd", dec!(5))
        ]).assert_failed_with("[Buy]: Invalid quantity was provided. This sale can only go through when 8.5 tokens are provided.");
    }

    #[test]
    fn test_cancel_sale() {
        let mut test_engine = init_dutch_auction();
        test_engine.call_method_with_badge("cancel_sale", "Ownership badge", env_args!());
        let nfts_owned = test_engine.current_balance("cars nft");
        assert_eq!(nfts_owned, dec!(4));
    }
}
