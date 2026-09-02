# Observed IOKit keys

Captured 2026-09-02 on **MacBook Pro, macOS 26.6.2 (25G83), Apple Silicon**
via `scripts/capture-fixtures.sh` (`ioreg -a -r -l -c AppleHPMDevice`).

**State at capture: nothing connected.** All ports show `ConnectionActive =
false`, `TransportsActive = []`. This is the empty-baseline fixture. The
e-marker / charger keys below are marked *(needs cable)* — their shape is
inferred and must be confirmed against an "occupied" capture (charger + data
cable/dock connected).

## Port controller nodes

Class `AppleTCControllerType10` (USB-C) / `AppleTCControllerType11` (MagSafe 3)
/ `AppleTCControllerType12` (seen on other models). Found as descendants of
`AppleHPMDevice`. One node per physical port.

| Concept | Key | Value shape | Confirmed |
|---|---|---|---|
| Port kind | `PortTypeDescription` | str: `"USB-C"`, `"MagSafe 3"` | yes |
| Port kind (enum) | `PortType` | int: 2 = USB-C, 17 = MagSafe 3 | yes |
| Stable id | `PortDescription` | str: `"Port-USB-C@1"` | yes |
| Port index | `PortNumber` | int, 1-based within a `PortTypeDescription` | yes |
| Occupied | `ConnectionActive` | bool | yes (false when empty) |
| Plug orientation | `PlugOrientation` | int (0 when empty) | yes |
| Active vs passive cable | `ActiveCable` | bool | yes (false when empty) |
| Optical cable | `OpticalCable` | bool | yes |
| Negotiated transports | `TransportsActive` | array<str>, subset of `["CC","USB2","USB3","CIO","DisplayPort","TBT"]`. `CIO` = Thunderbolt/USB4 tunnel. `[]` when empty | yes (empty) |
| Port capability | `TransportsSupported` | array<str>, e.g. `["CC","USB2","USB3","CIO","DisplayPort"]` | yes |
| SuperSpeed link up | `IOAccessoryUSBSuperSpeedActive` | bool | yes |
| Connection type | `IOAccessoryUSBConnectString` | str: `"None"` when empty | yes |
| Power current limits | `IOAccessoryPowerCurrentLimits` | array<int> len 5 (mA, all 0 when empty) *(needs cable)* | shape only |
| Display hot-plug | `HPDAsserted` | bool | yes |
| DP pin assignment | `DisplayPortPinAssignment` | bool or int | yes |
| Built-in port | `BuiltIn` | bool (true for all Mac ports) | yes |
| Registry location | `IORegistryEntryLocation` | str: `"1"`,`"2"`,`"3"` | yes |

## E-marker / cable identity *(needs cable)*

Not present on any node while nothing is connected. The `AppleHPMDevice`
class schema (from `ioreg` key census) exposes, on the accessory/cable node
that appears when an e-marked cable is attached:

| Concept | Key (expected) | Notes |
|---|---|---|
| Cable vendor id | `Vendor ID` | int; look up name in `assets/usbif_vendors.csv` |
| Cable product id | `USB PID` | hex string, e.g. `"73080000"` (this value is the Mac's own port PID when empty — ignore until `ConnectionActive`) |
| Cable revision | `Revision` / `Version` | int |
| Current rating | `IOAccessoryPowerCurrentLimits` | array<int> mA; max element ≈ cable/charger current cap |

**No raw USB-PD VDO blobs are exposed.** Apple pre-decodes into
`ActiveCable`, `TransportsActive`, `IOAccessoryPowerCurrentLimits`. So
plugcheck does **not** need VDO bit-parsing — `emarker.rs` is reduced to
string/array mapping + a vendor-id lookup + a current→watts helper.

## Charger / Power Delivery *(needs cable)*

- `IOPortFeaturePowerSource` / `IOPDPowerSource`: **not present** on this
  machine (0 nodes) even on battery. Charger PD detail likely surfaces under
  `AppleSmartBattery` → `AdapterDetails` (`Watts`, `Voltage`, `Amperage`,
  `Description`) when a charger is attached, and/or
  `IOAccessoryPowerCurrentLimits` on the charging port node.
- `AppleSmartBattery`: 1 node. `IOPMPowerSource`: 1 node.
- MVP charging line reads `AdapterDetails.Watts` if present, else
  `max(IOAccessoryPowerCurrentLimits) * negotiated_voltage`, else
  "wattage unavailable".

## Device tree

- `system_profiler -json SPUSBDataType`: **always an empty array on this
  hardware**, even with a hub + SSD + LAN + monitor connected. Do not use it.
- **Use `ioreg -a -l -p IOUSB`.** Roots are `AppleT6000USBXHCI` controllers;
  under them `IOUSBHostDevice` nodes with:
  - `USB Product Name` / `kUSBProductString`, `USB Vendor Name` /
    `kUSBVendorString`
  - `UsbLinkSpeed` — bits/second, exact (e.g. `5000000000`). Preferred.
  - `Device Speed` — enum fallback (0 low, 1 full, 2 high, 3 super, 4 super+,
    5 super+ x2)
  - `bDeviceClass` — `9` = hub
  - `locationID` — int; `(loc >> 24) & 0xFF` = XHCI bus id. Children nest via
    `IORegistryEntryChildren`.
- **Port ↔ device linkage**: there is *no* shared key between
  `AppleTCControllerType*` and the XHCI buses. plugcheck groups top-level USB
  devices by bus id, then assigns each group to an occupied port by matching
  `IOAccessoryUSBSuperSpeedActive`, falling back to sorted zip.
- `SPThunderboltDataType`: one entry per TB bus; `receptacle_N_tag` has
  `current_speed_key`, `receptacle_status_key`, `receptacle_id_key` (==
  `PortNumber`). Only populated for real Thunderbolt devices — a USB-C dock
  shows up under IOUSB, not here.

## DisplayPort Alt Mode

Port node key `TransportsActive` includes the string `"DisplayPort"` when the
port is carrying video (confirmed: DP-to-USB-C cable →
`["CC","USB2","DisplayPort"]`, `HPDAsserted = true`,
`DisplayPortPinAssignment = true`). `Port.dp_alt` is set from that string.

## Not found on this machine

`AppleHPMInterfaceType10/11/12` exist but as single singleton nodes, not
per-port — the per-port data is on `AppleTCControllerType1x`.
`IOPortTransportComponentCCUSBPDSOP*` — **not present**. `IOPortFeaturePowerSource`
— not present.
