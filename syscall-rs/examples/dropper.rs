use std::{ffi, fs, io::Write, os::fd::FromRawFd, process};

use syscall::{Result, syscall};

/// This example demonstrates a "fileless dropper" technique using `memfd_create`.
///
/// It creates an anonymous file in memory (which never touches the disk),
/// writes a small shell script payload into it, and then executes that payload
/// by referencing the file descriptor path in `/proc/self/fd/`.
///
/// This technique is often used to execute code without leaving forensic artifacts
/// on the file system.
fn main() -> Result<()> {
    env_logger::init();

    log::info!("running fileless dropper...");

    // shell script payload to be executed.
    let payload = b"#!/bin/sh\necho 'Hello from inside memory!'; sleep 1";

    // create an anonymous file in RAM using memfd_create.
    // "audio" is just a name for debugging purposes (visible in /proc/self/fd/).
    let name = ffi::CString::new("audio")?;
    let fd = syscall!(memfd_create(name.as_ptr(), 0))?;

    log::info!("anonymous memory file created with fd: {}", fd);

    // write the payload to the in-memory file.
    let mut mem_file = unsafe { fs::File::from_raw_fd(fd) };
    mem_file.write_all(payload)?;

    // construct the path to the file descriptor in the proc filesystem.
    // this allows other processes (like /bin/sh) to access the anonymous file.
    let proc_path = format!("/proc/self/fd/{}", fd);

    log::info!("executing payload via {}", proc_path);

    // execute the shell, passing the path to our in-memory script.
    process::Command::new("/bin/sh")
        .arg(proc_path)
        .stdout(process::Stdio::inherit())
        .status()?;

    log::info!("execution complete, no files were written to disk.");

    Ok(())
}
