#![allow(dead_code)]

mod error;
mod ptrace;

use std::{collections::BTreeMap, ffi::CString};

use crate::{
    error::{Error, Result},
    ptrace::{ptrace, Pid, PtraceAction},
};

/// A Linux debugger
#[derive(Debug)]
struct Debugger {
    /// State of tracees
    tracee: BTreeMap<Pid, TraceeState>,
}

#[derive(Debug)]
enum TraceeState {
    /// We spawned a new program and we're waiting for the debugger to check
    /// in with an initial stop before `exec()`
    WaitForInitialStop,
}

impl Debugger {
    pub fn spawn<T>(program: impl Into<Vec<u8>>, args: impl AsRef<[T]>) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        // convert program name and arguments to C strings
        let prog = CString::new(program)?;

        let mut c_args = Vec::new();

        for a in args.as_ref() {
            c_args.push(CString::new(a.as_ref().to_vec())?);
        }

        // fork a thread to `exec()` in
        let pid = unsafe { libc::fork() };

        // child thread
        if pid == 0 {
            // argument list and filename as null-terminated strings
            let mut raw_args = c_args.iter().map(|a| a.as_ptr()).collect::<Vec<_>>();
            raw_args.insert(0, prog.as_ptr());

            // request to be debugged
            ptrace(PtraceAction::TraceMe)?;

            // stop before exec, so parent can introspect
            if unsafe { libc::raise(libc::SIGSTOP) } != 0 {
                return Err(Error::IO(std::io::Error::last_os_error()));
            }

            // exec the program
            if unsafe { libc::execvp(prog.as_ptr(), raw_args.as_ptr()) } == -1 {
                return Err(Error::IO(std::io::Error::last_os_error()));
            };
        }

        // parent thread - register child as tracee
        let mut tracee = BTreeMap::new();
        tracee.insert(Pid(pid), TraceeState::WaitForInitialStop);

        Ok(Self { tracee })
    }
}

fn main() -> Result<()> {
    let debugger = Debugger::spawn("echo", ["Hello World"])?;
    dbg!(debugger);
    Ok(())
}
