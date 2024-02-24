//! Helpers for interacting with subprocesses

use std::{
    io::{Error as IoError, Read},
    os::fd::AsRawFd,
    process::{ChildStderr, ChildStdout},
    thread,
    time::Duration,
};

use timeout_readwrite::TimeoutReader;

/// Continuously reads from the provided subprocess pipes until both pipes received EOL (subprocess terminated / killed) or until the provided timeout is reached
///
/// Returns stdout and stderr as string (unknown chars encoded as unicode replacement character)
pub fn subprocess_wait_timeout(
    stdout: ChildStdout,
    stderr: ChildStderr,
    timeout: Duration,
) -> Result<(String, String), IoError> {
    let stdout_handle = thread::spawn(move || read_pipe_to_end(stdout, timeout));
    let stderr_handle = thread::spawn(move || read_pipe_to_end(stderr, timeout));

    let stdout_data = stdout_handle.join().unwrap()?;
    let stderr_data = stderr_handle.join().unwrap()?;

    let stdout_string = String::from_utf8_lossy(&stdout_data);
    let stderr_string = String::from_utf8_lossy(&stderr_data);

    Ok((stdout_string.into(), stderr_string.into()))
}

/// Read data from provided pipe into a vector until EOF is encountered (subprocess exited / has been terminated)
fn read_pipe_to_end<T: Read + AsRawFd>(pipe: T, timeout: Duration) -> Result<Vec<u8>, IoError> {
    let mut buf = Vec::with_capacity(1048576);

    let mut reader = TimeoutReader::new(pipe, timeout);
    reader.read_to_end(&mut buf)?;

    Ok(buf)
}
