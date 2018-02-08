//! **ergo_std**: items that could be in the standard library.
//!
//! This is the "core types" library as part of the [`ergo`] crates ecosystem. It contains useful
//! types, traits and functions for general purpose programming projects which do not fall
//! into the other [`ergo`] crates but which are boons to ergonomics and productivity.
//!
//! # How to Use
//!
//! In your `Cargo.toml`
//!
//! ```toml,no_compile
//! [dependencies]
//! ergo_std = "0.1"
//! serde = "1.0"
//! serde_derive = "1.0"
//! ```
//!
//! > You have to put the other crates in your `Cargo.toml` in order for `#[derive(...)]` to work
//! > correctly.
//!
//! ```rust
//! #[macro_use] extern crate ergo_std;
//! use ergo_std::*;
//! fn main() {
//!     /* Your code goes here */
//! }
//! ```
//!
//! # Exported Items
//!
//! The following crates and types are exported. See their docs for how to use them.
//!
//! - **[`std_prelude`]**: extends rust's `std::prelude` with commonly used types. The
//!   crate is well documented with justification and usecases for each type.
//! - **[`serde`]**: the defacto serialization library of rust. Also imports `serde_derive`
//!   so you can use `#[derive(Serialize, Deserialize)]`.
//! - **[`lazy_static!`]**: the `lazy_static!` macro is the current standard way to create
//!   global variables and constants. Warning that they are created lazily (at run time)!
//! - **[`itertools`]**: the itertools prelude provides traits that extends rust's already
//!   extensive iterator API.
//! - **[`maplit`]**: provides `hashmap!`, `hashset!`, `btreemap!` and `btreeset!` macros to
//!   compliment rust's existing `vec!` macro. These
//! - **[`Regex`]**: the regular expression type from the `regex` crate.
//!
//! [`ergo`]: https://github.com/rust-crates/ergo
//! [`std_prelude`]: ../std_prelude/index.html
//! [`itertools`]: ../itertools/index.html
//! [`lazy_static!`]: ../lazy_static/index.html
//! [`maplit`]: ../maplit/index.html
//! [`Regex`]: struct.Regex.html
//!
//! ### Special thanks
//!
//! The crates that are exported are:
//!
//! - [**serde**](https://github.com/serde-rs/serde): Serialization framework for Rust
//! - [**std_prelude**](https://github.com/vitiral/std_prelude): prelude that the rust stdlib
//!   should have always had
//! - [**expect_macro**](https://github.com/vitiral/expect_macro): The `expect!` macro
//! - [**lazy_static**](https://github.com/rust-lang-nursery/lazy-static.rs): A small macro for
//!   defining lazy evaluated static variables in Rust.
//! - [**itertools**](https://github.com/bluss/rust-itertools): Extra iterator adaptors, iterator
//!   methods, free functions, and macros.
//! - [**maplit**](https://github.com/bluss/maplit): Rust container / collection literal macros for
//!   HashMap, HashSet, BTreeMap, BTreeSet.
//! - [**regex**](https://github.com/rust-lang/regex): An implementation of regular expressions for
//!   Rust. This implementation uses finite automata and guarantees linear time matching on all
//!   inputs.
//!
//! Consider supporting their development individually and starring them on github.
#![allow(unused_imports)]

#[macro_use]
pub extern crate expect_macro;
#[macro_use]
pub extern crate itertools;
#[macro_use]
pub extern crate lazy_static;
#[macro_use]
pub extern crate maplit;
pub extern crate std_prelude;
pub extern crate regex;
pub extern crate serde;
#[macro_use]
pub extern crate serde_derive;

pub use expect_macro::*;
pub use std_prelude::*;
pub use lazy_static::*;
pub use itertools::Itertools;
pub use maplit::*;
pub use regex::Regex;
pub use serde::*;
pub use serde_derive::*;

