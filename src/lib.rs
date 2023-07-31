#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_code)]
#![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]

pub use self::maybe_dangling::MaybeDangling;
mod maybe_dangling;

pub use manually_drop::ManuallyDrop;
mod manually_drop;

#[rustfmt::skip]
/// I really don't get the complexity of `cfg_if!`…
macro_rules! cfg_match {
    (
        _ => { $($expansion:tt)* } $(,)?
    ) => (
        $($expansion)*
    );

    (
        $cfg:meta => $expansion:tt $(,
        $($($rest:tt)+)? )?
    ) => (
        #[cfg($cfg)]
        crate::cfg_match! { _ => $expansion } $($(

        #[cfg(not($cfg))]
        crate::cfg_match! { $($rest)+ } )?)?
    );

    // Bonus: expression-friendly syntax: `cfg_match!({ … })`
    ({
        $($input:tt)*
    }) => ({
        crate::cfg_match! { $($input)* }
    });
}
use cfg_match;
