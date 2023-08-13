mod nft_marketplace_tests {
    use test_engine::env_args;
    use test_engine::test_engine::TestEngine;

    fn bootstrap() -> TestEngine {
        let mut test_engine = TestEngine::new();
        test_engine.new_package("nft marketplace","tests/nft_marketplace/package");
        test_engine.new_component("bootstrap", "Bootstrap", "bootstrap", env_args!());
        test_engine.new_component("dutch", "DutchAuction", "instantiate_dutch_auction", );
        test_engine
    }

}