/* Copyright (c) 2018 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! Define the deepcopy function
use super::*;
use std::fs;
use std::io;

/// Do a deep copy of a directory from one location to another.
///
/// This will follow symlinks and copy the _contents_. Recursive paths will cause an
/// error to be raised.
///
/// Errors are sent over the `send_err` channel.
pub fn deep_copy<P: AsRef<Path>>(send_err: Sender<io::Error>, from: PathDir, to: P) {
    let to = ch_try!(
        send_err,
        create_dir_maybe(to).map_err(|err| err.into()),
        return
    );

    let (send_file, recv_file) = ch::bounded(128);

    // First thread walks and creates directories, and sends files to copy
    take!(=send_err as errs, =to as to_walk);
    spawn(move || {
        walk_and_create_dirs(from, to_walk, errs, send_file);
    });

    // Threadpool copy files into directories that are pre-created.
    for _ in 0..num_cpus::get() {
        take!(=send_err, =recv_file, =to);
        spawn(move || {
            for (from, to_postfix) in recv_file {
                ch_try!(
                    send_err,
                    from.copy(to.join(to_postfix)).map_err(|err| err.into()),
                    continue
                );
            }
        });
    }
}

/// Do a contents-first yeild and follow any symlinks -- we are doing an _actual_ copy
fn walk_and_create_dirs(
    from: PathDir,
    to: PathDir,
    send_err: Sender<io::Error>,
    send_file: Sender<(PathFile, PathBuf)>,
) {
    let mut it = from.walk().follow_links(true).into_iter();
    loop {
        let entry = match it.next() {
            Some(entry) => entry,
            None => break,
        };
        macro_rules! handle_err {
            ($entry:expr) => {
                match $entry {
                    Ok(e) => e,
                    Err(err) => {
                        ch!(send_err <- err.into());
                        continue;
                    }
                }
            };
        }
        let entry = handle_err!(entry);
        let to_postfix = entry
            .path()
            .strip_prefix(&from)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
        let to_postfix = handle_err!(to_postfix);

        match handle_err!(PathType::new(entry.path())) {
            PathType::Dir(_) => {
                // Create it immediately
                if let Err(err) = PathDir::create(to.join(to_postfix)) {
                    ch!(send_err <- err.into());
                    // We couldn't create the directory so it needs to be skipped.
                    it.skip_current_dir();
                }
            }
            PathType::File(from_file) => {
                ch!(send_file <- (from_file, to_postfix.to_path_buf()));
            }
        }
    }
}

fn create_dir_maybe<P: AsRef<Path>>(path: P) -> path_abs::Result<PathDir> {
    let path = path.as_ref();
    fs::create_dir(path)
        .map_err(|err| path_abs::Error::new(err, "creating dir", PathBuf::from(path).into()))?;
    PathDir::new(path)
}
