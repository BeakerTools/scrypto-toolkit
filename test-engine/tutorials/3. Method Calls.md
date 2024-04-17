# Method calls

Now that we have instantiated a component, we can make calls on it by using the `call_method` method:

```Rust
test_engine.call_method(
"buy_gumball", // Name of the method to call
env_args!(Environment::FungibleBucket("XRD", dec!(10))), // Arguments
);
```

This method will always call the current component. The previous method works fine for most use cases, but sometimes
we need to make more complex method calls. There are 6 ways to call a method in this package: 3 ways to make
[simple method calls](#simple-method-calls) and 3 other ways to make [complex method calls]().

Some [basic calls]() are also implemented to make life easier.

## Simple method calls

For simple method calls, we can use:

- `call_method` - for simple method calls.
- `call_method_with_badge` - for simple method calls that require a badge.
- `call_method_from` - for simple method calls to a global address.

To call a method that requires a badge, we can use the `call_method_with_badge` method:

```Rust
test_engine.call_method_with_badge(
"cancel_sale", // Name of the method to call
"Ownership badge", // Resource Reference to the badge to use
env_args!() // Arguments
);
```

## Complex method calls

For more complex method calls, we can use:

- `call_method_builder` - for complex calls on a given method.
- `call_method_builder_from` - for complex calls to method from a global address.
- `build_call` - for a totally manual complex call.

These three methods will return a `CallBuilder` which will enable you to choose more parameters or even to make multiple
method calls at once.

By default, a method call makes the faucet pays for fee and deposits all remaining resources to the calling account. We
can change this by making calls with the `custom_method_call` method:

```Rust
test_engine.call_methode_builder(
"buy_gumball",
env_args!(Environment::FungibleBucket("XRD", dec!(10))))
.lock_fee("default", 20) // The second argument is of any type that can be casted to a Decimal
.deposit_batch("User 2")
.execute()
```

/!\ Don't forget the `execute` at the end of the call to make sure that it is executed! We can also output the manifest:

```Rust
test_engine.call_methode_builder(
"buy_gumball",
env_args!(Environment::FungibleBucket("XRD", dec!(10))))
.deposit_batch("d Ef A    ult")
.output(output_path, file_name)
.execute()
```

## Basic calls

In addition to the manual method calls, a variety of usual calls are implemented:

- `transfer` - to transfer tokens between accounts.
- `transfer_non_fungibles` - to transfer nfts between accounts.
- `update_non_fungible_data` - to update an nft's data.

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
let price: Decimal = test_engine.call_method("get_price", env_args!()).get_return();
```

Note here that providing the expected returned type is required. Moreover, buckets and proofs are not properly supported
(returns a Bucket with a NodeID).