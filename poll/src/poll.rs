use std::{io::Result, net::TcpStream, os::fd::AsRawFd};

use crate::poll_sys;

/// System call wrapper.
///
/// Wrapper around `poll_sys` calls that checks `errno` on failure.
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(clippy::macro_metavars_in_unsafe)]
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
        if let Err(e) = syscall!(close(self.raw_fd)) {
            eprintln!("Error closing epoll file descriptor: {e:?}");
        }
    }
}

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Poll> {
        let raw_fd = syscall!(epoll_create(1))?;

        Ok(Poll {
            registry: Registry { raw_fd },
        })
    }

    /// Poll for events in [`Registry`].
    ///
    /// The method will block until either at least one event is available, or the timeout has expired.
    ///
    /// ## Arguments
    ///
    /// * `events` - Available events after [poll()](Poll::poll) returns.
    /// * `timeout` - Optional timeout in milliseconds. If `None`, block indefinitely until an event occurs.
    ///
    /// ## Returns
    ///
    /// * `Ok(usize)` - Number of events available.
    /// * `Err(std::io::Error)` - IO Error while waiting.
    pub fn poll(&self, events: &mut Events, timeout: Option<i32>) -> Result<usize> {
        let res = syscall!(epoll_wait(
            self.registry.raw_fd,
            events.as_mut_ptr(),
            events.capacity() as i32,
            timeout.unwrap_or(-1)
        ))?;

        unsafe {
            events.set_len(res as usize);
        }

        Ok(res as usize)
    }
}
