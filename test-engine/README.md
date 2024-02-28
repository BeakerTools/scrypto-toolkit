# Test Engine
This library is a layer on top of the `scrypto-unit` library to make writing test easier.

# Usage
To use the library, add the following dev dependency to the `Cargo.toml` file
```
[dev-dependencies]
test-engine = { git = "https://github.com/BeakerDAO/scrypto-toolkit", branch = "main"}
```

# Features

## Basics
To instantiate a new `TestEngine`, call the `new` function:
```Rust
let mut test_engine = TestEngine::new()
```
Note that at instantiation a default account is created and is referenced by `default`. We can create a new account by 
calling the `new_account` method and give its reference name (see later):
```Rust
test_engine.new_account("custom");
```
We can then set this new account as default:
```Rust
test_engine.set_current_account("custom")
```
To call the faucet with the current account, we can call the `call_faucet` method:
```Rust
test_engine.call_faucet();
```
We can also create a new token with a given name with the `new_token` method:
```Rust
test_engine.new_token("btc", 21000000);
```
The second argument is the initial supply and can be of any type that can be casted into a `Decimal`.

## Environment References
The main feature of this package is to be able to reference any entity by a given name or its metadata name. This includes:
- Accounts
- Packages
- Components
- Resources

Its main use case is for method calls. When a resource is created its symbol and its name are registered as environment 
references, which are not case-sensitive. For example, we can get the XRD balance of the current account in the 
following ways:
```Rust
let xrd_balance = test_engine.current_balance("xrd"); 
let xrd_balance = test_engine.current_balance("XrD"); // Not case-sensitive.
let xrd_balance = test_engine.current_balance("RAd_Ix"); // _ is replaced by an empty character.
let xrd_balance = test_engine.current_balance("RA   dIx"); // spaces are replaced by empty characters.
let xrd_balance = test_engine.current_balance("RAdIx"); // Resources can also be referenced by their name.
```
We can also query the XRD balance of the `default` account:
```Rust
let custom_xrd_balance = test_engine.balance_of("def ault", "XRD");
```

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
let mut test_engine = TestEngine::with_package("gumball package", &GUMBALL_PACKAGE);

// Add a global package to a test engine.
let mut test_engine = TestEngine::new();
test_engine.add_global_package("gumbal package", &GUMBALL_PACKAGE);

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
The `env_args!` macro is the macro used by this package to pass arguments with environment references to calls (see more in
next section). Note that the first component that has been instantiated is used as the default current component. We can
only call methods on the current component. We can set another component as current component by calling the 
`set_current_component` method:
```Rust
test_engine.set_current_component(component_ref);
```

## Method calls
Now that we have instantiated a component, we can make calls on it by using the `call_method` method:
```Rust
test_engine.call_method(
            "buy_gumball",
            env_args!(Environment::FungibleBucket("XRD", dec!(10))),
        );
```
In the `env_args!` macro, usual Scrypto types such as `ComponentAddress` or `ResourceAddress` can be used. `Envrionment`
arguments can also be used. Here, we have used the `FungibleBucket` environment argument. It enabled us to automatically 
create a bucket with 20 XRD. The `Envionment` enum is defined by:
```Rust
pub enum Environment<E: EnvRef + Clone> {
    Account(E),
    Component(E),
    Package(E),
    FungibleBucket(E, Decimal),
    NonFungibleBucket(E, Vec<NonFungibleLocalId>),
    FungibleProof(E, Decimal),
    NonFungibleProof(E, Vec<NonFungibleLocalId>),
    Resource(E),
}
```
where `E` is a reference to the given entity. This enum combined with the `env_args!` macro enables us to only care about the arguments of 
our tests without having to create the Buckets/Proofs manually. Another [example](tests/nft_marketplace/unit_tests.rs) 
is the instantiation of a `DutchAuction`:
```Rust
test_engine.new_component(
            "dutch_auction",
            "DutchAuction",
            "instantiate_dutch_auction",
            env_args![
                vec![Environment::NonFungibleBucket(
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

## Custom method calls
By default, a method call makes the faucet pays for fee and deposits all remaining resources to the calling account. We
can change this by making calls with the `custom_method_call` method:
```Rust
test_engine.custom_method_call(
        "buy_gumball",
        env_args!(Environment::FungibleBucket("XRD", dec!(10))))
    .lock_fee("default", 20) // The second argument is of any type that can be casted to a Decimal
    .execute()
```
Don't forget the `execute` at the end of the call to make sure that it is executed! We can also change the account to 
which every remaining resources are deposited and also output the manifest:
```Rust
test_engine.custom_method_call(
        "buy_gumball",
        env_args!(Environment::FungibleBucket("XRD", dec!(10))))
    .deposit_batch("d Ef A    ult")
    .output(output_path, file_name)
    .execute()
```

## Method calls with badges
To call a method that requires a badge, we can use the `call_method_with_badge` method:
```Rust
test_engine.call_method_with_badge("cancel_sale", "Ownership badge", env_args!());
```
If we are making a custom call, we can use the `with_badge` option:
```Rust
test_engine.custom_method_call(("cancel_sale", env_args!())
    .with_badge("Ownership badge")
    .execute();
```

## Method's return
The `call_method` (or `execute()` if we made a custom call) method returns a `TransactionReceipt` for the transaction.
This library implements other ways to interact with the receipt. First, we can check that the call succeeded:
```Rust
test_engine.call_method(
            "buy_gumball",
            env_args!(Environment::FungibleBucket("XRD", dec!(10))),
        ).assert_is_success();
```
We can also check that a `panic!` has successfully been triggered with the correct message:
```Rust
test_engine.call_method("buy", env_args![
            Environment::FungibleBucket("xrd", dec!(5))
        ]).assert_failed_with("[Buy]: Invalid quantity was provided. This sale can only go through when 8.5 tokens are provided.");
```

Finally, we can also get the return of a method call:
```Rust
let price: Decimal = test_engine.call_method("get_price", env_args!()).get_price();
```
Note here that providing the expected returned type is required. Moreover, buckets and proofs are not properly supported
(returns a Bucket with a NodeID).

## More Examples
To understand how to use this library, tests on some `scrypto-examples` packages are available:
- [Hello World](tests/hello_world/unit_tests.rs)
- [Gumball Machine](tests/gumball_machine/unit_tests.rs)
- [Radiswap](tests/radiswap/unit_tests.rs)
- [NFT Marketplace](tests/nft_marketplace/unit_tests.rs)
