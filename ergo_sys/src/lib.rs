//! **ergo_sys: make accessing system resources more ergonomic, therefore more fun!**
//!
//! This is the system/os library as part of the [`ergo`] crate ecosystem.
//!
//! TODO: flush out these docs more
//!
//! - add example of using ergo_sync with ctrlc.
//! - talk about rand a bit and the different kinds of types and their usecases.
//!
//! [`ergo`]: https://github.com/rust-crates/ergo
//!
//! ### Special thanks
//!
//! The crates that are exported are:
//!
//! - [**ctrlc**](https://github.com/Detegr/rust-ctrlc): Easy Ctrl-C handler for Rust projects
//! - [**rand**](https://github.com/rust-lang-nursery/rand): A Rust library for random number
//!   generators and other randomness functionality.
//!
//! Consider supporting their development individually and starring them on github.

pub extern crate rand;
pub extern crate ctrlc;
