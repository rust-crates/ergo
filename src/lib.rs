//! make creating and synchronizing threads ergonomic, therefore fun!
//!
//! This is the synchronization library as part of the `ergo` crates ecosystem. It contains useful
//! types, traits and functions for spawning threads and synchronizing them. It is named `sync`
//! because of `std::sync` and because it is _not_ async, which is/will be a spearate part of the
//! ergo ecocystem.
//!
//! The crates that are wraped/exported are:
//!
//! - [`crossbeam_channel`](https://github.com/crossbeam-rs/crossbeam-channel):
//!   Multi-producer multi-consumer channels for message passing
//! - [`rayon`](https://github.com/rayon-rs/rayon): Rayon: A data parallelism library for Rust
//! - [`taken`](https://github.com/vitiral/taken): macros for taking ownership
//!
//! Consider supporting their development individually and starring them on github.
//!
//! > **This crate is a WIP. More docs will be added in the future.**
//!
//! # Examples
//!
//! ## Example: producers and consumers
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let external_val = 42;
//!
//! // the thread scope allows us to access local variables
//! // and ensures that threads get joined.
//! let result = thread_scope(|sc| {
//!     // rendevous channel
//!     let (send, recv) = ch::bounded(0);
//!
//!     // -------------------------------
//!     // ---- spawn your consumers -----
//!     let consumer = sc.spawn(|| -> u64 {
//!         take!(recv); // same as `let recv = recv`
//!         recv.iter().sum()
//!     });
//!
//!     // -------------------------------
//!     // ---- spawn your producers -----
//!     take!(=send as s); // same as `let s = send.clone()`
//!     sc.spawn(|| {
//!         take!(s);
//!         // do some expensive function
//!         s.send(42_u64.pow(4)).unwrap();
//!     });
//!
//!     take!(=send as s);
//!     sc.spawn(|| {
//!         take!(s);
//!         // Each function can also use rayon's traits to do
//!         // iteration in parallel.
//!         (0..1000_u64).into_par_iter().for_each(|n| {
//!             s.send(n * 42).unwrap();
//!         });
//!     });
//!
//!     // Always have your final producer take `send` without
//!     // first cloning it. This will drop it and and prevent
//!     // deadlocks.
//!     sc.spawn(|| {
//!         take!(send, &external_val as val);
//!         send.send(expensive_fn(val)).unwrap();
//!     });
//!
//!     consumer.finish()
//! });
//!
//! assert_eq!(24_094_896, result);
//! # }
//!
//! /// Really expensive function
//! fn expensive_fn(v: &u32) -> u64 {
//!     println!("Doing expensive thing");
//!     sleep_ms(300);
//!     *v as u64 * 100
//! }
//! ```
//! ### Alternatives to `thread_scope`
//! You can also use [`rayon::scope`](../rayon/fn.scope.html) if you know that your threads
//! will be doing work (i.e. are not IO bound). However, do _not_ put both produers and consumers
//! into rayon threads, as rayon only guarantees that 1 or more threads will be running at a time
//! (hence you may inroduce deadlock).
//!
//! It is safe to mix [`spawn`], [`thread_scope`] and rayon threads.
//!
//! [`spawn`]: fn.spawn.html
//! [`thread_scope`]: fn.thread_scope.html
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
//! let receiving = {
//!     // This scope prevents us from forgetting to drop the sending channel,
//!     // as both `send` and `recv` are dropped at the end of the scope.
//!     let (send, recv) = ch::bounded(0);
//!
//!     // Kick off the receiving threads.
//!     //
//!     // Note that these do _not_ run in the rayon thread pool,
//!     // they are simple OS level threads from `std::thread::spawn`.
//!     let mut receiving = Vec::with_capacity(4);
//!     for _ in 0..4 {
//!         take!(=recv as r); // take a clone of `recv`
//!         receiving.push(spawn(|| {
//!             for letter in r {
//!                 println!("Received letter: {}", letter);
//!             }
//!         }));
//!     }
//!
//!     // Send values in parallel using the rayon thread pool.
//!     let mut chars: Vec<_> = "A man, a plan, a canal - Panama!"
//!         .chars()
//!         .collect();
//!     chars.into_par_iter().map(|letter| {
//!         take!(=send as s); // take a clone of `send`
//!         for _ in 0..10 {
//!             s.send(letter).unwrap();
//!         }
//!     });
//!
//!     // You must wait for the threads _outside_ of this scope, else you
//!     // will get deadlock.
//!     //
//!     // You could also call `drop(send)`, in which case you would not
//!     // need the scope at all. However, if you had more than one sending
//!     // channel you would also need to remember to drop _that_, etc etc.
//!     receiving
//! };
//!
//! // Wait until all threads have finished before exiting.
//! //
//! // Alternatively we could have used `ch::WaitGroup` in the
//! // receiving threads to keep track of when threads finished,
//! // however we would have to be diligent to make sure we don't
//! // forget to call `wg.add/done` at the appropriate times.
//! //
//! // `ch::WaitGroup` scales much better... but how often
//! // are you tracking more than 100 threads?
//! for r in receiving {
//!     r.finish();
//! }
//! # }
//! ```
//!
//! ## Example: using `select_loop` for channels
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (tx1, rx1) = ch::unbounded();
//! let (tx2, rx2) = ch::unbounded();
//!
//! spawn(move || ch!(tx1 <- "foo"));
//! spawn(move || ch!(tx2 <- "bar"));
//!
//! select_loop! {
//!     recv(rx1, msg) => {
//!         println!("Received a message from the first channel: {}", msg);
//!     }
//!     recv(rx2, msg) => {
//!         println!("Received a message from the second channel: {}", msg);
//!     }
//! }
//! # }
//! ```

#[allow(unused_imports)]
#[macro_use(take)]
extern crate taken;
#[allow(unused_imports)]
#[macro_use(select_loop)]
pub extern crate crossbeam_channel;
pub extern crate rayon;
pub extern crate std_prelude;
pub extern crate crossbeam_utils;


// -------- std_prelude exports --------
// Types
pub use std_prelude::{Arc, Duration, Mutex};
// Atomics
pub use std_prelude::{AtomicBool, AtomicIsize, AtomicOrdering, AtomicUsize, ATOMIC_USIZE_INIT};
// Functions
pub use std_prelude::{sleep, spawn};

// -------- macro exports--------
#[allow(unused_imports)]
#[doc(hidden)]
pub mod reexports {
    // hack to rexport macros
    #[doc(hidden)] pub use taken::*;
    #[doc(hidden)] pub use crossbeam_channel::*;
}
#[doc(hidden)]
pub use reexports::*;

// -------- other exports --------
pub use rayon::prelude::*;
pub use crossbeam_utils::scoped::{scope as thread_scope, Scope, ScopedJoinHandle, ScopedThreadBuilder};
/// Module for working with channels. Rexport of crossbeam_channel
pub mod ch {
    pub use crossbeam_channel::*;
}

use std_prelude::*;

/// Convinience trait mimicking `std::thread::JoinHandle` with better ergonomics.
pub trait FinishHandle<T>
where
    T: Send + 'static,
{
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
    fn finish(self) -> T;
}

impl<T: Send + 'static> FinishHandle<T> for ::std::thread::JoinHandle<T> {
    fn finish(self) -> T {
        self.join()
            .expect("finish failed to join, thread is poisoned")
    }
}

impl<T: Send + 'static> FinishHandle<T> for ScopedJoinHandle<T> {
    /// The scoped threads already panic when poisoned, so this is equivalent to
    /// `ScopedJoinHandle::join`
    ///
    /// > this behavior is not well documented. See [this issue].
    ///
    /// [this issue]: https://github.com/crossbeam-rs/crossbeam-utils/issues/6
    fn finish(self) -> T {
        self.join()
    }
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

/// Send or Receive on channels ergonomically.
///
/// This macro provides common syntax for using channels. `ch!(send <- value)` sends a value
/// and `ch!(<- recv)` receives a value.
///
/// The operation blocks until it can be performed. It panics when/if the operation is not possible
/// (because the other end of the channel has been closed).
///
/// Note that if a channel is leaked is it is possible for this operation to deadlock.
///
/// This macro works with both `crossbeam-channel` channels (which are exported by this crate) as
/// well as `std::mspc` channels.
///
/// # Examples
///
/// ## Using `ergo::chan` channels
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use ergo_sync::*;
/// # fn main() {
/// let (send, recv) = ch::bounded(3);
/// ch!(send <- 4);
/// ch!(send <- 7);
/// ch!(send <- 42);
/// assert_eq!(4, ch!(<- recv));
/// assert_eq!(7, ch!(<- recv));
/// let v = ch!(<- recv);
/// assert_eq!(42, v);
/// # }
/// ```
///
/// ## Using `std::mspc` channels
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use std::sync::mpsc::sync_channel;
/// # fn main() {
/// let (send, recv) = sync_channel(3);
/// ch!(send <- 4);
/// ch!(send <- 7);
/// ch!(send <- 42);
/// assert_eq!(4, ch!(<- recv));
/// assert_eq!(7, ch!(<- recv));
/// let v = ch!(<- recv);
/// assert_eq!(42, v);
/// # }
/// ```
#[macro_export]
macro_rules! ch {
    [$send:ident <- $value:expr] => {
        match $send.send($value) {
            Ok(_) => {},
            Err(err) => panic!("{} for `send`", err),
        }
    };
    [<- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => v,
            Err(err) => panic!("{} for `recv`", err),
        }
    }
}
