## Method calls

Now that we have instantiated a component, we can make calls on it by using the `call_method` method:

```Rust
test_engine.call_method(
"buy_gumball",
env_args!(Environment::FungibleBucket("XRD", dec!(10))),
);
```

In the `env_args!` macro, usual Scrypto types such as `ComponentAddress` or `ResourceAddress` can be used. `Environment`
arguments can also be used. Here, we have used the `FungibleBucket` environment argument. It enabled us to automatically
create a bucket with 20 XRD. The `Environment` enum is defined by:

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

where `E` is a reference to the given entity. This enum combined with the `env_args!` macro enables us to only care
about the arguments of
our tests without having to create the Buckets/Proofs manually. Another [example](tests/nft_marketplace/unit_tests.rs)
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

Here we used the `env_vec!` macro to make an array of `Environment` variables. To conclude, any type that implements the
trait `ManifestSbor` can also be used as direct argument in the `env_arg` macro. The following struct can be used:

```rust
#[ManifestSbor]
pub struct SomeType {
    some_fields: FieldsType
}

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