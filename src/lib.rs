//! **ergo_std**: items that could be in the standard library.
//!
//! This is the "core types" library as part of the [`ergo`] crates ecosystem. It contains useful
//! types, traits and functions for general purpose programming projects which do not fall
//! into the other [`ergo`] crates but which are boons to ergonomics and productivity.
//!
//! # How to Use
//!
//! ```rust
//! #[macro_use] extern crate failure;
//! extern crate serde;
//! #[macro_use] extern crate serde_derive;
//! #[macro_use] extern crate ergo_std;
//! use ergo_std::*;
//! # fn main() {
//! # }
//! ```
//!
//! > _As you notice, this crate does not include `serde` or `failure`. This is due to a bug
//! > which makes it impossible for this crate to rexport their `#[derive(...)]` macros.
//!
//! # Exported Items
//!
//! The following crates and types are exported. See their docs for how to use them.
//!
//! - **[`std_prelude`]**: extends rust's additional prelude with commonly used types. The
//!   crate is well documented with justification and usecases for each type.
//! - **[`lazy_static!`]**: the `lazy_static!` macro is the current standard way to create
//!   global variables and constants in a majority of crates.
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
//! - [**std_prelude**](https://github.com/vitiral/std_prelude):
//!   Multi-producer multi-consumer channels for message passing
//! - [**lazy_static**](TODO): TODO
//! - [**itertools**](TODO): TODO
//! - [**maplit**](TODO): TODO
//! - [**regex**](TODO): TODO
//!
//! Consider supporting their development individually and starring them on github.
//!
//! ## Future crates
//!
//! The following crates will be added in the future:
//!
//! - `indexmap`: the current crate is `ordermap`, which is renaming itself
//!   `indexmap` and changing what `ordermap` is... it's confusing but it
//!   will be comming shortly

#[macro_use]
pub extern crate itertools;
#[macro_use]
pub extern crate lazy_static;
#[macro_use]
pub extern crate maplit;
pub extern crate std_prelude;
pub extern crate regex;

pub use std_prelude::*;
pub use itertools::prelude::*;
pub use lazy_static::*;
pub use itertools::Itertools;
pub use maplit::*;
pub use regex::Regex;

