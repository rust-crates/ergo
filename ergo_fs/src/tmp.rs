// Almost all of this code is copy/pasted from the tempdir crate.
//
// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::fs;
use std::io;

use std_prelude::*;
use tempdir;
use path_abs::{PathArc, PathAbs, PathDir};

/// A `PathDir` that is automatically deleted when it goes out of scope.
///
/// > **Unlike the other `Path` types, this type is not cloneable.**
///
/// The [`PathTmp`] type creates a directory on the file system that is deleted once it goes out of
/// scope. At construction, the `PathTmp` creates a new directory with a randomly generated name,
/// and with a prefix of your choosing.
///
/// The default constructor, [`PathTmp::create`], creates directories in the location returned by
/// [`std::env::temp_dir()`], but `PathTmp` can be configured to manage a temporary directory in
/// any location by constructing with [`PathTmp::create_in`].
///
/// After creating a `PathTmp`, work with the file system by doing standard [`std::fs`] file system
/// operations on its [`Path`], which can be retrieved with [`PathTmp::path`]. Once the `PathTmp`
/// value is dropped, the directory at the path will be deleted, along with any files and
/// directories it contains. It is your responsibility to ensure that no further file system
/// operations are attempted inside the temporary directory once it has been deleted.
///
/// Various platform-specific conditions may cause `PathTmp` to fail to delete the underlying
/// directory. It's important to ensure that handles (like [`File`] and [`ReadDir`]) to files
/// inside the directory are dropped before the `PathTmp` goes out of scope. The `PathTmp`
/// destructor will silently ignore any errors in deleting the directory; to instead handle errors
/// call [`PathTmp::close`].
///
/// Note that if the program exits before the `PathTmp` destructor is run, such as via
/// [`std::process::exit`], by segfaulting, or by receiving a signal like `SIGINT`, then the
/// temporary directory will not be deleted.
///
/// [`File`]: http://doc.rust-lang.org/std/fs/struct.File.html
/// [`Path`]: http://doc.rust-lang.org/std/path/struct.Path.html
/// [`ReadDir`]: http://doc.rust-lang.org/std/fs/struct.ReadDir.html
/// [`PathTmp::close`]: struct.PathTmp.html#method.close
/// [`PathTmp::create`]: struct.PathTmp.html#method.new
/// [`PathTmp::create_in`]: struct.PathTmp.html#method.new_in
/// [`PathTmp::path`]: struct.PathTmp.html#method.path
/// [`PathTmp`]: struct.PathTmp.html
/// [`std::env::temp_dir()`]: https://doc.rust-lang.org/std/env/fn.temp_dir.html
/// [`std::fs`]: http://doc.rust-lang.org/std/fs/index.html
/// [`std::process::exit`]: http://doc.rust-lang.org/std/process/fn.exit.html
pub struct PathTmp {
    /// The reference to the absolute path
    dir: PathDir,
    /// The reference to the temporary file
    tmp: tempdir::TempDir,
}

impl PathTmp {
    /// Attempts to make a temporary directory inside of `env::temp_dir()` whose name will have the
    /// prefix, `prefix`. The directory and everything inside it will be automatically deleted once
    /// the returned `PathTmp` is destroyed.
    ///
    /// # Errors
    ///
    /// If the directory can not be created, `Err` is returned.
    ///
    /// # Examples
    /// ```
    /// use ergo_fs::{PathFile, PathTmp};
    ///
    /// let tmp_dir = PathTmp::create("example").unwrap()
    /// let file = PathFile::create(tmp_dir.join("temporary-note.txt")).unwrap();
    /// let message = "This file existed, but only for a moment.";
    /// file.write_str(message).unwrap();
    /// assert_eq!(file.read_string(), message);
    ///
    /// // Close the tmp_dir manually (would automatically happen when dropped).
    /// // All contents are automatically deleted.
    /// drop(tmp_dir.close().unwrap();
    /// assert!(!file.exists());
    /// ```
    pub fn create(prefix: &str) -> io::Result<PathTmp> {
        PathTmp::create_in(&env::temp_dir(), prefix)
    }

    /// Attempts to create a temporary directory inside of `base` whose name will have the prefix
    /// `prefix`. The created directory and everything inside it will be automatically deleted once
    /// the returned `PathTmp` is destroyed.
    ///
    /// # Errors
    ///
    /// If the directory can not be created, `Err` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use ergo_fs::{PathFile, PathTmp};
    ///
    /// let tmp_dir = PathTmp::create_in(".", "example").unwrap()
    /// let file = PathFile::create(tmp_dir.join("temporary-note.txt")).unwrap();
    /// let message = "This file existed, but only for a moment.";
    /// file.write_str(message).unwrap();
    /// assert_eq!(file.read_string(), message);
    ///
    /// // Close the tmp_dir manually (would automatically happen when dropped).
    /// // All contents are automatically deleted.
    /// drop(tmp_dir.close().unwrap();
    /// assert!(!file.exists());
    /// ```
    pub fn create_in<P: AsRef<Path>>(base: P, prefix: &str) -> io::Result<PathTmp> {
        let tmp = TempDir::new_in(&base, prefix).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when creating tmpdir in {}", err, base),
            )
        })?;

        Ok(PathTmp {
            dir: PathDir::new(tmp.path())?,
            tmp: tmp,
        })
    }

    /// Persist the temporary directory on the file system.
    ///
    /// This method consumes `self`, returning the location of the temporary directory as a regular
    /// `PathDir`. The directory will no longer be automatically deleted.
    ///
    /// # Examples
    ///
    /// ```
    /// use ergo_fs::PathTmp;
    ///
    /// let tmp_dir = PathTmp::create_in(".", "persist").unwrap()
    /// let dir = tmp_dir.persisit();
    ///
    /// // The directory is now persisted to disk
    /// assert!(dir.exists());
    ///
    /// // It can still be manually removed though.
    /// dir.remove().unwrap();
    /// ```
    pub fn persist(mut self) -> PathDir {
        self.tmp.into_path();
        self.dir
    }

    /// Closes and removes the temporary directory, returing a `Result`.
    ///
    /// Although `PathTmp` removes the directory on drop, in the destructor
    /// any errors are ignored. To detect errors cleaning up the temporary
    /// directory, call `close` instead.
    ///
    /// # Errors
    ///
    /// This function may return a variety of [`std::io::Error`]s that result from deleting
    /// the files and directories contained with the temporary directory,
    /// as well as from deleting the temporary directory itself. These errors
    /// may be platform specific.
    ///
    /// [`std::io::Error`]: http://doc.rust-lang.org/std/io/struct.Error.html
    pub fn close(mut self) -> io::Result<()> {
        self.tmp.close().map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("{} when removing {}", err, self.dir),
            )
        })
    }
}

impl fmt::Debug for PathTmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.dir.fmt(f)
    }
}

impl Hash for PathTmp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dir.hash(state);
    }
}

impl AsRef<PathDir> for PathTmp {
    fn as_ref(&self) -> &PathAbs {
        &self.dir
    }
}

impl AsRef<PathAbs> for PathTmp {
    fn as_ref(&self) -> &PathAbs {
        self.dir.as_ref()
    }
}

impl AsRef<Path> for PathTmp {
    fn as_ref(&self) -> &Path {
        self.dir.as_ref()
    }
}

impl AsRef<PathBuf> for PathTmp {
    fn as_ref(&self) -> &PathBuf {
        self.dir.as_ref()
    }
}

impl Deref for PathTmp {
    type Target = PathAbs;

    fn deref(&self) -> &PathAbs {
        &self.dir
    }
}

impl Into<PathAbs> for PathTmp {
    /// Downgrades the `PathTmp` into a `PathAbs`
    fn into(self) -> PathAbs {
        self.dir.into()
    }
}

impl Into<PathArc> for PathTmp {
    /// Downgrades the `PathTmp` into a `PathArc`
    fn into(self) -> PathArc {
        self.dir.into()
    }
}

impl Into<PathBuf> for PathTmp {
    /// Downgrades the `PathTmp` into a `PathBuf`. Avoids a clone if this is the only reference.
    fn into(self) -> PathBuf {
        self.dir.into()
    }
}
