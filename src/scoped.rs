//! Module for working with scoped threads. This is just [`crossbeam_utils::scoped`] rexported.
//!
//! [`crossbeam_utils::scoped`]: ../crossbeam_utils/scoped/index.html
//!
//! # Examples
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
//! let result = scoped::scope(|sc| {
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
//!         ch!(s <- 42_u64.pow(4));
//!     });
//!
//!     take!(=send as s);
//!     sc.spawn(|| {
//!         take!(s);
//!         // Each function can also use rayon's traits to do
//!         // iteration in parallel.
//!         (0..1000_u64).into_par_iter().for_each(|n| {
//!             ch!(s <- n * 42);
//!         });
//!     });
//!
//!     // Always have your final producer take `send` without
//!     // cloning it. This will drop it and and prevent
//!     // deadlocks.
//!     sc.spawn(|| {
//!         take!(send, &external_val as val);
//!         ch!(send <- expensive_fn(val));
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
//!
//! ## Example: multiple producers and multiple consumers using channels
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
//! scoped::scope(|sc| {
//!     let (send, recv) = ch::bounded(0);
//!
//!     // Kick off the receiving threads as scoped threads
//!     for _ in 0..4 {
//!         take!(=recv);
//!         sc.spawn(|| {
//!             for letter in recv {
//!                 println!("Received letter: {}", letter);
//!             }
//!         });
//!     }
//!
//!     // Send values in parallel using the rayon thread pool.
//!     let mut chars: Vec<_> = "A man, a plan, a canal - Panama!"
//!         .chars()
//!         .collect();
//!     chars.into_par_iter().map(|letter| {
//!         take!(=send); // take a clone of `send`
//!         for _ in 0..10 {
//!             ch!(send <- letter);
//!         }
//!     });
//!
//!     // Note: the following occurs in order because of the scope:
//!     // - `send` and `recv` are dropped
//!     // - All threads are joined
//! })
//! # }
//! ```

pub use crossbeam_utils::scoped::*;

impl<T: Send + 'static> super::FinishHandle<T> for ScopedJoinHandle<T> {
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
