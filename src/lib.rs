pub extern crate chan;
pub extern crate rayon;
pub extern crate std_prelude;

pub use std_prelude::{
    // Types
    Arc,
    Mutex,

    // Atomics
    AtomicBool,
    AtomicIsize,
    AtomicUsize,
    AtomicOrdering,
    ATOMIC_USIZE_INIT,

    // Used with sleep
    Duration,

    // Functions
    sleep, spawn
};


/// Convinience trait like `std::thread::JoinHandle` to avoid having to use
/// `thread.join().unwrap()` so much.
pub trait FinishHandle<T>
    where T: Send + 'static
{
    fn finish(self) -> T;
}

impl<T: Send + 'static> FinishHandle<T> for ::std::thread::JoinHandle<T> {
    /// Finishes the thread, returning the value.
    ///
    /// This is the same as `JoinHandle::join()` except the unwrap is automatic.
    ///
    /// # Panics
    /// Panics if the thread is poisoned (if a panic happened inside the thread).
    fn finish(self) -> T {
        self.join().expect("finish failed to join, thread is poisoned")
    }
}

/// Spawn multiple threads and then join them. These are run in the rayon threadpool and are not
/// necessarily guaranteed to run in parallel.
///
/// This is slower than using _either_ `rayon::join` or rayon's parallel iterators and requires
/// heap allocations. See the rayon documentation for [`scope`](../rayon/fn.scope.html) for more
/// details and alternatives.
///
/// Although it is less efficient than other APIs exposed by rayon, it can be unergonomic to use
/// rayon directly when you want to run more than 2 workloads in parallel. This _is_ ergonomic and
/// for most use cases is _efficient enough_.
///
/// # Examples
/// ```
/// #[macro_use] extern crate ergo_sync;
/// use ergo_sync::*;
///
/// # fn main() {
/// let (send, recv) = chan::sync(0); // rendezvous channel
///
/// // The consumer must be spawned in a thread or we risk deadlock
/// // Do NOT put the consumer in the threadpool, as it does not
/// // guarantee >1 thread running at a time.
/// let consumer = spawn(move|| {
///     let recv = recv;
///     recv.iter().sum()
/// });
///
/// join_pool!{
///     {
///         let s = send.clone();
///         s.send(4);
///     },
///     {
///         let s = send.clone();
///         s.send(12);
///     },
///     {
///         let s = send.clone();
///         s.send(26);
///     }
/// };
///
/// drop(send); // the channel must be dropped for iterator to stop.
/// assert_eq!(42, consumer.finish());
/// # }
/// ```
#[macro_export]
macro_rules! join_pool {
    ( $( $thread:expr ),* ) => {
        ::ergo_sync::rayon::scope(|s| {
            $(
                s.spawn(|_| $thread);
            )*
        });
    };
}
