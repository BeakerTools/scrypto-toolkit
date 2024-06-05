use crate::internal_prelude::*;
use crate::test_engine::TestEngine;

///
pub trait ReferenceName {
    fn format(&self) -> String;
}

/// Reference to a global entity (account or component).
pub trait ComponentReference {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress;
}

/// Reference to a resource.
pub trait ResourceReference {
    fn address(&self, test_engine: &TestEngine) -> ResourceAddress;
}

/// Reference to a global address (account, component or resource)
pub trait GlobalReference {
    fn address(&self, test_engine: &TestEngine) -> GlobalAddress;
}

impl ReferenceName for String {
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace(['_', ' '], "")
    }
}

impl<'a> ReferenceName for &'a String {
    fn format(&self) -> String {
        (*self).format()
    }
}

impl<'a> ReferenceName for &'a str {
    fn format(&self) -> String {
        self.to_string().format()
    }
}

impl<T: ReferenceName> ComponentReference for T {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress {
        test_engine.get_entity(self.format())
    }
}

impl ComponentReference for ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        *self
    }
}

impl<'a> ComponentReference for &'a ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        **self
    }
}
impl<T: ReferenceName> ResourceReference for T {
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

impl<T: ComponentReference> GlobalReference for T {
    fn address(&self, test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(self.address(test_engine))
    }
}

impl GlobalReference for ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(*self)
    }
}

impl<'a> GlobalReference for &'a ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> GlobalAddress {
        GlobalAddress::from(**self)
    }
}
