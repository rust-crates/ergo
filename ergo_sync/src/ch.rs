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
//! ## Example: non-blocking sends/receives
//!
//! ```
//! #[macro_use] extern crate ergo_sync;
//! use ergo_sync::*;
//!
//! # fn main() {
//! let (send, recv) = ch::bounded(1);
//! let data = "send data".to_string();
//! match ch!(send <-? data) {
//!     Some(data) => {
//!         println!("didn't send data, but got it back: {}", data);
//!         unreachable!(); // in this case we don't expect it
//!     }
//!     None => println!("message sent successfully"),
//! }
//!
//! // attempting to send additional data fails
//! let data = "more data".to_string();
//! assert_eq!(Some(data.clone()), ch!(send <-? data));
//!
//! match ch!(<-? recv) {
//!     Some(data) => println!("received data: {}", data),
//!     None => {
//!         println!("didn't receive any data yet");
//!         unreachable!(); // in this case we don't expect it
//!     }
//! }
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

/// Use with channels with ergonomic syntax and panic with helpful error messages when
/// sending/receiving on a channel is invalid.
///
///   - `ch!(send <- 42)` for sending a value.
///   - `let v = ch!(<- recv)` for receiving a value.
///   - `ch!(! <- recv)` to wait for channels to close.
///   - `<-?` for async operation support.
///
/// **Blocking syntax:**
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
/// **Non-Blocking syntax:**
///
/// - `ch!(send <-? value)`: returns `None` if the value was sent, `Some(value)` if the value
///   was not sent. Panics if all receivers are dropped.
/// - `ch!(<-? recv)`: returns `None` if no value is received, `Some(value)` if a value is
///   received. Panics if all senders are dropped.
/// - `ch!(! <-? recv)`: returns `true` if there are still senders and `false` if the seners have
///   been dropped. Panics if a value is received. Use with `while ch!(! <-? recv) { /* ... */ }`
///
/// > Non-Blocking syntax does _not_ work with `std::mspc` channels.
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
            Err($crate::ch::TrySendError::Full(v)) => Some(v),
            Err($crate::ch::TrySendError::Disconnected(_)) => {
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
            Err($crate::ch::TryRecvError::Empty) => None,
            Err($crate::ch::TryRecvError::Disconnected) => {
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
            Err($crate::ch::TryRecvError::Empty) => true,  // senders still exist
            Err($crate::ch::TryRecvError::Disconnected) => false, // no more senders
        }
    };
    [! <- $recv:ident] => {
        match $recv.recv() {
            Ok(v) => panic!("Got {:?} when expecting senders to be closed.", v),
            Err(err) => (),
        }
    };
}

/// Handle an expression that could be `Err` and send it over a channel if it is.
///
/// This is the same as the builtin `try!` macro, except if the expression fails than the `Err` is
/// sent on the `$send` channel and the requested action is performed.
///
/// Suggested possible actions:
/// - `continue`
/// - `return`
/// - `break`
/// - some expression that evaluates to a "default value" for that context.
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
///     let v = ch_try!(send_err, *item, continue);
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
    [$send:ident, $expr:expr, $action:expr] => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                ch!($send <- e);
                $action
            }
        }
    };
}
