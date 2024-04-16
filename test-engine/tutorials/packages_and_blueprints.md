## Instantiate a blueprint

To instantiate a blueprint, we first need to publish the package:

```Rust
test_engine.new_package("gumball package", "tests/gumball_machine/package");
```

The first argument is the name that we want to use to reference the package in the future and the second argument is the
path to the package from the `Cargo.toml` file of our package.

To avoid compiling the package in every single test, we can also use the `global_package!` macro and then instantiate an
engine with the global package or add it to an existing `test_engine`:

```Rust
global_package!(GUMBALL_PACKAGE, "tests/gumball_machine/package");

// Instantiate a new test engine with a global package.
let mut test_engine = TestEngine::with_package("gumball package", & GUMBALL_PACKAGE);

// Add a global package to a test engine.
let mut test_engine = TestEngine::new();
test_engine.add_global_package("gumbal package", & GUMBALL_PACKAGE);

```

Note that when we instantiate a package, it will be used
as the current default package for function calls. To change the current package, call the `set_current_package` method:

```Rust
test_engine.set_current_package(package_ref);
```

We can then instantiate our gumball component by calling the `new_component` method:

```Rust
test_engine.new_component(
"gumball comp", // Name to us as reference in the future
"GumballMachine", // Name of the component
"instantiate_gumball_machine", // Name of the function that instantiates the component
env_args!(dec!(5)), // Arguments to instantiate the package
);
```

The `env_args!` macro is the macro used by this package to pass arguments with environment references to calls (see more
in
next section). Note that the first component that has been instantiated is used as the default current component. We can
only call methods on the current component. We can set another component as current component by calling the
`set_current_component` method:

```Rust
test_engine.set_current_component(component_ref);
```