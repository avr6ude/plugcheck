//! Normalized, platform-agnostic snapshot of the machine's USB-C / Thunderbolt
//! ports. The macOS probe fills this in; `verdict` reads it. Everything here is
//! plain data and serializes straight to the frontend.

use std::collections::BTreeMap;

/// A link speed / protocol. Ordered by throughput via [`Transport::rank`] —
/// do NOT derive `PartialOrd`, the declaration order is not the speed order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    None,
    Usb2,       // 480 Mb/s
    Usb3Gen1,   // 5 Gb/s
    Usb3Gen2,   // 10 Gb/s
    Usb4Gen3,   // 20 Gb/s
    Usb4Gen4,   // 40 Gb/s
    Thunderbolt3,
    Thunderbolt4,
    DisplayPort,
}

impl Transport {
    /// Throughput rank, higher = faster. DisplayPort is data-irrelevant → 0.
    pub fn rank(self) -> u8 {
        match self {
            Transport::None | Transport::DisplayPort => 0,
            Transport::Usb2 => 1,
            Transport::Usb3Gen1 => 2,
            Transport::Usb3Gen2 => 3,
            Transport::Usb4Gen3 => 4,
            Transport::Thunderbolt3 => 5,
            Transport::Usb4Gen4 => 6,
            Transport::Thunderbolt4 => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CableType {
    Passive,
    Active,
    Captive,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmarkerInfo {
    pub vendor_id: Option<u16>,
    pub vendor_name: Option<String>,
    pub cable_type: CableType,
    /// The e-marker's claimed highest speed.
    pub max_speed: Transport,
    /// VBUS current rating in amps (3 or 5).
    pub current_amps: Option<u8>,
    /// Cable power capacity in watts (60 / 100 / 240).
    pub max_power_watts: Option<u16>,
    /// `false` => no e-marker on this cable (or nothing connected).
    pub present: bool,
}

impl Default for EmarkerInfo {
    fn default() -> Self {
        EmarkerInfo {
            vendor_id: None,
            vendor_name: None,
            cable_type: CableType::Unknown,
            max_speed: Transport::None,
            current_amps: None,
            max_power_watts: None,
            present: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeviceNode {
    pub name: String,
    pub vendor: Option<String>,
    pub speed: Transport,
    /// USB spec the device reports, e.g. "USB 3.2" (from `bcdUSB`).
    pub usb_version: Option<String>,
    /// Human device class, e.g. "Mass storage", "HID", "Hub" (from `bDeviceClass`).
    pub class: Option<String>,
    /// "05ac:8104" style vendor:product id.
    pub vid_pid: Option<String>,
    pub is_hub: bool,
    pub children: Vec<DeviceNode>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    /// Native pixel resolution, e.g. "2560 x 1440".
    pub pixels: Option<String>,
    pub hz: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Charger {
    pub negotiated_volts: Option<f32>,
    pub negotiated_amps: Option<f32>,
    /// Adapter rated wattage (from `AppleSmartBattery.AdapterDetails.Watts`).
    pub watts: Option<u16>,
    pub is_charging: bool,
    pub fully_charged: bool,
    pub battery_percent: Option<u8>,
    pub minutes_to_full: Option<u32>,
    /// Voltages the adapter advertises (its PDO menu), e.g. [5, 9, 15, 20].
    pub profile_volts: Vec<u16>,
    /// Cable's current cap from its e-marker, for bottleneck blame.
    pub cable_current_limit_amps: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Port {
    /// Stable per physical port, e.g. `"Port-USB-C@1"`.
    pub id: String,
    /// Human port kind, e.g. `"USB-C"`, `"MagSafe 3"`.
    pub kind: String,
    pub occupied: bool,
    pub orientation: Option<u8>,
    /// What is actually negotiated on the link right now.
    pub active_transport: Transport,
    /// Friendly names of what the port itself can carry, e.g.
    /// ["Thunderbolt / USB4", "DisplayPort", "USB 3.2", "USB 2.0"].
    pub supported: Vec<String>,
    /// Friendly names of what has actually been negotiated/provisioned.
    pub provisioned: Vec<String>,
    /// "passive" / "active" / "optical" / "unknown" — known even without an e-marker.
    pub cable_kind: String,
    /// Times a device has connected on this port since boot.
    pub connection_count: Option<u32>,
    /// Physical plug/unplug events since boot.
    pub plug_events: Option<u32>,
    /// Recorded overcurrent faults on this port.
    pub overcurrent_count: Option<u32>,
    /// Display hot-plug detect asserted (a monitor is talking to the port).
    pub hpd: bool,
    /// DisplayPort Alt Mode is carrying video on this port.
    pub dp_alt: bool,
    /// The external monitor this port is driving, if any.
    pub display: Option<DisplayInfo>,
    pub emarker: EmarkerInfo,
    pub charger: Option<Charger>,
    pub devices: Vec<DeviceNode>,
    /// Every scalar IOKit property of the port node, for Engineer mode.
    pub raw: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub ports: Vec<Port>,
    pub captured_ms: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "detail")]
pub enum ProbeError {
    CommandFailed(String),
    ParseFailed(String),
    Unsupported,
}

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeError::CommandFailed(s) => write!(f, "command failed: {s}"),
            ProbeError::ParseFailed(s) => write!(f, "parse failed: {s}"),
            ProbeError::Unsupported => {
                write!(f, "unsupported platform (needs Apple Silicon, macOS 14+)")
            }
        }
    }
}
impl std::error::Error for ProbeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_json_roundtrips_and_eq() {
        let snap = Snapshot {
            captured_ms: 42,
            ports: vec![Port {
                id: "Port-USB-C@1".into(),
                kind: "USB-C".into(),
                occupied: true,
                orientation: Some(1),
                active_transport: Transport::Usb2,
                supported: vec!["USB 3.2".into()],
                provisioned: vec![],
                cable_kind: "unknown".into(),
                connection_count: Some(3),
                plug_events: Some(5),
                overcurrent_count: Some(0),
                hpd: false,
                dp_alt: false,
                display: None,
                emarker: EmarkerInfo {
                    vendor_id: Some(0x05ac),
                    vendor_name: Some("Apple".into()),
                    cable_type: CableType::Passive,
                    max_speed: Transport::Usb4Gen3,
                    current_amps: Some(5),
                    max_power_watts: Some(240),
                    present: true,
                },
                charger: None,
                devices: vec![],
                raw: BTreeMap::new(),
            }],
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
        assert!(json.contains("\"cable_type\":\"passive\""));
        assert!(json.contains("\"active_transport\":\"usb2\""));
    }

    #[test]
    fn probe_error_displays() {
        assert_eq!(
            ProbeError::CommandFailed("ioreg timed out".into()).to_string(),
            "command failed: ioreg timed out"
        );
        assert_eq!(
            ProbeError::Unsupported.to_string(),
            "unsupported platform (needs Apple Silicon, macOS 14+)"
        );
    }

    #[test]
    fn transport_rank_orders_by_speed() {
        assert!(Transport::Thunderbolt4.rank() > Transport::Usb3Gen2.rank());
        assert!(Transport::Usb3Gen2.rank() > Transport::Usb2.rank());
        assert_eq!(Transport::DisplayPort.rank(), 0);
    }
}
