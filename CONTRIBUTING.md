# Contributing Guide

## Getting started

### Reporting an issue

* **Ensure the bug was not already reported** by searching on GitHub under [Issues](https://github.com/BeakerDAO/scrypto-toolkit/issues).
* If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/BeakerDAO/scrypto-toolkit/issues/new). Be sure to include:
    * a **title**,
    * a **clear description**,
    * as much **relevant information** as possible,
    * a **code sample** or an **executable test case** demonstrating the expected behavior that is not occurring.

### Workflows

Development flow:
1. Create feature branches using develop as a starting point to start new work;
2. Submit a new pull request to the `develop` branch
    * please ensure the PR description clearly describes the problem and solution and include the relevant issue number if applicable.

### Branches

* Feature - `feature/cool-bananas`
* Development  - `develop`

## Conventions

### Code style

We use the default code style specified by [rustfmt](https://github.com/rust-lang/rustfmt). Just run `cargo fmt` before 
making the pull request.