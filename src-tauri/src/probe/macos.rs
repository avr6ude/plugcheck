//! macOS probe: parse `ioreg` + `system_profiler` into a [`Snapshot`].
//!
//! Data sources (see docs/iokit-keys.md, observed macOS 26.6 / Apple Silicon):
//! - `ioreg -a -r -l -c AppleHPMDevice` — Type-C port controllers
//!   (`AppleTCControllerType1x`) live in this subtree with cable / orientation
//!   / transport state pre-decoded.
//! - `system_profiler -json SPUSBDataType SPThunderboltDataType` — device tree
//!   (filled in by the device-tree pass).

use std::collections::BTreeMap;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

use plist::Value;

use crate::emarker;
use crate::model::{CableType, Charger, EmarkerInfo, Port, ProbeError, Snapshot, Transport};

/// IOKit keys, observed on macOS 26.6.2 (25G83) / Apple Silicon.
mod keys {
    pub const PORT_CLASS_PREFIX: &str = "AppleTCControllerType";
    pub const CHILDREN: &str = "IORegistryEntryChildren";
    pub const OBJ_CLASS: &str = "IOObjectClass";

    pub const PORT_DESC: &str = "PortDescription"; // "Port-USB-C@1" -> Port.id
    pub const PORT_NAME: &str = "IORegistryEntryName"; // "Port-USB-C"
    pub const PORT_NUMBER: &str = "PortNumber";
    pub const KIND_DESC: &str = "PortTypeDescription"; // "USB-C" / "MagSafe 3"
    pub const OCCUPIED: &str = "ConnectionActive";
    pub const ORIENTATION: &str = "PlugOrientation";
    pub const ACTIVE_CABLE: &str = "ActiveCable";
    pub const OPTICAL_CABLE: &str = "OpticalCable";
    pub const TRANSPORTS_ACTIVE: &str = "TransportsActive";
    pub const CURRENT_LIMITS: &str = "IOAccessoryPowerCurrentLimits"; // [mA; 5]
    pub const VENDOR_ID: &str = "Vendor ID"; // present only with an e-marked cable
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
            let ioreg = run("ioreg", &["-a", "-r", "-l", "-c", "AppleHPMDevice"])?;
            let sp = run(
                "system_profiler",
                &["-json", "SPUSBDataType", "SPThunderboltDataType"],
            )?;
            Self::parse_snapshot(&ioreg, &sp)
        }
    }
}

impl MacosProbe {
    /// Pure: the two command outputs → a `Snapshot`. Testable off-device.
    pub fn parse_snapshot(ioreg_plist: &str, _sp_json: &str) -> Result<Snapshot, ProbeError> {
        let root = Value::from_reader_xml(Cursor::new(ioreg_plist.as_bytes()))
            .map_err(|e| ProbeError::ParseFailed(format!("ioreg plist: {e}")))?;

        let mut ports = Vec::new();
        for node in Walk::new(&root) {
            let Some(dict) = node.as_dictionary() else {
                continue;
            };
            let class = dict.get(keys::OBJ_CLASS).and_then(Value::as_string);
            if !class.is_some_and(|c| c.starts_with(keys::PORT_CLASS_PREFIX)) {
                continue;
            }
            ports.push(port_from_node(dict));
        }

        Ok(Snapshot {
            ports,
            captured_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        })
    }
}

fn port_from_node(d: &plist::Dictionary) -> Port {
    let s = |k: &str| d.get(k).and_then(Value::as_string).map(str::to_string);
    let b = |k: &str| d.get(k).and_then(Value::as_boolean);
    let i = |k: &str| {
        d.get(k)
            .and_then(|v| v.as_unsigned_integer().or_else(|| v.as_signed_integer().map(|x| x as u64)))
    };

    let occupied = b(keys::OCCUPIED).unwrap_or(false);

    let active: Vec<String> = d
        .get(keys::TRANSPORTS_ACTIVE)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_string).map(str::to_string).collect())
        .unwrap_or_default();

    let current_limits: Vec<u64> = d
        .get(keys::CURRENT_LIMITS)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_unsigned_integer).collect())
        .unwrap_or_default();
    let max_ma = current_limits.iter().copied().max().unwrap_or(0);

    let vendor_id = i(keys::VENDOR_ID).map(|v| v as u16);
    let active_cable = b(keys::ACTIVE_CABLE).unwrap_or(false);
    // "Has an e-marker" heuristic — refine against an occupied capture.
    let emarker_present =
        occupied && (vendor_id.is_some() || active_cable || max_ma > 0);

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
            max_speed: Transport::None, // not exposed without a real e-marker read
            current_amps,
            max_power_watts: current_amps.and_then(emarker::cable_watts),
            present: true,
        }
    } else {
        EmarkerInfo::default()
    };

    // ponytail: no IOPortFeaturePowerSource on this hardware — synthesize a
    // charger from the port's current-limit array. Real negotiated V/W needs
    // the occupied capture to locate AdapterDetails. Upgrade path: read
    // AppleSmartBattery.AdapterDetails.{Watts,Voltage,Amperage}.
    let charger = if occupied && max_ma > 0 {
        Some(Charger {
            negotiated_volts: None,
            negotiated_amps: Some(max_ma as f32 / 1000.0),
            cable_current_limit_amps: current_amps,
        })
    } else {
        None
    };

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
        emarker,
        charger,
        devices: Vec::new(),
        raw: raw_dump(d),
    }
}

/// Every property of the node as `String`, for Engineer mode.
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
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(scalar).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Dictionary(m) => {
            let parts: Vec<String> = m.iter().map(|(k, x)| format!("{k}={}", scalar(x))).collect();
            format!("{{{}}}", parts.join(", "))
        }
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

    const IOREG: &str = include_str!("../../tests/fixtures/ioreg_ports.plist");
    const SP: &str = include_str!("../../tests/fixtures/system_profiler.json");

    #[test]
    fn parses_baseline_ports() {
        let snap = MacosProbe::parse_snapshot(IOREG, SP).unwrap();
        // MacBook Pro: 3x USB-C + 1x MagSafe.
        assert_eq!(snap.ports.len(), 4);

        let ids: Vec<&str> = snap.ports.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"Port-USB-C@1"));
        assert!(ids.contains(&"Port-MagSafe 3@1"));

        // Nothing was connected at capture time.
        assert!(snap.ports.iter().all(|p| !p.occupied));
        assert!(snap.ports.iter().all(|p| !p.emarker.present));
        assert!(snap.ports.iter().all(|p| p.charger.is_none()));

        // Engineer-mode dump is populated for every port.
        assert!(snap.ports.iter().all(|p| !p.raw.is_empty()));
        let p1 = snap.ports.iter().find(|p| p.id == "Port-USB-C@1").unwrap();
        assert_eq!(p1.kind, "USB-C");
        assert_eq!(p1.active_transport, Transport::None);
        assert!(p1.raw.contains_key("TransportsSupported"));
    }

    #[test]
    fn empty_plist_is_ok_not_err() {
        let snap = MacosProbe::parse_snapshot(
            r#"<?xml version="1.0"?><!DOCTYPE plist><plist version="1.0"><array/></plist>"#,
            SP,
        )
        .unwrap();
        assert!(snap.ports.is_empty());
    }

    #[test]
    fn garbage_plist_is_err() {
        assert!(matches!(
            MacosProbe::parse_snapshot("not a plist", SP),
            Err(ProbeError::ParseFailed(_))
        ));
    }
}
