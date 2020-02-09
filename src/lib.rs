//! Abstractions common to bare metal systems.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::cell::UnsafeCell;
use core::marker::PhantomData;

/// Critical section token.
///
/// Indicates that you are executing code within a critical section.
#[derive(Clone, Copy)]
pub struct CriticalSection<'cs> {
    _0: PhantomData<&'cs ()>,
}

impl<'cs> CriticalSection<'cs> {
    /// Creates a critical section token.
    ///
    /// This method is meant to be used to create safe abstractions rather than
    /// meant to be directly used in applications.
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
}

impl<T> Mutex<T> {
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
const GH_6: () = ();

/// Interrupt number.
pub unsafe trait Nr {
    /// Returns the number associated with an interrupt.
    fn nr(&self) -> u8;
}

/// Trait for static (singleton) resources with managed ownership.
///
/// This trait allows application code and libraries to take ownership of resources that exist once
/// on every core, or once on the entire system.
///
/// # Safety
///
/// In order to safely implement this trait, the implementor must ensure that:
/// - A call to `take()` or `steal()` atomically ensures that no further call to `take()` will
///   succeed. This is commonly accomplished by using a static `AtomicBool` variable and a
///   compare-and-swap operation or a critical section.
/// - It is impossible to link multiple crates containing the synchronization state together. This
///   is usually accomplished by defining a well-known [`links = "..."`][links] key in the
///   `Cargo.toml`.
///
/// [links]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#the-links-manifest-key
pub unsafe trait StaticResource: Sized {
    /// Obtains ownership of this resource singleton and makes it unavailable to future callers of
    /// `take()`.
    ///
    /// If `take()` or `steal()` have been called before, this returns `None`.
    fn take() -> Option<Self>;

    /// Obtains an instance of this resource and makes all future calls to `take()` return `None`.
    ///
    /// This will not check if `take()` or `steal()` have already been called before. It is the
    /// caller's responsibility to use the returned instance in a safe way that does not conflict
    /// with other instances.
    ///
    /// This function is intended to be used when it is statically known that the resource is still
    /// available (for example, in generated code that runs immediately after reset). It generally
    /// has lower cost than `take().unwrap()`.
    unsafe fn steal() -> Self;

    /// Unsafely obtains an instance of this resource.
    ///
    /// This will not check if `take()` or `steal()` have already been called before. It is the
    /// caller's responsibility to use the returned instance in a safe way that does not conflict
    /// with other instances.
    ///
    /// Contrary to `steal()`, `conjure()` will *not* make future calls to `take()` return `None`.
    ///
    /// This function can be used to perform operations on a resource, ignoring any current
    /// ownership of the resource. The safety of this depends on the specific resource, and on the
    /// operations performed.
    unsafe fn conjure() -> Self;
}
