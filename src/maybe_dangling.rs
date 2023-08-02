use crate::ManuallyDrop;
use ::core::mem::ManuallyDrop as StdMD;

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
/// ### Opting into unstable `#[may_dangle]` and the `dropck_eyepatch`
///
/// Ironically, for this drop glue to be as smooth as it should be, the unstable
/// `#[may_dangle]` feature is needed.
///
/// But by virtue of being unstable, it cannot be offered by this crate on
/// stable Rust.
///
/// For the adventurous `nightly` users, you can enable the
/// `nightly-dropck_eyepatch` Cargo feature to opt into the usage of the
/// [eponymous `rustc` feature][RFC-1327] so as to get the `Drop` impl amended
/// accordingly.
///
/// Below are three scenarios that should illustrate how Drop Check (`dropck`)
/// handles transitive drop glue when checking for dangling pointers.
/// All three scenarios are theoretically sound, since no dangling pointer is
/// ever accessed in any destructor.
/// But none will compile without the `nightly-dropck_eyepatch` feature there
/// to relax Drop Check into thinking that we won't accidentially access a
/// dangling pointer.
/// The `nightly-dropck_eyepatch` provides the flexibility to make the first
/// two cases compile.
///
/// #### Scenario 1: `T` has no destructor
///
/// With `#[may_dangle]` we are able to communicate to Drop Check that
/// `MaybeDangling` won't access the potentially dangling `T` in its destructor,
/// *unless* `T` is invovled in transitive drop glue, i.e. `T` implements `Drop`
/// itself.
/// Since `T` does not implement `Drop`, Drop Check will allow this to compile,
/// even though the reference stored in `Wrapper` is dangling when `v` gets
/// dropped:
///
/// ```
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use maybe_dangling::MaybeDangling;
///
/// struct Wrapper<'a>(&'a str);
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(Wrapper(&s));
///     drop(s); // <- causes the reference in `Wrapper` to dangle
/// } // <- `v` dropped here
/// # }
/// ```
///
/// #### Scenario 2: `T` has a destructor, but uses `#[may_dangle]`
///
/// Now that `T` has a destructor, it must be executed when `v` is dropped.
/// With `#[may_dangle]` we tell Drop Check that we don't access the inner
/// reference, so it is safe for it to dangle when the destructor is executed:
///
/// ```
/// # #![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use maybe_dangling::MaybeDangling;
///
/// struct Wrapper<'a>(&'a str);
///
/// // we pinky-swear not to access the potentially dangling pointer, so `dropck`
/// // will let us compile this snippet
/// unsafe impl<#[may_dangle] 'a> Drop for Wrapper<'a> {
///     fn drop(&mut self) { }
/// }
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(Wrapper(&s));
///     drop(s); // <- causes the reference in `Wrapper` to dangle
/// } // <- `v` dropped here
/// # }
/// ```
///
/// #### Scenario 3: `T` has a destructor and does not use `#[may_dangle]`
///
/// This scenario is the same as the previous one, but without `#[maybe_dangle]`.
/// Drop Check does not know about the internals of `T`'s destructor, so it
/// can't tell whether the inner reference will be accessed.
/// Therefore, this will cause a compilation error, even with the
/// `nightly-dropck_eyepatch` feature enabled.
///
/// ```compile_fail
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use maybe_dangling::MaybeDangling;
///
/// struct Wrapper<'a>(&'a str);
///
/// // `dropck` will not know we don't access the dangling pointer and won't let
/// // us compile this snippet
/// impl<'a> Drop for Wrapper<'a> {
///     fn drop(&mut self) { }
/// }
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(Wrapper(&s));
///     drop(s); // <- causes the reference in `Wrapper` to dangle
/// } // <- `v` dropped here
/// # }
/// # #[cfg(not(feature = "nightly-dropck_eyepatch"))]
/// # {
/// #     // to make the snippet not compile when `nightly-dropck_eyepatch`
/// #     // feature is not enabled
/// #     fn main() { 0 }
/// # }
/// ```
///
/// Here a summary of which of the scenarios shown above can be compiled, with
/// or without the `nightly-dropck_eyepatch` feature enabled:
///
/// | `T` | With `nightly-dropck_eyepatch` | Without `nightly-dropck_eyepatch` |
/// | --- | --- | --- |
/// | Without destructor | ✅ | ❌ |
/// | With destructor and with `#[maybe_dangle]` | ✅ | ❌ |
/// | With destructor and without `#[maybe_dangle]` | ❌ | ❌ |
///
/// ### See also
///
/// For further information on how automatic drop glue works, see the
/// "Drop Check" sections of the [`Drop` trait][dropck-std] and the
/// [Rustonomicon][dropck-nomicon] and the section on how
/// [drop-checking works with generic parameters][dropck-generics], also found
/// in the Rustonomicon.
///
/// [RFC-1327]: https://rust-lang.github.io/rfcs/1327-dropck-param-eyepatch.html
/// [dropck-std]: https://doc.rust-lang.org/std/ops/trait.Drop.html#drop-check
/// [dropck-nomicon]: https://doc.rust-lang.org/nomicon/dropck.html
/// [dropck-generics]: https://doc.rust-lang.org/nomicon/phantom-data.html#generic-parameters-and-drop-checking
pub struct MaybeDangling<T> {
    value: ManuallyDrop<T>,
    #[cfg(feature = "nightly-dropck_eyepatch")]
    #[allow(nonstandard_style)]
    // disables `#[may_dangle]` for `T` with a destructor
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
        unsafe { ManuallyDrop::take(&mut StdMD::new(slot).value) }
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
