//! **make creating and synchronizing threads ergonomic, therefore fun!**
//!
//! This is the synchronization library as part of the [`ergo`] crates ecosystem. It contains useful
//! types, traits and functions for spawning threads and synchronizing them. It is named `sync`
//! because of `std::sync` and because it is _not_ async, which is/will be a separate part of the
//! ergo ecocystem.
//!
//! The crates that are wraped/exported are:
//!
//! - [`crossbeam_channel`](https://github.com/crossbeam-rs/crossbeam-channel):
//!   Multi-producer multi-consumer channels for message passing
//! - [`taken`](https://github.com/vitiral/taken): macros for taking ownership
//!
//! Consider supporting their development individually and starring them on github.
//!
//! [`ergo`]: https://github.com/rust-crates/ergo
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
//! - **[`ch!`]**:Use with channels with ergonomic syntax and panic with helpful error messages
//!   when sending/receiving on a channel is invalid.
//!   - `ch!(send <- 42)` for sending a value.
//!   - `let v = ch!(<- recv)` for receiving a value.
//!   - `ch!(! <- recv)` to wait for channels to close.
//!   - `<-?` for async operation support.
//! - **[`ch_try!`]**: to handle an expression that could be `Err` and send it over a channel if it
//!   is.
//! - **[`select_loop!`]**: for selecting from multiple channels.
//! - **[`take!`]**: for expressing ownership consisely. You will move or clone
//!   variables extremely often in threads, this helps you express that better than
//!   `let value = value`.
//!
//! [`ch` module]: ch/index.html
//! [`spawn`]: fn.spawn.html
//! [`rayon` prelude]: ../rayon/index.html
//! [`rayon::scope`]: file:///home/garrett/src/ruststuff/ergo_sync/target/doc/rayon/fn.scope.html
//! [`take!`]: macro.take.html
//! [`ch!`]: macro.ch.html
//! [`ch_try!`]: macro.ch_try.html
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
//! In the `ergo_sync` model you should:
//!
//! - Do "CPU work" using the rayon threadpool. This typically means using the rayon parallel
//!   iterators, but you can also use `rayon::scope`
//!
//! - Do "IO work" using system threads. A typically good number is `min(8, num_cpus())` since
//!   8 is a typicaly a high bar for the number of channels on an SSD or other storage device.
//!
//! A typical application might look like this:
//!
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
//! [`ergo_fs`]: https://github.com/rust-crates/ergo_fs
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//!
//! use std::fs;
//! use std::io;
//! use std::io::prelude::*;
//! use std::path::{Path, PathBuf};
//! use ergo_sync::*;
//!
//! /// List the dir and return any paths found
//! fn read_paths<P: AsRef<Path>>(
//!     dir: P, send_paths: &Sender<PathBuf>,
//!     errs: &Sender<io::Error>,
//! ) {
//!     for entry in ch_try!(errs, fs::read_dir(dir), return) {
//!         let entry = ch_try!(errs, entry, continue);
//!         let meta = ch_try!(errs, entry.metadata(), continue);
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
//!     let file = ch_try!(errs, fs::File::open(path), return);
//!     let buf = io::BufReader::new(file);
//!     for line in buf.lines() {
//!         // send the line but return immediately if any `io::Error` is hit
//!         ch!(send_lines <- ch_try!(errs, line, return));
//!     }
//! }
//!
//! /// Parse each line for "example", counting the number of times it appears.
//! fn count_examples(line: &str) -> u64 {
//!     // Probably use the `regex` crate in a real life example.
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
//!     // We read the lines using 8 threads (since this is IO bound)
//!     let (send_lines, recv_lines) = ch::bounded(128);
//!     for _ in 0..8 {
//!         take!(=recv_paths, =send_lines, =send_errs);
//!         spawn(|| {
//!             take!(recv_paths, send_lines, send_errs);
//!             for path in recv_paths {
//!                 read_lines(path, &send_lines, &send_errs);
//!             }
//!         });
//!     }
//!     // It's always good to drop your sender when you don't need
//!     // it to prevent deadlock.
//!     drop(send_lines);
//!
//!     // Now we do actual "CPU work" using the rayon thread pool
//!     let (send_count, recv_count) = ch::bounded(128);
//!
//!     // Create a pool of threads for actually doing the "work"
//!     for _ in 0..num_cpus::get() {
//!         take!(=recv_lines, =send_count);
//!         spawn(move || {
//!             for line in recv_lines.iter() {
//!                 let count = count_examples(&line);
//!                 if count != 0 {
//!                     ch!(send_count <- count);
//!                 }
//!             }
//!         });
//!     }
//!     drop(send_count);
//!     drop(send_errs);
//!
//!     // Finally we can get our count.
//!     let count: u64 = recv_count.iter().sum();
//!     # // assert_eq!(839, count);
//!
//!     // And assert we had no errors
//!     assert_eq!(0, handle_errs.finish());
//! }
//! ```
#![feature(trace_macros)]

#[allow(unused_imports)]
#[macro_use(take)]
extern crate taken;
#[allow(unused_imports)]
#[macro_use(select_loop)]
pub extern crate crossbeam_channel;
pub extern crate std_prelude;
pub extern crate num_cpus;

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

pub mod ch;

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

