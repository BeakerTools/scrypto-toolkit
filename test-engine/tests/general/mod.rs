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

#[test]
fn test_bug() {
    let mut simulator = LedgerSimulatorBuilder::new()
        .with_custom_genesis(CustomGenesis::default(
            Epoch::of(1),
            CustomGenesis::default_consensus_manager_config(),
        ))
        .without_kernel_trace()
        .build();

    let (pubkey, privkey, address) = simulator.new_account(false);

    let manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        address,
        "create_proof_of_amount",
        (XRD, dec!(100)),
    );
    let (manifest_builder, proof) =
        manifest_builder.add_instruction_advanced(InstructionV1::CreateProofFromAuthZoneOfAmount {
            resource_address: XRD,
            amount: dec!(100),
        });

    let transaction = manifest_builder.deposit_batch(address).build();

    simulator
        .execute_manifest(
            transaction,
            vec![NonFungibleGlobalId::from_public_key(&pubkey)],
        )
        .assert_is_success();
}
