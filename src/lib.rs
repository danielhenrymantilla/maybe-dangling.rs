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
macro_rules! match_cfg {
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
        crate::match_cfg! { _ => $expansion } $($(

        #[cfg(not($cfg))]
        crate::match_cfg! { $($rest)+ } )?)?
    );

    // Bonus: expression-friendly syntax: `match_cfg!({ … })`
    ({
        $($input:tt)*
    }) => ({
        crate::match_cfg! { $($input)* }
    });
}
use match_cfg;
