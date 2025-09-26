// main function for the sulfur debugger

use debugger::{Error, Pid, Ptrace, Result, ptrace, waitpid};

fn main() -> Result<()> {
    // get program argument as a path
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <program>", args[0]);
        std::process::exit(1);
    }

    // convert to CString
    let prog = std::ffi::CString::new(args[1].clone())?;
    dbg!(&prog);

    // create new thread to run the program
    let pid = unsafe { libc::fork() };

    if pid == -1 {
        return Err(Error::IO(std::io::Error::last_os_error()));
    } else if pid == 0 {
        // child process
        ptrace(Ptrace::TraceMe)?;

        // send SIGSTOP to self to allow parent to attach
        if unsafe { libc::raise(libc::SIGSTOP) } == -1 {
            return Err(Error::IO(std::io::Error::last_os_error()));
        }

        // execute the program
        if unsafe { libc::execlp(prog.as_ptr(), prog.as_ptr(), std::ptr::null::<i8>()) } == -1 {
            return Err(Error::IO(std::io::Error::last_os_error()));
        }
    }

    // parent process
    let child = Pid(pid);

    // wait for child to stop
    let status = waitpid(child)?;
    dbg!(status);

    Ok(())
}
