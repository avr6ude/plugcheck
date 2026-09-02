//! Cable / link decoding.
//!
//! macOS pre-decodes USB-PD e-marker data, so there is no VDO bit-parsing here
//! (see docs/iokit-keys.md). This module maps Apple's decoded strings/arrays
//! onto [`Transport`] and looks up vendor names.

use crate::model::Transport;

/// Map a port's `TransportsActive` string set (from `AppleTCControllerType1x`)
/// to the single highest active [`Transport`].
///
/// Values Apple emits: `CC`, `USB2`, `USB3`, `CIO`, `DisplayPort`, `TBT`.
/// `CIO` ("Converged IO") is the Thunderbolt/USB4 tunnel. The strings carry no
/// USB3 generation — the per-device `speed` in the system_profiler tree is
/// authoritative for that; `USB3` here just means SuperSpeed is up.
pub fn transport_from_active(active: &[String]) -> Transport {
    let has = |s: &str| active.iter().any(|a| a.eq_ignore_ascii_case(s));
    if has("TBT") || has("CIO") {
        // Cannot distinguish TB3 vs TB4/USB4 from this field; report the modern
        // label. The Thunderbolt bus entry in system_profiler refines it.
        Transport::Thunderbolt4
    } else if has("USB3") {
        Transport::Usb3Gen2
    } else if has("USB2") {
        Transport::Usb2
    } else {
        Transport::None
    }
}

/// Map a `system_profiler` `speed` / Thunderbolt `current_speed_key` string.
pub fn speed_str_to_transport(s: &str) -> Transport {
    let n = s.to_ascii_lowercase().replace([' ', '/'], "_");
    match n.as_str() {
        x if x.contains("480_mb") => Transport::Usb2,
        x if x.contains("5_gb") => Transport::Usb3Gen1,
        x if x.contains("10_gb") => Transport::Usb3Gen2,
        x if x.contains("20_gb") => Transport::Usb4Gen3,
        x if x.contains("40_gb") => Transport::Usb4Gen4,
        _ => Transport::None,
    }
}

pub fn transport_label(t: Transport) -> &'static str {
    match t {
        Transport::None => "no link",
        Transport::Usb2 => "USB 2.0 (480 Mb/s)",
        Transport::Usb3Gen1 => "USB 3.2 Gen 1 (5 Gb/s)",
        Transport::Usb3Gen2 => "USB 3.2 Gen 2 (10 Gb/s)",
        Transport::Usb4Gen3 => "USB4 Gen 3 (20 Gb/s)",
        Transport::Usb4Gen4 => "USB4 (40 Gb/s)",
        Transport::Thunderbolt3 => "Thunderbolt 3 (40 Gb/s)",
        Transport::Thunderbolt4 => "Thunderbolt 4 (40 Gb/s)",
        Transport::DisplayPort => "DisplayPort",
    }
}

/// Rough cable power capacity from its current rating. Full USB-PD table is
/// post-MVP; a 5 A cable is EPR-capable (240 W), 3 A is 60 W.
pub fn cable_watts(current_amps: u8) -> Option<u16> {
    match current_amps {
        5 => Some(240),
        3 => Some(60),
        _ => None,
    }
}

/// Vendor name for a USB-IF Vendor ID, from the bundled partial list.
pub fn vendor_name(vid: u16) -> Option<String> {
    const CSV: &str = include_str!("../assets/usbif_vendors.csv");
    for line in CSV.lines().skip(1) {
        let mut it = line.splitn(2, ',');
        let (Some(hex), Some(name)) = (it.next(), it.next()) else {
            continue;
        };
        let hex = hex.trim().trim_start_matches("0x");
        if u16::from_str_radix(hex, 16).ok() == Some(vid) {
            return Some(name.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_transports_pick_highest() {
        assert_eq!(
            transport_from_active(&["CC".into(), "USB2".into(), "USB3".into()]),
            Transport::Usb3Gen2
        );
        assert_eq!(
            transport_from_active(&["CC".into(), "USB2".into(), "CIO".into()]),
            Transport::Thunderbolt4
        );
        assert_eq!(transport_from_active(&["CC".into()]), Transport::None);
        assert_eq!(transport_from_active(&[]), Transport::None);
    }

    #[test]
    fn speed_strings_map() {
        assert_eq!(speed_str_to_transport("up_to_480_Mb_s"), Transport::Usb2);
        assert_eq!(speed_str_to_transport("up_to_10_Gb_s"), Transport::Usb3Gen2);
        assert_eq!(speed_str_to_transport("Up to 40 Gb/s"), Transport::Usb4Gen4);
        assert_eq!(speed_str_to_transport("weird"), Transport::None);
    }

    #[test]
    fn watts_from_current() {
        assert_eq!(cable_watts(5), Some(240));
        assert_eq!(cable_watts(3), Some(60));
        assert_eq!(cable_watts(1), None);
    }

    #[test]
    fn vendor_lookup() {
        assert_eq!(vendor_name(0x05ac).as_deref(), Some("Apple"));
        assert_eq!(vendor_name(0x2ce3).as_deref(), Some("CalDigit"));
        assert_eq!(vendor_name(0xffff), None);
    }
}
