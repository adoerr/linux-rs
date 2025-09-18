// main function for the sulfur debugger
use std::path::PathBuf;

use argh::FromArgs;
use debugger::Result;
use nix::libc::pid_t;

/// Sulfur Debugger
#[derive(FromArgs)]
struct Args {
    /// path to binary
    #[argh(option)]
    binary: Option<PathBuf>,

    /// pid of process to attach
    #[argh(option)]
    pid: Option<pid_t>,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    match (args.binary, args.pid) {
        (Some(path), None) => {
            println!("Binary path: {:?}", path);
            // launch debugger with binary
        }
        (None, Some(pid)) => {
            println!("PID: {}", pid);
            // attach debugger to pid
        }
        _ => {
            eprintln!("Specify either --binary <path> or --pid <pid>");
            std::process::exit(1);
        }
    }

    Ok(())
}
