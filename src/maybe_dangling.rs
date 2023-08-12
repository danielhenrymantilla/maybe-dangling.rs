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
/// [eponymous `rustc` feature][RFC-1327] so as to get the `Drop` implementation
/// amended accordingly.
///
/// Explanation:
///
/// <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
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
/// The key idea is that there are three categories of types here:
///
///   1. The types with no drop glue at all, _i.e._, types for which
///      `mem::needs_drop::<T>()` returns `false`: types 1. and 5.
///
///      Such types _should be allowed_ to go out of scope at a point
///      where their lifetime may be `'dangling`.
///
///   2. The types with drop glue known not to involve a dereference of
///      the `&'dangling` reference: type 3.
///
///      Such types _can be allowed_ to go out of scope (and thus, run
///      their drop glue) at a point where their lifetime may be
///      `'dangling`.
///
///   3. The types with drop glue (potentially) involving a dereference
///      of the `&'dangling` reference: types 2. and 4.
///
///      Such types _should never be allowed_ to go out of scope at a
///      point where their lifetime may be `'dangling`.
///
/// Notice how a useful distinction thus revolves around the presence
/// of drop glue or lack thereof, to determine whether we are in the
/// first category, or the other two. On the other hand, whether a type
/// _directly_ implements `Drop`, such as `Box` or `PrintOnDrop`, or
/// does not (wrapper types containing it, such as `String` w.r.t. the
/// `Drop` implementation of the underlying `Vec<u8>`, or `(u8, Box<...>)`
/// in the fourth example type above), is not enough information to
/// distinguish between the two, as
///
///   - types 2. and 3. both implement `Drop`, and yet belong to different
///     categories,
///
///   - type 4. does not implement `Drop`, and yet belongs to the same
///     category as type 2.
///
/// See the [`drop_bounds` lint] for more info.
///
/// The distinction between the second and third category is whether a generic
/// type, when dropped,
///
/// 1. merely drops its inner `T` (like `Box<T>` does) and
///
/// 2. makes it known to the [drop checker] that it does so.
///
/// If a type violates either restriction, either by unconditionally using any
/// other API of `T`, like `PrintOnDrop<T: Debug>` does, or by not making
/// it known to the drop checker that it merely drops its inner `T`, it will
/// belong to category 3, which can't be allowed to compile.
///
/// Making it known to the drop checker that `T` is merely dropped requires
/// the unstable [`#[may_dangle]`][RFC-1327] attribute.
/// The drop checker does not know the implementation details of any
/// `Drop` implementation.
/// It can't statically analyse how `T` is used in the destructor.
/// Instead, drop check requires every generic argument to strictly
/// outlive the wrapper type to guarantee soundness.
/// This can be overly restrictive when merely dropping `T`, making it
/// impossible to have `Drop` implementations where `T` might be dangling,
/// even if dropping a dangling `T` would be sound in the given context.
/// Hence the `#[may_dangle]` attribute is required to manually and _unsafely_
/// tell drop check that `T` is merely dropped in the generic type's
/// destructor, relaxing the drop checker in situations where its soundness
/// requirements are overly restrictive.
/// With the `nightly-dropck_eyepatch` feature enabled, <code>[MaybeDangling]\<T\></code>
/// uses `#[may_dangle]` under the hood to let drop check know that it won't
/// access the potentially dangling `T` (_e.g._, the `str` behind
/// `T = &'dangling str`) in its destructor, [*unless*][dropck-generics] `T`'s
/// `'dangling` lifetime is involved in transitive drop glue, _i.e._:
///   - whenever `T` implements `Drop` (without `#[may_dangle]`);
///   - or whenever `T` transitively owns some field with drop glue involving
///     `'dangling`.
///
/// With that context in mind, let's look at examples for the three categories:
///
/// #### Category 1: `T` has no drop glue (_e.g._, `T = &'dangling str`)
///
/// Since `T` does not have drop glue (`mem::needs_drop::<T>()` returns `false`),
/// the drop checker will allow this to compile, even though the reference
/// dangles when `v` gets dropped:
///
/// ```
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use ::maybe_dangling::MaybeDangling;
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(&s);
///     drop(s); // <- makes `&s` dangle
/// } // <- `v` dropped here, despite containing a `&'dangling s` reference!
/// # }
/// ```
///
/// #### Category 2: `T` has drop glue known not to involve `'dangling` (_e.g._, `T = Box<&'dangling str>`)
///
/// Now that `T` is has drop glue, it must be executed when `v` is dropped.
/// `Box<&'dangling str>`'s `Drop` implementation is known not to involve
/// `'dangling`, so it is safe for `&'dangling str` to dangle when the `Box`
/// is dropped:
///
/// ```
/// # #![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// use ::maybe_dangling::MaybeDangling;
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(Box::new(&s));
///     drop(s); // <- makes `&s` dangle
/// } // <- `v`, and thus `Box(&s)` dropped here
/// # }
/// ```
///
/// #### Category 3: `T` has drop glue (potentially) involving `'dangling` (_e.g._, `T = PrintOnDrop<&'dangling str>`)
///
/// Like the second category, `T` now has drop glue.
/// But unlike category 2., `T` now has drop glue either involving `'dangling`
/// or not informing the drop checker that `'dangling` is unused.
/// Let's look at an example where `'dangling` is involved in drop glue:
///
/// ```compile_fail
/// use ::maybe_dangling::MaybeDangling;
///
/// use ::std::fmt::Debug;
///
/// struct PrintOnDrop<T: Debug>(T);
///
/// impl<T: Debug> Drop for PrintOnDrop<T> {
///     fn drop(&mut self) {
///          println!("Using the potentially dangling `T` in our destructor: {:?}", self.0);
///     }
/// }
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(PrintOnDrop(&s));
///     drop(s); // <- makes `&s` dangle
/// } // <- `v`, and thus `PrintOnDrop(&s)` dropped here, causing a use-after-free ! ⚠️
/// ```
///
/// The example above should never be allowed to compile as `PrintOnDrop`
/// will dereference `&'dangling str`, which points to a `str` that already
/// got dropped and invalidated, causing undefined behavior.
///
/// An example for a type where `'dangling` is not involved in any drop glue
/// but does not relax the drop checker with `#[may_dangle]` would be:
///
/// ```compile_fail
/// use ::maybe_dangling::MaybeDangling;
///
/// struct MerelyDrop<T>(T);
///
/// impl<T> Drop for MerelyDrop<T> {
///     fn drop(&mut self) {
///          println!("Not using the potentially dangling `T` in our destructor");
///     }
/// }
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(MerelyDrop(&s));
///     drop(s); // <- makes `&s` dangle
/// } // <- `v`, and thus `MerelyDrop(&s)` dropped here
/// ```
///
/// To amend the example above and move from category 3. to category 2. and
/// make it compile, `#[may_dangle]` can be applied to `T` in `MerelyDrop`'s
/// `Drop` implementation:
///
/// ```
/// # #![cfg_attr(feature = "nightly-dropck_eyepatch", feature(dropck_eyepatch))]
/// # #[cfg(feature = "nightly-dropck_eyepatch")]
/// # {
/// #![feature(dropck_eyepatch)]
///
/// use ::maybe_dangling::MaybeDangling;
///
/// struct MerelyDrop<T>(T);
///
/// unsafe impl<#[may_dangle] T> Drop for MerelyDrop<T> {
///     fn drop(&mut self) {
///          println!("Not using the potentially dangling `T` in our destructor");
///     }
/// }
///
/// fn main() {
///     let s: String = "I will dangle".into();
///     let v = MaybeDangling::new(MerelyDrop(&s));
///     drop(s); // <- makes `&s` dangle
/// } // <- `v`, and thus `MerelyDrop(&s)` dropped here
/// # }
/// ```
///
/// Note that the `Drop` implementation is _unsafe_ now, as we are still free
/// to use the dangling `T` in the destructor.
/// We only pinky-swear to the drop checker that we won't.
///
/// </details>
///
/// #### Summary: when is a `MaybeDangling<...'dangling...>` allowed to go out of scope
///
/// This table summarises which of the categories shown above can be compiled, with
/// or without the `nightly-dropck_eyepatch` feature enabled:
///
/// | `MaybeDangling<T>`<br/><br/>`where T` | With `nightly-dropck_eyepatch` | Without `nightly-dropck_eyepatch` |
/// | --- | --- | --- |
/// | has no drop glue<br/>_e.g._<br/>`T = &'dangling str` | ✅ | ❌ |
/// | has drop glue known not to involve `'dangling`<br/>_e.g._<br/>`T = Box<&'dangling str>` | ✅ | ❌ |
/// | has drop glue (potentially) involving `'dangling`<br/>_e.g._<br/>`T = PrintOnDrop<&'dangling str>` | ❌ | ❌ |
///
/// [RFC-1327]: https://rust-lang.github.io/rfcs/1327-dropck-param-eyepatch.html
/// [`drop_bounds` lint]: https://doc.rust-lang.org/1.71.0/nightly-rustc/rustc_lint/traits/static.DROP_BOUNDS.html#explanation
/// [drop checker]: https://doc.rust-lang.org/1.71.0/nomicon/dropck.html
/// [dropck-generics]: https://doc.rust-lang.org/1.71.0/nomicon/phantom-data.html#generic-parameters-and-drop-checking
pub struct MaybeDangling<T> {
    value: ManuallyDrop<T>,
    #[cfg(feature = "nightly-dropck_eyepatch")]
    #[allow(nonstandard_style)]
    // disables `#[may_dangle]` for `T` invovled in transitive drop glue
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
