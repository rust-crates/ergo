/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! **Make rust's ecosystem more ergonomic, therefore more fun!**
//!
//! _This crate is in alpha status, please see the github project for
//! more details_
//!
//! https://github.com/rust-crates/ergo
#![allow(unused_imports)]

#[macro_use]
pub extern crate ergo_config;
#[macro_use]
pub extern crate ergo_fs;
#[macro_use]
pub extern crate ergo_std;
#[macro_use]
pub extern crate ergo_sync;
#[macro_use]
pub extern crate ergo_sys;

pub use ergo_config::*;
pub use ergo_fs::*;
pub use ergo_std::*;
pub use ergo_sync::*;
pub use ergo_sys::*;

mod deep_copy;
pub use deep_copy::deep_copy;
