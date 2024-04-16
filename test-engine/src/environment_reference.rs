use crate::test_engine::TestEngine;
use radix_engine_common::prelude::ResourceAddress;
use radix_engine_common::types::{ComponentAddress, GlobalAddress};

///
pub trait Reference {
    fn format(&self) -> String;
}

/// Reference to a global entity (account or component).
pub trait EntityReference {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress;
}

/// Reference to a resource.
pub trait ResourceReference {
    fn address(&self, test_engine: &TestEngine) -> ResourceAddress;
}

/// Reference to a global address (account, component or resource)
pub trait GlobalAddressReference {
    fn address(&self, test_engine: &TestEngine) -> GlobalAddress;
}

impl Reference for String {
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace(['_', ' '], "")
    }
}

impl<'a> Reference for &'a String {
    fn format(&self) -> String {
        (*self).format()
    }
}

impl<'a> Reference for &'a str {
    fn format(&self) -> String {
        self.to_string().format()
    }
}

impl<T: Reference> EntityReference for T {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress {
        test_engine.get_entity(self.format())
    }
}

impl EntityReference for ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        *self
    }
}

impl<'a> EntityReference for &'a ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        **self
    }
}
impl<T: Reference> ResourceReference for T {
    fn address(&self, test_engine: &TestEngine) -> ResourceAddress {
        test_engine.get_resource(self.format())
    }
}

impl ResourceReference for ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> ResourceAddress {
        *self
    }
}

impl<'a> ResourceReference for &'a ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> ResourceAddress {
        **self
    }
}

impl<T: EntityReference> GlobalAddressReference for T {
    fn address(&self, test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(self.address(test_engine))
    }
}

impl GlobalAddressReference for ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(*self)
    }
}

impl<'a> GlobalAddressReference for &'a ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(**self)
    }
}
