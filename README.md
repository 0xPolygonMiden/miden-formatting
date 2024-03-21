# `miden-formatting`

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/0xPolygonMiden/miden-formatting/blob/main/LICENSE)
[![RUST_VERSION](https://img.shields.io/badge/rustc-1.76+-lightgray.svg)]()
[![CRATE](https://img.shields.io/crates/v/miden-formatting)](https://crates.io/crates/miden-formatting)
[![CI](https://github.com/0xPolygonMiden/miden-formatting/actions/workflows/ci.yml/badge.svg)](https://github.com/0xPolygonMiden/miden-formatting/actions/workflows/ci.yml)

This crate provides some general infrastructure for pretty-printers and value foramtting that is needed by various Miden crates. Rather than implement this
stuff in every place where it is needed, we've extracted the most important and general bits and put them in this crate.

## Usage

Add `miden-formatting` to your `Cargo.toml`:

```toml
[dependencies]
miden-formatting = "0.1"
```

For `#![no_std]` builds:

```toml
[dependencies]
miden-formatting = { version = "0.1", default-features = false }
```

There is a `std` feature you can use to conditionally enable functionality that requires libstd to implement. For now this features is not actually needed, but is likely to be used in the future, so we're providing it now.

## Intro

Most likely you are pulling in this crate to make use of the pretty-printer infrastructure. See the documentation for the `PrettyPrint` trait for a comprehensive intro to how to get started with it.

You may also be interested in the example syntax tree defined [here](./formatting/src/prettier/tests.rs). This makes use of most features of the pretty printer in a small made-up language for learning.

# License

This project is [MIT licensed](./LICENSE)
