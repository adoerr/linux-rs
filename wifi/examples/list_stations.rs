use netlink_wi::NlSocket;

fn main() {
    env_logger::try_init().unwrap();

    let socket = NlSocket::connect().unwrap();
    let filter = std::env::args().nth(1);

    for interface in socket.list_interfaces().unwrap() {
        if filter.as_deref().map_or(true, |n| interface.name == n) {
            if let Some(ssid) = &interface.ssid {
                println!("SSID: {ssid}");
            }
        }
    }
}
