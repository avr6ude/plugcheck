//! Pure verdict logic: a [`Snapshot`] in, one plain-language verdict per port
//! out. Never touches the OS. This is where "what's the bottleneck" lives.

use crate::emarker::transport_label;
use crate::model::{Port, Snapshot, Transport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blame {
    Port,
    Cable,
    Device,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    Port,
    Data,
    Charging,
    Display,
    Cable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardStatus {
    Ok,
    Warn,
    Bad,
    Idle,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VerdictCard {
    pub kind: CardKind,
    pub status: CardStatus,
    pub title: String,
    /// One-line prose. Empty when `rows` carries the content instead.
    pub text: String,
    /// Structured label/value rows (used by Charging and Display).
    #[serde(default)]
    pub rows: Vec<[String; 2]>,
}

impl VerdictCard {
    fn prose(kind: CardKind, status: CardStatus, title: &str, text: String) -> Self {
        VerdictCard {
            kind,
            status,
            title: title.into(),
            text,
            rows: vec![],
        }
    }
}

fn row(k: &str, v: impl Into<String>) -> [String; 2] {
    [k.to_string(), v.into()]
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortVerdict {
    pub port_id: String,
    pub headline: String,
    /// Verdict cards, in display order (data, charging, display, cable).
    pub cards: Vec<VerdictCard>,
    pub trust_flags: Vec<String>,
    // legacy flat fields, kept for tests / any older caller
    pub data_line: String,
    pub data_blame: Blame,
    pub charging_line: Option<String>,
}

pub fn verdicts(snap: &Snapshot) -> Vec<PortVerdict> {
    snap.ports.iter().map(one).collect()
}

fn blame_status(b: Blame) -> CardStatus {
    match b {
        Blame::None => CardStatus::Ok,
        Blame::Cable | Blame::Port => CardStatus::Bad,
        Blame::Device => CardStatus::Warn,
    }
}

fn cable_text(p: &Port) -> String {
    let em = &p.emarker;
    if !em.present {
        return "No e-marker — cable capabilities unknown.".into();
    }
    let mut bits: Vec<String> = Vec::new();
    if let Some(v) = &em.vendor_name {
        bits.push(v.clone());
    } else if let Some(id) = em.vendor_id {
        bits.push(format!("VID {id:#06x}"));
    }
    bits.push(
        match em.cable_type {
            crate::model::CableType::Active => "active cable",
            crate::model::CableType::Passive => "passive cable",
            crate::model::CableType::Captive => "captive cable",
            crate::model::CableType::Unknown => "cable",
        }
        .into(),
    );
    if let Some(w) = em.max_power_watts {
        bits.push(format!("up to {w} W"));
    }
    if let Some(a) = em.current_amps {
        bits.push(format!("{a} A"));
    }
    bits.join(" · ")
}

fn cards(p: &Port) -> Vec<VerdictCard> {
    if !p.occupied {
        return Vec::new();
    }
    let mut out = Vec::new();
    let (data_line, data_blame) = data(p);
    let dev = fastest_device(p);
    let dp_only = p.dp_alt && dev.rank() <= Transport::Usb2.rank();

    if !p.supported.is_empty() {
        let mut text = format!("Handles {}.", p.supported.join(", "));
        match p.cable_kind.as_str() {
            "active" => text.push_str(" Active cable."),
            "optical" => text.push_str(" Optical cable."),
            _ => {}
        }
        if p.provisioned.iter().any(|t| t == "DisplayPort") && !p.dp_alt {
            text.push_str(" DisplayPort negotiated but no display is active.");
        }
        out.push(VerdictCard::prose(
            CardKind::Port,
            CardStatus::Idle,
            "Port",
            text,
        ));
    }

    if !dp_only {
        out.push(VerdictCard::prose(
            CardKind::Data,
            blame_status(data_blame),
            "Data speed",
            data_line,
        ));
    }

    if p.dp_alt || p.display.is_some() {
        out.push(display_card(p, dp_only));
    }

    if let Some(c) = charging_card(p) {
        out.push(c);
    }

    out.push(VerdictCard::prose(
        CardKind::Cable,
        CardStatus::Idle,
        "Cable",
        cable_text(p),
    ));

    out
}

fn display_card(p: &Port, dp_only: bool) -> VerdictCard {
    let Some(d) = &p.display else {
        let text = if dp_only {
            "DisplayPort video is active. USB data is USB 2.0 — normal for a video adapter.".into()
        } else {
            "DisplayPort video is active.".into()
        };
        return VerdictCard::prose(CardKind::Display, CardStatus::Ok, "Display", text);
    };

    let mut rows = Vec::new();
    rows.push(row("Monitor", d.name.clone()));
    match (&d.pixels, d.hz) {
        (Some(px), Some(hz)) => rows.push(row(
            "Resolution",
            format!("{} @ {hz} Hz", px.replace(" x ", " × ")),
        )),
        (Some(px), None) => rows.push(row("Resolution", px.replace(" x ", " × "))),
        _ => {}
    }
    if let Some(c) = &d.connection {
        rows.push(row("Link", c.clone()));
    }
    if let Some(dep) = &d.depth {
        // "30-Bit Color (ARGB2101010)" -> "30-bit"
        let short = dep
            .split_whitespace()
            .next()
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| dep.clone());
        rows.push(row("Colour", short));
    }
    if d.hdr {
        rows.push(row("HDR", "Yes"));
    }
    if d.mirrored {
        rows.push(row("Mode", "Mirrored"));
    }

    let status = if d.degraded {
        if let Some(nat) = &d.native_pixels {
            rows.push(row("Native", nat.replace(" x ", " × ")));
        }
        CardStatus::Warn
    } else {
        CardStatus::Ok
    };

    VerdictCard {
        kind: CardKind::Display,
        status,
        title: "Display".into(),
        text: if dp_only {
            "USB data is USB 2.0 — normal for a video adapter.".into()
        } else {
            String::new()
        },
        rows,
    }
}

fn charging_card(p: &Port) -> Option<VerdictCard> {
    let c = p.charger.as_ref()?;
    let mut rows = Vec::new();
    let mut status = CardStatus::Ok;

    if c.fully_charged || !c.is_charging {
        if let Some(w) = c.watts {
            let vi = match (c.negotiated_volts, c.negotiated_amps) {
                (Some(v), Some(a)) => format!(" ({v:.0} V / {a:.1} A)"),
                _ => String::new(),
            };
            rows.push(row("Adapter", format!("{w} W{vi}")));
        }
        rows.push(row("Status", "Battery full — not drawing power"));
    } else {
        match (c.watts, c.negotiated_volts, c.negotiated_amps) {
            (Some(w), Some(v), Some(a)) => {
                rows.push(row("Power", format!("{w} W ({v:.0} V / {a:.1} A)")))
            }
            (Some(w), _, _) => rows.push(row("Power", format!("{w} W"))),
            (None, Some(v), Some(a)) => {
                rows.push(row("Power", format!("~{:.0} W ({v:.0} V / {a:.1} A)", v * a)))
            }
            _ => {
                rows.push(row("Power", "wattage unavailable"));
                status = CardStatus::Warn;
            }
        }
        if let Some(lw) = c.live_watts {
            rows.push(row("Now", format!("{lw:.0} W to battery")));
        }
        if let Some(pct) = c.battery_percent {
            let v = match c.minutes_to_full {
                Some(m) => format!("{pct}% · full in {}", fmt_minutes(m)),
                None => format!("{pct}%"),
            };
            rows.push(row("Battery", v));
        }
    }

    if p.emarker.current_amps == Some(3) && c.watts.map_or(true, |w| w > 60) {
        rows.push(row("Cable limit", "~60 W (3 A cable)"));
        status = CardStatus::Warn;
    }

    Some(VerdictCard {
        kind: CardKind::Charging,
        status,
        title: "Charging".into(),
        text: String::new(),
        rows,
    })
}

fn fastest_device(p: &Port) -> Transport {
    fn deep(d: &crate::model::DeviceNode) -> Transport {
        d.children
            .iter()
            .map(deep)
            .chain(std::iter::once(d.speed))
            .max_by_key(|t| t.rank())
            .unwrap_or(Transport::None)
    }
    p.devices
        .iter()
        .map(deep)
        .max_by_key(|t| t.rank())
        .unwrap_or(Transport::None)
}

fn is_thunderbolt(t: Transport) -> bool {
    matches!(t, Transport::Thunderbolt3 | Transport::Thunderbolt4)
}

fn headline(p: &Port) -> String {
    if !p.occupied {
        return "Empty".into();
    }
    let dev = fastest_device(p);
    if is_thunderbolt(p.active_transport) || p.devices.iter().any(|d| is_thunderbolt(d.speed)) {
        return if matches!(p.active_transport, Transport::Thunderbolt3) {
            "Thunderbolt 3".into()
        } else {
            "Thunderbolt 4".into()
        };
    }
    let looks_display = |n: &str| {
        let n = n.to_lowercase();
        n.contains("display") || n.contains("monitor")
    };
    // DisplayPort Alt Mode carrying video, and no faster-than-USB2 data device
    // → this is a video adapter/cable.
    if p.dp_alt && dev.rank() <= Transport::Usb2.rank() {
        return "Display".into();
    }
    if p.devices.iter().any(|d| looks_display(&d.name)) {
        return "Display".into();
    }
    if dev.rank() > 0 || p.devices.iter().any(|d| !d.is_hub) {
        return "USB device".into();
    }
    if p.charger.is_some() {
        return "Charging only".into();
    }
    "Connected".into()
}

fn data(p: &Port) -> (String, Blame) {
    let active = p.active_transport;
    let dev = fastest_device(p);
    let em = &p.emarker;
    let target = [dev.rank(), if em.present { em.max_speed.rank() } else { 0 }]
        .into_iter()
        .max()
        .unwrap();

    // A video adapter with only slow USB alongside is working as intended.
    if p.dp_alt && dev.rank() <= Transport::Usb2.rank() {
        return (
            "DisplayPort video is active; USB data is USB 2.0 (normal for a video adapter)."
                .into(),
            Blame::None,
        );
    }

    if active.rank() == 0 && dev.rank() == 0 {
        return ("No data device connected.".into(), Blame::None);
    }
    if active.rank() >= target {
        // Report the device's actual speed when we have it — the port only
        // reports a coarse "USB3" with no generation.
        let shown = if dev.rank() > 0 { dev } else { active };
        return (
            format!("Running at full speed ({}).", transport_label(shown)),
            Blame::None,
        );
    }
    if em.present && em.max_speed.rank() < dev.rank() && active == em.max_speed {
        return (
            format!(
                "Cable caps this link at {}; the device supports {}.",
                transport_label(em.max_speed),
                transport_label(dev)
            ),
            Blame::Cable,
        );
    }
    if !em.present && active == Transport::Usb2 && dev.rank() > Transport::Usb2.rank() {
        return (
            "Link fell back to USB 2.0 — likely a charge-only cable.".into(),
            Blame::Cable,
        );
    }
    if active.rank() < dev.rank() {
        if em.present {
            return (
                "Port is negotiating below the cable and device capability.".into(),
                Blame::Port,
            );
        }
        return (
            format!(
                "Link is {} but the device can do {}.",
                transport_label(active),
                transport_label(dev)
            ),
            Blame::Cable,
        );
    }
    (
        format!("Limited by the device ({}).", transport_label(dev)),
        Blame::Device,
    )
}

fn one(p: &Port) -> PortVerdict {
    let (data_line, data_blame) = data(p);
    PortVerdict {
        port_id: p.id.clone(),
        headline: headline(p),
        cards: cards(p),
        trust_flags: trust_flags(p),
        data_line,
        data_blame,
        charging_line: charging_line(p),
    }
}

// --- charging + trust (plan Task 8) ---

fn fmt_minutes(m: u32) -> String {
    if m < 60 {
        format!("{m} min")
    } else {
        format!("{}h {:02}m", m / 60, m % 60)
    }
}

fn charging_line(p: &Port) -> Option<String> {
    let c = p.charger.as_ref()?;
    let vi = match (c.negotiated_volts, c.negotiated_amps) {
        (Some(v), Some(a)) => format!(" ({v:.0} V / {a:.1} A)"),
        _ => String::new(),
    };

    let mut s = if c.fully_charged || !c.is_charging {
        match c.watts {
            Some(w) => format!("Adapter connected ({w} W{vi}) — battery full, not drawing power."),
            None => "Adapter connected — battery is full.".into(),
        }
    } else {
        let mut t = match (c.watts, c.negotiated_volts, c.negotiated_amps) {
            (Some(w), _, _) => format!("Charging at {w} W{vi}"),
            (None, Some(v), Some(a)) => format!("Charging at ~{:.0} W{vi}", v * a),
            _ => "Charging (wattage unavailable)".into(),
        };
        if let Some(pct) = c.battery_percent {
            t.push_str(&format!(" — battery {pct}%"));
        }
        if let Some(m) = c.minutes_to_full {
            t.push_str(&format!(", full in {}", fmt_minutes(m)));
        }
        t.push('.');
        t
    };

    if c.is_charging {
        if let Some(lw) = c.live_watts {
            s.push_str(&format!(" Now {lw:.0} W into the battery."));
        }
    }
    if p.emarker.current_amps == Some(3) && c.watts.map_or(true, |w| w > 60) {
        s.push_str(" The cable's 3 A rating limits this to ~60 W.");
    }
    if !c.pdos.is_empty() {
        let v: Vec<String> = c
            .pdos
            .iter()
            .map(|p| format!("{} V/{:.1} A", p.volts, p.amps))
            .collect();
        s.push_str(&format!(" Adapter profiles: {}.", v.join(", ")));
    } else if !c.profile_volts.is_empty() {
        let v: Vec<String> = c.profile_volts.iter().map(|v| format!("{v} V")).collect();
        s.push_str(&format!(" Adapter offers {}.", v.join(", ")));
    }
    Some(s)
}

fn trust_flags(p: &Port) -> Vec<String> {
    let mut out = Vec::new();
    let em = &p.emarker;
    if em.present && em.vendor_id == Some(0) {
        out.push("E-marker vendor ID is 0x0000 (not registered with USB-IF).".into());
    }
    if em.current_amps == Some(5) && em.max_speed == Transport::Usb2 {
        out.push("Cable claims 5 A but the link is only USB 2.0.".into());
    }
    if let Some(n) = p.overcurrent_count.filter(|&n| n > 0) {
        out.push(format!(
            "This port has recorded {n} overcurrent event{}.",
            if n == 1 { "" } else { "s" }
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CableType, Charger, DeviceNode, EmarkerInfo, Snapshot};

    fn dev(name: &str, speed: Transport) -> DeviceNode {
        DeviceNode {
            name: name.into(),
            vendor: None,
            speed,
            usb_version: None,
            class: None,
            vid_pid: None,
            serial: None,
            is_hub: false,
            children: vec![],
        }
    }

    fn snap(
        occupied: bool,
        active: Transport,
        devices: Vec<DeviceNode>,
        emarker: EmarkerInfo,
        charger: Option<Charger>,
    ) -> Snapshot {
        snap_dp(occupied, active, false, devices, emarker, charger)
    }

    fn snap_dp(
        occupied: bool,
        active: Transport,
        dp_alt: bool,
        devices: Vec<DeviceNode>,
        emarker: EmarkerInfo,
        charger: Option<Charger>,
    ) -> Snapshot {
        Snapshot {
            captured_ms: 0,
            ports: vec![Port {
                id: "Port-USB-C@1".into(),
                kind: "USB-C".into(),
                occupied,
                orientation: Some(1),
                active_transport: active,
                supported: vec!["USB 3.2".into(), "USB 2.0".into()],
                provisioned: vec![],
                cable_kind: "passive".into(),
                connection_count: Some(2),
                plug_events: Some(4),
                overcurrent_count: Some(0),
                hpd: false,
                dp_alt,
                display: None,
                history_sig: None,
                history: None,
                emarker,
                charger,
                devices,
                raw: Default::default(),
            }],
        }
    }

    fn em(vid: Option<u16>, max: Transport, amps: Option<u8>) -> EmarkerInfo {
        EmarkerInfo {
            vendor_id: vid,
            vendor_name: None,
            cable_type: CableType::Passive,
            max_speed: max,
            current_amps: amps,
            max_power_watts: amps.and_then(crate::emarker::cable_watts),
            present: true,
        }
    }

    #[test]
    fn tb4_good_is_full_speed_no_blame() {
        let s = snap(
            true,
            Transport::Thunderbolt4,
            vec![dev("CalDigit TS4", Transport::Thunderbolt4)],
            EmarkerInfo::default(),
            None,
        );
        let v = &verdicts(&s)[0];
        assert_eq!(v.headline, "Thunderbolt 4");
        assert_eq!(v.data_blame, Blame::None);
        assert!(v.data_line.contains("full speed"));
    }

    #[test]
    fn no_emarker_usb2_fallback_blames_cable() {
        let s = snap(
            true,
            Transport::Usb2,
            vec![dev("Portable SSD", Transport::Usb3Gen2)],
            EmarkerInfo::default(),
            None,
        );
        let v = &verdicts(&s)[0];
        assert_eq!(v.data_blame, Blame::Cable);
        assert!(v.data_line.contains("charge-only cable"));
    }

    #[test]
    fn connected_headline_when_no_devices_no_charger() {
        let s = snap(true, Transport::None, vec![], EmarkerInfo::default(), None);
        let v = &verdicts(&s)[0];
        assert_eq!(v.headline, "Connected");
        assert_eq!(v.data_blame, Blame::None);
    }

    #[test]
    fn charging_only_headline_when_charger_present() {
        let s = snap(
            true,
            Transport::None,
            vec![],
            EmarkerInfo::default(),
            Some(Charger {
                watts: None,
                is_charging: true,
            fully_charged: false,
            battery_percent: Some(58),
            minutes_to_full: Some(129),
            live_watts: Some(24.0),
                profile_volts: vec![],
            pdos: vec![],
                negotiated_volts: None,
                negotiated_amps: Some(3.0),
                cable_current_limit_amps: None,
            }),
        );
        assert_eq!(verdicts(&s)[0].headline, "Charging only");
    }

    #[test]
    fn dp_adapter_reads_as_display_not_usb2() {
        let s = snap_dp(
            true,
            Transport::Usb2,
            true,
            vec![dev("USB-C To DP Cable", Transport::Usb2)],
            EmarkerInfo::default(),
            None,
        );
        let v = &verdicts(&s)[0];
        assert_eq!(v.headline, "Display");
        assert_eq!(v.data_blame, Blame::None);
        assert!(v.data_line.contains("DisplayPort video"));
    }

    #[test]
    fn empty_port_headline() {
        let s = snap(false, Transport::None, vec![], EmarkerInfo::default(), None);
        assert_eq!(verdicts(&s)[0].headline, "Empty");
    }

    #[test]
    fn emarker_mismatch_blames_cable_and_flags_trust() {
        let s = snap(
            true,
            Transport::Usb2,
            vec![dev("NVMe", Transport::Usb3Gen2)],
            em(Some(1452), Transport::Usb2, Some(5)),
            None,
        );
        let v = &verdicts(&s)[0];
        assert_eq!(v.data_blame, Blame::Cable);
        assert!(v.data_line.contains("Cable caps"));
        assert!(v
            .trust_flags
            .iter()
            .any(|f| f.contains("5 A") && f.contains("USB 2.0")));
    }

    #[test]
    fn charging_line_names_the_3a_cable() {
        let s = snap(
            true,
            Transport::None,
            vec![],
            em(None, Transport::None, Some(3)),
            Some(Charger {
                watts: None,
                is_charging: true,
            fully_charged: false,
            battery_percent: Some(58),
            minutes_to_full: Some(129),
            live_watts: Some(24.0),
                profile_volts: vec![],
            pdos: vec![],
                negotiated_volts: Some(20.0),
                negotiated_amps: Some(5.0),
                cable_current_limit_amps: Some(3),
            }),
        );
        let line = verdicts(&s)[0].charging_line.clone().unwrap();
        assert!(line.contains("3 A rating"), "got: {line}");
    }

    #[test]
    fn charging_line_reports_watts() {
        let s = snap(
            true,
            Transport::None,
            vec![],
            EmarkerInfo::default(),
            Some(Charger {
                watts: None,
                is_charging: true,
            fully_charged: false,
            battery_percent: Some(58),
            minutes_to_full: Some(129),
            live_watts: Some(24.0),
                profile_volts: vec![],
            pdos: vec![],
                negotiated_volts: Some(20.0),
                negotiated_amps: Some(4.5),
                cable_current_limit_amps: Some(5),
            }),
        );
        let line = verdicts(&s)[0].charging_line.clone().unwrap();
        assert!(line.contains("90 W"), "got: {line}");
    }

    #[test]
    fn trust_flag_on_zero_vid() {
        let s = snap(
            true,
            Transport::None,
            vec![],
            em(Some(0), Transport::None, Some(3)),
            None,
        );
        assert!(verdicts(&s)[0]
            .trust_flags
            .iter()
            .any(|f| f.contains("0x0000")));
    }

    #[test]
    fn cards_cover_data_and_cable_for_a_plain_usb_device() {
        let s = snap(
            true,
            Transport::Usb3Gen2,
            vec![dev("SSD", Transport::Usb3Gen2)],
            EmarkerInfo::default(),
            None,
        );
        let kinds: Vec<CardKind> = verdicts(&s)[0].cards.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, vec![CardKind::Port, CardKind::Data, CardKind::Cable]);
        assert_eq!(verdicts(&s)[0].cards[1].status, CardStatus::Ok);
    }

    #[test]
    fn dp_adapter_gets_display_card_no_data_card() {
        let s = snap_dp(
            true,
            Transport::Usb2,
            true,
            vec![dev("USB-C To DP Cable", Transport::Usb2)],
            EmarkerInfo::default(),
            None,
        );
        let kinds: Vec<CardKind> = verdicts(&s)[0].cards.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&CardKind::Display));
        assert!(!kinds.contains(&CardKind::Data));
    }

    #[test]
    fn empty_port_has_no_cards() {
        let s = snap(false, Transport::None, vec![], EmarkerInfo::default(), None);
        assert!(verdicts(&s)[0].cards.is_empty());
    }

    #[test]
    fn charger_adds_a_charging_card() {
        let s = snap(
            true,
            Transport::None,
            vec![],
            EmarkerInfo::default(),
            Some(Charger {
                watts: None,
                is_charging: true,
            fully_charged: false,
            battery_percent: Some(58),
            minutes_to_full: Some(129),
            live_watts: Some(24.0),
                profile_volts: vec![],
            pdos: vec![],
                negotiated_volts: Some(20.0),
                negotiated_amps: Some(4.5),
                cable_current_limit_amps: Some(5),
            }),
        );
        assert!(verdicts(&s)[0]
            .cards
            .iter()
            .any(|c| c.kind == CardKind::Charging && c.status == CardStatus::Ok));
    }
}
