use std::error::Error;

use netlink::ApScanner;

fn main() -> Result<(), Box<dyn Error>> {
    let mut scanner = ApScanner::new()?;
    let interfaces = scanner.get_interfaces()?;

    for interface in interfaces {
        println!(
            "Interface: {}",
            interface.name.as_deref().unwrap_or("unknown")
        );
        println!("  Index: {}", interface.index.unwrap_or(0));
        println!("  MAC: {:02X?}", interface.mac.unwrap_or([0; 6]));
        println!("  Phy: {}", interface.phy.unwrap_or(0));
        println!("-------------------");
    }

    Ok(())
}
