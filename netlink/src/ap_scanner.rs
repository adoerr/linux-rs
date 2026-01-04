use std::{error::Error, io::Cursor};

use neli::{
    FromBytes,
    consts::{
        genl::{CtrlAttr, CtrlCmd},
        nl::{NlmF, Nlmsg},
        socket::NlFamily,
    },
    genl::{AttrTypeBuilder, Genlmsghdr, GenlmsghdrBuilder, NlattrBuilder},
    nl::{NlPayload, NlmsghdrBuilder},
    socket::synchronous::NlSocketHandle,
    types::GenlBuffer,
    utils::Groups,
};
use neli_proc_macros::neli_enum;

#[neli_enum(serialized_type = "u8")]
pub enum Nl80211Command {
    Unspec = 0,
    GetWiphy = 1,
    GetScan = 32,
    TriggerScan = 33,
    NewScanResults = 34,
    ScanAborted = 35,
}

impl neli::consts::genl::Cmd for Nl80211Command {}

#[neli_enum(serialized_type = "u16")]
pub enum Nl80211Attr {
    Unspec = 0,
    Wiphy = 1,
    WiphyName = 2,
    Ifindex = 3,
    Ifname = 4,
    Bss = 47,
}

impl neli::consts::genl::NlAttrType for Nl80211Attr {}

#[neli_enum(serialized_type = "u16")]
pub enum Nl80211BssAttr {
    Unspec = 0,
    Bssid = 1,
    Frequency = 2,
    Tsf = 3,
    BeaconInterval = 4,
    Capability = 5,
    InformationElements = 6,
    SignalMbm = 7,
    SignalUnspec = 8,
    Status = 9,
    SeenMsAgo = 10,
    BeaconIes = 11,
}

impl neli::consts::genl::NlAttrType for Nl80211BssAttr {}

#[derive(Debug, Default)]
pub struct AccessPoint {
    pub bssid: Option<[u8; 6]>,
    pub ssid: Option<String>,
    pub frequency: Option<u32>,
    pub signal_mbm: Option<i32>,
    pub seen_ms_ago: Option<u32>,
}

impl AccessPoint {
    pub fn signal_dbm(&self) -> Option<f64> {
        self.signal_mbm.map(|mbm| mbm as f64 / 100.0)
    }
}

pub struct ApScanner {
    socket: NlSocketHandle,
    family_id: u16,
}

impl ApScanner {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut socket = NlSocketHandle::connect(NlFamily::Generic, None, Groups::empty())?;
        let family_id = resolve_genl_family(&mut socket, "nl80211")?;
        Ok(Self { socket, family_id })
    }

    pub fn scan(&mut self, interface: &str) -> Result<Vec<AccessPoint>, Box<dyn Error>> {
        let if_index = get_interface_index(interface)?;
        self.trigger_scan(if_index)?;
        std::thread::sleep(std::time::Duration::from_secs(2));
        self.get_scan_results(if_index)
    }

    fn trigger_scan(&mut self, if_index: u32) -> Result<(), Box<dyn Error>> {
        let mut attrs = GenlBuffer::new();
        attrs.push(
            NlattrBuilder::default()
                .nla_type(
                    AttrTypeBuilder::default()
                        .nla_type(Nl80211Attr::Ifindex)
                        .build()?,
                )
                .nla_payload(if_index)
                .build()?,
        );

        let genlhdr = GenlmsghdrBuilder::default()
            .cmd(Nl80211Command::TriggerScan)
            .version(1)
            .attrs(attrs)
            .build()?;

        let nlhdr = NlmsghdrBuilder::default()
            .nl_type(self.family_id)
            .nl_flags(NlmF::REQUEST | NlmF::ACK)
            .nl_payload(NlPayload::Payload(genlhdr))
            .build()?;

        self.socket.send(&nlhdr)?;
        let (iter, _) = self.socket.recv::<u16, Genlmsghdr<u8, u16>>()?;
        for msg in iter {
            let _ = msg?;
        }
        Ok(())
    }

    fn get_scan_results(&mut self, if_index: u32) -> Result<Vec<AccessPoint>, Box<dyn Error>> {
        let mut attrs = GenlBuffer::new();
        attrs.push(
            NlattrBuilder::default()
                .nla_type(
                    AttrTypeBuilder::default()
                        .nla_type(Nl80211Attr::Ifindex)
                        .build()?,
                )
                .nla_payload(if_index)
                .build()?,
        );

        let genlhdr = GenlmsghdrBuilder::default()
            .cmd(Nl80211Command::GetScan)
            .version(1)
            .attrs(attrs)
            .build()?;

        let nlhdr = NlmsghdrBuilder::default()
            .nl_type(self.family_id)
            .nl_flags(NlmF::REQUEST | NlmF::DUMP)
            .nl_payload(NlPayload::Payload(genlhdr))
            .build()?;

        self.socket.send(&nlhdr)?;

        let mut access_points = Vec::new();
        loop {
            let (iter, _) = self.socket.recv::<u16, Genlmsghdr<u8, u16>>()?;
            for msg in iter {
                let msg = msg?;
                if *msg.nl_type() == u16::from(Nlmsg::Done) {
                    return Ok(access_points);
                }
                if let NlPayload::Payload(genlmsg) = msg.nl_payload()
                    && let Some(ap) = parse_scan_result(genlmsg)?
                {
                    access_points.push(ap);
                }
            }
        }
    }
}

fn resolve_genl_family(
    socket: &mut NlSocketHandle,
    family_name: &str,
) -> Result<u16, Box<dyn Error>> {
    let mut attrs = GenlBuffer::new();
    attrs.push(
        NlattrBuilder::default()
            .nla_type(
                AttrTypeBuilder::default()
                    .nla_type(CtrlAttr::FamilyName)
                    .build()?,
            )
            .nla_payload(family_name)
            .build()?,
    );

    let genlhdr = GenlmsghdrBuilder::default()
        .cmd(CtrlCmd::Getfamily)
        .version(2)
        .attrs(attrs)
        .build()?;

    let nlhdr = NlmsghdrBuilder::default()
        .nl_type(neli::consts::nl::GenlId::Ctrl)
        .nl_flags(NlmF::REQUEST)
        .nl_payload(NlPayload::Payload(genlhdr))
        .build()?;

    socket.send(&nlhdr)?;
    let (iter, _) = socket.recv::<u16, Genlmsghdr<u8, u16>>()?;
    for msg in iter {
        let msg = msg?;
        if let NlPayload::Payload(genlmsg) = msg.nl_payload() {
            for attr in genlmsg.attrs().iter() {
                if *attr.nla_type().nla_type() == u16::from(CtrlAttr::FamilyId) {
                    let mut cursor = Cursor::new(attr.nla_payload().as_ref());
                    let id = u16::from_bytes(&mut cursor)?;
                    return Ok(id);
                }
            }
        }
    }
    Err("Family not found".into())
}

fn get_interface_index(interface: &str) -> Result<u32, Box<dyn Error>> {
    let path = format!("/sys/class/net/{}/ifindex", interface);
    let index = std::fs::read_to_string(path)?.trim().parse::<u32>()?;
    Ok(index)
}

fn parse_scan_result(genlmsg: &Genlmsghdr<u8, u16>) -> Result<Option<AccessPoint>, Box<dyn Error>> {
    for attr in genlmsg.attrs().iter() {
        if *attr.nla_type().nla_type() == u16::from(Nl80211Attr::Bss) {
            return Ok(Some(parse_bss_info(attr.nla_payload().as_ref())?));
        }
    }
    Ok(None)
}

fn parse_bss_info(bss_data: &[u8]) -> Result<AccessPoint, Box<dyn Error>> {
    let mut ap = AccessPoint::default();
    let mut offset = 0;
    while offset + 4 <= bss_data.len() {
        let len = u16::from_ne_bytes([bss_data[offset], bss_data[offset + 1]]) as usize;
        let type_ = u16::from_ne_bytes([bss_data[offset + 2], bss_data[offset + 3]]);

        if len < 4 {
            break;
        }
        let payload_len = len - 4;
        if offset + 4 + payload_len > bss_data.len() {
            break;
        }

        let payload = &bss_data[offset + 4..offset + 4 + payload_len];
        let type_masked = type_ & 0x3FFF;

        if type_masked == u16::from(Nl80211BssAttr::Bssid) {
            if payload.len() == 6 {
                let mut mac = [0u8; 6];
                mac.copy_from_slice(payload);
                ap.bssid = Some(mac);
            }
        } else if type_masked == u16::from(Nl80211BssAttr::Frequency) {
            if payload.len() == 4 {
                ap.frequency = Some(u32::from_ne_bytes(payload.try_into()?));
            }
        } else if type_masked == u16::from(Nl80211BssAttr::SignalMbm) {
            if payload.len() == 4 {
                ap.signal_mbm = Some(i32::from_ne_bytes(payload.try_into()?));
            }
        } else if type_masked == u16::from(Nl80211BssAttr::InformationElements) {
            ap.ssid = parse_ssid_from_ies(payload);
        } else if type_masked == u16::from(Nl80211BssAttr::SeenMsAgo) && payload.len() == 4 {
            ap.seen_ms_ago = Some(u32::from_ne_bytes(payload.try_into()?));
        }

        offset += (len + 3) & !3;
    }

    Ok(ap)
}

fn parse_ssid_from_ies(ie_data: &[u8]) -> Option<String> {
    let mut offset = 0;
    while offset + 2 <= ie_data.len() {
        let id = ie_data[offset];
        let len = ie_data[offset + 1] as usize;
        if offset + 2 + len > ie_data.len() {
            break;
        }

        if id == 0 {
            // SSID
            return String::from_utf8(ie_data[offset + 2..offset + 2 + len].to_vec()).ok();
        }
        offset += 2 + len;
    }
    None
}
