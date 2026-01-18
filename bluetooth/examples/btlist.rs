use std::{io, ptr};

use libc::{
    AF_BLUETOOTH, SOCK_CLOEXEC, SOCK_NONBLOCK, SOCK_RAW, bind, c_void, recv, send, sockaddr, socket,
};

const BTPROTO_HCI: i32 = 1;
const HCI_DEV_NONE: u16 = 0xffff;
const HCI_CHANNEL_CONTROL: u16 = 3;

const MGMT_OP_READ_INDEX_LIST: u16 = 0x0003;
const MGMT_EV_COMMAND_COMPLETE: u16 = 0x0001;

#[repr(C, packed)]
struct SockAddrHci {
    hci_family: u16,
    hci_dev: u16,
    hci_channel: u16,
}

#[repr(C, packed)]
struct MgmtHdr {
    opcode: u16,
    index: u16,
    len: u16,
}

#[allow(unused)]
#[repr(C, packed)]
struct MgmtEvCmdComplete {
    opcode: u16,
    status: u8,
}

fn main() -> io::Result<()> {
    let fd = unsafe {
        socket(
            AF_BLUETOOTH,
            SOCK_RAW | SOCK_CLOEXEC | SOCK_NONBLOCK,
            BTPROTO_HCI,
        )
    };

    if fd < 0 {
        return Err(io::Error::last_os_error());
    }

    println!("Socket opened (fd: {})", fd);

    let addr = SockAddrHci {
        hci_family: AF_BLUETOOTH as u16,
        hci_dev: HCI_DEV_NONE,
        hci_channel: HCI_CHANNEL_CONTROL,
    };

    let bind_res = unsafe {
        bind(
            fd,
            &addr as *const _ as *const sockaddr,
            size_of::<SockAddrHci>() as u32,
        )
    };

    if bind_res < 0 {
        let err = io::Error::last_os_error();
        unsafe { libc::close(fd) };
        return Err(err);
    }

    println!("Bound to HCI_CHANNEL_CONTROL");

    let cmd = MgmtHdr {
        opcode: MGMT_OP_READ_INDEX_LIST.to_le(),
        index: 0xffffu16.to_le(), // MGMT_INDEX_NONE
        len: 0u16.to_le(),
    };

    let sent = unsafe {
        send(
            fd,
            &cmd as *const _ as *const c_void,
            size_of::<MgmtHdr>(),
            0,
        )
    };

    if sent < 0 {
        let err = io::Error::last_os_error();
        unsafe { libc::close(fd) };
        return Err(err);
    }

    println!("Sent MGMT_OP_READ_INDEX_LIST...");

    let mut buffer = [0u8; 1024];

    loop {
        let bytes_recv = unsafe { recv(fd, buffer.as_mut_ptr() as *mut c_void, buffer.len(), 0) };

        if bytes_recv < 0 {
            let err = io::Error::last_os_error();

            if err.kind() == io::ErrorKind::WouldBlock {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            unsafe { libc::close(fd) };
            return Err(err);
        }

        let n = bytes_recv as usize;
        if n < size_of::<MgmtHdr>() {
            continue;
        }

        let hdr_ptr = buffer.as_ptr() as *const MgmtHdr;
        let hdr = unsafe { ptr::read_unaligned(hdr_ptr) };

        let opcode = u16::from_le(hdr.opcode);
        let _len = u16::from_le(hdr.len);

        if opcode == MGMT_EV_COMMAND_COMPLETE {
            println!("Received Event: Command Complete");

            if n >= (6 + 3) {
                let num_controllers = u16::from_le_bytes([buffer[9], buffer[10]]);
                println!(
                    "Success! Found {} Bluetooth controller(s).",
                    num_controllers
                );

                let mut offset = 11;

                for i in 0..num_controllers {
                    if n < offset + 2 {
                        break;
                    }
                    let idx = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
                    println!("Controller #{}: hci{}", i, idx);
                    offset += 2;
                }
            }
            break;
        }
    }

    unsafe { libc::close(fd) };
    Ok(())
}
