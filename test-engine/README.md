# Test Engine

This library is a layer on top of the `scrypto-unit` library to make writing test easier.

# Usage

To use the library, add the following dev dependency to the `Cargo.toml` file

```
[dev-dependencies]
test-engine = { git = "https://github.com/BeakerTools/scrypto-toolkit", branch = "main"}
```

# Main Features

- [Basics](tutorials/basics.md)
- [Packages and blueprints](tutorials/packages_and_blueprints.md)
- [Calling methods](tutorials/method_call.md)
- [Other features](tutorials/)

# Examples

To understand how to use this library, tests on some `scrypto-examples` packages are available:

- [Hello World](tests/hello_world/unit_tests.rs)
- [Gumball Machine](tests/gumball_machine/unit_tests.rs)
- [Radiswap](tests/radiswap/unit_tests.rs)
- [NFT Marketplace](tests/nft_marketplace/unit_tests.rs)

More features and broader test examples can be found at the following repos:

- [Shardz NFT project](https://github.com/Radix-Shardz/scrypto-blueprints)
