// main function for the sulfur debugger
use std::path::PathBuf;

use debugger::Result;

fn main() -> Result<()> {
    // get program argument as a path
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <program>", args[0]);
        std::process::exit(1);
    }

    let program = PathBuf::from(&args[1]);
    dbg!(program.clone());

    Ok(())
}
