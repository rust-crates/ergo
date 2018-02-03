//! **make creating and synchronizing threads ergonomic, therefore fun!**
//!
//! This is the synchronization library as part of the [`ergo`] crates ecosystem. It contains useful
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
//! - [`ergo`]: https://github.com/rust-crates/ergo
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
//! ## Example: producer / consumer
//!
//! The producer/consumer model is this library's bread and butter. Once you understand
//! channels you should next learn producer/consumer.
//!
//! In the ergo_sync model you should:
//!
//! - Do "CPU work" using the rayon threadpool. This means using either the `pool_scope` function
//!   or the rayon parallel iterators (or both!)
//! - Do "IO work" using system threads. A typically good number is `min(8, num_cpus())` since
//!   8 is a typicaly a high bar for the number of channels on an SSD or other storage device.
//!
//! A typical application might look like this:
//!
//! ```no_compile
//!  +-----------------------+
//!  | Feed Paths to Parse   |
//!  | (typically one thread |
//!  | using walkdir which   |
//!  | is rediculously fast) |
//!  +-----------------------+
//!         ___|___
//!        /   |   \
//!       v    v    v
//!  +------------------------+
//!  | 8 or more threads      |
//!  | receiving paths via    |
//!  | channels and reading   |
//!  | raw strings.           |
//!  |                        |
//!  | These are sent via     |
//!  | channel the next stage |
//!  +------------------------+
//!         ___|___
//!        /   |   \
//!       v    v    v
//!  +------------------------+
//!  | A rayon par_iter       |
//!  | reading the string     |
//!  | iterators and          |
//!  | processing them.       |
//!  |                        |
//!  | This is pure cpu work. |
//!  +------------------------+
//!            |
//!            |
//!            v
//!  +------------------------+
//!  | Results are collected  |
//!  | to prepare for next    |
//!  | step                   |
//!  +------------------------+
//! ```
//!
//! This example basically implements the above example using the source code
//! of this crate as the example. The below code searches through the crate
//! source looking for every use of the word "example".
//!
//! > Note: it is recommended to use [`ergo_fs`] to do filesystem operations, as all errors will
//! > have the _context_ (path and action) of what caused the error and you will have access to
//! > best in class filesystem operations like walking the directory structure and expressing
//! > the types you expect. We do not use it here so we can focus on `ergo_sync`'s API.
//!
//! [`ergo_fs`] https://github.com/rust-crates/ergo_fs
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//! use std::fs;
//! use std::io;
//! use std::io::prelude::*;
//! use std::path::{Path, PathBuf};
//!
//! /// List the dir and return any paths found
//! fn read_paths<P: AsRef<Path>>(
//!     dir: P, send_paths: &Sender<PathBuf>,
//!     errs: &Sender<io::Error>,
//! ) {
//!     for entry in ch_try!(fs::read_dir(dir), errs, return) {
//!         let entry = ch_try!(entry, errs, continue);
//!         let meta = ch_try!(entry.metadata(), errs, continue);
//!         if meta.is_file() {
//!             ch!(send_paths <- entry.path());
//!         } else if meta.is_dir() {
//!             // recurse into the path
//!             read_paths(entry.path(), send_paths, errs);
//!         } else {
//!             // ignore symlinks for this example
//!         }
//!     }
//! }
//!
//! /// Send one line at a time from the file
//! fn read_lines(path: PathBuf, send_lines: &Sender<String>, errs: &Sender<io::Error>) {
//!     let file = ch_try!(fs::File::open(path), errs, return);
//!     let buf = io::BufReader::new(file);
//!     for line in buf.lines() {
//!         // send the line but return immediately if any `io::Error` is hit
//!         ch!(send_lines <- ch_try!(line, errs, return));
//!     }
//! }
//!
//! /// Parse each line for "example", counting the number of times it appears.
//! fn count_examples(line: &str) -> u64 {
//!     line.match_indices("example").count() as u64
//! }
//!
//! fn main() {
//!     let (send_errs, recv_errs) = ch::bounded(128);
//!     let (send_paths, recv_paths) = ch::bounded(128);
//!
//!     // First we spawn a single thread to handle errors.
//!     // In this case we will just count and log them.
//!     let handle_errs = spawn(|| {
//!         take!(recv_errs);
//!         let mut count = 0_u64;
//!         for err in recv_errs.iter() {
//!             eprintln!("ERROR: {}", err);
//!             count += 1;
//!         }
//!         count
//!     });
//!
//!     // We spawn a single thread to "walk" the directory for paths.
//!     let errs = send_errs.clone();
//!     spawn(|| {
//!         take!(send_paths, errs);
//!         read_paths("src", &send_paths, &errs);
//!     });
//!
//!     // We read the lines using 8 threads (since this an IO bound)
//!     let (send_lines, recv_lines) = ch::bounded(128);
//!     for _ in 0..8 {
//!         take!(=recv_paths, =send_lines, =send_errs);
//!         spawn(|| {
//!             take!(recv_paths, send_lines, send_errs as errs);
//!             for path in recv_paths {
//!                 read_lines(path, &send_lines, &errs);
//!             }
//!         });
//!     }
//!     drop(send_lines);
//!
//!     // Now we do actual "CPU work" using the rayon thread pool
//!     let (send_count, recv_count) = ch::bounded(128);
//!
//!     // We set up the receiver before kicking off rayon since
//!     // rayon blocks until it is done.
//!     let counter = spawn(|| {
//!         take!(recv_count);
//!         recv_count.iter().count()
//!     });
//!
//!     // FIXME: how do I actually do this....???
//!     let received: Vec<_> = recv_lines.iter().collect();
//!     received.par_iter().for_each(|line| {
//!         take!(=send_count);
//!         ch!(send_count <- count_examples(line));
//!     });
//!
//!     // Finally we can get our count.
//!     drop(send_count);
//!     let count = counter.finish();
//!     # // assert_eq!(839, count);
//!
//!     // And assert we had no errors
//!     drop(send_errs);
//!     assert_eq!(0, handle_errs.finish());
//! }
//! ```
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

#![feature(trace_macros)]

#[allow(unused_imports)]
#[macro_use(take)]
extern crate taken;
#[allow(unused_imports)]
#[macro_use(select_loop)]
pub extern crate crossbeam_channel;
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
///
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
///
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use std::sync::mpsc::sync_channel;
///
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
/// ```
///
/// ## Example: using non-blocking syntax
///
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use ergo_sync::*;
/// # fn main() {
/// let (send, recv) = ch::bounded(3);
/// assert_eq!(None, ch!(<-? recv)); // no values sent yet
///
/// assert!(ch!(send <-? 4).is_none());
/// assert_eq!(Some(4), ch!(<-? recv));
/// assert_eq!(None, ch!(<-? recv));
///
/// assert!(ch!(send <-? 7).is_none());
/// assert!(ch!(send <-? 42).is_none());
/// assert!(ch!(send <-? 1).is_none());
/// // further attempts return the value
/// assert_eq!(Some(100), ch!(send <-? 100));
///
/// assert_eq!(Some(7), ch!(<-? recv));
///
/// assert_eq!(Some(42), ch!(<-? recv));
/// assert_eq!(Some(1), ch!(<-? recv));
/// assert_eq!(None, ch!(<-? recv));
/// assert!(ch!(! <-? recv)); // senders still exist
///
/// drop(send);
/// // ch!(<-? recv); // panics
/// ch!(! <-? recv);  // succeeds
/// # }
/// ```
#[macro_export]
macro_rules! ch {
    [$send:ident <-? $value:expr] => {
        match $send.try_send($value) {
            Ok(()) => None,
            Err(ch::TrySendError::Full(v)) => Some(v),
            Err(ch::TrySendError::Disconnected(_)) => {
                panic!("Attempted to send a value but receivers are disconnected");
            }
        }
    };

    [$send:ident <- $value:expr] => {
        match $send.send($value) {
            Ok(_) => {},
            Err(err) => panic!("{} for `send`.", err),
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
    [<- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => v,
            Err(err) => panic!("{} for `recv`.", err),
        }
    };


    [! <-? $recv:ident] => {
        match $recv.try_recv() {
            Ok(v) => panic!("Got {:?} when expecting senders to be closed.", v),
            Err(ch::TryRecvError::Empty) => true,  // senders still exist
            Err(ch::TryRecvError::Disconnected) => false, // no more senders
        }
    };
    [! <- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => panic!("Got {:?} when expecting senders to be closed.", v),
            Err(err) => (),
        }
    };
}

/// Same as the `try!` macro, except if the expression fails than the `Err` is sent on the
/// `$send` channel and the requested action is performed.
///
/// Suggested possible actions:
/// - `continue`
/// - `return`
/// - `break`
///
/// # Examples
///
/// ```rust
/// #[macro_use] extern crate ergo_sync;
/// use ergo_sync::*;
/// # fn main() {
/// let (send_err, recv_err) = ch::unbounded();
/// let items = &[Ok("this is alright"), Err("not ok"), Err("still not okay")];
/// # let mut okay = 0;
/// for item in items.iter() {
///     let v = ch_try!(*item, send_err, continue);
///     println!("got: {}", v);
///     # okay += 1;
/// }
///
/// drop(send_err);
/// let errs: Vec<_> = recv_err.iter().collect();
/// assert_eq!(vec!["not ok", "still not okay"], errs);
/// # assert_eq!(1, okay);
/// # }
/// ```
#[macro_export]
macro_rules! ch_try {
    [$expr:expr, $send:ident, continue] => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                ch!($send <- e);
                continue;
            }
        }
    };
    [$expr:expr, $send:ident, return] => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                ch!($send <- e);
                return;
            }
        }
    };
    [$expr:expr, $send:ident, return $expr:expr] => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                ch!($send <- e);
                return $expr;
            }
        }
    };
}
