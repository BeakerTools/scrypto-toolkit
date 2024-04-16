# Basics

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

The main feature of this package is to be able to reference any entity by a given name or its metadata name. This
includes:

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
