// main function for the sulfur debugger
use std::path::PathBuf;

use argh::FromArgs;
use debugger::Result;
use nix::libc::pid_t;

/// Sulfur Debugger
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    Binary(BinaryArgs),
    Pid(PidArgs),
}

/// Run and attach to binary using the given path
#[derive(FromArgs)]
#[argh(subcommand, name = "binary")]
struct BinaryArgs {
    /// path to binary
    #[argh(positional)]
    path: PathBuf,
}

/// Attach to a running process using the given pid
#[derive(FromArgs)]
#[argh(subcommand, name = "pid")]
struct PidArgs {
    /// pid of process to attach
    #[argh(positional)]
    pid: pid_t,
}
fn main() -> Result<()> {
    let args: Args = argh::from_env();

    match args.command {
        Command::Binary(path) => {
            println!("Attaching to binary: {:?}", path.path);
        }
        Command::Pid(pid) => {
            println!("Attaching to pid: {}", pid.pid);
        }
    }

    Ok(())
}
