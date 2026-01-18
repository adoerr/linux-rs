use std::os::fd::RawFd;

// Netlink Generic Protocol
pub const NETLINK_GENERIC: i32 = 16;

// Generic Netlink Controller (Standard ID)
pub const GENL_ID_CTRL: u16 = 0x10;

// Genl Controller Commands and Attributes
pub const CTRL_CMD_GETFAMILY: u8 = 3;
pub const CTRL_ATTR_FAMILY_ID: u16 = 1;
pub const CTRL_ATTR_FAMILY_NAME: u16 = 2;

// nl80211 Commands and Attributes
pub const NL80211_CMD_GET_WIPHY: u8 = 1;
pub const NL80211_ATTR_WIPHY_NAME: u16 = 1;

// Alignment macros
pub const NLMSG_ALIGNTO: usize = 4;
pub const NLA_ALIGNTO: usize = 4;

pub fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}

pub fn nla_align(len: usize) -> usize {
    (len + NLA_ALIGNTO - 1) & !(NLA_ALIGNTO - 1)
}

// Generic Netlink Header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GenlMsgHdr {
    pub(crate) cmd: u8,
    pub(crate) version: u8,
    reserved: u16,
}

// Helper to represent a raw attribute header
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NlAttr {
    pub(crate) nla_len: u16,
    pub(crate) nla_type: u16,
}

pub struct SocketGuard(pub RawFd);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
    }
}
