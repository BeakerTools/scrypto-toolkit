use test_engine::prelude::*;

#[test]
fn test_pre_allocated_token() {
    let mut test_engine = TestEngine::new();

    let address = ResourceAddress::new_or_panic([
        93, 11, 31, 125, 68, 106, 114, 113, 154, 80, 187, 244, 241, 233, 191, 51, 92, 8, 98, 88,
        43, 68, 5, 66, 3, 186, 89, 238, 225, 122,
    ]);

    test_engine.new_token_with_address(
        "test_token",
        1,
        "resource_tdx_2_1t5937l2ydfe8rxjsh060r6dlxdwqscjc9dzq2ssrhfv7act63say5g",
        NetworkDefinition::stokenet(),
    );

    let address_2 = test_engine.get_resource("test_token");
    assert_eq!(address, address_2)
}

#[test]
fn test_transfer() {
    let mut test_engine = TestEngine::new();

    test_engine.new_token("Test token", 1000, 18);

    test_engine.new_account("Recipient");
    assert_eq!(test_engine.balance_of("Recipient", "Test token"), dec!(0));

    test_engine
        .transfer("Recipient", "Test token", dec!(10))
        .assert_is_success();
    assert_eq!(test_engine.balance_of("Recipient", "Test token"), dec!(10));
}
