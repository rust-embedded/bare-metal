//! Abstractions common to bare metal systems.

#![deny(missing_docs)]
#![no_std]
#![doc(html_root_url="https://docs.rs/bare-metal/1.0")]

use core::cell::UnsafeCell;
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
#[derive(Debug)]
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex.
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
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    /// Unwraps the contained value, consuming the mutex.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Borrows the data for the duration of the critical section.
    pub fn borrow<'cs>(&'cs self, _cs: CriticalSection<'cs>) -> &'cs T {
        unsafe { &*self.inner.get() }
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
