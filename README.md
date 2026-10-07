# plugcheck

See what each USB-C / Thunderbolt cable and device on your Mac can actually
do — and, when a link is slow, which part is the bottleneck. Free, open
source, no telemetry. Inspired by [WhatCable](https://www.whatcable.uk/).

## Install

Download the `.dmg` from the
[latest release](https://github.com/avr6ude/plugcheck/releases/latest), open
it and drag **plugcheck** to Applications.

The app is not notarized by Apple, so macOS will refuse to open it the first
time. Either right-click the app → **Open**, or run:

```bash
xattr -dr com.apple.quarantine /Applications/plugcheck.app
```

## What it shows

A native macOS window with a 3D map of your MacBook's ports. Pick a side, pick
a port, and get:

- One plain sentence about what's connected and whether anything limits it
  (port · cable · device)
- Connection details: link speed, what the port supports, device, display,
  cable, e-marker, charger
- Connected-device tree, charger offers, and (optionally) raw IOKit data
- Power monitor, per-link negotiation, display modes, and named cable history
- Menu-bar icon and plug/unplug notifications

Follows the system light/dark appearance. Refreshes on a 3-second poll, on
plug events, and with ⌘R.

## Platform

**Apple Silicon, macOS 14+.** Reads `ioreg` + `system_profiler` — no
entitlements, no helper. (The window uses Tauri's `macos-private-api` only for
its translucent sidebar.) Intel Macs don't expose the
`AppleTCControllerType*` port-controller data this depends on. The probe
sits behind a `UsbProbe` trait so Linux / Windows back ends can be added
later; they are not implemented yet.

## Develop

```bash
npm install
npm run tauri dev      # launches the app window
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # backend logic + parser
npm run check                                     # svelte-check
npm run tauri build                               # bundle the .app and .dmg
```

## How it works

`src-tauri/src/`

| file | role |
|---|---|
| `probe/macos.rs` | run `ioreg` + `system_profiler`, parse into a `Snapshot` |
| `model.rs` | normalized `Snapshot` / `Port` / `EmarkerInfo` / … |
| `emarker.rs` | map Apple's decoded transport strings → `Transport`, vendor lookup |
| `verdict.rs` | pure: `Snapshot` → per-port plain-language verdict + blame |
| `lib.rs` | Tauri commands + the 3 s poll thread |

macOS pre-decodes USB-PD e-marker data, so there is **no VDO bit-parsing** —
`emarker.rs` works from `TransportsActive`, `ActiveCable`, and the
current-limit arrays. See `docs/iokit-keys.md` for the observed key schema
and `docs/2026-09-02-plugcheck-mvp-design.md` for the design.

## Known limits

No cable e-marker VDO detail (macOS doesn't expose it unprivileged), so cable
ratings are partly inferred. Not yet: USB-IF certificate database, CLI,
localisation, Intel Macs.

Field mapping for the e-marker / charger path was pinned against an
*empty-port* capture; it needs verification against a capture taken with a
charger + data cable connected (`./scripts/capture-fixtures.sh`).

## License

MIT — see [LICENSE](LICENSE).
