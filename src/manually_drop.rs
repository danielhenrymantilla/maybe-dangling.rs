use ::core::{
    cmp::*,
    fmt::{self, Debug},
    hash::{self, Hash},
    mem::MaybeUninit as MU,
    ops::{Deref, DerefMut},
};

/// A wrapper to inhibit compiler from automatically calling `T`’s destructor.
/// This wrapper is 0-cost.
///
/// See [`::core::mem::ManuallyDrop`] for more info.
///
/// # Differences with [`::core::mem::ManuallyDrop`]
///
///   - **No niches**
///
///     The current implementation uses [`::core::mem::MaybeUninit`] to make
///     sure the aliasing and `dereferenceable`ity properties of the inner `T`
///     are properly disabled.
///
///     The main side-effect of this implementation is that it disables
///     niches, thereby preventing **discriminant elision**.
///
///     For instance, an <code>[Option]<[ManuallyDrop]<[bool]>></code> will
///     occupy _two_ bytes rather than _one_.
///
///   - It does not implement
///     [`Structural{Partial,}Eq`][::core::marker::StructuralEq].
///
///   - Note that once stdlib's own [`::core::mem::ManuallyDrop`] properly gets
///     its aliasing/`dereferenceable`ity properties removed, this crate shall
///     be updated to just reëxport it (using a `build.rs` to prevent MSRV
///     breakage).
///
///     This means that the _lack of discriminant elision_ cannot be relied upon
///     either!
///
///   - Other than that, this is a `#[repr(transparent)]` wrapper around `T`,
///     thereby having:
///       - equal [`Layout`][::core::alloc::Layout];
///       - equal calling-convention ABI[^1]
///
/// [^1]: this is assuming `MaybeUninit<T>` has the same ABI as `T`, as it
/// currently advertises, despite that probably being a bad idea for
/// a "bag of bytes" `T`-ish wrapper, since it means that padding bytes
/// inside of `T` won't be preserved when working with a
/// `MaybeUninit<T>`. So, if the stdlib were to break the current
/// ABI promise of `MaybeUninit` to cater to that problem, then this crate would
/// probably do so well, unless the `maybe_dangling` changes were to make it to
/// the stdlib first.
#[derive(Copy)]
#[repr(transparent)]
pub struct ManuallyDrop<T> {
    /// Until stdlib guarantees `MaybeDangling` semantics for its `ManuallyDrop`,
    /// we have to polyfill it ourselves using `MaybeUninit`, the only type
    /// known to date to feature such semantics.
    ///
    /// So doing, quite unfortunately, disables niche optimizations.
    ///
    /// # SAFETY INVARIANT: the value must always be init `MU`-wise.
    value: MU<T>,
}

// SAFETY: as per the safety invariant above.
#[allow(unsafe_code)]
impl<T> ManuallyDrop<T> {
    /// Wrap a value to be manually dropped.
    ///
    /// See [`::core::mem::ManuallyDrop::new()`] for more info.
    #[inline]
    pub const fn new(value: T) -> ManuallyDrop<T> {
        Self {
            value: MU::new(value),
        }
    }

    /// Extracts the value from the `ManuallyDrop` container.
    ///
    /// See [`::core::mem::ManuallyDrop::into_inner()`] for more info.
    #[inline]
    pub const fn into_inner(slot: ManuallyDrop<T>) -> T {
        unsafe { MU::assume_init(slot.value) }
    }

    /// Takes the value from the `ManuallyDrop<T>` container out.
    ///
    /// See [`::core::mem::ManuallyDrop::take()`] for more info.
    #[must_use = "if you don't need the value, you can use `ManuallyDrop::drop` instead"]
    #[inline]
    pub unsafe fn take(slot: &mut ManuallyDrop<T>) -> T {
        unsafe { slot.value.as_ptr().read() }
    }

    /// Manually drops the contained value.
    ///
    /// See [`::core::mem::ManuallyDrop::drop()`] for more info.
    #[inline]
    pub unsafe fn drop(slot: &mut ManuallyDrop<T>) {
        unsafe { slot.value.as_mut_ptr().drop_in_place() }
    }
}

// Safety: as per the invariant mentioned above.
#[allow(unsafe_code)]
impl<T> DerefMut for ManuallyDrop<T> {
    /// See [`::core::mem::ManuallyDrop::deref_mut()`] for more info.
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        impl<T> Deref for ManuallyDrop<T> {
            type Target = T;

            #[inline]
            /// See [`::core::mem::ManuallyDrop::deref()`] for more info.
            fn deref(self: &Self) -> &T {
                unsafe { self.value.assume_init_ref() }
            }
        }

        unsafe { self.value.assume_init_mut() }
    }
}

impl<T: Default> Default for ManuallyDrop<T> {
    /// See [`::core::mem::ManuallyDrop::default()`] for more info.
    #[inline]
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone> Clone for ManuallyDrop<T> {
    /// See [`::core::mem::ManuallyDrop::clone()`] for more info.
    fn clone(self: &Self) -> Self {
        Self::new(T::clone(self))
    }

    JustDerefTM! {
        fn clone_from(self: &mut Self, source: &Self);
    }
}

JustDerefTM! {
    impl<T: Debug> Debug for ManuallyDrop<T> {
        fn fmt(self: &Self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    }

    impl<T: Hash> Hash for ManuallyDrop<T> {
        fn hash<__H: hash::Hasher>(self: &Self, state: &mut __H);
    }

    impl<T: Ord> Ord for ManuallyDrop<T> {
        fn cmp(self: &Self, other: &Self) -> Ordering;
    }

    impl<T: PartialOrd> PartialOrd for ManuallyDrop<T> {
        fn partial_cmp(self: &Self, other: &Self) -> Option<Ordering>;
    }

    impl<T: PartialEq> PartialEq for ManuallyDrop<T> {
        fn eq(self: &Self, other: &Self) -> bool;
    }

    impl<T: Eq> Eq for ManuallyDrop<T> {}
}

macro_rules! JustDerefTM {
    (
        $(
            $(#$attr:tt)*
            $($(@$if_unsafe:tt)?
                unsafe
            )?
            impl<T $(: $Bound:path)?>
                $($($Trait:ident)::+ for)?
                ManuallyDrop<T>
            {
                $($inner:tt)*
            }
        )*
    ) => (
        $(
            $(#$attr)*
            $($($if_unsafe)?
                unsafe
            )?
            impl<T $(: $Bound)?>
                $($($Trait)::+ for)?
                ManuallyDrop<T>
            {
                JustDerefTM! {
                    $($inner)*
                }
            }
        )*
        $(
            $(#$attr)*
            $($($if_unsafe)?
                unsafe
            )?
            impl<T $(: $Bound)?>
                $($($Trait)::+ for)?
                crate::MaybeDangling<T>
            {
                JustDerefTM! {
                    $($inner)*
                }
            }
        )*
    );

    (
        $(
            $(#$attr:tt)*
            $pub:vis
            fn $fname:ident
                $(<$H:ident $(: $Bound:path)?>)?
            (
                $(
                    $arg_name:ident : $ArgTy:ty
                ),* $(,)?
            ) $(-> $Ret:ty)? ;
        )*
    ) => (
        $(
            #[inline]
            $(#$attr)*
            #[doc = concat!(
                "\nSee [`::core::mem::ManuallyDrop::", stringify!($fname), "()`] for more info."
            )]
            $pub
            fn $fname
                $(<$H $(: $Bound)?>)?
            (
                $(
                    $arg_name : $ArgTy
                ),*
            ) $(-> $Ret)?
            {
                T::$fname($($arg_name),*)
            }
        )*
    );
}
use JustDerefTM;
