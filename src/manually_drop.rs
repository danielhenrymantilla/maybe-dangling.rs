::core::cfg_select! {
    not(feature = "acknowledge-unnecessary-manually-drop") => {
        /// Ever since Rust 1.96.0, and this library's version `0.1.3…`, [`ManuallyDrop`] is a mere
        /// re-export of [`::core::mem::ManuallyDrop`], as the latter has started featuring the desired
        /// [`crate::MaybeDangling`] semantics inside of it.
        ///
        /// See, for instance, <https://github.com/rust-lang/rust/pull/155750>.
        ///
        /// ### Deprecated
        ///
        /// As a nudging feature, usage of this type is marked deprecated unless you acknowledge
        /// this change by enabling the `"acknowledge-unnecessary-manually-drop"` Cargo feature.
        ///
        /// This is something you may wish to do if you intend to support Rust across old and newer
        /// MSRVs (although it is rather advisable to `#[allow(deprecated)]` usages of it).
        #[deprecated(
            since = "0.1.3",
            note = "you do not need to use this type anymore, \
                as you can directly use the one from the stdlib."
        )]
        pub type ManuallyDrop<T> = ::core::mem::ManuallyDrop<T>;
    },

    _ => {
        #[doc(no_inline)]
        pub use ::core::mem::ManuallyDrop;
    },
}
