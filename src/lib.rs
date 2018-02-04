//! **Make loading configuration more ergonomic, therefore fun!**
//!
//! This is the configuration loading libray as part of the [`ergo`] crate ecosystem.
//!
//! > This library should rarely be used on its own. Refer to the [`ergo`] crate ecosystem
//! > for how to use its exported features. For one thing it depends on `serde` to actually
//! > be used in most cases.
//!
//! ### Special thanks
//!
//! The crates that are exported are:
//!
//! - [**configure**](https://github.com/withoutboats/configure): pull in configuration from the
//!   environment.
//! - [**ron**](https://github.com/ron-rs/ron): Rusty Object Notation.
//! - [**serde_json**](https://github.com/serde-rs/json): Strongly typed JSON library for Rust.
//! - [**serde_yaml**](https://github.com/dtolnay/serde-yaml): Strongly typed YAML library for
//!   Rust.
//! - [**toml**](https://github.com/alexcrichton/toml-rs): A TOML encoding/decoding library for
//!   Rust.
//!
//! Consider supporting their development individually and starring them on github.
//!
//! [`ergo`]: https://github.com/rust-crates/ergo
#![allow(unused_imports)]

#[macro_use]
pub extern crate configure;
pub extern crate serde_json as json;
pub extern crate serde_yaml as yaml;
pub extern crate toml;

pub use configure::*;
