use std::any::Any;
use std::cell::UnsafeCell;
use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::sync::Once;
use std::thread;

use crossbeam::channel;

struct OnceSlot<T> {
    slot: UnsafeCell<MaybeUninit<T>>,
    once: Once,
}

impl<T> OnceSlot<T> {
    const fn new() -> Self {
        OnceSlot {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            once: Once::new(),
        }
    }

    fn get(&self, init: impl FnOnce() -> T) -> &T {
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeferDrop<T: Send + 'static> {
    inner: ManuallyDrop<T>,
}

impl<T: Send + 'static> DeferDrop<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        DeferDrop {
            inner: ManuallyDrop::new(value),
        }
    }

    pub fn into_inner(mut this: Self) -> T {
        let value = unsafe { ManuallyDrop::take(&mut this.inner) };
        mem::forget(this);
        value
    }
}

impl<T: Send + 'static> Drop for DeferDrop<T> {
    fn drop(&mut self) {
        static GARBAGE_CAN: OnceSlot<channel::Sender<Box<dyn Any + Send>>> = OnceSlot::new();
        let garbage_hole = GARBAGE_CAN.get(|| {
            let (sender, receiver) = channel::unbounded();
            // TODO: drops should ever panic, but if once does, we should
            // probably abort the process
            let _ = thread::spawn(move || receiver.into_iter().for_each(|thing| drop(thing)));
            sender
        });

        let value = unsafe { ManuallyDrop::take(&mut self.inner) };
        let boxed = Box::new(value);

        // This unwrap only panics if the GARBAGE_CAN thread panicked
        garbage_hole.send(boxed).unwrap();
    }
}

impl<T: Send + 'static> From<T> for DeferDrop<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: Send + 'static> AsRef<T> for DeferDrop<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T: Send + 'static> AsMut<T> for DeferDrop<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Send + 'static> Deref for DeferDrop<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Send + 'static> DerefMut for DeferDrop<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
