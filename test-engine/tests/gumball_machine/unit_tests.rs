mod gumball_machine_tests {
    use test_engine::prelude::*;

    global_package!(GUMBALL_PACKAGE, "tests/gumball_machine/package");

    fn instantiate_gumball() -> TestEngine {
        let mut test_engine = TestEngine::with_package("gumball package", &GUMBALL_PACKAGE);
        test_engine.new_component(
            "gumball comp",
            "GumballMachine",
            "instantiate_gumball_machine",
            env_args!(dec!(5)),
        );
        test_engine
    }

    #[test]
    fn test_buy_gumball_success() {
        let mut test_engine = instantiate_gumball();
        let receipt =
            test_engine.call_method("buy_gumball", env_args!(Fungible::FromAccount("XRD", 10)));
        receipt.assert_is_success();
        let amount_owned = test_engine.current_balance("GUM");
        assert_eq!(amount_owned, Decimal::one())
    }

    #[test]
    fn test_buy_gumball_fail() {
        let mut test_engine = instantiate_gumball();
        test_engine
            .call_method("buy_gumball", env_args!(Fungible::FromAccount("XRD", 1)))
            .assert_failed_with("");
        let amount_owned = test_engine.current_balance("GUM");
        assert_eq!(amount_owned, Decimal::zero())
    }

    #[test]
    fn test_get_price() {
        let mut test_engine = instantiate_gumball();
        let receipt = test_engine.call_method("get_price", env_args!());
        let price: Decimal = receipt.get_return();
        assert_eq!(price, dec!(5));
    }
}
