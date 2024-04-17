# Packages and blueprints

## Create a package

To add a package to the `TestEngine`, we can use the `new_package` method:

```Rust
test_engine.new_package("gumball package", "tests/gumball_machine/package");
```

The first argument is the name that we want to use to reference the package and the second argument is the
path to the package from the `Cargo.toml` file of our package.

The problem with this is that it will recompile the package for every test we are going to write. To avoid this issue,
, we can also use the `global_package!` macro and then instantiate an
engine with the global package or add it to an existing `test_engine`:

```Rust
global_package!(GUMBALL_PACKAGE, "tests/gumball_machine/package");

// Instantiate a new test engine with a global package.
let mut test_engine = TestEngine::with_package("gumball package", & GUMBALL_PACKAGE);

// Add a global package to a test engine.
let mut test_engine = TestEngine::new();
test_engine.add_global_package("gumbal package", & GUMBALL_PACKAGE);

```

/!\ Don't forget the `&` before the package name in the two previous methods.

Note that when we instantiate a package, it will be used
as the current default package for function calls. To change the current package, call the `set_current_package` method:

```Rust
test_engine.set_current_package( < package_reference_name>);
```

## Instantiate a blueprint

We can then instantiate our gumball component by calling the `new_component` method:

```Rust
test_engine.new_component(
"gumball comp", // Name to use as reference
"GumballMachine", // Name of the component in the package
"instantiate_gumball_machine", // Name of the function that instantiates the component
env_args!(dec!(5)), // Arguments to instantiate the package
);
```

If the component instantiation requires a badge, we can use the `new_component_with_badge` method:

```Rust
test_engine.new_component_with_badge(
"gumball comp", // Name to use as reference 
"GumballMachine", // Name of the component in the package
"instantiate_gumball_machine", // Name of the function that instantiates the component
< badge_resource_reference>, // Resource reference to the badge to use
env_args!(dec!(5)), // Arguments to instantiate the package
);
```

Note that the first component that has been instantiated is used as the default current component. We can
only call methods on the current component. We can set another component as current component by calling the
`set_current_component` method:

```Rust
test_engine.set_current_component(component_ref);
```

## Arguments macros

In the previous examples, we used the `env_args!` macro. This enables us to easily deal with arguments using
`ReferenceName`. In the `env_args!` macro, every usual Scrypto types such as `ComponentAddress` or `ResourceAddress` can
be used. Additionally, any variant of the `Environment` enum can be used:

```Rust
pub enum Environment<N: ReferenceName + Clone> {
    Account(N),
    Component(N),
    Package(N),
    WorkTopFungibleBucket(N, Decimal),
    FungibleBucket(N, Decimal),
    WorktopNonFungibleBucket(N, Vec<NonFungibleLocalId>),
    NonFungibleBucket(N, Vec<NonFungibleLocalId>),
    AuthZoneFungibleProof(N, Decimal),
    FungibleProof(N, Decimal),
    AuthZoneNonFungibleProof(N, Vec<NonFungibleLocalId>),
    NonFungibleProof(N, Vec<NonFungibleLocalId>),
    Resource(N),
}
```

where `N` is a `ReferenceName` of the given entity. This enum combined with the `env_args!` macro enables us to only
care
about the arguments of our tests without having to create the Buckets/Proofs manually. For example, if a component takes
a xrd bucket as its single argument to instantiate, we can write:

```Rust
test_engine.new_component(
"my component",
"MyComponent",
"instantiate_with_bucket",
env_args!(Environment::FungibleBucket("xrd", dec!5))
)
```

A more complex [example](tests/nft_marketplace/unit_tests.rs)
is the instantiation of a `DutchAuction`:

```Rust
test_engine.new_component(
"dutch_auction",
"DutchAuction",
"instantiate_dutch_auction",
env_args![
                env_vec![Environment::NonFungibleBucket(
                    "cars nft",
                    vec![car_id.unwrap()]
                )],
                Environment::Resource("xrd"),
                dec!(10),
                dec!(5),
                10 as u64
            ],
);
```

Here we used the `env_vec!` macro to make an array of `Environment` variables.

To use a custom type/enum in the `env_args!` macro, simply implement the trait `ManifestSbor` trait.
For example, the following type can be used:

```rust
#[ManifestSbor]
pub struct SomeType {
    some_fields: FieldsType
}

```