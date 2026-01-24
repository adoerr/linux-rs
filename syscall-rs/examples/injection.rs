use std::{ffi::CString, mem, ptr};

use syscall::{Result, syscall};

fn main() -> Result<()> {
    let pid = syscall!(fork())?;

    // child process
    if pid == 0 {
        syscall!(ptrace(libc::PTRACE_TRACEME))?;

        // execv the target binary (e.g., /bin/ls)
        let path = CString::new("/bin/ls")?;
        let arg1 = CString::new("ls")?;
        let args = [arg1.as_ptr(), ptr::null()];

        // block immediately before the first instruction due to TRACEME
        syscall!(execv(path.as_ptr(), args.as_ptr()))?;
    }
    // parent process
    else {
        log::info!("spawned child PID: {}", pid);
    }

    Ok(())
}
