use std::{cell::UnsafeCell, mem::MaybeUninit, sync::Once};

// TODO: This should be its own crate

/// A variant of `std::sync::Once` that provides shared, initialize-once access
/// to a variable.
#[derive(Debug)]
pub struct OnceSlot<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    once: Once,
}

impl<T> OnceSlot<T> {
    pub const fn new() -> Self {
        OnceSlot {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            once: Once::new(),
        }
    }

    /// Get the reference to the value in the slot, running the init function
    /// if it hasn't yet been initialized. Like one `Once`, the init function
    /// is guaranteed to only be run once, even if multiple threads call `get`.
    pub fn get(&self, init: impl FnOnce() -> T) -> &T {
        self.once.call_once(move || {
            let value = init();

            // Safety: our Once guarantees that this is the only existing
            // mutable reference to this OnceCell. It also guarantees that this
            // is the only write to the MaybeUninit.
            unsafe {
                (*self.slot.get()).as_mut_ptr().write(value);
            }
        });

        // Safety: The only mutable access to this cell is in call_once,
        // and the only way to reach this point is if it completed without
        // panicking.
        unsafe { &*(*self.slot.get()).as_ptr() }
    }
}

impl<T> Drop for OnceSlot<T> {
    fn drop(&mut self) {
        // Safety: because we have &mut self, we know that this is the only
        // reference to the cell. This means there's no possible
        // synchronization errors (where another thread is evaluating Once
        // while we're here in the destructor).
        // The read is safe because MaybeUninit doesn't destruct its contents.
        if self.once.is_completed() {
            drop(unsafe { (*self.slot.get()).as_ptr().read() });
        }
    }
}

// Safety: our interface ensures that synchronized access to the inner T is
// well-controlled
unsafe impl<T: Sync> Sync for OnceSlot<T> {}
