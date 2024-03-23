use crate::test_engine::TestEngine;
use radix_engine_common::prelude::ResourceAddress;
use radix_engine_common::types::ComponentAddress;

pub trait EnvRef {
    fn format(&self) -> String;
}

impl EnvRef for String {
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace(['_', ' '], "")
    }
}

impl<'a> EnvRef for &'a String {
    fn format(&self) -> String {
        (*self).format()
    }
}

impl<'a> EnvRef for &'a str {
    fn format(&self) -> String {
        self.to_string().format()
    }
}

pub trait EntityRef {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress;
}

impl<T: EnvRef> EntityRef for T {
    fn address(&self, test_engine: &TestEngine) -> ComponentAddress {
        test_engine.get_entity(self.format())
    }
}

impl EntityRef for ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        self.clone()
    }
}

impl<'a> EntityRef for &'a ComponentAddress {
    fn address(&self, _test_engine: &TestEngine) -> ComponentAddress {
        **self
    }
}

pub trait ResourceRef {
    fn address(&self, test_engine: &TestEngine) -> ResourceAddress;
}

impl<T: EnvRef> ResourceRef for T {
    fn address(&self, test_engine: &TestEngine) -> ResourceAddress {
        test_engine.get_resource(self.format())
    }
}

impl ResourceRef for ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> ResourceAddress {
        self.clone()
    }
}

impl<'a> ResourceRef for &'a ResourceAddress {
    fn address(&self, _test_engine: &TestEngine) -> ResourceAddress {
        **self
    }
}
