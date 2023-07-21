# `::maybe-dangling`

`ManuallyDrop<T>` and `MaybeDangling<T>` semantics in stable Rust as per <https://github.com/rust-lang/rfcs/pull/3336>

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs)
[![Latest version](https://img.shields.io/crates/v/maybe-dangling.svg)](
https://crates.io/crates/maybe-dangling)
[![Documentation](https://docs.rs/maybe-dangling/badge.svg)](
https://docs.rs/maybe-dangling)
[![MSRV](https://img.shields.io/badge/MSRV-1.65.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![License](https://img.shields.io/crates/l/maybe-dangling.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/maybe-dangling.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

This crates offers two types, `ManuallyDrop<T>`, and `MaybeDangling<T>`, which do not carry
[aliasing/`dereferenceable`-ity properties](https://github.com/rust-lang/rfcs/pull/3336) w.r.t. the
`T` they each contain, which means they are allowed to:
 1. have some expired value inside of them, such as `T = &'expired …`,
 1. be fed to a function that does not inspect its value (such as `::core::mem::forget()`),
 1. exhibit well-defined behavior (no UB!).

## References

  - **The RFC that shall eventually and ultimately supersede this very crate: <https://github.com/rust-lang/rfcs/pull/3336>**

  - The `miri` PR implementing the check against this: <https://github.com/rust-lang/miri/pull/2985>

  - The soundness problem of `::ouroboros` stemming from not using this: <https://github.com/joshua-maros/ouroboros/issues/88>

  - The soundness problem of `::yoke` stemming from not using this: <https://github.com/unicode-org/icu4x/issues/3696>

  - [An URLO thread on the topic, and my post exposing the intention to write this very crate](https://users.rust-lang.org/t/unsafe-code-review-semi-owning-weak-rwlock-t-guard/95706/15?u=yandros)
