use crate::call_builder::CallBuilder;
use crate::environment::EnvironmentEncode;
use crate::internal_prelude::*;
use crate::references::{GlobalReference, ResourceReference};

pub trait SimpleMethodCaller {
    /// Makes a simple call to a method of the current component.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_method(
        self,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt;

    /// Makes a simple call to a method of a given entity.
    ///
    /// # Arguments
    /// * `global_address`: reference or address of the entity to call.
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_method_from<G: GlobalReference>(
        self,
        global_address: G,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt;

    /// Calls a method of the current component with a given admin badge.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `admin_badge`: reference name or address of the resource to use as an admin badge.
    /// * `args`: environment arguments to call the method.
    fn call_method_with_badge<R: ResourceReference>(
        self,
        method_name: &str,
        admin_badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt;
}

pub trait ComplexMethodCaller {
    /// Returns a new call builder.
    fn call_builder(&mut self) -> CallBuilder;

    /// Returns a call builder with an initial method call.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call(&mut self, method_name: &str, args: Vec<Box<dyn EnvironmentEncode>>) -> CallBuilder;

    /// Returns a call builder with an initial method call with a given admin badge.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `admin_badge`: reference name or address of the resource to use as an admin badge.
    /// * `args`: environment arguments to call the method.
    fn call_with_badge<R: ResourceReference>(
        &mut self,
        method_name: &str,
        admin_badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> CallBuilder;

    /// Returns a call builder with an initial method call to a given entity.
    ///
    /// # Arguments
    /// * `global_address`: reference or address of the entity to call.
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_from<G: GlobalReference>(
        &mut self,
        global_address: G,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> CallBuilder;

    fn with_manifest_builder<F>(&mut self, f: F) -> CallBuilder
    where
        F: FnOnce(ManifestBuilder) -> ManifestBuilder;

    fn withdraw<R: ResourceReference>(&mut self, resource: R, amount: Decimal) -> CallBuilder;
}
