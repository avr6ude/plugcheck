# plugcheck MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a macOS desktop app that shows, per USB-C / Thunderbolt port, what the connected cable and device can actually do and where the bottleneck is.

**Architecture:** Tauri 2 app. Rust backend owns a `UsbProbe` trait with one macOS implementation that shells out to `ioreg` and `system_profiler`, parses their output into a normalized `Snapshot`, and a pure `verdict` module that turns a `Snapshot` into per-port plain-language verdicts. A 3-second poll task diffs snapshots and emits a Tauri event. Svelte 5 frontend renders one card per port from the snapshot + verdicts.

**Tech Stack:** Rust, Tauri 2, `plist` crate, `serde`/`serde_json`, `tokio` (bundled by Tauri), Svelte 5 (runes) + TypeScript + Vite, `@tauri-apps/api` v2.

**Spec:** `docs/2026-09-02-plugcheck-mvp-design.md`

## Global Constraints

- Platform: Apple Silicon, macOS 14+. The live probe is `#[cfg(target_os = "macos")]`; on any other target `UsbProbe::snapshot` returns `ProbeError::Unsupported`.
- Tauri version pinned to `2` (exact minor pinned in `Cargo.toml` + `package.json` once scaffolded — do not run `npm update` / `cargo update` for Tauri).
- No Tauri plugins in MVP (no fs, notification, or shell plugin). Shelling out is done from Rust with `std::process::Command`, not the shell plugin.
- All types crossing the Rust→JS boundary derive `serde::Serialize`; snapshot types also derive `PartialEq + Clone` for diffing.
- Every external command call has a 5-second timeout and maps failure to `ProbeError::CommandFailed`.
- Per-field decode failures degrade that field to `null` + a `log::warn!`; they never abort a snapshot.
- Bit-layout tables (VDO, PDO, speed enums) live only in `emarker.rs`, each with a comment citing its USB PD R3.1 section.
- IOKit key names live only in a `const` block in `probe/macos.rs`, annotated with the macOS build + SoC they were observed on.
- Poll interval is a single `const POLL_INTERVAL: Duration` in `lib.rs`.
- License: MIT.

---

## File Structure

```
docs/
  2026-09-02-plugcheck-mvp-design.md   spec (exists)
  iokit-keys.md                        Task 2 output: observed IOKit keys
  superpowers/plans/2026-09-02-plugcheck-mvp.md   this plan
src-tauri/
  Cargo.toml                           deps, pinned tauri
  tauri.conf.json                      window config, no plugins
  build.rs                             tauri-build (from scaffold)
  src/
    main.rs                            scaffold entry -> lib::run()
    lib.rs                             tauri::Builder, commands, poll task
    model.rs                           Snapshot + all nested types + ProbeError
    probe/mod.rs                       UsbProbe trait, re-exports
    probe/macos.rs                     MacosProbe, parse_snapshot(), IOKit key consts
    emarker.rs                         VDO/PDO decode, speed enums, VID lookup
    verdict.rs                         verdicts(&Snapshot) -> Vec<PortVerdict>
  tests/
    fixtures/
      ioreg.plist                      real `ioreg -a -l` capture (Task 2)
      system_profiler.json             real `system_profiler` capture (Task 2)
      snap_tb4_good.json               hand-authored Snapshot fixtures (Task 7)
      snap_usb2_charge_only.json
      snap_emarker_mismatch.json
      snap_no_emarker.json
      snap_cable_charging_bottleneck.json
      snap_hub_nested.json
    parse_macos.rs                     integration test: fixtures -> Snapshot
    verdict.rs                         integration test: Snapshot fixtures -> verdicts
  assets/
    usbif_vendors.csv                  partial VID->name list (Task 4)
src/
  app.css
  main.ts                              scaffold
  App.svelte                           window shell, refresh, engineer toggle
  lib/
    snapshot.svelte.ts                 store: invoke + event listen
    PortCard.svelte
    DeviceTree.svelte
    EngineerPanel.svelte
package.json                           pinned @tauri-apps deps
README.md                             build + run instructions (Task 13)
LICENSE                               MIT (Task 13)
```

---

### Task 1: Scaffold Tauri 2 + Svelte 5 project

**Files:**
- Create: whole scaffold (`src-tauri/`, `src/`, `package.json`, `index.html`, `vite.config.ts`, `.gitignore`)
- Modify: `src-tauri/Cargo.toml` (pin versions), `src-tauri/tauri.conf.json` (window title/size)

**Interfaces:**
- Consumes: nothing
- Produces: a buildable Tauri app; `src-tauri/src/lib.rs` with a `run()` fn and a default `greet` command (removed in Task 8); npm scripts `dev`, `build`, `tauri`.

- [ ] **Step 1: Generate the scaffold**

Run from the repo root (`~/Projects/plugcheck`, which already contains `docs/` and a git repo):

```bash
npm create tauri-app@latest . -- --template svelte-ts --manager npm --yes
```

If the tool refuses to run in a non-empty directory, generate into a temp dir and move files in:

```bash
cd .. && npm create tauri-app@latest plugcheck-scaffold -- --template svelte-ts --manager npm --yes
rsync -a --exclude .git plugcheck-scaffold/ plugcheck/ && rm -rf plugcheck-scaffold && cd plugcheck
```

- [ ] **Step 2: Install and pin dependencies**

```bash
npm install
```

Then edit `package.json` — change the `@tauri-apps/api` and `@tauri-apps/cli` version ranges from `^2.x.x` to the exact resolved version (read it from `package-lock.json`). Edit `src-tauri/Cargo.toml` — change `tauri = { version = "2", ... }` to the exact version from `Cargo.lock` (e.g. `"2.9.0"`), same for `tauri-build`.

- [ ] **Step 3: Add backend dependencies**

In `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
plist = "1"
serde_json = "1"
log = "0.4"
env_logger = "0.11"
```

`serde` with `derive` is already present from the scaffold. Run `cargo build --manifest-path src-tauri/Cargo.toml` to fetch.

- [ ] **Step 4: Configure the window**

In `src-tauri/tauri.conf.json`, set the main window:

```json
{
  "title": "plugcheck",
  "width": 520,
  "height": 720,
  "resizable": true
}
```

- [ ] **Step 5: Verify it builds and runs**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

Expected: both succeed. `cargo test` reports `0 passed`. (Do not run `cargo tauri dev` in CI/agent context — it needs a display. A human runs it once to eyeball the window.)

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2 + Svelte 5 project"
```

---

### Task 2: Capture real IOKit + system_profiler data, pin the keys

This task resolves spec open-risk #1 before any parser is written. Its deliverable is data + a key map that later tasks quote.

**Files:**
- Create: `src-tauri/tests/fixtures/ioreg.plist`, `src-tauri/tests/fixtures/system_profiler.json`, `docs/iokit-keys.md`

**Interfaces:**
- Consumes: nothing
- Produces: `docs/iokit-keys.md` with a table of the exact IOKit property names and value shapes observed on this machine, keyed by concept (port state, active transports, plug orientation, e-marker presence, Discover Identity VDO for SOP'/SOP''/port, negotiated PD voltage + current, device→port link). Later tasks reference these names.

- [ ] **Step 1: Capture the raw data**

With at least one USB-C cable plugged in (ideally a Thunderbolt cable + a charger), run:

```bash
ioreg -a -l > src-tauri/tests/fixtures/ioreg.plist
system_profiler -json SPUSBDataType SPThunderboltDataType > src-tauri/tests/fixtures/system_profiler.json
```

- [ ] **Step 2: Locate the port-controller nodes**

```bash
ioreg -a -r -l -c AppleHPMInterface | head -c 20000
ioreg -a -r -l -c AppleTCController | head -c 20000
```

Also try, if the above are empty:

```bash
ioreg -a -r -l -n AppleHPMInterfaceType10
ioreg -a -r -l -n AppleTCControllerType18
```

Note which class names actually exist on this machine.

- [ ] **Step 3: Find the PD / e-marker properties**

```bash
ioreg -a -l | grep -i -E "PDSOP|DiscoverIdentity|IDHeader|CableVDO|PowerSource|PDO|Emarker|Orientation|ActiveTransport" | sort -u
```

Record the property key names and whether values are numbers, data blobs, arrays, or dicts.

- [ ] **Step 4: Write `docs/iokit-keys.md`**

Fill this table with real observed values (example rows shown — replace with what you found):

```markdown
# Observed IOKit keys — macOS <build> / <SoC>, captured 2026-09-02

| Concept                  | IOKit class            | Property key           | Value shape          |
|--------------------------|------------------------|------------------------|----------------------|
| Port present / state     | AppleHPMInterfaceType10| ConnectionState        | int enum             |
| Active transports        | AppleHPMInterfaceType10| ActiveTransports       | int bitfield         |
| Plug orientation         | AppleTCControllerType18| Orientation            | int (0/1)            |
| E-marker present         | AppleTCControllerType18| CableEMarkerPresent    | bool                 |
| Discover Identity (SOP') | ...PDSOPp...            | IDHeaderVDO / CableVDO | data (4 bytes LE)    |
| Negotiated PD            | IOPortFeaturePowerSource| NegotiatedVoltage/... | int mV / mA          |
| Device -> port link      | (XHCI subtree)         | UsbIOPort / locationID | string / int         |

## Notes
- Classes NOT found on this machine: <list>
- Cables with no SOP'' (far-end e-marker): expected, treat as normal.
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/fixtures/ioreg.plist src-tauri/tests/fixtures/system_profiler.json docs/iokit-keys.md
git commit -m "test: capture real ioreg + system_profiler fixtures, pin IOKit keys"
```

---

### Task 3: Normalized data model + probe trait

**Files:**
- Create: `src-tauri/src/model.rs`, `src-tauri/src/probe/mod.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod model; mod probe;`)
- Test: inline `#[cfg(test)]` in `model.rs`

**Interfaces:**
- Consumes: nothing
- Produces:

```rust
// model.rs
pub enum Transport { Usb2, Usb3Gen1, Usb3Gen2, Usb4Gen2, Usb4Gen3, Usb4Gen4, Thunderbolt3, Thunderbolt4, DisplayPort, None }
pub enum CableType { Passive, Active, Captive, Unknown }
pub struct EmarkerInfo {
    pub vendor_id: Option<u16>,
    pub vendor_name: Option<String>,
    pub cable_type: CableType,
    pub max_speed: Transport,        // e-marker's claimed highest speed
    pub current_amps: Option<u8>,    // 3 or 5
    pub max_power_watts: Option<u16>,// 60/100/240
    pub present: bool,               // false => no e-marker on this cable
}
pub struct DeviceNode {
    pub name: String,
    pub vendor: Option<String>,
    pub speed: Transport,
    pub is_hub: bool,
    pub children: Vec<DeviceNode>,
}
pub struct Charger {
    pub negotiated_volts: Option<f32>,
    pub negotiated_amps: Option<f32>,
    pub cable_current_limit_amps: Option<u8>, // from e-marker, for blame
}
pub struct Port {
    pub id: String,                 // stable per physical port, e.g. "HPM10-0"
    pub occupied: bool,
    pub orientation: Option<u8>,
    pub active_transport: Transport, // what's actually negotiated on the link
    pub emarker: EmarkerInfo,
    pub charger: Option<Charger>,
    pub devices: Vec<DeviceNode>,
    pub raw: std::collections::BTreeMap<String, String>, // engineer-mode dump
}
pub struct Snapshot { pub ports: Vec<Port>, pub captured_ms: u64 }

pub enum ProbeError { CommandFailed(String), ParseFailed(String), Unsupported }
```

All structs/enums: `#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]`. Enums serialize with `#[serde(rename_all = "snake_case")]`. `ProbeError` also `impl std::fmt::Display + std::error::Error`.

```rust
// probe/mod.rs
use crate::model::{Snapshot, ProbeError};
pub trait UsbProbe: Send + Sync {
    fn snapshot(&self) -> Result<Snapshot, ProbeError>;
}
```

- [ ] **Step 1: Write the failing test**

In `src-tauri/src/model.rs`, at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_json_roundtrips_and_eq() {
        let snap = Snapshot {
            captured_ms: 42,
            ports: vec![Port {
                id: "HPM10-0".into(),
                occupied: true,
                orientation: Some(1),
                active_transport: Transport::Usb2,
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
                raw: Default::default(),
            }],
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
        assert!(json.contains("\"cable_type\":\"passive\""));
    }

    #[test]
    fn probe_error_displays() {
        let e = ProbeError::CommandFailed("ioreg timed out".into());
        assert_eq!(e.to_string(), "command failed: ioreg timed out");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml model::`
Expected: FAIL — `model` module / types not found.

- [ ] **Step 3: Write `model.rs` and `probe/mod.rs`**

Implement exactly the types in the Interfaces block above. `Display` for `ProbeError`:

```rust
impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeError::CommandFailed(s) => write!(f, "command failed: {s}"),
            ProbeError::ParseFailed(s) => write!(f, "parse failed: {s}"),
            ProbeError::Unsupported => write!(f, "unsupported platform (needs Apple Silicon, macOS 14+)"),
        }
    }
}
impl std::error::Error for ProbeError {}
```

Add `mod model;` and `mod probe;` to `lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml model::`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/model.rs src-tauri/src/probe/mod.rs src-tauri/src/lib.rs
git commit -m "feat: normalized Snapshot model + UsbProbe trait"
```

---

### Task 4: e-marker VDO / PDO decode + VID lookup

**Files:**
- Create: `src-tauri/src/emarker.rs`, `src-tauri/assets/usbif_vendors.csv`
- Modify: `src-tauri/src/lib.rs` (`mod emarker;`)
- Test: inline `#[cfg(test)]` in `emarker.rs`

**Interfaces:**
- Consumes: `crate::model::{Transport, CableType}`
- Produces:

```rust
pub struct IdHeader { pub vendor_id: u16, pub cable_type: CableType }
pub fn decode_id_header_vdo(vdo: u32) -> IdHeader;

pub struct CableCaps {
    pub max_speed: Transport,
    pub current_amps: Option<u8>,   // 3 or 5
    pub max_power_watts: Option<u16>,
}
pub fn decode_cable_vdo(vdo: u32, active: bool) -> CableCaps;

pub struct PdoVolts { pub volts: f32, pub amps: f32 }
pub fn decode_fixed_pdo(pdo: u32) -> Option<PdoVolts>; // None if not a Fixed Supply PDO

pub fn vendor_name(vid: u16) -> Option<String>; // from bundled CSV, include_str!
pub fn transport_label(t: Transport) -> &'static str;
```

Bit layouts — **verify against USB PD R3.1 while implementing**:

- **ID Header VDO** (§6.4.4.3.1.1): bits `[15:0]` = USB Vendor ID. Bits `[28:27]` = Product Type (Cable Plug) for SOP': `0b011` = Passive Cable → `CableType::Passive`, `0b100` = Active Cable → `CableType::Active`, else `Unknown`.
- **Passive/Active Cable VDO** (§6.4.4.3.1.6 / .7): bits `[2:0]` = USB Highest Speed: `0b000`=`Usb2`, `0b001`=`Usb3Gen1`, `0b010`=`Usb3Gen2`, `0b011`=`Usb4Gen3`, other→`Usb4Gen4` if `active` else `Usb3Gen2` (be conservative; comment the assumption). Bits `[6:5]` = VBUS Current Handling: `0b01`→`Some(3)`, `0b10`→`Some(5)`, else `None`. `max_power_watts` = `current_amps.map(|a| if a == 5 { 240 } else { 60 })` (comment: MVP approximation; full table is post-MVP).
- **Fixed Supply PDO** (§6.4.1.2.1): bits `[31:30]` must be `0b00`, else return `None`. Bits `[19:10]` = voltage in 50 mV units → `volts = raw * 0.05`. Bits `[9:0]` = max current in 10 mA units → `amps = raw * 0.01`.

`usbif_vendors.csv` — MVP partial list, header `vid_hex,name`:

```csv
vid_hex,name
0x05ac,Apple
0x0bda,Realtek
0x2109,VIA Labs
0x8087,Intel
0x0451,Texas Instruments
0x18d1,Google
```

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Transport, CableType};

    #[test]
    fn id_header_extracts_vid_and_passive_type() {
        // VID = 0x05AC, Product Type (Cable Plug) = 0b011 (passive) at bits [28:27]
        let vdo: u32 = (0b011 << 27) | 0x05AC;
        let h = decode_id_header_vdo(vdo);
        assert_eq!(h.vendor_id, 0x05AC);
        assert_eq!(h.cable_type, CableType::Passive);
    }

    #[test]
    fn cable_vdo_speed_and_current() {
        // Highest speed = 0b011 (USB4 Gen3), current = 0b10 (5A)
        let vdo: u32 = (0b10 << 5) | 0b011;
        let c = decode_cable_vdo(vdo, false);
        assert_eq!(c.max_speed, Transport::Usb4Gen3);
        assert_eq!(c.current_amps, Some(5));
        assert_eq!(c.max_power_watts, Some(240));
    }

    #[test]
    fn fixed_pdo_20v_5a() {
        // voltage 20V => 400 * 50mV ; current 5A => 500 * 10mA
        let pdo: u32 = (400 << 10) | 500;
        let p = decode_fixed_pdo(pdo).unwrap();
        assert!((p.volts - 20.0).abs() < 0.001);
        assert!((p.amps - 5.0).abs() < 0.001);
    }

    #[test]
    fn non_fixed_pdo_rejected() {
        let pdo: u32 = 0b01 << 30; // Battery Supply PDO
        assert!(decode_fixed_pdo(pdo).is_none());
    }

    #[test]
    fn vendor_lookup() {
        assert_eq!(vendor_name(0x05ac).as_deref(), Some("Apple"));
        assert_eq!(vendor_name(0xffff), None);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml emarker::`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement `emarker.rs`**

Write the decode functions per the bit layouts above. Each `fn` gets a doc comment citing its USB PD R3.1 section. `vendor_name` parses `include_str!("../assets/usbif_vendors.csv")` once (a plain linear scan is fine for ~6–50 rows; `parse` the `0x` hex with `u16::from_str_radix(&s[2..], 16)`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml emarker::`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/emarker.rs src-tauri/assets/usbif_vendors.csv src-tauri/src/lib.rs
git commit -m "feat: e-marker VDO/PDO decode + USB-IF vendor lookup"
```

---

### Task 5: macОS probe — parse ports + e-marker from fixtures

**Files:**
- Create: `src-tauri/src/probe/macos.rs`, `src-tauri/tests/parse_macos.rs`
- Modify: `src-tauri/src/probe/mod.rs` (`#[cfg(target_os = "macos")] pub mod macos;`)

**Interfaces:**
- Consumes: `crate::model::*`, `crate::emarker::*`, IOKit key names from `docs/iokit-keys.md` (Task 2)
- Produces:

```rust
pub struct MacosProbe;
impl MacosProbe {
    /// Pure: takes the two command outputs, returns a Snapshot. Testable off-device.
    pub fn parse_snapshot(ioreg_plist: &str, sp_json: &str) -> Result<Snapshot, ProbeError>;
}
impl crate::probe::UsbProbe for MacosProbe {
    fn snapshot(&self) -> Result<Snapshot, ProbeError>; // shells out, then parse_snapshot
}

// const block, annotated with observed macOS build + SoC:
mod keys {
    pub const HPM_CLASS: &str = "...";      // fill from docs/iokit-keys.md
    pub const K_CONN_STATE: &str = "...";
    pub const K_ACTIVE_TRANSPORTS: &str = "...";
    pub const K_ORIENTATION: &str = "...";
    pub const K_ID_HEADER_VDO: &str = "...";
    pub const K_CABLE_VDO: &str = "...";
    // ...
}
```

- [ ] **Step 1: Write the failing integration test**

`src-tauri/tests/parse_macos.rs`:

```rust
use std::fs;

#[test]
fn parses_real_fixture_into_ports() {
    let ioreg = fs::read_to_string("tests/fixtures/ioreg.plist").unwrap();
    let sp = fs::read_to_string("tests/fixtures/system_profiler.json").unwrap();
    let snap = plugcheck_lib::probe::macos::MacosProbe::parse_snapshot(&ioreg, &sp).unwrap();

    // The capture in Task 2 had at least one cable connected.
    assert!(!snap.ports.is_empty(), "expected at least one port");
    let occupied = snap.ports.iter().filter(|p| p.occupied).count();
    assert!(occupied >= 1, "expected at least one occupied port");

    // Every occupied port has a decoded active_transport and a populated raw map.
    for p in snap.ports.iter().filter(|p| p.occupied) {
        assert!(!p.raw.is_empty(), "engineer-mode raw dump must be populated");
    }
}

#[test]
fn missing_key_degrades_to_none_not_error() {
    // Feed valid plist header but strip the e-marker keys.
    let sp = fs::read_to_string("tests/fixtures/system_profiler.json").unwrap();
    let minimal_plist = r#"<?xml version="1.0"?><!DOCTYPE plist><plist version="1.0"><dict></dict></plist>"#;
    let snap = plugcheck_lib::probe::macos::MacosProbe::parse_snapshot(minimal_plist, &sp).unwrap();
    // No ports from an empty ioreg is fine; must not panic or Err.
    let _ = snap.ports.len();
}
```

Note: the scaffold's lib crate is named `plugcheck_lib` (Tauri default is `<name>_lib`). Confirm the exact name in `src-tauri/Cargo.toml` `[lib] name` and adjust the `use`. Make `pub mod probe;` / `pub mod model;` / `pub mod emarker;` public in `lib.rs` so integration tests can reach them.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test parse_macos`
Expected: FAIL — `parse_snapshot` not found.

- [ ] **Step 3: Implement `probe/macos.rs`**

1. Fill the `keys` const block from `docs/iokit-keys.md`.
2. `parse_snapshot`:
   - Parse `ioreg_plist` with `plist::Value::from_reader_xml`. Recursively walk the node tree (`plist::Value::as_dictionary`, `"IORegistryEntryChildren"` array — confirm the child key name from the fixture).
   - For each node whose `"IOObjectClass"` matches `keys::HPM_CLASS` (or the TC controller class), build a `Port`: `id` from the node's `"locationID"` or registry path; `occupied` from `K_CONN_STATE`; `orientation` from `K_ORIENTATION`; `active_transport` by mapping `K_ACTIVE_TRANSPORTS` bits to `Transport`.
   - E-marker: read `K_ID_HEADER_VDO` + `K_CABLE_VDO` (plist `Data`, 4 bytes little-endian → `u32::from_le_bytes`). Feed to `emarker::decode_id_header_vdo` / `decode_cable_vdo`. If keys absent → `EmarkerInfo { present: false, ..Default }`.
   - Charger: from `IOPortFeaturePowerSource` node — negotiated V/I keys per `docs/iokit-keys.md`.
   - `raw`: dump every scalar property of the port node into the `BTreeMap<String,String>` (`format!("{:?}", value)`).
   - Wrap every field decode in a helper that logs + returns `None` on failure.
   - Parse `sp_json` with `serde_json` for the device tree (Task 6 fills `devices`; here just leave `devices: vec![]`).
   - `ProbeError::ParseFailed` only if the plist itself won't parse at all.
3. `snapshot()`:

```rust
fn snapshot(&self) -> Result<Snapshot, ProbeError> {
    #[cfg(not(target_os = "macos"))]
    { return Err(ProbeError::Unsupported); }
    #[cfg(target_os = "macos")]
    {
        let ioreg = run_with_timeout("ioreg", &["-a", "-l"], 5)?;
        let sp = run_with_timeout("system_profiler", &["-json", "SPUSBDataType", "SPThunderboltDataType"], 5)?;
        Self::parse_snapshot(&ioreg, &sp)
    }
}
```

`run_with_timeout` spawns via `std::process::Command`, waits on a thread with `recv_timeout`, maps non-zero / timeout to `ProbeError::CommandFailed`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test parse_macos`
Expected: PASS (2 tests). If `parses_real_fixture_into_ports` fails on `ports.is_empty()`, the `keys` const block doesn't match the fixture — go back to `docs/iokit-keys.md` and the fixture, fix the class/key names.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/probe/macos.rs src-tauri/src/probe/mod.rs src-tauri/src/lib.rs src-tauri/tests/parse_macos.rs
git commit -m "feat: macOS probe parses ports + e-marker from ioreg/system_profiler"
```

---

### Task 6: macOS probe — device hierarchy + port linkage

**Files:**
- Modify: `src-tauri/src/probe/macos.rs`, `src-tauri/tests/parse_macos.rs`

**Interfaces:**
- Consumes: `parse_snapshot` from Task 5
- Produces: `Port.devices: Vec<DeviceNode>` populated; a `fn build_device_tree(sp_json: &serde_json::Value) -> Vec<(String /*location_id*/, DeviceNode)>` internal helper.

- [ ] **Step 1: Add the failing test**

Append to `src-tauri/tests/parse_macos.rs`:

```rust
#[test]
fn nests_devices_under_their_port() {
    let ioreg = std::fs::read_to_string("tests/fixtures/ioreg.plist").unwrap();
    let sp = std::fs::read_to_string("tests/fixtures/system_profiler.json").unwrap();
    let snap = plugcheck_lib::probe::macos::MacosProbe::parse_snapshot(&ioreg, &sp).unwrap();

    let total_devices: usize = snap.ports.iter()
        .flat_map(|p| p.devices.iter())
        .map(count_tree)
        .sum();
    // The Task 2 capture had a real device (hub/dock/keyboard) on at least one port.
    assert!(total_devices >= 1, "expected devices attached under a port");
}

fn count_tree(d: &plugcheck_lib::model::DeviceNode) -> usize {
    1 + d.children.iter().map(count_tree).sum::<usize>()
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test parse_macos nests_devices`
Expected: FAIL — `total_devices` is 0 (Task 5 left `devices` empty).

- [ ] **Step 3: Implement `build_device_tree`**

In `system_profiler` `SPUSBDataType` JSON, each entry has `_name`, `location_id` (e.g. `"0x02100000 / 3"`), `speed` (e.g. `"up_to_10_Gb_s"`), optional `_items` (children), and hub entries have `_name` containing `"Hub"` or a `bDeviceClass` of 9. Recursively map each node to `DeviceNode`:
- `speed` string → `Transport` via a match in `emarker::` (add `pub fn speed_str_to_transport(&str) -> Transport` there, table-commented): `"up_to_480_Mb_s"`→`Usb2`, `"up_to_5_Gb_s"`→`Usb3Gen1`, `"up_to_10_Gb_s"`→`Usb3Gen2`, `"up_to_20_Gb_s"`→`Usb4Gen3`, `"up_to_40_Gb_s"`→`Usb4Gen4`, else `None`.
- Link to a `Port` by matching the high bytes of `location_id` against the port's `id`/`locationID` (document the masking rule you derive from the fixture — typically the top 8 bits identify the controller port).
- Thunderbolt devices from `SPThunderboltDataType`: attach by port likewise; set their `speed` from the TB `_name` / `receptacle` gen fields to `Thunderbolt3`/`Thunderbolt4`.
- Unmatched devices → attach to the first occupied port and `log::warn!` (never drop them).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test parse_macos`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/probe/macos.rs src-tauri/src/emarker.rs src-tauri/tests/parse_macos.rs
git commit -m "feat: nest USB/TB device tree under physical ports"
```

---

### Task 7: Verdict engine — headline + data-speed verdict + blame

**Files:**
- Create: `src-tauri/src/verdict.rs`, `src-tauri/tests/verdict.rs`, fixtures `snap_tb4_good.json`, `snap_usb2_charge_only.json`, `snap_emarker_mismatch.json`, `snap_no_emarker.json`
- Modify: `src-tauri/src/lib.rs` (`pub mod verdict;`)

**Interfaces:**
- Consumes: `crate::model::{Snapshot, Port, Transport, CableType}`
- Produces:

```rust
pub enum Blame { Port, Cable, Device, None }
pub struct PortVerdict {
    pub port_id: String,
    pub headline: String,        // "Thunderbolt 4" | "USB device" | "Display" | "Charging only" | "Empty"
    pub data_line: String,       // one sentence
    pub data_blame: Blame,
    pub charging_line: Option<String>, // filled in Task 8
    pub trust_flags: Vec<String>,      // filled in Task 8
}
pub fn verdicts(snap: &Snapshot) -> Vec<PortVerdict>;
```

Derive on `Blame`/`PortVerdict`: `Debug, Clone, PartialEq, Serialize, Deserialize`, `Blame` as `snake_case`.

Verdict rules (from spec §"MVP feature set" 2 & 4):
- **headline:** empty port → `"Empty"`. Else if any device speed is TB or `active_transport` is TB → `"Thunderbolt 4"` if `Usb4`/`Tb4` else `"Thunderbolt 3"`. Else if a display device present → `"Display"`. Else if devices non-empty → `"USB device"`. Else (occupied, charger present, no data device) → `"Charging only"`.
- **data_line + data_blame:** compare `active_transport` (what negotiated) to the best of `emarker.max_speed` and the fastest device's declared speed.
  - `active_transport == None` and no devices → `"No data device connected."`, `Blame::None`.
  - `active_transport >= min(emarker.max_speed, fastest_device_speed)` → `"Running at full speed (<label>)."`, `Blame::None`.
  - `emarker.present && emarker.max_speed < fastest_device_speed && active_transport == emarker.max_speed` → `"Cable caps this link at <emarker label>; the device supports <device label>."`, `Blame::Cable`.
  - `!emarker.present && active_transport == Usb2 && fastest_device_speed > Usb2` → `"Link fell back to USB 2.0 — likely a charge-only cable."`, `Blame::Cable`.
  - `active_transport < fastest_device_speed && emarker ok` → `"Port is negotiating below the cable and device capability."`, `Blame::Port`.
  - device slower than everything → `"Limited by the device (<device label>)."`, `Blame::Device`.
- Add an `Ord` for `Transport` (define an explicit `fn rank(Transport) -> u8`; do NOT `#[derive(PartialOrd)]` on the enum — order must be by speed, comment the ranking).

- [ ] **Step 1: Author the fixtures**

Write four `Snapshot` JSON files under `src-tauri/tests/fixtures/`. Each is a hand-built single-port snapshot. Example `snap_emarker_mismatch.json`:

```json
{
  "captured_ms": 0,
  "ports": [{
    "id": "HPM10-0",
    "occupied": true,
    "orientation": 1,
    "active_transport": "usb2",
    "emarker": {
      "vendor_id": 1452, "vendor_name": "Apple", "cable_type": "passive",
      "max_speed": "usb2", "current_amps": 5, "max_power_watts": 240, "present": true
    },
    "charger": null,
    "devices": [{ "name": "SanDisk SSD", "vendor": "SanDisk", "speed": "usb3_gen2", "is_hub": false, "children": [] }],
    "raw": {}
  }]
}
```

`snap_tb4_good.json`: `active_transport":"usb4_gen3"`, emarker `max_speed":"usb4_gen3"`, one device at `usb4_gen3`.
`snap_usb2_charge_only.json`: `active_transport":"usb2"`, `emarker.present":false`, `devices":[]`, `charger` non-null.
`snap_no_emarker.json`: `active_transport":"usb2"`, `emarker.present":false`, one device at `usb3_gen2`.

- [ ] **Step 2: Write the failing test**

`src-tauri/tests/verdict.rs`:

```rust
use plugcheck_lib::{model::Snapshot, verdict::{verdicts, Blame}};
use std::fs;

fn load(name: &str) -> Snapshot {
    serde_json::from_str(&fs::read_to_string(format!("tests/fixtures/{name}")).unwrap()).unwrap()
}

#[test]
fn tb4_good_is_full_speed_no_blame() {
    let v = &verdicts(&load("snap_tb4_good.json"))[0];
    assert_eq!(v.headline, "Thunderbolt 4");
    assert_eq!(v.data_blame, Blame::None);
    assert!(v.data_line.contains("full speed"));
}

#[test]
fn emarker_mismatch_blames_cable() {
    let v = &verdicts(&load("snap_emarker_mismatch.json"))[0];
    assert_eq!(v.data_blame, Blame::Cable);
    assert!(v.data_line.contains("Cable caps"));
}

#[test]
fn charge_only_is_charging_headline() {
    let v = &verdicts(&load("snap_usb2_charge_only.json"))[0];
    assert_eq!(v.headline, "Charging only");
}

#[test]
fn no_emarker_usb2_fallback_blames_cable() {
    let v = &verdicts(&load("snap_no_emarker.json"))[0];
    assert_eq!(v.data_blame, Blame::Cable);
    assert!(v.data_line.contains("charge-only cable"));
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test verdict`
Expected: FAIL — `verdict` module not found.

- [ ] **Step 4: Implement `verdict.rs`**

Implement `verdicts` per the rules above. `charging_line: None` and `trust_flags: vec![]` for now.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test verdict`
Expected: PASS (4 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/verdict.rs src-tauri/tests/verdict.rs src-tauri/tests/fixtures/snap_*.json src-tauri/src/lib.rs
git commit -m "feat: verdict engine — headline + data-speed verdict + blame"
```

---

### Task 8: Verdict engine — charging line + trust flags

**Files:**
- Modify: `src-tauri/src/verdict.rs`, `src-tauri/tests/verdict.rs`
- Create: fixtures `snap_cable_charging_bottleneck.json`, `snap_hub_nested.json`

**Interfaces:**
- Consumes: Task 7 `verdicts`
- Produces: `PortVerdict.charging_line` and `.trust_flags` populated.

Rules:
- **charging_line** (spec §MVP 5): `charger` is `None` → stays `None`. Else `w = negotiated_volts * negotiated_amps`:
  - `emarker.current_amps == Some(3) && negotiated_amps > 3.0`-capable-charger → `Some("Cable limits charging to ~60 W (3 A cable).")`
  - else → `Some(format!("Charging at {w:.0} W ({v:.0} V / {a:.1} A).", v=volts, a=amps))`
  - missing V or I → `Some("Charging (wattage unavailable).")`
- **trust_flags** (spec §MVP 9): push `"E-marker vendor ID is 0x0000 (not registered with USB-IF)."` when `emarker.present && emarker.vendor_id == Some(0)`. Also push `"Cable claims 5 A but the link is only USB 2.0."` when `emarker.current_amps == Some(5) && emarker.max_speed == Transport::Usb2`.

- [ ] **Step 1: Author fixtures**

`snap_cable_charging_bottleneck.json`: occupied port, `emarker.current_amps":3`, `charger":{"negotiated_volts":20.0,"negotiated_amps":5.0,"cable_current_limit_amps":3}`, `devices":[]`.
`snap_hub_nested.json`: one port, `devices` = a hub `is_hub":true` with two `children`. (Used by a headline/tree regression check.)

- [ ] **Step 2: Add failing tests**

Append to `src-tauri/tests/verdict.rs`:

```rust
#[test]
fn charging_bottleneck_names_the_3a_cable() {
    let v = &verdicts(&load("snap_cable_charging_bottleneck.json"))[0];
    let line = v.charging_line.as_ref().unwrap();
    assert!(line.contains("3 A cable"), "got: {line}");
}

#[test]
fn trust_flag_on_5a_usb2_mismatch() {
    let v = &verdicts(&load("snap_emarker_mismatch.json"))[0];
    assert!(v.trust_flags.iter().any(|f| f.contains("5 A") && f.contains("USB 2.0")));
}

#[test]
fn hub_children_counted_in_tree() {
    let v = &verdicts(&load("snap_hub_nested.json"))[0];
    assert_eq!(v.headline, "USB device");
}
```

- [ ] **Step 3: Run to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test verdict`
Expected: FAIL on the 3 new tests (`charging_line` is `None`, `trust_flags` empty).

- [ ] **Step 4: Implement**

Add the charging + trust logic to `verdicts`.

- [ ] **Step 5: Run to verify all pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test verdict`
Expected: PASS (7 tests).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/verdict.rs src-tauri/tests/verdict.rs src-tauri/tests/fixtures/snap_*.json
git commit -m "feat: charging-bottleneck line + trust flags"
```

---

### Task 9: Tauri commands + poll task

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: inline `#[cfg(test)]` in `lib.rs`

**Interfaces:**
- Consumes: `MacosProbe`, `verdict::verdicts`, `model::Snapshot`
- Produces: Tauri commands `get_snapshot() -> Result<Snapshot, String>`, `get_verdicts() -> Result<Vec<PortVerdict>, String>`, `engineer_dump(port_id: String) -> Result<BTreeMap<String,String>, String>`; event `"snapshot-changed"` (payload `Snapshot`); `const POLL_INTERVAL: Duration = Duration::from_secs(3)`; pure helper `fn changed(old: &Option<Snapshot>, new: &Snapshot) -> bool`.

- [ ] **Step 1: Write the failing test**

In `src-tauri/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Snapshot;

    #[test]
    fn changed_is_false_for_equal_snapshots() {
        let s = Snapshot { ports: vec![], captured_ms: 1 };
        let s2 = Snapshot { ports: vec![], captured_ms: 999 }; // captured_ms ignored
        assert!(!changed(&Some(s), &s2));
    }

    #[test]
    fn changed_is_true_when_none() {
        let s = Snapshot { ports: vec![], captured_ms: 0 };
        assert!(changed(&None, &s));
    }
}
```

`changed` compares everything **except** `captured_ms` — implement by cloning and zeroing `captured_ms`, or compare `ports` only.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --lib changed`
Expected: FAIL — `changed` not defined.

- [ ] **Step 3: Implement commands + poll**

```rust
use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, Manager, State};
use crate::model::Snapshot;
use crate::probe::UsbProbe;
#[cfg(target_os = "macos")]
use crate::probe::macos::MacosProbe;

const POLL_INTERVAL: Duration = Duration::from_secs(3);

struct AppState { last: Mutex<Option<Snapshot>>, probe: Box<dyn UsbProbe> }

fn changed(old: &Option<Snapshot>, new: &Snapshot) -> bool {
    match old { None => true, Some(o) => o.ports != new.ports }
}

#[tauri::command]
fn get_snapshot(state: State<AppState>) -> Result<Snapshot, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    *state.last.lock().unwrap() = Some(snap.clone());
    Ok(snap)
}

#[tauri::command]
fn get_verdicts(state: State<AppState>) -> Result<Vec<crate::verdict::PortVerdict>, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    Ok(crate::verdict::verdicts(&snap))
}

#[tauri::command]
fn engineer_dump(port_id: String, state: State<AppState>) -> Result<std::collections::BTreeMap<String,String>, String> {
    let snap = state.probe.snapshot().map_err(|e| e.to_string())?;
    snap.ports.into_iter().find(|p| p.id == port_id)
        .map(|p| p.raw).ok_or_else(|| format!("no port {port_id}"))
}

pub fn run() {
    let _ = env_logger::try_init();
    #[cfg(target_os = "macos")]
    let probe: Box<dyn UsbProbe> = Box::new(MacosProbe);
    #[cfg(not(target_os = "macos"))]
    let probe: Box<dyn UsbProbe> = Box::new(UnsupportedProbe);

    tauri::Builder::default()
        .manage(AppState { last: Mutex::new(None), probe })
        .invoke_handler(tauri::generate_handler![get_snapshot, get_verdicts, engineer_dump])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(POLL_INTERVAL).await;
                    let state = handle.state::<AppState>();
                    if let Ok(snap) = state.probe.snapshot() {
                        let mut last = state.last.lock().unwrap();
                        if changed(&last, &snap) {
                            *last = Some(snap.clone());
                            drop(last);
                            let _ = handle.emit("snapshot-changed", snap);
                        }
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Add a `#[cfg(not(target_os = "macos"))] struct UnsupportedProbe;` impl returning `Err(ProbeError::Unsupported)`. Delete the scaffold's `greet` command and its frontend call.

- [ ] **Step 4: Run to verify tests pass + it builds**

Run:
```bash
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo build --manifest-path src-tauri/Cargo.toml
```
Expected: PASS + clean build.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: tauri commands + 3s poll emitting snapshot-changed"
```

---

### Task 10: Frontend store

**Files:**
- Create: `src/lib/snapshot.svelte.ts`
- Modify: `src/App.svelte` (remove scaffold demo content)

**Interfaces:**
- Consumes: Tauri commands `get_snapshot`, `get_verdicts`; event `"snapshot-changed"`
- Produces:

```ts
// snapshot.svelte.ts — Svelte 5 runes module
export const store: {
  snapshot: Snapshot | null;
  verdicts: PortVerdict[];
  error: string | null;
  loading: boolean;
};
export function startPolling(): Promise<() => void>; // returns unlisten
export function refresh(): Promise<void>;
export type Snapshot = { ports: Port[]; captured_ms: number };
export type Port = { id: string; occupied: boolean; orientation: number | null;
  active_transport: string; emarker: Emarker; charger: Charger | null;
  devices: DeviceNode[]; raw: Record<string,string> };
export type Emarker = { vendor_id: number | null; vendor_name: string | null;
  cable_type: string; max_speed: string; current_amps: number | null;
  max_power_watts: number | null; present: boolean };
export type Charger = { negotiated_volts: number | null; negotiated_amps: number | null;
  cable_current_limit_amps: number | null };
export type DeviceNode = { name: string; vendor: string | null; speed: string;
  is_hub: boolean; children: DeviceNode[] };
export type Blame = "port" | "cable" | "device" | "none";
export type PortVerdict = { port_id: string; headline: string; data_line: string;
  data_blame: Blame; charging_line: string | null; trust_flags: string[] };
```

- [ ] **Step 1: Implement the store**

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const store = $state({ snapshot: null, verdicts: [], error: null, loading: false });

export async function refresh() {
  store.loading = true;
  try {
    store.snapshot = await invoke("get_snapshot");
    store.verdicts = await invoke("get_verdicts");
    store.error = null;
  } catch (e) {
    store.error = String(e);
  } finally {
    store.loading = false;
  }
}

export async function startPolling() {
  await refresh();
  const unlisten = await listen("snapshot-changed", (ev) => {
    store.snapshot = ev.payload as any;
    invoke("get_verdicts").then((v) => (store.verdicts = v as any)).catch((e) => (store.error = String(e)));
  });
  return unlisten;
}
```

- [ ] **Step 2: Type-check**

Run: `npm run check` (svelte-check, present in the scaffold).
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/snapshot.svelte.ts src/App.svelte
git commit -m "feat: frontend snapshot store (invoke + event)"
```

---

### Task 11: PortCard + DeviceTree components

**Files:**
- Create: `src/lib/PortCard.svelte`, `src/lib/DeviceTree.svelte`

**Interfaces:**
- Consumes: `Port`, `PortVerdict`, `DeviceNode` types from the store
- Produces: `<PortCard port={port} verdict={verdict} onEngineer={fn} />`, `<DeviceTree nodes={devices} />`

- [ ] **Step 1: DeviceTree.svelte**

```svelte
<script lang="ts">
  import type { DeviceNode } from "./snapshot.svelte";
  import Self from "./DeviceTree.svelte";
  let { nodes }: { nodes: DeviceNode[] } = $props();
</script>

<ul class="tree">
  {#each nodes as n}
    <li>
      <span class="dev">{n.name}</span>
      <span class="speed">{n.speed.replaceAll("_", " ")}</span>
      {#if n.children.length}<Self nodes={n.children} />{/if}
    </li>
  {/each}
</ul>
```

- [ ] **Step 2: PortCard.svelte**

Render: headline (big), then the data verdict line with a colored dot keyed to `data_blame` (`none`→green, `cable`→amber, `port`/`device`→red), then `charging_line` if present, then each `trust_flags` entry as an amber warning row, then e-marker summary (`present ? "<vendor_name> · <max_speed> · <current_amps>A · <max_power_watts>W" : "No e-marker"`), then `<DeviceTree nodes={port.devices} />`, then a small "Engineer" button calling `onEngineer(port.id)`.

- [ ] **Step 3: Type-check**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/PortCard.svelte src/lib/DeviceTree.svelte
git commit -m "feat: PortCard + DeviceTree components"
```

---

### Task 12: App shell — wiring, refresh, engineer panel

**Files:**
- Modify: `src/App.svelte`
- Create: `src/lib/EngineerPanel.svelte`

**Interfaces:**
- Consumes: store `startPolling`, `refresh`, `store`; command `engineer_dump`
- Produces: the finished window.

- [ ] **Step 1: EngineerPanel.svelte**

```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  let { portId, onClose }: { portId: string; onClose: () => void } = $props();
  let rows = $state<Record<string,string>>({});
  $effect(() => { invoke("engineer_dump", { portId }).then((r) => (rows = r as any)); });
</script>
<div class="engineer">
  <button onclick={onClose}>close</button>
  <table>{#each Object.entries(rows) as [k, v]}<tr><td>{k}</td><td>{v}</td></tr>{/each}</table>
</div>
```

- [ ] **Step 2: App.svelte**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { store, startPolling, refresh } from "./lib/snapshot.svelte";
  import PortCard from "./lib/PortCard.svelte";
  import EngineerPanel from "./lib/EngineerPanel.svelte";

  let engineerPort = $state<string | null>(null);
  onMount(() => { const p = startPolling(); return () => { p.then((u) => u()); }; });

  function verdictFor(id: string) {
    return store.verdicts.find((v) => v.port_id === id);
  }
</script>

<main>
  <header>
    <h1>plugcheck</h1>
    <button onclick={refresh} disabled={store.loading}>Refresh</button>
  </header>

  {#if store.error}
    <p class="err">{store.error}</p>
  {:else if store.snapshot}
    {#each store.snapshot.ports as port}
      <PortCard {port} verdict={verdictFor(port.id)} onEngineer={(id) => (engineerPort = id)} />
    {/each}
  {:else}
    <p>Reading ports…</p>
  {/if}

  {#if engineerPort}
    <EngineerPanel portId={engineerPort} onClose={() => (engineerPort = null)} />
  {/if}
</main>
```

- [ ] **Step 3: Type-check + build the frontend**

Run:
```bash
npm run check
npm run build
```
Expected: both succeed.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte src/lib/EngineerPanel.svelte
git commit -m "feat: app shell — refresh, port list, engineer panel"
```

---

### Task 13: End-to-end verification + README + LICENSE

**Files:**
- Create: `README.md`, `LICENSE`
- Modify: `.gitignore` (ensure `target/`, `node_modules/`, `dist/` ignored — scaffold usually covers this)

**Interfaces:**
- Consumes: the whole app
- Produces: a documented, human-verified build.

- [ ] **Step 1: Full test run**

Run:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run check
```
Expected: all Rust tests pass (model 2, emarker 5, parse_macos 3, verdict 7, lib 2 = 19), svelte-check clean.

- [ ] **Step 2: Human smoke test**

A human runs `npm run tauri dev` on an Apple Silicon Mac, macOS 14+:
- Window opens, lists every USB-C port.
- Plug a known Thunderbolt cable + SSD → card shows "Thunderbolt 4" / full speed, no blame.
- Swap to a known USB 2.0 charge-only cable → card flips within ~3 s to "Charging only" or a cable-blame data line.
- Plug a charger → charging line shows watts.
- Click Engineer on a port → raw key/value table appears.
- Record a screenshot to `docs/screenshot.png`.

- [ ] **Step 3: Write README.md**

Cover: what it is (one paragraph), platform requirement (Apple Silicon, macOS 14+), `npm install` then `npm run tauri dev`, `npm run tauri build` for a bundle, where the spec + plan live, MIT license, the known-limitations list from the spec's "out of MVP" section.

- [ ] **Step 4: Add LICENSE (MIT)**

Standard MIT text, copyright holder = the repo owner, year 2026.

- [ ] **Step 5: Commit**

```bash
git add README.md LICENSE .gitignore docs/screenshot.png
git commit -m "docs: README, MIT license, verified MVP build"
```

---

## Self-Review

**1. Spec coverage:**

| Spec item | Task |
|---|---|
| Per-port cards | 11, 12 |
| Status headline (5 states) | 7 |
| Cable e-marker basics (speed/current/power/active-passive) | 4 (decode), 5 (populate), 11 (display) |
| Data-speed verdict + blame | 7 |
| Charging line + bottleneck sentence | 8 |
| Connected-device hierarchy | 6 (build), 11 (render) |
| Manual + 3s-poll refresh, emit on change | 9, 10 |
| Engineer mode raw dump | 5 (populate `raw`), 9 (`engineer_dump`), 12 (panel) |
| Trust flag: VID 0x0000 | 8 |
| `UsbProbe` trait boundary | 3 |
| macOS-only, `Unsupported` elsewhere | 3 (error), 5 & 9 (cfg) |
| ioreg + system_profiler data sources | 2 (capture/pin), 5 & 6 (parse) |
| Bit tables in emarker.rs w/ spec cites | 4 |
| IOKit keys in one const block | 2 (pin), 5 (use) |
| `ProbeError` variants + panel | 3, 9, 12 |
| 5s command timeout | 5 |
| Per-field degrade to null | 5 |
| verdict.rs pure + fixture table tests | 7, 8 |
| emarker decode tests vs PD spec hex | 4 |
| macos parse test vs real capture | 5, 6 |
| No frontend tests in MVP | (honored — 10/11/12 use `npm run check` only) |
| Poll interval single const | 9 |
| MIT license | 13 |

No gaps.

**2. Placeholder scan:** The `keys` const block in Task 5 is intentionally filled from Task 2's committed `docs/iokit-keys.md` — the plan states exactly where each value comes from and Task 2 produces it, so it is a real dependency, not a "TODO". All code steps carry runnable code. No "handle edge cases" hand-waves — degrade-to-null and timeout behavior are spelled out with the helper that implements them.

**3. Type consistency:** `Snapshot`/`Port`/`EmarkerInfo`/`DeviceNode`/`Charger`/`Transport`/`CableType`/`ProbeError` defined once in Task 3, consumed unchanged after. `PortVerdict`/`Blame` defined in Task 7, extended (not redefined) in Task 8, consumed by 9/10/11/12. `changed()` name consistent across Task 9. Frontend TS types in Task 10 mirror the Rust `serde` output (snake_case enums, `null` for `Option`). Lib crate name `plugcheck_lib` flagged in Task 5 Step 1 as "confirm exact name".

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-09-02-plugcheck-mvp.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
