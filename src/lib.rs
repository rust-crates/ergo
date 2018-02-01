//! make creating and synchronizing threads ergonomic, therefore fun!
//!
//! This is the synchronization library as part of the `ergo` crates ecosystem. It contains useful
//! types, traits and functions for spawning threads and synchronizing them. It is named `sync`
//! because of `std::sync` and because it is _not_ async, which is/will be a spearate part of the
//! ergo ecocystem.
//!
//! The crates that are wraped/exported are:
//!
//! - [`rayon`](https://github.com/rayon-rs/rayon): Rayon: A data parallelism library for Rust
//! - [`chan`](https://github.com/BurntSushi/chan): Multi-producer, multi-consumer concurrent
//!   channel for Rust.
//! - [`taken`](https://github.com/vitiral/taken): macros for taking ownership
//!
//! Consider supporting their development individually and starring them on github.
//!
//! > **This crate is a WIP. More docs will be added in the future.**
//!
//! # Examples
//!
//! ## Example: most of the features together
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let val = 42;
//!
//! // rendezvous channel
//! let (send, recv) = chan::sync(0);
//!
//! // The consumer must be spawned in its own thread or we risk
//! // deadlock. Do NOT put the consumer in the threadpool, as
//! // threadpools do not guarantee >1 threads running at a time.
//! let consumer = spawn(|| -> u64 {
//!     take!(recv); // same as `let recv = recv`
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
//!
//! # Example: multiple producers and multiple consumers using channels
//!
//! This example is addapted from the [chan docs].
//!
//! [chan docs]: https://docs.rs/chan/0.1.20/chan/#example-multiple-producers-and-multiple-consumers
//!
//! ```
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (s, r) = chan::sync(0);
//!
//! let v = vec!['a', 'b', 'c', 'd'];
//! v.into_par_iter().map(|letter| {
//!     take!(=s);
//!     for _ in 0..10 {
//!         s.send(letter);
//!     }
//! });
//!
//! // A wait group lets us synchronize the completion of multiple threads.
//! let wg = chan::WaitGroup::new();
//! for _ in 0..4 {
//!     wg.add(1);
//!     let wg = wg.clone();
//!     let r = r.clone();
//!     spawn(move || {
//!         for letter in r {
//!             println!("Received letter: {}", letter);
//!         }
//!         wg.done();
//!     });
//! }
//!
//! drop(s); // drop the sender, else you will get a deadlock
//!
//! // If this was the end of the process and we didn't call `wg.wait()`, then
//! // the process might quit before all of the consumers were done.
//! // `wg.wait()` will block until all `wg.done()` calls have finished.
//! wg.wait();
//! # }
//! ```

#[allow(unused_imports)]
#[macro_use(take)]
extern crate taken;
#[allow(unused_imports)]
#[macro_use(chan_select)]
pub extern crate chan;
pub extern crate rayon;
pub extern crate std_prelude;

#[allow(unused_imports)] // this actually exports the macro
use chan::*;
#[allow(unused_imports)] // this actually exports the macro
use taken::*;
use std_prelude::*;

pub use rayon::prelude::*;

// -------- std_prelude --------
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
///         let send = send.clone();
///         send.send(4);
///     },
///     {
///         take!(=send); // let send = send.clone()
///         send.send(12);
///     },
///     {
///         take!(=send as s); // let s = send.clone()
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
        rayon::scope(|s| {
            $(
                s.spawn(|_| $thread);
            )*
        });
    };
}

/// Just sleep for a certain number of milliseconds.
///
/// Equivalent of `sleep(Duration::from_millis(millis))`
///
/// This function exists in `std::thread`, so it created here instead.
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
