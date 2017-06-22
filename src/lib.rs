#![feature(const_fn)]
#![no_std]

use core::cell::UnsafeCell;
use core::marker::PhantomData;

pub mod ctxt;

/// A peripheral
#[derive(Debug)]
pub struct Peripheral<T>
where
    T: 'static,
{
    address: usize,
    _marker: PhantomData<&'static mut T>,
}

impl<T> Peripheral<T> {
    /// Creates a new peripheral
    ///
    /// `address` is the base address of the register block
    pub const unsafe fn new(address: usize) -> Self {
        Peripheral {
            address: address,
            _marker: PhantomData,
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

/// Critical section context
///
/// Indicates that you are executing code within a critical section
pub struct CriticalSection {
    _0: (),
}

impl CriticalSection {
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
    pub const fn new(value: T) -> Self {
        Mutex { inner: UnsafeCell::new(value) }
    }
}

impl<T> Mutex<T> {
    /// Borrows the data for the duration of the critical section
    pub fn borrow<'cs>(&self, _ctxt: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

// NOTE `Mutex` can be used as a channel so, the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. interrupt tokens) across
// different execution contexts (e.g. interrupts)
unsafe impl<T> Sync for Mutex<T>
where
    T: Send,
{
}

/// Interrupt number
pub unsafe trait Nr {
    /// Returns the number associated with this interrupt
    fn nr(&self) -> u8;
}
