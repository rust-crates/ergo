//! Module for working with channels. Rexport of [`crossbeam_channel`]
//!
//!
//! # Examples
//!
//! > Several of these examples are copies of the [`chan`] and [`crossbeam_channel`] crates.
//!
//! [`chan`]: https://github.com/BurntSushi/chan
//! [`crossbeam_channel`]: ../../crossbeam_channel/index.html
//!
//! ## Example: unbounded (async) channel
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (tx, rx) = ch::unbounded();
//!
//! // Can send an arbitrarily large number of messages.
//! for i in 0..1000 {
//!     ch!(tx <- i);
//! }
//! # }
//! ```
//!
//! ## Example: bounded (sync) channel
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! // Create a channel that can hold at most 5 messages at a time.
//! let (tx, rx) = ch::bounded(5);
//!
//! // Can send only 5 messages.
//! for i in 0..5 {
//!     ch!(tx <- i);
//! }
//!
//! // An attempt to send one more message will fail.
//! assert!(tx.try_send(5).is_err());
//! # }
//! ```
//!
//! ## Example: rendevous channel
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (send, recv) = ch::bounded(0);
//! spawn(move || ch!(send <- 5));
//! assert_eq!(ch!(<- recv), 5); // blocks until the previous send occurs
//! # }
//! ```
//!
//! ## Example: the sentinel channel idiom
//!
//! When writing concurrent programs with `ergo`, you will often find that you need
//! to somehow "wait" until some operation is done. For example, let's say you want
//! to run a function in a separate thread, but wait until it completes. Here's
//! one way to do it:
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! fn do_work(done: ch::Sender<()>) {
//!     // do something
//!
//!     // signal that we're done.
//!     ch!(done <- ());
//! }
//!
//! fn main() {
//!     let (sdone, rdone) = ch::bounded(0);
//!     spawn(move || do_work(sdone));
//!     // block until work is done, and then quit the program.
//!     ch!(<- rdone);
//! }
//! ```
//!
//! In effect, we've created a new channel that sends unit values. When we're
//! done doing work, we send a unit value and `main` waits for it to be delivered.
//!
//! Another way of achieving the same thing is to simply close the channel. Once
//! the channel is closed, any previously blocked receive operations become
//! immediately unblocked. What's even cooler is that channels are closed
//! automatically when all senders are dropped. So the new program looks something
//! like this:
//!
//! ```rust
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! fn do_work(_done: ch::Sender<()>) {
//!     // do something
//! }
//!
//! fn main() {
//!     let (sdone, rdone) = ch::bounded(0);
//!     spawn(move || do_work(sdone));
//!     // Block until the channel is closed.
//!     //
//!     // Note: this _expects_ the error that
//!     // all senders have been dropped and will
//!     // panic if a value is sent instead.
//!     ch!(! <- rdone);
//! }
//! ```
//!
//! We no longer need to explicitly do anything with the `_done` channel. We give
//! `do_work` ownership of the channel, but as soon as the function stops
//! executing, `_done` is dropped, the channel is closed and `rdone.recv()`
//! unblocks returning an error, which we expect with `ch!(! <- rdone)`.
//!
//! ## Example: non-blocking sends/receives using `try_send`
//!
//! ```
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (send, _recv) = ch::bounded(0);
//! let data = "send data".to_string();
//! match send.try_send(data) {
//!     Ok(()) => panic!("unexpected"),
//!     Err(ch::TrySendError
//!
//!     println!("Send failed, still own data: {}", data);
//! } else {
//! # }
//! ```
//!
//! ## Example: using `select_loop`
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
//!

pub use crossbeam_channel::{bounded, unbounded, IntoIter, Iter, Receiver, RecvError,
                            RecvTimeoutError, Select, SelectRecvError, SelectSendError, SendError,
                            SendTimeoutError, Sender, TryIter, TryRecvError, TrySendError};
