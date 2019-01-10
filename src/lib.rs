//! Abstractions common to bare metal systems

#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(
    all(feature = "const-fn", unstable_const_fn),
    feature(const_fn)
)]
#![no_std]

use core::cell::{RefCell, UnsafeCell};

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
    pub fn borrow<'cs>(&'cs self, _cs: &'cs CriticalSection) -> &'cs T {
        unsafe { &*self.inner.get() }
    }
}

/// ``` compile_fail
/// fn bad(cs: &bare_metal::CriticalSection) -> &u32 {
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

// NOTE A `Mutex` can be used as a channel so the protected data must be `Send`
// to prevent sending non-Sendable stuff (e.g. access tokens) across different
// execution contexts (e.g. interrupts)
unsafe impl<T> Sync for Mutex<T> where T: Send {}

/// A shared value wrapper
///
/// Uses `Mutex` internally, so the same caveats about multi-core systems apply
pub struct Shared<T> {
    inner: Mutex<RefCell<Option<T>>>,
}

impl<T> Shared<T> {
    /// Creates a new empty shared value
    #[cfg(feature = "const-fn")]
    pub const fn new() -> Self {
        Shared {
            inner: Mutex::new(RefCell::new(None)),
        }
    }

    /// Creates a new empty shared value
    #[cfg(not(feature = "const-fn"))]
    pub fn new() -> Self {
        Shared {
            inner: Mutex::new(RefCell::new(None)),
        }
    }

    /// Loads new contents into a shared value, if the value already contained
    /// data the old value is returned
    pub fn put(&self, cs: &CriticalSection, value: T) -> Option<T> {
        self.inner.borrow(cs).replace(Some(value))
    }

    /// Attempts to get a reference to the data in the shared value, may fail
    /// if there are current mutable references, or if the value is empty
    pub fn get<'a>(&'a self, cs: &'a CriticalSection) -> Option<core::cell::Ref<'a, T>> {
        self.inner
            .borrow(cs)
            .try_borrow()
            .ok()
            .filter(|inner| inner.is_some())
            .map(|inner| core::cell::Ref::map(inner, |v| v.as_ref().unwrap()))
    }

    /// Attempts to get a reference to the data in the shared value, may fail
    /// if there are current mutable or immutable references, or if the value
    /// is empty
    pub fn get_mut<'a>(&'a self, cs: &'a CriticalSection) -> Option<core::cell::RefMut<'a, T>> {
        self.inner
            .borrow(cs)
            .try_borrow_mut()
            .ok()
            .filter(|inner| inner.is_some())
            .map(|inner| core::cell::RefMut::map(inner, |v| v.as_mut().unwrap()))
    }
}
