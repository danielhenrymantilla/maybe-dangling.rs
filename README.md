# `::maybe-dangling`

`MaybeDangling<T>` in stable Rust, as per <https://github.com/rust-lang/rfcs/pull/3336>

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs)
[![Latest version](https://img.shields.io/crates/v/maybe-dangling.svg)](
https://crates.io/crates/maybe-dangling)
[![Documentation](https://docs.rs/maybe-dangling/badge.svg)](
https://docs.rs/maybe-dangling)
[![MSRV](https://img.shields.io/badge/MSRV-1.96.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![License](https://img.shields.io/crates/l/maybe-dangling.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/maybe-dangling.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/maybe-dangling.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

For context, `MaybeDangling<T>`, is like `ManuallyDrop<T>`, insofar it does not carry
[aliasing/`dereferenceable`-ity properties](https://github.com/rust-lang/rfcs/pull/3336) w.r.t. the
`T` it contains. Notably, it means it is allowed to:
 1. have some expired value inside of them, such as `T = &'expired …`,
 1. be fed to a function that does not inspect its value (such as `::core::mem::forget()`),
 1. exhibit well-defined behavior (no UB!).

Moreover, contrary to `ManuallyDrop<T>`, `MaybeDangling<T>` does not forgo `T`'s drop glue.

Another way to look at it is to say that:

 1. there is `MaybeDangling<T>`, which is a core primitive type, equivalent to `T` in almost every aspect (notably, it carries its very same drop glue), except in its lack of `T`'s default _implicit_ `dereferenceable`-ity-and-lack-of-aliasing semantics (wherein `T`'s _validity_ invariants mandates that these semantics be active, and remain true, upon every time the type is so much as "looked at" (this includes "inert" assignments or calling functions on it, **even a no-op one such as `mem::forget()`**)).

    `MaybeDangling<T>` forgos and loosens this requirement, expecting "only" as a _safety_ invariant
    that the _safety_ (and thus, _validity_) invariants of its inner `T` be upheld upon _actual_ use
    of `T`'s API, _e.g._, upon `Deref{,Mut}` of this wrapper.

    **Notably**, this includes, **in a wildly dangerous, maybe even footgunny way, its drop glue**, at least whenever `mem::needs_drop::<T>()`.

 1. In order to fix this last remark, another type is offered by the stdlib, `ManuallyDrop<T>`, which not only features the very same _loose_ lack-of-`dereferenceable`-ity-and-lack-of-aliasing semantics of `MaybeDangling<T>`, but it also fully disables the inherent/built-in (implicit) drop glue of the overall type. If drop of its inner `T` is desired, it can be achieved by doing so _explicitly_, through [`ManuallyDrop::into_inner()`](https://doc.rust-lang.org/stable/core/mem/struct.ManuallyDrop.html#method.into_inner) or [`ManuallyDrop::drop`](https://doc.rust-lang.org/stable/core/mem/struct.ManuallyDrop.html#method.drop)`{,_in_place}()`.

See the docs of [`::core::mem::MaybeDangling`](https://doc.rust-lang.org/stable/core/mem/struct.MaybeDangling.html) for more info about all this.

Currently, whilst `MaybeDangling<T>` is indeed offered by the stdlib, it is gated behind an `unstable` feature. This crate offers a polyfill of this, achieved by:

 1. wrapping `ManuallyDrop<T>`
 1. manually implementing `Drop` on it to call `ManuallyDrop::drop()`.

This does come with one small potential ergonomic issue compared to the stdlib's, related to `dropck`.

Read the docs of this crate's `MaybeDangling<T>` for more info about this.

## References

  - **The RFC that shall eventually and ultimately supersede this very crate: <https://github.com/rust-lang/rfcs/pull/3336>**

  - The `miri` PR implementing the check against this: <https://github.com/rust-lang/miri/pull/2985>

  - The soundness problem of `::ouroboros` stemming from not using this: <https://github.com/joshua-maros/ouroboros/issues/88>

  - The soundness problem of `::yoke` stemming from not using this: <https://github.com/unicode-org/icu4x/issues/3696>

  - [An URLO thread on the topic, and a post exposing the intention to write this very crate](https://users.rust-lang.org/t/unsafe-code-review-semi-owning-weak-rwlock-t-guard/95706/15?u=yandros)
