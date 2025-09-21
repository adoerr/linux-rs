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
    Run(RunArgs),
    Attach(AttachArgs),
}

/// Run a binary using the given path
#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
struct RunArgs {
    /// path to binary
    #[argh(positional)]
    path: PathBuf,
}

/// Attach to a running process using the given pid
#[derive(FromArgs)]
#[argh(subcommand, name = "attach")]
struct AttachArgs {
    /// pid of process to attach
    #[argh(positional)]
    pid: pid_t,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();

    match args.command {
        Command::Run(path) => {
            println!("Attaching to binary: {:?}", path.path);
        }
        Command::Attach(pid) => {
            println!("Attaching to pid: {}", pid.pid);
        }
    }

    Ok(())
}
