<script lang="ts">
  import type { Port } from "./snapshot.svelte";
  import { portLabel } from "./MacScene.svelte";
  import { store } from "./snapshot.svelte";

  const occ = $derived((store.snapshot?.ports ?? []).filter((p) => p.occupied));

  const ORDER = ["none", "usb2", "usb3_gen1", "usb3_gen2", "usb4_gen3", "thunderbolt3", "usb4_gen4", "thunderbolt4"];
  function name(s: string): string {
    return (
      {
        none: "—",
        usb2: "USB 2.0",
        usb3_gen1: "USB 3.2 Gen 1 (5 Gb/s)",
        usb3_gen2: "USB 3.2 Gen 2 (10 Gb/s)",
        usb4_gen3: "USB4 Gen 3 (20 Gb/s)",
        usb4_gen4: "USB4 (40 Gb/s)",
        thunderbolt3: "Thunderbolt 3",
        thunderbolt4: "Thunderbolt 4",
        displayport: "DisplayPort",
      }[s] ?? s
    );
  }
  function fastestDevice(p: Port): string {
    let best = "none";
    const walk = (ns: Port["devices"]) => {
      for (const d of ns) {
        if (ORDER.indexOf(d.speed) > ORDER.indexOf(best)) best = d.speed;
        walk(d.children);
      }
    };
    walk(p.devices);
    return best;
  }
  function weakLink(p: Port): "cable" | "port" | "device" | null {
    const dev = fastestDevice(p);
    const active = p.active_transport;
    if (ORDER.indexOf(active) >= ORDER.indexOf(dev)) return null; // running at device max
    // active below device — who's holding it back?
    if (!p.emarker.present && active === "usb2") return "cable";
    if (p.emarker.present && ORDER.indexOf(p.emarker.max_speed) <= ORDER.indexOf(active))
      return "cable";
    return "port";
  }
</script>

{#if occ.length === 0}
  <p class="empty-note">Nothing is connected. Plug in a cable to see what each part of the chain negotiated.</p>
{/if}
{#each occ as p}
  {@const dev = fastestDevice(p)}
  {@const weak = weakLink(p)}
  <h3 class="list-title">{portLabel(p)}</h3>
  <dl class="group">
    <div class="row"><dt>Mac port</dt><dd>{#if weak === "port"}<svg class="warn-mark" viewBox="0 0 20 20" width="14" height="14" aria-hidden="true"><circle cx="10" cy="10" r="10" /><path d="M10 5.6v5.3M10 14.3v.1" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" /></svg>{/if}{p.supported.join(", ") || "Unknown"}</dd></div>
    <div class="row"><dt>Cable</dt><dd>{#if weak === "cable"}<svg class="warn-mark" viewBox="0 0 20 20" width="14" height="14" aria-hidden="true"><circle cx="10" cy="10" r="10" /><path d="M10 5.6v5.3M10 14.3v.1" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" /></svg>{/if}{p.emarker.present ? name(p.emarker.max_speed) : "Unknown (no e-marker)"}</dd></div>
    <div class="row"><dt>Device</dt><dd>{#if weak === "device"}<svg class="warn-mark" viewBox="0 0 20 20" width="14" height="14" aria-hidden="true"><circle cx="10" cy="10" r="10" /><path d="M10 5.6v5.3M10 14.3v.1" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" /></svg>{/if}{dev !== "none" ? name(dev) : "Unknown"}</dd></div>
    <div class="row"><dt><b>Negotiated</b></dt><dd><b>{p.provisioned.join(", ") || "Nothing yet"}</b></dd></div>
  </dl>
  <p class="group-note">
    {#if weak === "cable"}The cable is the slowest part of this chain.
    {:else if weak === "port"}The Mac port is negotiating below what it can do.
    {:else}Running at the device’s maximum; the cable and port aren’t limiting it.{/if}
  </p>
{/each}

<style>
  b { color: var(--fg); font-weight: 600; }
  .warn-mark { flex: none; vertical-align: -2px; margin-right: 5px; } .warn-mark circle { fill: var(--warn); }
</style>
