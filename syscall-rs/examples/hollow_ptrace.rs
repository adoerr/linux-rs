use std::{
    ffi::CString,
    fs::OpenOptions,
    io::{Seek, SeekFrom, Write},
};

use nix::{
    sys::{
        ptrace,
        signal::Signal,
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, execv, fork},
};
use syscall::Result;

fn main() -> Result<()> {
    env_logger::init();

    // define the target process and args
    let target_path = CString::new("/bin/ls")?;

    // fork the process
    match unsafe { fork()? } {
        ForkResult::Child => {
            ptrace::traceme()?;
            // execute the legitimate binary. this will trigger a SIGTRAP immediately due to TRACEME,
            // pausing execution at the very first instruction of 'ls'.
            execv(&target_path, &[&target_path])?;
        }
        ForkResult::Parent { child } => {
            log::info!("spawned child PID: {}", child);

            // wait for the child to pause after execv
            waitpid(child, None)?;
            log::info!("child process paused at entry point.");

            // get the current registers to find the Instruction Pointer (RIP)
            let regs = ptrace::getregs(child)?;
            log::info!("original RIP (Entry Point): {:#x}", regs.rip);

            // dummy payload: 0xCC is the opcode for INT 3 (Breakpoint).
            // put your shellcode or ELF loader here instead.
            let payload: [u8; 1] = [0xCC];

            // write payload to child's memory at the RIP address.
            // Using /proc/PID/mem is often easier than PTRACE_POKETEXT for larger writes.
            let mem_path = format!("/proc/{}/mem", child);
            let mut mem_file = OpenOptions::new().write(true).read(true).open(mem_path)?;

            // seek to the instruction pointer and write
            mem_file.seek(SeekFrom::Start(regs.rip))?;
            mem_file.write_all(&payload)?;
            log::info!("injected malicious code (INT 3) at {:#x}", regs.rip);

            // resume the child
            ptrace::cont(child, None)?;

            // wait again to see what happens
            match waitpid(child, None)? {
                WaitStatus::Stopped(_, Signal::SIGTRAP) => {
                    log::info!("child hit our injected trap!");
                    log::info!("execution flow was successfully hijacked.");
                }
                status => {
                    log::info!("unexpected status: {:?}", status);
                }
            }

            // cleanup: kill the zombified process
            ptrace::kill(child)?;
        }
    }

    Ok(())
}
