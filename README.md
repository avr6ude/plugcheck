# plugcheck

See what each USB-C / Thunderbolt cable and device on your Mac can actually
do — and, when a link is slow, which part is the bottleneck. Free, open
source, no telemetry. Inspired by [WhatCable](https://www.whatcable.uk/).

## Status: MVP

Per-port cards showing:

- Headline: Thunderbolt / USB device / Display / Charging only / Empty
- Data-speed verdict with blame (port · cable · device)
- Charging line (negotiated watts, 3 A-cable cap)
- Cable e-marker summary and trust flags (e.g. VID 0x0000)
- Nested tree of connected devices
- Engineer view: raw IOKit properties per port

Refreshes on a 3-second poll and on demand.

## Platform

**Apple Silicon, macOS 14+.** Reads `ioreg` + `system_profiler` — no
entitlements, no helper, no private APIs. Intel Macs don't expose the
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
npm run tauri build                               # bundle a .app
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

## Known limits (MVP)

No negotiated volts/amps or cable e-marker VDO detail (macOS doesn't expose
them unprivileged) — charging watts and cable rating are partly inferred.
Not yet: USB-IF certificate database, charger PDO breakdown, display
resolution verdict, notifications, CLI, settings UI, menu-bar tray, cable
history, localisation.

Field mapping for the e-marker / charger path was pinned against an
*empty-port* capture; it needs verification against a capture taken with a
charger + data cable connected (`./scripts/capture-fixtures.sh`).

## License

MIT — see [LICENSE](LICENSE).
