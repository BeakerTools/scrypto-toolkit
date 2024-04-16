use crate::call_builder::CallBuilder;
use crate::environment::EnvironmentEncode;
use crate::references::{GlobalReference, ResourceReference};
use radix_engine::transaction::TransactionReceipt;

pub trait MethodCaller {
    /// Returns a new call builder.
    fn build_call(&mut self) -> CallBuilder;

    /// Returns a call builder with an initial method call.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_method_builder(
        &mut self,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> CallBuilder;

    /// Returns a call builder with an initial method call to a given entity.
    ///
    /// # Arguments
    /// * `global_address`: reference or address of the entity to call.
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_method_builder_from<G: GlobalReference>(
        &mut self,
        global_address: G,
        method_name: &str,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> CallBuilder;

    /// Makes a simple call to a method of the current component.
    ///
    /// # Arguments
    /// * `method_name`: name of the method.
    /// * `args`: environment arguments to call the method.
    fn call_method(
        &mut self,
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
        &mut self,
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
        &mut self,
        method_name: &str,
        admin_badge: R,
        args: Vec<Box<dyn EnvironmentEncode>>,
    ) -> TransactionReceipt;
}
