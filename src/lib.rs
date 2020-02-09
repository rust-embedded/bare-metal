//! Abstractions common to bare metal systems

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::cell::UnsafeCell;
use core::marker::PhantomData;

/// Critical section token
///
/// Indicates that you are executing code within a critical section
#[derive(Clone, Copy)]
pub struct CriticalSection<'cs> {
    _0: PhantomData<&'cs ()>,
}

impl<'cs> CriticalSection<'cs> {
    /// Creates a critical section token
    ///
    /// This method is meant to be used to create safe abstractions rather than
    /// meant to be directly used in applications.
    #[inline(always)]
    pub unsafe fn new() -> Self {
        CriticalSection { _0: PhantomData }
    }
}

/// A "mutex" based on critical sections
///
/// # Safety
///
/// **This Mutex is only safe on single-core systems.**
///
/// On multi-core systems, a `CriticalSection` **is not sufficient** to ensure exclusive access.
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> Mutex<T> {
    /// Borrows the data for the duration of the critical section
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
const GH_6: () = ();

/// Interrupt number
pub unsafe trait Nr {
    /// Returns the number associated with an interrupt
    fn nr(&self) -> u8;
}
