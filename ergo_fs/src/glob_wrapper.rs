/* Much of the documentation in this file was taken from the `glob` crate.
 *
 * Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 * Copyright (c) 2014 The Rust Project Developers.
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Wrapper around the `glob` crate.

use std::io;
use std_prelude::*;
use path_abs::{PathDir, PathFile, PathType};
use glob_crate;

/// Renamed [`glob::MatchOptions`](../glob/struct.MatchOptions.html)
pub type GlobOptions = glob_crate::MatchOptions;

/// Renamed [`glob::PatternError`](../glob/struct.PatternError.html)
pub type GlobPatternError = glob_crate::PatternError;

#[inline(always)]
/// Return an iterator that produces all the `PathType`s that match the given pattern, which may be
/// absolute or relative to the current working directory.
///
/// This may return an error if the pattern is invalid.
///
/// This method uses the default match options and is equivalent to calling
/// `glob_with(pattern, GlobOptions::new())`. Use [`glob_with`](fn.glob_with.html) directly if you
/// want to use non-default match options.
///
/// When iterating, each result is a `io::Result` which expresses the possibility that there was an
/// `io::Error` when attempting to read the contents of the matched path.
///
/// # Example
///
/// ```rust
/// # extern crate ergo_fs;
/// use ergo_fs::*;
///
/// # fn try_main() -> ::std::io::Result<()> {
/// let mut count = 0;
/// for entry in glob("src/glob_*.rs").unwrap() {
///     # count += 1; assert!(entry.is_ok());
///     match entry? {
///         PathType::File(file) => println!("file: {}", file.display()),
///         PathType::Dir(dir) => println!("dir: {}", dir.display()),
///     }
/// }
/// # assert_eq!(count, 1);
/// # Ok(()) } fn main() { try_main().unwrap() }
/// ```
///
/// The above code will print:
///
/// ```ignore
/// /path/to/crate/src/glob_wrapper.rs
/// ```
///
/// If there were more files with the prefix `glob_` it would print more.
pub fn glob(pattern: &str) -> Result<GlobPathTypes, GlobPatternError> {
    GlobPathTypes::new(pattern)
}

#[inline(always)]
/// The same as [`glob`](fn.glob.html) but with additional options.
pub fn glob_with(pattern: &str, options: &GlobOptions) -> Result<GlobPathTypes, GlobPatternError> {
    GlobPathTypes::with(pattern, options)
}

/// An iterator that yields `PathType`s from the filesystem that match a particular pattern.
///
/// Note that it yields `Result<PathType, path_abs::Error>` in order to report any IoErrors that
/// may arise during iteration.  If a directory matches but is unreadable, thereby preventing its
/// contents from being checked for matches, a `path_abs::Error` is returned to express this.
///
/// See the [`glob`](fn.glob.html) function for more details.
pub struct GlobPathTypes {
    paths: glob_crate::Paths,
}

/// Returns an iterator of only `PathFile`s, any directories that matched the glob are ignored.
///
/// See [`GlobPathTypes::files`](struct.GlobPathTypes.html#method=files)
pub struct GlobPathFiles {
    types: GlobPathTypes,
}

/// Returns an iterator of only `PathDirs`s, any files that matched the glob are ignored.
///
/// See [`GlobPathTypes::dirs`](struct.GlobPathTypes.html#method=dirs)
pub struct GlobPathDirs {
    types: GlobPathTypes,
}

impl GlobPathTypes {
    #[inline(always)]
    fn new(pattern: &str) -> Result<GlobPathTypes, glob_crate::PatternError> {
        Ok(GlobPathTypes {
            paths: glob_crate::glob(pattern)?,
        })
    }

    #[inline(always)]
    fn with(
        pattern: &str,
        options: &GlobOptions,
    ) -> Result<GlobPathTypes, glob_crate::PatternError> {
        Ok(GlobPathTypes {
            paths: glob_crate::glob_with(pattern, options)?,
        })
    }

    #[inline(always)]
    /// Consume self and return an iterator over only the files, ignoring any directories.
    ///
    /// # Example
    /// ```rust
    /// # extern crate ergo_fs;
    /// use ergo_fs::*;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// # let mut count = 0;
    /// // unwrap since we know we are inputing a good pattern
    /// for file in glob("src/glob*.rs").unwrap().files() {
    ///     println!("file: {}", file?.display());
    ///     # count += 1;
    /// }
    /// # assert_eq!(count, 1);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn files(self) -> GlobPathFiles {
        GlobPathFiles { types: self }
    }

    #[inline(always)]
    /// Consume self and return an iterator over only the directories, ignoring any files.
    ///
    /// # Example
    /// ```rust
    /// # extern crate ergo_fs;
    /// use ergo_fs::*;
    ///
    /// # fn try_main() -> ::std::io::Result<()> {
    /// # let mut count = 0;
    /// // unwrap since we know we are inputing a good pattern
    /// for dir in glob("src/*").unwrap().dirs() {
    ///     println!("dir: {}", dir?.display());
    ///     # count += 1;
    /// }
    /// # assert_eq!(count, 0);
    /// # Ok(()) } fn main() { try_main().unwrap() }
    /// ```
    pub fn dirs(self) -> GlobPathDirs {
        GlobPathDirs { types: self }
    }
}

impl Iterator for GlobPathTypes {
    type Item = io::Result<PathType>;
    // FIXME: if we can get an owned value of the io::Error then we can
    // make this return path_abs::Error
    fn next(&mut self) -> Option<io::Result<PathType>> {
        if let Some(result) = self.paths.next() {
            match result {
                Ok(path) => Some(PathType::new(path).map_err(|err| err.into())),
                Err(err) => Some(Err(io::Error::new(err.error().kind(), err))),
            }
        } else {
            None
        }
    }
}

impl Iterator for GlobPathFiles {
    // FIXME: make path_abs::Error
    type Item = io::Result<PathFile>;
    fn next(&mut self) -> Option<io::Result<PathFile>> {
        loop {
            match self.types.next() {
                Some(Ok(ty)) => match ty {
                    PathType::File(file) => return Some(Ok(file)),
                    // ignore directories
                    PathType::Dir(_) => {}
                },
                Some(Err(err)) => return Some(Err(err)),
                // iterator exahusted
                None => return None,
            }
        }
    }
}

impl Iterator for GlobPathDirs {
    // FIXME: make path_abs::Error
    type Item = io::Result<PathDir>;
    fn next(&mut self) -> Option<io::Result<PathDir>> {
        loop {
            match self.types.next() {
                Some(Ok(ty)) => match ty {
                    PathType::Dir(dir) => return Some(Ok(dir)),
                    // ignore files
                    PathType::File(_) => {}
                },
                Some(Err(err)) => return Some(Err(err)),
                None => return None, // iterator exahusted
            }
        }
    }
}
