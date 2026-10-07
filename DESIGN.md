---
name: plugcheck
description: A native macOS utility window that tells you, in one plain sentence, what each USB-C and Thunderbolt port is doing.
colors:
  accent: "#007aff"
  accent-dark: "#0a84ff"
  bg: "#f5f5f5"
  bg-dark: "#1e1e1e"
  group: "#ffffff"
  group-dark: "#282828"
  fg: "rgba(0, 0, 0, 0.85)"
  fg-dark: "rgba(255, 255, 255, 0.85)"
  muted: "rgba(0, 0, 0, 0.5)"
  muted-dark: "rgba(255, 255, 255, 0.55)"
  tertiary: "rgba(0, 0, 0, 0.26)"
  tertiary-dark: "rgba(255, 255, 255, 0.25)"
  line: "rgba(0, 0, 0, 0.1)"
  line-dark: "rgba(255, 255, 255, 0.1)"
  fill: "rgba(0, 0, 0, 0.05)"
  fill-dark: "rgba(255, 255, 255, 0.08)"
  selected: "rgba(0, 0, 0, 0.1)"
  selected-dark: "rgba(255, 255, 255, 0.1)"
  btn: "#ffffff"
  btn-dark: "rgba(255, 255, 255, 0.17)"
  ok: "#28cd41"
  ok-dark: "#32d74b"
  warn: "#ff9500"
  warn-dark: "#ff9f0a"
  bad: "#ff3b30"
  bad-dark: "#ff453a"
  sidebar-fallback: "#e9e9e9"
  sidebar-fallback-dark: "#2a2a2a"
  shell-hi: "#f6f7f8"
  shell-lo: "#b4b8bd"
  shell-hi-dark: "#6b6f74"
  shell-lo-dark: "#2b2d30"
  key: "#26272a"
  key-dark: "#0e0f10"
  pad: "#e6e8ea"
  pad-dark: "#3a3c40"
  hole: "#1c1d1f"
  hole-dark: "#050506"
typography:
  headline:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.25
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.2
  list-title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: 1.4
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
  chip-label:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "12px"
    fontWeight: 600
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.35
  figure:
    fontFamily: "-apple-system, BlinkMacSystemFont, Helvetica Neue, sans-serif"
    fontSize: "34px"
    fontWeight: 600
    letterSpacing: "-0.02em"
    fontFeature: "tnum"
  mono:
    fontFamily: "ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.35
rounded:
  menu-item: "4px"
  control: "5px"
  sidebar-item: "6px"
  segmented: "7px"
  chip: "8px"
  group: "10px"
  switch: "11px"
  glyph: "50%"
spacing:
  hairline: "0.5px"
  segment-gap: "2px"
  row-block: "7px"
  row-inset: "12px"
  sidebar-pad: "10px"
  group-gap: "16px"
  list-title-top: "20px"
  pane: "24px"
components:
  toolbar:
    textColor: "{colors.fg}"
    typography: "{typography.title}"
    height: "52px"
    padding: "0 20px"
  sidebar-item:
    textColor: "{colors.fg}"
    typography: "{typography.body}"
    rounded: "{rounded.sidebar-item}"
    height: "28px"
    padding: "0 8px"
  sidebar-item-active:
    backgroundColor: "{colors.selected}"
    rounded: "{rounded.sidebar-item}"
  list-title:
    textColor: "{colors.fg}"
    typography: "{typography.list-title}"
    padding: "20px 0 6px 12px"
  group:
    backgroundColor: "{colors.group}"
    rounded: "{rounded.group}"
    padding: "0 0 0 12px"
  group-row:
    textColor: "{colors.fg}"
    typography: "{typography.body}"
    height: "34px"
    padding: "7px 12px 7px 0"
  group-row-value:
    textColor: "{colors.muted}"
    typography: "{typography.body}"
  group-note:
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    padding: "6px 12px 0"
  button-push:
    backgroundColor: "{colors.btn}"
    textColor: "{colors.fg}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "22px"
    padding: "0 12px"
  segmented:
    backgroundColor: "{colors.fill}"
    rounded: "{rounded.segmented}"
    padding: "2px"
  segmented-item:
    textColor: "{colors.fg}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "22px"
  segmented-item-selected:
    backgroundColor: "{colors.btn}"
    rounded: "{rounded.control}"
  switch:
    backgroundColor: "{colors.selected}"
    rounded: "{rounded.switch}"
    width: "38px"
    height: "22px"
  switch-on:
    backgroundColor: "{colors.accent}"
    rounded: "{rounded.switch}"
  popup-button:
    backgroundColor: "{colors.btn}"
    textColor: "{colors.fg}"
    rounded: "{rounded.control}"
    height: "22px"
    padding: "0 6px 0 10px"
  popup-menu:
    backgroundColor: "{colors.group}"
    rounded: "{rounded.segmented}"
    padding: "5px"
  menu-item-highlighted:
    backgroundColor: "{colors.accent}"
    textColor: "#ffffff"
    rounded: "{rounded.menu-item}"
    height: "22px"
    padding: "0 10px"
  field:
    backgroundColor: "{colors.fill}"
    textColor: "{colors.fg}"
    rounded: "{rounded.control}"
    height: "22px"
    padding: "0 6px"
  port-chip:
    backgroundColor: "{colors.group}"
    textColor: "{colors.fg}"
    typography: "{typography.chip-label}"
    rounded: "{rounded.chip}"
    height: "40px"
    width: "120px"
    padding: "0 10px"
  port-chip-selected:
    backgroundColor: "{colors.accent}"
    textColor: "#ffffff"
    rounded: "{rounded.chip}"
  port-chip-selected-inactive:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.fg}"
    rounded: "{rounded.chip}"
  status-glyph:
    backgroundColor: "{colors.ok}"
    textColor: "#ffffff"
    rounded: "{rounded.glyph}"
    size: "24px"
---

# Design System: plugcheck

## Overview

**Creative North Star: "The Fourth System Utility"**

plugcheck is built to sit in the Utilities folder next to System Information and About This Mac and not look like a guest. It is an Apple utility, not a web dashboard: the window borrows the system's materials, type, colours and controls, and then gets out of the way so one plain sentence about a port can do the talking. Nothing in it is branded. The accent is whatever the user picked in System Settings, the sidebar is the window's own vibrancy, and both appearances are first-class.

The density is macOS density: 13 px text, 22 px controls, 34 px list rows, hairline separators. Every view is a source list on the left and a content pane on the right whose data lives in grouped inset lists, the System Settings pattern. The one illustrative object is a 3D wireframe MacBook drawn in silver aluminium (space grey in dark appearance) that turns to show the side you care about; ports are picked from popover-like chips beside it.

The world was chosen against a rejected predecessor, a dark charcoal theme with a salmon brand accent, metric tiles, tinted alert cards and bordered pills. None of those devices come back.

**Key Characteristics:**
- System appearance, light and dark, every token defined for both.
- Translucent sidebar material from the window itself, not painted.
- SF at macOS sizes: 13 body, 11 secondary, 17 for the one headline sentence.
- System accent for selection and focus; green, orange and red appear only as status glyphs.
- Grouped inset lists with hairline separators; flat surfaces, no drop shadows on containers.
- Native control imitations: push button, segmented control, switch, pop-up button.
- Selection goes grey when the window is inactive, as in AppKit.

## Colors

A colourless system palette: translucent black or white labels on near-neutral grounds, with one system accent and three system status hues held to glyph size.

### Primary
- **System Accent** (`accent` / `accent-dark`): selection and focus only. Fills the selected port chip, the on switch, the highlighted menu item; strokes the sidebar icons, the selected callout leader, the USB-C openings on the Mac model and the power chart line. Where the engine supports it the token is replaced at runtime by the CSS `AccentColor`, so it follows the user's System Settings choice; the hex values are the macOS default blue for each appearance. Text selection and the focus ring are this colour mixed to 30% and 55% over transparent.

### Tertiary
- **Status Green** (`ok` / `ok-dark`): the "working as expected" glyph, the "in use" dot on a port chip, and the live callout circle on the model.
- **Status Orange** (`warn` / `warn-dark`): the warning glyph, including the small inline mark that flags the limiting link in Negotiation and a degraded display.
- **Status Red** (`bad` / `bad-dark`): the problem glyph.

### Neutral
- **Window Ground** (`bg` / `bg-dark`): the content pane behind everything; the Mac model sits directly on it.
- **Group Ground** (`group` / `group-dark`): grouped lists, port chips, popup menus.
- **Label** (`fg` / `fg-dark`): primary text.
- **Secondary Label** (`muted` / `muted-dark`): values in list rows, subtitles, notes, descriptions under settings.
- **Tertiary Label** (`tertiary` / `tertiary-dark`): idle status glyph, idle dot, callout leader lines, chart ceiling.
- **Separator** (`line` / `line-dark`): every hairline: row dividers, group outline ring, toolbar and sidebar edges.
- **Control Fill** (`fill` / `fill-dark`): segmented-control track and text-field ground.
- **Selection Grey** (`selected` / `selected-dark`): active sidebar row and the inactive-window version of any accent selection.
- **Control Face** (`btn` / `btn-dark`): push button, pop-up button and selected segment.
- **Sidebar Fallback** (`sidebar-fallback` / `sidebar-fallback-dark`): painted only when running in a plain browser; inside the app the sidebar is transparent and the window's material shows.

### Mac Model
- **Aluminium highlight and shade** (`shell-hi`, `shell-lo`, dark variants): each shell facet is a mix of the two weighted by its angle to a fixed light, so the model shades itself.
- **Keycap** (`key`), **Trackpad** (`pad`), **Opening** (`hole`), with dark variants. The screen is a fixed near-black glass gradient and does not change with appearance.

### Named Rules
**The Borrowed Accent Rule.** There is no brand colour. The only chromatic colour that is not a status is the system accent, and it means selection or focus, nothing else.

**The Glyph-Sized Status Rule.** Green, orange and red appear only as the fill of a round status glyph or a 6 px dot. They never tint a card, a row, a background or a run of text.

**The Two Appearances Rule.** Every colour has a light and a dark value and the app follows `prefers-color-scheme`; nothing is designed for one appearance and inverted for the other.

## Typography

**Display Font:** none; the system face throughout
**Body Font:** SF Pro via `-apple-system` (with BlinkMacSystemFont, Helvetica Neue)
**Label/Mono Font:** SF Mono via `ui-monospace` (with Menlo), for raw data only

**Character:** The Mac's own voice at the Mac's own sizes. Hierarchy comes from weight and the label colour ramp, not from size jumps.

### Hierarchy
- **Headline** (600, 17px, 1.25): the one plain sentence about the selected port, and the empty or error state in a pane. One per view.
- **Title** (700, 15px, 1.2): the view name in the toolbar.
- **List Title** (600, 13px): the heading above each grouped list, sentence case.
- **Body** (400, 13px, 1.4): row labels, values, controls, sidebar items. Values use tabular figures.
- **Chip Label** (600, 12px): the port name inside a port chip.
- **Label** (400, 11px, 1.35): toolbar subtitle, notes under a group, setting descriptions, chip status, sidebar footer.
- **Figure** (600, 34px, -0.02em, tabular): the live wattage in Power Monitor, the only oversized number.
- **Mono** (400, 11px, 1.35): raw macOS keys in the Technical tab and cable signatures.

### Named Rules
**The Thirteen and Eleven Rule.** Running text is 13 px or 11 px. The only larger sizes are the headline sentence, the toolbar title and the live power figure.

**The No Shouting Rule.** No uppercase labels, no tracking, no small caps. Headings are sentence case and bolded, as in System Settings.

## Layout

The window is a two-column grid: a 200 px source list and a fluid content column. The content column is a 52 px toolbar (title, optional subtitle, hairline below) over a scrolling pane with 24 px gutters.

- **Ports view**: the pane splits 1.5 : 1 into the model column (left, on the bare window ground) and an inspector column of at least 320 px (right, scrolls on its own). Below 1100 px window width they stack: the model gets a fixed 260 px band and the inspector follows under a hairline, and the whole pane scrolls.
- **Every other view**: one centred column, max 640 px, the System Settings measure.
- **Grouped list rhythm**: list titles sit 20 px above their group and 6 px from it, indented 12 px to align with row text; rows are at least 34 px tall (44 px when they carry a description), with 12 px inset; notes under a group are 6 px below.
- **Window chrome**: the title bar is an overlay with the title hidden. Traffic lights are inset to (20, 28) so they sit in the sidebar's 52 px title zone, aligned with the toolbar band. The toolbar and the sidebar's title zone both drag the window.
- The window opens at 1180 × 800 and cannot shrink below 760 × 560.

**The Title Bar Is Not a Toolbar Rule.** The 52 px band holds the view title and subtitle only. Controls belong in the pane, because macOS keeps the clicks in that zone for dragging.

## Elevation & Depth

Depth comes from material and tone, not from shadow. The sidebar is the window's own `sidebar` vibrancy effect (following the window's active state), the content pane is an opaque window ground, and groups are a lighter (or, in dark, slightly lifted) ground outlined by a half-pixel hairline ring. Shadows exist only where macOS draws them on controls and floating things.

### Shadow Vocabulary
- **Hairline ring** (`box-shadow: 0 0 0 .5px var(--line)`): the outline of every group and port chip. It is a border, not a lift.
- **Control bezel** (`--btn-edge`; light `0 0 0 .5px rgba(0,0,0,.14), 0 1px 1.5px rgba(0,0,0,.1)`, dark `0 0 0 .5px rgba(0,0,0,.3), inset 0 .5px 0 rgba(255,255,255,.12)`): push button, pop-up button and selected segment.
- **Switch knob** (`0 1px 2px rgba(0,0,0,.3), 0 0 0 .5px rgba(0,0,0,.06)`): the white thumb of a switch.
- **Floating menu** (`0 0 0 .5px var(--line), 0 8px 24px rgba(0,0,0,.2)`): the pop-up button's menu, the one element that floats.

### Named Rules
**The Flat Container Rule.** Groups, chips and panes never cast a shadow. A drop shadow is reserved for things that physically sit above the window: an open menu and a switch knob.

## Shapes

Gentle, nested corners in AppKit proportions: the bigger the container, the rounder the corner, and nothing reaches pill shape except the switch and round glyphs. Groups are 10 px, port chips 8 px, the segmented track and menus 7 px, sidebar rows 6 px, controls and fields 5 px, menu items 4 px. Status glyphs and chip dots are full circles. Edges are hairlines (0.5 px rings, 1 px separators), never thick borders. Icons are 16 px outline drawings at a 1.4 stroke with round caps, tinted with the accent in the sidebar.

## Components

### Buttons
Quiet and small, the AppKit push button.
- **Shape:** gently rounded (5px), 22 px tall, 12 px side padding.
- **Primary:** control face on the control bezel, label in body text. Used for "Try Again" and "Forget".
- **Pressed:** brightness drops to 94%. There is no hover state, matching macOS.
- **Focus:** the global 3 px accent ring at 55% strength.

### Segmented Control
- **Style:** a 2 px-padded control-fill track (7px) holding 22 px segments (5px). The selected segment takes the control face and bezel; others are bare text.
- **Use:** the Overview / Devices / Power / Technical tabs in the inspector, and the Left / Right side toggle above the model (64 px fixed segments, labelled "Side").

### Switch
- **Style:** 38 × 22 track, fully rounded, selection-grey (dark: 18% white) with an inset hairline; 18 px white knob with its shadow.
- **On:** track fills with the accent and the knob slides 16 px. Both move over 160 ms ease and snap under reduced motion.

### Pop-up Button
- **Style:** control face and bezel, 22 px tall, label then a 7 × 11 double-chevron at 70% opacity.
- **Menu:** group ground, 7px corners, 5 px padding, floating-menu shadow; items are 22 px rows that fill with the accent and turn white on highlight.

### Inputs / Fields
- **Style:** borderless 22 px field on the control fill, 5px corners, 6 px padding. Inside a list row it right-aligns like a value and left-aligns while focused.
- **Focus:** the global accent ring.

### Cards / Containers: the Grouped List
The one container every view uses.
- **Corner Style:** 10px.
- **Background:** group ground with the hairline ring; no shadow.
- **Rows:** label left in body text, value right in secondary label, tabular figures, selectable as text. Rows are divided by a 1 px separator that starts at the 12 px inset.
- **Heading and note:** a list title above, an 11 px secondary note below when needed.
- **Variants:** findings rows lead with a 16 px status glyph and stack a 13 px head over an 11 px explanation; settings rows stack a label over an 11 px description and end in a control; the Technical list sets rows in mono.

### Navigation
- **Style:** a transparent source list over the window's sidebar material, 10 px side padding, its top 52 px left empty for the traffic lights.
- **Items:** 28 px rows, 16 px accent outline icon, 7 px gap, body text. The active row takes selection grey with 6px corners.
- **Groups:** five views at the top; Refresh Now and Settings pinned to the bottom; a footer with the app icon, name, version and device count in 11 px secondary text above a hairline.

### Status Glyph
A filled circle with a white check, exclamation, cross or dash: green, orange, red, or tertiary grey for idle. 24 px beside the headline sentence, 16 px in findings rows, 14 px inline beside a limiting value. It is the only place status colour appears at more than dot size.

### Mac Model and Port Chips
The signature object.
- **Model:** a perspective SVG of a 14-inch MacBook Pro seen 20 degrees from above, yawed 52 degrees toward the chosen side. Facets shade between the aluminium highlight and shade tokens; silhouette edges draw at 0.8 px in 42% black (white in dark). Tracked ports are outlined in the accent; other openings are plain holes. Changing sides turns the model over 650 ms with cubic in-out easing (instant under reduced motion), and the chips appear once the turn settles.
- **Port chips:** 120 px (104 px in narrow panes) by 40 px, group ground, 8px corners, hairline ring. A 12 px semibold port name over an 11 px status line with a 6 px dot, green when in use. A tertiary leader line runs to a 4.5 px circle on the opening.
- **Selected:** the chip fills with the accent and turns white, its leader and circle turn accent. When the window loses focus the selection drops to selection grey and the leader to secondary grey.

## Do's and Don'ts

### Do:
- **Do** take colour from the system: the `AccentColor` accent, the label alpha ramp, the separator, for both appearances.
- **Do** put every set of facts in a grouped list: list title, 10px group, label left, secondary value right.
- **Do** lead a view with one plain-sentence headline (17px semibold) and its status glyph, then the facts.
- **Do** keep text at 13 px and 11 px and build hierarchy with weight and label colour.
- **Do** imitate native controls at native size (22 px tall, 5px corners) and keep their native states, including the grey inactive-window selection.
- **Do** let values be selectable and use tabular figures for numbers.
- **Do** honour reduced motion: the model turn and switch transitions drop to instant.

### Don't:
- **Don't** bring back the card dashboard: no metric tiles, no tinted alert cards, no bordered pills, no brand accent.
- **Don't** return to the charcoal and salmon theme or paint a custom background behind the sidebar inside the app.
- **Don't** use green, orange or red as a background, border or text colour; status is a glyph.
- **Don't** cast a shadow from a group, chip or pane.
- **Don't** put controls in the 52 px title band.
- **Don't** use uppercase eyebrows, tracked labels or a custom display face.
