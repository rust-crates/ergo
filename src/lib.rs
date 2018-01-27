/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! ergo_fs: types for making working with the filesystem ergonomic, therefore fun.
//!
//! ## Purpose
//!
//! This crate provides a minimal set of common types and methods for working with the filesystem.
//! These types aim to provide:
//!
//! - Descriptive error messages
//! - Good performance, but not necessarily at _all_ costs.
//! - As much type safety as is possible when dealing with the filesystem, which can change at any
//!   time.
//!
//! The crates it wraps/rexports are:
//!
//! - [`path_abs`](https://github.com/vitiral/path_abs): Ergonomic paths and files in rust.
//! - [`tar-rs`](https://github.com/alexcrichton/tar-rs): A library for reading and writing TAR
//!   archives.
//! - [`tempdir`](https://github.com/rust-lang-nursery/tempdir): Temporary directories
//!   of files.
//! - [`walkdir`](https://github.com/BurntSushi/walkdir): Provides an efficient and cross platform
//!   implementation of recursive directory traversal.
//! - [`std_prelude`]: prelude that the rust stdlib should have always had.
//!
//! Consider supporting their development individually and starring them on github.
//!
//! ## How to Use
//! ergo_fs is intended to be a "standard library" of filesystem types. Therefore you should
//! use like so:
//!
//! ```
//! extern crate ergo_fs;
//! use ergo_fs::*;
//! # fn try_main() -> ::std::io::Result<()> {
//! # Ok(()) } fn main() { try_main().unwrap() }
//! ```
//!
//! This will also export the [`std_prelude`] crate into your namespace, giving you automatic
//! access to the `io::{Read, Write}` traits, `path::{Path, PathBuf}` types and more.
//!
//! [`std_prelude`]: http://github.com/vitiral/std_prelude
//!
//! # Types
//! This library provides several kinds of types which improve and expand on `std::fs` and
//! `std::path`, as well as provide new functionality like temporary files and tar archives.
//!
//! ## Path, Dir and File Types
//! The following types are exported from [`path_abs`][`path_abs`]. These types provide improved
//! error messages and type safety when working with paths and files. See the [crate documentation]
//! [`path_abs`] for more details.
//!
//! - [`PathArc`](struct.PathArc.html): a reference counted `PathBuf` with methods reimplemented
//!   with better error messages. Use this for a generic serializable path that may or may
//!   not exist.
//! - [`PathAbs`](struct.PathAbs.html): a reference counted absolute (canonicalized) path that is
//!   guaranteed (on initialization) to exist.
//! - [`PathFile`](struct.PathFile.html): a `PathAbs` that is guaranteed to be a file, with
//!   associated methods.
//! - [`PathDir`](struct.PathDir.html): a `PathAbs` that is guaranteed to be a directory, with
//!   associated methods.
//! - [`PathType`](struct.PathType.html): an enum containing either a PathFile or a PathDir.
//!   Returned by [`PathDir::list`][dir_list]
//! - [`FileRead`](struct.FileRead.html): a read-only file handle with `path()` attached and
//!   improved error messages. Contains only the methods and trait implementations which are
//!   allowed by a read-only file.
//! - [`FileWrite`](struct.FileWrite.html): a write-only file handle with `path()` attached and
//!   improved error messages. Contains only the methods and trait implementations which are
//!   allowed by a write-only file.
//! - [`FileEdit`](struct.FileEdit.html): a read/write file handle with `path()` attached and
//!   improved error messages. Contains methods and trait implements for both readable _and_
//!   writeable files.
//!
//! ## Temporary Directories
//!
//! There is one type exported which mimicks the above `Path*` objects.
//! [`PathTmp`](struct.PathTmp.html), which is a `PathDir` that is deleted when it goes out of
//! scope.  This is a wrapper around the crate `tempdir::TempDir` with methods that mimick the
//! `Path` types in this crate.
//!
//! ## Walkdir
//!
//! The [`Walkdir`](struct.WalkDir.html) type is a direct export from the `walkdir` crate.
//! The crate already has excellent error messages, and although it returns the regular
//! `std::path::PathBuf` type, it is easy to convert to a file if you would like to.
//!
//! > TODO: although the WalkDir error can be auto-converted to std::io::Error, it
//! > does not preserve the pretty output. See
//! > [this ticket](https://github.com/BurntSushi/walkdir/pull/91)
//!
//! ### Examples
//! ```rust
//! # extern crate ergo_fs;
//! use ergo_fs::*;
//!
//! # fn try_main() -> ::std::io::Result<()> {
//! let dir = PathDir::new("src")?;
//! for entry in dir.walk().max_depth(1) {
//!     match PathType::from_entry(entry?)? {
//!         PathType::File(file) => println!("got file {}", file.display()),
//!         PathType::Dir(dir) => println!("got dir {}", dir.display()),
//!     }
//! }
//! # Ok(()) } fn main() { try_main().unwrap() }
//! ```
//!
//! ## Tar Files
//! Similarly to walkdir, this is a direct export of the `tar` crate. It is recommended that you
//! use the types like `PathDir` and `FileWrite` when interacting with this crate.
//!
//! > TODO: Add two "80%" methods:
//! >
//! > - `pack_tar(obj: Write, dir: &PathDir, src: Option<Path>)`: tars the `dir` into the `obj`,
//! >   using `walkdir` under the hood for speed and giving pretty error messages for all errors.
//! > - `unpack_tar(obj: Read, dst: &PathDir)`: npacks the contents tarball into the specified dst
//! >   with cleaner error messages.
//! >
//! > Or... maybe consider just pretifying the error messages within the `tar` crate. Maybe they
//! > would be interested in using `path_abs` under the hood?
//!
//! ```rust
//! # extern crate ergo_fs;
//! use ergo_fs::*;
//! use ergo_fs::tar::Builder;
//!
//! # fn try_main() -> ::std::io::Result<()> {
//! // We are going to tar the source code of this library
//!
//! let tmp = PathTmp::create("tmp")?;
//! let mut tarfile = FileWrite::create(tmp.join("src.tar"))?;
//!
//! // tar the source directory
//! let mut tar = Builder::new(tarfile);
//! tar.append_dir_all("src", ".")?;
//! tar.finish();
//! let tarfile = tar.into_inner()?;
//!
//! // A tarfile now exists, do whatever you would like with it.
//! # Ok(()) } fn main() { try_main().unwrap() }
//! ```

pub extern crate std_prelude;
pub extern crate tempdir;
pub extern crate path_abs;

// -------------------------------
// External Crate Exports
pub extern crate tar;
pub extern crate walkdir;

pub use std_prelude::*;
pub use path_abs::{PathAbs, PathArc, PathFile, PathDir, PathType, FileRead, FileWrite, FileEdit};
pub use walkdir::{WalkDir, Error as WalkError};

// -------------------------------
// Local Modules and Exports

mod tmp;
pub use tmp::PathTmp;

/// Extension method on the `Path` type.
pub trait PathDirExt
    where Self: AsRef<Path>
{
    /// Walk the `PathDir`, returning the `WalkDir` builder.
    ///
    /// # Examples
    /// ```rust
    /// # extern crate ergo_fs;
    /// use ergo_fs::*;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// let dir = PathDir::new("src")?;
    /// for entry in dir.walk().max_depth(1) {
    ///     match PathType::from_entry(entry?)? {
    ///         PathType::File(file) => println!("got file {}", file.display()),
    ///         PathType::Dir(dir) => println!("got dir {}", dir.display()),
    ///     }
    /// }
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    fn walk(&self) -> walkdir::WalkDir {
        walkdir::WalkDir::new(&self)
    }
}

/// Extended methods for `PathType`
pub trait PathTypeExt {
    /// Create a `PathType` from a `walkdir::DirEntry` using fewer syscalls.
    ///
    /// See [`PathDir::walk`]
    ///
    /// [`PathDir::walk`]: trait.PathDirExt.html#method.walk
    fn from_entry(entry: walkdir::DirEntry) -> path_abs::Result<PathType> {
        let abs = PathAbs::new(entry.path())?;
        let ty = entry.file_type();
        if ty.is_file() {
            Ok(PathType::File(PathFile::from_abs_unchecked(abs)))
        } else if ty.is_dir() {
            Ok(PathType::Dir(PathDir::from_abs_unchecked(abs)))
        } else {
            // it is a symlink and we _must_ use a syscall to resolve the type.
            PathType::from_abs(abs)
        }
    }
}

impl PathDirExt for PathDir {}
impl PathTypeExt for PathType {}
