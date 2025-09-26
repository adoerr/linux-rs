// main function for the sulfur debugger

use debugger::Result;

fn main() -> Result<()> {
    // get program argument as a path
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <program>", args[0]);
        std::process::exit(1);
    }

    // Convert to CString
    let prog = std::ffi::CString::new(args[1].clone())?;
    dbg!(&prog);

    Ok(())
}
