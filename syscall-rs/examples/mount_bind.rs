//! This example demonstrates a process hiding technique using bind mounts.
//!
//! The program mounts a decoy binary (e.g., `/usr/bin/top`) over its own executable path.
//! This causes tools like `ls` or `file` to inspect the decoy file instead of the
//! running malware executable, effectively masking its presence on the filesystem
//! while the process continues to execute.
//!
//! **Note:** This requires root privileges (CAP_SYS_ADMIN) to perform the mount operation.

use std::{env, path::Path, thread};

use nix::mount::{MsFlags, mount};
use syscall::Result;

fn main() -> Result<()> {
    env_logger::init();

    // identify the current executable path
    let current_path = env::current_exe()?;
    log::info!("current executable path: {:?}", current_path);

    // run 'file' command on the current executable path before the bind mount
    let out = std::process::Command::new("file")
        .arg(&current_path)
        .output()?;

    let decoy_path = Path::new("/usr/bin/top");

    // verify the decoy path exists
    if !decoy_path.exists() {
        log::error!("decoy path does not exist: {:?}", decoy_path);
        return Ok(());
    }

    log::info!(
        "attempting to bind mount {:?} over {:?}",
        decoy_path,
        current_path
    );

    // perform the bind mount
    mount(
        Some(decoy_path),
        &current_path,
        None::<&str>,
        MsFlags::MS_BIND | MsFlags::MS_REC,
        None::<&str>,
    )?;

    log::info!("bind mount successful, file on disk now appears as 'top'");
    log::info!("simulating forensic check of the binary...");

    // run 'file' command on the current executable path after the bind mount
    let out_new = std::process::Command::new("file")
        .arg(&current_path)
        .output()?;

    log::info!(
        "'file' output before bind mount: {}",
        String::from_utf8_lossy(&out.stdout)
    );

    log::info!(
        "'file' output after bind mount: {}",
        String::from_utf8_lossy(&out_new.stdout)
    );

    log::info!("malware continues to run in background...");
    thread::sleep(std::time::Duration::from_secs(3));
    log::info!("malware stopped.");

    Ok(())
}
