//! Abstractions common to bare metal systems.
//!
//! This crate is superseded by the [`critical-section`](critical_section) crate.

#![deny(missing_docs)]
#![no_std]
#![doc(html_root_url = "https://docs.rs/bare-metal/1.0")]

/// Critical section token.
///
/// An instance of this type indicates that the current thread is executing code within a critical
/// section.
#[deprecated(
    since = "1.1.0",
    note = "use `critical_section::CriticalSection` instead"
)]
pub type CriticalSection<'cs> = critical_section::CriticalSection<'cs>;

/// A mutex based on critical sections.
///
/// # Design
///
/// [`std::sync::Mutex`] has two purposes. It converts types that are [`Send`]
/// but not [`Sync`] into types that are both; and it provides
/// [interior mutability]. [`bare_metal::Mutex`](Mutex), on the other hand, only adds
/// [`Sync`]. It does *not* provide interior mutability.
///
/// This was a conscious design choice. It is possible to create multiple
/// [`CriticalSection`] tokens, either by nesting critical sections or [`Copy`]ing
/// an existing token. As a result, it would not be sound for [`Mutex::borrow`]
/// to return `&mut T`, because there would be nothing to prevent calling
/// `borrow` multiple times to create aliased `&mut T` references.
///
/// The solution is to include a runtime check to ensure that each resource is
/// borrowed only once. This is what [`std::sync::Mutex`] does. However, this is
/// a runtime cost that may not be required in all circumstances. For instance,
/// `Mutex<Cell<T>>` never needs to create `&mut T` or equivalent.
///
/// If `&mut T` is needed, the simplest solution is to use `Mutex<RefCell<T>>`,
/// which is the closest analogy to `std::sync::Mutex`. [`RefCell`](core::cell::RefCell)
/// inserts the exact runtime check necessary to guarantee that the `&mut T` reference
/// is unique.
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
#[deprecated(since = "1.1.0", note = "use `critical_section::Mutex` instead")]
pub type Mutex<T> = critical_section::Mutex<T>;
