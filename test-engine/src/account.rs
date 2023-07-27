use crate::engine_interface::EngineInterface;
use radix_engine::types::Secp256k1PublicKey;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::prelude::NonFungibleGlobalId;
use radix_engine_interface::types::ComponentAddress;

#[derive(Clone)]
pub struct Account {
    component_address: ComponentAddress,
    public_key: Secp256k1PublicKey,
}

impl Account {
    pub fn new(engine_interface: &mut EngineInterface) -> Self {
        let (public_key, _, component_address) = engine_interface.new_account();
        Self {
            public_key,
            component_address,
        }
    }

    pub fn address(&self) -> &ComponentAddress {
        &self.component_address
    }

    pub fn proof(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.public_key)
    }
}
