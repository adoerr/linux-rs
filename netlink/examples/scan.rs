use std::error::Error;

use netlink::ApScanner;

fn main() -> Result<(), Box<dyn Error>> {
    let interface = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "wlan0".to_string());
    println!("Scanning on interface: {}", interface);

    let mut scanner = ApScanner::new()?;
    let aps = scanner.scan(&interface)?;

    for ap in aps {
        println!("BSSID: {:02X?}", ap.bssid.unwrap_or([0; 6]));
        println!("SSID: {}", ap.ssid.as_deref().unwrap_or("<hidden>"));
        println!("Freq: {} MHz", ap.frequency.unwrap_or(0));
        println!("Signal: {:.2} dBm", ap.signal_dbm().unwrap_or(0.0));
        println!("-------------------");
    }

    Ok(())
}
