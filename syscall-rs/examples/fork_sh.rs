use nix::{
    sys::wait::{WaitStatus, waitpid},
    unistd::{ForkResult, execv, fork},
};
use syscall::Result;

fn main() -> Result<()> {
    env_logger::init();

    match unsafe { fork()? } {
        ForkResult::Child => {
            let target_path = "/bin/sh";
            let cstr_path = std::ffi::CString::new(target_path)?;
            let args = [cstr_path.clone()];

            log::info!("child: executing {}", target_path);
            execv(&cstr_path, &args)?;
        }
        ForkResult::Parent { child } => {
            log::info!("parent: spawned child with PID {}", child);

            log::info!("parent: waiting for child to exit...");

            match waitpid(child, None)? {
                WaitStatus::Exited(_, status) => {
                    log::info!("parent: child exited with status {}", status);
                }
                WaitStatus::Signaled(_, signal, _) => {
                    log::info!("parent: child killed by signal {:?}", signal);
                }
                _ => {}
            }
        }
    }

    Ok(())
}
