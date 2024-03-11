use std::{io::Result, net::TcpStream, os::fd::AsRawFd};

use crate::poll_sys;

/// System call wrapper.
///
/// Wrapper around `poll_sys` calls that checks `errno` on failure.
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { poll_sys::$fn($($arg, )*) };

        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

type Events = Vec<poll_sys::Event>;

pub struct Registry {
    raw_fd: i32,
}

impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interest: i32) -> Result<()> {
        let mut event = poll_sys::Event {
            events: interest as u32,
            epoll_data: token,
        };

        syscall!(epoll_ctl(
            self.raw_fd,
            poll_sys::EPOLL_CTL_ADD,
            source.as_raw_fd(),
            &mut event
        ))?;

        Ok(())
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        todo!()
    }
}
