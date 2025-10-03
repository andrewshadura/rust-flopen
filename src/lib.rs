// Copyright (C) 2021 Andrej Shadura
// SPDX-License-Identifier: MIT
use nix::fcntl::{flock, FlockArg};
use std::fs::{metadata, File, OpenOptions};
use std::io::Result;
use std::os::unix::{fs::MetadataExt, io::AsRawFd};
use std::path::Path;

/// This trait provides a way to reliably open and lock a file
///
/// `OpenAndLock` trait provides methods implementing the algorithm of the
/// [`flopen`][] function available on BSD systems. It is roughly equivalent
/// to opening a file and calling [`flock`][] with an `operation` argument
/// set to `LOCK_EX`, but it also attempts to detect and handle races between
/// opening or creating the file and locking it. This makes it well-suited
/// for opening lock files, PID files, spool files, mailboxes and other kinds
/// of files which are used for synchronisation between processes.
///
/// This trait extends [`OpenOptions`], so it can be used the following way:
/// ```no_run
/// # use flopen::OpenAndLock;
/// # use std::fs::OpenOptions;
/// let file = OpenOptions::new()
///     .read(true)
///     .write(true)
///     .create(true)
///     .open_and_lock("/path/to/file")?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// [`flopen`]: https://manpages.debian.org/flopen
/// [`flock`]: https://manpages.debian.org/2/flock
pub trait OpenAndLock {
    /// Open and lock a file.
    ///
    /// Opens a file and locks it in an exclusive mode, blocking until the lock
    /// is possible. Retries if the file disappeared or recreated immediately after
    /// locking.
    ///
    /// This method waits until the file can be locked, so unless an unrelated I/O
    /// error occurs, it will eventually succeed once the file has been released
    /// if it’s been held by a different process.
    fn open_and_lock<P: AsRef<Path>>(&self, path: P) -> Result<File>;

    /// Try to open and lock a file.
    ///
    /// Opens a file and locks it in an exclusive mode, failing with
    /// [`std::io::ErrorKind::WouldBlock`] if the lock is not possible.
    /// Retries if the file disappeared or recreated immediately after
    /// locking.
    ///
    /// This method returns an error immediately when the file cannot be
    /// locked, allowing the called to handle it and retry if necessary.
    fn try_open_and_lock<P: AsRef<Path>>(&self, path: P) -> Result<File>;
}

fn open_and_lock<P: AsRef<Path>>(
    options: &OpenOptions,
    path: P,
    lock_mode: FlockArg,
) -> Result<File> {
    loop {
        let file = options.open(&path)?;
        flock(file.as_raw_fd(), lock_mode)?;
        if let Ok(metadata_at_path) = metadata(&path) {
            let file_metadata = file.metadata()?;
            if metadata_at_path.dev() != file_metadata.dev()
                || metadata_at_path.ino() != file_metadata.ino()
            {
                continue;
            }
            return Ok(file);
        } else {
            continue;
        }
    }
}

impl OpenAndLock for OpenOptions {
    fn open_and_lock<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        open_and_lock(self, path, FlockArg::LockExclusive)
    }

    fn try_open_and_lock<P: AsRef<Path>>(&self, path: P) -> Result<File> {
        open_and_lock(self, path, FlockArg::LockExclusiveNonblock)
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenAndLock;
    use std::fs::OpenOptions;
    use std::io;
    use tempfile::tempdir;

    #[test]
    fn try_lock_locked() {
        let dir = tempdir().unwrap();
        let mut lock_path = dir.path().to_owned();
        lock_path.push("foo.lock");

        let _file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open_and_lock(&lock_path)
            .unwrap();

        let error = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .try_open_and_lock(&lock_path)
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    }
}
