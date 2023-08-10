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
/// Explanation:
///
/// <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
/// Below are three scenarios that should illustrate how Drop Check (`dropck`)
/// handles transitive drop glue when checking for dangling pointers.
/// All three scenarios are theoretically sound, since no dangling pointer is
/// ever accessed in any destructor.
/// But none will compile without the `nightly-dropck_eyepatch` feature enabled
/// to relax Drop Check into thinking that we won't accidentially access a
/// dangling pointer.
/// The `nightly-dropck_eyepatch` provides the flexibility to make the first
/// two cases compile.
///
/// #### What does it mean to have a "dangling `T`"?
///
/// Note that the terminology of a "dangling `T`" can be a bit hard to
/// visualize. The idea is to consider some `'dangling` lifetime (_e.g._,
/// some `&'dangling` borrow), to then imagine different type definitions
/// involving it.
///
/// For instance:
///
///  1. `T = &'dangling str`
///  2. `T = PrintOnDrop<&'dangling str>`,
///  3. `T = Box<&'dangling str>`,
///  4. `T = (u8, Box<PrintOnDrop<&'dangling str>>)`,
///  5. `T = &mut PrintOnDrop<&'dangling str>`,
///
/// The key idea is that there are three kind of types, here:
///   - the types with no drop glue at all, _i.e._, types for which
///     `mem::needs_drop::<T>()` returns `false`: `1.` and `5.`.
///
///     Such types should be allowed to go out of scope at a point
///     where their lifetime may be `'dangling`.
///
///   - the types with drop glue involving a dereference of the
///     `&'dangling` reference: `2.` and `4.`
///
///     Such types should never be allowed to go out of scope at a
///     point where their lifetime may be `'dangling`.
///
///   - the types with drop glue, but not involving a dereference of the
///     `&'dangling` reference: `3.`
///
///     Such types _can be allowed_ to go out of scope (and thus, run
///     their drop glue) at a point where their lifetime may be
///     `'dangling`.
///
/// Notice how a useful distinction thus revolves around the presence
/// of "drop glue" or lack thereof, to determine whether we are in the
/// first group, or the other two. On the other hand, whether a type
/// _directly_ `impl`ements `Drop`, such as `Box` or `PrintOnDrop`, or
/// does not (wrapper types containing it, such as `String` w.r.t the
/// `Drop impl` of `Vec<u8>`, or `(u8, Box<...>, )` in that `4.`th example),
/// is not enough information to distinguish between the two:
///   - `2.` and `3.` both `impl Drop`, and yet belong to different
///     categories,
///   - `4.` does not `impl Drop`, and yet belongs to the same
///     category as `2.`.
///
/// See the [`drop_bounds` lint](
/// https://doc.rust-lang.org/1.71.0/nightly-rustc/rustc_lint/traits/static.DROP_BOUNDS.html#explanation)
/// for more info.
///
/// The real distinction between the second and third groups is
/// whether the wrapper type, when dropped, _merely drops_ its
/// inner `T` (like `Box<T>` does), or if it unconditionally uses any
/// other API of the type, like `PrintOnDrop<T : Debug>` does.
///
/// With that context in mind, let's look at the three scenarios:
///
/// #### Scenario 1: `T` has no drop glue (_e.g._, `T = &'dangling str`)
///
/// With `#[may_dangle]` we are able to communicate to Drop Check that
/// `MaybeDangling` won't access the potentially dangling `T` (`&'dangling`)
/// in its destructor (_e.g._, the `str` behind `T = &'dangling str`),
/// *unless* `T`'s `'dangling` lifetime is involved in transitive drop glue, _i.e._:
///   - whenever `T` implements `Drop` (without `#[may_dangle]`);
///   - or whenever `T` transitively owns some field with drop glue involving `'dangling`.
///
/// Since `T` does not have drop glue (`mem::needs_drop::<T>()` returns `false`),
/// Drop Check will allow this to compile,
/// even though the reference stored in `Wrapper` is dangling when `v` gets
/// dropped:
///
/// ```
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use ::maybe_dangling::MaybeDangling;
///
/// struct Wrapper<'a>(&'a str);
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(Wrapper(&s));
///     drop(s); // <- causes the reference in `Wrapper` to dangle
/// } // <- `v` dropped here, despite containing a `&'dangling s` reference!
/// # }
/// ```
///
/// #### Scenario 2: `T` has drop glue known not to involve `'dangling`
///
/// Now that `T` has a destructor, it must be executed when `v` is dropped.
/// With `#[may_dangle]` we tell Drop Check that we don't access the inner
/// reference, so it is safe for it to dangle when the destructor is executed:
///
/// ```
/// # #![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use ::maybe_dangling::MaybeDangling;
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
/// #### Scenario 3: `T` has drop glue (potentially) involving `'dangling`
///
/// This scenario is the same as the previous one, but without `#[may_dangle]`.
/// Drop Check does not know about the internals of `T`'s destructor, so it
/// can't tell whether the inner `&'dangling` reference will be accessed.
/// Therefore, this will cause a compilation error, even with the
/// `nightly-dropck_eyepatch` feature enabled.
///
/// ```compile_fail
/// use ::maybe_dangling::MaybeDangling;
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
/// ```
///
/// </details>
///
/// #### Summary: when is a `MaybeDangling<...'dangling...>` allowed to go out of scope
///
/// Here is a summary of which of the scenarios shown above can be compiled, with
/// or without the `nightly-dropck_eyepatch` feature enabled:
///
/// | `MaybeDangling<T>`<br/><br/>`where T` | With `nightly-dropck_eyepatch` | Without `nightly-dropck_eyepatch` |
/// | --- | --- | --- |
/// | has no drop glue<br/>_e.g._<br/>`T=&'dangling str` | ✅ | ❌ |
/// | has drop glue but not involving `'dangling`<br/>_e.g._<br/>`T=Box<&'dangling str>` | ✅ | ❌ |
/// | has drop glue involving `'dangling`<br/>_e.g._<br/>`T=PrintOnDrop<&'dangling str>` | ❌ | ❌ |
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
/// [dropck-std]: https://doc.rust-lang.org/1.71.0/std/ops/trait.Drop.html#drop-check
/// [dropck-nomicon]: https://doc.rust-lang.org/1.71.0/nomicon/dropck.html
/// [dropck-generics]: https://doc.rust-lang.org/1.71.0/nomicon/phantom-data.html#generic-parameters-and-drop-checking
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
