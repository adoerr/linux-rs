#![allow(dead_code)]

use std::io::Result;

mod poll;
mod poll_sys;

fn main() -> Result<()> {
    println!("Hello, world!");
    Ok(())
}
