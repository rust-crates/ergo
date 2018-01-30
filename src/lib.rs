//! ergo_sync: making creating and synchronizing threads ergonomic, therefore fun!
//!
//! This is the synchronization library as part of the `ergo` crates ecosystem. It contains useful
//! types, traits and functions for spawning threads and synchronizing them. It is named `sync`
//! because of `std::sync` and because it is _not_ async, which is a spearate part of the
//! ergo ecocystem.
//!
//!
//! The crates that are wraped/exported are:
//!
//! - [`rayon`](https://github.com/rayon-rs/rayon): Rayon: A data parallelism library for Rust
//! - [`chan`](https://github.com/BurntSushi/chan): Multi-producer, multi-consumer concurrent
//!   channel for Rust.
//!
//! Consider supporting their development individually and starring them on github.
//!
//! # Examples
//!
//! Here is an example of all of the features together:
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//! use ergo_sync::rayon::prelude::*;
//! use std_prelude::*;
//!
//! # fn main() {
//!  // rendezvous channel
//! let (send, recv) = chan::sync(0);
//!
//! // The consumer must be spawned in a thread or we risk deadlock.
//! // Do NOT put the consumer in the threadpool, as threadpools
//! // do not guarantee >1 thread running at a time.
//! let consumer = spawn(move|| -> u64 {
//!     let recv = recv;
//!     recv.iter().sum()
//! });
//!
//! // spawn and join N number threads
//! join_pool!{
//!     {
//!         // Use "regular" functions that might have to do a lot of work.
//!         let s = send.clone();
//!         // do some expensive function
//!         let res: u64 = 42_u64.pow(2);
//!         s.send(res);
//!     },
//!     {
//!         // Each function can also use rayon's traits to do iteration in parallel.
//!         let s = send.clone();
//!         (0..1000_u64).into_par_iter().for_each(|n| {
//!             s.send(n * 42);
//!         });
//!     }
//! };
//!
//! drop(send); // the channel must be dropped for iterator to stop.
//! assert_eq!(20_980_764, consumer.finish());
//! # }
//! ```
pub extern crate chan;
pub extern crate rayon;
pub extern crate std_prelude;

pub use std::iter::IntoIterator;

pub use std_prelude::{
    // Traits
    FromIterator,

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
pub use rayon::prelude::*;


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
