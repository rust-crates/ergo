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
//! Here is an example of all of the features together.
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//! use ergo_sync::rayon::prelude::*;
//! use std_prelude::*;
//!
//! # fn main() {
//! let val = 42;
//!
//! // rendezvous channel
//! let (send, recv) = chan::sync(0);
//!
//! // The consumer must be spawned in a thread or we risk deadlock.
//! // Do NOT put the consumer in the threadpool, as threadpools
//! // do not guarantee >1 threads running at a time.
//! let consumer = spawn(move|| -> u64 {
//!     let recv = recv;
//!     recv.iter().sum()
//! });
//!
//! // spawn and join N number threads
//! join!{
//!     {
//!         let s = send.clone();
//!         // do some expensive function
//!         s.send(42_u64.pow(4));
//!     },
//!     {
//!         // Each function can also use rayon's traits to do iteration in parallel.
//!         take!(=send as s); // same as `let s = send.clone()`
//!         let s = send.clone();
//!         (0..1000_u64).into_par_iter().for_each(|n| {
//!             s.send(n * 42);
//!         });
//!     },
//!     {
//!         take!(=send as s, &val);
//!         s.send(expensive_fn(val));
//!     },
//! };
//!
//! drop(send); // the channel must be dropped for iterator to stop.
//! assert_eq!(24_094_896, consumer.finish());
//! # }
//!
//! /// Really expensive function
//! fn expensive_fn(v: &u32) -> u64 {
//!     println!("Doing expensive thing");
//!     sleep_ms(300);
//!     *v as u64 * 100
//! }
//! ```

#[allow(unused_imports)]
#[macro_use]
extern crate taken;
pub extern crate chan;
pub extern crate rayon;
pub extern crate std_prelude;

pub use taken::*;
pub use chan::{after, after_ms, tick, tick_ms, Receiver, Sender};
pub use rayon::prelude::*;

// Types
pub use std_prelude::{Arc, Duration, Mutex};

// Atomics
pub use std_prelude::{AtomicBool, AtomicIsize, AtomicOrdering, AtomicUsize, ATOMIC_USIZE_INIT};

// Functions
pub use std_prelude::{sleep, spawn};

/// Convinience trait mimicking `std::thread::JoinHandle` with better ergonomics.
pub trait FinishHandle<T>
where
    T: Send + 'static,
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
    ///
    /// # Examples
    /// ```rust
    /// # extern crate ergo_sync;
    /// # use ergo_sync::*;
    /// # fn main() {
    /// // sleep for half a second
    /// let th = spawn(|| sleep_ms(100));
    /// th.finish(); // as opposed to `th.join().unwrap()`
    /// # }
    /// ```
    fn finish(self) -> T {
        self.join()
            .expect("finish failed to join, thread is poisoned")
    }
}

/// Spawn multiple _scoped_ threads and then join them. These are run using the _current scope_ in
/// the rayon threadpool and are not necessarily guaranteed to run in parallel.
///
/// The fact that they are scoped means that you can reference variables from the current stack,
/// since the thread is guaranteed to terminate after the `join!` statement is complete.
///
/// This is slower than using _either_ `rayon::join` or rayon's parallel iterators. It also
/// requires heap allocations. See the rayon documentation for [`scope`](../rayon/fn.scope.html)
/// for more details and alternatives.
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
/// join!{
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
///     },
/// };
///
/// drop(send); // the channel must be dropped for iterator to stop.
/// assert_eq!(42, consumer.finish());
/// # }
/// ```
#[macro_export]
macro_rules! join {
    ( $( $thread:expr ),* $(,)* ) => {
        ::ergo_sync::rayon::scope(|s| {
            $(
                s.spawn(|_| $thread);
            )*
        });
    };
}

/// Just sleep for a certain number of milliseconds
///
/// Equivalent of `sleep(Duration::from_millis(millis))`
///
/// # Examples
/// ```rust
/// # extern crate ergo_sync;
/// # use ergo_sync::*;
/// # fn main() {
/// // sleep for half a second
/// sleep_ms(500);
/// # }
/// ```
#[inline(always)]
pub fn sleep_ms(millis: u64) {
    sleep(Duration::from_millis(millis))
}
