//!
//! This file is part of syscall-rs
//!

use std::{fmt, mem, os::unix::prelude::RawFd, str::FromStr};

use crate::{Error, Result};

/// Operating system signal.
///
/// Note that we would want to use `lib::c_int` as the repr attribute. This is
/// not (yet) supported, however.
#[repr(i32)]
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Signal {
    /// Hangup
    SIGHUP = libc::SIGHUP,
    /// Interrupt
    SIGINT = libc::SIGINT,
    /// Quit
    SIGQUIT = libc::SIGQUIT,
    /// Illegal instruction (not reset when caught)
    SIGILL = libc::SIGILL,
    /// Trace trap (not reset when caught)
    SIGTRAP = libc::SIGTRAP,
    /// Abort
    SIGABRT = libc::SIGABRT,
    /// Bus error
    SIGBUS = libc::SIGBUS,
    /// Floating point exception
    SIGFPE = libc::SIGFPE,
    /// Kill (cannot be caught or ignored)
    SIGKILL = libc::SIGKILL,
    /// User defined signal 1
    SIGUSR1 = libc::SIGUSR1,
    /// Segmentation violation
    SIGSEGV = libc::SIGSEGV,
    /// User defined signal 2
    SIGUSR2 = libc::SIGUSR2,
    /// Write on a pipe with no one to read it
    SIGPIPE = libc::SIGPIPE,
    /// Alarm clock
    SIGALRM = libc::SIGALRM,
    /// Software termination signal from kill
    SIGTERM = libc::SIGTERM,
    /// Stack fault (obsolete)
    SIGSTKFLT = libc::SIGSTKFLT,
    /// To parent on child stop or exit
    SIGCHLD = libc::SIGCHLD,
    /// Continue a stopped process
    SIGCONT = libc::SIGCONT,
    /// Sendable stop signal not from tty
    SIGSTOP = libc::SIGSTOP,
    /// Stop signal from tty
    SIGTSTP = libc::SIGTSTP,
    /// To readers pgrp upon background tty read
    SIGTTIN = libc::SIGTTIN,
    /// Like TTIN if (tp->t_local&LTOSTOP)
    SIGTTOU = libc::SIGTTOU,
    /// Urgent condition on IO channel
    SIGURG = libc::SIGURG,
    /// Exceeded CPU time limit
    SIGXCPU = libc::SIGXCPU,
    /// Exceeded file size limit
    SIGXFSZ = libc::SIGXFSZ,
    /// Virtual time alarm
    SIGVTALRM = libc::SIGVTALRM,
    /// Profiling time alarm
    SIGPROF = libc::SIGPROF,
    /// Window size changes
    SIGWINCH = libc::SIGWINCH,
    /// Input/output possible signal
    SIGIO = libc::SIGIO,
    /// Power failure imminent.
    SIGPWR = libc::SIGPWR,
    /// Bad system call
    SIGSYS = libc::SIGSYS,
}

impl Signal {
    /// Return signal name
    pub const fn as_str(self) -> &'static str {
        match self {
            Signal::SIGHUP => "SIGHUP",
            Signal::SIGINT => "SIGINT",
            Signal::SIGQUIT => "SIGQUIT",
            Signal::SIGILL => "SIGILL",
            Signal::SIGTRAP => "SIGTRAP",
            Signal::SIGABRT => "SIGABRT",
            Signal::SIGBUS => "SIGBUS",
            Signal::SIGFPE => "SIGFPE",
            Signal::SIGKILL => "SIGKILL",
            Signal::SIGUSR1 => "SIGUSR1",
            Signal::SIGSEGV => "SIGSEGV",
            Signal::SIGUSR2 => "SIGUSR2",
            Signal::SIGPIPE => "SIGPIPE",
            Signal::SIGALRM => "SIGALRM",
            Signal::SIGTERM => "SIGTERM",
            Signal::SIGSTKFLT => "SIGSTKFLT",
            Signal::SIGCHLD => "SIGCHLD",
            Signal::SIGCONT => "SIGCONT",
            Signal::SIGSTOP => "SIGSTOP",
            Signal::SIGTSTP => "SIGTSTP",
            Signal::SIGTTIN => "SIGTTIN",
            Signal::SIGTTOU => "SIGTTOU",
            Signal::SIGURG => "SIGURG",
            Signal::SIGXCPU => "SIGXCPU",
            Signal::SIGXFSZ => "SIGXFSZ",
            Signal::SIGVTALRM => "SIGVTALRM",
            Signal::SIGPROF => "SIGPROF",
            Signal::SIGWINCH => "SIGWINCH",
            Signal::SIGIO => "SIGIO",
            Signal::SIGPWR => "SIGPWR",
            Signal::SIGSYS => "SIGSYS",
        }
    }
}

impl FromStr for Signal {
    type Err = Error;

    /// Parse [`Signal`] from a signal name
    fn from_str(s: &str) -> Result<Signal> {
        Ok(match s {
            "SIGHUP" => Signal::SIGHUP,
            "SIGINT" => Signal::SIGINT,
            "SIGQUIT" => Signal::SIGQUIT,
            "SIGILL" => Signal::SIGILL,
            "SIGTRAP" => Signal::SIGTRAP,
            "SIGABRT" => Signal::SIGABRT,
            "SIGBUS" => Signal::SIGBUS,
            "SIGFPE" => Signal::SIGFPE,
            "SIGKILL" => Signal::SIGKILL,
            "SIGUSR1" => Signal::SIGUSR1,
            "SIGSEGV" => Signal::SIGSEGV,
            "SIGUSR2" => Signal::SIGUSR2,
            "SIGPIPE" => Signal::SIGPIPE,
            "SIGALRM" => Signal::SIGALRM,
            "SIGTERM" => Signal::SIGTERM,
            "SIGSTKFLT" => Signal::SIGSTKFLT,
            "SIGCHLD" => Signal::SIGCHLD,
            "SIGCONT" => Signal::SIGCONT,
            "SIGSTOP" => Signal::SIGSTOP,
            "SIGTSTP" => Signal::SIGTSTP,
            "SIGTTIN" => Signal::SIGTTIN,
            "SIGTTOU" => Signal::SIGTTOU,
            "SIGURG" => Signal::SIGURG,
            "SIGXCPU" => Signal::SIGXCPU,
            "SIGXFSZ" => Signal::SIGXFSZ,
            "SIGVTALRM" => Signal::SIGVTALRM,
            "SIGPROF" => Signal::SIGPROF,
            "SIGWINCH" => Signal::SIGWINCH,
            "SIGIO" => Signal::SIGIO,
            "SIGPWR" => Signal::SIGPWR,
            "SIGSYS" => Signal::SIGSYS,
            _ => {
                return Err(Error::Syscall(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid signal",
                )))
            }
        })
    }
}

impl TryFrom<libc::c_int> for Signal {
    type Error = Error;

    /// Try to convert `signum` into a [`Signal`]
    ///
    /// This function only supports signal numbering for standard signals
    /// according to the `signal(7)` man page
    fn try_from(signum: libc::c_int) -> std::result::Result<Self, Self::Error> {
        if 0 < signum && signum < 32 {
            Ok(unsafe { mem::transmute(signum) })
        } else {
            Err(Error::Syscall(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid signal number",
            )))
        }
    }
}

impl AsRef<str> for Signal {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

/// A group of multiple different [`Signal`]s
///
/// This is a `Newtype` for [`libc::sigset_t`] and the related
/// functions used to manipulate a signal set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignalSet(libc::sigset_t);

impl SignalSet {
    /// Initialize a set to contain all signals
    pub fn fill() -> Result<SignalSet> {
        let mut set = mem::MaybeUninit::uninit();

        syscall!(sigfillset(set.as_mut_ptr()))?;

        Ok(unsafe { SignalSet(set.assume_init()) })
    }

    /// Initialize a set to contain no signals
    ///
    /// Note that this function will never fail, since `sigemptyset()` has no
    /// errors defined.
    pub fn empty() -> Result<SignalSet> {
        let mut set = mem::MaybeUninit::uninit();

        syscall!(sigemptyset(set.as_mut_ptr()))?;

        Ok(unsafe { SignalSet(set.assume_init()) })
    }

    /// Add `signal` to a set
    pub fn add(&mut self, signal: Signal) -> Result<()> {
        syscall!(sigaddset(
            &mut self.0 as *mut libc::sigset_t,
            signal as libc::c_int
        ))?;

        Ok(())
    }

    /// Remove `signal` from a set
    pub fn remove(&mut self, signal: Signal) -> Result<()> {
        syscall!(sigdelset(
            &mut self.0 as *mut libc::sigset_t,
            signal as libc::c_int
        ))?;

        Ok(())
    }
}

impl AsRef<libc::sigset_t> for SignalSet {
    fn as_ref(&self) -> &libc::sigset_t {
        &self.0
    }
}

impl From<&[Signal]> for SignalSet {
    /// Return a [`SignalSet`] from a slice of [`Signal`]s
    ///
    /// Note that this conversion is save since [`SignalSet::empty()`] never
    /// fails and the slice of [`Signal`]s contains valid signals only, ever.
    fn from(signals: &[Signal]) -> Self {
        *signals.iter().fold(
            &mut SignalSet::empty().expect("syscall failed"),
            |set, sig| {
                set.add(*sig).expect("syscall failed");
                set
            },
        )
    }
}

/// Magic value for creating a new signal fd
const SIGNALFD_NEW: libc::c_int = -1;

/// Special file descriptor from which [`Signal`]s directed to the caller can
/// be read
///
/// Note that a signal fd must **not** derive neither [`Clone`] nor [`Copy`]!
#[derive(Debug, PartialEq, Eq)]
pub struct SignalFd(RawFd);

impl SignalFd {
    /// Return a [`SignalFd`] that will be able to read the signals given in
    /// the signal set `signals`.
    pub fn new(signals: SignalSet) -> Result<SignalFd> {
        let fd = syscall!(signalfd(
            SIGNALFD_NEW,
            signals.as_ref() as *const libc::sigset_t,
            0
        ))?;

        Ok(SignalFd(fd))
    }

    /// Read and return a [`Signal`]
    ///
    /// Note this function will **block** until a signal could be read.
    pub fn read_signal(&mut self) -> Result<Signal> {
        let mut siginfo = mem::MaybeUninit::<libc::signalfd_siginfo>::uninit();
        let size = mem::size_of_val(&siginfo);

        let num = syscall!(read(
            self.0,
            siginfo.as_mut_ptr() as *mut libc::c_void,
            size
        ))?;

        // not enough signal info data
        if num as usize != size {
            return Err(Error::Syscall(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid signal",
            )));
        }

        let siginfo = unsafe { siginfo.assume_init() };
        let signum = siginfo.ssi_signo as libc::c_int;

        signum.try_into()
    }
}

impl Drop for SignalFd {
    /// Drop the [`SignalFd`]
    ///
    /// Dropping a signal fd involves calling [`libc::close()`]. Note that
    /// `close` is special in the sense that all errors except an invalid
    /// signal fd should simply be ignored.
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };

        // an invalid signal fd is the only error we panic for
        if std::io::Error::last_os_error().raw_os_error().unwrap_or(0) == libc::EBADF {
            panic!("closing invalid signal fd");
        }
    }
}

/// Add the signals in `set` to the current process signal mask and return the
/// previous process signal mask.
///
/// Note that the process signal mask will be set to the **union** of the current
/// process signal mask and `set`.
pub fn block(set: SignalSet) -> Result<SignalSet> {
    let mut old = SignalSet::empty()?;

    syscall!(sigprocmask(
        libc::SIG_BLOCK,
        &set.0 as *const libc::sigset_t,
        &mut old.0 as &mut libc::sigset_t
    ))?;

    Ok(old)
}

/// Set the process signal mask to the signals in `set` and return the previous
/// process signal mask.
///
/// Note that this will also unblock those signals which have previously been
/// blocked by a call to [`block()`]
pub fn restore(set: SignalSet) -> Result<SignalSet> {
    let mut old = SignalSet::empty()?;

    syscall!(sigprocmask(
        libc::SIG_SETMASK,
        &set.0 as *const libc::sigset_t,
        &mut old.0 as &mut libc::sigset_t
    ))?;

    Ok(old)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{block, restore, Signal, SignalFd, SignalSet};

    #[test]
    fn signal_set_add() -> Result<()> {
        let mut set = SignalSet::empty()?;

        set.add(Signal::SIGCHLD)?;
        set.add(Signal::SIGPIPE)?;

        assert_ne!(set, SignalSet::empty()?);
        assert_ne!(set, SignalSet::fill()?);

        Ok(())
    }

    #[test]
    fn signal_set_remove() -> Result<()> {
        let mut set = SignalSet::empty()?;

        set.add(Signal::SIGHUP)?;
        set.add(Signal::SIGQUIT)?;

        set.remove(Signal::SIGHUP)?;
        set.remove(Signal::SIGQUIT)?;

        assert_eq!(set, SignalSet::empty()?);
        assert_ne!(set, SignalSet::fill()?);

        Ok(())
    }

    #[test]
    fn signal_set_remove_unknown() -> Result<()> {
        let mut set = SignalSet::empty()?;

        set.remove(Signal::SIGCHLD)?;

        set.add(Signal::SIGHUP)?;

        set.remove(Signal::SIGCHLD)?;
        set.remove(Signal::SIGHUP)?;

        assert_eq!(set, SignalSet::empty()?);
        assert_ne!(set, SignalSet::fill()?);

        Ok(())
    }

    #[test]
    fn signal_try_from() -> Result<()> {
        let signum = Signal::SIGQUIT as libc::c_int;
        let sig: Signal = signum.try_into()?;

        assert_eq!(signum, libc::SIGQUIT);
        assert_eq!(sig, Signal::SIGQUIT);

        let res: std::result::Result<Signal, _> = (255 as libc::c_int).try_into();

        assert_eq!(
            format!("{:?}", res.err().unwrap()),
            "Syscall(Custom { kind: InvalidData, error: \"invalid signal number\" })"
        );

        Ok(())
    }

    #[test]
    fn block_signals() -> Result<()> {
        let signals = vec![
            Signal::SIGCHLD,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTERM,
        ];

        // nothing has been blocked before
        let old = block(signals.as_slice().into())?;

        assert_eq!(old, SignalSet::empty()?);
        assert_ne!(old, SignalSet::fill()?);

        // return the previously blocked signal set
        let blocked = block(old)?;

        assert_eq!(blocked, signals.as_slice().into());

        Ok(())
    }

    #[test]
    fn restore_signals() -> Result<()> {
        let signals = vec![
            Signal::SIGCHLD,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGTERM,
        ];

        let old = block(signals.as_slice().into())?;

        let blocked = restore(old)?;

        assert_eq!(blocked, signals.as_slice().into());

        Ok(())
    }

    #[test]
    fn signalfd_new() -> Result<()> {
        let _ = SignalFd::new(SignalSet::empty()?)?;
        Ok(())
    }

    #[test]
    #[should_panic(expected = "closing invalid signal fd")]
    fn signalfd_drop_invalid() {
        let fake = SignalFd(-1);
        drop(fake);
    }
}
