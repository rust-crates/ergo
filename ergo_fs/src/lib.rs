//! ergo_fs: types for making working with the filesystem ergonomic, therefore fun.
//!
//! This crate provides a minimal set of common types and modules for working with the filesystem.
//!
//! # Types
//! This library provides several kinds of types which improve and expand on `std::fs` and
//! `std::path`, as well as provide new functionality like temporary files and tar archives.
//!
//! ## Path and File Types
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
//! - [`PathTmp`](struct.PathTmp.html): a `PathDir` that is deleted when it goes out of scope.
//!   This is a wrapper around the crate `tempdir::TempDir` with methods that mimick the
//!   `Path` types in this crate.
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

extern crate std_prelude;
pub extern crate path_abs;
pub extern crate tar;
extern crate tempdir;
pub extern crate walkdir;

mod tmp;

pub use path_abs::{PathAbs, PathArc, PathFile, PathDir, PathType, FileRead, FileWrite, FileEdit};
pub use tmp::PathTmp;
