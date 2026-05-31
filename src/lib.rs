#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_code)]
#![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]

pub use self::maybe_dangling::MaybeDangling;
mod maybe_dangling;

#[doc(hidden)]
/** Not part of the public API */
pub mod ඞ {
    pub use ::core;
}
