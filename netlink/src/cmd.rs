use std::{ffi::CString, mem, os::fd::RawFd, ptr};

use crate::{
    CTRL_ATTR_FAMILY_ID, CTRL_ATTR_FAMILY_NAME, CTRL_CMD_GETFAMILY, GENL_ID_CTRL, GenlMsgHdr,
    NETLINK_GENERIC, NL80211_ATTR_WIPHY_NAME, NL80211_CMD_GET_WIPHY, NlAttr, SocketGuard,
    nla_align, nlmsg_align,
};

pub fn run() -> Result<(), String> {
    // 1. Open Netlink Generic Socket
    let fd: RawFd = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, NETLINK_GENERIC) };
    if fd < 0 {
        return Err("Failed to create NETLINK_GENERIC socket".to_string());
    }
    let _guard = SocketGuard(fd);

    // 2. Bind (optional but good practice)
    let mut sa: libc::sockaddr_nl = unsafe { mem::zeroed() };
    sa.nl_family = libc::AF_NETLINK as u16;
    if unsafe {
        libc::bind(
            fd,
            &sa as *const _ as *const _,
            mem::size_of_val(&sa) as u32,
        )
    } < 0
    {
        return Err("Failed to bind socket".to_string());
    }

    // 3. Resolve "nl80211" family ID
    let family_id = unsafe { get_family_id(fd, "nl80211") }?;
    // println!("Resolved nl80211 family ID: {}", family_id);

    // 4. Dump Wiphys (Physical Interfaces)
    unsafe { dump_wiphys(fd, family_id) }?;

    Ok(())
}

// --- Step 3: Resolve Family ID ---

unsafe fn get_family_id(fd: RawFd, name: &str) -> Result<u16, String> {
    let name_c = CString::new(name).map_err(|_| "Invalid name")?;
    let name_bytes = name_c.as_bytes_with_nul();

    // Calculate lengths
    // Header = nlmsghdr + genlmsghdr
    // Attr = nlattr + padded_name
    let attr_len = nla_align(mem::size_of::<NlAttr>() + name_bytes.len());
    let total_len =
        nlmsg_align(mem::size_of::<libc::nlmsghdr>() + mem::size_of::<GenlMsgHdr>() + attr_len);

    let mut buffer = vec![0u8; total_len];

    // Fill nlmsghdr
    let nl_hdr = buffer.as_mut_ptr() as *mut libc::nlmsghdr;
    unsafe {
        (*nl_hdr).nlmsg_len = total_len as u32;
        (*nl_hdr).nlmsg_type = GENL_ID_CTRL; // Send to Controller
        (*nl_hdr).nlmsg_flags = libc::NLM_F_REQUEST as u16;
        (*nl_hdr).nlmsg_seq = 1;
        (*nl_hdr).nlmsg_pid = 0;
    }

    let attr_ptr = unsafe { fill_genl_msg_hdr(&mut buffer, name_bytes.len()) };

    // Copy name data
    unsafe {
        let data_ptr = (attr_ptr as *mut u8).add(mem::size_of::<NlAttr>());
        ptr::copy_nonoverlapping(name_bytes.as_ptr(), data_ptr, name_bytes.len());
    }

    // Send
    if unsafe { libc::send(fd, buffer.as_ptr() as *const _, buffer.len(), 0) } < 0 {
        return Err("Failed to send family resolution request".to_string());
    }

    // Receive Response
    let mut recv_buf = vec![0u8; 4096];
    let len = unsafe { libc::recv(fd, recv_buf.as_mut_ptr() as *mut _, recv_buf.len(), 0) };
    if len <= 0 {
        return Err("Failed to receive family resolution response".to_string());
    }

    // Parse Response to find CTRL_ATTR_FAMILY_ID
    let nl_hdr_resp = recv_buf.as_ptr() as *const libc::nlmsghdr;
    if unsafe { (*nl_hdr_resp).nlmsg_type } == libc::NLMSG_ERROR as u16 {
        return Err("Received Netlink Error on resolution".to_string());
    }

    // Skip Headers to get to Attributes
    let mut offset =
        nlmsg_align(mem::size_of::<libc::nlmsghdr>()) + nlmsg_align(mem::size_of::<GenlMsgHdr>());
    let msg_len = unsafe { (*nl_hdr_resp).nlmsg_len } as usize;

    while offset + mem::size_of::<NlAttr>() <= msg_len {
        unsafe {
            let attr = recv_buf.as_ptr().add(offset) as *const NlAttr;
            let attr_len = (*attr).nla_len as usize;
            let attr_type = (*attr).nla_type & 0x3FFF; // Mask out nested flags if any

            if attr_type == CTRL_ATTR_FAMILY_ID {
                let val_ptr = (attr as *const u8).add(mem::size_of::<NlAttr>()) as *const u16;
                return Ok(*val_ptr);
            }

            offset += nla_align(attr_len);
        }
    }

    Err("Family ID not found in response".to_string())
}

unsafe fn fill_genl_msg_hdr(buffer: &mut [u8], name_len: usize) -> *mut NlAttr {
    let attr_ptr: *mut NlAttr;
    unsafe {
        // Fill genlmsghdr
        let genl_hdr = buffer
            .as_mut_ptr()
            .add(nlmsg_align(mem::size_of::<libc::nlmsghdr>()))
            as *mut GenlMsgHdr;
        (*genl_hdr).cmd = CTRL_CMD_GETFAMILY;
        (*genl_hdr).version = 1;

        // Fill Attribute (CTRL_ATTR_FAMILY_NAME)
        attr_ptr =
            (genl_hdr as *mut u8).add(nlmsg_align(mem::size_of::<GenlMsgHdr>())) as *mut NlAttr;
        (*attr_ptr).nla_len = (mem::size_of::<NlAttr>() + name_len) as u16;
        (*attr_ptr).nla_type = CTRL_ATTR_FAMILY_NAME;
    }
    attr_ptr
}

// --- Step 4: Dump Wiphys ---

unsafe fn dump_wiphys(fd: RawFd, family_id: u16) -> Result<(), String> {
    // Construct Request: NL80211_CMD_GET_WIPHY with DUMP flag
    let total_len = nlmsg_align(mem::size_of::<libc::nlmsghdr>() + mem::size_of::<GenlMsgHdr>());
    let mut buffer = vec![0u8; total_len];

    let nl_hdr = buffer.as_mut_ptr() as *mut libc::nlmsghdr;
    unsafe {
        (*nl_hdr).nlmsg_len = total_len as u32;
        (*nl_hdr).nlmsg_type = family_id;
        (*nl_hdr).nlmsg_flags = (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16;
        (*nl_hdr).nlmsg_seq = 2;
    }

    let genl_hdr = unsafe {
        buffer
            .as_mut_ptr()
            .add(nlmsg_align(mem::size_of::<libc::nlmsghdr>())) as *mut GenlMsgHdr
    };
    unsafe {
        (*genl_hdr).cmd = NL80211_CMD_GET_WIPHY;
        (*genl_hdr).version = 1;
    }

    if unsafe { libc::send(fd, buffer.as_ptr() as *const _, buffer.len(), 0) } < 0 {
        return Err("Failed to send dump request".to_string());
    }

    // Receive Loop (Multipart message)
    let mut recv_buf = vec![0u8; 8192];
    loop {
        let len = unsafe { libc::recv(fd, recv_buf.as_mut_ptr() as *mut _, recv_buf.len(), 0) };
        if len < 0 {
            return Err("Failed to read dump response".to_string());
        }
        if len == 0 {
            break;
        }

        let mut offset = 0;
        let len = len as usize;

        while offset < len {
            unsafe {
                let hdr = recv_buf.as_ptr().add(offset) as *const libc::nlmsghdr;
                let msg_len = (*hdr).nlmsg_len as usize;

                if msg_len < mem::size_of::<libc::nlmsghdr>() {
                    break;
                }

                let msg_type = (*hdr).nlmsg_type;

                if msg_type == libc::NLMSG_DONE as u16 {
                    return Ok(());
                }
                if msg_type == libc::NLMSG_ERROR as u16 {
                    return Err("Error during dump".to_string());
                }

                // We expect messages with type == family_id
                if msg_type == family_id {
                    // Parse Attributes inside this message
                    // Skip headers
                    let attr_start = offset
                        + nlmsg_align(mem::size_of::<libc::nlmsghdr>())
                        + nlmsg_align(mem::size_of::<GenlMsgHdr>());
                    parse_wiphy_attrs(recv_buf.as_ptr(), attr_start, offset + msg_len);
                }

                offset += nlmsg_align(msg_len);
            }
        }
    }
    Ok(())
}

unsafe fn parse_wiphy_attrs(base: *const u8, mut offset: usize, limit: usize) {
    let mut wiphy_name = String::new();
    let mut wiphy_idx = -1;

    while offset + mem::size_of::<NlAttr>() <= limit {
        unsafe {
            let attr = base.add(offset) as *const NlAttr;
            let attr_len = (*attr).nla_len as usize;
            let attr_type = (*attr).nla_type & 0x3FFF;

            if attr_len < mem::size_of::<NlAttr>() {
                break;
            }

            // NL80211_ATTR_WIPHY_NAME = 1
            if attr_type == NL80211_ATTR_WIPHY_NAME {
                let data_ptr = (attr as *const u8).add(mem::size_of::<NlAttr>());
                let data_len = attr_len - mem::size_of::<NlAttr>();
                let slice = std::slice::from_raw_parts(data_ptr, data_len);
                // Trim nulls
                let clean_len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                wiphy_name = String::from_utf8_lossy(&slice[..clean_len]).to_string();
            }
            // NL80211_ATTR_WIPHY (Index) = 2
            else if attr_type == 2 {
                let data_ptr = (attr as *const u8).add(mem::size_of::<NlAttr>()) as *const u32;
                wiphy_idx = *data_ptr as i32;
            }

            offset += nla_align(attr_len);
        }
    }

    if !wiphy_name.is_empty() {
        println!(
            "Found Physical Adapter: {} (Index: {})",
            wiphy_name, wiphy_idx
        );
    }
}
