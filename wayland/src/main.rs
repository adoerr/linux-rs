mod error;

use error::Result;

fn main() -> Result<()> {
    env_logger::init();

    log::info!("Hello world!");

    Ok(())
}
