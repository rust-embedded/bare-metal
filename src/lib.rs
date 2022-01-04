//! Abstractions common to bare metal systems.

#![deny(missing_docs)]
#![no_std]
#![doc(html_root_url = "https://docs.rs/bare-metal/1.0")]

use core::cell::{Ref, RefCell, RefMut, UnsafeCell};
use core::marker::PhantomData;

/// Critical section token.
///
/// An instance of this type indicates that the current core is executing code within a critical
/// section. This means that no interrupts must be enabled that could preempt the currently running
/// code.
#[derive(Clone, Copy, Debug)]
pub struct CriticalSection<'cs> {
    _0: PhantomData<&'cs ()>,
}

impl<'cs> CriticalSection<'cs> {
    /// Creates a critical section token.
    ///
    /// This method is meant to be used to create safe abstractions rather than being directly used
    /// in applications.
    ///
    /// # Safety
    ///
    /// This must only be called when the current core is in a critical section. The caller must
    /// ensure that the returned instance will not live beyond the end of the critical section.
    /// Moreover, the caller must use adequate fences to prevent the compiler from moving the
    /// instructions inside the critical section to the outside of it. Sequentially consistent fences are
    /// suggested immediately after entry and immediately before exit from the critical section.
    ///
    /// Note that the lifetime `'cs` of the returned instance is unconstrained. User code must not
    /// be able to influence the lifetime picked for this type, since that might cause it to be
    /// inferred to `'static`.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        CriticalSection { _0: PhantomData }
    }
}

/// A "mutex" based on critical sections.
///
/// # Safety
///
/// **This Mutex is only safe on single-core systems.**
///
/// On multi-core systems, a `CriticalSection` **is not sufficient** to ensure exclusive access.
///
/// # Design
///
/// [`std::sync::Mutex`] has two purposes. It converts types that are [`Send`]
/// but not [`Sync`] into types that are both; and it provides
/// [interior mutability]. `bare_metal::Mutex`, on the other hand, only adds
/// `Sync`. It does *not* provide interior mutability.
///
/// This was a conscious design choice. It is possible to create multiple
/// [`CriticalSection`] tokens, either by nesting critical sections or `Copy`ing
/// an existing token. As a result, it would not be sound for [`Mutex::borrow`]
/// to return `&mut T`, because there would be nothing to prevent calling
/// `borrow` multiple times to create aliased `&mut T` references.
///
/// The solution is to include a runtime check to ensure that each resource is
/// borrowed only once. This is what `std::sync::Mutex` does. However, this is
/// a runtime cost that may not be required in all circumstances. For instance,
/// `Mutex<Cell<T>>` never needs to create `&mut T` or equivalent.
///
/// If `&mut T` is needed, the simplest solution is to use `Mutex<RefCell<T>>`,
/// which is the closest analogy to `std::sync::Mutex`. [`RefCell`] inserts the
/// exact runtime check necessary to guarantee that the `&mut T` reference is
/// unique.
///
/// To reduce verbosity when using `Mutex<RefCell<T>>`, we reimplement some of
/// `RefCell`'s methods on it directly.
///
/// ```
/// # use bare_metal::{CriticalSection, Mutex};
/// # use std::cell::RefCell;
///
/// static FOO: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(42));
///
/// fn main() {
///     let cs = unsafe { CriticalSection::new() };
///     // Instead of calling this
///     let _ = FOO.borrow(cs).take();
///     // Call this
///     let _ = FOO.take(cs);
///     // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
///     // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
///     let _: &mut i32 = &mut *FOO.borrow_ref_mut(cs);
/// }
/// ```
///
/// [`std::sync::Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
/// [interior mutability]: https://doc.rust-lang.org/reference/interior-mutability.html
#[derive(Debug)]
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    #[inline]
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value),
        }
    }

    /// Gets a mutable reference to the contained value when the mutex is already uniquely borrowed.
    ///
    /// This does not require locking or a critical section since it takes `&mut self`, which
    /// guarantees unique ownership already. Care must be taken when using this method to
    /// **unsafely** access `static mut` variables, appropriate fences must be used to prevent
    /// unwanted optimizations.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    /// Unwraps the contained value, consuming the mutex.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Borrows the data for the duration of the critical section.
    #[inline]
    pub fn borrow<'cs>(&'cs self, _cs: CriticalSection<'cs>) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

impl<T> Mutex<RefCell<T>> {
    /// Borrow the data and call [`RefCell::replace`]
    ///
    /// This is equivalent to `self.borrow(cs).replace(t)`
    #[inline]
    pub fn replace<'cs>(&'cs self, cs: CriticalSection<'cs>, t: T) -> T {
        self.borrow(cs).replace(t)
    }

    /// Borrow the data and call [`RefCell::replace_with`]
    ///
    /// This is equivalent to `self.borrow(cs).replace_with(f)`
    #[inline]
    pub fn replace_with<'cs, F>(&'cs self, cs: CriticalSection<'cs>, f: F) -> T
    where
        F: FnOnce(&mut T) -> T,
    {
        self.borrow(cs).replace_with(f)
    }

    /// Borrow the data and call [`RefCell::borrow`]
    ///
    /// This is equivalent to `self.borrow(cs).borrow()`
    #[inline]
    pub fn borrow_ref<'cs>(&'cs self, cs: CriticalSection<'cs>) -> Ref<'cs, T> {
        self.borrow(cs).borrow()
    }

    /// Borrow the data and call [`RefCell::borrow_mut`]
    ///
    /// This is equivalent to `self.borrow(cs).borrow_mut()`
    #[inline]
    pub fn borrow_ref_mut<'cs>(&'cs self, cs: CriticalSection<'cs>) -> RefMut<'cs, T> {
        self.borrow(cs).borrow_mut()
    }
}

impl<T: Default> Mutex<RefCell<T>> {
    /// Borrow the data and call [`RefCell::take`]
    ///
    /// This is equivalent to `self.borrow(cs).take()`
    #[inline]
    pub fn take<'cs>(&'cs self, cs: CriticalSection<'cs>) -> T {
        self.borrow(cs).take()
    }
}

// NOTE A `Mutex` can be used as a channel so the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. access tokens) across different
// execution contexts (e.g. interrupts)
unsafe impl<T> Sync for Mutex<T> where T: Send {}

/// ``` compile_fail
/// fn bad(cs: bare_metal::CriticalSection) -> &u32 {
///     let x = bare_metal::Mutex::new(42u32);
///     x.borrow(cs)
/// }
/// ```
#[allow(dead_code)]
#[doc(hidden)]
const GH_6: () = ();
