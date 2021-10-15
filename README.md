Reliably open and lock files
============================

This crates provides a way to reliably open and lock a file, for example
for opening lock files, PID files, spool files, mailboxes and other kinds
of files which are used for synchronisation between processes.

The `OpenAndLock` trait implements the algorithm of the
[`flopen`][] function available on BSD systems. It is roughly equivalent
to opening a file and calling [`flock`][] with an `operation` argument
set to `LOCK_EX`, but it also attempts to detect and handle races between
opening or creating the file and locking it.

The trait provides two implementations, a blocking and a non-blocking one.

The `open_and_lock()` method waits until the file can be locked, so unless
an unrelated I/O error occurs, it will eventually succeed once the file has
been released if it’s been held by a different process.

The `try_open_and_lock()` method returns an error immediately when the file
cannot be locked, allowing the called to handle it and retry if necessary.

Both methods retry automatically when a race condition occurs and a file
gets deleted or recreated directly after the lock has been acquired.

This trait extends `OpenOptions`, so it can be used the following way:
```
let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open_and_lock("/path/to/file")?;
```

At the moment, this crate supports UNIX-like platforms only.

[`flopen`]: https://manpages.debian.org/flopen
[`flock`]: https://manpages.debian.org/2/flock

License
-------

[MIT license](LICENSE-MIT), also known as the Expat license.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any additional
terms or conditions.
