# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

Mac owners troubleshooting or identifying USB-C / Thunderbolt cables, chargers,
displays, docks, and connected devices.

## Product Purpose

Plugcheck is a local macOS utility that explains what each connected USB-C /
Thunderbolt port, cable, and device can actually do, and identifies the part of
the chain limiting speed or charging. Success means a user can understand the
current connection and next action without decoding raw hardware data.

## Positioning

The product reads the whole connection chain locally and turns port, cable,
charger, display, and device data into plain-language answers. It is a free,
open-source alternative in the same category as WhatCable.

## Operating Context

The app is a Tauri desktop window on Apple Silicon Macs running macOS 14+.
Users connect or disconnect hardware and expect the view to update automatically.
The primary workflow is a quick scan of each port, followed by optional detail
when a connection is limited or needs investigation.

## Capabilities and Constraints

- Per-port status and plain-language verdicts.
- Data-speed bottleneck attribution to port, cable, device, or none.
- Charging and USB-PD details where macOS exposes them.
- Cable e-marker identity and trust signals where available.
- Nested connected-device tree, display details, power monitoring, negotiation
  diagnostics, saved cable names/history, settings, and raw engineer data.
- Local-only operation with no external service dependency.
- Preserve existing behavior and data paths while redesigning the frontend.

## Brand Commitments

- Product name: PlugCheck (the terminal command stays `plugcheck`).
- Free, open source, no telemetry.
- The interface should prioritize ease of use and plain-language answers.
- Look and feel is native macOS (user decision, 2026-10-06): system appearance,
  sidebar material, SF type, system colors and controls. No custom brand theme.

## Evidence on Hand

- Existing source in `src/routes/+page.svelte` and `src/lib/*.svelte`.
- Product and platform details in `README.md` and
  `docs/2026-09-02-plugcheck-mvp-design.md`.
- Live snapshot and verdict models in `src/lib/snapshot.svelte.ts`.
- WhatCable's public guide and product site used as the category usability
  benchmark; no WhatCable assets or copy are to be copied.

## Product Principles

- Answer the user's question before exposing implementation detail.
- Make the limiting part of a connection obvious and actionable.
- Treat missing data as a normal state and explain it without alarm.
- Keep the app calm, local, fast, and trustworthy.

## Accessibility & Inclusion

Preserve semantic controls, keyboard access, visible focus, readable contrast,
and non-color status cues while redesigning the interface.
