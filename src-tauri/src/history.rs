//! "Have I seen this cable/dock before?" — a JSON store keyed by a stable
//! signature derived from the devices hanging off a port.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;

use crate::model::{HistoryEntry, Port, Snapshot};

type Store = BTreeMap<String, HistoryEntry>;

fn path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    let _ = std::fs::create_dir_all(&dir);
    dir.join("history.json")
}

fn load(app: &tauri::AppHandle) -> Store {
    std::fs::read_to_string(path(app))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write(app: &tauri::AppHandle, s: &Store) {
    if let Ok(j) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(path(app), j);
    }
}

/// Days since the Unix epoch as a `YYYY-MM-DD` string (UTC, no chrono dep).
fn today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    // civil-from-days (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// A stable id for whatever is plugged into this port.
pub fn signature(p: &Port) -> Option<String> {
    if !p.occupied {
        return None;
    }
    let mut ids: Vec<String> = Vec::new();
    fn walk(d: &crate::model::DeviceNode, out: &mut Vec<String>) {
        if let Some(v) = &d.vid_pid {
            out.push(v.clone());
        }
        d.children.iter().for_each(|c| walk(c, out));
    }
    p.devices.iter().for_each(|d| walk(d, &mut ids));
    ids.sort();
    ids.dedup();
    if !ids.is_empty() {
        return Some(format!("dev:{}", ids.join("+")));
    }
    if let Some(v) = p.emarker.vendor_id.filter(|_| p.emarker.present) {
        return Some(format!("emk:{v:04x}"));
    }
    None
}

/// Stamp each occupied port with its history entry, bumping the store.
pub fn touch(app: &tauri::AppHandle, snap: &mut Snapshot) {
    let mut store = load(app);
    let day = today();
    let mut dirty = false;

    for p in snap.ports.iter_mut() {
        let Some(sig) = signature(p) else { continue };
        let e = store.entry(sig.clone()).or_insert_with(|| {
            dirty = true;
            HistoryEntry {
                name: None,
                count: 0,
                first_seen: day.clone(),
                last_seen: day.clone(),
            }
        });
        if e.last_seen != day {
            e.count += 1;
            e.last_seen = day.clone();
            dirty = true;
        } else if e.count == 0 {
            e.count = 1;
            dirty = true;
        }
        p.history_sig = Some(sig);
        p.history = Some(e.clone());
    }

    if dirty {
        write(app, &store);
    }
}

pub fn rename(app: &tauri::AppHandle, sig: &str, name: Option<String>) {
    let mut store = load(app);
    if let Some(e) = store.get_mut(sig) {
        e.name = name.filter(|s| !s.trim().is_empty());
        write(app, &store);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_is_well_formed() {
        let t = today();
        assert_eq!(t.len(), 10);
        assert_eq!(&t[4..5], "-");
        let y: i64 = t[..4].parse().unwrap();
        assert!(y >= 2024 && y < 2100);
        let m: i64 = t[5..7].parse().unwrap();
        let d: i64 = t[8..].parse().unwrap();
        assert!((1..=12).contains(&m) && (1..=31).contains(&d));
    }

    #[test]
    fn signature_uses_sorted_device_ids() {
        use crate::model::*;
        let mk = |vp: &str| DeviceNode {
            name: "x".into(),
            vendor: None,
            speed: Transport::None,
            usb_version: None,
            class: None,
            vid_pid: Some(vp.into()),
            serial: None,
            is_hub: false,
            children: vec![],
        };
        let mut p: Port = serde_json::from_str(
            r#"{"id":"P","kind":"USB-C","occupied":true,"orientation":null,
            "active_transport":"none","supported":[],"provisioned":[],"cable_kind":"passive",
            "connection_count":null,"plug_events":null,"overcurrent_count":null,"hpd":false,
            "dp_alt":false,"display":null,"history_sig":null,"history":null,
            "emarker":{"vendor_id":null,"vendor_name":null,"cable_type":"unknown","max_speed":"none",
            "current_amps":null,"max_power_watts":null,"present":false},
            "charger":null,"devices":[],"raw":{}}"#,
        )
        .unwrap();
        p.devices = vec![mk("bbbb:2222"), mk("aaaa:1111")];
        assert_eq!(
            signature(&p).as_deref(),
            Some("dev:aaaa:1111+bbbb:2222")
        );
    }
}
