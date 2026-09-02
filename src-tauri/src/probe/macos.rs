//! macOS probe: parse `ioreg` into a [`Snapshot`].
//!
//! Data sources (see docs/iokit-keys.md, observed macOS 26.6 / Apple Silicon):
//! - `ioreg -a -r -l -c AppleHPMDevice` — Type-C port controllers
//!   (`AppleTCControllerType1x`) with cable / orientation / transport state
//!   pre-decoded.
//! - `ioreg -a -l -p IOUSB` — the USB device tree. `system_profiler
//!   SPUSBDataType` is **empty** on this hardware, so IORegistry is the only
//!   source for connected devices.
//! - `system_profiler -json SPThunderboltDataType` — Thunderbolt topology
//!   (only populated for real TB devices; a USB-C dock shows up under IOUSB).

use std::collections::BTreeMap;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

use plist::Value;

use crate::emarker;
use crate::model::{
    CableType, Charger, DeviceNode, EmarkerInfo, Port, ProbeError, Snapshot, Transport,
};

/// IOKit keys, observed on macOS 26.6.2 (25G83) / Apple Silicon.
mod keys {
    pub const PORT_CLASS_PREFIX: &str = "AppleTCControllerType";
    pub const CHILDREN: &str = "IORegistryEntryChildren";
    pub const OBJ_CLASS: &str = "IOObjectClass";

    // port controller
    pub const PORT_DESC: &str = "PortDescription"; // "Port-USB-C@1" -> Port.id
    pub const PORT_NAME: &str = "IORegistryEntryName";
    pub const PORT_NUMBER: &str = "PortNumber";
    pub const KIND_DESC: &str = "PortTypeDescription"; // "USB-C" / "MagSafe 3"
    pub const OCCUPIED: &str = "ConnectionActive";
    pub const ORIENTATION: &str = "PlugOrientation";
    pub const ACTIVE_CABLE: &str = "ActiveCable";
    pub const OPTICAL_CABLE: &str = "OpticalCable";
    pub const TRANSPORTS_ACTIVE: &str = "TransportsActive"; // e.g. ["CC","USB3","DisplayPort"]
    pub const TRANSPORTS_SUPPORTED: &str = "TransportsSupported";
    pub const TRANSPORTS_PROVISIONED: &str = "TransportsProvisioned";
    pub const SUPERSPEED_ACTIVE: &str = "IOAccessoryUSBSuperSpeedActive";
    pub const CURRENT_LIMITS: &str = "IOAccessoryPowerCurrentLimits"; // [mA; 5]
    pub const VENDOR_ID: &str = "Vendor ID";
    pub const CONNECTION_COUNT: &str = "ConnectionCount";
    pub const PLUG_EVENTS: &str = "Plug Event Count";
    pub const OVERCURRENT: &str = "Overcurrent Count";
    pub const HPD: &str = "HPDAsserted";

    // IOUSB device
    pub const USB_DEVICE_CLASS: &str = "IOUSBHostDevice";
    pub const USB_NAME: &str = "USB Product Name";
    pub const USB_VENDOR: &str = "USB Vendor Name";
    pub const USB_LINK_SPEED: &str = "UsbLinkSpeed"; // bits/second
    pub const USB_DEVICE_SPEED: &str = "Device Speed"; // enum fallback
    pub const USB_DEVICE_CLASS_NUM: &str = "bDeviceClass"; // 9 = hub
    pub const USB_LOCATION: &str = "locationID"; // int; >>24 == bus id
    pub const USB_BCD: &str = "bcdUSB"; // BCD USB spec, e.g. 0x0320
    pub const USB_VID: &str = "idVendor";
    pub const USB_PID: &str = "idProduct";
    pub const USB_SERIAL: &str = "kUSBSerialNumberString";
}

/// USB base-class code → human label (USB-IF class list, common subset).
fn usb_class_label(code: i64) -> Option<&'static str> {
    Some(match code {
        1 => "Audio",
        2 => "Communications",
        3 => "HID",
        5 => "Physical",
        6 => "Imaging",
        7 => "Printer",
        8 => "Mass storage",
        9 => "Hub",
        10 => "CDC data",
        11 => "Smart card",
        13 => "Content security",
        14 => "Video",
        16 => "Audio/Video",
        0xdc => "Diagnostic",
        0xe0 => "Wireless",
        0xef => "Miscellaneous",
        0xfe => "Application-specific",
        0xff => "Vendor-specific",
        _ => return None,
    })
}

pub struct MacosProbe;

impl super::UsbProbe for MacosProbe {
    fn snapshot(&self) -> Result<Snapshot, ProbeError> {
        #[cfg(not(target_os = "macos"))]
        {
            Err(ProbeError::Unsupported)
        }
        #[cfg(target_os = "macos")]
        {
            let ports = run("ioreg", &["-a", "-r", "-l", "-c", "AppleHPMDevice"])?;
            let iousb = run("ioreg", &["-a", "-l", "-p", "IOUSB"])?;
            let tb = run("system_profiler", &["-json", "SPThunderboltDataType"])
                .unwrap_or_default();
            let battery = run("ioreg", &["-a", "-r", "-c", "AppleSmartBattery"])
                .unwrap_or_default();
            let displays = run("system_profiler", &["-json", "SPDisplaysDataType"])
                .unwrap_or_default();
            Self::parse_snapshot(&ports, &iousb, &tb, &battery, &displays)
        }
    }
}

impl MacosProbe {
    /// Pure: command outputs → a `Snapshot`. Testable off-device.
    pub fn parse_snapshot(
        ioreg_ports: &str,
        ioreg_iousb: &str,
        sp_thunderbolt_json: &str,
        ioreg_battery: &str,
        sp_displays_json: &str,
    ) -> Result<Snapshot, ProbeError> {
        let root = Value::from_reader_xml(Cursor::new(ioreg_ports.as_bytes()))
            .map_err(|e| ProbeError::ParseFailed(format!("ioreg ports plist: {e}")))?;

        let mut ports: Vec<Port> = Vec::new();
        for node in Walk::new(&root) {
            let Some(dict) = node.as_dictionary() else {
                continue;
            };
            let class = dict.get(keys::OBJ_CLASS).and_then(Value::as_string);
            if !class.is_some_and(|c| c.starts_with(keys::PORT_CLASS_PREFIX)) {
                continue;
            }
            let p = port_from_node(dict);
            // The registry can surface a port under two providers; keep the
            // occupied copy.
            match ports.iter_mut().find(|x| x.id == p.id) {
                Some(existing) => {
                    if p.occupied && !existing.occupied {
                        *existing = p;
                    }
                }
                None => ports.push(p),
            }
        }

        attach_iousb_devices(&mut ports, ioreg_iousb);
        attach_thunderbolt(&mut ports, sp_thunderbolt_json);
        attach_adapter(&mut ports, ioreg_battery);
        attach_displays(&mut ports, sp_displays_json);

        Ok(Snapshot {
            ports,
            captured_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        })
    }
}

// --- USB device tree from `ioreg -p IOUSB` ---

fn attach_iousb_devices(ports: &mut [Port], iousb_plist: &str) {
    let Ok(root) = Value::from_reader_xml(Cursor::new(iousb_plist.as_bytes())) else {
        return;
    };

    // Every IOUSBHostDevice whose parent is NOT another IOUSBHostDevice is a
    // top-level tree (hangs directly off an XHCI controller).
    let mut groups: BTreeMap<u64, Vec<DeviceNode>> = BTreeMap::new();
    collect_usb_roots(&root, false, &mut groups);
    if groups.is_empty() {
        return;
    }

    // Occupied data ports, in port order.
    let mut targets: Vec<usize> = ports
        .iter()
        .enumerate()
        .filter(|(_, p)| p.occupied && p.kind != "MagSafe 3")
        .map(|(i, _)| i)
        .collect();
    if targets.is_empty() {
        // nothing marked occupied — fall back to the first port
        if !ports.is_empty() {
            targets.push(0);
        } else {
            return;
        }
    }

    // Match each USB bus group to a port: prefer a port whose SuperSpeed state
    // matches the group's top speed; otherwise zip remaining groups to
    // remaining ports in order.
    // ponytail: there is no shared key between AppleTCControllerType* and the
    // XHCI buses on this hardware. This heuristic is right for the common
    // one-dock / one-adapter case; a multi-hub oddball may mis-assign.
    let bus_ids: Vec<u64> = groups.keys().copied().collect();
    let mut used_ports = vec![false; ports.len()];
    let mut assign: Vec<(u64, usize)> = Vec::new();

    for &bus in &bus_ids {
        let group_ss = groups[&bus]
            .iter()
            .any(|d| d.speed.rank() >= Transport::Usb3Gen1.rank());
        if let Some(&pi) = targets.iter().find(|&&pi| {
            !used_ports[pi] && port_superspeed(&ports[pi]) == group_ss
        }) {
            used_ports[pi] = true;
            assign.push((bus, pi));
        }
    }
    // leftovers → sorted zip
    let mut free_ports = targets.iter().copied().filter(|&pi| !used_ports[pi]);
    for &bus in &bus_ids {
        if assign.iter().any(|(b, _)| *b == bus) {
            continue;
        }
        if let Some(pi) = free_ports.next() {
            assign.push((bus, pi));
        } else if let Some(&pi) = targets.first() {
            assign.push((bus, pi)); // pile onto the first if we run out
        }
    }

    for (bus, pi) in assign {
        if let Some(devs) = groups.remove(&bus) {
            ports[pi].devices.extend(devs);
        }
    }
}

fn port_superspeed(p: &Port) -> bool {
    p.raw
        .get(keys::SUPERSPEED_ACTIVE)
        .map(|v| v == "true")
        .unwrap_or_else(|| p.active_transport.rank() >= Transport::Usb3Gen1.rank())
}

/// Walk the IOUSB plist; for each `IOUSBHostDevice` that is a *root* (parent is
/// not itself a USB device), build its subtree and bucket it by bus id.
fn collect_usb_roots(
    node: &Value,
    parent_is_usb: bool,
    out: &mut BTreeMap<u64, Vec<DeviceNode>>,
) {
    let nodes: Vec<&Value> = match node.as_array() {
        Some(a) => a.iter().collect(),
        None => vec![node],
    };
    for n in nodes {
        let Some(d) = n.as_dictionary() else { continue };
        let is_usb = d.get(keys::OBJ_CLASS).and_then(Value::as_string)
            == Some(keys::USB_DEVICE_CLASS);

        if is_usb && !parent_is_usb {
            let bus = d
                .get(keys::USB_LOCATION)
                .and_then(int_of)
                .map(|loc| (loc >> 24) & 0xFF)
                .unwrap_or(0);
            out.entry(bus).or_default().push(usb_device_node(d));
            // its children are covered inside usb_device_node
            continue;
        }

        if let Some(kids) = d.get(keys::CHILDREN).and_then(Value::as_array) {
            for k in kids {
                collect_usb_roots(k, is_usb, out);
            }
        }
    }
}

fn usb_device_node(d: &plist::Dictionary) -> DeviceNode {
    let name = d
        .get(keys::USB_NAME)
        .and_then(Value::as_string)
        .or_else(|| d.get(keys::PORT_NAME).and_then(Value::as_string))
        .unwrap_or("USB device")
        .to_string();

    let speed = d
        .get(keys::USB_LINK_SPEED)
        .and_then(int_of)
        .map(emarker::link_speed_to_transport)
        .or_else(|| {
            d.get(keys::USB_DEVICE_SPEED)
                .and_then(|v| v.as_signed_integer())
                .map(emarker::device_speed_enum_to_transport)
        })
        .unwrap_or(Transport::None);

    let is_hub = d
        .get(keys::USB_DEVICE_CLASS_NUM)
        .and_then(|v| v.as_signed_integer())
        == Some(9)
        || name.to_lowercase().contains("hub");

    let children: Vec<DeviceNode> = d
        .get(keys::CHILDREN)
        .and_then(Value::as_array)
        .map(|kids| {
            kids.iter()
                .filter_map(|k| k.as_dictionary())
                .filter(|kd| {
                    kd.get(keys::OBJ_CLASS).and_then(Value::as_string)
                        == Some(keys::USB_DEVICE_CLASS)
                })
                .map(usb_device_node)
                .collect()
        })
        .unwrap_or_default();

    let usb_version = d.get(keys::USB_BCD).and_then(int_of).and_then(|b| {
        // BCD: 0x0320 -> "3.2". bcdUSB is often decimal-encoded in the plist.
        let hi = (b >> 8) & 0xff;
        let lo = (b >> 4) & 0xf;
        if hi == 0 {
            None
        } else {
            Some(format!("USB {hi}.{lo}"))
        }
    });

    let class = d
        .get(keys::USB_DEVICE_CLASS_NUM)
        .and_then(|v| v.as_signed_integer())
        .filter(|&c| c != 0) // 0 = defined at interface level, not useful here
        .and_then(usb_class_label)
        .map(str::to_string);

    let vid_pid = match (
        d.get(keys::USB_VID).and_then(int_of),
        d.get(keys::USB_PID).and_then(int_of),
    ) {
        (Some(v), Some(p)) => Some(format!("{v:04x}:{p:04x}")),
        _ => None,
    };

    DeviceNode {
        name,
        vendor: d
            .get(keys::USB_VENDOR)
            .and_then(Value::as_string)
            .map(str::to_string),
        speed,
        usb_version,
        class,
        vid_pid,
        serial: d
            .get(keys::USB_SERIAL)
            .and_then(Value::as_string)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        is_hub,
        children,
    }
}

fn int_of(v: &Value) -> Option<u64> {
    v.as_unsigned_integer()
        .or_else(|| v.as_signed_integer().map(|x| x as u64))
}

// --- Thunderbolt (real TB devices only) ---

fn attach_thunderbolt(ports: &mut [Port], sp_json: &str) {
    let Ok(sp) = serde_json::from_str::<serde_json::Value>(sp_json) else {
        return;
    };
    let Some(buses) = sp.get("SPThunderboltDataType").and_then(|v| v.as_array()) else {
        return;
    };
    for bus in buses {
        let recpt = bus
            .get("receptacle_1_tag")
            .and_then(|r| r.get("receptacle_id_key"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok());
        let Some(items) = bus.get("_items").and_then(|v| v.as_array()) else {
            continue;
        };
        let devs: Vec<DeviceNode> = items.iter().map(tb_node).collect();
        if devs.is_empty() {
            continue;
        }
        let target = ports
            .iter()
            .position(|p| port_number(&p.id) == recpt)
            .or_else(|| ports.iter().position(|p| p.occupied))
            .or(if ports.is_empty() { None } else { Some(0) });
        if let Some(i) = target {
            ports[i].devices.extend(devs);
        }
    }
}

fn port_number(id: &str) -> Option<u64> {
    id.rsplit('@').next().and_then(|s| s.parse().ok())
}

// --- external displays from `system_profiler SPDisplaysDataType` ---

fn attach_displays(ports: &mut [Port], sp_json: &str) {
    let Ok(sp) = serde_json::from_str::<serde_json::Value>(sp_json) else {
        return;
    };
    let mut externals: Vec<crate::model::DisplayInfo> = Vec::new();
    // walk every "spdisplays_ndrvs" array in the tree
    fn collect(v: &serde_json::Value, out: &mut Vec<crate::model::DisplayInfo>) {
        match v {
            serde_json::Value::Object(m) => {
                if let Some(arr) = m.get("spdisplays_ndrvs").and_then(|x| x.as_array()) {
                    for d in arr {
                        let conn = d
                            .get("spdisplays_connection_type")
                            .and_then(|x| x.as_str())
                            .unwrap_or("");
                        if conn == "spdisplays_internal" {
                            continue; // built-in panel
                        }
                        let name = d
                            .get("_name")
                            .and_then(|x| x.as_str())
                            .unwrap_or("Display")
                            .to_string();
                        let res = d
                            .get("spdisplays_resolution")
                            .or_else(|| d.get("_spdisplays_resolution"))
                            .and_then(|x| x.as_str())
                            .unwrap_or("");
                        // current resolution (from the "W x H @ R Hz" string)
                        let pixels = res
                            .split('@')
                            .next()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        // native panel resolution
                        let native = d
                            .get("_spdisplays_pixels")
                            .and_then(|x| x.as_str())
                            .map(str::to_string)
                            .filter(|s| !s.is_empty());
                        let hz = res
                            .split('@')
                            .nth(1)
                            .and_then(|s| s.trim().trim_end_matches("Hz").trim().parse::<f64>().ok())
                            .map(|f| f.round() as u32);
                        let area = |s: &str| -> Option<u64> {
                            let mut it = s.split('x');
                            let w: u64 = it.next()?.trim().parse().ok()?;
                            let h: u64 = it.next()?.trim().parse().ok()?;
                            Some(w * h)
                        };
                        let degraded = match (&pixels, &native) {
                            (Some(cur), Some(nat)) => matches!(
                                (area(cur), area(nat)),
                                (Some(c), Some(n)) if c + c / 20 < n
                            ),
                            _ => false,
                        };
                        let native_pixels = native.filter(|n| Some(n) != pixels.as_ref());
                        out.push(crate::model::DisplayInfo {
                            name,
                            pixels,
                            native_pixels,
                            hz,
                            degraded,
                        });
                    }
                }
                for x in m.values() {
                    collect(x, out);
                }
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| collect(x, out)),
            _ => {}
        }
    }
    collect(&sp, &mut externals);
    if externals.is_empty() {
        return;
    }

    // Hand each external display to a dp_alt / HPD port, in order.
    let targets: Vec<usize> = ports
        .iter()
        .enumerate()
        .filter(|(_, p)| p.occupied && (p.dp_alt || p.hpd))
        .map(|(i, _)| i)
        .collect();
    for (disp, &pi) in externals.into_iter().zip(targets.iter()) {
        ports[pi].display = Some(disp);
    }
}

// --- power adapter from `ioreg -rc AppleSmartBattery` ---

/// Parse `AdapterDetails` and hang it on the best-guess charging port (the
/// occupied non-MagSafe port with the most devices — i.e. the dock — else the
/// first occupied port). macOS exposes one global adapter, not per-port.
fn attach_adapter(ports: &mut [Port], battery_plist: &str) {
    let Ok(root) = Value::from_reader_xml(Cursor::new(battery_plist.as_bytes())) else {
        return;
    };
    let mut battery = None;
    for n in Walk::new(&root) {
        if let Some(b) = n.as_dictionary() {
            if b.get("AdapterDetails").and_then(Value::as_dictionary).is_some() {
                battery = Some(b.clone());
                break;
            }
        }
    }
    let Some(bat) = battery else { return };
    let d = bat.get("AdapterDetails").and_then(Value::as_dictionary).unwrap();

    let watts = d.get("Watts").and_then(int_of).map(|w| w as u16);
    if watts.unwrap_or(0) == 0 {
        return; // no adapter connected
    }

    let is_charging = bat.get("IsCharging").and_then(Value::as_boolean).unwrap_or(false);
    let fully_charged = bat.get("FullyCharged").and_then(Value::as_boolean).unwrap_or(false);
    let battery_percent = bat.get("CurrentCapacity").and_then(int_of).map(|c| c.min(100) as u8);
    let minutes_to_full = bat
        .get("AvgTimeToFull")
        .or_else(|| bat.get("TimeRemaining"))
        .and_then(int_of)
        .filter(|&m| is_charging && m > 0 && m < 60_000)
        .map(|m| m as u32);
    let volts = d.get("AdapterVoltage").and_then(int_of).map(|mv| mv as f32 / 1000.0);
    let amps = d.get("Current").and_then(int_of).map(|ma| ma as f32 / 1000.0);
    let profile_volts: Vec<u16> = d
        .get("UsbHvcMenu")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|p| p.as_dictionary())
                .filter_map(|p| p.get("MaxVoltage").and_then(int_of))
                .map(|mv| (mv / 1000) as u16)
                .collect()
        })
        .unwrap_or_default();

    let charger = Charger {
        negotiated_volts: volts,
        negotiated_amps: amps,
        watts,
        is_charging,
        fully_charged,
        battery_percent,
        minutes_to_full,
        profile_volts,
        cable_current_limit_amps: None,
    };

    let target = ports
        .iter()
        .enumerate()
        .filter(|(_, p)| p.occupied && p.kind != "MagSafe 3")
        .max_by_key(|(_, p)| device_count(&p.devices))
        .map(|(i, _)| i)
        .or_else(|| ports.iter().position(|p| p.occupied));
    if let Some(i) = target {
        let limit = ports[i].emarker.current_amps;
        ports[i].charger = Some(Charger {
            cable_current_limit_amps: limit,
            ..charger
        });
    }
}

fn device_count(ds: &[DeviceNode]) -> usize {
    ds.iter().map(|d| 1 + device_count(&d.children)).sum()
}

/// Map raw `TransportsSupported` / `TransportsActive` tokens to friendly names.
fn friendly_transports(raw: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for t in raw {
        let name = match t.to_ascii_uppercase().as_str() {
            "CIO" | "TBT" => "Thunderbolt / USB4",
            "DISPLAYPORT" => "DisplayPort",
            "USB3" => "USB 3.2",
            "USB2" => "USB 2.0",
            "CC" => continue, // configuration channel, not user-facing
            _ => continue,
        };
        if !out.iter().any(|x| x == name) {
            out.push(name.to_string());
        }
    }
    out
}

fn tb_node(v: &serde_json::Value) -> DeviceNode {
    let name = v.get("_name").and_then(|x| x.as_str()).unwrap_or("Thunderbolt device");
    let speed = v
        .get("current_speed_key")
        .and_then(|x| x.as_str())
        .map(emarker::speed_str_to_transport)
        .unwrap_or(Transport::Thunderbolt4);
    DeviceNode {
        name: name.to_string(),
        vendor: v.get("vendor_name_key").and_then(|x| x.as_str()).map(str::to_string),
        speed: if speed == Transport::Usb4Gen4 {
            Transport::Thunderbolt4
        } else {
            speed
        },
        usb_version: None,
        class: Some("Thunderbolt".into()),
        vid_pid: None,
        serial: None,
        is_hub: false,
        children: v
            .get("_items")
            .and_then(|x| x.as_array())
            .map(|a| a.iter().map(tb_node).collect())
            .unwrap_or_default(),
    }
}

// --- port controller node ---

fn port_from_node(d: &plist::Dictionary) -> Port {
    let s = |k: &str| d.get(k).and_then(Value::as_string).map(str::to_string);
    let b = |k: &str| d.get(k).and_then(Value::as_boolean);
    let i = |k: &str| d.get(k).and_then(int_of);

    let occupied = b(keys::OCCUPIED).unwrap_or(false);

    let active: Vec<String> = d
        .get(keys::TRANSPORTS_ACTIVE)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_string).map(str::to_string).collect())
        .unwrap_or_default();
    let dp_alt = active.iter().any(|t| t.eq_ignore_ascii_case("displayport"));

    let supported_raw: Vec<String> = d
        .get(keys::TRANSPORTS_SUPPORTED)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_string).map(str::to_string).collect())
        .unwrap_or_default();
    let provisioned_raw: Vec<String> = d
        .get(keys::TRANSPORTS_PROVISIONED)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_string).map(str::to_string).collect())
        .unwrap_or_default();

    let cable_kind = if b(keys::OPTICAL_CABLE).unwrap_or(false) {
        "optical"
    } else if b(keys::ACTIVE_CABLE).unwrap_or(false) {
        "active"
    } else if occupied {
        "passive"
    } else {
        "unknown"
    }
    .to_string();

    let current_limits: Vec<u64> = d
        .get(keys::CURRENT_LIMITS)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_unsigned_integer).collect())
        .unwrap_or_default();
    let max_ma = current_limits.iter().copied().max().unwrap_or(0);

    let vendor_id = i(keys::VENDOR_ID).map(|v| v as u16);
    let active_cable = b(keys::ACTIVE_CABLE).unwrap_or(false);
    let emarker_present = occupied && (vendor_id.is_some() || active_cable || max_ma > 0);

    let current_amps = match max_ma {
        m if m >= 4500 => Some(5u8),
        m if m >= 1500 => Some(3u8),
        _ => None,
    };

    let emarker = if emarker_present {
        EmarkerInfo {
            vendor_id,
            vendor_name: vendor_id.and_then(emarker::vendor_name),
            cable_type: if b(keys::OPTICAL_CABLE).unwrap_or(false) || active_cable {
                CableType::Active
            } else {
                CableType::Passive
            },
            max_speed: Transport::None,
            current_amps,
            max_power_watts: current_amps.and_then(emarker::cable_watts),
            present: true,
        }
    } else {
        EmarkerInfo::default()
    };

    // Charger is filled by attach_adapter from AppleSmartBattery.
    Port {
        id: s(keys::PORT_DESC).unwrap_or_else(|| {
            format!(
                "{}@{}",
                s(keys::PORT_NAME).unwrap_or_else(|| "Port".into()),
                i(keys::PORT_NUMBER).unwrap_or(0)
            )
        }),
        kind: s(keys::KIND_DESC).unwrap_or_else(|| "USB-C".into()),
        occupied,
        orientation: i(keys::ORIENTATION).map(|v| v as u8),
        active_transport: emarker::transport_from_active(&active),
        supported: friendly_transports(&supported_raw),
        provisioned: friendly_transports(&provisioned_raw),
        cable_kind,
        connection_count: i(keys::CONNECTION_COUNT).map(|v| v as u32),
        plug_events: i(keys::PLUG_EVENTS).map(|v| v as u32),
        overcurrent_count: i(keys::OVERCURRENT).map(|v| v as u32),
        hpd: b(keys::HPD).unwrap_or(false),
        dp_alt,
        display: None,
        emarker,
        charger: None,
        devices: Vec::new(),
        raw: raw_dump(d),
    }
}

fn raw_dump(d: &plist::Dictionary) -> BTreeMap<String, String> {
    d.iter()
        .filter(|(k, _)| *k != keys::CHILDREN)
        .map(|(k, v)| (k.clone(), scalar(v)))
        .collect()
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Real(r) => r.to_string(),
        Value::Array(a) => format!("[{}]", a.iter().map(scalar).collect::<Vec<_>>().join(", ")),
        Value::Dictionary(m) => format!(
            "{{{}}}",
            m.iter().map(|(k, x)| format!("{k}={}", scalar(x))).collect::<Vec<_>>().join(", ")
        ),
        Value::Data(b) => format!("<{} bytes>", b.len()),
        other => format!("{other:?}"),
    }
}

/// Depth-first walk over an `ioreg` plist tree (roots may be an array).
struct Walk<'a> {
    stack: Vec<&'a Value>,
}
impl<'a> Walk<'a> {
    fn new(root: &'a Value) -> Self {
        let stack = match root.as_array() {
            Some(a) => a.iter().rev().collect(),
            None => vec![root],
        };
        Walk { stack }
    }
}
impl<'a> Iterator for Walk<'a> {
    type Item = &'a Value;
    fn next(&mut self) -> Option<&'a Value> {
        let node = self.stack.pop()?;
        if let Some(kids) = node
            .as_dictionary()
            .and_then(|d| d.get(keys::CHILDREN))
            .and_then(Value::as_array)
        {
            self.stack.extend(kids.iter().rev());
        }
        Some(node)
    }
}

#[cfg(target_os = "macos")]
fn run(cmd: &str, args: &[&str]) -> Result<String, ProbeError> {
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::time::Duration;

    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| ProbeError::CommandFailed(format!("{cmd}: {e}")))?;

    let (tx, rx) = mpsc::channel();
    let out = child.stdout.take().unwrap();
    std::thread::spawn(move || {
        let mut buf = String::new();
        use std::io::Read;
        let mut out = out;
        let _ = out.read_to_string(&mut buf);
        let _ = tx.send(buf);
    });

    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(buf) => {
            let _ = child.wait();
            Ok(buf)
        }
        Err(_) => {
            let _ = child.kill();
            Err(ProbeError::CommandFailed(format!("{cmd} timed out")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASELINE: &str = include_str!("../../tests/fixtures/ioreg_ports.plist");
    const OCC_PORTS: &str = include_str!("../../tests/fixtures/ioreg_ports_occupied.plist");
    const OCC_USB: &str = include_str!("../../tests/fixtures/ioreg_iousb_occupied.plist");
    const BATTERY: &str = include_str!("../../tests/fixtures/ioreg_battery_charging.plist");
    const NO_TB: &str = r#"{"SPThunderboltDataType":[]}"#;
    const NO_BATT: &str = "";
    const NO_DISP: &str = "";

    fn count(d: &DeviceNode) -> usize {
        1 + d.children.iter().map(count).sum::<usize>()
    }
    fn all_devices(s: &Snapshot) -> Vec<&DeviceNode> {
        fn rec<'a>(d: &'a DeviceNode, out: &mut Vec<&'a DeviceNode>) {
            out.push(d);
            d.children.iter().for_each(|c| rec(c, out));
        }
        let mut out = Vec::new();
        for p in &s.ports {
            for d in &p.devices {
                rec(d, &mut out);
            }
        }
        out
    }

    #[test]
    fn baseline_four_empty_ports_no_devices() {
        let snap = MacosProbe::parse_snapshot(BASELINE, "", NO_TB, NO_BATT, NO_DISP).unwrap();
        assert_eq!(snap.ports.len(), 4);
        assert!(snap.ports.iter().all(|p| !p.occupied));
        assert!(snap.ports.iter().all(|p| p.devices.is_empty()));
        assert!(snap.ports.iter().all(|p| !p.dp_alt));
    }

    #[test]
    fn garbage_ports_plist_is_err() {
        assert!(matches!(
            MacosProbe::parse_snapshot("not a plist", "", NO_TB, NO_BATT, NO_DISP),
            Err(ProbeError::ParseFailed(_))
        ));
    }

    #[test]
    fn occupied_capture_two_ports_active() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        assert_eq!(snap.ports.len(), 4, "3 USB-C + MagSafe, deduped");
        let occ: Vec<&Port> = snap.ports.iter().filter(|p| p.occupied).collect();
        assert_eq!(occ.len(), 2);
    }

    #[test]
    fn dp_cable_port_is_flagged_dp_alt() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        // Port 3 had TransportsActive ["CC","USB2","DisplayPort"].
        let p3 = snap.ports.iter().find(|p| p.id == "Port-USB-C@3").unwrap();
        assert!(p3.dp_alt);
        assert!(p3.occupied);
    }

    #[test]
    fn hub_tree_is_attached_and_nested() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        let devs = all_devices(&snap);
        let names: Vec<&str> = devs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"USB3.1 Hub"), "got {names:?}");
        assert!(names.contains(&"USB 10_100_1000 LAN"));
        assert!(names.contains(&"LG Monitor Controls"));
        assert!(names.contains(&"USB-C To DP Cable"));

        let hub = devs.iter().find(|d| d.name == "USB3.1 Hub").unwrap();
        assert!(hub.is_hub);
        assert!(count(hub) >= 3, "hub has nested children");
        assert_eq!(hub.speed, Transport::Usb3Gen1); // 5 Gb/s
        assert_eq!(hub.vendor.as_deref(), Some("GenesysLogic"));
    }

    #[test]
    fn devices_land_on_occupied_ports_only() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        for p in &snap.ports {
            if !p.devices.is_empty() {
                assert!(p.occupied, "devices on an unoccupied port: {}", p.id);
            }
        }
    }

    #[test]
    fn adapter_attaches_to_the_dock_port() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, BATTERY, NO_DISP).unwrap();
        let charged: Vec<&Port> = snap.ports.iter().filter(|p| p.charger.is_some()).collect();
        assert_eq!(charged.len(), 1, "one port carries the adapter");
        let c = charged[0].charger.as_ref().unwrap();
        assert_eq!(c.watts, Some(50));
        assert_eq!(c.negotiated_volts, Some(20.0));
        assert_eq!(c.profile_volts, vec![5, 9, 15, 20]);
        // the dock (most devices), not the DP-cable port
        assert!(!charged[0].devices.is_empty());
    }

    #[test]
    fn port_capabilities_are_friendly() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        let p1 = snap.ports.iter().find(|p| p.id == "Port-USB-C@1").unwrap();
        assert!(p1.supported.contains(&"Thunderbolt / USB4".to_string()));
        assert!(p1.supported.contains(&"DisplayPort".to_string()));
        assert!(!p1.supported.iter().any(|s| s == "CC"));
    }

    #[test]
    fn usb_version_decoded_from_bcd() {
        let snap = MacosProbe::parse_snapshot(OCC_PORTS, OCC_USB, NO_TB, NO_BATT, NO_DISP).unwrap();
        let mut all = Vec::new();
        fn rec<'a>(d: &'a DeviceNode, o: &mut Vec<&'a DeviceNode>) {
            o.push(d);
            d.children.iter().for_each(|c| rec(c, o));
        }
        for p in &snap.ports {
            for d in &p.devices {
                rec(d, &mut all);
            }
        }
        let hub = all.iter().find(|d| d.name == "USB3.1 Hub").unwrap();
        assert_eq!(hub.usb_version.as_deref(), Some("USB 3.2"));
    }
}
