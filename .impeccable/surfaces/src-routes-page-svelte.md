---
version: 1
slug: "src-routes-page-svelte"
primary_target: "src/routes/+page.svelte"
related_targets: ["src/lib/MacScene.svelte","src/lib/PortInspector.svelte","src/lib/Sidebar.svelte","src/lib/SettingsPanel.svelte"]
---

# Ports window (app shell + Ports surface)

Mode: Operate. A Mac owner plugs something in and wants to know, within seconds, what each port is doing and what limits it. Frequent short glances; occasional deep dives (devices, power, raw data).

Pinned by user (2026-10-06): native macOS look (canon path, played straight). Keep the 3D wireframe Mac with side toggle, and the sidebar navigation. Inspector panel and charcoal/salmon theme were rejected as slop.

Quality bar: System Settings, System Information, About This Mac.

## Direction contract

THESIS: plugcheck is an Apple utility, not a web dashboard. It refuses the dark card-dashboard (metric tiles, tinted alert cards, bordered pills, brand accent).

OWN-WORLD: system appearance (light and dark), translucent sidebar material, SF text at macOS sizes (13 body, 11 secondary), system label/secondary/separator colors, system accent for selection, system green/orange/red only as status glyphs. Grouped inset lists with hairline separators, no shadows, 10px group radius, native segmented control and switches.

STORY: the user sees the Mac from the side they care about, picks a port, and reads one plain sentence about it, then the facts as a grouped list.

FIRST VIEWPORT: sidebar (source list) left; unified toolbar with view title + subtitle beside traffic lights; content split: Mac model left on the window background, port detail right as a grouped-list column with headline sentence and status glyph at top, segmented tabs beneath.

FORM: canon (native macOS), user-pinned; no seed key, concept roll skipped by pinned direction.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
