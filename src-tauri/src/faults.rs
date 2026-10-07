//! Mid-session fault detection: compares two consecutive snapshots using the
//! port's own counters, so it catches problems that only show under load.

use serde::Serialize;

use crate::model::{Port, Snapshot};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultKind {
    /// The port cut power because the device or cable drew too much current.
    Overcurrent,
    /// The connection dropped and came back while something stayed plugged in.
    Reconnect,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Fault {
    pub port_id: String,
    pub port: String,
    pub kind: FaultKind,
    pub title: String,
    pub text: String,
}

/// "USB-C 1", "MagSafe 3": the name printed next to the port, not the IOKit id.
pub fn port_label(p: &Port) -> String {
    if p.kind.contains("MagSafe") {
        return p.kind.clone();
    }
    match p.id.rsplit_once('@') {
        Some((_, n)) => format!("{} {n}", p.kind),
        None => p.kind.clone(),
    }
}

fn grew(old: Option<u32>, new: Option<u32>) -> bool {
    matches!((old, new), (Some(o), Some(n)) if n > o)
}

pub fn faults(old: &Snapshot, new: &Snapshot) -> Vec<Fault> {
    let mut out = Vec::new();
    for np in &new.ports {
        let Some(op) = old.ports.iter().find(|p| p.id == np.id) else { continue };
        let port = port_label(np);
        if grew(op.overcurrent_count, np.overcurrent_count) {
            out.push(Fault {
                port_id: np.id.clone(),
                port: port.clone(),
                kind: FaultKind::Overcurrent,
                title: format!("{port}: power cut for drawing too much current"),
                text: "The port shut off power to protect your Mac. Try another cable, or unplug other devices from the same hub.".into(),
            });
        }
        // Plugged in before and after, yet the port saw a new connection: it dropped and came back.
        if op.occupied && np.occupied && (grew(op.connection_count, np.connection_count) || grew(op.plug_events, np.plug_events)) {
            out.push(Fault {
                port_id: np.id.clone(),
                port: port.clone(),
                kind: FaultKind::Reconnect,
                title: format!("{port}: connection dropped and came back"),
                text: "The link reset while the cable stayed plugged in. A loose connector or a marginal cable is the usual cause.".into(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(id: &str, occupied: bool, conn: u32, plugs: u32, oc: u32) -> Port {
        let mut p: Port = serde_json::from_value(serde_json::json!({
            "id": id, "kind": "USB-C", "occupied": occupied, "orientation": null,
            "active_transport": "none", "supported": [], "provisioned": [], "cable_kind": "unknown",
            "connection_count": conn, "plug_events": plugs, "overcurrent_count": oc,
            "hpd": false, "dp_alt": false, "display": null, "history_sig": null, "history": null,
            "emarker": { "vendor_id": null, "vendor_name": null, "cable_type": "unknown", "max_speed": "none",
                         "current_amps": null, "max_power_watts": null, "present": false },
            "charger": null, "devices": [], "raw": {}
        }))
        .expect("fixture");
        p.occupied = occupied;
        p
    }
    fn snap(ports: Vec<Port>) -> Snapshot {
        Snapshot { ports, captured_ms: 0 }
    }

    #[test]
    fn labels_ports_like_the_case_does() {
        assert_eq!(port_label(&port("Port-USB-C@2", false, 0, 0, 0)), "USB-C 2");
    }

    #[test]
    fn overcurrent_increment_is_a_fault() {
        let f = faults(&snap(vec![port("Port-USB-C@1", true, 1, 2, 0)]), &snap(vec![port("Port-USB-C@1", true, 1, 2, 1)]));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, FaultKind::Overcurrent);
    }

    #[test]
    fn reconnect_while_plugged_is_a_fault_but_a_fresh_plug_is_not() {
        let drop = faults(&snap(vec![port("Port-USB-C@1", true, 1, 2, 0)]), &snap(vec![port("Port-USB-C@1", true, 2, 4, 0)]));
        assert_eq!(drop.iter().map(|f| &f.kind).collect::<Vec<_>>(), vec![&FaultKind::Reconnect]);
        let fresh = faults(&snap(vec![port("Port-USB-C@1", false, 1, 2, 0)]), &snap(vec![port("Port-USB-C@1", true, 2, 3, 0)]));
        assert!(fresh.is_empty());
    }
}
