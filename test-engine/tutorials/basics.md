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

## References

The main feature of this package is to be able to reference entities(account, packages, resources, components) by given
names or address called a `NameReference`. These references are not case-sensitive and spaces and underscores are
ignored. The following strings refer to the same `NameReferences`: `xrd`, `XrD`, `RAd_Ix`, `RA   dIx`, `RAdIx`.  
Additionnaly, there are 3 other types of references:

- `ComponentReference`: A string reference or `ComponentAddress` of a component (an account name also works).
- `ResourceReference`: A string reference or `ResourceAddress` of a resource.
- `GlobalReference`: A string reference or `ComponentAddress` or `ResourceAddress` of a component/resource.

For example, we can get the XRD balance of the current account in the following ways:

```Rust
let xrd_balance = test_engine.current_balance("xrd");
let xrd_balance = test_engine.current_balance("XrD"); // Not case-sensitive.
let xrd_balance = test_engine.current_balance("RAd_Ix"); // _ is replaced by an empty character.
let xrd_balance = test_engine.current_balance("RA   dIx"); // spaces are replaced by empty characters.
let xrd_balance = test_engine.current_balance("RAdIx"); // Resources can also be referenced by their name.
let xrd_balance = test.engine.current_balance( < xrd_resource_address>);
```

References are created manually when a `ReferenceName` is supplied or automatically from resources and components
metadata. For a resource, its `name` and `symbol` are parsed and can be used as a reference. For a component, its `name`
metadata(if it exists) can be used as reference.