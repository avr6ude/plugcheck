import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Emarker {
  vendor_id: number | null;
  vendor_name: string | null;
  cable_type: string;
  max_speed: string;
  current_amps: number | null;
  max_power_watts: number | null;
  present: boolean;
}
export interface DeviceNode {
  name: string;
  vendor: string | null;
  speed: string;
  usb_version: string | null;
  class: string | null;
  vid_pid: string | null;
  serial: string | null;
  is_hub: boolean;
  children: DeviceNode[];
}
export interface Pdo {
  volts: number;
  amps: number;
  watts: number;
}
export interface Charger {
  negotiated_volts: number | null;
  negotiated_amps: number | null;
  watts: number | null;
  is_charging: boolean;
  fully_charged: boolean;
  battery_percent: number | null;
  minutes_to_full: number | null;
  live_watts: number | null;
  profile_volts: number[];
  pdos: Pdo[];
  cable_current_limit_amps: number | null;
}
export interface Port {
  id: string;
  kind: string;
  occupied: boolean;
  orientation: number | null;
  active_transport: string;
  supported: string[];
  provisioned: string[];
  cable_kind: string;
  connection_count: number | null;
  plug_events: number | null;
  overcurrent_count: number | null;
  hpd: boolean;
  dp_alt: boolean;
  history_sig: string | null;
  history: {
    name: string | null;
    count: number;
    first_seen: string;
    last_seen: string;
  } | null;
  display: {
    name: string;
    pixels: string | null;
    native_pixels: string | null;
    hz: number | null;
    degraded: boolean;
    connection: string | null;
    depth: string | null;
    hdr: boolean;
    mirrored: boolean;
    main: boolean;
  } | null;
  emarker: Emarker;
  charger: Charger | null;
  devices: DeviceNode[];
  raw: Record<string, string>;
}
export interface Snapshot {
  ports: Port[];
  captured_ms: number;
}
export type Blame = "port" | "cable" | "device" | "none";
export type CardKind = "port" | "data" | "charging" | "display" | "cable";
export type CardStatus = "ok" | "warn" | "bad" | "idle";
export interface VerdictCard {
  kind: CardKind;
  status: CardStatus;
  title: string;
  text: string;
  rows: [string, string][];
  chip: string;
}
export interface PortVerdict {
  port_id: string;
  headline: string;
  cards: VerdictCard[];
  trust_flags: string[];
  data_line: string;
  data_blame: Blame;
  charging_line: string | null;
}

export const store = $state<{
  snapshot: Snapshot | null;
  verdicts: PortVerdict[];
  error: string | null;
  loading: boolean;
}>({ snapshot: null, verdicts: [], error: null, loading: false });

export async function refresh(): Promise<void> {
  store.loading = true;
  try {
    store.snapshot = await invoke<Snapshot>("get_snapshot");
    store.verdicts = await invoke<PortVerdict[]>("get_verdicts");
    store.error = null;
  } catch (e) {
    store.error = String(e);
  } finally {
    store.loading = false;
  }
}

export async function startPolling(): Promise<UnlistenFn> {
  await refresh();
  return listen<Snapshot>("snapshot-changed", (ev) => {
    store.snapshot = ev.payload;
    invoke<PortVerdict[]>("get_verdicts")
      .then((v) => (store.verdicts = v))
      .catch((e) => (store.error = String(e)));
  });
}

export function verdictFor(id: string): PortVerdict | undefined {
  return store.verdicts.find((v) => v.port_id === id);
}

export async function renameCable(sig: string, name: string | null): Promise<void> {
  try {
    await invoke("rename_cable", { sig, name });
    await refresh();
  } catch (e) {
    store.error = String(e);
  }
}

export interface Settings {
  notifications: boolean;
  poll_secs: number;
  launch_at_login: boolean;
  menu_bar_only: boolean;
  hide_empty: boolean;
}

export const settings = $state<Settings>({
  notifications: true,
  poll_secs: 3,
  launch_at_login: false,
  menu_bar_only: false,
  hide_empty: false,
});

export async function loadSettings(): Promise<void> {
  try {
    Object.assign(settings, await invoke<Settings>("get_settings"));
  } catch {
    /* keep defaults */
  }
}

export async function saveSettings(): Promise<void> {
  try {
    await invoke("set_settings", { next: { ...settings } });
  } catch (e) {
    store.error = String(e);
  }
}
