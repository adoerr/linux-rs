use netlink_wi::NlSocket;

fn main() {
    env_logger::try_init().unwrap();

    let socket = NlSocket::connect().unwrap();
    let interfaces = socket.list_interfaces().unwrap();

    for i in interfaces {
        if !i.name.is_empty() {
            println!(
                "Interface: {}, SSID: {}",
                i.name,
                i.ssid.unwrap_or_else(|| "n/a".to_string())
            );

            let stations = socket.list_stations(i.interface_index).unwrap();
            for s in stations {
                println!(
                    "    Station: {}, Signal: {} dBm",
                    s.mac,
                    s.signal.unwrap_or(0)
                );
            }
        }
    }
}
