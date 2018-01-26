//! ergo_fs: types for making working with the filesystem ergonomic, therefore fun.
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
//! - [`path_abs`][TODO]: Ergonomic paths and files in rust.
//! - [`tar`][TODO]: A library for reading and writing TAR archives
//! - [`tempdir`][TODO]: Temporary directories of files.
//! - [`walkdir`][TODO]: Provides an efficient and cross platform implementation of recursive
//!   directory traversal
//! - TODO: the file-lock crate
//!
//! Consider supporting their development individually and starring them on github.
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
//! > note: `PathTmp` is a wraper around `tempdir::TempDir`
//!
//! ## Walkdir
//!
//! The [`Walkdir`](struct.WalkDir.html) type is a direct export from the `walkdir` crate.
//! The crate already has excellent error messages, and although it returns the regular
//! `std::path::PathBuf` type, it is easy to convert to a file if you would like to.
//!
//! > TODO: although the WalkDir error can be auto-converted to std::io::Error, it
//! > does not preserve the pretty output. Open a ticket and PR to fix.
//!
//! ### Examples
//! ```rust
//! use ergo_fs::{WalkDir, PathType};
//!
//! # fn try_main() -> Result<(), ::std::io::Error> {
//! for entry in WalkDir::new("foo").min_depth(1) {
//!     match PathType::new(entry?)? {
//!         PathType::File(file) => println!("got file {}", file.display()),
//!         PathType::Dir(dir) => println!("got dir {}", dir.display()),
//!     }
//! }
//! # Ok(())
//! # }
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
//! use ergo_fs::{PathTmp, FileWrite};
//! use ergo_fs::tar::Builder;
//!
//! # fn try_main() -> Result<(), ::std::io::Error> {
//! // We are going to tar the source code of this library
//!
//! let tmp = PathTmp::create("tmp")?;
//! let mut tarfile = PathFile::create(tmp.join("src.tar"))?;
//!
//! // tar the source directory
//! let mut tar = Builder::new(tarfile)
//! tar.append_dir_all("src", ".")?;
//! tar.finish();
//! let tarfile = tar.into_inner();
//! tarfile.flush();
//!
//! // A tarfile now exists, do whatever you would like with it.
//! # Ok(())
//! # }
//! ```

extern crate std_prelude;
extern crate tempdir;
extern crate path_abs;

// -------------------------------
// External Crate Exports
pub extern crate tar;
pub extern crate walkdir;

pub use path_abs::{PathAbs, PathArc, PathFile, PathDir, PathType, FileRead, FileWrite, FileEdit};
pub use walkdir::{WalkDir, Error as WalkError};

// -------------------------------
// Local Modules and Exports
mod tmp;
pub use tmp::PathTmp;
