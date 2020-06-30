//! Abstractions common to bare metal systems

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate bare_metal;
pub use bare_metal::{CriticalSection, Mutex};

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
    pub const unsafe fn new(address: usize) -> Self {
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

/// Interrupt number
pub unsafe trait Nr {
    /// Returns the number associated with an interrupt
    fn nr(&self) -> u8;
}
