//! A utility type that allows you to defer dropping your data to a background
//! thread. See [`DeferDrop`] for details.
//!
//! Inspired by [https://abramov.io/rust-dropping-things-in-another-thread](https://abramov.io/rust-dropping-things-in-another-thread)

use std::{
    any::Any,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    thread,
};

use crossbeam_channel::{self as channel, Sender};
use once_cell::sync::OnceCell;

/// Wrapper type that, when dropped, sends the inner value to a global
/// background thread to be dropped. Useful in cases where a value takes a
/// long time to drop (for instance, a windows file that might block on close,
/// or a large data structure that has to extensively recursively trawl
/// itself).
///
/// `DeferDrop` implements `Deref` and `DerefMut`, meaning it can be
/// dereferenced and freely used like a container around its inner type.
///
/// # Notes:
///
/// Carefully consider whether this pattern is necessary for your use case.
/// Like all worker-thread abstractions, sending the value to a separate
/// thread comes with its own costs, so it should only be done if performance
/// profiling indicates that it's a performance gain.
///
/// There is only one global worker thread. Dropped values are enqueued in an
/// unbounded channel to be consumed by this thread; if you produce more
/// garbage than the thread can handle, this will cause unbounded memory
/// consumption. There is currently no way for the thread to signal or block
/// if it is overwhelmed.
///
/// All of the standard non-determinism threading caveats apply here. The
/// objects are guaranteed to be destructed in the order received through a
/// channel, which means that objects sent from a single thread will be
/// destructed in order. However, there is no guarantee about the ordering of
/// interleaved values from different threads. Additionally, there are no
/// guarantees about how long the values will be queued before being dropped,
/// or even that they will be dropped at all. If your `main` thread terminates
/// before all drops could be completed, they will be silently lost (as though
/// via a [`mem::forget`]).This behavior is entirely up to your OS's thread
/// scheduler. There is no way to receive a signal indicating when a particular
/// object was dropped.
///
/// # Example
///
/// ```
/// use defer_drop::DeferDrop;
/// use std::time::{Instant, Duration};
///
/// let massive_vec: Vec<Vec<i32>> = (0..1000000)
///     .map(|_| vec![1, 2, 3])
///     .collect();
///
/// let deferred = DeferDrop::new(massive_vec.clone());
///
/// fn timer(f: impl FnOnce()) -> Duration {
///     let start = Instant::now();
///     f();
///     Instant::now() - start
/// }
///
/// let drop1 = timer(move || drop(massive_vec));
/// let drop2 = timer(move || drop(deferred));
///
/// assert!(drop2 < drop1);
/// ```
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeferDrop<T: Send + 'static> {
    inner: ManuallyDrop<T>,
}

impl<T: Send + 'static> DeferDrop<T> {
    /// Create a new `DeferDrop` value.
    #[inline]
    pub fn new(value: T) -> Self {
        DeferDrop {
            inner: ManuallyDrop::new(value),
        }
    }

    /// Unwrap the `DeferDrop`, returning the inner value. This has the effect
    /// of cancelling the deferred drop behavior; ownership of the inner value
    /// is transferred to the caller.
    pub fn into_inner(mut this: Self) -> T {
        let value = unsafe { ManuallyDrop::take(&mut this.inner) };
        mem::forget(this);
        value
    }
}

static GARBAGE_CAN: OnceCell<Sender<Box<dyn Any + Send>>> = OnceCell::new();

impl<T: Send + 'static> Drop for DeferDrop<T> {
    fn drop(&mut self) {
        let garbage_can = GARBAGE_CAN.get_or_init(|| {
            let (sender, receiver) = channel::unbounded();
            // TODO: drops should never panic, but if once does, we should
            // probably abort the process
            let _ = thread::spawn(move || receiver.into_iter().for_each(drop));
            sender
        });

        let value = unsafe { ManuallyDrop::take(&mut self.inner) };
        let boxed = Box::new(value);

        // This unwrap only panics if the GARBAGE_CAN thread panicked
        garbage_can.send(boxed).unwrap();
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

#[cfg(test)]
mod tests {
    use crossbeam_channel as channel;
    use std::thread;
    use std::time::Duration;

    use crate::DeferDrop;

    #[test]
    fn test() {
        /// This struct, when dropped, reports the thread ID of its dropping
        /// thread to the channel
        struct ThreadReporter {
            chan: channel::Sender<thread::ThreadId>,
        }

        impl Drop for ThreadReporter {
            fn drop(&mut self) {
                self.chan.send(thread::current().id()).unwrap();
            }
        }

        let (sender, receiver) = channel::bounded(1);
        let this_thread_id = thread::current().id();

        let thing = DeferDrop::new(ThreadReporter { chan: sender });
        drop(thing);

        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(id) => assert_ne!(
                id, this_thread_id,
                "thing wasn't dropped in a different thread"
            ),
            Err(_) => assert!(
                false,
                "thing wasn't dropped within one second of being dropped"
            ),
        }
    }
}
