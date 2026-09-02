# plugcheck — MVP design

Date: 2026-09-02
Status: approved for spec review

## Goal

Desktop app that inspects USB-C / Thunderbolt ports on a Mac and says, in plain
language, what each connected cable + device can actually do and where the
bottleneck is. Free, open source. Feature target is WhatCable's free tier;
this doc is the MVP subset.

Reference implementation (MIT, for IOKit key names only):
https://github.com/darrylmorley/whatcable

## Platform

- Apple Silicon, macOS 14+ only. Intel Macs do not expose the IOKit
  port-controller classes this relies on.
- Cross-platform is a future concern. The probe boundary is a trait so
  `linux.rs` / `windows.rs` can slot in later. MVP ships macOS only — no stub
  files for the others yet.

## Stack

- Tauri 2, Rust backend.
- Svelte 5 frontend, plain single window (no menu-bar tray in MVP).
- No external services. All work is local.

## MVP feature set

1. **Per-port cards** — one card per USB-C / TB port on the machine.
2. **Status headline** per port: `Thunderbolt/USB4` · `USB device` · `Display` ·
   `Charging only` · `Empty`.
3. **Cable e-marker basics** (from SOP′ Discover Identity VDOs): speed
   capability, current rating (3A / 5A), power capacity (60–240W),
   active vs passive. Absent e-marker is shown as "no e-marker" not an error.
4. **Data-speed verdict**: active transport (USB 2 / USB 3.x / USB4 / TB / DP),
   plus a one-sentence bottleneck blame — `port` | `cable` | `device` | `none`.
5. **Charging line**: negotiated wattage + one bottleneck sentence
   (e.g. "Cable limits charging to 60 W"). No full PDO breakdown in MVP.
6. **Connected-device hierarchy**: devices nested under the port / hub they
   hang off.
7. **Refresh**: manual button + 3 s poll that diffs the snapshot and emits an
   event on change. IOKit change-notification is deferred.
   `ponytail: 3s poll; swap to IOKit IOServiceAddMatchingNotification if CPU shows up.`
8. **Engineer mode**: raw IOKit property dump per port, behind a toggle. Falls
   out of the probe for free and is the escape hatch when a key is missing.
9. **One trust flag**: e-marker vendor ID `0x0000` (unregistered with USB-IF)
   is flagged. No bundled cert DB in MVP.

### Explicitly out of MVP

USB-IF certificate database + full trust-signal set · charger PDO list
breakdown · display resolution / refresh / degraded-mode verdict ·
notifications and fault banners · CLI · settings UI (ship sane defaults) ·
menu-bar tray · cable history · localisation · Linux / Windows probes.

## Architecture

### Rust (`src-tauri/src/`)

```
probe/mod.rs    UsbProbe trait -> Snapshot { ports, devices, chargers, displays }
probe/macos.rs  the only impl: runs ioreg + system_profiler, parses, normalizes
model.rs        normalized serde structs, serialized to the frontend
verdict.rs      PURE. Snapshot -> Vec<PortVerdict>. platform-agnostic. unit-tested
emarker.rs      VDO bit decode + negotiated-PDO decode + USB-IF VID name lookup
lib.rs          tauri commands + poll/watch task
```

- `UsbProbe` trait: `fn snapshot(&self) -> Result<Snapshot, ProbeError>`.
- `Snapshot` and everything under it are plain data, `Serialize`.
- `verdict.rs` takes a `Snapshot` and returns verdicts. It never touches the
  OS. This is where all the "what's the bottleneck" logic lives and where the
  test weight goes.
- `emarker.rs` holds the bit-layout tables (VDO, PDO, speed enum -> label).
  Each table carries a comment citing the USB PD R3.1 section it came from.
  `ponytail: tables are USB PD R3.1; revisit for R3.2 / USB4 v2 cables.`

### Data sources (macos.rs)

`ioreg -a -l` (archived plist, parsed with the `plist` crate), scoped to:

| IOKit class / key                              | gives                                            |
|------------------------------------------------|-------------------------------------------------|
| `AppleHPMInterfaceType10/11/12`,               | per-port state, active transports, plug          |
| `AppleTCControllerType10/11/18`                | orientation, e-marker presence                   |
| `IOPortFeaturePowerSource`                      | negotiated PD profile (MVP: negotiated only)     |
| `IOPortTransportComponentCCUSBPDSOP*`           | Discover Identity VDOs — port, SOP′, SOP″        |
| XHCI subtree + `UsbIOPort`                      | device -> physical port linkage                  |

`system_profiler SPUSBDataType SPThunderboltDataType -json` (parsed with
`serde_json`): device tree, TB per-lane topology. `SPDisplaysDataType` is not
needed in MVP (no display verdict).

Key names are collected in one `const` block in `macos.rs`, annotated
"observed on macOS 26 / M-series". Availability varies by macOS point release
and SoC generation — an unknown or missing key degrades that one field to
`null` and still surfaces in Engineer mode; it never aborts the snapshot.

### Frontend (`src/`)

```
App.svelte        window shell, refresh button, engineer toggle
lib/PortCard.svelte    headline + e-marker rows + data verdict + charging line
lib/DeviceTree.svelte  recursive nested device list
lib/EngineerPanel.svelte  raw key/value dump
lib/snapshot.ts    store; invoke get_snapshot / get_verdicts; subscribe to
                   "snapshot-changed" event
```

### Tauri surface

- Commands: `get_snapshot() -> Snapshot`, `get_verdicts() -> Vec<PortVerdict>`,
  `engineer_dump(port_id) -> Map<String,Value>`.
- Event: `snapshot-changed` emitted by the poll task when the diff is non-empty.

## Data flow

1. Poll task (3 s) calls `MacosProbe::snapshot()`.
2. If the new `Snapshot` differs from the last, store it and emit
   `snapshot-changed`.
3. Frontend store re-invokes `get_snapshot` + `get_verdicts`, re-renders cards.
4. Manual refresh button calls the same commands directly.

## Error handling

- `ProbeError`: `CommandFailed`, `ParseFailed`, `Unsupported` (non-macOS /
  Intel / macOS < 14). `Unsupported` renders a single explanatory panel, not
  a crash.
- Per-field decode failures are swallowed to `null` + logged; the card renders
  with the missing field blank.
- `system_profiler` / `ioreg` timeout: 5 s, surfaced as `CommandFailed` with a
  retry button.

## Testing

- `verdict.rs` — table tests over fixture `Snapshot` JSON files
  (`src-tauri/tests/fixtures/`): good TB4 cable; USB 2 charge-only; e-marker
  claims 5 A but link is USB 2 (mismatch); no e-marker present; charging
  bottlenecked by cable; hub with nested devices. `assert_eq!` on verdict
  sentence + blame enum.
- `emarker.rs` — decode unit tests using VDO / PDO hex from the USB PD R3.1
  spec examples.
- `probe/macos.rs` — parse test against one real checked-in `ioreg` +
  `system_profiler` capture in `tests/fixtures/`; asserts normalized model
  fields. The live probe call is `#[cfg(target_os = "macos")]`; the parse
  test runs anywhere.
- Frontend tests: none in MVP. Playwright smoke is a later addition.

## Calibration knobs (things the physical world will not match on paper)

- e-marker VDO bit layouts, PDO decode, transport-speed enum -> label:
  one table each in `emarker.rs`, each commented with its USB PD R3.1 section.
- ioreg key-name `const` block in `macos.rs`, marked with the macOS / SoC it
  was observed on.
- Poll interval (3 s) is a single `const`.

## Open risks

1. The exact IOKit keys may differ on this machine's macOS build. First
   implementation task is to capture a real `ioreg` dump and pin the keys
   before writing the parser.
2. SOP″ (far-end e-marker) may be absent on many cables — treat as normal.
3. Tauri 2 tray/window defaults change between minor versions; pin the Tauri
   version in `Cargo.toml` and `package.json`.
