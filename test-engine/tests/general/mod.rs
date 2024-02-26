use radix_engine_common::network::NetworkDefinition;
use radix_engine_common::prelude::ResourceAddress;
use test_engine::test_engine::TestEngine;

#[test]
fn test_pre_allocated_token() {
    let mut test_engine = TestEngine::new();

    let address = ResourceAddress::new_or_panic([
        93, 11, 31, 125, 68, 106, 114, 113, 154, 80, 187, 244, 241, 233, 191, 51, 92, 8, 98, 88,
        43, 68, 5, 66, 3, 186, 89, 238, 225, 122,
    ]);

    test_engine.add_token(
        "test_token",
        1,
        "resource_tdx_2_1t5937l2ydfe8rxjsh060r6dlxdwqscjc9dzq2ssrhfv7act63say5g",
        NetworkDefinition::stokenet(),
    );

    let address_2 = test_engine.get_resource("test_token");
    assert_eq!(address, address_2)
}
