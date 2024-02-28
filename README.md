# Scrypto Toolkit
This library aims at providing open source tools for Scrypto developers.
At the moment, these include:  
- [Better testing engine](test-engine/README.md)
- [Data structures](data-structures/README.md)
- [Maths library](maths/README.md) (Needs an overhaul)



## Contribute:
To contribute please follow the [contribution guide](CONTRIBUTING.md). The following features are open for contribution.

### Data structures
- [ ] Add more features to `BigVec`
- [ ] Implement more data structures

### Test Engine
- [ ] Add a feature to manage static packages to avoid compilation of the same package multiple times during tests
- [ ] Implement the `Environment` trait for objects implementing the `ScryptoSbor` trait
- [ ] Implement a better way to deal with Buckets/Proofs return.
- [ ] Implement a nice way of querying component states.
- [ ] Deal with more transaction failures.

### Maths
- [ ] Replace methods' implementation with Padé approximants to improve performance
- [ ] Create a CustomDecimal macro to create custom 256 bits decimal types
- [ ] Implement other maths function
- [ ] Do performance tests

