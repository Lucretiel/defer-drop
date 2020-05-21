use std::{
    any::Any,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    thread,
};

use crossbeam::channel::{self, Sender};

mod once_slot;
use once_slot::OnceSlot;

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
        static GARBAGE_CAN: OnceSlot<Sender<Box<dyn Any + Send>>> = OnceSlot::new();
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
