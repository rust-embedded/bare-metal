//! Abstractions common to bare metal systems

#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(feature = "const-fn", feature(const_fn, const_unsafe_cell_new))]
#![no_std]

use core::cell::UnsafeCell;

/// A peripheral
#[derive(Debug)]
pub struct Peripheral<T>
where
    T: 'static,
{
    address: *mut T,
}

impl<T> Peripheral<T> {
    /// Creates a new peripheral
    ///
    /// `address` is the base address of the register block
    #[cfg(feature = "const-fn")]
    pub const unsafe fn new(address: usize) -> Self {
        Peripheral {
            address: address as *mut T,
        }
    }

    /// Creates a new peripheral
    ///
    /// `address` is the base address of the register block
    #[cfg(not(feature = "const-fn"))]
    pub unsafe fn new(address: usize) -> Self {
        Peripheral {
            address: address as *mut T,
        }
    }

    /// Borrows the peripheral for the duration of a critical section
    pub fn borrow<'cs>(&self, _ctxt: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.get() }
    }

    /// Returns a pointer to the register block
    pub fn get(&self) -> *mut T {
        self.address as *mut T
    }
}

/// Critical section token
///
/// Indicates that you are executing code within a critical section
pub struct CriticalSection {
    _0: (),
}

impl CriticalSection {
    /// Creates a critical section token
    ///
    /// This method is meant to be used to create safe abstractions rather than
    /// meant to be directly used in applications.
    pub unsafe fn new() -> Self {
        CriticalSection { _0: () }
    }
}

/// A "mutex" based on critical sections
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Creates a new mutex
    #[cfg(feature = "const-fn")]
    pub const fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value),
        }
    }

    /// Creates a new mutex
    #[cfg(not(feature = "const-fn"))]
    pub fn new(value: T) -> Self {
        Mutex {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> Mutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&self, _cs: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

/// Interrupt number
pub unsafe trait Nr {
    /// Returns the number associated with an interrupt
    fn nr(&self) -> u8;
}

// NOTE A `Mutex` can be used as a channel so the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. access tokens) across different
// execution contexts (e.g. interrupts)
unsafe impl<T> Sync for Mutex<T>
where
    T: Send,
{
}
