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
//! # How to Use
//!
//! Use this library with:
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//! # fn main() {}
//! ```
//!
//! It provides the following types and modules for most use cases.
//!
//! - **[`ch` module]**: for channel types (also see the [`ch!`] and [`select_loop!`] macros).
//! - **[`scoped` module]**: for creating scoped threads.
//! - **[`rayon` prelude]**: for parallizing iterators using a work-stealing threadpool. Use this
//!   (`par_iter()` method) if you have to parallize a ton of things and you want it to just happen
//!   as quickly as possible with as few threads as possible.
//! - **[`spawn`]**: the standad `std::thread::spawn` which spawns a regular OS thread. The
//!   advantage of this (over scoped threads) is that it can outlive the current function. The
//!   disadvantage is that as far as the compiler knows it _always_ outlives the current function,
//!   meaning it must own all of its variables (or they have to be `'static`).
//!
//! In addition it provides the following helper macros:
//!
//! - **[`take!`]**: for expressing ownership consisely. You will move or clone
//!   variables extremely often in threads, this helps you express that better than
//!   `let v = v.clone()`.
//! - **[`ch!`]**: deal with channels with golang-like syntax and panic with helpful error messages
//!   on when sending/receiving on a channel is invalid.
//! - **[`select_loop!`]**: for selecting from multiple channels.
//!
//! [`ch` module]: ch/index.html
//! [`scoped` module]: scoped/index.html
//! [`spawn`]: fn.spawn.html
//! [`rayon` prelude]: ../rayon/index.html
//! [`take!`]: macro.take.html
//! [`ch!`]: macro.ch.html
//! [`select_loop!`]: macro.select_loop.html
//!
//! # Examples
//!
//! ## Example: Channels
//! See the docs for the [`ch` module].
//!
//!
//! ## Example: Scoped Threads
//! See the docs for the [`scoped` module].
//!
//!
//! # Additional Types
//! The types and modules exported by default represent the most ones used. However, the sub-crates
//! in this crate also support more specialized needs.
//!
//! These are slightly less ergonomic out of necessity, however they are not too bad once you get
//! used to them.
//!
//! ## Creating and Using Thread Pools
//!
//! **[`rayon::ThreadPool`]** can be used create a rayon thread pool with an explicit number of
//! threads. This can also create scoped threads. Note that the thread pool controls the number
//! of threads executed for _all_ rayon functions, so while externally they are not ergonomic
//! (you have to do quite a bit of work to set them up) any function internally will just
//! "do what you expect" and use the threads you have initialized.
//!
//! `ThreadPool` is not exported explicitly as the usecases are rare, but definitely do exist. This
//! is especially useful if you want to do (for example) IO work on a list of files. You know you
//! want more threads than are nessary for pure "work" but still want to limit them.
//!
//! [`rayon::ThreadPool`]: ../rayon/struct.ThreadPool.html

#[allow(unused_imports)]
#[macro_use(take)]
extern crate taken;
#[allow(unused_imports)]
#[macro_use(select_loop)]
pub extern crate crossbeam_channel;
pub extern crate crossbeam_utils;
pub extern crate rayon;
pub extern crate std_prelude;

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
    #[doc(hidden)]
    pub use taken::*;
    pub use crossbeam_channel::*;
}
pub use reexports::*;

// -------- other exports --------
pub use rayon::prelude::*;

pub mod ch;
pub mod scoped;

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
/// This macro provides common syntax for using channels.
///
/// Blocking syntax:
///
/// - `ch!(send <- value)`: blocks until a value is sent, panics if all receivers are dropped.
/// - `ch!(<- recv)`: blocks until a value is received, panics if all senders are dropped.
/// - `ch!(! <- recv)`: blocks until all senders are dropped, panics if a value is received. Used
///   for signaling.
///
/// > This syntax works with both `crossbeam-channel` channels (which are exported by this crate) as
/// > well as `std::mspc` channels.
///
/// > Note that these operations can deadlock if a channel is leaked.
///
/// Non-Blocking syntax:
///
/// - `ch!(send <-? value)`: returns `None` if the value was sent, `Some(value)` if the value
///   was not sent. Panics if all receivers are dropped.
/// - `ch!(<-? recv)`: returns `None` if no value is received, `Some(value)` if a value is
///   received. Panics if all senders are dropped.
/// - `ch!(! <-? recv)`: returns `true` if there are still senders and `false` if the seners have
///   been dropped. Panics if a value is received. Use with `while ch!(! <-? recv) { /* ... */ }`
///
/// > This syntax does _not_ work with `std::mspc` channels.
///
/// # Examples
///
/// ## Example: Using `ergo::chan` channels
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
///
/// drop(send);
/// // ch!(<- recv); // panics
/// ch!(! <- recv);  // succeeds
/// # }
/// ```
///
/// ## Example: Using `std::mspc` channels
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
///
/// drop(send);
/// // ch!(<- recv); // panics
/// ch!(! <- recv);  // succeeds
/// # }
///
/// ## Example: using `try_send` and `try_recv` but panicing if disconnected
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use ergo_sync::*;
/// # fn main() {
/// let (send, recv) = ch::bounded(3);
/// assert_eq!(None, ch!(<-? recv)); // no values sent yet
///
/// assert!(ch!(send <- 4).is_none());
/// assert_eq!(Some(4), ch!(<-? recv));
/// assert_eq!(None, ch!(<-? recv));
///
/// assert!(ch!(send <- 7).is_none());
/// assert!(ch!(send <- 42).is_none());
/// assert!(ch!(send <- 1).is_none());
/// // further attemps return the value
/// assert_eq!(Some(100), ch!(send <- 100));
///
/// assert_eq!(7, ch!(<-? recv));
///
/// assert_eq!(42, ch!(<-? recv));
/// assert_eq!(None, ch!(<-? recv));
/// assert!(ch!(! <-? recv)); // senders still exist
///
/// drop(send);
/// // ch!(?<- recv); // panics
/// ch!(! ?<- recv);  // succeeds
/// # }
/// ```
#[macro_export]
macro_rules! ch {
    [$send:ident <- $value:expr] => {
        match $send.send($value) {
            Ok(_) => {},
            Err(err) => panic!("{} for `send`.", err),
        }
    };

    [$send:ident <-? $value:expr] => {
        match $send.try_send($value) {
            Ok(()) => None,
            Err(ch::TrySendError::Full(v)) => Some(v),
            Err(ch::TrySendError::Disconnected(_)) => {
                panic!("Attempted to send a value but receivers are disconnected");
            }
        }
    };

    [<- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => v,
            Err(err) => panic!("{} for `recv`.", err),
        }
    };

    [<-? $recv:ident] => {
        match $recv.try_recv() {
            Ok(v) => Some(v),
            Err(ch::TryRecvError::Empty) => None,
            Err(ch::TryRecvError::Disconnected) => {
                panic!("Attempted to recv a value but senders are disconnected");
            }
        }
    };

    [! <- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => panic!("Got {:?} when expecting senders to be closed.", v),
            Err(err) => (),
        }
    };
    [! <-? $recv:ident] => {
        match $recv.recv() {
            Ok(v) => panic!("Got {:?} when expecting senders to be closed.", v),
            Err(ch::TryRecvError::Empty) => true,  // senders still exist
            Err(ch::TryRecvError::Disconnected) => false, // no more senders
        }
    };
}
