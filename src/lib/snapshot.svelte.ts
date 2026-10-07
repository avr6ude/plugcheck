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
  head: string;
  text: string;
}
export interface PortVerdict {
  port_id: string;
  headline: string;
  subline: string;
  cards: VerdictCard[];
  cable_details: string[];
  trust_flags: string[];
  data_line: string;
  data_blame: Blame;
  charging_line: string | null;
}

export interface Fault {
  port_id: string;
  port: string;
  kind: "overcurrent" | "reconnect";
  title: string;
  text: string;
  at: number;
}
export interface Update {
  version: string;
  url: string;
}

export const store = $state<{
  faults: Fault[];
  update: Update | null;
  snapshot: Snapshot | null;
  verdicts: PortVerdict[];
  error: string | null;
  loading: boolean;
  version: string;
  power: { t: number; w: number }[];
}>({
  faults: [],
  update: null,
  snapshot: null,
  verdicts: [],
  error: null,
  loading: false,
  version: "",
  power: [],
});

function samplePower(s: Snapshot | null) {
  if (!s) return;
  const c = s.ports.map((p) => p.charger).find((c) => c);
  const w = c?.live_watts ?? c?.watts ?? 0;
  store.power.push({ t: Date.now(), w });
  if (store.power.length > 150) store.power.shift();
}

invoke<string>("app_version")
  .then((v) => (store.version = "v" + v))
  .catch(() => {});

export async function refresh(): Promise<void> {
  store.loading = true;
  try {
    store.snapshot = await invoke<Snapshot>("get_snapshot");
    store.verdicts = await invoke<PortVerdict[]>("get_verdicts");
    samplePower(store.snapshot);
    store.error = null;
  } catch (e) {
    store.error = String(e);
  } finally {
    store.loading = false;
  }
}

export async function startPolling(): Promise<UnlistenFn> {
  await refresh();
  invoke<Update | null>("get_update").then((u) => (store.update = u)).catch(() => {});
  const offs = await Promise.all([
    listen<Snapshot>("snapshot-changed", (ev) => {
      store.snapshot = ev.payload;
      samplePower(ev.payload);
      invoke<PortVerdict[]>("get_verdicts")
        .then((v) => (store.verdicts = v))
        .catch((e) => (store.error = String(e)));
    }),
    // Overcurrent / drop-and-reconnect seen by the backend between two scans; kept for this session.
    listen<Omit<Fault, "at">[]>("faults", (ev) => {
      store.faults = [...ev.payload.map((f) => ({ ...f, at: Date.now() })), ...store.faults].slice(0, 20);
    }),
    listen<Update>("update-available", (ev) => (store.update = ev.payload)),
  ]);
  return () => offs.forEach((off) => off());
}

export function dismissFault(f: Fault) {
  store.faults = store.faults.filter((x) => x !== f);
}

export function openRelease(url: string) {
  invoke("open_release", { url }).catch(() => {});
}

export function verdictFor(id: string): PortVerdict | undefined {
  return store.verdicts.find((v) => v.port_id === id);
}

export interface SavedCable {
  sig: string;
  name: string | null;
  count: number;
  first_seen: string;
  last_seen: string;
}
export async function savedCables(): Promise<SavedCable[]> {
  try {
    return await invoke<SavedCable[]>("saved_cables");
  } catch {
    return [];
  }
}
export async function forgetCable(sig: string): Promise<void> {
  try {
    await invoke("forget_cable", { sig });
  } catch {
    /* ignore */
  }
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
  show_technical: boolean;
  menu_bar_watts: boolean;
  update_checks: boolean;
}

export const settings = $state<Settings>({
  notifications: true,
  poll_secs: 3,
  launch_at_login: false,
  menu_bar_only: false,
  show_technical: false,
  menu_bar_watts: false,
  update_checks: true,
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
