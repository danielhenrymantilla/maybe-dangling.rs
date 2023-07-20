use {
    ::core::{
        mem::{ManuallyDrop as StdMD},
    },
    crate::{
        ManuallyDrop,
    },
};

/// Like [`crate::ManuallyDrop`] but for having `drop` glue.
/// This wrapper is 0-cost.
///
/// In other words, a <code>[MaybeDangling]\<T\></code> is just like `T`, but
/// for having been stripped of aliasing/`dereferenceable`-ity properties.
///
/// Its usage should be quite rare and advanced: if you are intending to keep
/// hold of a potentially dangling / exhausted value, chances are you won't
/// want implicit/automatic drop glue of it without having previously checked
/// for lack of exhaustion ⚠️.
///
/// That is, it is strongly advisable to be using
/// <code>[crate::ManuallyDrop]\<T\></code> instead!
///
/// ### Opting into `#[may_dangle]` and the `dropck_eyepatch`
///
/// Ironically, for this drop glue to be as smooth as it should be, the unstable
/// `#[may_dangle]` feature is needed.
///
/// But by virtue of being unstable, it cannot be offered by this crate on
/// stable Rust.
///
/// For the adventurous `nightly` users, you can enable the
/// `nightly-dropck_eyepatch` Cargo feature to opt into the usage of the
/// eponymous `rustc` feature so as to get the `Drop` impl amended accordingly.
pub struct MaybeDangling<T> {
    value: ManuallyDrop<T>,
    #[cfg(feature = "nightly-dropck_eyepatch")]
    #[allow(nonstandard_style)]
    _owns_T: ::core::marker::PhantomData<T>,
}

impl<T> MaybeDangling<T> {
    pub const fn new(value: T) -> MaybeDangling<T> {
        Self {
            value: ManuallyDrop::new(value),
            #[cfg(feature = "nightly-dropck_eyepatch")]
            _owns_T: ::core::marker::PhantomData,
        }
    }

    /// Extracts the value from the `MaybeDangling` container.
    ///
    /// See [`::core::mem::ManuallyDrop::into_inner()`] for more info.
    #[inline]
    pub fn into_inner(slot: MaybeDangling<T>) -> T {
        #![allow(unsafe_code)]
        // Safety: this is the defuse inherent drop glue pattern.
        unsafe {
            ManuallyDrop::take(&mut StdMD::new(slot).value)
        }
    }
}

// The main difference with `ManuallyDrop`: automatic drop glue!
crate::cfg_match! {
    feature = "nightly-dropck_eyepatch" => {
        #[allow(unsafe_code)]
        unsafe impl<#[may_dangle] T> Drop for MaybeDangling<T> {
            fn drop(&mut self) {
                unsafe {
                    ManuallyDrop::drop(&mut self.value)
                }
            }
        }
    },

    _ => {
        impl<T> Drop for MaybeDangling<T> {
            fn drop(&mut self) {
                #![allow(unsafe_code)]
                unsafe {
                    ManuallyDrop::drop(&mut self.value)
                }
            }
        }
    },
}


impl<T> ::core::ops::DerefMut for MaybeDangling<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        impl<T> ::core::ops::Deref for MaybeDangling<T> {
            type Target = T;

            #[inline]
            fn deref(self: &Self) -> &T {
                let Self { value, .. } = self;
                value
            }
        }

        let Self { value, .. } = self;
        value
    }
}

impl<T: Default> Default for MaybeDangling<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Clone for MaybeDangling<T> {
    fn clone(self: &Self) -> Self {
        Self::new(T::clone(self))
    }

    fn clone_from(self: &mut Self, source: &Self) {
        T::clone_from(self, source)
    }
}
